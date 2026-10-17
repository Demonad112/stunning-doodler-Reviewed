# Adding a session mode

Grep for an existing mode (e.g. `version-compare`) and mirror it in:

- `types/session.ts` and its test
- `sessionCatalog.ts` and its test
- `sessionFactory.ts`
- `router.ts` (`src/app/router.ts`)
- `commandRegistry.ts`: every Ctrl+Alt letter is taken, so use Ctrl+Shift
- `AppLayout.vue`: menus, `sessionIcon`, nav group
- `HomeView.vue`: `homeTileGroups`
- `shellChrome.ts`: help topic
- all 8 locales
- the Rust `session-core` `SessionType` enum, plus the `cli-core` label match

`statusBarPhrases.ts` is needed only for a custom status bar.
