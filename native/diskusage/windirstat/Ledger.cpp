// altWinDirStat - folder ledger export and baseline comparison (fork-owned; not part of upstream WinDirStat)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// at your option any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//

#include "pch.h"
#include "Ledger.h"
#include "Item.h"
#include "Options.h"

#include <deque>

namespace
{
    // Fixed English header: ledgers stay comparable whatever UI language wrote them.
    constexpr std::wstring_view ledgerHeader = L"Path,Relative Path,Size (bytes),Files,Subfolders";
    constexpr std::wstring_view ledgerSuffix = L".ledger.csv";
    constexpr std::wstring_view rootRelative = L".";
    constexpr std::wstring_view filtersPrefix = L"#filters=";

    std::wstring Key(std::wstring_view text)
    {
        std::wstring key(text);
        if (!key.empty()) CharLowerBuffW(key.data(), static_cast<DWORD>(key.size()));
        return key;
    }

    std::wstring TrimSlashes(std::wstring path)
    {
        while (!path.empty() && path.back() == L'\\') path.pop_back();
        return path;
    }

    // Relative path of 'path' below 'root' (both absolute), "." for the root, or nullopt if outside.
    std::optional<std::wstring> RelativeTo(const std::wstring& root, const std::wstring& path)
    {
        if (root.empty()) return TrimSlashes(path);
        const std::wstring base = TrimSlashes(root);
        const std::wstring full = TrimSlashes(path);
        if (_wcsicmp(base.c_str(), full.c_str()) == 0) return std::wstring(rootRelative);
        if (full.size() <= base.size() + 1 || _wcsnicmp(base.c_str(), full.c_str(), base.size()) != 0 ||
            full[base.size()] != L'\\') return std::nullopt;
        return full.substr(base.size() + 1);
    }

    bool IsSameOrBelow(const std::wstring& ancestor, const std::wstring& path)
    {
        return !ancestor.empty() && !path.empty() && RelativeTo(ancestor, path).has_value();
    }

    // Relative path of the parent row: "." for top-level folders, empty for the root itself.
    std::wstring ParentRelative(const std::wstring& relative)
    {
        if (relative == rootRelative) return {};
        const size_t slash = relative.find_last_of(L'\\');
        return slash == std::wstring::npos ? std::wstring(rootRelative) : relative.substr(0, slash);
    }

    void SortRows(std::vector<Ledger::Row>& rows)
    {
        std::ranges::sort(rows, [](const Ledger::Row& a, const Ledger::Row& b)
        {
            if (a.relative == rootRelative || b.relative == rootRelative) return a.relative == rootRelative && b.relative != rootRelative;
            return _wcsicmp(a.relative.c_str(), b.relative.c_str()) < 0;
        });
    }

    std::string ToUtf8(const std::wstring_view text)
    {
        if (text.empty()) return {};
        const int size = WideCharToMultiByte(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), nullptr, 0, nullptr, nullptr);
        std::string out(static_cast<size_t>(size), '\0');
        WideCharToMultiByte(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), out.data(), size, nullptr, nullptr);
        return out;
    }

    std::wstring FromUtf8(const std::string_view text)
    {
        if (text.empty()) return {};
        const int size = MultiByteToWideChar(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), nullptr, 0);
        std::wstring out(static_cast<size_t>(size), L'\0');
        MultiByteToWideChar(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), out.data(), size);
        return out;
    }

    std::wstring Quote(const std::wstring_view text)
    {
        std::wstring out = L"\"";
        for (const wchar_t c : text)
        {
            if (c == L'"') out += L'"';
            out += c;
        }
        return out + L'"';
    }

    // Splits one CSV line; handles quoted fields and doubled quotes.
    std::vector<std::wstring> SplitCsv(const std::wstring_view line)
    {
        std::vector<std::wstring> fields(1);
        bool quoted = false;
        for (size_t i = 0; i < line.size(); ++i)
        {
            const wchar_t c = line[i];
            if (quoted)
            {
                if (c == L'"' && i + 1 < line.size() && line[i + 1] == L'"') { fields.back() += L'"'; ++i; }
                else if (c == L'"') quoted = false;
                else fields.back() += c;
            }
            else if (c == L'"') quoted = true;
            else if (c == L',') fields.emplace_back();
            else fields.back() += c;
        }
        return fields;
    }

    bool ParseNumber(const std::wstring& text, ULONGLONG& value)
    {
        if (text.empty() || !std::ranges::all_of(text, [](const wchar_t c) { return c >= L'0' && c <= L'9'; })) return false;
        try { value = std::stoull(text); }
        catch (const std::out_of_range&) { return false; }
        return true;
    }

    // Writes "<path>.tmp" and renames it over 'path', so a crash or full disk never leaves a
    // half-written file under the real name.
    bool WriteFile(const std::wstring& path, const std::wstring& text)
    {
        const std::wstring temp = path + L".tmp";
        {
            std::ofstream out(std::filesystem::path(temp), std::ios::binary | std::ios::trunc);
            if (!out.is_open()) return false;
            out << "\xEF\xBB\xBF" << ToUtf8(text); // BOM so Excel opens non-ASCII paths correctly
            out.flush();
            if (!out.good())
            {
                out.close();
                DeleteFileW(temp.c_str());
                return false;
            }
        }
        if (MoveFileExW(temp.c_str(), path.c_str(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)) return true;
        DeleteFileW(temp.c_str());
        return false;
    }

    // Keeps only rows at or below 'root' and re-expresses them relative to it.
    Ledger::Snapshot Rebase(const Ledger::Snapshot& snapshot, const std::wstring& root)
    {
        Ledger::Snapshot rebased{ .source = snapshot.source, .root = root, .filters = snapshot.filters };
        for (const auto& row : snapshot.rows)
        {
            if (auto relative = RelativeTo(root, row.path); relative)
                rebased.rows.push_back({ .path = row.path, .relative = std::move(*relative),
                    .size = row.size, .files = row.files, .folders = row.folders });
        }
        SortRows(rebased.rows);
        return rebased;
    }
}

