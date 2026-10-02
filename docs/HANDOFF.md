# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Ship DeepServer 1.0.0: fix what the rc4 laptop test found, cut rc5, owner re-checks, then tag `v1.0.0`.

## State

- `main` at `86e0673` (Merge #29). Latest release: **v1.0.0-rc4** (pre-release, `da507d2`), installed on the laptop.
- Open PRs from the rc4 test (CI was running at handoff; merge when green, then `/release-rc v1.0.0-rc5`):
  - #30 `fix/sync-direction`: Folder Sync planned right-newer files as Copy Left to Right (wrong-direction overwrite); `src/app/pathUnderRoot.ts` shared with Folder Compare; Leave rows show the path; empty "Username:" log line removed.
  - #31 `fix/menu-width`: menu-bar panels sized to content (were fixed 240px, labels cut at 175% scaling); sidebar New list adds Transfer Monitor and Disk Usage.
  - #32 `fix/dark-mode`: ~750 hard-coded light colours → 88 `--ds-<role>-<hex>` tokens (light value unchanged, dark value by role); style tests point at the tokens.
- #30–#32 are unit-tested only; the owner verifies them in the installed rc5.
- #30 and #32 touch the same folder views; #31 and #32 touch `AppLayout.vue` / `HomeView.vue`. Merge #32 last; if it conflicts, merge `main` into it (no rebase).

## Done (this session: rc4 laptop test)

- #29 merged: `deepserver-cli compare-folders` now compares contents of same-size files (they counted as "same").
- Scripted checks on installed rc4 passed: files, Uninstall entry 1.0.0, 8 HKLM Explorer keys, CLI exit codes (0/1/4), engine `/saveto /noelevate` ledger.
- Main CI "cancelled with 0 jobs" runs were transient; rerun 37012692838 green.
- Full report and root causes: `2026-10-02-deepserver-rc4-test-reference.md` (repo root, untracked).

## Next

1. Merge #30, #31, #32 when green; `/release-rc v1.0.0-rc5`.
2. Owner checks rc5: dark mode, menus, Folder Sync direction, sidebar.
3. Owner's "squished dropdown" picture: if it's the native `<select>` popup (not the menu bar), still open.
4. Look into: Sync "Update Both" decides by mtime only; mirror `row_is_same` ignores size (`sync-core`); Text Compare single pick fills both sides; Folder Sync shows "0 bytes" for folders and a large empty area.
5. Untested on the laptop: Explorer right-click menus, Transfer Monitor (incl. hardware cases), Disk Usage scans/snapshots, other compare modes, locales.
6. Before `v1.0.0`: `CHANGELOG.md` is still OpenDiff's (no 1.0.0 entry, kygo8 links); Docs/Support links (`src/app/appMeta.ts`); delete rc1/rc2.

## Blocked / needs the owner

- Remove stale worktrees: from PowerShell, `Remove-Item -LiteralPath '\\?\C:\Users\Addy7\Projects\Work\DeepServerV1.0\.claude\worktrees\batch-4-unified-ui' -Recurse -Force`, then `git worktree remove --force .claude/worktrees/batch-5-transfer-monitor`, `git worktree prune`. After #30–#32 merge, also remove `fix-*` worktrees and branches.
- Delete merged remote branches: **Actions → Repo maintenance** (`dry_run: true` first).
- Delete rc1/rc2: same workflow, one tag + release per run.
- Archive `open-diff` and `altWinDirStat` (original plan says ask the owner first).
- Silent-install the offline installer on a Windows Server VM/Sandbox (never done outside CI).
- Delete test fixtures: `C:\Users\Addy7\DSTest`, `%TEMP%\ds-cli-test`, `%TEMP%\dsc`.

## Notes

- The laptop runs at 175% scaling (AppliedDPI 168); check layout bugs at that scale.
- Computer-use can't see WebView2 `<select>` popups (masked) or type in the folder picker; type paths into the app's own path fields instead. Explorer is click-only (no right-click).
- `deepserver-cli open --session folder-sync L R` reported success but the running app didn't pick up the paths (not investigated).
- Dark mode: `localStorage['open-diff-theme']`; tokens live in the `:root` / `html[data-theme='dark']` blocks at the top of `src/styles/main.css`. New chrome colours should use a token, not a hex.
- Playwright Chromium is installed locally; `tests/e2e/helpers/tauriMock.ts` fakes `diff_text`, so dark-mode screenshots with real diff rows are possible without Tauri.
- Handoff and notes files stay local (no PR for them).
