# DeepServer: notes for Claude

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

Grep for an existing mode (e.g. `version-compare`) and mirror it in:

- `types/session.ts` and its test
- `sessionCatalog.ts` and its test
- `sessionFactory.ts`
- `router.ts`
- `commandRegistry.ts`: every Ctrl+Alt letter is taken, so use Ctrl+Shift
- `AppLayout.vue`: menus, `sessionIcon`, nav group
- `HomeView.vue`: `homeTileGroups`
- `shellChrome.ts`: help topic
- all 8 locales
- the Rust `session-core` `SessionType` enum, plus the `cli-core` label match

`statusBarPhrases.ts` is needed only for a custom status bar.

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
- CI (`.github/workflows/`):
  - `ci.yml`: quality + tests on every PR; a `changes` job gates the heavy Windows jobs by path
  - `build-windows.yml` (reusable, called by `ci.yml` and `release.yml`): engine build, `Test-ForkChanges.ps1`, the real-engine `diskusage-core` test, the standard installer (`tauri build`) and the offline one (`tauri bundle` + `tauri.offline.conf.json`), the portable zip, `SHA256SUMS.txt`, and `scripts/windows/Test-Installer.ps1` against both installers
  - `ci.yml` job `engine-upstream-tests` (only when `native/` changes): the upstream PS 7.6 suites and the stress test
  - `release.yml`: `v*` tags only (or a manual run for an existing tag); see `docs/releasing.md`. Optional signing through `scripts/windows/sign.ps1`
  - `prune-storage.yml`: manual only
