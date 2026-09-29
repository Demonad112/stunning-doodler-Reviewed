> This file was compiled by an assistant from a prior conversation on this
> project. Treat it as background material, not verified fact — reason
> independently before relying on anything here, especially items presented
> as correct, decided, or working. It's meant to bring a new collaborator up
> to speed, not to define how they should proceed.

# Reference: OpenDiff → merged repo (stunning-doodler-Reviewed)

## Quick Recap

The user forked **Open Diff**, an open-source Beyond Compare alternative (Tauri 2 + Vue 3 + Rust workspace, v1.2.0), at https://github.com/Demonad112/open-diff, and got a Windows NSIS installer building on GitHub Actions. In the most recent session:

- Rust was installed on the dev PC.
- A `tauri dev` crash was fixed.
- A Text Compare usability fix was made, and PR #4 was opened.
- Several app modes were smoke-tested.
- A large new **Transfer Monitor** feature was planned but not started.

The user now intends to combine three things into a new, currently empty repo, https://github.com/Demonad112/stunning-doodler-Reviewed:

1. this OpenDiff fork
2. https://github.com/Demonad112/altWinDirStat
3. a separate "Claude-built app"

GitHub is meant to hold the files and do most of the building; the local PC runs the final working app.

This session never looked at altWinDirStat or the "Claude-built app", and never discussed how the merge would work.

## Working Context

### OpenDiff at a glance

- **Frontend:** Vue 3 + Vite + naive-ui + Pinia, installed with pnpm through corepack. Node 24 is installed locally.
- **Backend:** Tauri 2.11. `src-tauri/` is a Cargo workspace with about 30 `*-core` crates (diff, folder, sync, snapshot, job, report, logging, vfs, remote, shell, policy, and so on).
- **Binaries:** `open-diff-app` (the GUI) and `open-diff-cli`.
- **Commands:** about 62 Tauri commands, all in one ~8k-line `src-tauri/src/commands.rs` and registered in `src-tauri/src/lib.rs`. They are all synchronous request/response calls; nothing emits events from Rust to the UI yet.
- **Docs:** many are in Chinese.
- **Identity:** bundle identifier `io.github.kygo8.open-diff`. The homepage/repo URLs and the WiX `upgradeCode` still point at upstream `kygo8/open-diff`.

### Git and GitHub state (as of the end of this session; worth re-checking)

**Local checkout:** `%USERPROFILE%\Projects\Work\ODiff`, on branch `stage0-smoke-fixes`. Untracked items in it:

- `handoff/`
- `installer/` (downloaded build outputs, ~27 MB)
- `installer2.zip`, which the assistant didn't create and never inspected

**Pull requests on the fork:**

| PR  | Branches                        | State                                     |
| --- | ------------------------------- | ----------------------------------------- |
| #1  | `windows-installer` → `master`  | Merged; the installer work is on `master` |
| #2  | `master` → `windows-installer`  | Merged by the user                        |
| #3  | `windows-installer` → `master`  | Open, probably redundant                  |
| #4  | `stage0-smoke-fixes` → `master` | Open; the fixes below                     |

The desktop app is watching PR #4 with Auto-fix on. Its CI was pending at the time of the handoff.

**Commit `24406b0`** on `stage0-smoke-fixes`:

- `vite.config.ts` now ignores `**/src-tauri/**` in the watcher. Before, cargo locking `target\debug\...\open_diff_app.exe` caused an `EBUSY` crash that killed `tauri dev`.
- In Text Compare, pressing Enter in either path field loads both files once both are set (`loadTypedTextPaths` in `src/views/TextCompareView.vue`), with two tests in `src/views/sessionLinkage.test.ts`.

**Branch cleanup (not done):** the fork carries **370 branches**. 368 of them are identical copies of upstream `kygo8/open-diff` branches; `master` and `windows-installer` are the user's own. The user asked to delete everything except their own and the assistant's branches. Every deletion was rejected with **GH013 (repository rule violations)**, because a ruleset on the fork blocks branch deletion. Nothing was deleted. The ruleset is at https://github.com/Demonad112/open-diff/rules, and the user chose to handle that setting themselves.

