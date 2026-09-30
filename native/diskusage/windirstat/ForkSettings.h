// altWinDirStat - fork-only settings (fork-owned; not part of upstream WinDirStat)
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

#pragma once

#include "pch.h"
#include "Options.h"

// Persisted with upstream's settings (HKCU\Software\DeepServer\DiskUsage, or the portable ini) in their own section.
struct ForkSettings final
{
    inline static constexpr std::wstring_view Section = L"DeepServer";
    inline static Setting<bool> TrackChanges{ Section, L"TrackChanges", true };
    inline static Setting<bool> ShowRelativeAge{ Section, L"ShowRelativeAge", true };
    // Total size limit for all change-history snapshots, in MB (0 = no limit). Oldest go first.
    inline static Setting<int> HistoryCapMB{ Section, L"HistoryCapMB", 2048, 0, 1024 * 1024 };
};
