# Silent install / uninstall smoke test for a DeepServer setup.exe (standard or offline flavour).
# Installs with /S, checks the files and the HKLM Explorer menu keys, uninstalls with /S and waits for the cleanup.
# Needs an elevated shell (the installer is per-machine). Used by .github/workflows/build-windows.yml.
param(
  [Parameter(Mandatory = $true)]
  [string] $Setup,
  [int] $TimeoutSeconds = 300
)

$ErrorActionPreference = 'Stop'

$setupPath = (Resolve-Path -LiteralPath $Setup).Path
Write-Output "Installing $setupPath"
$p = Start-Process $setupPath -ArgumentList '/S' -PassThru
if (-not $p.WaitForExit($TimeoutSeconds * 1000)) { throw 'installer timed out' }
if ($p.ExitCode -ne 0) { throw "installer exited with $($p.ExitCode)" }

$installDir = Join-Path $env:ProgramFiles 'DeepServer'
$exe = Join-Path $installDir 'DeepServer.exe'
$engine = Join-Path $installDir 'deepserver-diskusage.exe'
foreach ($file in $exe, $engine) {
  if (-not (Test-Path -LiteralPath $file)) { throw "missing $file" }
}

$keys = @(
  'HKLM:\Software\Classes\*\shell\DeepServer',
  'HKLM:\Software\Classes\Directory\shell\DeepServerSelectLeft'
)
foreach ($k in $keys) {
  if (-not (Test-Path -LiteralPath "$k\command")) { throw "missing $k" }
}
$cmd = (Get-ItemProperty -LiteralPath "$($keys[0])\command").'(default)'
if ($cmd -notlike '*DeepServer.exe*--shell-compare*') { throw "unexpected menu command: $cmd" }

$diskKey = 'HKLM:\Software\Classes\Drive\shell\DeepServer Disk Usage\command'
if ((Get-ItemProperty -LiteralPath $diskKey).'(default)' -notlike '*deepserver-diskusage.exe*') { throw "missing $diskKey" }
$keys += 'HKLM:\Software\Classes\Directory\shell\DeepServer Disk Usage'

foreach ($root in 'Directory', 'Drive') {
  $copyKey = "HKLM:\Software\Classes\$root\shell\DeepServerCopyVerify"
  $copyCmd = (Get-ItemProperty -LiteralPath "$copyKey\command").'(default)'
  if ($copyCmd -notlike '*DeepServer.exe*--copy-verify*') { throw "unexpected copy-verify command: $copyCmd" }
  $keys += $copyKey
}

Write-Output 'Uninstalling'
$p = Start-Process (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -PassThru
$p.WaitForExit($TimeoutSeconds * 1000) | Out-Null
# The NSIS uninstaller re-launches itself from %TEMP%, so poll for the cleanup.
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
while ((Test-Path -LiteralPath $exe) -or (Test-Path -LiteralPath $engine) -or @($keys | Where-Object { Test-Path -LiteralPath $_ }).Count) {
  if ((Get-Date) -gt $deadline) { throw 'uninstall left files or menu keys behind' }
  Start-Sleep -Seconds 2
}
Write-Output "install/uninstall OK: $(Split-Path -Leaf $setupPath)"
