# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Two tracks: (1) finish V1: merge #36, cut rc5, owner re-checks, tag `v1.0.0`. (2) Build DeepServer 2.0 in `V2.0/`: Compare, Record and Disk Cleanup only, Windows 11 look. Plan: `V2.0/docs/plan.md` (Batches 0–5).

## State

- `main` at `1edf92e` (Merge #35). Latest release **v1.0.0-rc4**; rc5 not cut.
- PR #36 `fix/menu-row-width`: GREEN, mergeable. V1 dropdown rows clipped to 9em (`AppLayout.vue` `.menus button` also matched panel rows); scoped to `.menus > .menu-group > button`, plus a new e2e test.
- PR #37 `v2/batch-0-scaffold`: GREEN, mergeable. V2 Batch 0 shell.
- Worktrees: `.claude/worktrees/fix-menu-row-width`, `.claude/worktrees/v2-batch-0` (delete after merge).

## Done this session

- Owner decided V2 scope (2026-10-02): in-app disk analyzer (drop the WinDirStat engine), drop every other compare mode, `V2.0/` in this repo, Windows 11 Fluent/Mica look.
- #36: V1 dropdown fix (the "squished dropdown" was the menu bar, not `<select>`).
- #37: `V2.0/` Tauri 2 + Vue 3 + Tailwind v4 app (`com.demonad112.deepserver2`, dev port 1430): frameless title bar, Mica on Win11, nav rail, light/dark/system theme, placeholder pages, `V2.0/CLAUDE.md`, `.github/workflows/v2.yml` (V2.0/** only). Root prettier/eslint skip `V2.0/`.

## Next

1. Owner merges #36 and #37, then `/release-rc v1.0.0-rc5`.
2. On "go": V2 Batch 1: `scan-core` (recursive size tree, progress via `tauri::ipc::Channel`) + Compare page (two path pickers, summary strip, tree table, missing in red).
3. V2 Batch 2: Record (vendor V1 `transfer-core` + `job-core` + `logging-core`; replace its `folder_core`/`file_core` calls; Watch + Copy modes, live panel, report).
4. V2 Batches 3–5 per plan (Disk Cleanup, Reports/export, installer).
5. V1 leftovers before `v1.0.0`: Docs/Support links (`src/app/appMeta.ts`); delete rc1/rc2; Sync "Update Both" decides by mtime only.

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
