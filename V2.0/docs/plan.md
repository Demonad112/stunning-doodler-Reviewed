# DeepServer 2.0 — fresh, focused, client-grade

## Context

V1 (rc4) merged OpenDiff (13 compare modes) + a WinDirStat fork. In front of clients it looks cheap:
crowded ribbons, 30+ item menus with truncated labels, a dated tile grid on Home, and Disk Usage
pops a separate 2005-style window with an elevation prompt. The owner wants V2.0 to do **three
things well** and look like a professional Windows 11 tool:

1. **Compare** two folders by size — pick left, pick right, done.
2. **Record** a copy between those same two folders — live progress, then a report with every
   missed/failed file in red.
3. **Clean** — a full disk-space analyzer (treemap, biggest folders/files, file types, cleanup)
   built into the app, no WinDirStat window, no elevation popup.

Decisions (owner, 2026-10-02): in-app analyzer (drop the GPL engine) · drop every other compare
mode · lives in `V2.0/` in this repo · Windows 11 native look (Fluent/Mica).

V1 stays at the repo root (rc5/1.0.0 can still ship from it).

### V1 dropdown bug (screenshot 3) — root cause found, NOT fixed in rc5

`src/layouts/AppLayout.vue:2548` `.menus button { max-width: 9em; overflow: hidden;
text-overflow: ellipsis }` is meant for the top-level menu buttons but also hits every row inside
the dropdown panel (the panel lives inside `nav.menus`). Rows are capped at ~108px, the shortcut
column keeps its width, so labels collapse to "T." / "Show ..." and long shortcuts show alone.
PR #31 only widened the panel. Also hit by `:2416` (fixed 21.5px height) and `:3358`.
Fix (tiny V1 PR `fix/menu-row-width`, before rc5): scope those three rules to
`.menus > .menu-group > button`, update the density test that pins them. V2 shares nothing at
runtime: it gets its own Tauri app, Cargo workspace, package.json, CI job and identifier.

## Stack (fresh, small)

- **Tauri 2 + Rust** backend (keeps the proven installer/WebView2 path; Windows 10/11/Server).
- **Vue 3 + TypeScript + Vite**, **Tailwind CSS v4** with design tokens, **Reka UI**
  (headless, accessible primitives: menus, dialogs, tabs, tooltips — no truncation bugs, keyboard
  OK), **lucide-vue-next** icons, **Pinia** only if a store is really shared.
- Window: `decorations: false` custom title bar + `windowEffects: ["mica"]` with a solid
  token background fallback (Win10/Server have no Mica).
- Typography: Segoe UI Variable (system), tabular numbers for sizes. One accent colour
  (DeepServer blue), light/dark/system theme via CSS variables only.
- No i18n in the first release (en-US only, strings in one file so it can be added later).

Skipped vs V1: 8 locales, session catalog, command registry, ribbons, 30 crates, e2e mock
framework. Add when a real need appears.

## App shape

```
┌ title bar (logo · DeepServer · machine name · theme · min/max/close) ─────────┐
│ nav rail │  page                                                             │
│  Home    │                                                                   │
│  Compare │                                                                   │
│  Cleanup │                                                                   │
│  Reports │                                                                   │
│  ⚙       │                                                                   │
└──────────┴───────────────────────────────────────────────────────────────────┘
```

- **Home**: 3 large cards (Compare & Record · Disk Cleanup · Recent reports) + drive tiles with
  used/free bars for this machine. Nothing else.
- **Compare & Record** (one page, two steps):
  - Top: two path pickers side by side (Browse + type/paste + drag-drop), swap button,
    one primary button **Compare**.
  - Result: summary strip (Source size · Destination size · difference · files only in source in
    red) + a tree table: Name · Source size · Dest size · Δ · status chip. Filters: All /
    Different / Missing. Folder rows sum recursively.
  - Secondary button **Record copy** on the same page (same two paths): modes
    _Watch_ (another tool/robocopy/Explorer copies; we record) and _Copy for me_. Live panel below:
    progress bar, files/bytes done, speed, ETA, current file. When it ends → verification pass →
    **Report**: totals, verified OK (green), missing/failed/size-mismatch rows in **red** with
    reason, actions _Retry missed_, _Copy missed to…_, _Export HTML/CSV_.
