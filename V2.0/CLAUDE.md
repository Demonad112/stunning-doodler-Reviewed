# DeepServer 2.0: notes for Claude

A fresh app, separate from V1 (the repo root). Three tools: **Compare** two folders by size, **Record** a copy between them (report with missed files in red), **Disk Cleanup** (in-app analyzer). Plan: `docs/plan.md`. V2 never imports from V1 paths; code worth keeping is copied into `src-tauri/crates/` and trimmed.

## Commands (run in `V2.0/`)

```bash
corepack pnpm install
corepack pnpm quality      # prettier, vue-tsc, rustfmt, clippy -D warnings
corepack pnpm test         # vitest (jsdom)
corepack pnpm rust:test
corepack pnpm tauri dev    # dev server on port 1430, so V1 (1420) can run alongside
```

CI: `.github/workflows/v2.yml` (only when `V2.0/**` changes). Releases: `release-v2.yml` for `v2.*` tags. V2 replaces V1 in place (same name and folder). Portable mode (`portable.txt` next to the exe, `scan_core::portable_data_dir`) moves records and reports to `Data\`. See `docs/install.md`.

## Stack and layout

- Tauri 2 + Rust, Vue 3 + TS, Tailwind CSS v4, `@lucide/vue` icons.
- `src/styles.css`: all colours are `--app-*` tokens (light in `:root`, dark in `[data-theme='dark']`), exposed to Tailwind as `bg-card`, `text-muted`, `text-accent`, ... Never use raw hex or Tailwind palette colours in components.
- `src/router.ts`: `sections` drives both the routes and the nav rail.
- `src/lib/`: plain TS helpers, each with a `*.test.ts` when it has logic.
- Commands live in one file per area (`src-tauri/src/compare.rs`, `record.rs`, `cleanup.rs`, `reports.rs`). Long jobs are `async` + `spawn_blocking`, stream progress over a `tauri::ipc::Channel` every 100 ms, and keep big results in managed state: the UI fetches one folder level at a time, never the whole tree. TS wrappers in `src/lib/<area>.ts`, mocked in tests with `vi.mock`.
- Window: no OS frame (`TitleBar.vue` draws it), created hidden and transparent; `startShell()` adds `html.mica` on Windows 11 and then shows the window.

## Crates (`src-tauri/crates/`)

- `scan-core`: folder walker and size compare. Every node has `size` (logical; Compare/Record) and `disk` (size on disk; Disk Cleanup). Reparse points are listed, never followed; OneDrive online-only files are flagged, never read. Written fresh: the WinDirStat fork is GPL-2 and V2 is Apache-2.0, so never copy its source in. Perf check: `cargo test -p scan-core --release --test perf -- --ignored --nocapture`.
- `transfer-core`: Record. `prepare`, then `run::run_copy` or `watch::watch`; `run::retry` and `recovery::copy_to_recovery` work on stored runs. Runs live under `%LOCALAPPDATA%\DeepServer2\Records` (`DEEPSERVER2_RECORDS_DIR` overrides) and are never pruned. UI: `RecordPage.vue`, `RecordReportPage.vue`.
- `cleanup-core`: Disk Cleanup on a `scan-core` tree. `Protected` lists what may never be deleted; deletes go to the Recycle Bin by default. Several items or any permanent delete need admin (`NEEDS_ADMIN`, `app_restart_admin`). Every deletion is appended to `<data dir>\Logs\deletions.jsonl`. UI: `CleanupPage.vue`.
- `report-core`: saved reports as JSON in `%LOCALAPPDATA%\DeepServer2\Reports` (`DEEPSERVER2_REPORTS_DIR` overrides), rendered to standalone HTML (hashed CSP, no network) or CSV. Every new report field needs `#[serde(default)]`. Explorer fixture refresh: `UPDATE_FIXTURES=1 cargo test -p report-core explorer_fixture`.

## Linux builds

The app crate needs WebKitGTK. Without it, check with `rustup target add x86_64-pc-windows-msvc` and `cargo clippy --all-targets --target x86_64-pc-windows-msvc -- -D warnings` (this also covers `#[cfg(windows)]` code).

## Design rules

Windows 11 Fluent: Segoe UI Variable, 14px body, 28px page titles, 8px card radius, one accent, left nav with the accent pill. One obvious primary action per screen. Sizes use tabular numbers.
