param(
  [string]$VerbKey = 'DeepServer'
)

$ErrorActionPreference = 'Stop'

$keys = @(
  "HKCU:\Software\Classes\*\shell\$VerbKey",
  "HKCU:\Software\Classes\Directory\shell\$VerbKey",
  "HKCU:\Software\Classes\*\shell\${VerbKey}SelectLeft",
  "HKCU:\Software\Classes\Directory\shell\${VerbKey}SelectLeft",
  "HKCU:\Software\Classes\Directory\shell\${VerbKey}CopyVerify",
  "HKCU:\Software\Classes\Drive\shell\${VerbKey}CopyVerify"
)

foreach ($key in $keys) {
  if (Test-Path -LiteralPath $key) {
    Remove-Item -LiteralPath $key -Recurse -Force
  }
}

Write-Host 'Removed DeepServer Explorer context menu entries for files and folders.'
