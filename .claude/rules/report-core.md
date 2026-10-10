---
paths:
  - 'V2.0/src-tauri/crates/report-core/**'
---

- Reports are standalone HTML: no network, no external assets. Escape every file name (untrusted input) wherever it reaches HTML or a generated script; the JSON data island escapes `< > & U+2028/9`.
- Every new report field needs `#[serde(default)]`.
- Detail: `V2.0/docs/architecture.md`.
