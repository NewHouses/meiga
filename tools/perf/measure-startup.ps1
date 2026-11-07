<#
.SYNOPSIS
Measures how long meiga.exe takes to open its window.

.DESCRIPTION
Starts meiga.exe -Runs times, one after another. For each run, it measures the
time from starting the process until meiga has painted its window and waits
for input (WaitForInputIdle), which is the time until the first frame. Then
it closes the window, as the X button does, checks that meiga ends with exit
code 0 and waits 0.5 s before the next run.

The first -Warmup runs are not counted: they bring the executable and its
DLLs into the file cache, so the results describe a normal start and not the
first one after a reboot.

Prints the lowest, median, 95th percentile and highest times, and saves every
run as CSV in .perf\, which git ignores because the times describe this
machine. Numbers in the CSV always use a dot as the decimal separator.

.PARAMETER Path
The executable to measure. Defaults to target\release\meiga.exe.

.PARAMETER Runs
Number of runs that are counted.

.PARAMETER Warmup
Number of runs before them that are not counted.

.PARAMETER Label
Name of the measurement, for the CSV file. Defaults to the name of the
executable.

.PARAMETER OutDir
Folder for the CSV file. Defaults to .perf\ in the repository.

.PARAMETER TimeoutSeconds
How long to wait for the window, and then for the process to end, in each
run.

.EXAMPLE
pwsh -File .\tools\perf\measure-startup.ps1

.EXAMPLE
pwsh -File .\tools\perf\measure-startup.ps1 -Runs 50 -Label meiga-1.1
#>
[CmdletBinding()]
param(
    [string]$Path,

    [ValidateRange(1, 10000)]
    [int]$Runs = 20,

    [ValidateRange(0, 100)]
    [int]$Warmup = 2,

    [ValidatePattern('^[\w.-]+$')]
    [string]$Label,

    [string]$OutDir,

    [ValidateRange(1, 600)]
    [int]$TimeoutSeconds = 10
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

# Not in param(): there, Windows PowerShell 5.1 leaves $PSScriptRoot empty and
# fails before the check above.
if (-not $Path) {
    $Path = Join-Path $PSScriptRoot '..\..\target\release\meiga.exe'
}
$Path = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path)
if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "$Path does not exist. Build it first with: cargo build --release"
}
if (-not $OutDir) {
    $OutDir = Join-Path $PSScriptRoot '..\..\.perf'
}
$OutDir = [IO.Path]::GetFullPath($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutDir))
$exe = Split-Path -Leaf $Path
if (-not $Label) {
    $Label = [IO.Path]::GetFileNameWithoutExtension($Path)
}
$invariant = [cultureinfo]::InvariantCulture

# Starts the program once and returns the milliseconds until its window was
# ready.
function Measure-Start {
    $info = [Diagnostics.ProcessStartInfo]::new($Path)
    $info.UseShellExecute = $false
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $process = [Diagnostics.Process]::Start($info)
    try {
        $idle = try { $process.WaitForInputIdle($TimeoutSeconds * 1000) } catch { $false }
        $clock.Stop()
        if (-not $idle) {
            if ($process.HasExited) {
                throw "$exe ended with exit code $($process.ExitCode) before it opened its window."
            }
            throw "$exe did not open its window within $TimeoutSeconds s."
        }
        if (-not $process.CloseMainWindow()) {
            throw "Could not ask the window of $exe to close."
        }
        if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
            throw "$exe was still running $TimeoutSeconds s after its window was asked to close."
        }
        if ($process.ExitCode -ne 0) {
            throw "$exe ended with exit code $($process.ExitCode) instead of 0."
        }
    }
    finally {
        if (-not $process.HasExited) {
            $process.Kill()
        }
        $process.Dispose()
    }
    # Let the closed window disappear before the next start.
    Start-Sleep -Milliseconds 500
    $clock.Elapsed.TotalMilliseconds
}

Write-Host "Starting $exe $Warmup + $Runs times..."
for ($run = 1; $run -le $Warmup; $run++) {
    $null = Measure-Start
}
$start = Get-Date
$times = for ($run = 1; $run -le $Runs; $run++) {
    [pscustomobject]@{ run = $run; ms = Measure-Start }
}

# Nearest-rank percentile of the sorted times.
$sorted = @($times.ms | Sort-Object)
function Get-Percentile([double]$Percent) {
    $sorted[[int][math]::Max(0, [math]::Ceiling($Percent / 100 * $sorted.Count) - 1)]
}
$middle = [int][math]::Floor($sorted.Count / 2)
$median = if ($sorted.Count % 2) { $sorted[$middle] } else { ($sorted[$middle - 1] + $sorted[$middle]) / 2 }
[pscustomobject]@{
    runs      = $Runs
    min_ms    = [math]::Round($sorted[0], 1)
    median_ms = [math]::Round($median, 1)
    p95_ms    = [math]::Round((Get-Percentile 95), 1)
    max_ms    = [math]::Round($sorted[-1], 1)
} | Format-Table -AutoSize | Out-Host

# The CSV uses the invariant culture: with the Galician or Spanish culture,
# Export-Csv would write 1,5 instead of 1.5.
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$file = Join-Path $OutDir ('startup-{0}-{1}.csv' -f $start.ToString('yyyyMMdd-HHmmss', $invariant), $Label)
$times | ForEach-Object {
    [pscustomobject]@{ run = $_.run; ms = $_.ms.ToString('0.000', $invariant) }
} | Export-Csv -LiteralPath $file -NoTypeInformation -Encoding utf8NoBOM
Write-Host "Time from starting $exe until its window is painted and waits for input."
Write-Host "Saved $file"
