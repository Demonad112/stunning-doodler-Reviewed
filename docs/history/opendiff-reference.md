> This file was compiled by an assistant from a prior conversation on this
> project. Treat it as background material, not verified fact — reason
> independently before relying on anything here, especially items presented
> as correct, decided, or working. It's meant to bring a new collaborator up
> to speed, not to define how they should proceed.

# Reference: OpenDiff Windows installer build + planned transfer-monitor features

Written 2026-09-29. Machine: Windows 11 Home, user `Addy7`. Working folder: `%USERPROFILE%\Projects\Work\ODiff` (a git checkout of the user's fork).

---

## Quick Recap

The user forked an open-source Beyond Compare alternative, **Open Diff** (Tauri 2 + Vue 3 + Rust workspace, v1.2.0), at https://github.com/Demonad112/open-diff (fork of `kygo8/open-diff`). They wanted it packaged as a locally installable Windows `.exe` with all features usable, and installable on client machines without anyone needing Rust. Because of that, compilation happens on **GitHub Actions**, not locally, and the finished installer is downloaded.

In this session: an NSIS all-users `setup.exe` (plus the existing MSI and a portable zip) was added to the build, along with an Explorer right-click menu and a desktop shortcut. It was built on GitHub, downloaded, and installed on the user's PC. The GUI launches and Folder Compare produced results. Not everything was tested: a Text Compare run appeared stuck on "Comparing…", and most other modes, the right-click flow, and uninstall were not exercised.

The user also wants a **new feature set** for file transfers (live monitoring of a destination folder, flagging files that failed to copy with reasons, and an easy way to collect the un-copied files). That has not been started; it is intended to be planned in the next chat.

---

## Working Context

### What the app is

- Frontend: Vue 3 + Vite + naive-ui + Pinia, pnpm 11.9 via corepack (`packageManager` pin). Node 24 is installed locally.
- Backend: Tauri 2.11. `src-tauri/` is a Cargo workspace with ~30 `*-core` crates (diff-core, folder-core, sync-core, job-core, report-core, snapshot-core, policy-core, shell-core, vfs-core, remote-core, update-core, logging-core, and others). Two binaries: `open-diff-app` (GUI) and `open-diff-cli`.
- Modes visible on the Home screen: Folder Compare / Merge / Sync, Text Compare / Merge / Edit, Hex, Media, Picture, Registry, Table, Version compare. Sessions, HTML/CSV/Markdown reports, and scripting exist.
- Many repo docs under `docs/` are in Chinese. README is in English (with translations).

### Packaging decisions made this session

- Build on GitHub Actions because the user does not want Rust installed locally or on client machines. (Rust is **not** installed on this PC. Node, pnpm, git, gh, VS 2022 MSVC, WebView2 are.)
- `src-tauri/tauri.conf.json`: `bundle.targets` changed from `["msi"]` to `["nsis","msi"]`; added `bundle.windows.nsis` = `{installMode: "perMachine", installerIcon: "icons/icon.ico", compression: "lzma", installerHooks: "windows/installer-hooks.nsh"}`.
- `src-tauri/windows/installer-hooks.nsh` (new): on post-install writes HKLM `Software\Classes\{*,Directory}\shell\{OpenDiff,OpenDiffSelectLeft}` keys (labels "Compare with Open Diff", "Select Left File/Folder for Compare"; commands `"$INSTDIR\open-diff-app.exe" --shell-compare [--select-left] "%1"`), creates a desktop shortcut `$DESKTOP\${PRODUCTNAME}.lnk`; post-uninstall removes them. Key names deliberately mirror `scripts/windows/register-shell-extension.ps1` and `shell-core`, which use HKCU; the installer uses HKLM so it applies to all users.
- `.github/workflows/windows-installer.yml` (new): windows-latest; runs on `workflow_dispatch` and pushes to `windows-installer`; steps: checkout, pnpm/node setup, Rust toolchain (cached), `pnpm install --frozen-lockfile`, `pnpm test:unit`, `pnpm tauri build`, portable-zip script, uploads artifact `OpenDiff-windows` (setup.exe, MSI, portable zip, SHA256SUMS.txt). No GitHub Release is published by this workflow. It does not run cargo fmt/clippy (the repo's separate `CI` workflow runs "Quality checks").
- `src/app/packagingConfig.test.ts`: updated for the new targets/NSIS fields/hook keys.
- `src/styles/folderChromeDensity.test.ts` line 91: `:size="16"` → `:size="14"`. This test was already failing on upstream master (the view uses 14px icons); it blocked the new workflow's unit-test step. Treated as a stale assertion — worth re-checking whether the test or the view was the intended source of truth.

### Git / GitHub state

- Local repo was created by `git init` + fetch from the fork (not `git clone`). Repo-local git identity set: `<owner email redacted>` / `Demonad112`. `.claude/` is excluded via `.git/info/exclude`.
- Branch `windows-installer` commits: `6a482fd` (installer + workflow + tests), `8ddbcf9` (stale test fix). Both committed with `--no-verify` because the husky pre-commit hook runs cargo fmt/clippy and Rust is not installed; the user approved this explicitly. Prettier + eslint + the packaging test were run by hand.
- PR #1 (fork → fork, `windows-installer` → `master`): https://github.com/Demonad112/open-diff/pull/1 — `gh` reports it **MERGED** at 2026-09-29T17:34Z by `Demonad112` (the user's account); merge commit `03a5f01`. Checks on it: "Quality checks" and "Build Windows installer" both SUCCESS. The local checkout may still be on the `windows-installer` branch.
- Note for `gh pr create` on this fork: without `-R Demonad112/open-diff` it can default to the upstream `kygo8/open-diff` repo.
- `package.json` / `tauri.conf.json` still point homepage/repository at `kygo8/open-diff`, and the bundle identifier is `io.github.kygo8.open-diff` (left unchanged on purpose — changing it moves the app-data location).

### Build outputs

- Successful run: Actions run `36603190669` (workflow "Windows Installer"). Downloaded into `%USERPROFILE%\Projects\Work\ODiff\installer\` (untracked): `OpenDiff_1.2.0_x64-setup.exe` (~7.1 MB), `OpenDiff_1.2.0_x64_en-US.msi` (~10 MB), `OpenDiff_1.2.0_windows_x64_portable.zip` (~10 MB), `SHA256SUMS.txt`. Setup.exe SHA256 `b4024e61…0e648a9` matched the checksum file.
- First run (`36602336907`) failed at the unit-test step (the stale assertion above).

### Install / test results on the user's PC

- The user first installed the **MSI** by mistake (uninstall key was a `{GUID}`; no `uninstall.exe`; no right-click keys). They uninstalled it and installed `setup.exe`.
- After the `setup.exe` install: `C:\Program Files\OpenDiff\` contains `open-diff-app.exe`, `open-diff-cli.exe`, `uninstall.exe`; uninstall registry key `HKLM\...\Uninstall\OpenDiff` exists; desktop shortcut present; the four HKLM shell-menu keys exist (`OpenDiff`, `OpenDiffSelectLeft` under both `*\shell` and `Directory\shell`). (An early "missing keys" reading was a bad `reg query /ve` invocation, not a real absence.)
- **CLI blocked:** running `open-diff-cli.exe --help` from PowerShell returned "An Application Control policy has blocked this file. Malicious binary reputation". `HKLM\SYSTEM\CurrentControlSet\Control\CI\Policy\VerifiedAndReputablePolicyState` read `1`, which the assistant **inferred** means Smart App Control is enforcing — not confirmed. The GUI `open-diff-app.exe` did launch. Nothing was changed in system security settings. The binaries are unsigned.
- **GUI results (computer-use screenshots):**
  - Home screen renders correctly.
  - Folder Compare: worked on the user's two folders (`%USERPROFILE%\Projects\Work\ODiff` vs `%USERPROFILE%\Projects\Work\AltWin`) — showed Left only / Right only entries, expandable groups, toolbar and report export buttons. The user drove the path entry and ran that compare.
  - Text Compare: with left `ODiff\README.md` and right `AltWin\README.md`, the app returned "One or more paths could not be found" — that was the assistant's error (the file is `AltWin\altWinDirStat\README.md`). After correcting the right path and clicking Run Diff, the view stayed on "Comparing…" for 15+ seconds with an empty diff pane while the status bar said "Same" and "Load time: 0.11 seconds" (likely stale values). Cause **not determined** — could be an app bug, a UI-automation timing issue, or interference from the user typing in the window at the same time. It was about to be re-tried with small sample files.
  - Not tested: Text Merge/Edit, Hex, Media, Picture, Registry, Table, Version, Folder Merge/Sync, sessions, HTML report export, the Explorer right-click "Select Left → Compare" flow, uninstall, CLI (blocked), the in-app Settings shell-integration toggle.
- Sample test files created in the session scratchpad: `%USERPROFILE%\AppData\Local\Temp\claude\C--Users-Addy7-Projects-Work-ODiff\5e634b85-10bd-4c4b-bbc8-5d5518c01b80\scratchpad\{a,b}\` (`t.txt`, `t.csv`, `left.txt`, `right.txt`).

### Environment gotchas seen

- Git Bash could not execute `open-diff-cli.exe` ("Permission denied"); PowerShell gave the Application Control message.
- `corepack enable` prints an EPERM on `C:\Program Files\nodejs\pnpx` (not admin) but `corepack pnpm …` works.
- The computer-use tool cannot control elevated windows (UAC/installers) and restricts typing while a native file dialog is open; several typed paths landed in the wrong field earlier while the user was also interacting with the window.
- `.gitattributes` forces LF; a file written with CRLF fails `prettier --check` and the pre-commit hook.
- Windows shows SmartScreen warnings for the unsigned installer.

---

## Requested new features (not started)

The user wants these staged into a plan in the next chat, and welcomes suggestions and questions:

1. **Live capture / transfer monitor.** When files are transferred from one location to another, see what was moved. Monitor changes to the destination ("new") folder in real time as files are imported.
2. **Flag anything not copied**, in its own area of the UI, **with the reason** why (where determinable).
3. **Recovery / easy location of failures.** If a file does not reach the chosen destination, optionally place a copy in a separate area — or otherwise offer an easy way to find all files that did not move, so the user does not have to dig.
4. The next chat is invited to propose additional features or better designs for the above during planning.

### Assistant's design notes (unverified brainstorming, not decisions)

- Observing changes: a Rust file-watcher (e.g. the `notify` crate, which wraps ReadDirectoryChangesW on Windows) pushing Tauri events to the Vue UI; possibly the NTFS USN journal for completeness on large trees. Watchers can drop events under load, so a periodic reconciliation pass (source manifest vs destination) would likely be needed anyway.
- "What was moved" vs "what should have moved": this needs a **source manifest** captured at the start (path, size, mtime, optional hash) so missing/mismatched files can be computed rather than only observed. The existing `snapshot-core`, `folder-core`, `sync-core`, `job-core`, `report-core`, `logging-core` crates are natural places to look first — not yet inspected in this session for fit.
- Failure reasons worth distinguishing: access denied, file in use/locked, path too long (>260), illegal characters/reserved names, destination out of space, name/case collisions, excluded by filter, skipped as identical, symlink/junction not followed, cloud placeholder (OneDrive) not hydrated, source disappeared mid-copy, size/hash mismatch after copy, timestamp-only difference.
- Findable results: a "Not copied" panel grouped by reason, exportable (CSV/HTML via existing report exporters), with "retry", "copy to quarantine folder", and "open in Explorer" actions; a stable quarantine folder per transfer run (`_not_copied/<run-id>/…`) preserving relative paths, plus a manifest file.
- Open questions to ask the user: Is the transfer done _by_ OpenDiff (its own copy/sync) or by Explorer/robocopy/another tool while OpenDiff only watches? Local disks only, or network shares/USB/cloud too? How large (file counts/sizes)? Should a hash verify be on by default? Where should the "quarantine copy" live, and is a second copy of failed files acceptable when the failure is e.g. disk-full? Is this for client deliverables (audit report)?

---

## Additional ideas the assistant had but did not raise during the session

- Ship `open-diff-cli.exe` on PATH (installer currently places it in Program Files but does not add it to PATH); and consider whether unsigned CLI blocking by Smart App Control matters for scripted client use.
- **Code signing** (OV/EV cert, or Azure Trusted Signing) to avoid SmartScreen warnings and Smart App Control blocks on client PCs — likely the single biggest client-deployment issue.
- Rebrand/identity for the fork: product name, publisher, homepage/repo URLs, bundle identifier, WiX `upgradeCode` (kept upstream's), icons.
- A release workflow for the fork that publishes installers to a GitHub Release on tag (existing upstream `release.yml` builds all 3 OSes and also uses upstream-specific storage-pruning logic).
- Add `cargo fmt` / `clippy` / `cargo test` to the Windows workflow or rely on the existing `CI` workflow; consider `cargo audit`/dependabot for supply chain.
- `security.csp` is `null` in `tauri.conf.json`; tightening it could break local file/asset loading, so it would need testing.
- `update-core` looks like it has only placeholder update URLs (`example.com`) — no real auto-update feed; Tauri's updater plugin or a winget manifest could cover client updates.
- Dropbox/OneDrive OAuth is documented as not production-hardened in the README.
- Repo docs are largely Chinese; if the user wants English docs/onboarding, that is a documentation task.
- Uninstall leaves user settings/app-data (untested); silent install flags (`/S`) for mass client rollout, and an MSI-vs-NSIS policy so users don't mix the two (as happened here).
- Installer shows no license page or custom branding beyond the icon; a first-run "enable Explorer menu" prompt vs installer-forced registration is a product choice.
- Smoke-test automation: a Playwright/tauri-driver end-to-end test against the installed build so this does not need manual GUI clicks (the repo already has `playwright.config.ts` and `tests/`).

---

## Deep Reference

### Key files (in the checkout)

- `src-tauri/tauri.conf.json` — bundle targets, NSIS config, WebView2 `downloadBootstrapper` (silent), WiX `upgradeCode` `90ffd755-2be3-5b35-8809-0f6022d8f999`.
- `src-tauri/windows/installer-hooks.nsh` — NSIS hooks (registry + desktop shortcut).
- `.github/workflows/windows-installer.yml` — the Windows build workflow.
- `.github/workflows/release.yml` — upstream's tag-triggered 3-OS release (not modified).
- `scripts/windows/package-portable.ps1`, `register-shell-extension.ps1`, `unregister-shell-extension.ps1` — upstream scripts; shell key names are the reference for the hook.
- `src-tauri/crates/shell-core/src/lib.rs` — shell-compare flow and Windows registration script generation.
- `src/app/packagingConfig.test.ts`, `src/styles/folderChromeDensity.test.ts` — tests touched.
- `installer/` — downloaded build outputs (untracked, ~27 MB).
- `handoff/` — these handoff files (untracked).

### NSIS hook body (as committed)

```nsis
!macro NSIS_HOOK_POSTINSTALL
  SetRegView 64
  WriteRegStr HKLM "Software\Classes\*\shell\OpenDiff" "MUIVerb" "Compare with Open Diff"
  ... (Icon + command for *, Directory, and the SelectLeft variants)
  SetShellVarContext all
  CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
!macroend
!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKLM "Software\Classes\*\shell\OpenDiff"  ; + Directory + SelectLeft variants
  Delete "$DESKTOP\${PRODUCTNAME}.lnk"
!macroend
```

(Full file: `src-tauri/windows/installer-hooks.nsh`.)

### Useful commands

- Watch/download a build: `gh run list -R Demonad112/open-diff -w "Windows Installer"`; `gh run download <run-id> -R Demonad112/open-diff -n OpenDiff-windows -D installer`.
- Re-run frontend checks locally (no Rust needed): `corepack pnpm test:unit`, `corepack pnpm build`, `corepack pnpm format:check`, `corepack pnpm lint`.
- Verify install registry: `reg query "HKLM\Software\Classes\Directory\shell\OpenDiff\command"` (use `MSYS_NO_PATHCONV=1` under Git Bash and avoid `/ve` quoting mistakes).
- Full-app local dev needs Rust + Tauri prerequisites (`corepack pnpm tauri:dev`) — not available on this PC as of this session.

### Plan file from this session

`%USERPROFILE%\.claude\plans\go-through-this-repo-synchronous-blanket.md` (the approved installer plan; outside the project folder).
