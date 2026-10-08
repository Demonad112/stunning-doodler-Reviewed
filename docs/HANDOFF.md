# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Two tracks: (1) finish V1: cut rc5, owner re-checks, tag `v1.0.0`. (2) Build DeepServer 2.0 in `V2.0/`: Compare, Record and Disk Cleanup only, Windows 11 look. Plan: `V2.0/docs/plan.md` (Batches 0–5; Batch 6 "Field tools" below).

## State (2026-10-08)

- `main` at `a9a92dd`: V2 Batches 0–4 are merged (Compare #38/#39, Record #40/#41, Disk Cleanup #42, Reports + export + quick wins #47). Latest release **v1.0.0-rc4**; rc5 not cut.
- The repo is **public** again (owner, 2026-10-08): the private repo's Actions minutes had run out ("recent account payments have failed"). CI runs normally while it is public. Keeping it public vs. fixing billing / org trial (`obsidianintelligenceyyc`) / self-hosted runner is still undecided.
- **#44** (Rust group) and **#45** (npm group): Dependabot, not merged. #44 breaks: `ureq::AgentBuilder` is gone in `remote-core`, needs a ureq 3 migration. #45 breaks: typescript-eslint does not support TS 7.0, hold `typescript` below 7. `.github/dependabot.yml` does not cover `V2.0/`.
- All local worktrees are removed; the repo root is clean (old reference files, recordings and screenshots are in `~/.claude/handoffs/deepserver-root-2026-10-08/`).

## Done this session (laptop, 2026-10-08)

- Found why CI was red: billing, not code (job annotation "recent account payments have failed"). Owner made the repo public; checks run again.
- Batch 4 existed twice (local worktree A, cloud PR B). Kept A (unified `report-core`: Compare, Cleanup and Record reports, branded HTML/CSV, `cleanup_core::quick`, `JobFields`), ported B's retry-by-reason, replaced #47 with a pinned force-push, merged.
- Added: Restart as administrator is refused while a record runs (`RecordState::is_running`). `SHQUERYRBINFO` checked on the laptop: Windows rejects the packed 20-byte layout, our 24-byte layout is right on x64.
- Real-app check over CDP (scratch dirs): Copy record with a locked file lists it as `fileLocked`; retrying another reason leaves it, retrying `fileLocked` copies it; Compare report; quick clean removed only files older than a day; Reports page shows all three kinds with client/ticket; HTML + CSV exports checked. Fixed mid-word breaks in the HTML export (`overflow-wrap:anywhere`). Screenshots: `V2.0/docs/screenshots/batch4-*.png`.

## Next

1. Owner pass on Batch 4 on the real laptop data (Records folder was empty before this session): Reports page with real records, each quick win (Recycle Bin size and empty, Restart as administrator prompt), 175% scaling, light and dark.
2. Batch 5 (Installer) on "go": NSIS standard + offline WebView2, portable mode with records next to the exe, `v2.*` release workflow. Decide side by side vs replace V1 first.
3. Fix #44 and #45 as one small V1 PR (ureq 3 migration; hold `typescript` < 7); add `V2.0/` npm and cargo entries to `dependabot.yml`.
4. V1: `/release-rc v1.0.0-rc5`, owner checks it, then tag `v1.0.0`. Leftovers: Docs/Support links (`src/app/appMeta.ts`), delete rc1/rc2.
5. Batch 4 follow-ups: CSV sizes as raw bytes (today "1.25 GB" text, Excel cannot sort or sum; needs a raw-value path through the four report kinds in `report-core`); reports and the `%TEMP%\DeepServer2 reports` HTML exports are never pruned; Settings (default verify level, report folder).
6. Batch 6 "Field tools": "Copy missing only" from a Compare result, pre-flight review plus conflict choice, protected files via admin/backup rights, share sign-in, CLI for RMM tools, Explorer right-click, re-check an old record against its manifest, import a robocopy log.

## Open questions for the owner

- V1 and V2 both install as "DeepServer": side by side, or should V2 replace V1? Decide before Batch 5.
- CI long term: stay public, fix billing, org trial, or self-hosted runner.
- New logo for 2.0 (the HTML report brands with the company name from Settings, else plain "DeepServer"). MFT speed vs. no elevation prompt (Cleanup).

## Blocked / needs the owner

- Delete merged remote branches (`claude/brave-allen-tiokj8`, `claude/friendly-galileo-2b664k`, `fix/menu-row-width`, `v2/batch-0-scaffold`, `v2/batch-2-record`, `v2/batch-3-cleanup`, `v2/batch-4-reports`, `v2/record-fixes`) and rc1/rc2: **Actions → Repo maintenance** (`dry_run: true` first, read the log, then run with `dry_run` off).
- Archive `open-diff` and `altWinDirStat` (ask first).

## Notes

- Defender ASR blocks Cargo build scripts outside the two excluded `target` folders. From a worktree, set `CARGO_TARGET_DIR=C:/Users/Addy7/Projects/Work/DeepServerV1.0/V2.0/src-tauri/target` (V2) or `.../DeepServerV1.0/src-tauri/target` (V1). Smart App Control is off. Windows Application Control also blocks the `deepserver_lib` test binary locally (os error 4551): run `cargo test --workspace --no-fail-fast` and let CI cover the app crate.
- The repo-root pre-push hook only checks V1. If a push changes only V2, run `corepack pnpm quality` + `corepack pnpm test` in `V2.0/` and push with `PREPUSH_SKIP=1`.
- Real-app testing without a mouse: start with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333`, drive the page over CDP (`Runtime.evaluate`, `window.__TAURI_INTERNALS__.invoke`; a Channel argument is the string `__CHANNEL__:<transformCallback(cb)>`). Point `TEMP`, `DEEPSERVER2_REPORTS_DIR` and `DEEPSERVER2_RECORDS_DIR` at a scratch folder for safe quick-clean tests. Stopping the shell can leave vite on port 1430: kill it by port. Editing Rust while `tauri dev` runs rebuilds and restarts the app.
- Cloud sessions can't build the V2 app crate (no WebKitGTK). Check Windows code with `rustup target add x86_64-pc-windows-msvc` and `CARGO_FEATURE_PURE=1 cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings`.
- Laptop runs at 175% scaling; V2's default window is 1200×720.
- The owner opts out of PR watching: don't `subscribe_pr_activity`; use `ci-watch`.
- Handoff and notes files: committed on `main` directly, no PR, when working locally.
