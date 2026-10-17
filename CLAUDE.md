# DeepServer: notes for Claude

> **V2 replaces V1.** V1 (this root) is frozen: no new releases. New work is in `V2.0/` (see `V2.0/CLAUDE.md`, plan in `V2.0/docs/plan.md`, state in `docs/HANDOFF.md`). The rest of this file describes V1.

DeepServer 1.0 = OpenDiff (Tauri 2 + Vue 3 + Rust, Apache-2.0, at the repo root) + a WinDirStat fork (C++, GPL-2) bundled as the **Disk Usage engine** (`native/diskusage`, shipped as the sidecar `deepserver-diskusage.exe`). One NSIS installer, Windows 10/11 and Server.

## How work is organised

- The roadmap is `docs/history/deepserver-merge-plan.md`: Batches 0–6 plus the error table E1–E24. Check the batch's section there before starting; `docs/history/` also has the Transfer Monitor plan and older references.
- **One batch = one branch + one PR into `main`**, using `.github/pull_request_template.md`. The owner merges and says "go" before each new batch. Deviations from the plan text are listed in the PR.
- **Push and open PRs only in `Demonad112/stunning-doodler-Reviewed`.** The old `open-diff` and `altWinDirStat` repos are read-only.
- Build each batch in a git worktree under `.claude/worktrees/`; other sessions may be using the main checkout.

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

- `src/api/*.ts`: typed `invoke()` wrappers, one file per area, each with a `*.test.ts`. `src/app/`: pure helpers. `src/views/`: one view per session mode. `src/stores/`: Pinia, persisted to localStorage (`open-diff-*` keys are legacy names; new ones use `deepserver-*`).
- `src-tauri/src/commands.rs` and `diskusage.rs`: Tauri commands. Register each one in `generate_handler!` in `src-tauri/src/lib.rs`. Logic lives in `src-tauri/crates/*-core`.
- `diskusage-core`: drives, snapshot history (`%LOCALAPPDATA%\DeepServer\History`, the same layout the engine writes), `/saveto` and `/compare` runs. The engine is found through `DEEPSERVER_DISKUSAGE_EXE`, then next to the app exe, then (debug builds only) the native build output.
- i18n: 8 flat locale files in `src/i18n/locales/`. `languageSkeletons.test.ts` fails if any en-US key is missing elsewhere.

## Adding a session mode

Follow `docs/add-session-mode.md` (the file list, mirrored from an existing mode).

## Gotchas

- `pickNativePath()` returns `null` outside the Tauri runtime and for anything that isn't a non-empty string. The same goes for `detectExecutables()`.
- `tests/e2e/helpers/tauriMock.ts` answers unknown commands with `{}`. Give every command a new e2e path calls its own stub.
- The pre-commit hook (lint-staged) runs prettier, eslint `--fix` and `cargo fmt` on staged files. `cargo fmt` can stall while another cargo process holds the lock.
- ESLint rules that often trip:
  - `restrict-template-expressions` (use `String(n)`)
  - `explicit-function-return-type` (also in tests)
  - `prefer-nullish-coalescing`
  - `no-unnecessary-condition` (array indexing is typed non-undefined)
  - `consistent-type-imports` (no `import()` types in `vi.mock`)
- `native/` and `docs/history/` are excluded from prettier, eslint and stylelint, and `native/**` is `-text` in `.gitattributes` (CRLF and BOM files).
- Tests that re-seed module mocks per test need `vi.clearAllMocks()` in `beforeEach`, or call counts leak between tests.
- Colours: chrome colours are `--ds-<role>-<hex>` / `--app-*` tokens (top of `src/styles/main.css`, light + `html[data-theme='dark']`). Don't add raw hex in styles; dark mode won't remap it. `src/styles/*Density.test.ts` assert on CSS source, so update them with a style change.
- Bash tool: heredocs halve backslashes; write files containing backslashes with Write/Edit. Git Bash rewrites `origin/main:path`; prefix `MSYS_NO_PATHCONV=1`.
- Full `pnpm test:unit` takes >10 min locally; run it in the background or scope to the touched files.
- CI workflows and the cloud-session limits table: `docs/ci.md`.

## Session workflow (saves tokens)

- **Start** by reading `docs/HANDOFF.md`. **End** each work block by updating it with `/handoff` (local edit; don't open a PR or branch just for it). This is the one committed handoff file in this repo; it is the project exception to the global `/relay-handoff` rule. Reference notes go in the repo (`docs/`), never only in the scratchpad: containers restart.
- **GitHub status**: `scripts/gh-status.sh release <tag> | run <id>|latest [wf] | runs [wf] | pr <n> | branches`. Prefer it over connector reads (releases, jobs, logs return 5–15k tokens).
- **CI after a PR**: bind the PR with `ccd_pr`, call `set_monitor` once with auto-fix, and let the app wake the session. Never poll CI, and never use a subagent to watch it.
- **Releases**: `/release-rc <tag>` (dispatches `release.yml` with `create_tag: true`).
- **Before every push**: `scripts/prepush.sh` (the `.claude/settings.json` hook runs it on `git push`; `PREPUSH_SKIP=1` in the command bypasses). Code under `#[cfg(windows)]` isn't seen by Linux clippy: re-read it by hand, or `PREPUSH_WINDOWS=1` where the msvc target is installed.
- **Never merge on red.** A flaky test is a bug: find the race and fix it in the PR.

Cloud session limits (blocked pushes, deletes and edits, with the replacement for each) are in `docs/ci.md`.

## Local setup and plan (Windows laptop)

- `main` tracks `origin/main`; run `git fetch` and fast-forward before relying on `.claude/`, `scripts/` or `docs/HANDOFF.md`. Fork-branch tooling (`/release-rc`, `/handoff`) is already in the repo.
- Locally `gh` is authenticated (`Demonad112`), so use `scripts/gh-status.sh` or `gh` through Bash instead of the GitHub connector (it fails here: bad auth header).
- Analysis passes: low effort, `git` + `gh` only; don't load the root-level `*-reference.md` handoff files unless needed. Archive them under `~/.claude/handoffs/`.
- Don't search `src-tauri/target/`, `node_modules/`, `native/diskusage/build/` or `docs/history/opendiff/`.
- Release testing: `laptop-check` agent (scripted half), then the owner's manual pass; plan in `docs/testing-plan.md`, current step in `docs/HANDOFF.md`.
- Laptop runs at 175% scaling. Computer-use can't see WebView2 `<select>` popups or type in folder pickers; type paths into the app's path fields.
