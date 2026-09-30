// altWinDirStat - fork-only headless command lines (fork-owned; not part of upstream WinDirStat)
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
#include "ForkCli.h"
#include "Ledger.h"

namespace
{
    bool IsFlag(const std::wstring& arg, const std::wstring_view name)
    {
        return arg.size() == name.size() + 1 && (arg[0] == L'/' || arg[0] == L'-') &&
            MakeLower(arg.substr(1)) == name;
    }

    std::vector<std::wstring> CommandLineArgs()
    {
        int argc = 0;
        const std::unique_ptr<wchar_t*, decltype(&LocalFree)> argv(CommandLineToArgvW(GetCommandLineW(), &argc), LocalFree);
        if (argv == nullptr) return {};
        return { argv.get(), argv.get() + argc };
    }

    // Best effort: the exit code already reports the failure; this only says why.
    void WriteError(const std::wstring& outPath, const std::wstring& message)
    {
        const std::wstring errPath = outPath + L".err";
        const int size = WideCharToMultiByte(CP_UTF8, 0, message.data(), static_cast<int>(message.size()), nullptr, 0, nullptr, nullptr);
        std::string utf8(static_cast<size_t>(size), '\0');
        WideCharToMultiByte(CP_UTF8, 0, message.data(), static_cast<int>(message.size()), utf8.data(), size, nullptr, nullptr);
        std::ofstream(errPath, std::ios::binary | std::ios::trunc) << utf8;
    }

    bool CompareLedgers(const std::wstring& baselinePath, const std::wstring& currentPath,
        const std::wstring& outPath, const bool all)
    {
        DeleteFile((outPath + L".err").c_str()); // never leave a stale reason next to a fresh result

        std::wstring error;
        auto baseline = Ledger::Load(baselinePath, error);
        if (!baseline)
        {
            WriteError(outPath, L"Cannot read baseline ledger " + baselinePath + L": " + error);
            return false;
        }
        auto current = Ledger::Load(currentPath, error);
        if (!current)
        {
            WriteError(outPath, L"Cannot read current ledger " + currentPath + L": " + error);
            return false;
        }

        Ledger::Summary summary;
        const auto rows = Ledger::Compare(*baseline, *current, summary);
        std::vector<Ledger::DiffRow> picked;
        for (const size_t i : Ledger::Significant(rows, Ledger::DefaultMinOwnDelta, all)) picked.push_back(rows[i]);
        if (!Ledger::SaveComparison(outPath, picked))
        {
            WriteError(outPath, L"Cannot write comparison to " + outPath);
            return false;
        }
        return true;
    }
}

bool ForkCli::NoElevateRequested()
{
    static const bool requested = std::ranges::any_of(CommandLineArgs() | std::views::drop(1),
        [](const std::wstring& arg) { return IsFlag(arg, L"noelevate"); });
    return requested;
}

void ForkCli::RunIfRequested()
{
    const std::vector<std::wstring> args = CommandLineArgs();
    const int argc = static_cast<int>(args.size());
    if (argc < 2 || !IsFlag(args[1], L"compare")) return;

    const bool all = argc == 6 && IsFlag(args[5], L"all");
    if (argc != 5 && !all) ExitProcess(2);
    ExitProcess(CompareLedgers(args[2], args[3], args[4], all) ? 0 : 1);
}
