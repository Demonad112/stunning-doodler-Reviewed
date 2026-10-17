# CI and cloud sessions

Reference for `.github/workflows/`. The root `CLAUDE.md` links here.

## Workflows

- `ci.yml`: quality + tests on every PR; a `changes` job gates the heavy Windows jobs by path
- `build-windows.yml` (reusable, called by `ci.yml` and `release.yml`): engine build, `native/diskusage/tests/Test-ForkChanges.ps1`, the real-engine `diskusage-core` test, the standard installer (`tauri build`) and the offline one (`tauri bundle` + `tauri.offline.conf.json`), the portable zip, `SHA256SUMS.txt`, and `scripts/windows/Test-Installer.ps1` against both installers
- `ci.yml` job `engine-upstream-tests` (only when `native/` changes): the upstream PS 7.6 suites and the stress test
- `release.yml`: `v*` tags only (or a manual run for an existing tag); see `docs/releasing.md`. Optional signing through `scripts/windows/sign.ps1`
- `prune-storage.yml`: manual only
- `repo-maintenance.yml`: manual only; deletes merged branches and/or one tag + release (`dry_run` defaults to true)

## Cloud session limits (don't retry these)

| Blocked                                          | Use instead                                                      |
| ------------------------------------------------ | ---------------------------------------------------------------- |
| `git push` of tags, `git push --delete`          | `release.yml` with `create_tag: true`; Repo maintenance workflow |
| Delete branch / edit release (no connector tool) | Repo maintenance workflow; owner via GitHub UI                   |
| `rm` with a relative glob after `cd`             | absolute paths, or `git rm` / `git clean -n` first               |
| Editing `.github/workflows/*` in auto mode       | allowed in `.claude/settings.json`; if still refused, ask once   |
