# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

DeepServer 2.0 (`V2.0/`: Compare, Record, Disk Cleanup) **replaces** V1. V1 gets no more releases. Plan: `V2.0/docs/plan.md`; install details: `V2.0/docs/install.md`.

## State (2026-10-08)

- `main`: V2 Batches 0-5 merged (#54). **v2.0.0-rc1** is released (pre-release) and installed on the owner's laptop over V1 (V1 leftovers gone). The app has not been launched there yet: owner test pending.
- Open PR **`v2/scan-accuracy`** (this session), not merged until CI is green and the owner says so:
  - **Scanner**: `scan-core/src/list.rs` lists folders with `NtQueryDirectoryFile` in bulk (std `read_dir` fallback). New `Node.disk` = size on disk (clusters, compressed/sparse/WOF via `GetCompressedFileSizeW`, hard links once, cloud files 0). `Node.size` stays logical for Compare/Record. Disk Cleanup (`cleanup-core`, `src-tauri/src/cleanup.rs`) now shows `disk`. Written fresh from the WinDirStat fork's approach (`native/diskusage/windirstat/FinderBasic.cpp`); no source copied (fork is GPL-2, V2 is Apache-2.0).
  - **Reports**: shared HTML renderer (`report-core/src/doc.rs`) redesigned: gradient header, verdict pill, tile accents, "found at destination" bar for Compare (`Document.share`), row counts, zebra rows, dark mode, phone width, print styles. CSV unchanged.
  - Not done: no 1440px / 390px screenshots (no browser tool in that session). Open `Document::html` output in a browser before merging. No real-disk comparison against WinDirStat yet.

## Next (suggested order)

1. Owner: launch rc1, scan a real drive and compare the total with WinDirStat / Explorer "Size on disk"; merge the PR if good; cut rc2.
2. Scanner follow-ups:
   - Optional MFT fast path for admin runs (the fork's `FinderNtfs.cpp` shows the technique; re-implement, don't copy). Cleanup already has an admin relaunch.
   - UI toggle "Size on disk / Size" in Disk Cleanup, and a hardlink marker on rows (the data exists: repeat links have `disk` 0).
   - Show "N hard links, M compressed files" in the Cleanup report note.
   - Perf check: `cargo test -p scan-core --release --test perf -- --ignored --nocapture` (2x100k files < 5 s) was not re-run after the rewrite.
3. Reports: Disk Cleanup report with a proportional bar per top folder; Record report with a copied/missed bar; CSV sizes as raw bytes; prune old reports and `%TEMP%\DeepServer2 reports`.
4. Batch 6 "Field tools": Copy missing only from Compare, pre-flight review + conflict choice, admin/backup-rights reads, share sign-in, RMM CLI, Explorer right-click, re-check a record against its manifest, robocopy log import.
5. Product polish: new logo (icon is still V1's), code signing (`scripts/windows/sign.ps1` hook exists; builds are unsigned), Settings page (default verify level, report folder), in-app "check for update".

## Needs the owner

- Dependabot PRs #50-#53, #55 (review/merge).
- Stray folder `C:\Dev\projects\DeepServerV1.0\--clobber\` (untracked, delete with approval).
- Repo maintenance workflow (`dry_run: true` first): merged branches incl. `v2/batch-5-installer`, V1 rc1/rc2 releases. Archive `open-diff` and `altWinDirStat` only after the owner confirms.
- CI long term: stay public, fix billing, org trial, or self-hosted runner.

## Notes

- Windows Application Control blocks the `deepserver_lib` test binary locally (os error 4551): use `cargo test --workspace --no-fail-fast`; CI covers the app crate.
- Local cargo from a worktree: `CARGO_TARGET_DIR=C:/Dev/projects/DeepServerV1.0/V2.0/src-tauri/target`. `windows-sys` `CreateFileW` needs the `Win32_Security` feature.
- Clippy (1.98) flags `chunks_exact(const)`: use `as_chunks`.
