# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

DeepServer 2.0 (`V2.0/`: Compare, Record, Disk Cleanup) **replaces** V1. Ship **v2.0.0-rc4** with the four report/cleanup changes (A-D). Plan: `V2.0/docs/plan.md`; rc4 plan: `docs/v2-rc4-plan.md` (owner's checkout; not in the repo); install: `V2.0/docs/install.md`.

## State (2026-10-10)

- `main` = 24590b9. Latest release: **v2.0.0-rc3** (pre-release, installed on the owner's laptop).
- rc4 batches: 1 report engine (#60, merged), 2 Record successes (#61, merged), 4 Disk usage drill-down (#62, merged), **3 Compare recovery block (#63, `v2/compare-recovery`, open, CI running at last look; built locally by the owner)**.
- Open Dependabot PRs: #50, #51, #58, #59.
- Not started: Batch 5 (junk finder + safer bulk delete), Batch 6 (Compare "Copy missing" + Re-check in the app), then bump to rc4 and `/release-rc`.

## Done this session

- Batch 4 (#62): `report-core/src/explorer.rs` saves a pruned size tree (flat parent-index list, 20,000 nodes max, "(N smaller items)" fold rows) and the biggest 200 files per type (15,000 max) in `UsageReport.explorer`. The report page draws the explorer with its own second script and style (`Document.explorer`; CSP lists both hashes).
- Tests: Rust (tree pruning, caps, deep folders, category table vs `src/lib/fileTypes.ts`), jsdom (`src/lib/reportExplorer.test.ts` on `tests/fixtures/explorer.html`). Checked in Chromium at 1440/390 px, light and dark.

## Next

1. Owner: merge #63 when green. Check that its script changes did not break `csp_hash_matches_the_script_and_blocks_everything_else` or the explorer fixture (`UPDATE_FIXTURES=1 cargo test -p report-core explorer_fixture` only if the explorer parts changed).
2. Batch 5 `v2/cleanup-delete` (junk finder in a chosen folder, dry-run preview, multi-select delete, admin gating, deletion log). Plan section "Batch 5". Needs the owner's "go".
3. Batch 6 `v2/compare-copy-missing`, then rc4 version bump and `/release-rc v2.0.0-rc4`.

## Blocked / needs the owner

- Measure Batch 4 on `C:\`: `$env:EXPLORER_PATH='C:\'; $env:EXPLORER_OUT="$env:TEMP\usage.html"; cargo test -p report-core --release measure -- --ignored --nocapture` (report size and open time were only measured on Linux: 2.4 MB at 289k items).
- Branch protection on `main`: GitHub, Settings, Branches, `main`: require "V2 quality and tests" and "V2 installers". Not confirmed done.
- Dependabot #50, #51, #58, #59: review/merge. Stray `C:\Dev\projects\DeepServerV1.0\--clobber\` (untracked): delete with approval.
- Repo maintenance workflow (`dry_run: true` first) can delete the merged branches listed by `scripts/gh-status.sh branches`.

## Notes

- The harness may auto-subscribe a session to a new PR. The owner opts out: unsubscribe and use the `ci-watch` agent.
- Cloud (Linux) sessions: `cargo test -p report-core -p scan-core -p cleanup-core -p transfer-core` works; the app crate needs WebKitGTK and the MSVC target needs `ml64.exe`, so app-crate clippy is CI-only there. `scan-core` `hard_links_take_disk_space_once` fails on Linux (pre-existing).
- Windows: Application Control blocks the `deepserver_lib` test binary (os error 4551): use `cargo test --workspace --no-fail-fast`. Worktree builds: `CARGO_TARGET_DIR=C:/Dev/projects/DeepServerV1.0/V2.0/src-tauri/target`. Push with `PREPUSH_SKIP=1` (the V1 hook fails locally).
- Disk usage reports are now a few MB each and `report_core::list` parses all of them for the Reports page; report pruning is in the parked list.
