# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

(1) Finish V1: cut rc5, owner re-checks, tag `v1.0.0`. (2) DeepServer 2.0 in `V2.0/` (Compare, Record, Disk Cleanup; plan `V2.0/docs/plan.md`).

## State (2026-10-08)

- `main`: V2 Batches 0–4 merged (Reports, HTML/CSV export, quick cleanups, retry by reason: #47). Latest release **v1.0.0-rc4**; rc5 not cut. No worktrees.
- Repo is **public** (owner's choice): the private repo's Actions minutes ran out ("account payments have failed"). CI runs while public.
- **#44** (Rust) and **#45** (npm): Dependabot, unmerged. #44 breaks on `ureq::AgentBuilder` (needs a ureq 3 migration in `remote-core`); #45 breaks because typescript-eslint doesn't support TS 7 (hold `typescript` < 7). `dependabot.yml` doesn't cover `V2.0/`.

## Next

1. Owner tries Batch 4 on real laptop data (Reports with real records, Recycle Bin quick win, Restart as administrator) at 175%, light and dark.
2. Batch 5 on "go": NSIS standard + offline WebView2, portable mode (records next to the exe), `v2.*` release workflow. Decide side by side vs replace V1 first.
3. One small V1 PR for #44 and #45; add `V2.0/` to `dependabot.yml`.
4. V1: `/release-rc v1.0.0-rc5`, owner checks, tag `v1.0.0`. Leftovers: Docs/Support links (`src/app/appMeta.ts`), delete rc1/rc2.
5. Batch 4 follow-ups: CSV sizes as raw bytes (today text like "1.25 GB"; needs a raw-value path in `report-core`); prune old reports and the `%TEMP%\DeepServer2 reports` exports; Settings (default verify level, report folder).
6. Batch 6 "Field tools": Copy missing only from Compare, pre-flight review + conflict choice, admin/backup-rights reads, share sign-in, RMM CLI, Explorer right-click, re-check an old record against its manifest, robocopy log import.

## Open questions for the owner

- V2 side by side with V1, or replace it? (before Batch 5)
- CI long term: stay public, fix billing, org trial (`obsidianintelligenceyyc`), or self-hosted runner.
- New logo for 2.0. MFT speed vs. no elevation prompt (Cleanup).

## Needs the owner

- Delete merged remote branches (`claude/brave-allen-tiokj8`, `claude/friendly-galileo-2b664k`, `fix/menu-row-width`, `v2/batch-0-scaffold`, `v2/batch-2-record`, `v2/batch-3-cleanup`, `v2/batch-4-reports`, `v2/record-fixes`) and rc1/rc2: **Actions → Repo maintenance**, `dry_run: true` first.
- Archive `open-diff` and `altWinDirStat` (ask first).

## Notes

- Windows Application Control blocks the `deepserver_lib` test binary locally (os error 4551): use `cargo test --workspace --no-fail-fast`; CI covers the app crate.
- A V2-only push: run `corepack pnpm quality` + `corepack pnpm test` in `V2.0/`, push with `PREPUSH_SKIP=1`.
- Real-app testing over CDP (`--remote-debugging-port=9333`): pass a Tauri Channel as the string `__CHANNEL__:<transformCallback(cb)>`. Point `TEMP`, `DEEPSERVER2_REPORTS_DIR`, `DEEPSERVER2_RECORDS_DIR` at a scratch folder for safe cleanup tests. Stopping the shell can leave vite on port 1430 (kill by port); editing Rust during `tauri dev` restarts the app.
