# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Ship DeepServer 1.0.0: hands-on laptop test of rc4, fix what it finds, then tag `v1.0.0`.

## State

- `main` at `da507d2` (Merge #23). No open PRs.
- **v1.0.0-rc4** (`da507d2`): pre-release, 4 assets, `sha256sum -c` OK; both installers ship `DeepServer.exe`, `deepserver-diskusage.exe`, `deepserver-cli.exe` (offline adds the WebView2 runtime). Release run 36993574542.
  - `DeepServer_1.0.0_x64-setup.exe` SHA-256 `68c5404e94cf846513a2567e44875082c8f16e25b66bf8579e73a26793e4052a`
  - `DeepServer_1.0.0_x64-offline-setup.exe` SHA-256 `a5d31e518cea178609a38278622dc058756ba0d060c8a633e9b2ba7e82e7b1ac`
- rc1–rc3 are older pre-releases (rc1/rc2 ship the old `open-diff-cli.exe` name).

## Done (last session)

- #21: post-install hook deletes rc1/rc2's `open-diff-cli.exe` (Tauri 2.11.4's NSIS never runs the old uninstaller for `/S` or same-version installs, and every rc is `1.0.0`). `Test-Installer.ps1` now tests an in-place upgrade over a planted leftover and requires uninstall to remove the install folder. `docs/install.md` fixes.
- Merged branches deleted by the owner.

## Next

1. Owner: install rc4 on the laptop (`docs/install.md` → "Download with the GitHub CLI"); optionally install rc2 first and upgrade, to see `open-diff-cli.exe` removed.
2. Full app test; report as needs fixing / look into further / improvements.
3. Fix findings, one PR each; cut rc5 or `v1.0.0`.

## Blocked / needs the owner

- Delete rc1/rc2 if wanted: **Actions → Repo maintenance** (one tag + release per run, `dry_run: true` first).
- Delete branch `claude/friendly-archimedes-x6dyyw` after its handoff PR merges.

## Notes

- Open decisions: in-app Docs/Support links point at the private repo (`src/app/appMeta.ts`); `%USERPROFILE%\.config\open-diff` keeps its OpenDiff name (renaming needs a migration).
- Every build reports version `1.0.0`, so Programs and the reinstall page can't tell rcs apart, and any file a future build stops shipping is left behind the same way.
- Laptop checks CI doesn't cover: settings and history survive an upgrade; uninstall while the app or engine is running.
- Waiting on a run: check the runs API `.status=="completed"`; `gh-status.sh run <id>` prints per-job `success` lines mid-run.
- Builds are unsigned: SmartScreen / Smart App Control may block them.
