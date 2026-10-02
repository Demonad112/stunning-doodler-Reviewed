# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Ship DeepServer 1.0.0: finish the rc fixes below, hands-on test, then tag `v1.0.0`.

## State

- `main` at `8b3f777` (PR #21 merge: rc1/rc2 `open-diff-cli.exe` removed on upgrade, `docs/install.md` fixes).
- v1.0.0-rc3 (`bc3d3e8`) published as a pre-release, 4 assets, checksums OK (`scripts/gh-status.sh release v1.0.0-rc3`).
- Session tooling PR (gh-status, prepush, ci-watch, `/release-rc`, `/handoff`, Repo maintenance workflow) on `claude/funny-carson-d84a60`.

## Next

1. Merge the tooling PR, then run **Actions → Repo maintenance** with `dry_run: true`, check the list, rerun with `dry_run: false`.
2. Cut rc4 with `/release-rc v1.0.0-rc4`.
3. Owner: hands-on laptop test of the installed rc4.

## Blocked / needs the owner

- `batch-4-unified-ui` isn't matched by the maintenance job (its tip differs from its merged PR head). Delete it in the GitHub UI (Branches page) if it's no longer needed.

## Notes

- One batch or fix = one branch + one PR; the owner merges.
