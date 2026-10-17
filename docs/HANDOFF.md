# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

DeepServer 2.0 (`V2.0/`: Compare, Record, Disk Cleanup) **replaces** V1. V1 gets no more releases. Plan: `V2.0/docs/plan.md`; install details: `V2.0/docs/install.md`.

## State (2026-10-17)

- `main` (`5a6157c`): V2 Batches 0-5 merged, plus Batch 6 pieces: scan accuracy (#56), report engine (#60), Record successes (#61), compare recovery (#63), junk finder and bulk delete (#64), Compare copy-missing and re-check (#65), dates and attributes kept (#66), permissions, owner, audit rules, streams (#67).
- Latest release: **v2.0.0-rc4** (pre-release). rc1 was installed on the owner's laptop; the owner's test of rc4 is not recorded here, so treat it as pending.
- Open PRs: dependabot #69-#72 (do not merge until the full build and type-check pass).
- Engine note: the MFT fast path, the size-on-disk toggle and the perf re-check are still open (see Next).

## Next (suggested order)

1. Owner: test rc4 on a real drive; compare Disk Cleanup totals with Explorer "Size on disk".
2. Scanner follow-ups:
   - Optional MFT fast path for admin runs (the fork's `FinderNtfs.cpp` shows the technique; re-implement, don't copy).
   - UI toggle "Size on disk / Size" in Disk Cleanup, and a hardlink marker on rows.
   - Perf check: `cargo test -p scan-core --release --test perf -- --ignored --nocapture` (2x100k files < 5 s). Not re-run since the rewrite.
3. Reports: Disk Cleanup and Record bars; CSV sizes as raw bytes; prune old reports.
4. Batch 6 "Field tools", remaining items (check `V2.0/docs/plan.md` first): pre-flight review and conflict choice, share sign-in, RMM CLI, Explorer right-click, robocopy log import.
5. Product polish: new logo (icon is still V1's), code signing (`scripts/windows/sign.ps1`; builds are unsigned), Settings page, in-app update check.

## Needs the owner

- Dependabot PRs #69-#72 (review after the build and type-check pass).
- Harness branch `origin/Claude/fervent-blackwell-ea2c43` (unmerged; rewrites root `CLAUDE.md`). Decide whether to merge.
- Repo maintenance workflow (`dry_run: true` first) for merged branches and V1 rc1/rc2 releases. Archive `open-diff` and `altWinDirStat` only after the owner confirms.
- Harness audit follow-ups (2026-10-17): PR #68 edits `.claude/hooks/session-start.sh` too; the local `cd` fix conflicts with it, so merge carefully. The main checkout (`chore/session-start-hook-parallel`) has uncommitted changes to `.claude/settings.json`, five `.github/workflows/*.yml` files and this file; inspect before touching. The `ci-watch` agent (`.claude/agents/ci-watch.md`) is no longer referenced; delete it if the owner agrees.

## Notes

- Windows Application Control blocks the `deepserver_lib` test binary locally (os error 4551): use `cargo test --workspace --no-fail-fast`; CI covers the app crate.
- Local cargo from a worktree: `CARGO_TARGET_DIR=C:/Dev/projects/DeepServerV1.0/V2.0/src-tauri/target`. `windows-sys` `CreateFileW` needs the `Win32_Security` feature.
- Clippy (1.98) flags `chunks_exact(const)`: use `as_chunks`.
