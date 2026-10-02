# DeepServer Disk Usage engine: notes for Claude

**What this folder is:** a fork of WinDirStat 2.x (C++, Windows only; custom `UiFramework`, not MFC; GPL-2), bundled in DeepServer as a separate program, `deepserver-diskusage.exe`. It started as the standalone altWinDirStat fork, whose repo is now read-only. Read `HANDOFF.md` first. It holds the fork-diff tables (altWinDirStat, DeepServer, and per-batch), the manual checklist and known gaps.

## Layout
- `windirstat/`: app sources (`windirstat.vcxproj`). The pre-build PowerShell scripts are in `windirstat/Build/`; `Compress Strings.ps1` re-sorts `res/fork/lang_en.txt` and generates `res/LangStrings.h` (untracked).
- Fork-owned sources: `Fork*.cpp/.h`, `Ledger`, `History`, `ForkSettings.h`, `Views/FileChangesView`, `Dialogs/LedgerCompareDlg`, `res/fork/lang_en.txt`.
- `tests/Test-ForkChanges.ps1`: DeepServer's fork tests (Windows PowerShell 5.1; the GUI checks use a portable copy, so no registry writes).
- `tests/Test-WinDirStat.ps1`, `.github/scripts/Stress-LargeScan.ps1`: upstream suites (PowerShell 7.6+).
- Inert here, kept for upstream merges: `.github/workflows/build.yml`, `installer/altWinDirStat.iss`, `setup/`, `docs/releases/`.

## Build and test (from `native/diskusage`)
```powershell
& "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -SkipAutomaticLocation
msbuild windirstat.sln /m /p:Configuration=Release /p:Platform=x64 "/p:ExternalCompilerOptions=/DPRODUCTION=1"
powershell -NoProfile -ExecutionPolicy Bypass -File tests\Test-ForkChanges.ps1 -ExePath build\altWinDirStat_x64.exe   # -SkipGui for headless only
```
- The output is still `build\altWinDirStat_<arch>.exe`. DeepServer CI (`.github/workflows/build-windows.yml`) copies it to `src-tauri/binaries/deepserver-diskusage-x86_64-pc-windows-msvc.exe`.
- The toolset defaults to v143 (VS2022) and switches to v145 under VS2026.
- DeepServer's end-to-end check against this exe runs from the repo root: `DISKUSAGE_ENGINE=<exe> cargo test -p diskusage-core --manifest-path src-tauri/Cargo.toml -- --ignored`. It covers snapshot → change → snapshot → compare.
- The upstream suites and the stress test need pwsh 7.6+, so they run in CI (`engine-upstream-tests` job). The `Ui` suite is left out because it waits for windows titled "WinDirStat", and this build's title is "DeepServer Disk Usage".

## Rules
- **Keep the upstream diff small.** Prefer fork-owned files and appended blocks, and add every upstream edit to the tables in `HANDOFF.md`. Fork IDs use ranges upstream doesn't: 900, 1900–1909, 33900–33906.
- **Settings live under `HKCU\Software\DeepServer\DiskUsage`**, section `DeepServer`. Never write to the WinDirStat or altWinDirStat keys.
- **`IDS_APP_TITLE` stays "WinDirStat"**, because CSV headers use it. The visible name comes from `strWinDirStat` in `Constants.h`.
- **Ledgers**, whether saved by `/saveto x.ledger.csv` or as automatic snapshots:
  - header, then exactly one `.` root row, then trailing `#`-prefixed metadata such as `#filters=`
  - readers have to skip `#` lines
- **Snapshot history** is `%LOCALAPPDATA%\DeepServer\History\<FNV-1a-64 of the location key>\`; `DEEPSERVER_HISTORY_DIR` overrides it. It must stay in step with `src-tauri/crates/diskusage-core`.
- **Headless runs DeepServer starts pass `/noelevate`.** `/compare` writes `<out>.err` on failure and `<out>.warn` when the filters differ.
- **Versions:**
  - the version is `ALT_VER_*` at the end of `windirstat/Version.h` (default 1.0.0)
  - releases come from DeepServer's `release.yml`, not engine tags
  - the 2014–2016 legacy code exists only at the old repo's tags
