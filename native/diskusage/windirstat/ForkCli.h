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

#pragma once

#include "pch.h"

namespace ForkCli
{
    // Handles "/compare <baseline.ledger.csv> <current.ledger.csv> <out.csv> [/all]" before upstream's
    // command-line parser sees it: writes the significant (or, with /all, every) folder change and exits
    // the process with 0 on success, 1 when a ledger cannot be read or the output written, 2 on bad usage.
    // Returns normally when the command line is not a fork command. On exit code 1 the reason is written
    // to "<out.csv>.err" (UTF-8), since this GUI exe has no console for a caller to read.
    void RunIfRequested();

    // True when "/noelevate" is on the command line. DeepServer passes it for headless runs so the process
    // it started does the work itself: auto-elevation would relaunch a new elevated copy and exit this one
    // with 0 before anything ran. Upstream's parser ignores the unknown flag.
    bool NoElevateRequested();
}
