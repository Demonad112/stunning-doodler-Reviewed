---
paths:
  - 'V2.0/src-tauri/crates/scan-core/**'
---

- The WinDirStat fork is GPL-2 and V2 is Apache-2.0: the scanner is modelled on its approach, written fresh. Never copy its source in.
- `Node.size` is logical (Compare/Record use it); `Node.disk` is size on disk (Disk Cleanup uses it). Reparse points are listed, never followed; OneDrive online-only files are never read.
- Detail: `V2.0/docs/architecture.md`.
