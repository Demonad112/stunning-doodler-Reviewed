// altWinDirStat - human-friendly formatting helpers (fork-owned; not part of upstream WinDirStat)
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

namespace ForkFormat
{
    FILETIME Now();

    // "just now", "5 min ago", "7 hr ago", "12 days ago", "4 mo ago", "3 yrs ago". Future times give "".
    std::wstring RelativeAge(const FILETIME& then, const FILETIME& now);

    // Last Change cell text: "3 yrs ago (date)" when relative ages are on, otherwise upstream's date text.
    std::wstring LastChangeText(const FILETIME& t);

    // True when relative ages are on and t is more than 365 days ago.
    bool ShouldDimLastChange(const FILETIME& t);
}
