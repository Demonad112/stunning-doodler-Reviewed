# Plan: Transfer Monitor for OpenDiff (plus smoke test + rollout)

## Context
The user's fork of OpenDiff (Tauri 2 + Vue 3 + Rust) now installs on Windows. The next step is a feature for client file transfers. It should show live what arrives at a destination, list every file that did **not** make it together with the reason, and make those files easy to recover without searching by hand. Every run should also leave a report that can be handed to a client.

Decisions from Q&A:
- **Two modes.** In *Copy* mode, OpenDiff copies the files itself. In *Watch* mode, another tool (Explorer, robocopy, …) does the copying; the user still picks the source folder, and OpenDiff checks the destination against it.
- **Targets:** local disks, USB and network shares. Cloud placeholder folders are out of scope.
- **Scale:** mixed and unknown. The design must handle 100k+ files.
- **Failed files:** a "Not copied" panel with one-click actions. Nothing is copied automatically.
- **Verification:** size + modified time by default, with an optional content hash.
- **Copy only.** No move and no deletes.
- **Client report:** an exportable audit report is required.
- **Order:** finish the smoke test first. Install Rust on the **dev PC only**; clients still get the prebuilt installer.

## What exists (from the code survey) and what's missing
**Reuse:**
- `snapshot-core`: `SnapshotEntry` has path, size, mtime and an unused `content_hash`; `scan_directory_snapshot`, `SnapshotComparer` → source manifest and diff.
- `folder-core`: scanner (`scan_local_folder_with_options`), `FileFilters`/`wildcard_matches`, `calculate_crc32`.
- `sync-core`: `SyncExecutionItemResult`, `SyncExecutionStatus`.
- `job-core`: `JobStatus`, `JobProgress`, `CancellationToken`.
- `report-core`: `UnifiedReport` plus HTML/CSV/JSON renderers.
- `logging-core`: `StructuredLogEvent`.
- Frontend:
  - `src/api/integration.ts` → `revealPathInOs`, `openPathExternal`
  - the unused `src/stores/jobs.ts` and `src/components/jobs/JobsProgressPanel.vue`
  - the virtual list pattern in `src/views/FolderCompareView.vue` (~L589–607)
  - `src/app/compareCompleteNotify.ts`
  - `StructuredErrorPanel.vue`

**Missing (must build):**
- **Progress to the UI.** No Rust→UI events exist; every command is synchronous, and the only `listen` is `desktopDrop.ts`.
- **Background jobs.** No job registry and no background threads.
- **File watching.** The `notify` crate is not a dependency.
- **Error detail.** No OS error-code classification: `VfsError` only has NotFound/Readonly/Io(String).
- **Windows path and file handling.** No `\\?\` long-path support, no retry, and copies aren't verified.
- **Sync copy path.** The app's sync copies with plain `fs::copy` (`commands.rs` ~L4396) and reports no progress.
- **Scanner errors.** Any unreadable entry aborts the whole scan instead of recording a per-entry error.

---

## Stage 0: dev environment + smoke test (first)
1. Install `rustup` (stable MSVC; VS 2022 is already present). Check that `corepack pnpm tauri:dev`, `cargo test --workspace` (in `src-tauri/`) and the husky pre-commit hook all run.
2. **Text Compare "Comparing…" hang:** reproduce in `tauri:dev` with the small sample files, then find the root cause with the systematic-debugging approach.
   - Start from the Text Compare view's run handler and the `compare_text`-style command in `commands.rs`.
   - Fix it with a regression test.
3. **Check the other modes by hand:** Text Merge/Edit, Hex, Media, Picture, Registry, Table, Version, Folder Merge/Sync, sessions, HTML report, Explorer right-click Select Left → Compare, the Settings shell toggle, and uninstall.
   - Record the results in `docs/smoke-test-windows.md`.
   - File an issue for each failure found; fix only blockers.
4. Settle the `folderChromeDensity.test.ts` 14 vs 16 px question by checking the upstream git history for that view.

## Stage 1: backend foundation (new crate `src-tauri/crates/transfer-core`)
- **Manifest:** a source scan built on the snapshot-core/folder-core scanner, changed to **record per-entry errors instead of aborting** (unreadable folders become manifest rows with a reason).
  - Rows are streamed to a JSONL file in the run folder so memory stays flat at 100k+ files.
- **`FailureReason` enum** with a `classify(io::Error, Side)` function built on `raw_os_error()`:

  | Reason | Windows error codes |
  |---|---|
  | AccessDenied | 5 |
  | FileLocked | 32, 33 |
  | DiskFull | 39, 112 |
  | PathTooLong | 206; also detected before copying from the path length |
  | InvalidName | 123, plus reserved names and trailing dot/space found by pre-flight |
  | NetworkLost | 53, 64, 67, 1231 |
  | DeviceRemoved | 21, 1117 |
  | MediaError | 23 |
  | SourceVanished | 2/3 on the source side |
  | TargetExistsDifferent | conflict policy: skip / overwrite-if-newer; default is skip + flag |
  | VerifyMismatch | post-copy size/mtime/hash check failed |
  | ExcludedByFilter, SkippedIdentical | informational, not failures |
  | Cancelled | run stopped by the user |
  | Unknown(code, msg) | anything else |

  Each reason has a plain-English explanation and a suggested fix, shown in the UI.
- **Pre-flight check** (a new idea, cheap and very useful): before any copy, check free space against total size, paths that would be longer than 260 characters, illegal or reserved names, write permission on the destination, and case collisions. These problems show up in the Not-copied panel *before* the run starts, so the user can fix them first.
- **Copy engine:**
  - Streams with a buffer (so large files don't load into memory) using `\\?\` long paths.
  - Writes to `name.odpart` and renames on success, so a partial file never looks complete.
  - Preserves the modified time.
  - Retries up to 3 times with backoff, only for transient reasons (FileLocked, NetworkLost).
  - Checks the cancel token between files and between buffer chunks.
  - Verifies by size + mtime; optionally hashes with **BLAKE3** (new dependency, fast) while copying, then re-reads the destination.
- **Job runner** in the app crate (`src-tauri/src/transfer.rs`, new):
  - Runs the job on `tauri::async_runtime::spawn_blocking` and keeps a registry keyed by `JobId`.
  - Emits `transfer://progress` (counters, bytes, current file) and `transfer://items` (a batch of item results), throttled to about 100 ms.
  - Commands:
    - `transfer_preflight`
    - `transfer_start` (returns the job id)
    - `transfer_cancel`
    - `transfer_retry` (a list of paths, or "all with reason X")
    - `transfer_copy_to_recovery`
    - `transfer_list_runs`
    - `transfer_load_run`
    - `transfer_export_report`
