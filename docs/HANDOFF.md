# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Ship DeepServer 1.0.0: cut rc5 with the rc4-test fixes, owner re-checks, then tag `v1.0.0`.

## State

- `main` at `7d6af3c` (docs commit on top of Merge #32 `c66bcdf`). No open PRs.
- Latest release: **v1.0.0-rc4** (installed on the laptop). **rc5 not cut yet**: waiting for CI run 37026556951 on `main` to go green, then `/release-rc v1.0.0-rc5`.
- Uncommitted WIP: worktree `.claude/worktrees/fix-mirror-size` (branch `fix/mirror-size`): `sync-core` `row_is_same` now also requires equal file sizes, so Mirror copies a file whose size differs when mtimes match; new test; `cargo test -p sync-core` 15 passed, clippy clean. Not committed (owner said stop); commit + PR when wanted.

## Done (this session: rc4 laptop test)

- Merged #29 (CLI `compare-folders` compares contents of same-size files), #30 (Folder Sync right-newer rows planned as Copy Left to Right → wrong-direction overwrite; `src/app/pathUnderRoot.ts`; Leave rows show the path; empty "Username:" log line removed), #31 (menu panels sized to content; sidebar New adds Transfer Monitor and Disk Usage), #32 (dark mode: ~750 hard-coded colours → 88 `--ds-<role>-<hex>` tokens). #30–#32 unit-tested only; owner verifies in rc5.
- `7d6af3c`: CLAUDE.md gotchas (colour tokens, Bash backslashes, slow unit suite), engine CLAUDE.md workflow name, this file.
- Scripted checks on installed rc4 passed: files, Uninstall entry 1.0.0, 8 HKLM Explorer keys, CLI exit codes (0/1/4), engine `/saveto /noelevate` ledger.
- Main CI "cancelled with 0 jobs" runs were transient; rerun 37012692838 green.
- Full report and root causes: `2026-10-02-deepserver-rc4-test-reference.md` (repo root, untracked).

## Next

1. When run 37026556951 is green: `/release-rc v1.0.0-rc5`.
2. Owner checks rc5: dark mode, menus, Folder Sync direction, sidebar.
3. Owner's "squished dropdown" picture: if it's the native `<select>` popup (not the menu bar), still open.
4. Look into: Sync "Update Both" decides by mtime only; mirror `row_is_same` ignores size (`sync-core`); Text Compare single pick fills both sides; Folder Sync shows "0 bytes" for folders and a large empty area.
5. Untested on the laptop: Explorer right-click menus, Transfer Monitor (incl. hardware cases), Disk Usage scans/snapshots, other compare modes, locales.
6. Before `v1.0.0`: `CHANGELOG.md` is still OpenDiff's (no 1.0.0 entry, kygo8 links); Docs/Support links (`src/app/appMeta.ts`); delete rc1/rc2.

## Blocked / needs the owner

- Remove stale worktrees: from PowerShell, `Remove-Item -LiteralPath '\\?\C:\Users\Addy7\Projects\Work\DeepServerV1.0\.claude\worktrees\batch-4-unified-ui' -Recurse -Force`, then `git worktree remove --force .claude/worktrees/batch-5-transfer-monitor`, `git worktree prune`. Also delete the leftover `fix-sync-direction`, `fix-menu-width`, `fix-dark-mode` worktree folders the same way (branches already deleted; git hit "Filename too long"), then `git worktree prune`.
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
- Handoff and notes files: commit on `main` directly, no PR (owner's call).
