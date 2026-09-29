> This file was compiled by an assistant from a prior conversation on this project. Treat it as background material, not verified fact. Reason independently before relying on anything here, especially items presented as correct, decided or working. It's meant to bring a new collaborator up to speed, not to define how they should proceed.

# Reference: altWinDirStat (as of 2026-09-29)

## Quick Recap

altWinDirStat (`github.com/Demonad112/altWinDirStat`) is an unofficial, rebranded fork of **official WinDirStat 2.x** (C++, upstream's own MFC-replacement UI framework, Windows only).

- **Releases this session** (all verified: assets, SHA-256 checksums, exe FileVersion):
  - **v2.0.0**: first 2.x-based release.
  - **v2.0.1**: fork Report Bug link, and a Cleanups button-layout fix ported from upstream.
  - **v2.1.0**: new **folder ledger** feature. v2.1.0 is GitHub's "Latest".
- **Folder ledger:**
  - Export a folder-only CSV snapshot of a scan.
  - Compare a baseline ledger against a later scan, or against another ledger, in a side-by-side, colour-coded dialog.
- **CI covers:**
  - x64, Win32 and ARM64 builds
  - smoke tests
  - 7 upstream test suites (916 checks)
  - an installer test
  - a large-scan stress test (700k-file tree plus all of `C:\`, probing for UI hangs)
  - a guard against upstream publishing files coming back
- **Releases** are cut by manually running the Build workflow with `release_tag`.
- **Master:** at `bf40619` (PR #14 merged); the default version is 2.1.1.

All work so far ran in a Linux cloud session that could not compile or run the app. Everything Windows-side was verified only through GitHub Actions. **The ledger compare dialog has not been driven by automation, and this conversation never confirmed a manual test of it.**

## Working Context

### History in one paragraph
- **Legacy code:** the repo began as a 2014–2016 WinDirStat fork, revived and released as v1.0.0. That code scanned on the UI thread and froze on large drives.
- **PR #5:** replaced master with official WinDirStat 2.8.8 via a history-preserving merge, then rebranded it.
- **This session:**
  - fork-owned version numbering
  - manual-run releases with checksums
  - CI hardening (stress test, more upstream suites, fork-guard)
  - two upstream fixes ported
  - the Report Bug link redirected
  - the folder ledger feature
  - a maintenance workflow
  - three releases

### Design principle carried through the session
**Keep the fork diff small so upstream merges stay easy.**
- Fork features go in fork-owned files.
- Upstream files get only one-line hooks, or appended blocks.
- Every fork change is listed in the table in `HANDOFF.md`.

The weekly `sync-upstream.yml` workflow merges upstream master into a PR. It hasn't been observed running successfully yet, and it needs the repo setting "Allow GitHub Actions to create and approve pull requests", whose state is unknown.

### Versioning / release flow
- **Version source:** the fork version is `ALT_VER_MAJOR/MINOR/PATCH`, in a block appended to `windirstat/Version.h`. It `#undef`s upstream's `PRD_*`.
- **Release stamping:** release builds pass `/p:AltVerMajor=… /p:AltVerMinor=… /p:AltVerPatch=…`. The fork-owned `project.early.props` (auto-imported by the vcxproj) defines `ALT_VER_*` for both `cl` and `rc`, because `ExternalCompilerOptions` only reaches `cl`.
- **Safety check:** CI asserts that the exe's FileVersion equals the expected version.
- **Release:** add `docs/releases/vX.Y.Z.md` → merge → **Actions → Build → Run workflow** with `release_tag=vX.Y.Z`. A `-suffix` tag makes a prerelease.
  - The release job depends on fork-guard, build and stress.
  - It creates the tag, uploads the 6 zip/Setup files plus `SHA256SUMS.txt`, and uses the notes file as the body.
- **After a release:** bump the `ALT_VER_*` default (see `CLAUDE.md`).

### Why some things are the way they are
- **Tag operations via workflows:** the cloud session's git proxy blocked tag pushes and deletions. That's why releases are cut by manual workflow runs and why `maintenance.yml` exists (it deletes a *prerelease* and its tag, and refuses final releases). A local session probably doesn't have this limitation.
- **Separate fork strings file:** fork UI strings live in `windirstat/res/fork/lang_en.txt`. Upstream's `Build/Compress Strings.ps1` globs `res\lang_*.txt` recursively, so the file is picked up with no upstream edits.
  - Upstream's Settings suite enforces identical keys across all 25 `res/langs/lang_*.txt` files, and keeping the fork strings separate avoids touching them.
  - Other languages fall back to English for the `IDS_LEDGER_*` keys.
- **Fork resource IDs:** these live in `ForkResource.h`, in ranges upstream doesn't use (dialog 900, controls 1900–1906, commands 33900–33901). The ledger dialog template is in `Fork.rc`, which has no BOM (like `windirstat.rc`).
- **Settings suite not run:** upstream's Settings suite rebuilds the solution without `/p:PlatformToolset=v143`, so it can't run on the VS2022 runners.

### Known state of repo settings / external items
- **Issues:** enabled (the user did this); Help → Report Bug opens `/issues/new`.
- **Code signing:** not configured, so builds are unsigned. **Smart App Control blocks unsigned exes, including locally built ones.**
- **Stale branches:** `Problem-commit`, `attempt-removal-of-ConcRT-deps` and `fix-blank-header-ctrl` (2014–2016 legacy), and `claude/no-command-line` (a legacy-code feature). None is merged; all were deliberately kept.

## Deep Reference

### Local build and test (Windows)
```powershell
git clone https://github.com/Demonad112/altWinDirStat.git
cd altWinDirStat
# VS2022 (v143). With VS2026, drop the toolset override.
msbuild windirstat.sln /m /p:Configuration=Release /p:Platform=x64 /p:PlatformToolset=v143
# -> build\altWinDirStat_x64.exe (CI copies it to altWinDirStat.exe for zip/installer)

# Upstream suites as CI runs them (needs PowerShell 7.6+; for the Win32 exe use a 32-bit pwsh, since the Ui suite needs matching bitness)
pwsh -File tests\Test-WinDirStat.ps1 -ExePath build\altWinDirStat_x64.exe -Only Filtering,Cli,EdgeCases,Permissions,Enumeration,Unc,Ui

# Large-scan stress test (generates ~700k files under $env:RUNNER_TEMP or %TEMP%; default also scans all of C:\)
pwsh -File .github\scripts\Stress-LargeScan.ps1 -ExePath build\altWinDirStat_x64.exe            # full
pwsh -File .github\scripts\Stress-LargeScan.ps1 -ExePath build\altWinDirStat_x64.exe -SkipDriveScan -GridTop 20
```
- **Watch out for 8.3 temp paths:** on the runner, `GetTempPath()` returned an 8.3 short path (`RUNNER~1`), which broke the root-row match in the stress script. The script prefers `$env:RUNNER_TEMP` for that reason. A local `%TEMP%` may be a short path too.
- **Smart App Control:** it may block the freshly built exe. The usual workarounds are a VM or Windows Sandbox, or turning SAC off (which can't be turned back on without a reset).

### Manual test checklist for the folder ledger (never done via automation)
1. Scan a folder → **File → Export Folder Ledger...** → save `base.ledger.csv`.
2. Add a folder, delete a folder, grow a file, then press F5.
3. **File → Compare with Folder Ledger...** → pick `base.ledger.csv`. You should see:
   - coloured Added, Removed and Grown rows
   - a summary line with counts and net size/files
4. Try:
   - the filter
   - Show unchanged
   - sorting every column
   - Export Comparison (CSV)
   - double-click to open in Explorer
   - resizing the dialog
   - dark mode
5. With no scan open, the compare command should prompt for a second ledger.
6. Scan all of `C:`, then compare against a `C:\SomeFolder` baseline. It should narrow to that folder automatically.
7. Headless: `altWinDirStat.exe /saveto X:\b.ledger.csv X:\Folder` should write a ledger with no window.

### Folder ledger: files and behaviour
| File | Role |
|---|---|
| `windirstat/Ledger.h/.cpp` | `FromScan` (walks `IT_MYCOMPUTER/IT_DRIVE/IT_DIRECTORY` by exact `GetItemType()`), `Save`, `Load`, `Compare`, `SaveComparison`, `ChangeName`, `IsLedgerPath` |
| `windirstat/ForkCommands.cpp` | `CWinDirStatModel::OnFolderLedgerExport / OnUpdateFolderLedgerExport / OnFolderLedgerCompare` |
| `windirstat/Dialogs/LedgerCompareDlg.h/.cpp` | Owner-data list: custom-draw colours, sort, filter, export, double-click opens folder |
| `windirstat/ForkResource.h`, `windirstat/Fork.rc`, `windirstat/res/fork/lang_en.txt` | IDs, dialog template, strings |
| Hooks in upstream files | `windirstat.rc` (include plus 2 File-menu items), `WinDirStatModel.h` (include, 3 declarations, 3 routes), `CsvLoader.cpp` (1-line `*.ledger.csv` hook in `SaveResults`), `windirstat.vcxproj` (separate `ItemGroup`) |

**Ledger CSV format:** UTF-8 with BOM, CRLF, and this fixed English header (not localized, so ledgers stay comparable across UI languages):
```
Path,Relative Path,Size (bytes),Files,Subfolders
"D:\Data",".",2662685730,700001,6323
"D:\Data\grid","grid",...
```
- **Sizes and counts:** size is logical size. Files and subfolders are recursive totals from `CItem::GetFilesCount/GetFoldersCount`. The recursive semantics were inferred, and the stress test's root row matching `folders = total - 1` is consistent with them.
- **Matching:** `Compare` matches folders by lower-cased relative path.
  - If one root contains the other, the broader snapshot is narrowed in place (`Rebase`).
  - Unrelated roots are matched by relative path.
- **Change classes:** Added, Removed, Grown, Shrunk, FilesChanged (same size, but the file or folder count differs), Unchanged.
- **Summary deltas:** taken from the root (`.`) row.

**Logic tests:**
- 17 checks passed on Linux, using a g++ harness with shimmed Win32 functions and a fake `CItem`. The harness lived only in the cloud scratchpad and **isn't in the repo**.
- On Windows, CI's stress job checks a real ledger: 6,324 folder rows for the synthetic tree, with exact root totals.

### CI (`.github/workflows/build.yml`) jobs
- **fork-guard:** fails if `publish-*-to-winget-pkgs.yml`, `.github/FUNDING.yml`, `setup/chocolatey/` or `setup/store/` exist.
- **build** (x64, Win32, ARM64; windows-2022), step by step:
  1. compute version
  2. msbuild
  3. FileVersion assertion
  4. optional signing (Azure Trusted Signing or .pfx secrets)
  5. portable zip
  6. smoke plus screenshots (a title-branding check, and a check that nothing is written under `HKCU\Software\WinDirStat`)
  7. upstream suites (Win32 downloads x86 pwsh)
  8. Inno installer build and silent install/uninstall test
- **stress** (x64): `Stress-LargeScan.ps1`
  - headless `/saveto` correctness (every file listed, exact root totals)
  - ledger export check
  - GUI scans of the synthetic tree and of `C:\`
  - `SendMessageTimeout(WM_NULL)` probes every 250 ms, failing at 5 s or more
  - scan completion detected from the title pattern `" - NN% "` / `" - Scanning"`
- **release:** runs only on a tag push or a manual run with `release_tag`. Needs fork-guard, build and stress.

### Measured results (CI, windows-2022, x64)
| Scenario | Result |
|---|---|
| Headless `/saveto`, 700,001 files | ~2.5 s, ~280k items/s, exact totals |
| Ledger export, same tree | 6,324 rows, 0.7 s, 571 KB |
| GUI scan, synthetic tree | ~1 s, worst UI probe 1–89 ms, peak 123 MB |
| GUI scan, all of `C:\` (MFT, elevated) | ~18 s, worst UI probe 196–417 ms, peak ~690 MB |

### Code-review notes on freeze risk (2.x scan pipeline)
- **Threading:** scanning runs on a wrapper `std::jthread` plus per-volume `BlockingQueue` worker pools. The UI thread only runs a 25 ms timer, which updates progress and re-sorts visible rows about every 375 ms.
- **Stop/abort/rescan:** these wait with `CWinApp::RunTaskWithUiUpdates` (which pumps messages), so a worker's `SendMessage` to the UI can't deadlock.
- **Post-scan work:** finalize sort, hardlink adjustment (`std::execution::par`) and extension stats all run on the worker. The treemap draws into a cached DIB, and its layout is bounded by pixels.

### PR / release log (this session)
| PR / release | Content |
|---|---|
| #7 | Fork version stamping, manual-run release, SHA256SUMS, CLAUDE.md |
| v2.0.0 | Released (via the `v2.0.0-rc.1` trial, now deleted) |
| #8 | Default version bumped to 2.0.1 |
| #9 | fork-guard, legacy-registry audit (no code change needed) |
| #10 | Stress test, upstream suites, Report Bug link, ported Cleanups-button and test fixes |
| #11 / v2.0.1 | Release notes / release |
| #12 / v2.1.0 | Folder ledger / release |
| #13 | Default version bumped to 2.1.1 |
| #14 | `maintenance.yml` plus HANDOFF refresh; `v2.0.0-rc.1` deleted with it |
