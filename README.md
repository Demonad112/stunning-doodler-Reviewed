# DeepServer

DeepServer is one Windows app for looking after files on workstations and servers:

- **Compare and merge**: text, folders, tables, hex, images, media, registry and version info.
- **Copy and sync**: folder sync, plus a verified copy (Transfer Monitor) that groups any files that failed and says why.
- **Disk usage**: a treemap of what fills a drive, and a "what changed since last time" view.

It installs from a single `setup.exe`. Windows 10/11 and Windows Server 2016+ (Desktop Experience) are supported.

## Install

Download from the [Releases](https://github.com/Demonad112/stunning-doodler-Reviewed/releases) page:

- `DeepServer_<version>_x64-setup.exe`: for machines with internet access
- `DeepServer_<version>_x64-offline-setup.exe`: for offline or locked-down servers (includes the WebView2 runtime)
- `DeepServer_<version>_windows_x64_portable.zip`: no install, no Explorer menus

For servers and scripted rollouts (elevated):

```powershell
.\DeepServer_1.0.0_x64-offline-setup.exe /S
& "$env:ProgramFiles\DeepServer\uninstall.exe" /S
```

The builds aren't code-signed yet, so Windows SmartScreen may warn on first run. [docs/install.md](docs/install.md) covers checksums, custom install folders, Intune/SCCM detection rules and what gets installed. [docs/releasing.md](docs/releasing.md) covers how releases are made.

## Repository layout

| Path                 | What it is                                                                                            |
| -------------------- | ----------------------------------------------------------------------------------------------------- |
| `src/`, `src-tauri/` | The DeepServer app (Tauri 2 + Vue 3 + Rust), based on [Open Diff](https://github.com/kygo8/open-diff) |
| `native/diskusage/`  | The disk-usage engine (C++), based on [WinDirStat](https://github.com/windirstat/windirstat) 2.x      |
| `docs/history/`      | Design notes and plans carried over from the two original projects                                    |

## Building locally

Requirements: Node 24 with corepack, Rust (MSVC), and Visual Studio 2022 with the C++ desktop workload.

```powershell
corepack pnpm install
corepack pnpm tauri:dev     # run the app
corepack pnpm test:unit     # frontend tests
cargo test --workspace --manifest-path src-tauri/Cargo.toml
```

Release builds come from GitHub Actions: push a `v*` tag (see [docs/releasing.md](docs/releasing.md)).

## Licenses

DeepServer ships two separate programs in one installer:

- the app, under the Apache License 2.0 (`LICENSE`)
- the disk-usage engine, under the GNU GPL v2 (`native/diskusage/LICENSE.md`); its full source is in this repository

See `NOTICE` for credits.
