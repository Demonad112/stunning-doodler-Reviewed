# DeepServer v1.0 — merge OpenDiff + altWinDirStat into one installable app

## Context
the owner has two working Windows tools in separate repos:
- **OpenDiff** (Tauri 2 + Vue 3 + Rust, Apache-2.0): compare, merge and sync files/folders. Local checkout `%USERPROFILE%\Projects\Work\ODiff` (branch `stage0-smoke-fixes`, PR #4 open).
- **altWinDirStat** (C++ WinDirStat 2.x fork, custom `UiFramework`, not MFC; GPL-2): disk-usage treemap, folder ledgers, `/compare`. Newest work, the `feature/change-tracking` branch (Task 1 of 6 done), exists **only in a temp scratchpad clone and is not pushed**.

Goal: one product, **DeepServer 1.0**, in the empty repo `github.com/Demonad112/stunning-doodler-Reviewed` (local root `%USERPROFILE%\Projects\Work\DeepServerV1.0`). One `setup.exe`, built on GitHub Actions, that installs on Windows 10/11 workstations and Windows Server, with easier point-and-click UI.

Decisions (from Q&A):
- **One app with a bundled engine.** The OpenDiff shell becomes DeepServer. altWinDirStat ships inside it as the "Disk Usage" engine, launched from the app. Nothing is rewritten.
- The name is **DeepServer**, version 1.0.0.
- There is no third app.
- **Fresh copy, no git history.** The old repos stay as archives.

Rule for all batches: **reuse, don't rebuild.** Existing features, the change-tracking plan and the Transfer Monitor plan are carried over and finished. Nothing is re-derived.

---

## Repo layout (OpenDiff stays at the root so its pnpm/husky/eslint config keeps working)
```
DeepServerV1.0/            (git root → stunning-doodler-Reviewed)
  src/  src-tauri/  tests/  scripts/  public/  assets/  ...   ← OpenDiff (copied)
  native/diskusage/        ← altWinDirStat source (copied from the scratch clone, branch feature/change-tracking)
  docs/history/            ← the 3 handoff/reference files now in the root, plus the change-tracking spec/plan and the Transfer Monitor plan
  LICENSE (Apache-2.0), native/diskusage/LICENSE.md (GPL-2), NOTICE (credits kygo8/open-diff + WinDirStat, source-offer note)
  .github/workflows/       ← merged, Windows-only
```
Excluded from the copy: `.git`, `node_modules`, `installer/`, `installer2.zip`, `handoff/` (moved to docs/history if useful), `target/`, `build/`, `.superpowers/` (ledger copied to docs/history), and altWinDirStat's `sync-upstream.yml` and `maintenance.yml`.

---

## Errors and issues found during the survey (each is fixed in the batch shown)
| # | Where | Issue | Batch |
|---|---|---|---|
| E1 | altWinDirStat scratch clone | The only copy of the change-tracking branch is in `%TEMP%` and can be deleted at any time | 0 |
| E2 | root `.gitattributes` (`eol=lf`) | Would convert the C++/`.rc`/BOM files in native/ to LF. Add a `native/** -text` override | 0 |
| E3 | `.prettierignore`, eslint, stylelint, husky | The pre-commit hook would lint and format native/ and docs/history. Add them to the ignores | 0 |
| E4 | `native/.../windirstat.vcxproj:41` | Asks for toolset v145 (VS2026). VS2022 builds fail without `/p:PlatformToolset=v143`. Set a default of v143 (conditional) | 2 |
| E5 | altWinDirStat `WinDirStat.cpp:388`, `HelpersTasks.cpp:368` | When auto-elevate is on, the launched process exits with 0 and an elevated copy does the work. DeepServer would read a false success on headless runs. Add a `/noelevate` flag | 2 |
| E6 | `ForkCli.cpp:31-33` | `/compare` throws away its error text (GUI exe, no console). Write errors to `<out>.err` | 2 |
| E7 | `Ledger.cpp:276,290-292,355` | Folders whose names differ only by letter case collapse to one key, so rows are dropped or doubled | 3 |
| E8 | `Ledger.cpp` Load/Save | No check for a missing or duplicate `.` root row, and Save isn't atomic. Write via `.tmp` + rename | 3 |
| E9 | `Ledger.cpp:324-335` | The comparison CSV's `Change` column is localized. Write fixed English codes (`Added/Removed/Grown/Shrunk`) and localize only in the UI | 3 |
| E10 | `tests/Test-ForkChanges.ps1` | Never run in CI. In PS 5.1 `WaitForExit(ms)` can leave `$null` exit codes. Arguments aren't escaped. The exit-code checks can't tell old from new. Use `WaitForExit()` without a timeout, quote args properly, and drop the weak checks | 3 |
| E11 | `folder-core/src/lib.rs:1327-1406` | The scanner stops on the first unreadable entry (for example `System Volume Information`). Collect per-entry errors and continue | 5 |
| E12 | `sync-core/src/lib.rs:258-265` | Sync reads each **whole file into RAM**, with no progress. Switch to the new streaming copy from transfer-core | 5 |
| E13 | `commands.rs:4393,4420` | The recursive copy runs `remove_dir_all` on the target first (data-loss risk), then uses plain `fs::copy`. Route it through transfer-core; never delete the target first | 5 |
| E14 | `installer-hooks.nsh:10-20` | Hardcodes `open-diff-app.exe`. Use `${MAINBINARYNAME}` | 1 |
| E15 | `installer-hooks.nsh` + `shell-core/src/lib.rs:268-289` | The in-app "register shell extension" writes HKCU, but uninstall removes only HKLM, leaving dead menu entries. Uninstall also deletes the HKCU keys, and the in-app toggle is hidden when the installer has already registered them | 1 |
| E16 | `installer-hooks.nsh:22-23` | Always creates a second desktop shortcut and ignores the user's choice. Remove it and use Tauri's own shortcut option | 1 |
| E17 | `tauri.conf.json:31` | The MSI target doesn't run the NSIS hooks (no menu, no cleanup). Ship NSIS only | 1 |
| E18 | 14 files, 38 lines | `kygo8` identity: bundle id, homepage, `appMeta.ts` URLs, `package.json`, the issue template, `packagingConfig.test.ts`, WiX `upgradeCode` | 1 |
| E19 | `tauri.conf.json` WebView2 = download bootstrapper | **Offline servers can't install it.** Use `offlineInstaller` for the server build (see Batch 6) | 6 |
| E20 | `release.yml` | A manual run with no tag uses the branch name as the tag. It also builds for macOS and Linux (wasted minutes). Make it Windows-only and require a tag | 6 |
| E21 | `prune-storage.yml` | Runs with `actions: write` on PR events, which is risky and fails on fork PRs. Make it manual-only | 6 |
| E22 | `SettingsView.vue:2499,2524,3193,3426,3465` | Executable and folder paths can only be typed. Add Browse buttons | 4 |
| E23 | altWinDirStat design gap | Changing exclusion filters between scans shows up as "Removed". Store a filter fingerprint in the snapshot and warn when it differs | 3 |
| E24 | altWinDirStat snapshots | 5 snapshots per location × large drives or UNC shares can reach hundreds of MB and nothing reports it. Show the total size, add "Clean up history" and a cap setting | 3 |

Licensing: Apache-2.0 (the app) and GPL-2 (the engine) ship as **separate programs in one installer**, which counts as aggregation and is allowed. The public repo provides the GPL source. The NOTICE file and the About box state both licenses.

---

## Batch 0: Rescue and seed the repo (small, do first)
1. Push `feature/change-tracking` from the scratch clone to `Demonad112/altWinDirStat` as a backup branch (E1). Skip the four BOM-only files.
2. In DeepServerV1.0: run `git init`, add remote `stunning-doodler-Reviewed`, copy OpenDiff (working tree at `24406b0`, which already has PR #4's fixes) and altWinDirStat into the layout above, and move the 3 root reference files to `docs/history/`.
3. Fix the ignores and line-ending rules (E2, E3). Add `NOTICE` and `README.md` covering what DeepServer is, how to install it and the licenses.
4. Run `corepack pnpm install`, then `pnpm test:unit` and `cargo test --workspace` locally to confirm nothing broke in the move. Commit "Import OpenDiff 1.2.0 + altWinDirStat change-tracking (fresh copy)" and push `main`.
5. Old repos: left untouched. All DeepServer work (branches and PRs) happens only in `stunning-doodler-Reviewed`; `open-diff` and `altWinDirStat` are read-only sources.

## Batch 1: Identity and installer
- `tauri.conf.json`:
  - `productName` "DeepServer", `version` 1.0.0, `identifier` `com.demonad112.deepserver`. This is a new app-data location, so existing OpenDiff settings don't carry over (acceptable, no migration).
  - Homepage points at the new repo. Targets `["nsis"]` only (E17).
- Rename the main binary to `deepserver` and the CLI to `deepserver-cli`, and update every reference.
- Replace every `kygo8` URL (E18), including `src/app/appMeta.ts`, `package.json` and `packagingConfig.test.ts`. Rename the `OpenDiff` strings in the UI, i18n (8 locales) and window titles.
- `installer-hooks.nsh`:
  - use `${MAINBINARYNAME}` (E14) and clean up HKCU on uninstall (E15); drop the forced shortcut (E16)
  - one cascading Explorer menu, **DeepServer ▸**, with: Compare…, Select as left side, Analyze disk usage, Copy with verification (Batch 5)
  - on install, detect an existing OpenDiff and offer to uninstall it, so there are no duplicate menus
- Test: update `packagingConfig.test.ts` to assert the new identity.

## Batch 2: Build and bundle the disk-usage engine
- `native/diskusage`:
  - make v143 the default toolset (E4)
  - add a `/noelevate` flag in the fork CLI path (E5) and write `/compare` errors to `<out>.err` (E6)
  - change the product/exe name to `DeepServer Disk Usage`, and move the registry root to `HKCU\Software\DeepServer\DiskUsage`
- CI job `build-engine` (windows-2022): msbuild x64 Release with `/DPRODUCTION=1`, run `Test-ForkChanges.ps1`, upload `diskusage.exe`.
- Tauri bundling:
  - copy it to `src-tauri/binaries/deepserver-diskusage-x86_64-pc-windows-msvc.exe` and declare it in `bundle.externalBin`
  - launch it from Rust with `std::process::Command`, reusing the pattern in `open_path_external`, `commands.rs:2643`. No shell plugin is needed.
- New Rust commands in a small `diskusage-core` crate:
  - `diskusage_open(path)`: GUI, allowed to elevate
  - `diskusage_snapshot(path) -> ledger path`: `/saveto x.ledger.csv /noelevate`, headless, waits for exit
  - `diskusage_compare(a,b,all) -> rows`: calls `/compare` and parses the CSV with the `csv` crate (strip the BOM)
  - `list_drives()`: fixed, removable and network drives, with label, free and total space

## Batch 3: Finish altWinDirStat change tracking (Tasks 2–6 of the existing plan)
- Carry out `docs/history/2026-09-29-change-tracking.md` Tasks 2–5 **as written**. Task 1 is done; don't repeat it. Confirm the plan's flagged `UiFramework` API guesses (`CButton::GetCheck`, the `Create(LPCWSTR…)` constructors, `GetNameView()`, the `WM_CTLCOLOR` route) against `UiFramework.h` first.
- Fold in fixes E7–E10, E23 and E24.
- Adapted Task 6: the default layout change stays; the CI step goes into the DeepServer `build-engine` job; HANDOFF.md goes to `native/diskusage/FORK-DIFF.md`; no separate altWinDirStat release.
- Snapshot folder moves to `%LOCALAPPDATA%\DeepServer\History\...`, so DeepServer's Rust side can list the snapshots.
- Do the manual Compare-dialog check (the missed Task 1 step 8) here.

## Batch 4: Unified, easier UI (the "work together" batch)
- **Home screen**, regrouped into 3 tile groups: *Compare* (existing modes), *Copy & Sync* (Folder Sync + Transfer Monitor), and *Disk* (Disk Usage). Tiles open with pickers, not empty text boxes.
- **New "Disk Usage" mode**, added with the known 11-file checklist (`types/session.ts`, `sessionCatalog.ts`, `sessionFactory.ts`, `router.ts`, `HomeView.vue`, `commandRegistry.ts`, `AppLayout.vue`, `shellChrome.ts`, `statusBarPhrases.ts`, 8 locales, `session-core` enum). The view has:
  - a **Location dropdown** (drives from `list_drives`, recent locations, and a network share entered once), a **Browse…** button (reusing `pickNativePath`, `src/app/filePicker.ts:52`) and "Open treemap"
  - **What changed**: two dropdowns (*Baseline snapshot* / *Current*, listed by date) and a sortable table built from `diskusage_compare`, a "Show all" toggle, a "Take snapshot now" button, and "Compare these two folders" on any row, which opens Folder Compare
- **Cross-launch:**
  - the engine gets built-in fork menu items (`ForkCommands.cpp`, which already exists): *Compare in DeepServer*, *Copy with verification*. They run `deepserver.exe --compare-left <path>` and so on, using the same argument form as the existing `OpenDiffSelectLeft` handling.
  - Folder Compare gains *Analyze disk usage of this side*
- **No typed paths:** add Browse buttons to the 5 Settings fields (E22). Offer an OS-detected dropdown for git/VS Code paths.

## Batch 5: Transfer Monitor (existing approved plan, re-homed)
- Carry out stages 1–4 of `docs/history/transfer-monitor-plan.md`, copied from `%USERPROFILE%\.claude\plans\c-users-addy7-projects-work-odiff-hando-bubbly-ritchie.md`, as designed:
  - `transfer-core`, which classifies Windows errors, runs a pre-flight check, streams copies to `.odpart` files then renames them, handles `\\?\` long paths, keeps mtimes, retries, supports optional BLAKE3, and emits `transfer://` events
  - a Copy-mode UI, a Watch mode and the recovery folder
  - `ReportKind::Transfer` HTML/CSV reports
- **Failed files grouped:** the "Not copied" panel groups failures by reason (Access denied / In use / Disk full / Path too long / Network lost …). Each group has a count and total size, and actions *Retry group*, *Copy group to recovery folder*, *Reveal*, *Export list*.
- The pre-flight check shows source size against destination free space, using `list_drives`.
- Fix E11, E12 and E13 here, since transfer-core replaces the old copy paths.
- Stage 5 extras (resume, CLI, bandwidth limit) are deferred.

## Batch 6: Release and server rollout
- Workflows:
  - `ci.yml`: runs on PRs; quality checks, tests, the engine build and the engine tests
  - `release.yml`: Windows only, runs on a `v*` tag, requires the tag (E20). It builds the engine, then `tauri build`, then attaches to a GitHub Release `DeepServer_1.0.0_x64-setup.exe`, the portable zip and `SHA256SUMS`.
  - `prune-storage.yml` becomes manual-only (E21), and the old `windows-installer.yml` is deleted.
- **Two installer flavours**, both from the same build:
  - *standard*: WebView2 bootstrapper, small
  - *offline/server*: `webviewInstallMode: offlineInstaller`, about 130 MB larger, for air-gapped servers (E19)
- Document silent install for servers: `DeepServer_1.0.0_x64-setup.exe /S`. Supported: Windows 10/11 and Server 2016+ with Desktop Experience. Not supported: Server Core, which has no GUI.
- Signing: keep the optional Azure Trusted Signing / PFX secret hooks from altWinDirStat's `build.yml`. Until they're set up, SmartScreen will warn (this is documented).

---

## Verification (each batch ends green before the next starts)
- **Batch 0:** `pnpm test:unit`, `cargo test --workspace` and `pnpm quality` pass in the new repo; `git status` is clean; the push is on GitHub.
- **Batch 1–2:** the CI artifact installs on this PC. Start-menu "DeepServer" launches; the Explorer ▸ DeepServer menu works; "Analyze disk usage" opens the engine. Uninstall removes the menus (HKLM and HKCU) and the files. Run `Test-ForkChanges.ps1` (including `/noelevate` exit codes).
- **Batch 3:** fork tests run in CI. Manual checklist from the plan: Compare dialog, Changes tab, ages, tooltips, dark mode (the owner clicks through, or computer use is granted for the dev exe).
- **Batch 4:** vitest for the new mode and locale completeness (`languageSkeletons.test.ts`). Manual: pick a drive from the dropdown, take a snapshot, change a file, snapshot again, and check that What changed lists it.
- **Batch 5:** transfer-core unit tests with injected Windows errors (locked file, read-only destination, path >260 chars, a full USB stick). E2E: copy a folder containing one locked and one access-denied file, and check that the panel shows 2 groups, that retry and recovery work, and that the report exports.
- **Batch 6:** tag `v1.0.0-rc1`, then check the Release assets. Silent-install the offline flavour on a Windows Server VM or Sandbox, and uninstall cleanly.

Each batch is one PR to `main` in the new repo, so progress can stop and resume safely between batches.
