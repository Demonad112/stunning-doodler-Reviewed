# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Two tracks: (1) finish V1: cut rc5, owner re-checks, tag `v1.0.0`. (2) Build DeepServer 2.0 in `V2.0/`: Compare, Record and Disk Cleanup only, Windows 11 look. Plan: `V2.0/docs/plan.md` (Batches 0–5; Batch 6 "Field tools" added below).

## State (2026-10-08)

- `main` at `c1e8585`: V2 Batches 0–3 merged (Compare #38/#39, Record #40/#41, Disk Cleanup #42), plus Dependabot #43. Latest release **v1.0.0-rc4**; rc5 not cut.
- **#47** (draft) `v2/batch-4-reports`, head `1cdd0fd`: V2 Batch 4, see below. Worktree `.claude/worktrees/v2-batch-4` (cloud only; the local checkout needs its own).
- **#46** (draft) `claude/friendly-galileo-2b664k`: a handoff refresh from the cloud session (this file replaces it).
- **#44** (Rust group) and **#45** (npm group): Dependabot, not merged. #44 breaks: `ureq::AgentBuilder` is gone in `remote-core`, needs a ureq 3 migration. #45 breaks: typescript-eslint does not support TS 7.0, hold `typescript` below 7. `.github/dependabot.yml` does not cover `V2.0/`.
- **CI is not running**: every job on #46/#47 failed to start with "recent account payments have failed or your spending limit needs to be increased" (no steps ran). Not a code failure. The owner is handling it; until CI runs, do not merge anything.

## Done this session (cloud, 2026-10-08)

- Verified state; found this file stale (Batch 3 was already merged). Read the Repo maintenance dry run: it succeeded but the log was not read, so the list of branches it would delete is unseen.
- Merged #43 (`download-artifact` 7->8) after its failed e2e jobs passed on one re-run (`page.goto` timeouts, unrelated).
- **V2 Batch 4 (#47)**, all new tests passing on Linux, clippy clean on Linux and `x86_64-pc-windows-msvc`:
  - `transfer-core`: `ReportMeta` (client, ticket, technician) in `run.json`; `export.rs` writes `report.html` (escaped, self-contained, prints) and `not-copied.csv` (BOM, formula-safe) into the run folder; `RunStore::save_meta` and `delete`.
  - `src-tauri/src/record.rs`: `record_set_meta`, `record_export`, `record_open_export` (only `report.html`/`not-copied.csv` inside the records folder), `record_delete` (refused while a record runs); `record_retry` takes an optional `reason`.
  - `src/pages/ReportsPage.vue` (`/reports`, replaces `PlaceholderPage.vue`): list, search, Export, Delete with confirm. `RecordReportPage.vue`: job-detail boxes, Export report, reason chips retry only that reason.
  - `cleanup-core/src/quickwins.rs` + `QuickWins.vue` on the Cleanup start screen: user Temp (over 1 day), browser caches, Recycle Bin, old installers in Downloads (over 30 days, to the Recycle Bin), Windows Update and Windows Temp (administrator). Places are built from the environment; the page sends only an id; a `TEMP` not named Temp/Tmp is ignored; links are removed, never followed. `os.rs`: `IsUserAnAdmin`, `ShellExecuteW runas` (Restart as administrator), `SHQueryRecycleBinW`, `SHEmptyRecycleBinW`.
  - `V2.0/CLAUDE.md` updated.

## Next

1. **Owner pass on #47 on the laptop** (the Windows-only calls and UI were never run): Reports page, Export report, retry by reason, each quick win (Recycle Bin size and empty, Restart as administrator prompt), at 175% scaling in light and dark.
2. When CI runs: re-trigger #47 (and #46), then merge #47 only if green. Owner merges.
3. Fix #44 and #45 as one small V1 PR (ureq 3 migration; hold `typescript` < 7); add `V2.0/` npm and cargo entries to `dependabot.yml`.
4. V1: `/release-rc v1.0.0-rc5`, owner checks it, then tag `v1.0.0`. Leftovers: Docs/Support links (`src/app/appMeta.ts`), delete rc1/rc2.
5. Batch 4 leftovers: Settings (default verify level, report folder), re-check an old record against its manifest, import a robocopy log. Compare and Cleanup results are not saved as reports (Reports lists copy records only).
6. Batch 5 (Installer): NSIS standard + offline WebView2, portable mode with records next to the exe, `v2.*` release workflow. Decide side by side vs replace V1 first.
7. Batch 6 "Field tools": "Copy missing only" from a Compare result, pre-flight review plus conflict choice, protected files via admin/backup rights, share sign-in, CLI for RMM tools, Explorer right-click.

## Open questions for the owner

- V1 and V2 both install as "DeepServer": side by side, or should V2 replace V1? Decide before Batch 5.
- Report branding/logo (the HTML report uses plain "DeepServer" text). MFT speed vs. no elevation prompt (Cleanup).

## Blocked / needs the owner

- New logo for 2.0.
- Delete merged branches (`claude/brave-allen-tiokj8`, `fix/menu-row-width`, `v2/batch-0-scaffold`, `v2/batch-2-record`, `v2/batch-3-cleanup`, `v2/record-fixes`) and rc1/rc2: **Actions → Repo maintenance** (`dry_run: true` first, read the log, then run with `dry_run` off). Then `git worktree remove .claude/worktrees/v2-batch-2` and `.../v2-record-fixes` locally.
- Archive `open-diff` and `altWinDirStat` (ask first).

## Notes

- Cloud sessions can't build the V2 app crate (no WebKitGTK). Check Windows code with `rustup target add x86_64-pc-windows-msvc` and `CARGO_FEATURE_PURE=1 cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings` (the env var only lets blake3 build without the MSVC assembler).
- Defender ASR blocks Cargo build scripts outside the two excluded `target` folders. From a worktree, set `CARGO_TARGET_DIR=C:/Users/Addy7/Projects/Work/DeepServerV1.0/V2.0/src-tauri/target` (V2) or `.../DeepServerV1.0/src-tauri/target` (V1). Smart App Control is off.
- The repo-root pre-push hook only checks V1. If a push changes only V2, run `corepack pnpm quality` + `corepack pnpm test` in `V2.0/` and push with `PREPUSH_SKIP=1`.
- Real-app testing without screenshots: start with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333`, then drive the page over CDP.
- Laptop runs at 175% scaling; V2's default window is 1200×720.
- The owner opts out of PR watching: after creating a PR, don't `subscribe_pr_activity` (the harness re-subscribes on its own; unsubscribe again).
- Handoff and notes files: committed on `main` directly, no PR, when working locally. The cloud session can only push its own branch, so its handoff arrives as a PR.
