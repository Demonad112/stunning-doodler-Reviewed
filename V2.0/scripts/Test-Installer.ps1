# Silent install / upgrade / uninstall smoke test for a DeepServer 2.0 setup.exe (standard or offline).
# Plants DeepServer 1.x leftovers (menu keys, extra programs), installs with /S, checks 2.0 removed them,
# installs again over itself, then uninstalls with /S and waits for the cleanup.
# Needs an elevated shell (the installer is per-machine). Used by .github/workflows/v2.yml and release-v2.yml.
param(
  [Parameter(Mandatory = $true)]
  [string] $Setup,
  [int] $TimeoutSeconds = 300
)

$ErrorActionPreference = 'Stop'

$setupPath = (Resolve-Path -LiteralPath $Setup).Path
$installDir = Join-Path $env:ProgramFiles 'DeepServer'
$exe = Join-Path $installDir 'DeepServer.exe'
$v1Files = 'deepserver-cli.exe', 'deepserver-diskusage.exe', 'open-diff-cli.exe' | ForEach-Object { Join-Path $installDir $_ }
$v1Key = 'HKLM:\Software\Classes\Directory\shell\DeepServerCopyVerify'

function Install-DeepServer {
  $p = Start-Process $setupPath -ArgumentList '/S' -PassThru
  if (-not $p.WaitForExit($TimeoutSeconds * 1000)) { throw 'installer timed out' }
  if ($p.ExitCode -ne 0) { throw "installer exited with $($p.ExitCode)" }
  if (-not (Test-Path -LiteralPath $exe)) { throw "missing $exe" }
}

Write-Output "Installing $setupPath"
Install-DeepServer

Write-Output 'Upgrading over planted DeepServer 1.x leftovers'
foreach ($file in $v1Files) { Set-Content -LiteralPath $file -Value 'x' }
New-Item -Path "$v1Key\command" -Force | Out-Null
Set-ItemProperty -LiteralPath "$v1Key\command" -Name '(default)' -Value '"DeepServer.exe" --copy-verify "%1"'
Install-DeepServer
foreach ($file in $v1Files) { if (Test-Path -LiteralPath $file) { throw "upgrade left $file behind" } }
if (Test-Path -LiteralPath $v1Key) { throw "upgrade left $v1Key behind" }

Write-Output 'Uninstalling'
$p = Start-Process (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -PassThru
$p.WaitForExit($TimeoutSeconds * 1000) | Out-Null
# The NSIS uninstaller re-launches itself from %TEMP%, so poll for the cleanup.
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
while (Test-Path -LiteralPath $installDir) {
  if ((Get-Date) -gt $deadline) {
    $left = @(Get-ChildItem -LiteralPath $installDir -Recurse -Force -ErrorAction SilentlyContinue | ForEach-Object FullName)
    throw "uninstall left files behind: $($left -join ', ')"
  }
  Start-Sleep -Seconds 2
}
Write-Output "install/upgrade/uninstall OK: $(Split-Path -Leaf $setupPath)"
