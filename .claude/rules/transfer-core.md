---
paths:
  - 'V2.0/src-tauri/crates/transfer-core/**'
---

- Records (run folders under `%LOCALAPPDATA%\DeepServer2\Records`) are never pruned: they are client proof.
- Copy mode lists OneDrive online-only files as `CloudOnly` instead of downloading them, unless `download_cloud` is set.
- Detail: `V2.0/docs/architecture.md`.