bool Ledger::IsLedgerPath(const std::wstring& path)
{
    return path.size() > ledgerSuffix.size() &&
        _wcsicmp(path.c_str() + path.size() - ledgerSuffix.size(), ledgerSuffix.data()) == 0;
}

std::wstring Ledger::FilterFingerprint()
{
    const std::wstring text = std::format(L"{}{}{}{}{}{}{}{}{}{}{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        COptions::ExcludeJunctions.Obj(), COptions::ExcludeSymbolicLinksDirectory.Obj(),
        COptions::ExcludeVolumeMountPoints.Obj(), COptions::ExcludeHiddenDirectory.Obj(),
        COptions::ExcludeProtectedDirectory.Obj(), COptions::ExcludeDropboxIgnored.Obj(),
        COptions::ExcludeSymbolicLinksFile.Obj(), COptions::ExcludeHiddenFile.Obj(),
        COptions::ExcludeProtectedFile.Obj(), COptions::FilteringUseRegex.Obj(), COptions::FollowVolumeMountPoints.Obj(),
        COptions::FilteringSizeMinimum.Obj(), COptions::FilteringSizeUnits.Obj(), COptions::FilteringSizeComparison.Obj(),
        COptions::FilteringMaxAgeDays.Obj(), COptions::FilteringMaxAgeComparison.Obj(),
        COptions::FilteringExcludeDirs.Obj(), COptions::FilteringExcludeFiles.Obj(),
        COptions::FilteringIncludeDirs.Obj(), COptions::FilteringIncludeFiles.Obj());

    std::uint64_t hash = 14695981039346656037ull;
    for (const wchar_t c : text)
    {
        hash ^= static_cast<std::uint64_t>(c);
        hash *= 1099511628211ull;
    }
    return std::format(L"{:016x}", hash);
}

bool Ledger::FiltersDiffer(const Snapshot& baseline, const Snapshot& current)
{
    return !baseline.filters.empty() && !current.filters.empty() && baseline.filters != current.filters;
}

Ledger::Snapshot Ledger::FromScan(const CItem* root)
{
    Snapshot snapshot;
    if (root == nullptr) return snapshot;
    snapshot.filters = FilterFingerprint();

    // A multi-drive scan has a synthetic "My Computer" root: keep absolute paths as the key.
    const bool multiRoot = root->GetItemType() == IT_MYCOMPUTER;
    snapshot.root = multiRoot ? std::wstring() : root->GetPath();

    for (std::vector stack{ root }; !stack.empty();)
    {
        const CItem* item = stack.back();
        stack.pop_back();

        const ITEMTYPE type = item->GetItemType();
        if (type != IT_MYCOMPUTER && type != IT_DRIVE && type != IT_DIRECTORY) continue;

        Row row{ .size = item->GetSizeLogical(), .files = item->GetFilesCount(), .folders = item->GetFoldersCount() };
        if (item == root)
        {
            row.path = multiRoot ? item->GetName() : item->GetPath();
            row.relative = rootRelative;
        }
        else
        {
            row.path = item->GetPath();
            row.relative = RelativeTo(snapshot.root, row.path).value_or(TrimSlashes(row.path));
        }
        snapshot.rows.push_back(std::move(row));

        if (item->HasChildren())
            for (const CItem* child : item->GetChildren()) stack.push_back(child);
    }

    SortRows(snapshot.rows);
    return snapshot;
}

