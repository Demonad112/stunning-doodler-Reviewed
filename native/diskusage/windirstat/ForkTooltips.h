// altWinDirStat - plain-language tooltips on list column headers (fork-owned; not part of upstream WinDirStat)
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

namespace ForkTooltips
{
    // Maps a column's subitem id to a string key (IDS_TIP_*), or an empty view for no tooltip.
    using Lookup = std::wstring_view (*)(int subitem);

    // Adds a hover tooltip to each header column of 'list'. Safe to call once per list; cleans up with the header.
    void AttachHeader(CListCtrl& list, Lookup lookup);

    std::wstring_view FileTreeTip(int subitem);
    std::wstring_view TopListTip(int subitem);
    std::wstring_view ChangesTip(int subitem);
}
