# Handoff

Session state for the next Claude session. Keep it short.

## Goal

DeepServer 2.0 (`V2.0/`: Compare, Record, Disk Cleanup) **replaces** V1. V1 gets no more releases. Plan: `V2.0/docs/plan.md`; install details: `V2.0/docs/install.md`. Finishing plan (approved 2026-10-17): Phases B and C done, then D (Field tools), then E (polish).

## State (2026-10-17)

- `main` (`5be3c43`): V2 Batches 0-5 merged, plus Batch 6 pieces #56, #60, #61, #63-#67, and harness audit #73.
- Open PRs from this session (CI bound with auto-fix; merge only on green):
  - [#74](https://github.com/Demonad112/stunning-doodler-Reviewed/pull/74) `v2/batch-6a-size-toggle`: Batch 6a.
  - [#75](https://github.com/Demonad112/stunning-doodler-Reviewed/pull/75) `v2/batch-6b-reports`: Batch 6b (branched from `main`, independent of #74).
- Dependabot #69-#72 still open (Phase A, owner's parallel session).
- Latest release: **v2.0.0-rc4** (pre-release). The owner's test of rc4 is not recorded; treat as pending.

## Done this session

- Perf check re-run after the scanner rewrite: 2x100k files scan + compare 0.53 s (limit 5 s). MFT fast path therefore deferred.
- 6a (#74): "Size on disk / Size" toggle in Disk Cleanup (`sizeMode`, `src/lib/cleanup.ts`), `Node::hard_linked` + link icon on rows, `logical` on rows, files and types.
- 6b (#75): `Share` bars for Record, Disk usage, Quick cleanup; CSV raw-bytes test for every kind; `report_core::prune` + `reports_prune` + Settings "Keep saved reports" (default: for ever).

## Next

1. Merge #74 and #75 when CI is green (the app crate cannot build locally, so CI is the first check of `reports.rs` / `lib.rs`).
2. Phase D, 6c: write the Batch 6 table in `V2.0/docs/plan.md`, then pre-flight review + conflict choice before Record, and share sign-in for UNC paths (`WNetAddConnection2`).
3. Phase D, 6d: RMM CLI (headless, exit codes, JSON), Explorer right-click verbs, robocopy log import.
4. Phase E (decide after D): new logo, Settings completeness, in-app update check, code signing (`scripts/windows/sign.ps1`), rc5, owner test.

## Blocked / needs the owner

- Phase A housekeeping runs in the owner's own parallel session: dependabot #69-#72 (after build + type-check), harness branch `origin/Claude/fervent-blackwell-ea2c43`, PR #68 vs the local `cd` fix in `.claude/hooks/session-start.sh`, uncommitted files in the main checkout, repo-maintenance workflow (`dry_run: true` first), V1 rc1/rc2 releases.
- Deferred to ship time (accepted until then): rc4 test result, MFT fast path decision, signing certificate, logo source.

## Notes

- Windows Application Control blocks build scripts and the `deepserver_lib` test binary locally (os error 4551): test and clippy per crate (`-p scan-core -p cleanup-core -p report-core`); CI covers the app crate.
- Local cargo from a worktree: `CARGO_TARGET_DIR=C:/Dev/projects/DeepServerV1.0/V2.0/src-tauri/target`. A fresh worktree needs `corepack pnpm install --frozen-lockfile` in `V2.0/`.
- The largest-files list is the top 100 by disk size in both size modes (logical mode relabels, does not re-rank).
- Prune uses file modified time, never touches Record runs.
- Clippy (1.98) flags `chunks_exact(const)`: use `as_chunks`. `windows-sys` `CreateFileW` needs `Win32_Security`.
- Pause points requested by the owner: compact after B+C (now), then D+D, then decide.
