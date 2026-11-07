<#
.SYNOPSIS
Checks that measure.ps1 measures correctly, with processes whose cost is
known.

.DESCRIPTION
Starts two PowerShell processes and measures them with measure.ps1:

- busy:  an empty endless loop, which keeps one core busy (100 %).
- sleep: a loop that sleeps 100 ms, which wakes up about 10 times per second.

Then compares the results with what is expected and with Get-Process, which
reads the same counters through other Windows functions. Fails if any check
does not pass.

.PARAMETER Duration
Length of the measurement window, in seconds.
#>
[CmdletBinding()]
param(
    [ValidateRange(2, 600)]
    [int]$Duration = 10
)

# Windows PowerShell 5.1 can read this file but not run it: tell how to run
# it with PowerShell 7. tools/check-powershell.ps1 checks that this works.
if ($PSVersionTable.PSVersion.Major -lt 7) {
    $command = "pwsh -File $(Resolve-Path -Relative $PSCommandPath)"
    if ($PSBoundParameters.Count -gt 0) { $command += ' <the same parameters>' }
    Write-Host "This script needs PowerShell 7. Run it with: $command" -ForegroundColor Yellow
    exit 1
}
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$invariant = [cultureinfo]::InvariantCulture
$pwsh = (Get-Process -Id $PID).Path
$outDir = Join-Path ([IO.Path]::GetTempPath()) "meiga-test-measure-$PID"
$checks = [System.Collections.Generic.List[object]]::new()

function Add-Check([string]$check, [string]$expected, $value, [bool]$passed) {
    $checks.Add([pscustomobject]@{
            Check    = $check
            Expected = $expected
            Measured = $value
            Result   = if ($passed) { 'ok' } else { 'FAILED' }
        })
}

function Start-Loop([string]$command) {
    Start-Process $pwsh -ArgumentList '-NoProfile', '-NonInteractive', '-Command', $command -NoNewWindow -PassThru
}

$busy = Start-Loop 'while ($true) {}'
$sleep = Start-Loop 'while ($true) { Start-Sleep -Milliseconds 100 }'
try {
    # Let both processes finish starting.
    Start-Sleep -Seconds 3

    $cpuBefore = (Get-Process -Id $busy.Id).TotalProcessorTime
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $rows = & (Join-Path $PSScriptRoot 'measure.ps1') -Id $busy.Id, $sleep.Id -Duration $Duration -SampleInterval 1 `
        -Label self-test -OutDir $outDir -PassThru
    $reference = ((Get-Process -Id $busy.Id).TotalProcessorTime - $cpuBefore).TotalSeconds / $clock.Elapsed.TotalSeconds * 100
    $busyNow = Get-Process -Id $busy.Id

    $busyRow = $rows | Where-Object { $_.kind -eq 'process' -and $_.pid -eq $busy.Id }
    $sleepRow = $rows | Where-Object { $_.kind -eq 'process' -and $_.pid -eq $sleep.Id }
    $groupRows = @($rows | Where-Object kind -EQ 'group')
    $terminal = $groupRows | Where-Object group -EQ 'terminal'
    $system = $groupRows | Where-Object group -EQ 'system'

    Add-Check 'busy: CPU' '80-110' $busyRow.cpu_pct ($busyRow.cpu_pct -ge 80 -and $busyRow.cpu_pct -le 110)
    Add-Check 'busy: CPU, Get-Process' "$([math]::Round($reference, 1)) +/- 5" $busyRow.cpu_pct ([math]::Abs($busyRow.cpu_pct - $reference) -le 5)
    $privateMb = $busyNow.PrivateMemorySize64 / 1MB
    Add-Check 'busy: private MB, Get-Process' "$([math]::Round($privateMb, 1)) +/- 20 %" $busyRow.private_mb_max `
        ([math]::Abs($busyRow.private_mb_max - $privateMb) -le 0.2 * $privateMb)
    Add-Check 'busy: threads, Get-Process' "$($busyNow.Threads.Count) +/- 5" $busyRow.threads_max ([math]::Abs($busyRow.threads_max - $busyNow.Threads.Count) -le 5)
    Add-Check 'sleep: wakeups per second' '8-200' $sleepRow.ctx_per_s ($sleepRow.ctx_per_s -ge 8 -and $sleepRow.ctx_per_s -le 200)
    Add-Check 'sleep: CPU' 'under 20' $sleepRow.cpu_pct ($sleepRow.cpu_pct -lt 20)
    Add-Check 'terminal: processes' '2' $terminal.processes ($terminal.processes -eq 2)
    Add-Check 'system: CPU' "$($busyRow.cpu_pct) or more" $system.cpu_pct ($system.cpu_pct -ge $busyRow.cpu_pct - 5)
    Add-Check 'groups in order' "$([Meiga.Perf.Tracker]::GroupNames.Count)" $groupRows.Count `
        (($groupRows.group -join ',') -eq ([Meiga.Perf.Tracker]::GroupNames -join ','))

    # The CSV must have the same rows, with numbers that do not depend on the
    # language of Windows.
    $csv = @(Get-ChildItem -LiteralPath $outDir -Filter 'measure-*-self-test.csv' | Import-Csv)
    $numeric = 'cpu_pct', 'ctx_per_s', 'private_mb_avg', 'seconds'
    $parsed = @($csv | Where-Object kind -EQ 'group' | Where-Object processes -NE '0' | ForEach-Object {
            foreach ($column in $numeric) {
                $number = 0.0
                [double]::TryParse($_.$column, [Globalization.NumberStyles]::Float, $invariant, [ref]$number)
            }
        })
    Add-Check 'CSV: rows' "$($rows.Count)" $csv.Count ($csv.Count -eq $rows.Count)
    Add-Check 'CSV: numbers with a dot' "$($parsed.Count)" @($parsed | Where-Object { $_ }).Count `
        ($parsed.Count -gt 0 -and $parsed -notcontains $false)

    $harness = $groupRows | Where-Object group -EQ 'harness'
    Write-Host "The measurement itself used $($harness.cpu_pct) % of one core."
}
finally {
    Stop-Process -Id $busy.Id, $sleep.Id -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $outDir -Recurse -Force -ErrorAction SilentlyContinue
}

$checks | Format-Table -AutoSize | Out-Host
$failed = @($checks | Where-Object Result -NE 'ok').Count
if ($failed -gt 0) {
    throw "$failed of $($checks.Count) checks failed."
}
Write-Host "All $($checks.Count) checks passed."
