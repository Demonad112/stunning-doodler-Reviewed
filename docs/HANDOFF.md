# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Ship DeepServer 1.0.0: finish the rc fixes below, hands-on test, then tag `v1.0.0`.

## State

- `main` at `bb159cb` (PR #20 merge).
- v1.0.0-rc3 (`bc3d3e8`) published as a pre-release, 4 assets, checksums OK (`scripts/gh-status.sh release v1.0.0-rc3`).
- Session tooling PR (gh-status, prepush, ci-watch, `/release-rc`, `/handoff`, Repo maintenance workflow) on `claude/funny-carson-d84a60`.

## Next

1. Merge the tooling PR, then run **Actions → Repo maintenance** with `dry_run: true`, check the list, rerun with `dry_run: false`.
2. Installer upgrade leftover: the old `open-diff-cli.exe` is probably not removed when upgrading from rc1/rc2. Reproduce in `Test-Installer.ps1`, fix in the NSIS hooks.
3. `docs/install.md`: the offline installer is ~205 MB larger (213 vs 8.4 MB), not "130 MB"; private-repo downloads need a GitHub login; list `%USERPROFILE%\.config\open-diff` as a data folder.
4. Cut rc4 with `/release-rc v1.0.0-rc4`.
5. Owner: hands-on laptop test of the installed rc4.

## Blocked / needs the owner

- `batch-4-unified-ui` isn't matched by the maintenance job (its tip differs from its merged PR head). Delete it in the GitHub UI (Branches page) if it's no longer needed.

## Notes

- One batch or fix = one branch + one PR; the owner merges.
