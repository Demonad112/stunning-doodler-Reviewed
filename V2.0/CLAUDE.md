# DeepServer 2.0: notes for Claude

A fresh app, separate from V1 (the repo root). Three tools only: **Compare** two folders by size, **Record** a copy between them (report with missed files in red), **Disk Cleanup** (in-app analyzer). Plan and batch order: `docs/plan.md`. One batch = one branch + one PR into `main`. Same batch/PR rules as the root file.

V2 never imports from V1 paths. Code worth keeping from V1 is copied into `src-tauri/crates/` and trimmed.

## Commands (run in `V2.0/`)

```bash
corepack pnpm install
corepack pnpm quality      # prettier, vue-tsc, rustfmt, clippy -D warnings
corepack pnpm test         # vitest (jsdom)
corepack pnpm rust:test
corepack pnpm tauri dev    # dev server on port 1430, so V1 (1420) can run alongside
```

CI: `.github/workflows/v2.yml`, only when `V2.0/**` changes; it also builds and smoke-tests both installers (`build-v2-windows.yml`, reusable). V2 **replaces** V1 in place (same name and folder). Portable mode (`portable.txt` next to the exe, `scan_core::portable_data_dir`) moves records and reports to `Data\`. See `docs/install.md`.

Release: `gh workflow run release-v2.yml -f tag=v2.0.0-rcN -f create_tag=true`, then watch the run with the `ci-watch` agent. Never reuse `v2.0.0-rc2` (a stray tag on the broken TypeScript 7 build); the next candidate is rc4 or later. TypeScript is pinned to 5.9.3 because vue-tsc 3.x can't load 7.x; Dependabot is told to skip TypeScript majors, so keep it that way.

## Stack and layout

- Tauri 2 + Rust, Vue 3 + TS, Tailwind CSS v4, `@lucide/vue` icons. No component library yet; add Reka UI when the first menu/dialog needs it.
- `src/styles.css`: all colours are `--app-*` tokens (light in `:root`, dark in `[data-theme='dark']`), exposed to Tailwind as `bg-card`, `text-muted`, `border-stroke`, `text-accent`, `text-danger`, ... Never use raw hex or Tailwind palette colours in components.
- `src/router.ts`: `sections` drives both the routes and the nav rail. `src/lib/`: plain TS helpers, each with a `*.test.ts` when it has logic.
- Commands live in one file per area (`src-tauri/src/compare.rs`); long jobs are `async` + `spawn_blocking` and stream progress over a `tauri::ipc::Channel`. TS wrappers in `src/lib/<area>.ts`, mocked in tests with `vi.mock`.
- Crates in `src-tauri/crates/` (full descriptions: `docs/architecture.md`; per-crate rules load from `.claude/rules/` when you touch them):
  - `scan-core`: folder walker and size compare. GPL-2 fork's source must never be copied in.
  - `transfer-core`: Record. Records are never pruned (client proof).
  - `cleanup-core`: Disk Cleanup. `Protected` list; deletes go through `Tree::remove`.
  - `report-core`: saved reports (HTML/CSV). Escape every file name in HTML.
- The app crate needs WebKitGTK on Linux; clippy workaround for cloud containers: repo-root `docs/cloud.md`.

## Design rules

Windows 11 Fluent: Segoe UI Variable, 14px body, 28px page titles, 8px card radius, one accent, left nav with the accent pill. One obvious primary action per screen. Sizes use tabular numbers. Check at 175% scaling and in both themes before a PR.