bool Ledger::Save(const std::wstring& path, const Snapshot& snapshot)
{
    std::wstring text(ledgerHeader);
    for (const auto& row : snapshot.rows)
        text += std::format(L"\r\n{},{},{},{},{}", Quote(row.path), Quote(row.relative), row.size, row.files, row.folders);
    // Last, so CSV readers that take the first line as the header still see the normal columns.
    if (!snapshot.filters.empty()) text += std::format(L"\r\n{}{}", filtersPrefix, snapshot.filters);
    return WriteFile(path, text);
}

std::optional<Ledger::Snapshot> Ledger::Load(const std::wstring& path, std::wstring& error)
{
    std::ifstream in(std::filesystem::path(path), std::ios::binary);
    if (!in.is_open())
    {
        error = TranslateError();
        return std::nullopt;
    }
    std::string bytes((std::istreambuf_iterator(in)), std::istreambuf_iterator<char>());
    if (bytes.starts_with("\xEF\xBB\xBF")) bytes.erase(0, 3);
    const std::wstring text = FromUtf8(bytes);

    Snapshot snapshot{ .source = path };
    std::wistringstream lines(text);
    std::wstring line;
    bool sawHeader = false;
    size_t lineNumber = 0;
    while (std::getline(lines, line))
    {
        ++lineNumber;
        if (!line.empty() && line.back() == L'\r') line.pop_back();
        if (line.empty()) continue;
        if (!sawHeader)
        {
            if (line != ledgerHeader)
            {
                error = L"Unrecognized header (expected \"" + std::wstring(ledgerHeader) + L"\")";
                return std::nullopt;
            }
            sawHeader = true;
            continue;
        }
        if (line.starts_with(L'#'))  // Metadata line; unknown ones are ignored for forward compatibility
        {
            if (line.starts_with(filtersPrefix)) snapshot.filters = line.substr(filtersPrefix.size());
            continue;
        }

        auto fields = SplitCsv(line);
        Row row;
        if (fields.size() != 5 || fields[1].empty() || !ParseNumber(fields[2], row.size) ||
            !ParseNumber(fields[3], row.files) || !ParseNumber(fields[4], row.folders))
        {
            error = std::format(L"Malformed row on line {}", lineNumber);
            return std::nullopt;
        }
        row.path = std::move(fields[0]);
        row.relative = std::move(fields[1]);
        if (row.relative == rootRelative) snapshot.root = row.path;
        snapshot.rows.push_back(std::move(row));
    }

    if (!sawHeader)
    {
        error = L"The file is empty";
        return std::nullopt;
    }
    // Every ledger this app writes has exactly one "." row; without it the root size change and the
    // narrowing in Compare are wrong, and a second one means the file was edited or concatenated.
    if (const auto roots = std::ranges::count(snapshot.rows, std::wstring_view(rootRelative), &Row::relative); roots != 1)
    {
        error = roots == 0 ? L"No root row (Relative Path \".\")" : L"More than one root row (Relative Path \".\")";
        return std::nullopt;
    }
    SortRows(snapshot.rows);
    return snapshot;
}

