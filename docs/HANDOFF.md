# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Two tracks: (1) finish V1: merge #36, cut rc5, owner re-checks, tag `v1.0.0`. (2) Build DeepServer 2.0 in `V2.0/`: Compare, Record and Disk Cleanup only, Windows 11 look. Plan: `V2.0/docs/plan.md` (Batches 0–5).

## State

- `main` at `d28f3c0` (#36 V1 dropdown fix and #37 V2 Batch 0 merged). Latest release **v1.0.0-rc4**; rc5 not cut.
- V2 Batch 1 (`scan-core` + Compare page) on branch `claude/brave-allen-tiokj8` (cloud session), draft PR open. Built in the cloud: Linux tests + Windows-target clippy pass; never run as a desktop app yet.

## Done this session (cloud, 2026-10-03)

- `V2.0/src-tauri/crates/scan-core`: parallel walker (std `read_dir` + rayon), recursive folder totals, junctions/symlinks not followed, OneDrive online-only files flagged (attributes only, no download), unreadable folders kept with an error. Size compare: missing / extra / size differs / type differs, case-insensitive names, missing bytes rolled up. 2x100k files scan + compare in 0.54 s on Linux.
- `compare.rs`: `compare_start` (Channel progress every 100 ms, both sides in parallel), `compare_children` (lazy, one level per call), `compare_cancel`. `tauri-plugin-dialog` for Browse.
- Polish pass: Explorer "Copy as path" quotes stripped; same/nested folders rejected with a clear message; sortable columns; arrow-key tree navigation; right-click menu (show in Explorer, copy path) and double-click to reveal (`tauri-plugin-opener`); "Copy missing list" (header + relative paths, for email/tickets); clickable Missing/Other tiles; elapsed timer, per-side Done, Esc to cancel, cancel notice; recent compares on Compare and Home; layout fits the 900×560 minimum window.
- Compare page: Source/Destination fields (type, Browse, drag-drop from Explorer, swap), Compare/Cancel, live progress, 4 summary tiles (missing in red), filters All/Differences/Missing, virtualised tree table. Screenshots in `V2.0/docs/screenshots/`.

## Next

1. Owner on the laptop: `cd V2.0; corepack pnpm install; corepack pnpm tauri dev`, compare two real folders (one under OneDrive), check 175% scaling and both themes; then merge the Batch 1 PR.
2. `/release-rc v1.0.0-rc5` for V1 when convenient.
3. On "go": V2 Batch 2 Record (vendor V1 `transfer-core` + `job-core` + `logging-core`; replace its `folder_core` walk with `scan-core`; Watch + Copy modes, live panel, report). Add the "Record copy" button on the Compare page then.
4. V2 Batches 3–5 per plan (Disk Cleanup reuses `scan-core`, Reports/export, installer).
5. V1 leftovers before `v1.0.0`: Docs/Support links (`src/app/appMeta.ts`); delete rc1/rc2; Sync "Update Both" decides by mtime only.

## Open questions for the owner

- V1 and V2 both install as "DeepServer": side by side, or should V2 replace V1? Decide before Batch 5.
- Should reports carry company branding/logo for client hand-off (Batch 4)?
- Must Record survive an app restart/reboot mid-copy (Batch 2)?
- Full-drive scan speed vs. no elevation popup: MFT reading needs admin (Batch 3).

## Blocked / needs the owner

- New logo for 2.0 (V2 reuses the V1 icon in title bar and taskbar).
- Remove stale worktrees: PowerShell `Remove-Item -LiteralPath '\\?\C:\Users\Addy7\Projects\Work\DeepServerV1.0\.claude\worktrees\<name>' -Recurse -Force` for `batch-4-unified-ui`, `fix-sync-direction`, `fix-menu-width`, `fix-dark-mode`; then `git worktree remove --force .claude/worktrees/batch-5-transfer-monitor`; `git worktree prune`.
- Delete merged remote branches and rc1/rc2: **Actions → Repo maintenance** (`dry_run: true` first).
- Archive `open-diff` and `altWinDirStat` (ask first).
- Silent-install the offline installer on a Windows Server VM/Sandbox.
- Delete test fixtures: `C:\Users\Addy7\DSTest`, `%TEMP%\ds-cli-test`, `%TEMP%\dsc`.

## Notes

- Laptop runs at 175% scaling: V2 default window is 1200×720 so it fits above the taskbar.
- Computer-use can't grant a dev build (`target\debug\deepserver.exe` isn't an installed app). Capture the window with PowerShell `CopyFromScreen` on its `MainWindowHandle` rect (DPI-aware) instead.
- Running `tauri dev` from `V2.0/`: don't set a relative `CARGO_TARGET_DIR` (cargo runs in `src-tauri/`, so it nests `src-tauri/src-tauri/target`). Leave it unset.
- V2 Playwright screenshots: run a script from a checkout with root `node_modules` (import `@playwright/test`), against `http://localhost:1430`.
- `transfer-core` already marks source files missing at the destination as NotCopied in Watch mode; that is V2's Record feature.
- Handoff and notes files: commit on `main` directly, no PR (owner's call).
