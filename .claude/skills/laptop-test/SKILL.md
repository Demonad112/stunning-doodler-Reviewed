---
name: laptop-test
description: Run the DeepServer laptop test of a release candidate (e.g. /laptop-test v1.0.0-rc4): download, verify, install, check, then walk the test inventory and report in three groups. User-invoked only.
argument-hint: <tag>
disable-model-invocation: true
---

Test release `$ARGUMENTS` (default: newest pre-release from `gh release list --limit 3`). Low effort; give the owner PowerShell blocks and let them run them (elevated). Don't load the root-level `*-reference.md` files.

Cheapest first: spawn the `laptop-check` agent (Haiku) with the tag for the scriptable checks and a ready report skeleton, then do only its MANUAL list.

## 1. Download, verify, install (PowerShell, elevated)

```powershell
$repo='Demonad112/stunning-doodler-Reviewed'; $tag='<tag>'
$dir="$env:USERPROFILE\Downloads\DeepServer-$tag"; New-Item -ItemType Directory -Force $dir | Out-Null
gh release download $tag -R $repo -D $dir --clobber -p '*_x64-setup.exe' -p '*_x64-offline-setup.exe' -p '*portable.zip' -p 'SHA256SUMS.txt'
Get-Content "$dir\SHA256SUMS.txt" | % { $h,$n = $_ -split '\s+\*?',2; '{0}  {1}' -f $(if((Get-FileHash "$dir\$n").Hash -eq $h.ToUpper()){'OK  '}else{'FAIL'}),$n }
Start-Process (Get-Item "$dir\*_x64-setup.exe").FullName -ArgumentList '/S' -Wait -Verb RunAs
```

Post-install: `DeepServer.exe`, `deepserver-diskusage.exe`, `deepserver-cli.exe` exist in `C:\Program Files\DeepServer`; Explorer keys exist (`scripts/windows/Test-Installer.ps1` lists them); Uninstall entry shows the version.
Optional upgrade check: install rc2 first, confirm `open-diff-cli.exe`, then install the new rc and confirm it is gone and settings/history survive.

## 2. Inventory (full detail: `deepserver-local-test-reference.md` "Full test inventory", engine items in `native/diskusage/HANDOFF.md`)

- Install/upgrade/uninstall (folder and `HKCR` keys gone; user data kept on purpose)
- Disk Usage: scan `C:\` and a folder; Stop within 2 s writes no snapshot; snapshot, change files, snapshot, What changed; engine checklist 1-17
- Transfer Monitor: size+time and Hash verify; locked file, Not copied, Retry, recovery copy; Watch during an Explorer copy; USB pulled, SMB share, 50k files; HTML/CSV/JSON reports; Clean up now
- Folder Compare: filters, selection, sorting; `C:\` shows "Unreadable"; report times local; Sync stops (not skips) on unreadable
- Other modes: Text, Table, Image, Hex, Version, Merge, archives, snapshots
- Explorer menus on file, folder, drive, two folders (Win 11 "Show more options")
- CLI: `deepserver-cli --help`, `compare-folders` exit codes (0 same, 1 different), unreadable folder on stderr
- UI: 8 locales, dark/light, 125%/150% DPI, resize, jobs panel, Help links (no OpenDiff URLs), About says 1.0.0
- Resources: idle CPU/RAM, no WebView2 processes after closing

Data: `%LOCALAPPDATA%\DeepServer\{History,Transfers}`, `%LOCALAPPDATA%\com.demonad112.deepserver`, `%USERPROFILE%\.config\open-diff`.

## 3. Report

Three groups, one line per item with a repro: **Needs fixing**, **Look into further**, **Improvements**. Then say which fixes become PRs (one each, into `main`) before the next rc.
