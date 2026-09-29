// altWinDirStat - automatic scan history and change detection (fork-owned; not part of upstream WinDirStat)
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
#include "History.h"
#include "ForkSettings.h"
#include "Item.h"

namespace
{
    struct Session
    {
        bool resolved = false;                    // Baseline lookup done for this location
        std::optional<Ledger::Snapshot> baseline;
        FILETIME baselineTime{};
        std::filesystem::path file;               // This process's snapshot for the location
    };

    std::mutex s_lock;
    std::unordered_map<std::wstring, Session> s_sessions;   // By location key
    std::unordered_set<std::wstring> s_written;             // Lower-cased paths this process wrote

    std::wstring Lower(std::wstring text)
    {
        if (!text.empty()) CharLowerBuffW(text.data(), static_cast<DWORD>(text.size()));
        return text;
    }

    // FNV-1a 64 over the UTF-16 code units; diskusage-core's hash_key must give the same result.
    std::wstring HashKey(const std::wstring& key)
    {
        std::uint64_t hash = 14695981039346656037ull;
        for (const wchar_t c : key)
        {
            hash ^= static_cast<std::uint64_t>(c);
            hash *= 1099511628211ull;
        }
        return std::format(L"{:016x}", hash);
    }

    std::wstring NewSnapshotName()
    {
        SYSTEMTIME t{};
        GetSystemTime(&t);
        return std::format(L"{:04}{:02}{:02}-{:02}{:02}{:02}-{:03}.ledger.csv",
            t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond, t.wMilliseconds);
    }

    // Snapshot files of one location, newest first (names sort chronologically). Names starting with
    // '.' are DeepServer's in-progress ".partial-*" scans and are neither baselines nor pruned.
    std::vector<std::filesystem::path> ListSnapshots(const std::filesystem::path& dir)
    {
        std::vector<std::filesystem::path> files;
        std::error_code ec;
        for (const auto& entry : std::filesystem::directory_iterator(dir, ec))
        {
            const std::wstring name = entry.path().filename().wstring();
            if (entry.is_regular_file(ec) && !name.starts_with(L'.') && Ledger::IsLedgerPath(name)) files.push_back(entry.path());
        }
        std::ranges::sort(files, std::greater{});
        return files;
    }

    void ResolveBaseline(const std::filesystem::path& dir, Session& session)
    {
        session.resolved = true;
        for (const auto& file : ListSnapshots(dir))
        {
            if (s_written.contains(Lower(file.wstring()))) continue;
            std::wstring error;
            auto snapshot = Ledger::Load(file.wstring(), error);
            if (!snapshot)
            {
                VTRACE(L"History: skipping unreadable snapshot {}: {}", file.wstring(), error);
                continue;
            }
            WIN32_FILE_ATTRIBUTE_DATA data{};
            if (GetFileAttributesExW(file.c_str(), GetFileExInfoStandard, &data)) session.baselineTime = data.ftLastWriteTime;
            session.baseline = std::move(snapshot);
            return;
        }
    }

    void Prune(const std::filesystem::path& dir)
    {
        const auto files = ListSnapshots(dir);
        std::error_code ec;
        for (size_t i = History::KeepPerLocation; i < files.size(); ++i) std::filesystem::remove(files[i], ec);
    }

    // UTF-16 with BOM so any path round-trips; CREATE_NEW leaves an existing file alone.
    void WriteLocationFile(const std::filesystem::path& dir, const std::wstring& key)
    {
        const HANDLE file = CreateFileW((dir / L"location.txt").c_str(), GENERIC_WRITE, 0, nullptr,
            CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr);
        if (file == INVALID_HANDLE_VALUE) return;
        constexpr wchar_t bom = 0xFEFF;
        DWORD written = 0;
        WriteFile(file, &bom, sizeof(bom), &written, nullptr);
        WriteFile(file, key.data(), static_cast<DWORD>(key.size() * sizeof(wchar_t)), &written, nullptr);
        CloseHandle(file);
    }
}

std::filesystem::path History::Root()
{
    std::wstring overrideDir(MAX_PATH, L'\0');
    if (const DWORD n = GetEnvironmentVariableW(L"DEEPSERVER_HISTORY_DIR", overrideDir.data(), MAX_PATH); n > 0 && n < MAX_PATH)
    {
        overrideDir.resize(n);
        return overrideDir;
    }

    PWSTR local = nullptr;
    std::filesystem::path root;
    if (SUCCEEDED(SHGetKnownFolderPath(FOLDERID_LocalAppData, 0, nullptr, &local))) root = local;
    CoTaskMemFree(local);
    return root / L"DeepServer" / L"History";
}

std::wstring History::LocationKey(const CItem* root)
{
    std::vector<std::wstring> parts;
    if (root->GetItemType() == IT_MYCOMPUTER)
    {
        for (const CItem* child : root->GetChildren()) parts.push_back(child->GetPath());
    }
    else parts.push_back(root->GetPath());

    for (auto& part : parts)
    {
        std::ranges::replace(part, L'/', L'\\');
        while (!part.empty() && part.back() == L'\\') part.pop_back();
        part = Lower(std::move(part));
    }
    std::ranges::sort(parts);

    std::wstring key;
    for (const auto& part : parts)
    {
        if (!key.empty()) key += L'|';
        key += part;
    }
    return key;
}

std::shared_ptr<const History::Result> History::OnScanComplete(const CItem* root)
{
    if (root == nullptr || !ForkSettings::TrackChanges) return nullptr;
    try
    {
        const std::wstring key = LocationKey(root);
        const auto dir = Root() / HashKey(key);
        std::filesystem::create_directories(dir);
        WriteLocationFile(dir, key);

        auto result = std::make_shared<Result>();
        result->current = Ledger::FromScan(root);

        const std::lock_guard guard(s_lock);
        auto& session = s_sessions[key];
        if (!session.resolved) ResolveBaseline(dir, session);
        if (session.file.empty()) session.file = dir / NewSnapshotName();

        // Save before comparing: Compare may narrow 'current' in place. Save is atomic (.tmp + rename).
        if (!Ledger::Save(session.file.wstring(), result->current))
            VTRACE(L"History: could not write {}", session.file.wstring());
        s_written.insert(Lower(session.file.wstring()));
        Prune(dir);

        if (!session.baseline) return nullptr;
        result->baseline = *session.baseline;  // Copy: Compare may narrow it in place
        result->baselineTime = session.baselineTime;
        result->rows = Ledger::Compare(result->baseline, result->current, result->summary);
        return result;
    }
    catch (const std::exception& e)
    {
        VTRACE(L"History: {}", std::wstring(e.what(), e.what() + strlen(e.what())));
        return nullptr;
    }
}

void History::PublishScan(const CItem* root)
{
    (void)OnScanComplete(root);
}
