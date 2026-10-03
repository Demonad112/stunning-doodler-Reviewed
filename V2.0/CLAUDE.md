# DeepServer 2.0: notes for Claude

A fresh app, separate from V1 (the repo root). Three tools only: **Compare** two folders by size,
**Record** a copy between them (report with missed files in red), **Disk Cleanup** (in-app
analyzer). Plan and batch order: `docs/plan.md`. One batch = one branch + one PR into `main`,
same rules as the root `CLAUDE.md` (worktree, owner says "go", push only to
`Demonad112/stunning-doodler-Reviewed`).

V2 never imports from V1 paths. Code worth keeping from V1 is copied into `src-tauri/crates/` and
trimmed.

## Commands (run in `V2.0/`)

```bash
corepack pnpm install
corepack pnpm quality      # prettier, vue-tsc, rustfmt, clippy -D warnings
corepack pnpm test         # vitest (jsdom)
corepack pnpm rust:test
corepack pnpm tauri dev    # dev server on port 1430, so V1 (1420) can run alongside
```

CI: `.github/workflows/v2.yml`, only when `V2.0/**` changes.

## Stack and layout

- Tauri 2 + Rust, Vue 3 + TS, Tailwind CSS v4, `@lucide/vue` icons. No component library yet;
  add Reka UI when the first menu/dialog needs it.
- `src/styles.css`: all colours are `--app-*` tokens (light in `:root`, dark in
  `[data-theme='dark']`), exposed to Tailwind as `bg-card`, `text-muted`, `border-stroke`,
  `text-accent`, `text-danger`, ... Never use raw hex or Tailwind palette colours in components.
- `src/router.ts`: `sections` drives both the routes and the nav rail.
- `src/lib/`: plain TS helpers, each with a `*.test.ts` when it has logic.
- `src-tauri/crates/scan-core`: folder walker (`scan`, std `read_dir` + rayon, reparse points
  listed but never followed, OneDrive online-only files flagged from their attributes, never read)
  and the size compare (`compare` -> `DiffTree`). Flat arenas; children contiguous, largest first.
  `cargo test -p scan-core` runs on Linux; the perf check is
  `cargo test -p scan-core --release --test perf -- --ignored --nocapture` (2x100k files < 5 s).
- `src-tauri/crates/transfer-core`: Record, vendored from V1 and trimmed (no filters, no
  `job-core`/`logging-core`/`folder-core`; the source listing is a `scan-core` scan, see
  `walk.rs`). `prepare` lists the source, then `run::run_copy` (Copy for me) or `watch::watch`
  (another tool copies); `run::retry` and `recovery::copy_to_recovery` work on stored runs.
  Runs are files under `%LOCALAPPDATA%\DeepServer2\Records` (`DEEPSERVER2_RECORDS_DIR`
  overrides it), pruned at startup after 90 days / 200 runs. Commands: `src-tauri/src/record.rs`,
  one record at a time; UI `RecordPage.vue` (`/compare/record`) and `RecordReportPage.vue`
  (`/compare/record/:id`); the running job lives in `recordJob` (`src/lib/record.ts`) so leaving
  the page keeps Finish/Cancel.
- Commands live in one file per area (`src-tauri/src/compare.rs`). Long jobs are `async` +
  `spawn_blocking`, stream progress over a `tauri::ipc::Channel` every 100 ms, and keep big
  results in managed state: the UI fetches one folder level at a time (`compare_children`), never
  the whole tree. TS wrappers in `src/lib/<area>.ts`, mocked in tests with `vi.mock`.
- The app crate needs WebKitGTK to build on Linux. In a cloud container without it, check it with
  `rustup target add x86_64-pc-windows-msvc` and
  `cargo clippy --all-targets --target x86_64-pc-windows-msvc -- -D warnings` (also covers
  `#[cfg(windows)]` code).
- Window: no OS frame (`TitleBar.vue` draws it), created hidden and transparent; `startShell()`
  adds `html.mica` on Windows 11 and then shows the window. Windows 10/Server keep the solid
  `--app-bg`.

## Design rules

Windows 11 Fluent: Segoe UI Variable, 14px body, 28px page titles, 8px card radius, one accent,
left nav with the accent pill. One obvious primary action per screen. Sizes use tabular numbers.
Check at 175% scaling and in both themes before a PR.
