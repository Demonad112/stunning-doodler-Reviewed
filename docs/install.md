# Installing DeepServer

DeepServer installs per machine (into `C:\Program Files\DeepServer`) and adds its Explorer menus for all users. Installing and uninstalling need administrator rights.

## Supported systems

| System                                                     | Supported                                  |
| ---------------------------------------------------------- | ------------------------------------------ |
| Windows 10 / 11 (x64)                                      | Yes                                        |
| Windows Server 2016, 2019, 2022, 2025 (Desktop Experience) | Yes                                        |
| Windows Server Core / Nano Server                          | No (there's no desktop to show the app in) |
| ARM64 or 32-bit Windows                                    | No (x64 build only)                        |

## Which file to download

Every [release](https://github.com/Demonad112/stunning-doodler-Reviewed/releases) has these assets. The repository is private, so downloads need a GitHub account with access: use a browser signed in to GitHub, or the GitHub CLI (below). Plain `releases/download/...` links return 404 when you aren't signed in.

| File                                            | Use it when                                                                                                                                                                                 |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `DeepServer_<version>_x64-setup.exe`            | The machine has internet access. If the Microsoft Edge WebView2 runtime is missing, the installer downloads it (Windows 11 and up-to-date Windows 10 already have it).                      |
| `DeepServer_<version>_x64-offline-setup.exe`    | Servers with no internet access, or where downloads are blocked. It carries the WebView2 runtime itself, so it's about 215 MB larger.                                                       |
| `DeepServer_<version>_windows_x64_portable.zip` | You can't install anything. Unzip it and run `DeepServer.exe`. It also has the command-line tool `deepserver-cli.exe`. There are no Explorer menus, and WebView2 must already be installed. |
| `SHA256SUMS.txt`                                | Checking the downloads.                                                                                                                                                                     |

Both installers install the same app. Running either one over an existing install upgrades it in place.

### Download with the GitHub CLI

```powershell
winget install --id GitHub.cli -e   # once
gh auth login                       # once
$tag = 'v1.0.0'                     # the release you want, e.g. v1.0.0-rc4
gh release download $tag -R Demonad112/stunning-doodler-Reviewed -p '*_x64-offline-setup.exe' -p 'SHA256SUMS.txt'
```

## Check the download

```powershell
Get-FileHash .\DeepServer_1.0.0_x64-offline-setup.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

The two hashes must match.

## Interactive install

Run the setup file and follow the prompts. The builds aren't code-signed yet, so SmartScreen may show "Windows protected your PC": choose **More info** → **Run anyway**. On a server with Smart App Control or WDAC policies, allow the file by its hash first.

## Silent install (servers and scripted rollouts)

From an elevated PowerShell or Command Prompt:

```powershell
# Default location (C:\Program Files\DeepServer)
Start-Process .\DeepServer_1.0.0_x64-offline-setup.exe -ArgumentList '/S' -Wait

# Custom location: /D= must be the last argument and must not be quoted, even with spaces
Start-Process .\DeepServer_1.0.0_x64-offline-setup.exe -ArgumentList '/S /D=D:\Tools\DeepServer' -Wait
```

- `/S` installs with no UI. The WebView2 runtime is also installed silently if it's missing.
- An install over an existing one upgrades it; settings and Disk Usage history are kept. Upgrading from 1.0.0-rc1 or rc2 also removes the old `open-diff-cli.exe`.
- Exit code `0` means success.

Check the result:

```powershell
Test-Path "$env:ProgramFiles\DeepServer\DeepServer.exe"
Test-Path "$env:ProgramFiles\DeepServer\deepserver-diskusage.exe"
Test-Path "$env:ProgramFiles\DeepServer\deepserver-cli.exe"
```

### Deploying through Intune, SCCM/MECM or GPO

- **Install command:** `DeepServer_<version>_x64-offline-setup.exe /S`
- **Uninstall command:** `"%ProgramFiles%\DeepServer\uninstall.exe" /S`
- **Detection rule:** file `%ProgramFiles%\DeepServer\DeepServer.exe` exists (optionally, version ≥ `<version>`)
- **Install context:** System
- Use the offline installer so endpoints don't need to reach Microsoft's WebView2 download servers.

## Silent uninstall

```powershell
Start-Process "$env:ProgramFiles\DeepServer\uninstall.exe" -ArgumentList '/S' -Wait
```

This removes the program files and the Explorer menus (for all users and the current user). The uninstaller re-launches itself from `%TEMP%`, so the files can take a few seconds to disappear after the command returns. Per-user data stays unless you delete it:

| Data                              | Location                                                                                           |
| --------------------------------- | -------------------------------------------------------------------------------------------------- |
| Disk Usage snapshot history       | `%LOCALAPPDATA%\DeepServer\History`                                                                |
| Transfer Monitor runs and reports | `%LOCALAPPDATA%\DeepServer\Transfers`                                                              |
| App settings (WebView2 data)      | `%LOCALAPPDATA%\com.demonad112.deepserver`                                                         |
| Remote profiles, admin policy     | `%USERPROFILE%\.config\open-diff` (the OpenDiff name is kept; `OPEN_DIFF_CONFIG_DIR` overrides it) |

Transfer Monitor runs older than 90 days, and all but the newest 200, are removed automatically when DeepServer starts (**Previous runs → Clean up now** does it on demand). Recovery folders (`<destination>_NotCopied`) are never removed automatically.

## What gets installed

- `DeepServer.exe`: the app (compare and merge, sync, Transfer Monitor, Disk Usage views)
- `deepserver-diskusage.exe`: the Disk Usage engine (GPL v2; source in `native/diskusage`)
- `deepserver-cli.exe`: the command-line tool (`deepserver-cli --help`). It isn't added to `PATH`, so call it by its full path.
- Explorer menus: **Compare with DeepServer** and **Select Left … for Compare** on files and folders, **Analyze disk usage** on folders and drives, and **Copy with verification (DeepServer)** on folders and drives
