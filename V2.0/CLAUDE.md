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

CI: `.github/workflows/v2.yml`, only when `V2.0/**` changes; it also builds and smoke-tests both
installers (`build-v2-windows.yml`, reusable). Releases: `release-v2.yml` for `v2.*` tags. V2
**replaces** V1 in place (same name and folder). Portable mode (`portable.txt` next to the exe,
`scan_core::portable_data_dir`) moves records and reports to `Data\`. See `docs/install.md`.

## Stack and layout

- Tauri 2 + Rust, Vue 3 + TS, Tailwind CSS v4, `@lucide/vue` icons. No component library yet;
  add Reka UI when the first menu/dialog needs it.
- `src/styles.css`: all colours are `--app-*` tokens (light in `:root`, dark in
  `[data-theme='dark']`), exposed to Tailwind as `bg-card`, `text-muted`, `border-stroke`,
  `text-accent`, `text-danger`, ... Never use raw hex or Tailwind palette colours in components.
- `src/router.ts`: `sections` drives both the routes and the nav rail.
- `src/lib/`: plain TS helpers, each with a `*.test.ts` when it has logic.
- `src-tauri/crates/scan-core`: folder walker (`scan`, rayon; `list.rs` lists a folder with
  `NtQueryDirectoryFile` in bulk on Windows, std `read_dir` elsewhere; reparse points listed but
  never followed, OneDrive online-only files flagged from their attributes, never read). Every
  node has `size` (logical; Compare/Record use it) and `disk` (size on disk: clusters,
  compressed/sparse/WOF via `GetCompressedFileSizeW`, hard links counted once, cloud = 0;
  Disk Cleanup uses it). The approach is modelled on the WinDirStat fork's scanner, written
  fresh (the fork is GPL-2, V2 is Apache-2.0: never copy its source in)
  and the size compare (`compare` -> `DiffTree`). Flat arenas; children contiguous, largest first.
  `cargo test -p scan-core` runs on Linux; the perf check is
  `cargo test -p scan-core --release --test perf -- --ignored --nocapture` (2x100k files < 5 s).
- `src-tauri/crates/transfer-core`: Record, vendored from V1 and trimmed (no filters, no
  `job-core`/`logging-core`/`folder-core`; the source listing is a `scan-core` scan, see
  `walk.rs`). `prepare` lists the source, then `run::run_copy` (Copy for me) or `watch::watch`
  (another tool copies); `run::retry` and `recovery::copy_to_recovery` work on stored runs.
  Runs are files under `%LOCALAPPDATA%\DeepServer2\Records` (`DEEPSERVER2_RECORDS_DIR`
  overrides it; portable mode uses `<exe dir>\Data\Records`), never pruned (they are client proof). Copy mode lists OneDrive online-only files
  as `CloudOnly` instead of downloading them unless `download_cloud` is set; "Ignore system and
  temp files" is `scan_core::is_junk`, shared with Compare. `meta.rs` keeps created/accessed/modified
  times and attributes after each copy (`PreserveOptions` on `TransferSettings`, default on;
  folders DeepServer creates are dated last, deepest first; a failure is a warning in
  `ItemResult.message`). ACL, owner and streams are not kept yet. The app manifest
  (`src-tauri/windows/app.manifest`) sets `longPathAware`. Commands: `src-tauri/src/record.rs`,
  one record at a time, keeping the PC awake while it runs; UI `RecordPage.vue` (`/compare/record`) and `RecordReportPage.vue`
  (`/compare/record/:id`); the running job lives in `recordJob` (`src/lib/record.ts`) so leaving
  the page keeps Finish/Cancel.
- `src-tauri/crates/cleanup-core`: Disk Cleanup on a `scan-core` tree. `rows` (one folder level),
  `overview` (root totals, 100 largest files, every extension), `Protected` (what may never be
  deleted: outside the scanned folder, Windows, Program Files, Users and profile roots, known
  folders, pagefile/hiberfil, `$Recycle.Bin`, the Records folder), `drives` and `recycle`
  (`SHFileOperationW` with undo; Windows asks before deleting anything too big for the bin).
  Sizes are `Node::disk` (space on disk; OneDrive online-only files are 0).
  Deletes call `Tree::remove`, which subtracts from every folder above. Commands:
  `src-tauri/src/cleanup.rs`; UI `CleanupPage.vue` (`/cleanup`, `?scan=C:\` starts a scan) with
  `TreemapView.vue` (squarified, one level, `src/lib/treemap.ts`) and `DriveTiles.vue` (also on
  Home). The scan lives in `cleanupJob` (`src/lib/cleanup.ts`) so it survives leaving the page.
  File-type colours are the `--app-type-*` tokens; groups in `src/lib/fileTypes.ts`.
  Quick cleanups (`cleanup_core::quick`): a fixed allow-list of throw-away places (user and
  Windows temp, browser caches, Windows Update downloads, Recycle Bin, Downloads setup files over
  90 days, recycled). `Places::targets` names the folders; nothing outside them is touched, the
  folders themselves stay, temp files under a day old stay. The system ones need admin:
  `app_restart_admin` relaunches elevated (`ShellExecuteW` "runas").
  Junk finder (`junk.rs`, `cleanup_junk`): suggests temp files, crash dumps, old logs,
  thumbnails, installers, dev caches and empty folders from the scanned tree; never ticked for
  the user, `Protected` applies. Bulk delete (`bulk.rs`): `cleanup_delete_preview` is a dry run
  (statuses go / protected / reparse / gone / inside; never deletes), `cleanup_delete_many`
  deletes one by one and returns an `ItemResult` each (done, inUse, accessDenied, notFound,
  skipped, failed); junctions and symlinks are refused. Every result is appended to
  `<data dir>\Logs\deletions.jsonl` (`log.rs`, `report_core::logs_root`) and counted in
  `UsageReport.removed`. Admin gating: one item to the Recycle Bin is open to everyone;
  several items or any permanent delete return `NEEDS_ADMIN` unless `app_elevated`. The UI
  mirrors it with `canDelete` (`src/lib/cleanup.ts`): disabled buttons offer "Restart as
  administrator", and `TitleBar.vue` shows an Administrator pill. UI: `BulkDeleteDialog.vue`
  (preview, confirm with the exact list and total, per-item results; Recycle Bin is the default,
  "Delete permanently" is a separate red button) and `JunkPanel.vue`.
- `src-tauri/crates/report-core`: saved reports. Compare and Disk Cleanup results (scan, and the
  quick cleanups) are JSON files in `%LOCALAPPDATA%\DeepServer2\Reports`
  (`DEEPSERVER2_REPORTS_DIR` overrides it), saved automatically; records stay in their run
  folders, with the job details in `job.json`. `list` merges both for the Reports page. Every
  report becomes a `Document` (explicit `verdict`, tiles, tables, `sections`: folds, copy/save
  code blocks, notes) that renders as standalone HTML (branded with the company name from
  Settings) or CSV (first table, BOM, formulas defused, raw "Bytes" column after the size
  column). HTML carries one shared inline script under a hashed CSP `<meta>` (no network, no
  other scripts), a JSON data island with `< > & U+2028/9` escaped, and works with JS off
  (filter/Copy/Save are `data-js hidden` until it runs). Lists cap at 5,000 HTML rows (CSV is
  complete); print keeps the first 200 rows. Disk usage reports also carry an `Explorer` (`report-core/src/explorer.rs`: pruned size tree as a flat parent-index list, capped at 20,000 nodes, plus the biggest 200 files per type) drawn by its own second script and style (`Document.explorer`, CSP lists both hashes); `tests/fixtures/explorer.html` feeds `src/lib/reportExplorer.test.ts` (`UPDATE_FIXTURES=1 cargo test -p report-core explorer_fixture` refreshes it). Every new report field needs `#[serde(default)]`. Commands:
  `src-tauri/src/reports.rs`; UI `ReportsPage.vue`. Client / Ticket / Technician
  (`JobFields.vue`, `src/lib/job.ts`) are typed once and sent with every run.
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
