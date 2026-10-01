# Signs one file with signtool. Used by the Windows build workflow for the engine and, through Tauri's
# bundle.windows.signCommand, for DeepServer.exe, the CLI and the installers.
#
# The workflow prepares the environment (see .github/workflows/build-windows.yml):
#   DEEPSERVER_SIGN_MODE   azure | pfx | none (none = do nothing, so unsigned builds still work)
#   azure: DEEPSERVER_SIGN_DLIB (Azure.CodeSigning.Dlib.dll) and DEEPSERVER_SIGN_METADATA (metadata.json);
#          the dlib authenticates with AZURE_TENANT_ID / AZURE_CLIENT_ID / AZURE_CLIENT_SECRET.
#   pfx:   DEEPSERVER_SIGN_PFX (path to the .pfx) and DEEPSERVER_SIGN_PFX_PASSWORD.
param(
  [Parameter(Mandatory = $true, Position = 0)]
  [string] $Path
)

$ErrorActionPreference = 'Stop'

$mode = $env:DEEPSERVER_SIGN_MODE
if ([string]::IsNullOrWhiteSpace($mode) -or $mode -eq 'none') {
  Write-Output "Signing not configured; leaving $Path unsigned."
  exit 0
}

$signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter signtool.exe |
  Where-Object FullName -match '\\x64\\' |
  Sort-Object FullName -Descending |
  Select-Object -First 1
if (-not $signtool) { throw 'signtool.exe not found (Windows SDK)' }

switch ($mode) {
  'azure' {
    & $signtool.FullName sign /v /fd SHA256 /tr http://timestamp.acs.microsoft.com /td SHA256 `
      /dlib $env:DEEPSERVER_SIGN_DLIB /dmdf $env:DEEPSERVER_SIGN_METADATA $Path
  }
  'pfx' {
    & $signtool.FullName sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 `
      /f $env:DEEPSERVER_SIGN_PFX /p $env:DEEPSERVER_SIGN_PFX_PASSWORD $Path
  }
  default { throw "Unknown DEEPSERVER_SIGN_MODE '$mode'" }
}
if ($LASTEXITCODE -ne 0) { throw "signtool failed for $Path (exit $LASTEXITCODE)" }
