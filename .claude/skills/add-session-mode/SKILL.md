---
name: add-session-mode
description: Checklist for adding a new session mode to the V1 app (root Tauri/Vue app), plus the ESLint rules that often trip. Run as /add-session-mode.
disable-model-invocation: true
---

Grep for an existing mode (e.g. `version-compare`) and mirror it in:

- `types/session.ts` and its test
- `sessionCatalog.ts` and its test
- `sessionFactory.ts`
- `router.ts`
- `commandRegistry.ts`: every Ctrl+Alt letter is taken, so use Ctrl+Shift
- `AppLayout.vue`: menus, `sessionIcon`, nav group
- `HomeView.vue`: `homeTileGroups`
- `shellChrome.ts`: help topic
- all 8 locales in `src/i18n/locales/` (`languageSkeletons.test.ts` fails if any en-US key is missing)
- the Rust `session-core` `SessionType` enum, plus the `cli-core` label match

`statusBarPhrases.ts` is needed only for a custom status bar.

## ESLint rules that often trip

- `restrict-template-expressions` (use `String(n)`)
- `explicit-function-return-type` (also in tests)
- `prefer-nullish-coalescing`
- `no-unnecessary-condition` (array indexing is typed non-undefined)
- `consistent-type-imports` (no `import()` types in `vi.mock`)
