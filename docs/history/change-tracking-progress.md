# SDD ledger — plan: docs/superpowers/plans/2026-09-29-change-tracking.md
Spec: docs/superpowers/specs/2026-09-29-change-tracking-design.md
Pre-flight:
- T1→T4: Significant/SignedBytes/SignedCount/ChangeColor/CLedgerListCtrl produced in T1, consumed identically in T4 — consistent.
- T2→T3/T4: ForkSettings::{TrackChanges,ShowRelativeAge}, History::Result/OnScanComplete/PublishScan — consistent; T4 replaces PublishScan body as planned.
- T3→T4: ForkFormat::RelativeAge/Now — consistent.
- T5→T4 file: ChangesTip column order matches CFileChangesView Column enum (Folder,Change,Before,Now,Delta,Files) — consistent.
Task 1: Ruling: plan's expected significant list omitted a\b\c (a leaf that grew 5 MiB on its own; spec lists it) — test expectation corrected to a\b\c|edge|gone|new — cost if wrong: none, spec is explicit.
Task 1: Ruling: upstream pre-build formatter adds UTF-8 BOMs to untouched files (ForkResource.h, ForkCommands.cpp, lang_en.txt, Stress-LargeScan.ps1) — excluded from commits unless the file is edited by the task — cost if wrong: cosmetic diff noise later.
Task 1: implemented+committed 3f151dcd; fork tests 6/6 pass; Step 8 (manual compare-dialog check) NOT done — computer-use access to dev build interrupted by user.
