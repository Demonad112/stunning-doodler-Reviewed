---
paths:
  - 'V2.0/src-tauri/crates/cleanup-core/**'
---

- Anything in the `Protected` list (outside the scanned folder, Windows, Program Files, profile roots, known folders, pagefile/hiberfil, `$Recycle.Bin`, the Records folder) must never be deletable, in every path (quick, junk, bulk).
- Deletes go through `Tree::remove`, which subtracts from every folder above. Junctions and symlinks are refused. Junk suggestions are never pre-ticked.
- Detail: `V2.0/docs/architecture.md`.
