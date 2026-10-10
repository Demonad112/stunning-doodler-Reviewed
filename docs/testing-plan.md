# Testing and update plan

Goal: ship `v1.0.0` and keep every later update cheap to test. Spend tokens and owner time only where an automated layer can't cover.

## Layers (cheapest first)

| Layer             | Who                          | Covers                                                                                            | Cost                               |
| ----------------- | ---------------------------- | ------------------------------------------------------------------------------------------------- | ---------------------------------- |
| 1. CI on every PR | GitHub Actions               | quality, unit, e2e, engine tests, both installers built and installed by `Test-Installer.ps1`     | free; wait with `ci-watch`         |
| 2. Release check  | `laptop-check` agent (Haiku) | asset names, SHA-256, install contents, uninstall entry, Explorer keys, CLI exit codes, data dirs | one short run per rc               |
| 3. Manual pass    | owner                        | GUI, elevation, hardware, upgrade/uninstall with the app running                                  | time-boxed, only for changed areas |

Rule: if a manual item can be scripted without hardware, move it down a layer (add to `Test-Installer.ps1` or `laptop-check`) after its first run.

## Per-release loop

1. `/release-rc <tag>` publishes the rc; `ci-watch` confirms the run.
2. `laptop-check <tag>`: any "Needs fixing" becomes a PR. Nothing else yet.
3. Owner runs the MANUAL list below, scoped by the diff since the last rc (`git diff --stat <prev-rc>..<tag>`): only areas touched, plus the always-run smoke.
4. Report as **Needs fixing / Look into further / Improvements**. Fixes: one PR each, green CI, then the next rc. Repeat until a pass finds nothing in the first group.
5. Final: `v1.0.0` = last green rc re-tagged; delete rc1/rc2, decide Docs/Support links, publish the draft.

## Always-run smoke (about 10 min, every rc)

- Fresh install, launch, About says `1.0.0`; uninstall removes folder and Explorer keys, keeps user data.
- Disk Usage scan of a folder; Stop returns within 2 s.
- Folder Compare of two small folders; `deepserver-cli compare-folders` exit 0/1.
- Explorer menu on a file and a folder (Win 11 "Show more options").

## By-area passes (run when the diff touches the area)

- **Upgrade**: install the previous rc, change a setting, scan once, install the new rc; settings and history survive, stale files gone.
- **Disk Usage**: `C:\` scan, snapshot, change files, snapshot, What changed; engine checklist in `native/diskusage/HANDOFF.md`.
- **Transfer Monitor**: size+time and Hash verify; locked file, Not copied, Retry; Watch during an Explorer copy; reports (HTML/CSV/JSON); Clean up now. Hardware (USB pull, SMB, 50k files) only before `v1.0.0` and when `transfer*` crates change.
- **Folder Compare**: `C:\` shows "Unreadable"; Sync stops on unreadable; report times local.
- **Other modes**: Text, Table, Image, Hex, Version, Merge, archives, snapshots: open one file each.
- **UI**: 8 locales, dark/light, 125%/150% DPI, jobs panel, Help links (no OpenDiff URLs). Locales/DPI only when `src/i18n` or layout changes.
- **Resources**: idle CPU/RAM, no WebView2 left after close.

## Keeping the automation sharp

- After each manual pass, list items that needed no human judgement and move them to `Test-Installer.ps1` or `.claude/agents/laptop-check.md`.
- Keep `docs/HANDOFF.md` current by hand; start each checkpoint in a new chat.
- Token rules: low effort, `git`/`gh` via `scripts/gh-status.sh`, Haiku agents for waiting and checking, no root-level reference files.
- Once `v1.0.0` ships, the same loop applies to patch releases: CI, `laptop-check`, then only the by-area passes the diff touches.
