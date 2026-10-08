# Installing and releasing DeepServer 2.0

DeepServer 2.0 replaces DeepServer 1.x. Windows 10/11 and Windows Server 2016+ (Desktop Experience).

## Files

| File                                      | Use it for                                                         |
| ----------------------------------------- | ------------------------------------------------------------------ |
| `DeepServer_<v>_x64-setup.exe`            | PCs with internet access (downloads WebView2 if it's missing)      |
| `DeepServer_<v>_x64-offline-setup.exe`    | Offline or locked-down PCs and servers (WebView2 runtime included) |
| `DeepServer_<v>_windows_x64_portable.zip` | No install: unzip, run `DeepServer.exe`                            |
| `SHA256SUMS.txt`                          | Checksums                                                          |

Silent install for RMM tools: `DeepServer_<v>_x64-offline-setup.exe /S`. Installs per machine into
`C:\Program Files\DeepServer`. Silent uninstall: `"C:\Program Files\DeepServer\uninstall.exe" /S`.

## Upgrading from 1.x

Run the 2.0 installer. It uses the same name, publisher and folder, so Windows treats it as an upgrade.
Because Tauri doesn't run the old uninstaller for silent installs, the 2.0 installer removes the 1.x
leftovers itself: `deepserver-cli.exe`, `deepserver-diskusage.exe`, and the 1.x Explorer right-click
entries (they pass arguments 2.0 doesn't understand). 2.0 has no right-click menu yet (Batch 6).
1.x data (`%LOCALAPPDATA%\DeepServer`) is left alone; 2.0 keeps its own in `%LOCALAPPDATA%\DeepServer2`.

## Portable mode

A file named `portable.txt` next to `DeepServer.exe` turns it on (the portable zip ships with one).
Records and reports are then saved in `Data\` next to the exe, so the folder can move between PCs, and
Disk Cleanup never deletes anything in it. Delete `portable.txt` to go back to `%LOCALAPPDATA%\DeepServer2`.
Settings kept by the web view (theme, recent folders, company name) stay on the PC, not in `Data\`.
`DEEPSERVER2_RECORDS_DIR` and `DEEPSERVER2_REPORTS_DIR` still override both.

## Releasing

`.github/workflows/release-v2.yml` builds `v2.*` tags (`release.yml` handles `v1.*` only). Keep the same
version in `V2.0/package.json`, `V2.0/src-tauri/tauri.conf.json` and `V2.0/src-tauri/Cargo.toml`
(then `cargo check` to update `Cargo.lock`); a tag `v2.0.0-rc1` needs `2.0.0`. Merge the bump, then run
**Actions → Release V2 → Run workflow** with `tag` and `create_tag` ticked, or push the tag.
`-rc` tags publish a pre-release; a final `v2.0.0` creates a draft to review and publish.
Builds are unsigned until signing is added (SmartScreen will warn).
`v2.yml` builds and smoke-tests both installers on every V2 pull request.
