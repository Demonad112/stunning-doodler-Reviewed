# Changelog

All notable changes to DeepServer will be documented in this file.

The format is based on Keep a Changelog, and this project uses Semantic
Versioning for release tags.

## [Unreleased]

## [1.0.0] - Unreleased

First DeepServer release: OpenDiff 1.2.0 and a WinDirStat fork (the Disk
Usage engine) in one Windows app and installer.

### Added

- Disk Usage mode: pick a drive, share or folder, open the treemap, take
  snapshots, and see what changed between two snapshots. "Compare these two
  folders" opens Folder Compare on any row.
- Disk Usage engine (`deepserver-diskusage.exe`), built from the WinDirStat
  fork, with change tracking, a headless `/saveto … /noelevate` scan and
  `/compare`. Snapshots live in `%LOCALAPPDATA%\DeepServer\History`.
- Transfer Monitor: verified copies (streamed to temporary files, then
  renamed), long-path support, retries, optional BLAKE3 checks, a Watch mode,
  a "Not copied" panel grouped by reason, and HTML/CSV transfer reports.
- Home grouped into Compare, Copy & Sync, and Disk; cross-launch between
  the engine and the compare modes.
- Explorer right-click entries for compare, select-left, disk usage and
  verified copy.
- Two installers (standard, and offline with WebView2 for servers), a
  portable zip and `SHA256SUMS.txt`. Silent install with `/S`.

### Changed

- Renamed from Open Diff: app `deepserver.exe`, CLI `deepserver-cli.exe`,
  new app identifier (Open Diff settings don't carry over).
- Windows only. Supported on Windows 10/11 and Server 2016+ with Desktop
  Experience.

### Fixed

- `deepserver-cli compare-folders` compares contents, so same-size files
  with different contents are reported as different.
- Folder Sync and Folder Compare planned right-to-left copies as
  left-to-right when roots used backslashes, so Sync Now could overwrite a
  newer file.
- Mirror sync copies a file whose size differs even when the times match.
- Dark mode covers the whole window; menu-bar dropdowns fit their labels at
  high display scaling; folder paths no longer show "0 bytes".
- Upgrading from rc1/rc2 removes the old `open-diff-cli.exe`.

### Known limitations

- Builds aren't code-signed yet, so SmartScreen may warn.
- Not supported on Server Core or Nano Server.

## Open Diff history

Releases below are from Open Diff, which DeepServer is built on.

## [1.1.1] - 2026-09-04

### Changed

- Home no longer invents recent history or sample sessions; empty states tell
  users how to start.
- Folder Compare Run Sync now executes a real sync instead of status-only
  success.
- Saved-session launches fill Folder Sync, Folder Merge, and Text Edit.
- Unfinished Home tree/edit actions and unimplemented remote protocols stay
  disabled with honest labels.

### Fixed

- Windows CI clippy `needless_return` in shell registration and live registry
  query.
- CI now installs Playwright Chromium so e2e can launch on `windows-latest`.
- Packaging already on master: Intel Mac builds natively on `macos-15-intel`
  (no ARM→x86_64 OpenSSL cross-compile); Windows uses the ssh2 portable API.

### Known limitations

- Still unimplemented: Open With / Align With, S3, Dropbox, OneDrive, FTPS,
  SVN remote, 7z archives, full BC script language, live Windows registry
  hives from `.reg` only off-host, Home tree +/− and Edit, custom keyboard
  shortcuts, extra report formats.

## [1.1.0] - 2026-08-18

### Added

- Text merge wired, ignore rules, real picture compare, table TSV/Excel,
  reports, and child folder open.
- Real ZIP/TAR compare (not a hex-tab fake), script runner, SFTP/FTP, patch
  apply, sync overrides, and git/svn `--write`.
- WebDAV, more script file ops, folder criteria, follow-system theme, and
  policy at startup.
- TypeScript 5.9.3 quality gate and a real `shell-compare` implementation.

### Changed

- Empty Open no longer seeds demo data.
- Open With and Align With stay unimplemented and report that honestly.
- `merge-text` requires `--automerge`.

### Tests

- UI↔command linkage coverage for shipped features.
- 100 consecutive greens for unit, cargo, and e2e.

### Known limitations

- Still unimplemented: S3, Dropbox, OneDrive, SVN, 7z, and live registry off
  Windows.

[Unreleased]: https://github.com/Demonad112/stunning-doodler-Reviewed/compare/v1.0.0-rc4...HEAD
[1.0.0]: https://github.com/Demonad112/stunning-doodler-Reviewed/releases
[1.1.1]: https://github.com/kygo8/open-diff/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/kygo8/open-diff/compare/v1.0.1...v1.1.0
