# CI workflows (`.github/workflows/`)

- `ci.yml`: quality + tests on every PR; a `changes` job gates the heavy Windows jobs by path.
- `build-windows.yml` (reusable, called by `ci.yml` and `release.yml`): engine build, `Test-ForkChanges.ps1`, the real-engine `diskusage-core` test, the standard installer (`tauri build`) and the offline one (`tauri bundle` + `tauri.offline.conf.json`), the portable zip, `SHA256SUMS.txt`, and `scripts/windows/Test-Installer.ps1` against both installers.
- `ci.yml` job `engine-upstream-tests` (only when `native/` changes): the upstream PS 7.6 suites and the stress test.
- `release.yml`: `v*` tags only (or a manual run for an existing tag); see `docs/releasing.md`. Optional signing through `scripts/windows/sign.ps1`.
- `prune-storage.yml`: manual only.
- `repo-maintenance.yml`: manual only; deletes merged branches and/or one tag + release (`dry_run` defaults to true).
