# Silent install / upgrade / uninstall smoke test for a DeepServer setup.exe (standard or offline flavour).
# Installs with /S, checks the files and the HKLM Explorer menu keys, installs again over a planted rc1/rc2 leftover
# (open-diff-cli.exe) to check the in-place upgrade removes it, then uninstalls with /S and waits for the cleanup.
# Needs an elevated shell (the installer is per-machine). Used by .github/workflows/build-windows.yml.
param(
  [Parameter(Mandatory = $true)]
  [string] $Setup,
  [int] $TimeoutSeconds = 300
)

$ErrorActionPreference = 'Stop'

$setupPath = (Resolve-Path -LiteralPath $Setup).Path
$installDir = Join-Path $env:ProgramFiles 'DeepServer'
$exe = Join-Path $installDir 'DeepServer.exe'
$engine = Join-Path $installDir 'deepserver-diskusage.exe'
$cli = Join-Path $installDir 'deepserver-cli.exe'
$oldCli = Join-Path $installDir 'open-diff-cli.exe'

function Install-DeepServer {
  $p = Start-Process $setupPath -ArgumentList '/S' -PassThru
  if (-not $p.WaitForExit($TimeoutSeconds * 1000)) { throw 'installer timed out' }
  if ($p.ExitCode -ne 0) { throw "installer exited with $($p.ExitCode)" }
  foreach ($file in $exe, $engine, $cli) {
    if (-not (Test-Path -LiteralPath $file)) { throw "missing $file" }
  }
}

Write-Output "Installing $setupPath"
Install-DeepServer

Write-Output 'Upgrading in place over an rc1/rc2 leftover'
Copy-Item -LiteralPath $cli -Destination $oldCli
Install-DeepServer
if (Test-Path -LiteralPath $oldCli) { throw "upgrade left $oldCli behind" }

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
while ((Test-Path -LiteralPath $installDir) -or @($keys | Where-Object { Test-Path -LiteralPath $_ }).Count) {
  if ((Get-Date) -gt $deadline) {
    $left = @(Get-ChildItem -LiteralPath $installDir -Recurse -Force -ErrorAction SilentlyContinue | ForEach-Object FullName)
    $left += @($keys | Where-Object { Test-Path -LiteralPath $_ })
    throw "uninstall left files or menu keys behind: $($left -join ', ')"
  }
  Start-Sleep -Seconds 2
}
Write-Output "install/upgrade/uninstall OK: $(Split-Path -Leaf $setupPath)"