### Build setup

- **Windows installer workflow:** `.github/workflows/windows-installer.yml` runs on windows-latest via manual dispatch or a push to `windows-installer`. Steps: pnpm install → `pnpm test:unit` → `pnpm tauri build` → portable zip. It uploads the artifact `OpenDiff-windows` (NSIS `setup.exe`, MSI, portable zip, SHA256SUMS). It doesn't publish a Release.
- **Separate CI workflow:** runs the "Quality checks".
- **NSIS config:** `perMachine`, with hooks in `src-tauri/windows/installer-hooks.nsh` that write HKLM Explorer right-click keys and a desktop shortcut.
- **Local builds work now:** rustup 1.29.1 / rustc 1.98.1 (MSVC, installed via winget), with VS 2022 already present. `cargo test --workspace` passes, and `corepack pnpm tauri:dev` runs.
- **Pre-commit hook** (husky: lint-staged plus `pnpm quality`) runs prettier on the whole repo, then eslint, stylelint, vue-tsc, cargo fmt and clippy `-D warnings`. The first cold clippy run took more than 15 minutes on this PC; warm runs take about 15 seconds.
- **Prettier gotcha:** `prettier . --check` also scans untracked files such as `handoff/*.md`, so unformatted notes there make the hook fail.

### Smoke-test results (in the dev build unless noted)

| Mode           | Result                                                                                                                                                                                                                |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Text Compare   | Works. The earlier "Comparing…" hang could not be reproduced. The likely cause was confusion: typed paths only loaded via the ↻ button, while Run Diff diffs the editor contents. That's an inference, not confirmed. |
| Hex Compare    | Works. The layout is stretched and stacked, and looks the same in the installed build, so it comes from upstream.                                                                                                     |
| Table Compare  | Works.                                                                                                                                                                                                                |
| Folder Sync    | The preview planned correctly. Sync Now was never executed.                                                                                                                                                           |
| Folder Compare | Worked in an earlier session.                                                                                                                                                                                         |

Never tested: Text Merge/Edit, Media, Picture, Registry, Version, Folder Merge, sessions, HTML report, the Explorer right-click flow, uninstall, and the CLI.

The CLI was blocked on this PC by an application-control policy ("Malicious binary reputation"). That's assumed, not confirmed, to be Smart App Control reacting to an unsigned binary. The locally built dev binaries did run.

### The planned feature: Transfer Monitor (approved plan, nothing built)

Plan file: `%USERPROFILE%\.claude\plans\c-users-addy7-projects-work-odiff-hando-bubbly-ritchie.md`, outside the repo.

**What the user chose in Q&A:**

- **Two modes:**
  - _Copy_: OpenDiff does the copying.
  - _Watch_: another tool copies; the user picks the source too, and OpenDiff reconciles the destination against a manifest of the source.
- **Targets:** local disks, USB, and network shares. Cloud placeholder folders are out of scope.
- **Scale:** unknown, so the design aims at 100k+ files.
- **Failures:** a "Not copied" panel grouped by reason, with Retry / Copy to recovery folder / Reveal / Export. Nothing is copied automatically.
- **Verification:** size + modified time by default, with optional hashing.
- **Copy only:** no move.
- **Reporting:** an exportable client audit report (HTML/CSV).
- **Order:** finish smoke testing first.

**Stages in the plan:**

- **0.** Dev environment and smoke test (mostly done).
- **1.** A new `transfer-core` crate:
  - Windows error-code → reason classification: 5 AccessDenied, 32/33 Locked, 39/112 DiskFull, 206 PathTooLong, 123 InvalidName, 53/64/67/1231 NetworkLost, 21/1117 DeviceRemoved, 23 MediaError.
  - A pre-flight check.
  - A streaming copy using `\\?\` long paths, writing a `.odpart` temp file then renaming, preserving mtime, retrying transient errors, with an optional BLAKE3 hash.
  - A background job runner that emits `transfer://progress` and `transfer://items` events.
  - JSONL run storage in app data.
