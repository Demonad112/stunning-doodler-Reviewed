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

#pragma once

#include "pch.h"
#include "Ledger.h"

class CItem;

// Every completed scan is saved as a folder ledger under Root()\<location hash>\. The first scan of a
// location in this process picks the newest snapshot written by an earlier process as its baseline and
// keeps it for the whole session, so refreshes keep comparing against "last time", not "a moment ago".
// DeepServer (src-tauri/crates/diskusage-core) reads and writes the same layout.
namespace History
{
    constexpr size_t KeepPerLocation = 5;

    struct Result
    {
        Ledger::Snapshot baseline;
        Ledger::Snapshot current;
        std::vector<Ledger::DiffRow> rows;  // Points into baseline/current: keep Result where it was built
        Ledger::Summary summary;
        FILETIME baselineTime{};            // When the baseline snapshot was written (UTC)
    };

    // %LOCALAPPDATA%\DeepServer\History, or DEEPSERVER_HISTORY_DIR when set.
    std::filesystem::path Root();

    // Lower-cased root path without trailing backslashes; for multi-drive scans the sorted drive roots
    // joined with '|'.
    std::wstring LocationKey(const CItem* root);

    // Scan-worker entry: saves this session's snapshot and compares it with the baseline. Returns nullptr
    // when tracking is off, there is no baseline yet, or anything failed (failures are traced, never thrown).
    std::shared_ptr<const Result> OnScanComplete(const CItem* root);

    // Called by the scan worker after a completed (not stopped) scan.
    void PublishScan(const CItem* root);

    struct Usage
    {
        ULONGLONG bytes = 0;
        size_t snapshots = 0;
        size_t locations = 0;
    };

    // Disk use of all snapshots under Root() (DeepServer's in-progress ".partial-*" files excluded).
    Usage GetUsage();

    // Deletes every snapshot and location folder under Root(), except in-progress ".partial-*" files,
    // and forgets this session's baselines. Returns the number of snapshots deleted.
    size_t CleanUp();
}
