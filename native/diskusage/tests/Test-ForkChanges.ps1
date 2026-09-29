<#
    altWinDirStat fork tests: /compare significance filter and automatic scan history.
    Usage: powershell -File tests\Test-ForkChanges.ps1 -ExePath build\altWinDirStat_x64.exe [-SkipGui]
#>
param(
    [Parameter(Mandatory)] [string] $ExePath,
    [switch] $SkipGui
)
$ErrorActionPreference = 'Stop'
$ExePath = (Resolve-Path $ExePath).Path
$Fixtures = Join-Path $PSScriptRoot 'fixtures\changes'
$Work = Join-Path ([IO.Path]::GetTempPath()) ("awds-fork-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $Work | Out-Null
$failures = [System.Collections.Generic.List[string]]::new()
$passes = 0

function Check([bool] $condition, [string] $name, [string] $detail = '') {
    if ($condition) { $script:passes++; Write-Host "PASS  $name" }
    else { $script:failures.Add("$name $detail"); Write-Host "FAIL  $name $detail" -ForegroundColor Red }
}

# Quotes one argument for CommandLineToArgvW: backslashes before a quote are doubled, quotes escaped.
function ConvertTo-Arg([string] $value) {
    if ($value -ne '' -and $value -notmatch '[\s"]') { return $value }
    '"' + ($value -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"'
}

function Invoke-App([string[]] $arguments, [int] $timeoutSec = 60) {
    $p = Start-Process -FilePath $ExePath -ArgumentList (($arguments | ForEach-Object { ConvertTo-Arg $_ }) -join ' ') -PassThru
    if (-not $p.WaitForExit($timeoutSec * 1000)) { Stop-Process -Id $p.Id -Force; return -999 }
    # PS 5.1: after a timed WaitForExit the exit code can still read $null; the untimed wait fills it in.
    $p.WaitForExit()
    return [int] $p.ExitCode
}

function Read-Folders([string] $csv) {
    @(Import-Csv -LiteralPath $csv -Encoding UTF8 | ForEach-Object { $_.Folder })
}

# --- /compare: significance filter -------------------------------------------------
$base = Join-Path $Fixtures 'base.ledger.csv'
$cur = Join-Path $Fixtures 'cur.ledger.csv'

$out = Join-Path $Work 'significant.csv'
$code = Invoke-App @('/compare', $base, $cur, $out)
Check ($code -eq 0) '/compare exits 0' "(exit $code)"
if (Test-Path $out) {
    $folders = Read-Folders $out
    Check (($folders -join '|') -eq 'a\b\c|edge|gone|new') 'significant rows are a\b\c, edge, gone, new' "(got '$($folders -join '|')')"
} else { Check $false '/compare wrote output' }

$outAll = Join-Path $Work 'all.csv'
$code = Invoke-App @('/compare', $base, $cur, $outAll, '/all')
Check ($code -eq 0) '/compare /all exits 0' "(exit $code)"
if (Test-Path $outAll) {
    $folders = Read-Folders $outAll
    $expected = '.|a|a\b|a\b\c|edge|gone|gone\sub|new|new\inner|small'
    Check (($folders -join '|') -eq $expected) '/all lists every changed row' "(got '$($folders -join '|')')"
} else { Check $false '/compare /all wrote output' }

$code = Invoke-App @('/compare', (Join-Path $Fixtures 'bad.ledger.csv'), $cur, (Join-Path $Work 'bad.csv'))
Check ($code -eq 1) 'malformed ledger gives exit 1' "(exit $code)"

$code = Invoke-App @('/compare', $base)
Check ($code -eq 2) 'missing arguments give exit 2' "(exit $code)"

# --- /compare: folders differing only by letter case stay separate (E7) -----------------
function Read-Changes([string] $csv) {
    [string[]] $rows = @(Import-Csv -LiteralPath $csv -Encoding UTF8 | ForEach-Object { "$($_.Folder)=$($_.Change)" })
    [Array]::Sort($rows, [StringComparer]::Ordinal)
    $rows
}
$caseBase = Join-Path $Fixtures 'case-base.ledger.csv'
$caseCur = Join-Path $Fixtures 'case-cur.ledger.csv'
$out = Join-Path $Work 'case-all.csv'
$code = Invoke-App @('/compare', $caseBase, $caseCur, $out, '/all')
$got = if (Test-Path $out) { (Read-Changes $out) -join '|' } else { '' }
Check ($code -eq 0 -and $got -ceq '.=Grown|Data=Grown|Data\sub=Grown|data=Grown|logs=Added') 'case-only siblings are matched separately' "(exit $code, got '$got')"
$out = Join-Path $Work 'case-sig.csv'
$code = Invoke-App @('/compare', $caseBase, $caseCur, $out)
$got = if (Test-Path $out) { (Read-Changes $out) -join '|' } else { '' }
Check ($code -eq 0 -and $got -ceq 'Data\sub=Grown|data=Grown|logs=Added') 'case-only siblings are significant on their own' "(exit $code, got '$got')"

# --- /compare: a ledger needs exactly one root row (E8) ---------------------------------
foreach ($name in 'noroot', 'duproot') {
    $out = Join-Path $Work "$name.csv"
    $code = Invoke-App @('/compare', (Join-Path $Fixtures "$name.ledger.csv"), $cur, $out)
    $err = if (Test-Path "$out.err") { (Get-Content -LiteralPath "$out.err" -Raw).Trim() } else { '' }
    Check ($code -eq 1 -and $err -match 'root') "$name ledger is rejected" "(exit $code, err '$err')"
}

# --- GUI scan history (added in Task 2) -----------------------------------------------
if (-not $SkipGui) {
    $history = Join-Path $Work 'history'
    $env:DEEPSERVER_HISTORY_DIR = $history
    $tree = Join-Path $Work 'tree'
    New-Item -ItemType Directory -Force (Join-Path $tree 'deep\nested') | Out-Null
    New-Item -ItemType Directory -Force (Join-Path $tree 'other') | Out-Null
    Set-Content -LiteralPath (Join-Path $tree 'other\keep.txt') -Value 'x'

    function Get-Snapshots {
        if (-not (Test-Path $history)) { return @() }
        @(Get-ChildItem -LiteralPath $history -Recurse -Filter '*.ledger.csv' | Sort-Object Name)
    }

    function Get-NewestWrite {
        $s = Get-Snapshots | Sort-Object LastWriteTimeUtc | Select-Object -Last 1
        if ($s) { $s.LastWriteTimeUtc } else { [datetime]::MinValue }
    }

    # Scans $target in the GUI and returns once a snapshot newer than any before the scan exists (or times out).
    function Invoke-GuiScan([string] $target) {
        $since = Get-NewestWrite
        $p = Start-Process -FilePath $ExePath -ArgumentList ('/noelevate ' + (ConvertTo-Arg $target)) -PassThru
        $sw = [Diagnostics.Stopwatch]::StartNew()
        while ((Get-NewestWrite) -le $since -and $sw.Elapsed.TotalSeconds -lt 60 -and -not $p.HasExited) {
            Start-Sleep -Milliseconds 200
        }
        Start-Sleep -Milliseconds 300
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        $p.WaitForExit(10000) | Out-Null
    }

    # 1. First scan writes one snapshot and a readable location.txt
    Invoke-GuiScan $tree
    $snaps = Get-Snapshots
    Check ($snaps.Count -eq 1) 'first scan writes one snapshot' "(found $($snaps.Count))"
    $locations = @(Get-ChildItem -LiteralPath $history -Directory -ErrorAction SilentlyContinue)
    Check ($locations.Count -eq 1) 'one location folder' "(found $($locations.Count))"
    if ($locations.Count -ge 1) {
        $loc = Get-Content -LiteralPath (Join-Path $locations[0].FullName 'location.txt') -Raw
        # Compare only the leaf: %TEMP% may be an 8.3 short path that the app does not expand.
        Check ($loc.Trim() -clike '*\tree') 'location.txt holds the lower-cased root' "(got '$loc')"
        Check ($locations[0].Name -match '^[0-9a-f]{16}$') 'location folder is a 16-hex hash' "(got '$($locations[0].Name)')"
    }

    # 2. Grow a nested folder by 5 MB and scan again in a new process
    $bytes = New-Object byte[] (5MB)
    [IO.File]::WriteAllBytes((Join-Path $tree 'deep\nested\big.bin'), $bytes)
    Invoke-GuiScan $tree
    $snaps = Get-Snapshots
    Check ($snaps.Count -eq 2) 'second session writes a second snapshot' "(found $($snaps.Count))"

    # 3. The two snapshots compare to exactly the nested folder
    if ($snaps.Count -eq 2) {
        $diff = Join-Path $Work 'gui-diff.csv'
        $code = Invoke-App @('/compare', $snaps[0].FullName, $snaps[1].FullName, $diff)
        $folders = if (Test-Path $diff) { Read-Folders $diff } else { @() }
        Check ($code -eq 0 -and ($folders -join '|') -eq 'deep\nested') 'snapshot diff shows only deep\nested' "(exit $code, got '$($folders -join '|')')"
    }

    # 4. Same folder with different case and a trailing slash maps to the same location
    Invoke-GuiScan ($tree.ToUpperInvariant() + '\')
    $locations = @(Get-ChildItem -LiteralPath $history -Directory)
    Check ($locations.Count -eq 1) 'case/trailing-slash variants share one location' "(found $($locations.Count))"

    # 5. A corrupt newer snapshot, and DeepServer's in-progress file, are skipped, not fatal
    $locDir = $locations[0].FullName
    Set-Content -LiteralPath (Join-Path $locDir '29991231-235959-999.ledger.csv') -Value 'garbage'
    Set-Content -LiteralPath (Join-Path $locDir '.partial-29991231-235959-998.ledger.csv') -Value 'garbage'
    $before = (Get-Snapshots).Count
    Invoke-GuiScan $tree
    Check ((Get-Snapshots).Count -eq $before + 1 -or (Get-Snapshots).Count -eq 5) 'scan still writes with a corrupt snapshot present'
    Remove-Item -LiteralPath (Join-Path $locDir '29991231-235959-999.ledger.csv') -ErrorAction SilentlyContinue
    Check (Test-Path -LiteralPath (Join-Path $locDir '.partial-29991231-235959-998.ledger.csv')) 'retention leaves in-progress .partial-* files alone'
    Remove-Item -LiteralPath (Join-Path $locDir '.partial-29991231-235959-998.ledger.csv') -ErrorAction SilentlyContinue

    # 6. Retention keeps the newest 5
    for ($i = 0; $i -lt 6; $i++) { Invoke-GuiScan $tree }
    $count = @(Get-ChildItem -LiteralPath $locDir -Filter '*.ledger.csv').Count
    Check ($count -eq 5) 'retention keeps 5 snapshots' "(found $count)"
    Check (@(Get-ChildItem -LiteralPath $locDir -Filter '*.tmp').Count -eq 0) 'no leftover .tmp files'

    Remove-Item Env:\DEEPSERVER_HISTORY_DIR
}

Remove-Item $Work -Recurse -Force -ErrorAction SilentlyContinue
Write-Host ""
Write-Host "$passes passed, $($failures.Count) failed"
if ($failures.Count -gt 0) { exit 1 }
exit 0
