# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Ship DeepServer 1.0.0: hands-on laptop test of rc4, fix what it finds, then tag `v1.0.0`.

## State

- `main` at `84631c6` (Merge #25). No open PRs besides the handoff PR for this file.
- **v1.0.0-rc4** (`da507d2`): pre-release, 4 assets; both installers ship `DeepServer.exe`, `deepserver-diskusage.exe`, `deepserver-cli.exe` (offline adds WebView2). Release run 36993574542.
  - `DeepServer_1.0.0_x64-setup.exe` SHA-256 `68c5404e94cf846513a2567e44875082c8f16e25b66bf8579e73a26793e4052a`
  - `DeepServer_1.0.0_x64-offline-setup.exe` SHA-256 `a5d31e518cea178609a38278622dc058756ba0d060c8a633e9b2ba7e82e7b1ac`
- rc1–rc3 are older pre-releases (rc1/rc2 ship the old `open-diff-cli.exe` name).
- Last CI on `main` was cancelled runs (likely concurrency); no green run confirmed at the head commit yet.

## Done (this session: Checkpoint 1, automation setup)

- #25: `.claude/hooks/session-start.sh` (SessionStart: HEAD vs origin, open PRs, latest release), `repo-state` agent (Haiku, read-only), user-only `/laptop-test <tag>` skill, `Read` deny rules for `src-tauri/target`, `node_modules`, `native/diskusage/build`, `docs/history/opendiff`, `gh` read-only allow entries replacing `mcp__github__*`, `CLAUDE.md` "Local setup and plan".
- Moved the 7 root `*-reference.md` files to `~/.claude/handoffs/`.

## Next

1. Checkpoint 2: owner runs `/laptop-test v1.0.0-rc4` in a new chat (install, upgrade rc2 to rc4, full app test); report as needs fixing / look into further / improvements.
2. Fix findings, one PR each with green CI; cut rc5 via `/release-rc`.
3. `v1.0.0`: delete rc1/rc2, decide Docs/Support links, publish the draft.

## Blocked / needs the owner

- Remove stale worktrees (git hit "Filename too long", auto mode blocked the delete): from PowerShell, `Remove-Item -LiteralPath '\?\C:\Users\Addy7\Projects\Work\DeepServerV1.0\.claude\worktrees\batch-4-unified-ui' -Recurse -Force`, then `git worktree remove --force .claude/worktrees/batch-5-transfer-monitor`, `git worktree prune`, `git branch -d batch-4-wrapup batch-5-transfer-monitor`.
- Delete remote branches `claude/friendly-archimedes-x6dyyw`, `claude/funny-carson-d84a60`, `claude/automation-setup`, `claude/handoff-checkpoint-1` after merge: **Actions → Repo maintenance** (`dry_run: true` first).
- Delete rc1/rc2 if wanted: same workflow, one tag + release per run.
- Untracked in the repo root, not mine: `DeepServer-windows.zip`, `DeepServer-windows/`, two `Recording *.mp4`; delete or move them.
- The failing `plugin:github` MCP is a user-level plugin; disable it in the Claude app if the error noise bothers you.

## Notes

- Run `git push` as its own Bash call: the pre-push hook fires on the whole command text, so chaining prettier/amend before it fails and skips the earlier steps.
- Open decisions: in-app Docs/Support links point at the private repo (`src/app/appMeta.ts`); `%USERPROFILE%\.config\open-diff` keeps its OpenDiff name (renaming needs a migration).
- Every build reports version `1.0.0`, so rcs can't be told apart in Programs; a file a future build stops shipping is left behind (rc1/rc2's CLI is handled by #21).
- Laptop checks CI doesn't cover: settings and history survive an upgrade; uninstall while the app or engine runs; Transfer Monitor hardware cases (USB pull, SMB, 50k files, Watch).
- Waiting on a run: use the `ci-watch` agent; `gh-status.sh run <id>` prints per-job `success` lines mid-run.
- Builds are unsigned: SmartScreen / Smart App Control may block them.
