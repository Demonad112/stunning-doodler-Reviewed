> This file was compiled by an assistant from a prior conversation on this
> project. Treat it as background material, not verified fact. Reason
> independently before relying on anything here, especially items presented
> as correct, decided or working. It's meant to bring a new collaborator up
> to speed, not to define how they should proceed. It supersedes nothing: the
> older `altwindirstat-reference.md` in this folder still describes the repo,
> the CI and the release flow.

# Reference: altWinDirStat, change-tracking work (session of 2026-09-28/29)

## Quick Recap

altWinDirStat (github.com/Demonad112/altWinDirStat) is the owner's fork of WinDirStat 2.x (C++, Windows). This session was the first on a real Windows machine.

1. **Local build.** The app was built from source (VS 2022 Community, v143) and a portable copy was placed in `%USERPROFILE%\Projects\Work\AltWin\altWinDirStat\` (exe, LICENSE.txt, README.md). It's stamped as a production v2.1.0 build. It ran fine, and Smart App Control did not block it.
2. **New features.** the owner asked for "better, more features, ease of use", for use on their own PCs. A design was agreed:
   - an automatic **Changes** tab, showing what changed since the last scan of a location
   - readable "3 yrs ago" ages, with stale folders dimmed
   - column-header tooltips
   - Logical size hidden by default

   A spec and a 6-task implementation plan were written and committed.
3. **Progress.** Execution started inline. **Task 1 of 6 is committed, and its automated tests pass.** Tasks 2–6 haven't started. Nothing is pushed to GitHub.

## Working Context

### Where the code is (important)
- The working clone is in a **temporary session scratchpad**:
  `%USERPROFILE%\AppData\Local\Temp\claude\C--Users-Addy7-Projects-Work-AltWin\162754c5-5cec-4d86-83f8-db97ee243cee\scratchpad\src`
  It's a full clone on branch `feature/change-tracking`, **not pushed**. Temp folders can be cleaned up, so pushing the branch, or moving the clone somewhere durable, may be worth doing early. the owner said they didn't want the whole repo kept in the project folder, so that's their call.
- Branch commits on top of master `bf40619`:
  - `9e382b2` design spec
  - `4f08b0e` spec adjustments
  - `d3b7926` implementation plan
  - `3f151dc` Task 1: significant-change filter and `/compare` CLI
- Git identity is set repo-locally as `Demonad112 <id+Demonad112@users.noreply.github.com>`, so the owner's real email isn't published.
- Uncommitted leftovers in the clone: UTF-8 BOMs that upstream's pre-build formatter added to `ForkResource.h`, `ForkCommands.cpp`, `res/fork/lang_en.txt` and `.github/scripts/Stress-LargeScan.ps1`. These were deliberately left out of commits. Whether to commit them is open; they're harmless.

### Documents (in the clone)
- Spec: `docs/superpowers/specs/2026-09-29-change-tracking-design.md`
- Plan: `docs/superpowers/plans/2026-09-29-change-tracking.md`. It holds full code for every task, test scripts, manual checklists and a "Review Focus" list.
- Execution ledger (git-ignored): `.superpowers/sdd/2026-09-29-change-tracking/progress.md`. It has pre-flight notes and rulings. The same folder has `build.ps1` and `test.ps1` helpers.

### Design decisions (agreed with the owner; reasoning worth re-checking)
- **Main user:** the owner, on their own PCs. Priorities they picked: "see what changed" and "clearer views". They explicitly chose: automatic snapshots, a Changes tab only (no growth column in the main tree), and hiding pass-through parent folders.
- **Snapshots:**
  - Location: `%LOCALAPPDATA%\altWinDirStat\History\<FNV-1a-64 hex of location key>\YYYYMMDD-HHMMSS-mmm.ledger.csv`. The env var `ALTWDS_HISTORY_DIR` overrides it for tests.
  - Retention: keep 5 per location.
  - Writes: atomic, via `.tmp` then rename.
  - Written on the scan worker thread, never the UI thread.
  - Skipped for stopped scans (`stopReason == Default` only) and for `/saveto`.
- **Baseline semantics:** the newest snapshot not written by this process, resolved once per location per session. F5 keeps comparing against "last time", not "a moment ago". Each session leaves one snapshot per location (its final state).
- **Significance rule (`Ledger::Significant`):**
  - Topmost Added and Removed folders are listed.
  - Grown and Shrunk folders are listed when their *own* delta is ≥ 1 MiB. Own delta = size change minus the size changes of direct child rows.
  - A "show all" mode lists every changed row.
- **Keeping the fork diff small:**
  - Both on/off switches, "Track Changes Between Scans" and "Show Relative Ages", are **Options-menu items**, not a settings-dialog page (to avoid editing upstream dialogs).
  - Tab tooltips were dropped because `CTabControl` keeps tab rectangles private. Column-header tooltips remain.
  - The Changes tab is appended **last**, so upstream tab indices don't shift.
- **Upstream files the plan touches with small hooks:**
  - `WinDirStat.cpp`
  - `WinDirStatModel.Actions.cpp`
  - `WinDirStatModel.h`
  - `windirstat.rc`
  - `FileTabbedView.h/.cpp`
  - `WdsListControl.h/.cpp`: a new `GetSubItemTextColor` virtual
  - `Item.h`, `Item.Extended.cpp`
  - `FileTreeView.cpp`
  - `Options.cpp`

  Every one is meant to be listed in HANDOFF.md's fork-diff table (Task 6).

### Plan tasks and status
1. `Ledger::Significant` plus the shared helpers (`SignedBytes`, `SignedCount`, `ChangeColor`), `CLedgerListCtrl` (moved out of `LedgerCompareDlg`) and `ForkCli` (`/compare`). **Done, committed `3f151dc`, fork tests 6/6.** Step 8, a manual check that the old Compare dialog still works after the refactor, was **not done**.
2. `ForkSettings.h`, `History.h/.cpp`, the scan-worker hook, and GUI snapshot tests (10 checks). Not started.
3. `ForkFormat` (relative ages), the per-cell dim colour, and the two Options-menu toggles. Not started.
4. `Views/FileChangesView` (the Changes tab), the `FileTabbedView` hooks, and `PublishScan` handing results to the UI. Not started.
5. `ForkTooltips` (header tooltips via `SetWindowSubclass`). Not started.
6. Default layout (Logical size off on fresh profiles), a CI step running `tests/Test-ForkChanges.ps1`, HANDOFF.md updates, push, PR, and replacing the local exe. Not started.

The execution method chosen was **native/inline** (executing-plans skill), with one whole-branch review at the end.

### Things learned the hard way
- **Launch-VsDevShell warning:** the dev shell prints `'vswhere.exe' is not recognized`. It's harmless; the build still succeeds.
- **Pre-build formatting:** upstream's pre-build scripts re-sort `res/fork/lang_en.txt` and add BOMs to sources. New string keys go in sorted order, or the tree ends up dirty.
- **Elevation:** unknown command-line flags make the *old* app treat the launch as a normal GUI start and **re-launch itself elevated** ("(Administrator)" in the title). A non-elevated shell can't kill or automate those windows (Access denied; UIPI). A normal folder launch of the new dev build ran **non-elevated**. the owner also had their own installed altWinDirStat 2.1.0 open, elevated, scanning C: during the session.
- **Computer use:** it's granted per executable. The "altWinDirStat" grant maps to the *installed* app (`%LOCALAPPDATA%\Programs\altwindirstat\altwindirstat.exe`), not the dev build `altWinDirStat_x64.exe`. The request to add `altwindirstat_x64.exe` was interrupted, and the owner paused that action, so UI checks weren't done. The alternatives are the owner clicking through the manual checklists, or a new grant.
- **Test fixture:** the plan's original expectation for the significant list was wrong. It omitted `a\b\c`, a leaf that grew by itself. The test was corrected to `a\b\c|edge|gone|new`, which matches the spec, and a ruling was logged.
- **Tooling:** Python heredocs through Git Bash mangled backslash sequences (`\b` became a backspace byte). PowerShell `.Replace()` or the Edit tool were more reliable for paths like `a\b\c`.

## Deep Reference

### Local build and test
```powershell
# From the clone root
& "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -SkipAutomaticLocation | Out-Null
msbuild windirstat.sln /m /nologo /v:minimal /p:Configuration=Release /p:Platform=x64 /p:PlatformToolset=v143
# -> build\altWinDirStat_x64.exe   (add "/p:ExternalCompilerOptions=/DPRODUCTION=1" /p:AltVerMajor=2 /p:AltVerMinor=1 /p:AltVerPatch=0 to drop "Beta" and stamp a version)

