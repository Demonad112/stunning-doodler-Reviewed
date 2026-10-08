# Builds the DeepServer 2.0 portable zip: DeepServer.exe + portable.txt (turns on portable mode: records and
# reports are saved in a Data folder next to the exe) + README/LICENSE. Run after `corepack pnpm tauri build`.
# Needs the WebView2 runtime on the PC (Windows 11 and current Windows 10 have it).
param(
  [string] $TargetDir = "",
  [string] $Version = ""
)

$ErrorActionPreference = 'Stop'

$v2Root = Resolve-Path (Join-Path $PSScriptRoot '..')
$repoRoot = Resolve-Path (Join-Path $v2Root '..')

if ([string]::IsNullOrWhiteSpace($TargetDir)) {
  $TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $v2Root 'src-tauri\target' }
}
if ([string]::IsNullOrWhiteSpace($Version)) {
  $Version = [string] (Get-Content -LiteralPath (Join-Path $v2Root 'package.json') -Raw | ConvertFrom-Json).version
}

$appExe = Join-Path $TargetDir 'release\DeepServer.exe'
if (-not (Test-Path -LiteralPath $appExe)) { throw "Missing $appExe (run tauri build first)" }

$portableRoot = Join-Path $TargetDir 'release\bundle\portable'
$staging = Join-Path $portableRoot 'DeepServer'
$archive = Join-Path $portableRoot "DeepServer_${Version}_windows_x64_portable.zip"

if (Test-Path -LiteralPath $staging) { Remove-Item -LiteralPath $staging -Recurse -Force }
New-Item -ItemType Directory -Path $staging -Force | Out-Null

Copy-Item -LiteralPath $appExe -Destination (Join-Path $staging 'DeepServer.exe')
Set-Content -LiteralPath (Join-Path $staging 'portable.txt') -Encoding ASCII -Value @(
  'DeepServer portable mode is on while this file sits next to DeepServer.exe.',
  'Records and reports are saved in the Data folder here instead of on the PC.',
  'Delete this file to use the PC''s own folder (%LOCALAPPDATA%\DeepServer2) again.'
)
Copy-Item -LiteralPath (Join-Path $repoRoot 'LICENSE') -Destination (Join-Path $staging 'LICENSE')
Copy-Item -LiteralPath (Join-Path $repoRoot 'NOTICE') -Destination (Join-Path $staging 'NOTICE')

if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive -Force }
Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $archive -CompressionLevel Optimal
Write-Output $archive