std::vector<Ledger::DiffRow> Ledger::Compare(Snapshot& baseline, Snapshot& current, Summary& summary)
{
    // If one side scanned a parent of the other's root (e.g. baseline of C:\Data, current scan of C:\),
    // narrow the broader side to the common root so folders line up. Different, unrelated roots
    // (e.g. D:\Data vs E:\Backup\Data) are matched by relative path, which compares copies.
    if (!baseline.root.empty() && !current.root.empty() &&
        _wcsicmp(TrimSlashes(baseline.root).c_str(), TrimSlashes(current.root).c_str()) != 0)
    {
        if (IsSameOrBelow(current.root, baseline.root)) current = Rebase(current, baseline.root);
        else if (IsSameOrBelow(baseline.root, current.root)) baseline = Rebase(baseline, current.root);
    }

    // Match rows by exact relative path first, then case-insensitively among the rows left over. On
    // case-sensitive folders "Data" and "data" are two folders and must not collapse into one, but a
    // case-only rename still lines up.
    std::unordered_map<std::wstring, size_t> currentByRelative;
    currentByRelative.reserve(current.rows.size());
    for (size_t i = 0; i < current.rows.size(); ++i) currentByRelative.emplace(current.rows[i].relative, i);
    std::vector<bool> matched(current.rows.size());

    std::vector<DiffRow> diff;
    std::vector<size_t> unmatched;
    diff.reserve(std::max(baseline.rows.size(), current.rows.size()));
    for (const auto& before : baseline.rows)
    {
        DiffRow row{ .relative = before.relative, .before = &before };
        if (const auto it = currentByRelative.find(before.relative); it != currentByRelative.end() && !matched[it->second])
        {
            row.after = &current.rows[it->second];
            matched[it->second] = true;
        }
        else unmatched.push_back(diff.size());
        diff.push_back(std::move(row));
    }
    if (!unmatched.empty())
    {
        std::unordered_map<std::wstring, std::deque<size_t>> currentByKey;  // Unmatched rows, in order
        for (size_t i = 0; i < current.rows.size(); ++i)
            if (!matched[i]) currentByKey[Key(current.rows[i].relative)].push_back(i);
        for (const size_t d : unmatched)
        {
            const auto it = currentByKey.find(Key(diff[d].relative));
            if (it == currentByKey.end() || it->second.empty()) continue;
            diff[d].after = &current.rows[it->second.front()];
            matched[it->second.front()] = true;
            it->second.pop_front();
        }
    }
    for (size_t i = 0; i < current.rows.size(); ++i)
        if (!matched[i]) diff.push_back({ .relative = current.rows[i].relative, .after = &current.rows[i] });

    summary = {};
    for (auto& row : diff)
    {
        const ULONGLONG beforeSize = row.before ? row.before->size : 0, afterSize = row.after ? row.after->size : 0;
        const ULONGLONG beforeFiles = row.before ? row.before->files : 0, afterFiles = row.after ? row.after->files : 0;
        row.sizeDelta = static_cast<LONGLONG>(afterSize) - static_cast<LONGLONG>(beforeSize);
        row.filesDelta = static_cast<LONGLONG>(afterFiles) - static_cast<LONGLONG>(beforeFiles);
        if (!row.before) row.change = Change::Added;
        else if (!row.after) row.change = Change::Removed;
        else if (row.sizeDelta > 0) row.change = Change::Grown;
        else if (row.sizeDelta < 0) row.change = Change::Shrunk;
        else if (row.filesDelta != 0 || row.before->folders != row.after->folders) row.change = Change::FilesChanged;
        else row.change = Change::Unchanged;

        ++summary.counts[std::to_underlying(row.change)];
        if (row.relative == rootRelative)
        {
            summary.sizeDelta = row.sizeDelta;
            summary.filesDelta = row.filesDelta;
        }
    }

    std::ranges::sort(diff, [](const DiffRow& a, const DiffRow& b)
    {
        if (a.relative == rootRelative || b.relative == rootRelative) return a.relative == rootRelative && b.relative != rootRelative;
        return _wcsicmp(a.relative.c_str(), b.relative.c_str()) < 0;
    });
    return diff;
}

std::wstring Ledger::ChangeName(const Change change)
{
    switch (change)
    {
    case Change::Added:        return Localization::Lookup(IDS_LEDGER_ADDED);
    case Change::Removed:      return Localization::Lookup(IDS_LEDGER_REMOVED);
    case Change::Grown:        return Localization::Lookup(IDS_LEDGER_GROWN);
    case Change::Shrunk:       return Localization::Lookup(IDS_LEDGER_SHRUNK);
    case Change::FilesChanged: return Localization::Lookup(IDS_LEDGER_FILES_CHANGED);
    default:                   return Localization::Lookup(IDS_LEDGER_UNCHANGED);
    }
}

// Fixed English codes for the CSV so scripts and DeepServer can parse it whatever the UI language is
// (like the ledger's fixed header). ChangeName() stays the localized text for the dialogs.
static std::wstring_view ChangeCode(const Ledger::Change change)
{
    switch (change)
    {
    case Ledger::Change::Added:        return L"Added";
    case Ledger::Change::Removed:      return L"Removed";
    case Ledger::Change::Grown:        return L"Grown";
    case Ledger::Change::Shrunk:       return L"Shrunk";
    case Ledger::Change::FilesChanged: return L"FilesChanged";
    default:                           return L"Unchanged";
    }
}

