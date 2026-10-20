# DeepServer: notes for Claude

> **V2 replaces V1.** V1 (this root) is frozen: no new releases. New work is in `V2.0/` (see `V2.0/CLAUDE.md`, plan in `V2.0/docs/plan.md`). The rest of this file describes V1.

DeepServer 1.0 = OpenDiff (Tauri 2 + Vue 3 + Rust, Apache-2.0, at the repo root) + a WinDirStat fork (C++, GPL-2) bundled as the **Disk Usage engine** (`native/diskusage`, shipped as the sidecar `deepserver-diskusage.exe`). One NSIS installer, Windows 10/11 and Server.

The roadmap is `docs/history/deepserver-merge-plan.md`. PRs go to `Demonad112/stunning-doodler-Reviewed`; the old `open-diff` and `altWinDirStat` repos are read-only.

## Commands (repo root)

```bash
corepack pnpm install
corepack pnpm quality          # prettier, eslint, stylelint, vue-tsc, rustfmt, clippy -D warnings
corepack pnpm test:unit        # vitest (jsdom)
corepack pnpm test             # unit + Playwright e2e (needs: corepack pnpm exec playwright install chromium)
corepack pnpm tauri:dev        # desktop app; debug builds find the engine at native/diskusage/build/altWinDirStat_x64.exe
cargo test --workspace --manifest-path src-tauri/Cargo.toml
```

The engine's build and tests are in `native/diskusage/CLAUDE.md`.

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
- The pre-commit hook (lint-staged) runs prettier, eslint `--fix` and `cargo fmt` on staged files.
- ESLint rules that often trip: `restrict-template-expressions` (use `String(n)`), `explicit-function-return-type` (also in tests), `prefer-nullish-coalescing`, `no-unnecessary-condition`, `consistent-type-imports` (no `import()` types in `vi.mock`).
- `native/` and `docs/history/` are excluded from prettier, eslint and stylelint, and `native/**` is `-text` in `.gitattributes` (CRLF and BOM files).
- Tests that re-seed module mocks per test need `vi.clearAllMocks()` in `beforeEach`.
- Colours: chrome colours are `--ds-<role>-<hex>` / `--app-*` tokens (top of `src/styles/main.css`). Don't add raw hex in styles; dark mode won't remap it. `src/styles/*Density.test.ts` assert on CSS source, so update them with a style change.
- Bash tool on Windows: heredocs halve backslashes, so write files containing backslashes with Write/Edit.
- Full `pnpm test:unit` is slow; scope it to the touched files.
- CI workflows and the cloud-session limits table: `docs/ci.md`; releases: `docs/releasing.md`.