powershell -NoProfile -ExecutionPolicy Bypass -File tests\Test-ForkChanges.ps1 -ExePath build\altWinDirStat_x64.exe [-SkipGui]
```
The workspace helpers `.superpowers\sdd\2026-09-29-change-tracking\build.ps1 -Log <file>` and `test.ps1 [-SkipGui]` wrap these commands.

### `/compare` CLI (Task 1, implemented)
- **Usage:** `altWinDirStat.exe /compare <baseline.ledger.csv> <current.ledger.csv> <out.csv> [/all]`
- **Exit codes:** 0 on success, 1 when a ledger can't be read or the output can't be written, 2 on bad usage.
- **Where it runs:** `ForkCli::RunIfRequested()`, called in `CDirStatApp::InitInstance` right after the bootstrap `Localization::LoadResource`, so upstream's strict parser never sees it.
- **Output:** the `Ledger::SaveComparison` CSV (header `Folder,Change,...`), rows in relative-path order.

### Task 1 fixture expectations (`tests/fixtures/changes/`)
- base → cur significant: `a\b\c | edge | gone | new`
- `/all`: `. | a | a\b | a\b\c | edge | gone | gone\sub | new | new\inner | small`
- `small` is +1,048,575 bytes, one byte under the 1 MiB threshold, so it's hidden. `edge` is exactly +1 MiB, so it's shown.

### Upstream hook points found while planning
- **Scan-complete point:** `WinDirStatModel.Actions.cpp`, inside the scan `std::jthread`, after `CItem::ScanItemsFinalize(GetRootItem()); Get()->RebuildExtensionData();` and before the `/saveto` / `/savedupesto` / `/savepermsto` exits and the UI `InvokeInMessageThread` block. `stopReason` is in scope there (`Default` / `Stop` / `Abort`); `Abort` returns earlier.
- **`CMainFrame::InvokeInMessageThread`:** a synchronous `SendMessage` from worker threads.
- **Tabs:** `CFileTabbedView::AddPane<T>(index, IDS_key)`. Optional tabs are hidden in `ResetOptionalTabVisibility()` on `MODEL_CHANGE_NEW_ROOT`.
- **Settings:** `Setting<bool>{section, name, default}` self-registers into `PersistedSetting::GetPropertySet()`, so a fork-owned header with `inline static Setting<bool>` persists automatically.
- **String keys:** `IDS_*` are `constexpr std::wstring_view` constants generated by `Build/Compress Strings.ps1` from every `res/**/lang_*.txt`, including `res/fork/lang_en.txt`.
- **Colour:** per-row text colour comes from `CWdsListItem::GetItemTextColor()`, used in `CWdsListControl::DrawItem`. There's no per-cell hook upstream, hence the planned `GetSubItemTextColor` virtual.
- **Default columns:** fresh-profile file-tree column visibility is initialized in `Options.cpp`: `visibility = { 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0 };` (index 4 = `COL_SIZE_LOGICAL`).
- **Selecting from another view:** `CWinDirStatModel::Get()->NotifyPanes(MODEL_CHANGE_SELECTION_ACTION, item)` plus `SetActiveFileTreeView()`. `CItem::FindItemByPath` only works from a drive root and compares case-sensitively, which is why the plan has its own walk.
- **Existing features:** upstream already has Largest Files, Duplicates, Search (size/regex), size and age filters, Watcher, Permissions and Storage Analytics tabs. Last Change for folders is already the newest time in the subtree.