- **Disk Cleanup**: pick drive/folder → fast scan with streaming progress → treemap (canvas,
  squarified) + biggest folders list + file-types breakdown + "Largest files" + quick-win cards
  (Temp, Windows Update cache, Recycle Bin, browser caches, old installers, duplicate-size
  candidates). Actions: Open in Explorer, Recycle, Delete (confirm dialog with total), Rescan.
  Snapshot + "what changed since last scan" kept as a later phase.
  Runs unelevated; system-wide cleanups (Windows Update cache, other users' temp) show a
  "Restart as administrator" button instead of a startup popup.
- **Reports**: list of saved Compare/Record/Cleanup reports (open, export, delete).
- **Settings**: theme, default verify level (size / size+time / hash), report folder, about.

## Backend (reuse what works, trimmed)

Vendored into `V2.0/src-tauri/crates/` (copied, then trimmed — V2 never depends on V1 paths):

- `transfer-core` (4.3k LOC, V1 Transfer Monitor stages 1–4). It already is the "Record"
  feature: _Watch_ mode (notify + polling fallback) marks every source file missing at the
  destination as NotCopied and size/time mismatches as VerifyMismatch; _Copy_ mode copies via
  `.odpart` + rename with retry and blake3 verify; each `ItemResult` has `reason`, `osCode`,
  `recoverable`; `run::retry` and `recovery::copy_to_recovery` exist; runs persist as JSONL.
  Its sibling imports are few: `folder_core::{walk_local_folder, FileFilters,
FolderCompareOptions, FolderWalkEntry}` (run.rs, prepare.rs), `file_core::path_volume_info`,
  `job_core::CancellationToken`, `logging_core` log types, `report_core` transfer HTML.
  Vendor `job-core` (170 LOC) and `logging-core` (84) as-is; replace the folder/file-core calls
  with `scan-core`; port `report-core/src/transfer.rs` into a branded report template.
  V1 Tauri glue to copy from: `src-tauri/src/transfer.rs` (commands, `TransferJobs`, 100 ms
  throttled events).
- New `scan-core`: parallel directory walker (std + rayon or `jwalk`) producing a size tree with
  progress callbacks; used by Compare (two trees, diffed by relative path) and Cleanup (one tree).
  Compare rolls folder sizes up recursively (V1 never did: folder rows showed the metadata size,
  hence "4096 bytes" / "0 bytes"). Plain `FindFirstFileExW` walk first (large fetch, basic
  info); NTFS MFT reading (WizTree speed, what the C++ engine's `FinderNtfs.cpp` does) only if
  full-drive scans are too slow, and only when already elevated.
- New `cleanup-core`: known-safe cleanup locations + Recycle via `SHFileOperation`/`IFileOperation`
  (send to Recycle Bin, never hard delete without explicit confirm).
- Tauri commands, one file per area (`compare.rs`, `record.rs`, `cleanup.rs`), progress via
  `tauri::ipc::Channel` (streamed, cancellable) instead of global events.

## Build order (one PR each into `main`, worktree per batch, owner says "go")

| #   | Batch                                                                                                                                                                                                        | Done when                                                                                                             |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| —   | V1 hotfix `fix/menu-row-width` (above), then rc5 as planned                                                                                                                                                  | View menu readable at 175%                                                                                            |
| 0   | Scaffold `V2.0/`: this plan saved as `V2.0/docs/plan.md`, `V2.0/CLAUDE.md`, Tauri app, identifier `com.demonad112.deepserver2`, Tailwind+Reka, shell (title bar, nav rail, theme), CI job gated on `V2.0/**` | `pnpm tauri dev` shows the empty shell in light/dark, CI green                                                        |
| 1   | `scan-core` + Compare page                                                                                                                                                                                   | Two folders compared, sizes + missing in red, 100k-file folder < few s                                                |
| 2   | Record: vendored `transfer-core`, Watch + Copy modes, live panel, report page                                                                                                                                | robocopy into dest while recording → report lists every skipped file in red; Retry works                              |
| 3   | Disk Cleanup: drive tiles, scan, treemap, lists, file types, recycle/delete                                                                                                                                  | Scan C:\ with progress, delete a file to Recycle Bin, treemap updates                                                 |
| 4   | Quick-win cleanups + Reports page + branded HTML/CSV export                                                                                                                                                  | Report opens from history, export looks client-ready                                                                  |
| 5   | Installer (NSIS, standard + offline WebView2), portable zip, release workflow for `v2.*` tags                                                                                                                | Replaces V1 in place (same name and folder); owner decision 2026-10-08. Portable mode: `portable.txt` next to the exe |

## Design research applied

- WizTree/TreeSize/SpaceSniffer: speed first, treemap + sorted list side by side, file-type
  colours; TreeSize is the reference for "proper Windows app" polish.
- Windows 11 Fluent: Mica backdrop, 8px radius cards, generous spacing, one accent, subtle
  elevation, left navigation, no ribbon. Every screen has one obvious primary action.

## Verification

- `cargo test --workspace` in `V2.0/src-tauri` (scan tree sums, compare diff, transfer-core's
  existing tests carried over, cleanup path allow-list).
- `pnpm test:unit` (vitest) for pure helpers (size formatting, treemap layout, diff rows).
- Manual on the laptop (175% scaling, light + dark): each batch's "Done when" row; screenshots
  attached to the PR.
- CI: new `v2.yml` job (quality + tests + `tauri build`) only when `V2.0/**` changes.

## Not now

Text/hex/picture/registry/table/version compare, merge, sync, archives, cloud paths, CLI,
Explorer context menus, 8 locales, snapshot history, MFT scanning. Each can come back later if a
client job needs it.
