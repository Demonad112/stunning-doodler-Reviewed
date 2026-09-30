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

#include "pch.h"
#include "ForkFormat.h"
#include "ForkSettings.h"

namespace
{
    constexpr ULONGLONG TicksPerSecond = 10'000'000ull;

    ULONGLONG Ticks(const FILETIME& t) { return (static_cast<ULONGLONG>(t.dwHighDateTime) << 32) | t.dwLowDateTime; }
}

FILETIME ForkFormat::Now()
{
    FILETIME now{};
    GetSystemTimeAsFileTime(&now);
    return now;
}

std::wstring ForkFormat::RelativeAge(const FILETIME& then, const FILETIME& now)
{
    if (Ticks(then) == 0 || Ticks(then) > Ticks(now)) return {};
    const ULONGLONG seconds = (Ticks(now) - Ticks(then)) / TicksPerSecond;
    const ULONGLONG minutes = seconds / 60, hours = minutes / 60, days = hours / 24;
    if (minutes < 1) return Localization::Lookup(IDS_AGE_JUST_NOW);
    if (hours < 1) return Localization::Format(IDS_AGE_MINUTES, minutes);
    if (hours < 48) return Localization::Format(IDS_AGE_HOURS, hours);
    if (days < 60) return Localization::Format(IDS_AGE_DAYS, days);
    if (days < 730) return Localization::Format(IDS_AGE_MONTHS, days / 30);
    return Localization::Format(IDS_AGE_YEARS, days / 365);
}

std::wstring ForkFormat::LastChangeText(const FILETIME& t)
{
    std::wstring date = FormatFileTime(t);
    if (!ForkSettings::ShowRelativeAge || date.empty()) return date;
    const std::wstring age = RelativeAge(t, Now());
    return age.empty() ? date : std::format(L"{} ({})", age, date);
}

bool ForkFormat::ShouldDimLastChange(const FILETIME& t)
{
    const ULONGLONG then = Ticks(t), now = Ticks(Now());
    return ForkSettings::ShowRelativeAge && then != 0 && then < now &&
        (now - then) / TicksPerSecond > 365ull * 24 * 60 * 60;
}
