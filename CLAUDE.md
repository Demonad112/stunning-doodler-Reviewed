# DeepServer: notes for Claude

> **V2 replaces V1.** V1 (this root) is frozen: no new releases. New work is in `V2.0/` (see `V2.0/CLAUDE.md`, plan in `V2.0/docs/plan.md`). The rest of this file describes V1.

DeepServer 1.0 = OpenDiff (Tauri 2 + Vue 3 + Rust, Apache-2.0, at the repo root) + a WinDirStat fork (C++, GPL-2) bundled as the **Disk Usage engine** (`native/diskusage`, shipped as the sidecar `deepserver-diskusage.exe`). One NSIS installer, Windows 10/11 and Server.

## How work is organised

- The roadmap is `docs/history/deepserver-merge-plan.md` (Batches 0–6, error table E1–E24); `docs/history/` has older references.
- **One batch = one branch + one PR into `main`**, using `.github/pull_request_template.md`. The owner merges and says "go" before each new batch. List deviations from the plan in the PR.
- **Push and open PRs only in `Demonad112/stunning-doodler-Reviewed`.** The old `open-diff` and `altWinDirStat` repos are read-only.
- Build each batch in a git worktree under `.claude/worktrees/`; other sessions may use the main checkout.
- Cloud-session limits and clippy/`cfg(windows)` notes: `docs/cloud.md`. CI workflows: `docs/ci.md`. New session mode: `/add-session-mode`.

## Commands (repo root)

```bash
corepack pnpm install
corepack pnpm quality          # prettier, eslint, stylelint, vue-tsc, rustfmt, clippy -D warnings
corepack pnpm test:unit        # vitest (jsdom)
corepack pnpm test             # unit + Playwright e2e (needs: corepack pnpm exec playwright install chromium)
corepack pnpm tauri:dev        # desktop app; debug builds find the engine at native/diskusage/build/altWinDirStat_x64.exe
cargo test --workspace --manifest-path src-tauri/Cargo.toml
```

From a worktree, set `CARGO_TARGET_DIR` to the main checkout's `src-tauri/target` to reuse its build cache. The engine's build and tests are in `native/diskusage/CLAUDE.md`.

## Layout

- `src/api/*.ts`: typed `invoke()` wrappers, one file per area, each with a `*.test.ts`. `src/app/`: pure helpers. `src/views/`: one view per session mode. `src/stores/`: Pinia, persisted to localStorage (`open-diff-*` keys are legacy; new ones use `deepserver-*`).
- `src-tauri/src/commands.rs` and `diskusage.rs`: Tauri commands; register each in `generate_handler!` in `src-tauri/src/lib.rs`. Logic lives in `src-tauri/crates/*-core`.
- `diskusage-core`: drives, snapshot history (`%LOCALAPPDATA%\DeepServer\History`, same layout as the engine), `/saveto` and `/compare` runs. The engine is found via `DEEPSERVER_DISKUSAGE_EXE`, then next to the app exe, then (debug only) the native build output.
- i18n: 8 flat locale files in `src/i18n/locales/`; `languageSkeletons.test.ts` fails if any en-US key is missing elsewhere.

## Gotchas

- `pickNativePath()` and `detectExecutables()` return `null` outside the Tauri runtime and for anything that isn't a non-empty string.
- `tests/e2e/helpers/tauriMock.ts` answers unknown commands with `{}`. Give every command a new e2e path calls its own stub.
- npm `@tauri-apps/api` minor must match the Rust `tauri` crate (PR #48, `Demonad112/stunning-doodler-Reviewed`).
- The pre-commit hook (lint-staged) runs prettier, eslint `--fix` and `cargo fmt` on staged files; `cargo fmt` can stall while another cargo process holds the lock.
- `native/` and `docs/history/` are excluded from prettier, eslint and stylelint; `native/**` is `-text` in `.gitattributes` (CRLF and BOM files).
- Tests that re-seed module mocks per test need `vi.clearAllMocks()` in `beforeEach`, or call counts leak between tests.
- Colours: use `--ds-<role>-<hex>` / `--app-*` tokens (top of `src/styles/main.css`, light + `html[data-theme='dark']`), never raw hex; dark mode won't remap it. `src/styles/*Density.test.ts` assert on CSS source, so update them with a style change.
- Git Bash rewrites `origin/main:path`; prefix `MSYS_NO_PATHCONV=1`.
- Full `pnpm test:unit` takes >10 min locally; run it in the background or scope to the touched files.

## Session workflow

- **Start** by reading `docs/HANDOFF.md`. **End** a work block by updating it by hand (local edit; no PR or branch just for it). Reference notes go in the repo (`docs/`), never only in the scratchpad.
- **GitHub status**: `scripts/gh-status.sh release <tag> | run <id>|latest [wf] | runs [wf] | pr <n> | branches`; prefer it over connector reads (5–15k tokens each).
- **Waiting on CI**: spawn the `ci-watch` agent (Haiku) in the background with `pr <n>` or `run <id>`. Don't poll. The owner opts out of PR watching: no `subscribe_pr_activity`.
- **Releases**: `/release-rc <tag>`. **Before every push**: `scripts/prepush.sh` (the `.claude/settings.json` hook runs it; `PREPUSH_SKIP=1` bypasses).
- **Never merge on red.** A flaky test is a bug: find the race and fix it in the PR.
- `main` tracks `origin/main`; `git fetch` and fast-forward before relying on `.claude/`, `scripts/` or `docs/HANDOFF.md`.
- Don't search `src-tauri/target/`, `node_modules/`, `native/diskusage/build/` or `docs/history/opendiff/`.
- Release testing: `laptop-check` agent, then the owner's manual pass (`docs/testing-plan.md`). The laptop runs at 175% scaling; computer-use can't see WebView2 `<select>` popups or type in folder pickers.