- **2.** A Copy-mode UI as a new session type, backed by a Pinia store.
- **3.** Watch mode using the `notify` crate, with debounce, periodic rescans, a polling fallback for SMB shares, and "inferred" reasons.
- **4.** A recovery folder plus a `report-core` `ReportKind::Transfer`.
- **5.** Later extras: resume a run, CLI, auto-pause, bandwidth limit, presets.
- **6.** Rollout: code signing (Azure Trusted Signing or an OV certificate), NSIS-only installs for clients plus `/S` silent install, fork identity, a release workflow.

**Existing code survey:**

- **Reusable:**
  - `snapshot-core` (`SnapshotEntry` has an unused `content_hash` field)
  - `folder-core` (scanner, filters)
  - `sync-core` (plan/result types)
  - `job-core` (`JobStatus`, `CancellationToken`, but no registry)
  - `report-core` (HTML/CSV renderers)
  - `logging-core` (in-memory only)
  - frontend: `revealPathInOs`, the unused `stores/jobs.ts` / `JobsProgressPanel.vue`, and the manual virtual list in `FolderCompareView.vue`
- **Missing:**
  - no Rust→UI events at all
  - no file watcher
  - no OS error classification (`VfsError` only has NotFound/Readonly/Io)
  - no long-path handling, retry or verification
  - the app's sync copies with plain `fs::copy`
  - the scanner aborts on the first unreadable entry
- **Adding a new mode touches about 12 files by hand:** session types, catalog, factory, router, HomeView tile lists, command registry, AppLayout, shellChrome, status bar, toolbars, and i18n in 8 locales (checked by completeness tests).

## Deep Reference

### Useful commands (PowerShell unless noted)

```text
# Rust on PATH for a shell (rustup installed per-user)
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"

# Dev app / tests
corepack pnpm tauri:dev
cd src-tauri; cargo test --workspace
corepack pnpm vitest run <file>
corepack pnpm quality          # same gate as pre-commit

# Build artifacts from GitHub
gh run list -R Demonad112/open-diff -w "Windows Installer"
gh run download <run-id> -R Demonad112/open-diff -n OpenDiff-windows -D installer

# gh on this fork: always pass -R, otherwise it may target upstream kygo8/open-diff
gh pr create -R Demonad112/open-diff --base master --head <branch>
```

### Lessons learned about operating on this machine

- **Blocking waits froze a session.** A long `until grep …` wait on `tauri dev` output hung it. Short, time-limited polls against log files worked better. Also, `tauri dev` output contains ANSI colour codes, so a plain grep for `Running` can miss the match.
- **Stray cargo processes can hold the lock.** Background commits whose tool call was killed left orphan `cargo clippy` processes holding the build-directory lock, and the next run sat at "Blocking waiting for file lock". Look for them with `Get-CimInstance Win32_Process -Filter "Name='cargo.exe'"`.
- **`git fetch` from the fork is slow** because of the 370 branches; one fetch timed out at 30 seconds. Narrowing `remote.origin.fetch`, or deleting the branches, would help.
- **Computer-use access to the dev window** has to be granted by exe name (`open-diff-app.exe`). The installed app is a separate grant (`OpenDiff`). A debug build also opens a console window.
- **Typing into the app:** triple-click doesn't select long paths in its inputs; Ctrl+A works. Native `<select>` pop-ups are masked in screenshots, so keyboard arrows are the way to change them.
- **Line endings:** `.gitattributes` forces LF, and files written with CRLF fail prettier.

### Merge-related facts gathered so far (very little)

- A local copy of altWinDirStat exists at `%USERPROFILE%\Projects\Work\AltWin\altWinDirStat`. Its README is 107 lines with CRLF endings. Nothing else about it (language, framework, build) was examined.
- The "Claude-built app" was never identified in this session.
- The target repo https://github.com/Demonad112/stunning-doodler-Reviewed is described by the user as empty.
- OpenDiff's history carries hundreds of upstream commits and branches. Importing it with or without history (for example a `git subtree`, or a fresh copy) makes a big difference to repo size, and to how easy it is to pull future upstream fixes.
