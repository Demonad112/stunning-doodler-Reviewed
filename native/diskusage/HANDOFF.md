# altWinDirStat: Handoff Notes

> Background for whoever works on this repo next (human or Claude). Check claims against the code and `git log` before relying on them.

## What this repo is now

* **Since the WinDirStat 2.x adoption, altWinDirStat is an unofficial fork of the official [WinDirStat](https://github.com/windirstat/windirstat) 2.x.**
* **Why it changed:** the legacy altWinDirStat (2014–2016 MFC fork, releases up to `v0.1.0`) scanned on the UI thread. It froze on a laptop's C: drive and couldn't handle multi-terabyte servers. Official 2.x already has what we needed:
  * multithreaded scanning and direct NTFS MFT reading
  * suspend, resume and stop
  * duplicates, cleanups and search
* **How the history was joined:** `git merge -s ours --allow-unrelated-histories` joined the two histories. The files are upstream's; the old code is still reachable at tag `v0.1.0`.

## altWinDirStat changes on top of upstream (keep this list current)

Keep this diff small, because every upstream sync has to merge through it.

| File | Change | Why |
|---|---|---|
| `windirstat/Constants.h` | `strWinDirStat` = `altWinDirStat`; `strUninstall` → `...\Uninstall\altWinDirStat` | Drives titles, the Explorer context-menu key, the exe-name match and the uninstall key |
| `windirstat/Version.h` | Product name, description, exe name, repository, company/copyright | Version resource shows altWinDirStat; the copyright still credits the WinDirStat Team |
| `windirstat/Version.h` (block at the end) | `ALT_VER_*` (default 2.0.0) replace upstream's `PRD_MAJVER/MINVER/PATCH` via `#undef` | The fork has its own version number. Kept at the end of the file so upstream version bumps merge without conflicts. **Bump the defaults after each release** |
| `windirstat/Property.cpp` | Registry root `Software\altWinDirStat\altWinDirStat\` | Settings don't collide with official WinDirStat |
| `windirstat/WinDirStat.cpp` | "Reset preferences" deletes `Software\altWinDirStat` | Matches the key above |
| `windirstat/Localization.cpp/.h` | `ApplyForkBranding()` rewrites "WinDirStat" → "altWinDirStat" in loaded strings. It keeps "WinDirStat Team" and adds a fork notice to the About text | Rebrands the UI without editing 25 `lang_*.txt` files |
| `windirstat/windirstat.rc` | `IDS_URL_REPORT_BUG` → `github.com/Demonad112/altWinDirStat/issues/new` | Fork bug reports shouldn't land in upstream's tracker. Help → manual still opens upstream's wiki (same features) |
| `windirstat/windirstat.vcxproj` | `TargetName` = `altWinDirStat_<arch>` | Output exe name; the portable INI follows it (`altWinDirStat.ini`) |
| Removed: `.github/workflows/publish-*-to-winget-pkgs.yml`, `.github/FUNDING.yml`, `setup/chocolatey/`, `setup/store/` | | These publish under the official identity. **If a sync re-adds them, delete them again.** CI's `fork-guard` job fails when any of them is present, and it also blocks releases |
| Added: `.github/workflows/build.yml`, `.github/workflows/sync-upstream.yml`, `installer/altWinDirStat.iss`, `README.md`, `HANDOFF.md`, `CLAUDE.md`, `docs/releases/` | | Our CI, sync, installer, docs and release notes |
| `windirstat/windirstat.rc` | `#include "ForkResource.h"` plus two File-menu items (`ID_FOLDER_LEDGER_EXPORT`, `ID_FOLDER_LEDGER_COMPARE`) | Folder ledger menu entries |
| `windirstat/WinDirStatModel.h` | `#include "ForkResource.h"`, three method declarations and three routes | Folder ledger commands; the bodies live in `ForkCommands.cpp` |
| `windirstat/CsvLoader.cpp` | One-line hook at the top of `SaveResults`: `*.ledger.csv` targets write a folder ledger | Save Results and `/saveto` can produce ledgers |
| `windirstat/windirstat.vcxproj` | One separate `ItemGroup` listing the fork sources below | Keeps upstream's item lists untouched |
| Added: `ForkResource.h`, `Fork.rc`, `Ledger.cpp/.h`, `ForkCommands.cpp`, `Dialogs/LedgerCompareDlg.cpp/.h`, `res/fork/lang_en.txt` | | Folder ledger export and side-by-side comparison. IDs use ranges upstream doesn't (900, 1900-1906, 33900-33901). Strings live in the fork's own `lang_en.txt`, which `Compress Strings.ps1` picks up recursively. That keeps the 25 upstream language files and their key-parity test untouched, and other languages fall back to English |
| Added: `project.early.props` | | The vcxproj already imports it if present. When MSBuild gets `/p:AltVerMajor/AltVerMinor/AltVerPatch`, it defines `ALT_VER_*` for both `cl` and `rc` (`ExternalCompilerOptions` reaches only `cl`, so it can't stamp FileVersion) |

`setup/msi` (upstream's WiX MSI) is left untouched and unused.

## DeepServer changes (on top of altWinDirStat; keep this list current)

This folder is the disk-usage engine bundled in DeepServer (`native/diskusage` in stunning-doodler-Reviewed). DeepServer ships it as `deepserver-diskusage.exe` next to `DeepServer.exe` and drives it through the headless CLI (`src-tauri/crates/diskusage-core`).

| File | Change | Why |
|---|---|---|
| `windirstat/Constants.h` | `strWinDirStat` = `DeepServer Disk Usage` | Titles and the engine's own Explorer-menu key; the DeepServer installer writes the same key name so the two overlay |
| `windirstat/Localization.cpp` | Fork name and About header say DeepServer Disk Usage, linking this repo | Branding |
| `windirstat/Version.h` | Product name, description, exe name `deepserver-diskusage.exe`, company, repository; `ALT_VER_*` default 1.0.0 | Version resource matches DeepServer |
| `windirstat/Property.cpp`, `windirstat/WinDirStat.cpp` (reset preferences) | Registry root `HKCU\Software\DeepServer\DiskUsage` | Settings live with DeepServer's, separate from altWinDirStat/WinDirStat |
| `windirstat/windirstat.rc` | `IDS_URL_REPORT_BUG` → this repo's issues | Bug reports go to DeepServer |
| `windirstat/WinDirStat.cpp` | `autoElevate` and the elevation prompt also require `!ForkCli::NoElevateRequested()` | `/noelevate`: a headless run DeepServer starts must do the work itself instead of relaunching elevated and exiting 0 |
| `windirstat/ForkCli.cpp/.h` | `/noelevate` flag; `/compare` writes the failure reason to `<out>.err` (UTF-8) | The GUI exe has no console, so callers only saw exit code 1 |
| `windirstat/Ledger.cpp` | Comparison CSV `Change` column uses fixed English codes (`Added`, `Removed`, `Grown`, `Shrunk`, `FilesChanged`, `Unchanged`) | Parsable whatever the UI language; dialogs still show localized names |
| `windirstat/windirstat.vcxproj` | Default toolset v143 (VS2022); v145 only with VS2026+ | Builds on CI and dev PCs without `/p:PlatformToolset` |
| `.github/scripts/Stress-LargeScan.ps1`, `ForkCommands.cpp`, `ForkResource.h`, `res/fork/lang_en.txt` | UTF-8 BOM / key order as the pre-build formatter writes them | Committed as formatted so every build leaves a clean tree |
| Removed: `.github/workflows/sync-upstream.yml`, `maintenance.yml` | | Upstream sync and prerelease cleanup belonged to the standalone fork; `build.yml` here is inert (DeepServer CI builds the engine in `windows-installer.yml`) |

Change tracking and view polish (DeepServer Batch 3; design in `docs/superpowers/specs/2026-09-29-change-tracking-design.md`, plan in `docs/superpowers/plans/2026-09-29-change-tracking.md`):

| File | Change | Why |
|---|---|---|
| `windirstat/WinDirStat.cpp` | `#include "ForkCli.h"` and `ForkCli::RunIfRequested()` after the bootstrap language load | Headless `/compare`, handled before upstream's strict parser |
| `windirstat/WinDirStatModel.Actions.cpp` | `#include "History.h"` and one line after `RebuildExtensionData()`: `History::PublishScan` when `stopReason == Default` and not `/saveto` | Automatic snapshots and the Changes tab; stopped scans are excluded |
| `windirstat/WinDirStatModel.h` | five more declarations and routes in the fork block | Options-menu toggles and history clean-up |
| `windirstat/windirstat.rc` | three Options-menu items plus a separator | Track Changes, Show Relative Ages, Clean Up Change History |
| `windirstat/Views/FileTabbedView.h/.cpp` | forward declaration, include, two members, three inline methods, `SetChangesTabVisibility`, `AddPane` (added last), and the Changes view added to `ResetOptionalTabVisibility`, `OnUpdate` and `CycleTab` | Hosts the Changes tab without shifting upstream tab indices |
| `windirstat/Controls/WdsListControl.h/.cpp` | `CWdsListItem::GetSubItemTextColor` virtual (defaults to the row colour); `DrawItem` calls it | Per-cell colour for dimmed ages |
| `windirstat/Item.h`, `windirstat/Item.Extended.cpp` | `GetSubItemTextColor` override; Last Change text via `ForkFormat::LastChangeText` | Readable ages; stale folders dimmed |
| `windirstat/Views/FileTreeView.cpp` | `ForkTooltips::AttachHeader` at the end of the All Files and Largest Files `InitializeColumns` | Header tooltips |
| `windirstat/Options.cpp` | default `FileTreeColumnVisibility`: Logical size off | Less clutter on fresh profiles; existing layouts untouched |
| `windirstat/Ledger.cpp/.h` (fork-owned) | Exact-path matching before case-insensitive fallback in `Compare`/`Significant` (E7); `Load` requires exactly one root row, every write goes through `<path>.tmp` + rename (E8); trailing `#filters=<fingerprint>` line and `FiltersDiffer` (E23) | Case-sensitive folders, corrupt files, filter changes misread as deletions |
| `windirstat/ForkCli.cpp` (fork-owned) | `/compare` writes `<out>.warn` when the two ledgers' filter fingerprints differ | DeepServer can show the warning next to its compare table |
| `windirstat/windirstat.vcxproj`, `windirstat/ForkResource.h` | new sources in the fork `ItemGroup`; commands 33902–33904, controls 1907–1909 | |
| Added: `ForkSettings.h` (`TrackChanges`, `ShowRelativeAge`, `HistoryCapMB` in section `DeepServer`), `History`, `ForkFormat`, `ForkTooltips`, `Views/FileChangesView`, `tests/fixtures/changes/*` | | Snapshots live in `%LOCALAPPDATA%\DeepServer\History\<16-hex FNV-1a-64 of the location key>\` (`DEEPSERVER_HISTORY_DIR` overrides), the same layout `diskusage-core` reads. `.partial-*` files (DeepServer's in-progress scans) are skipped. Newest 5 per location; `HistoryCapMB` (default 2048, 0 = off) prunes the oldest across locations but keeps each location's newest |

Cross-launch (DeepServer Batch 4):

| File | Change | Why |
|---|---|---|
| `windirstat/windirstat.rc` | one item in the tree/list right-click menu (`IDR_POPUP_TREE`), after Open | *Compare in DeepServer* |
| `windirstat/WinDirStatModel.h` | two declarations and two routes in the fork block | The handler and its enable check live in `ForkCommands.cpp` |
| `windirstat/ForkResource.h`, `res/fork/lang_en.txt` (fork-owned) | command 33905, `IDS_MENU_FORK_COMPARE_IN_DEEPSERVER` | |
| `windirstat/ForkCommands.cpp` (fork-owned) | *Compare in DeepServer*: runs `DeepServer.exe --shell-compare "<folder>"` from the engine's folder. One selected folder works like Explorer's *Compare with DeepServer* (first pick = left side, next pick opens the compare); two selected folders set the left side (`--select-left`), wait up to 10 s, then open the compare. Disabled unless 1–2 folders/drives are selected and `DeepServer.exe` sits next to the engine | Reuses DeepServer's existing shell-compare path, so no new argument parsing on either side |

## Build

* **Visual Studio 2026:** build `windirstat.sln` as is.
* **Visual Studio 2022:** add `/p:PlatformToolset=v143`. It builds cleanly (verified locally and in CI):
  ```
  msbuild windirstat.sln /m /p:Configuration=Release /p:Platform=x64 /p:PlatformToolset=v143
  ```
* **Output:** `build\altWinDirStat_{x64|x86|arm64}.exe`.
* **Pre-build steps:** they run PowerShell scripts (`windirstat\Build\*.ps1`) that format the sources and compress the language strings into `res\lang_combined.bin`.

## CI (`.github/workflows/build.yml`)

* **Build matrix:** x64, Win32 and ARM64 on `windows-2022`. ARM64 is build-only.
* **Tests (x64 and Win32):**
  * Smoke/GUI scan of the repo and `C:\Program Files`. It fails if the window stops responding for more than 3 seconds, or if the title isn't "altWinDirStat".
  * Upstream's `tests/Test-WinDirStat.ps1 -Only Filtering,Cli,EdgeCases,Permissions,Enumeration,Unc,Ui`. Settings is left out because it rebuilds the solution without `PlatformToolset=v143`, which the VS2022 runner needs. Mtp and Reparse are left out because they need a phone or scratch drives. It needs PowerShell 7.6+, which CI downloads if the runner is older.
  * Inno installer silent install + uninstall.
* **Large-scan stress test** (`stress` job, `.github/scripts/Stress-LargeScan.ps1`, x64):
  * **Headless export:** generates a ~700k-file tree (a folder grid, a flat folder with 100k files, and a path deeper than MAX_PATH). A headless `/saveto` export must list every file, and the root's file count and byte total must match exactly.
  * **GUI scans:** it then scans that tree (generic multithreaded finder) and all of `C:\` (MFT finder, since runners are elevated). Throughout each scan and for 15 s after it, it probes the window every 250 ms with `SendMessageTimeout(WM_NULL)`, and it resizes the window after each scan to force a treemap re-layout.
  * **Fails on:** any probe of 5 s or more, which is Windows' "Not Responding" threshold. Scan times, worst and p99 UI latency, and peak memory are written to the job summary.
  * **Releases depend on it.**
* **Signing:** optional, via repository secrets.
  * **Azure Trusted Signing:** `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `SIGNING_ENDPOINT` (e.g. `https://eus.codesigning.azure.net/`), `SIGNING_ACCOUNT`, `SIGNING_PROFILE`.
  * **Or a .pfx:** `SIGNING_PFX_BASE64` + `SIGNING_PFX_PASSWORD`.
  * **Without either:** builds are unsigned and a notice appears on the run. **Smart App Control blocks unsigned builds**, which is why the official signed WinDirStat ran on the owner's laptop but our builds didn't.
* **Versioning:**
  * Plain builds report the `ALT_VER_*` default from `Version.h` and are named `<ver>-dev.<run>`.
  * Release builds stamp the tag's version into the exe.
  * The "Collect binary" step fails if the exe's FileVersion doesn't match.
  * A tag with a `-suffix` (e.g. `v2.1.0-rc.1`) is a prerelease: "Beta" stays in the title, and it isn't marked as the latest release.
* **Release**, either way:
  * **Actions → Build → Run workflow** with `release_tag` = `vX.Y.Z`. The release job creates the tag on the commit that was built, and refuses if the tag already exists.
  * Or `git tag vX.Y.Z && git push origin vX.Y.Z`.
  * The release gets the installers, the portable zips and `SHA256SUMS.txt`. Its body is `docs/releases/<tag>.md` if that file exists; otherwise GitHub generates the notes.

## Syncing with upstream (`.github/workflows/sync-upstream.yml`)

* **What it does:**
  * Runs Mondays and on demand.
  * Merges `windirstat/windirstat` master into `sync/upstream-<date>` and opens a PR.
  * Starts the Build workflow on that branch.
* **Conflicts:** the conflict markers are committed so they can be resolved on the PR branch.
* **Repo setting needed:** Settings → Actions → General → **"Allow GitHub Actions to create and approve pull requests"** must be on, or the PR step fails. The branch is still pushed, so you can open the PR by hand.
* **Manual sync:**
  ```
  git remote add wds https://github.com/windirstat/windirstat.git
  git fetch wds && git merge wds/master
  ```

## Settings carried over from legacy 1.x

Legacy 1.x and 2.x both use `HKCU\Software\altWinDirStat\altWinDirStat\<section>`. Registry key names are case-insensitive, so legacy `options` is 2.x's `Options`.

* **Values 2.x picks up:** only `Options\ListStripes` and `Options\ListFullRowSelection`. Both are bool DWORDs with the same meaning, so they carry over correctly.
* **Everything else from legacy is never read.** That includes the `persistence` section and the treemap values, which 2.x keeps in other sections. These values are harmless leftovers, and **Reset preferences** removes them.
* **No crash risk from mismatched values:** `Property.cpp` reads with strict types (`QueryDWORDValue`, and `REG_SZ` only for strings) and falls back to the default on a mismatch.

## Maintenance

* **Deleting a prerelease:** go to **Actions → Maintenance → Run workflow** with `delete_prerelease=<tag>`.
  * It deletes a prerelease and its git tag.
  * It refuses final releases.
  * It exists because cloud Claude sessions can't delete tags through their git proxy.
* **Old branches:** `Problem-commit`, `attempt-removal-of-ConcRT-deps` and `fix-blank-header-ctrl` are 2014–2016 legacy branches, and `claude/no-command-line` is a legacy-codebase feature branch. None is merged into master. They're kept for history; delete them only on purpose.

## Manual checklist: Changes tab, ages, tooltips and history

`tests/Test-ForkChanges.ps1` (run by DeepServer CI in `windows-installer.yml`, GUI checks included) covers `/compare`, snapshots, retention, the size cap and filter fingerprints. These need a person (x64 build, history at its default location):

1. Scan a scratch folder `%TEMP%\awds-manual` containing `a\b` and `c`. Close the app.
2. Add a 20 MB file to `a\b`, delete `c`, create `d\e`. Scan the same folder again. Expected: a **Changes** tab; the summary reads `Since <date> (…): net …, 1 new, 1 removed, 1 grew, 0 shrank`; rows `a\b` (Grown, orange), `c` (Removed, red), `d` (Added, green); `a` and `.` not listed.
3. Tick **Show all changed folders**: `a` and `.` appear. Untick: they disappear.
4. Click each column header: it sorts; clicking again reverses.
5. Double-click `a\b`: All Files activates with `a\b` selected. Double-click `c`: nothing happens.
6. F5: the tab stays and still compares against the first session.
7. Options → Track Changes Between Scans off: the tab disappears; a rescan shows no tab and writes no snapshot. Turn it back on.
8. Scan a folder never scanned before: no Changes tab.
9. Change an exclusion filter and rescan: the summary starts with the filter warning.
10. Options → Clean Up Change History: the dialog shows the snapshot count, size and limit; Yes deletes the history and hides the tab.
11. Last Change shows `N yrs ago (date)`; folders untouched for over a year are grey; Options → Show Relative Ages toggles both.
12. Hover the All Files, Largest Files and Changes headers: a one-line explanation appears and follows the column under the mouse, also after reordering columns; the header's right-click menu still works.
13. Dark mode: the Changes list, its header, the summary and the grey ages are readable. Resize narrow and wide: the summary truncates with an ellipsis and the checkbox stays right-aligned.
14. File → Compare Folder Ledgers with two ledgers: the dialog shows rows with colours (the dialog check from the ledger feature).
15. Scan `C:\` and press Stop within 2 seconds: no new snapshot is written.
16. Installed DeepServer build: right-click one folder → **Compare in DeepServer**, then another → Compare in DeepServer again: DeepServer opens Folder Compare with the two. Select two folders and use it once: same result. Portable engine without `DeepServer.exe` beside it: the item is greyed out.

## Known gaps / next steps

1. **Code signing.** Without a certificate, SAC-enabled PCs can't run our builds.
2. **Help → manual** still opens upstream's wiki. That's accurate for the features. Help → Report Bug already opens this repo's issues.
3. **Folder ledger follow-ups:**
   * translations of the `IDS_LEDGER_*` strings
   * a CLI compare mode (e.g. `/compareledger base.ledger.csv`) for scheduled change reports
   * a UI-automation test for the compare dialog
4. Possible fork-specific addition: a simpler "beginner" mode, if there's demand. Everything else comes from upstream.
