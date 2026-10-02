param(
  [string] $Configuration = "release",
  [string] $Version = "",
  [switch] $SkipBuild
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")

if ([string]::IsNullOrWhiteSpace($Version)) {
  $packageJson = Get-Content -LiteralPath (Join-Path $repoRoot "package.json") -Raw | ConvertFrom-Json
  $Version = [string] $packageJson.version
  if ([string]::IsNullOrWhiteSpace($Version)) {
    throw "Could not read version from package.json"
  }
}

$releaseRoot = Join-Path $repoRoot "src-tauri\target\$Configuration"
$portableRoot = Join-Path $repoRoot "src-tauri\target\$Configuration\bundle\portable"
$stagingRoot = Join-Path $portableRoot "DeepServer"
$archivePath = Join-Path $portableRoot "DeepServer_${Version}_windows_x64_portable.zip"

if (-not $SkipBuild) {
  corepack pnpm tauri:build
}

# `tauri build` renames the app binary to mainBinaryName (DeepServer.exe); a plain
# `cargo build` leaves the Cargo target name (open-diff-app.exe).
$appExe = Join-Path $releaseRoot "DeepServer.exe"
if (-not (Test-Path -LiteralPath $appExe)) {
  $appExe = Join-Path $releaseRoot "open-diff-app.exe"
}
$cliExe = Join-Path $releaseRoot "deepserver-cli.exe"
if (-not (Test-Path -LiteralPath $appExe)) {
  throw "Missing $appExe (build first or omit -SkipBuild)"
}
if (-not (Test-Path -LiteralPath $cliExe)) {
  throw "Missing $cliExe (build first or omit -SkipBuild)"
}

if (Test-Path -LiteralPath $stagingRoot) {
  Remove-Item -LiteralPath $stagingRoot -Recurse -Force
}

New-Item -ItemType Directory -Path $stagingRoot -Force | Out-Null

Copy-Item -LiteralPath $appExe -Destination (Join-Path $stagingRoot "DeepServer.exe") -Force
Copy-Item -LiteralPath $cliExe -Destination (Join-Path $stagingRoot "deepserver-cli.exe") -Force
# The disk-usage engine is a Tauri sidecar: `tauri build --config src-tauri/tauri.engine.conf.json`
# copies it next to the app. Builds without it still package; the Disk Usage features then say it's missing.
$engineExe = Join-Path $releaseRoot "deepserver-diskusage.exe"
if (Test-Path -LiteralPath $engineExe) {
  Copy-Item -LiteralPath $engineExe -Destination (Join-Path $stagingRoot "deepserver-diskusage.exe") -Force
} else {
  Write-Warning "Disk-usage engine not found at $engineExe; packaging without it."
}
Copy-Item -LiteralPath (Join-Path $repoRoot "README.md") -Destination (Join-Path $stagingRoot "README.md") -Force
Copy-Item -LiteralPath (Join-Path $repoRoot "LICENSE") -Destination (Join-Path $stagingRoot "LICENSE") -Force
Copy-Item -LiteralPath (Join-Path $repoRoot "NOTICE") -Destination (Join-Path $stagingRoot "NOTICE") -Force


if (Test-Path -LiteralPath $archivePath) {
  Remove-Item -LiteralPath $archivePath -Force
}

Compress-Archive -Path (Join-Path $stagingRoot "*") -DestinationPath $archivePath -CompressionLevel Optimal

Write-Output $archivePath