- **Run storage:** `%APPDATA%\<app id>\transfer-runs\<timestamp-id>\` holds `manifest.jsonl`, `results.jsonl`, `run.json` (settings and summary) and `audit.jsonl`.
  - The audit log uses `logging-core` with a new `LogDomain::Transfer` and a JSONL writer.
  - Runs survive an app restart and can be reopened or resumed.

## Stage 2: Copy mode UI (new session type `transfer-monitor`)
- Wire up the mode the way `folder-sync` is wired in, touching these places:
  - `src/types/session.ts`, `src/app/sessionCatalog.ts`, `src/app/sessionFactory.ts`, `src/app/router.ts` (`/transfer`)
  - `src/views/HomeView.vue` tile lists, `src/app/commandRegistry.ts` (`open.transferMonitor`)
  - `AppLayout.vue` icon and menus, `shellChrome.ts`, `statusBarPhrases.ts`, `sessionToolbars.ts`
  - i18n keys in all 8 locales. Only English gets real text; the other locales use the English text as a placeholder so the completeness tests pass.
- **State:** a new Pinia store `src/stores/transfer.ts`. Unlike FolderSync, the state lives in a store, not in local view refs.
  - It subscribes to the events through `src/api/transfer.ts` (`invoke` + `listen`).
  - It drives the existing `jobs` store/panel for a global progress indicator.
- **View:** `src/views/TransferMonitorView.vue`
  - **Setup bar:** Source, Destination, filters, verify level (Size+date / Hash), conflict policy, and the Pre-flight → Start / Cancel buttons.
  - **Summary strip:** planned / copied / skipped / failed counts, bytes, rate, ETA.
  - **Live feed** of arrivals, virtualized using the FolderCompare pattern and capped in the DOM.
  - **"Not copied" panel** (the main requirement): grouped by reason, each with a count, an explanation and a fix hint. Each row shows the relative path, source path, reason and OS message. Actions:
    - Retry (row, group or all)
    - Copy to recovery folder
    - Reveal source / reveal destination (`revealPathInOs`)
    - Copy path list to clipboard
  - **Previous runs** list, which reopens a stored run.
- Completion notice through `compareCompleteNotify.ts`.

## Stage 3: Watch mode (another tool is copying)
- Add the `notify` crate (it uses ReadDirectoryChangesW on Windows). Watch the destination recursively and debounce events.
- A file counts as **arriving** until its size is stable for N seconds and it can be opened for exclusive read. Then it becomes **arrived** and is verified against the source manifest entry.
- **Events can be dropped** (buffer overflow, and ReadDirectoryChangesW is unreliable on some SMB/NAS shares). So:
  - a periodic **reconcile rescan** of the destination runs anyway;
  - on network shares, polling takes over automatically when the watcher reports an error or overflow.
- **Finish** runs automatically after a configurable quiet period, or when the user clicks Finish. It performs the final reconcile:
  - source-manifest items that are missing at the destination become Not copied (reason: *Missing*; the source is checked for AccessDenied, FileLocked, PathTooLong or InvalidName so a probable cause can be given);
  - size or mtime mismatches become *VerifyMismatch*;
  - files still `.tmp`/partial or locked become *Incomplete*.
- These reasons are labelled **"inferred"** in the UI, because OpenDiff didn't do the copy itself.
- The same view is used with a Copy / Watch mode switch, sharing the Not-copied panel and the actions. "Retry" in Watch mode means "copy the missing files now with OpenDiff's engine".

## Stage 4: Recovery + client report
- **Copy to recovery folder:**
  - The user picks the location once (remembered in settings). The default is a sibling of the destination: `<dest>_NotCopied\<run-id>\`.
  - Relative folder structure is kept, and a `NOT-COPIED.csv` (path, reason, OS message, size, source) is written next to the files.
  - Where copying is impossible (DiskFull on the same volume, source vanished), the action is disabled and the tooltip explains why.
- **Export report:**
  - Add `ReportKind::Transfer` in `report-core` and render it as HTML (a client-ready summary plus tables per reason) and CSV (full rows). Hook it into the existing `src/app/reportExports.ts` format list.
  - The report contains source/destination paths, start/end times, the machine and user name, the verify level, totals, and every non-copied file with its reason. With hashing on, it also includes a source/destination hash column as proof.

## Stage 5: extras worth adding later (suggested, not committed)
- **Resume an interrupted run** (crash, USB pulled) from `results.jsonl`.
- **CLI:** `open-diff-cli transfer --source --dest --verify hash --report out.html` for scripted and unattended client jobs. Only useful once the binaries are signed (see Stage 6).
- **Auto-pause** when the destination volume disappears, and auto-resume when it comes back.
- **Bandwidth limit** for network transfers.
- **Watch presets** (saved source/destination pairs).

## Stage 6: client rollout
- **Code signing** is the main blocker: Smart App Control already blocked the unsigned CLI on this PC. Recommended options:
  - Azure Trusted Signing (low monthly cost, works in GitHub Actions)
  - an OV certificate
  Sign `setup.exe`, both binaries and the uninstaller in `windows-installer.yml`.
- Ship **NSIS only** to clients (drop the MSI from the client artifacts to avoid the mix-up seen earlier), and document the silent install `setup.exe /S`.
- **Fork identity:** product/publisher name, repo/homepage URLs, a tag-triggered release workflow publishing to the fork's GitHub Releases, and optionally the Tauri updater.
- Optionally add the CLI folder to the system PATH in the NSIS hook.

---

## Critical files
- **New:**
  - `src-tauri/crates/transfer-core/`
  - `src-tauri/src/transfer.rs`
  - `src/api/transfer.ts`
  - `src/stores/transfer.ts`
  - `src/views/TransferMonitorView.vue` (+ `src/components/transfer/NotCopiedPanel.vue`, `LiveFeed.vue`)
  - `src/types/transfer.ts`
- **Modified:**
  - `src-tauri/Cargo.toml` (workspace member, `notify`, `blake3`)
  - `src-tauri/src/lib.rs` (register commands)
  - `crates/logging-core` (Transfer domain + JSONL sink)
  - `crates/report-core` (Transfer kind)
  - `crates/folder-core` (optional per-entry error mode for the scanner, keeping the current behaviour as the default)
  - the mode-registration files listed in Stage 2
  - `tests/e2e/helpers/tauriMock.ts` (mock `listen` / event emission)

## Verification
- **Rust unit tests (transfer-core):**
  - `classify()` over `io::Error::from_raw_os_error(n)` for each code
  - pre-flight checks for long paths, reserved names and free space
  - manifest diffing
- **Rust integration tests with temp dirs:**
  - a locked source file (open with `share_mode(0)` via `OpenOptionsExt`) → FileLocked, then Retry succeeds after release
  - a read-only destination folder → AccessDenied
  - a path longer than 260 characters copies successfully through `\\?\`
  - a size mismatch → VerifyMismatch
  - cancel mid-run → Cancelled rows and no `.odpart` files left
  - a watch-mode test that copies with `robocopy` or `fs::copy` from another thread and checks that the reconcile finds a deliberately skipped file
- **Frontend:**
  - vitest for `stores/transfer.ts` (event batches → grouped Not-copied state, retry flow) and `TransferMonitorView` (mocked API)
  - a Playwright spec `tests/e2e/transfer-monitor.spec.ts` using the extended event mock
- **Manual in `tauri:dev` and then the CI-built installer:**
  - a 50k-file local copy (checks UI responsiveness)
  - a USB copy with the stick pulled mid-way (DeviceRemoved, then resume)
  - a copy to a network share
  - Watch mode during an Explorer drag-copy
  - exporting the HTML report and checking that it reads well as a client deliverable
- **CI:** `windows-installer.yml` stays green (unit tests + build). Add `cargo test -p transfer-core`.