bool Ledger::SaveComparison(const std::wstring& path, const std::vector<DiffRow>& rows)
{
    std::wstring text = L"Folder,Change,Baseline Size (bytes),Current Size (bytes),Size Change (bytes),"
        L"Baseline Files,Current Files,Files Change";
    const auto value = [](const Row* row, ULONGLONG Row::* field) { return row ? std::to_wstring(row->*field) : std::wstring(); };
    for (const auto& row : rows)
    {
        text += std::format(L"\r\n{},{},{},{},{},{},{},{}", Quote(row.relative), Quote(std::wstring(ChangeCode(row.change))),
            value(row.before, &Row::size), value(row.after, &Row::size), row.sizeDelta,
            value(row.before, &Row::files), value(row.after, &Row::files), row.filesDelta);
    }
    return WriteFile(path, text);
}

std::vector<size_t> Ledger::Significant(const std::vector<DiffRow>& rows, const ULONGLONG minOwnDelta, const bool all)
{
    std::unordered_map<std::wstring, size_t> byRelative;
    byRelative.reserve(rows.size());
    for (size_t i = 0; i < rows.size(); ++i) byRelative.emplace(rows[i].relative, i);

    // Parents are found by exact path, so "Data\sub" never lands under a sibling "data". The
    // case-insensitive map is only built if a parent is missing (e.g. after a case-only rename).
    std::unordered_map<std::wstring, size_t> byKey;
    const auto findParent = [&](const std::wstring& relative) -> size_t
    {
        const std::wstring parentRelative = ParentRelative(relative);
        if (parentRelative.empty()) return SIZE_MAX;
        if (const auto it = byRelative.find(parentRelative); it != byRelative.end()) return it->second;
        if (byKey.empty())
            for (size_t i = 0; i < rows.size(); ++i) byKey.emplace(Key(rows[i].relative), i);
        const auto it = byKey.find(Key(parentRelative));
        return it == byKey.end() ? SIZE_MAX : it->second;
    };

    // A folder's own delta is its change minus the change of its direct child rows.
    std::vector<LONGLONG> own(rows.size());
    std::vector<size_t> parent(rows.size(), SIZE_MAX);
    for (size_t i = 0; i < rows.size(); ++i)
    {
        own[i] += rows[i].sizeDelta;
        if (const size_t p = findParent(rows[i].relative); p != SIZE_MAX && p != i)
        {
            parent[i] = p;
            own[p] -= rows[i].sizeDelta;
        }
    }

    const auto isAddedOrRemoved = [&](const size_t i)
    {
        return i != SIZE_MAX && (rows[i].change == Change::Added || rows[i].change == Change::Removed);
    };

    std::vector<size_t> result;
    for (size_t i = 0; i < rows.size(); ++i)
    {
        const Change change = rows[i].change;
        if (all)
        {
            if (change != Change::Unchanged) result.push_back(i);
            continue;
        }
        if ((change == Change::Added || change == Change::Removed) && !isAddedOrRemoved(parent[i]))
            result.push_back(i);
        else if ((change == Change::Grown || change == Change::Shrunk) &&
            static_cast<ULONGLONG>(own[i] < 0 ? -own[i] : own[i]) >= minOwnDelta)
            result.push_back(i);
    }
    return result;
}

std::wstring Ledger::SignedBytes(const LONGLONG delta)
{
    if (delta == 0) return L"0";
    const auto magnitude = static_cast<ULONGLONG>(delta < 0 ? -delta : delta);
    return (delta < 0 ? L"−" : L"+") + FormatBytes(magnitude);
}

std::wstring Ledger::SignedCount(const LONGLONG delta)
{
    if (delta == 0) return L"0";
    const auto magnitude = static_cast<ULONGLONG>(delta < 0 ? -delta : delta);
    return (delta < 0 ? L"−" : L"+") + FormatCount(magnitude);
}

COLORREF Ledger::ChangeColor(const Change change)
{
    const bool dark = DarkMode::IsDarkModeActive();
    switch (change)
    {
    case Change::Added:        return dark ? RGB(110, 220, 120) : RGB(0, 128, 0);
    case Change::Removed:      return dark ? RGB(255, 120, 120) : RGB(192, 0, 0);
    case Change::Grown:        return dark ? RGB(255, 190, 90) : RGB(176, 96, 0);
    case Change::Shrunk:       return dark ? RGB(120, 180, 255) : RGB(0, 90, 190);
    case Change::FilesChanged: return dark ? RGB(200, 160, 255) : RGB(110, 50, 160);
    default:                   return DarkMode::SystemColor(COLOR_WINDOWTEXT);
    }
}
