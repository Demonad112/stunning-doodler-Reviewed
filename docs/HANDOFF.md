# Handoff

Session state for the next Claude session. Update with `/handoff`; keep it short.

## Goal

Two tracks: (1) finish V1: cut rc5, owner re-checks, tag `v1.0.0`. (2) Build DeepServer 2.0 in `V2.0/`: Compare, Record and Disk Cleanup only, Windows 11 look. Plan: `V2.0/docs/plan.md` (Batches 0–5).

## State

- `main` at `afee64c`: V2 Batch 1 Compare (#38, #39), Batch 2 Record (#40), Record fixes (#41) and **Batch 3 Disk Cleanup (#42)** are merged. Batch 3 not yet tried by the owner in the real app. Latest release **v1.0.0-rc4**; rc5 not cut.
- Open PRs (2026-10-08), all Dependabot, none merged because CI is red: #43 `download-artifact` 7->8 (e2e `page.goto` timeouts, unrelated to the bump; failed jobs re-run once), #44 Rust group (breaks: `ureq::AgentBuilder` gone in remote-core, needs a ureq 3 migration), #45 npm group (breaks: typescript-eslint does not support TS 7.0, hold `typescript` below 7). `.github/dependabot.yml` does not cover `V2.0/`.
- The repo is **public** for now (owner's choice, 2026-10-03): the private repo's Actions minutes ran out ("recent account payments have failed").
- Worktrees `.claude/worktrees/v2-batch-2` and `v2-record-fixes`: both merged, can go.

## Done this session (laptop, 2026-10-03)

- #40 merged: Record (Watch + Copy for me, live panel, report with missed files in red, Retry, Copy missed to…). Tested in the real app with robocopy: it listed the excluded file and the locked file, and Retry copied both.
- #41: Copy for me no longer downloads OneDrive online-only files. They are listed as `CloudOnly` unless the "Download OneDrive online-only files" box is ticked; Retry and Copy missed to… skip them too.
- #41: "Ignore system and temp files" toggle, on by default and shared by Compare and Record. Filtered in `scan_core::is_junk`.
- #41: the PC stays awake while a record runs (`KeepAwake` in `record.rs`). Records are no longer pruned at startup; the prune code is deleted.
- `.claude/settings.json`: 21 read-only allow rules added (vitest, vue-tsc, `reg query`, PowerShell readers, browser/computer-use read tools).

## Next

1. Owner tries #41 and #42 in the app: Copy for me on a OneDrive folder (online-only files listed, not downloaded); Disk Cleanup scan of `C:\` and recycling one file; check boxes at 175% scaling and in dark mode.
2. Owner said "go" (2026-10-08) for: Batch A housekeeping (this handoff, Dependabot triage, Repo maintenance dry run), then V2 Batch 4 (Reports + quick wins), with V1 rc5 alongside.
3. Dependabot: #44 needs a ureq 3 migration in `remote-core`; #45 needs `typescript` held below 7; add `V2.0/` ecosystems to `dependabot.yml`.
4. Batch 4 (Reports + quick-win cleanups) also gets: client/ticket/tech fields on records and compares, per-reason retry (the backend `Selection.reason` already exists), re-checking an old record against its manifest, importing a robocopy log.
5. Batch 5 (Installer) also gets portable mode, with records saved next to the exe.
6. A new Batch 6, "Field tools": "Copy missing only" from a Compare result, a pre-flight review screen plus the conflict choice, reading protected files with admin/backup rights, signing in to a share, the CLI for RMM tools, the Explorer right-click.
7. V1: `/release-rc v1.0.0-rc5`, owner checks it, then tag `v1.0.0`. Leftovers: Docs/Support links (`src/app/appMeta.ts`), delete rc1/rc2.

## Open questions for the owner

- Keep the repo public, or move it into an organization under the Enterprise trial (`obsidianintelligenceyyc`: 3,000 min/month, 28 days left on 2026-10-03) and make it private again? Or run CI on the laptop (a self-hosted runner, which needs an ASR exclusion for its work folder)?
- V1 and V2 both install as "DeepServer": side by side, or should V2 replace V1? Decide before Batch 5.
- Report branding/logo (Batch 4). MFT speed vs. no elevation prompt (Batch 3).

## Blocked / needs the owner

- New logo for 2.0.
- Delete merged branches (`claude/brave-allen-tiokj8`, `fix/menu-row-width`, `v2/batch-0-scaffold`, `v2/batch-2-record`, `v2/record-fixes`) and rc1/rc2: **Actions → Repo maintenance** (`dry_run: true` first).
- Then remove the worktrees: `git worktree remove .claude/worktrees/v2-batch-2` and `git worktree remove .claude/worktrees/v2-record-fixes`.
- Archive `open-diff` and `altWinDirStat` (ask first).

## Notes

- Defender ASR blocks Cargo build scripts outside the two excluded `target` folders. From a worktree, set `CARGO_TARGET_DIR=C:/Users/Addy7/Projects/Work/DeepServerV1.0/V2.0/src-tauri/target` (V2) or `.../DeepServerV1.0/src-tauri/target` (V1). Smart App Control is off.
- The repo-root pre-push hook only checks V1. If a push changes only V2, run `corepack pnpm quality` + `corepack pnpm test` in `V2.0/` and push with `PREPUSH_SKIP=1` (from a worktree its clippy builds into an unexcluded `target` and fails with os error 5).
- Testing the real app without screenshots: start it with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333`, then drive the page over CDP (`Runtime.evaluate`, `window.__TAURI_INTERNALS__.invoke`).
- Laptop runs at 175% scaling; V2's default window is 1200×720.
- Handoff and notes files: commit on `main` directly, no PR (owner's call).
