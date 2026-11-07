<#
.SYNOPSIS
Measures what the frames of meiga cost: a partial frame and a full frame.

.DESCRIPTION
For each mode, starts meiga.exe maximized with --bench, which draws frames
without a pause, and with its frame log (--trace-frames):

- partial: a 64-pixel square crosses the window, so each frame presents a
  small part of it;
- full: the whole window changes colour, so each frame presents all of it.

After -Warmup seconds, measure.ps1 measures meiga, dwm.exe (which composes the
windows on the screen) and the Remote Desktop service for -Duration seconds.
Then the window is closed, and the frame log gives the frames per second and
the median and 95th percentile of the time to compose a frame and to present
it, from the end of the warm-up to the end.

Prints a table and saves it as CSV in .perf\, with the frame logs, which git
ignores because they describe this machine. Numbers in the CSV always use a
dot as the decimal separator. Do not use the computer while it measures: the
other windows and the mouse change what dwm.exe and Remote Desktop do.

.PARAMETER Mode
partial, full or both.

.PARAMETER Duration
Seconds of each measurement.

.PARAMETER Warmup
Seconds between the start of meiga and the measurement.

.PARAMETER Path
The executable to measure. Defaults to target\release\meiga.exe.

.PARAMETER Label
Name of the measurement, for the files. Defaults to meiga.

.PARAMETER OutDir
Folder for the files. Defaults to .perf\ in the repository.

.EXAMPLE
pwsh -File .\tools\perf\measure-frames.ps1

.EXAMPLE
pwsh -File .\tools\perf\measure-frames.ps1 -Mode full -Duration 60
#>
[CmdletBinding()]
param(
    [ValidateSet('partial', 'full', 'both')]
    [string]$Mode = 'both',

    [ValidateRange(1, 3600)]
    [int]$Duration = 20,

    [ValidateRange(0, 600)]
    [int]$Warmup = 3,

    [string]$Path,

    [ValidatePattern('^[\w.-]+$')]
    [string]$Label = 'meiga',

    [string]$OutDir
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
$measure = Join-Path $PSScriptRoot 'measure.ps1'
$exe = Split-Path -Leaf $Path
$invariant = [cultureinfo]::InvariantCulture
$modes = if ($Mode -eq 'both') { @('partial', 'full') } else { @($Mode) }
$run = Get-Date
$stamp = $run.ToString('yyyyMMdd-HHmmss', $invariant)
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# Nearest-rank percentile of numbers that are already sorted.
function Get-Percentile([double[]]$Sorted, [double]$Percent) {
    if ($Sorted.Count -eq 0) { return $null }
    $Sorted[[int][math]::Max(0, [math]::Ceiling($Percent / 100 * $Sorted.Count) - 1)]
}

function Get-Median([double[]]$Sorted) {
    if ($Sorted.Count -eq 0) { return $null }
    $middle = [int][math]::Floor($Sorted.Count / 2)
    if ($Sorted.Count % 2) { $Sorted[$middle] } else { ($Sorted[$middle - 1] + $Sorted[$middle]) / 2 }
}

function Measure-Mode([string]$Bench) {
    $log = Join-Path $OutDir "frames-$stamp-$Label-$Bench.trace.csv"
    $info = [Diagnostics.ProcessStartInfo]::new($Path)
    $info.UseShellExecute = $true
    $info.WindowStyle = [Diagnostics.ProcessWindowStyle]::Maximized
    $info.Arguments = "--bench $Bench --trace-frames `"$log`""
    $process = [Diagnostics.Process]::Start($info)
    try {
        $idle = try { $process.WaitForInputIdle(10000) } catch { $false }
        if (-not $idle) {
            throw "$exe did not open its window."
        }
        Start-Sleep -Seconds $Warmup
        $rows = @(& $measure -Id $process.Id -Duration $Duration -Label "$Label-frames-$Bench" -OutDir $OutDir -Quiet -PassThru)
        if (-not $process.CloseMainWindow() -or -not $process.WaitForExit(10000)) {
            throw "$exe did not close its window."
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

    $frames = @(Import-Csv -LiteralPath $log | Where-Object {
            $_.kind -eq 'frame' -and [double]::Parse($_.time_ms, $invariant) -ge $Warmup * 1000
        })
    if ($frames.Count -lt 2) {
        throw "meiga drew $($frames.Count) frames after the warm-up in $Bench mode. See $log"
    }
    $first = [double]::Parse($frames[0].time_ms, $invariant)
    $last = [double]::Parse($frames[-1].time_ms, $invariant)
    $compose = [double[]]@($frames | ForEach-Object { [double]$_.compose_us } | Sort-Object)
    $present = [double[]]@($frames | ForEach-Object { [double]$_.present_us } | Sort-Object)
    $pixels = [double[]]@($frames | ForEach-Object { [double]$_.pixels } | Sort-Object)
    $group = @{}
    foreach ($row in $rows | Where-Object kind -EQ 'group') {
        $group[$row.group] = $row
    }
    [pscustomobject][ordered]@{
        mode            = $Bench
        width           = [int]$frames[-1].width
        height          = [int]$frames[-1].height
        frames          = $frames.Count
        fps             = [math]::Round(($frames.Count - 1) / ($last - $first) * 1000, 1)
        kpixels         = [math]::Round((Get-Median $pixels) / 1000, 1)
        compose_us      = Get-Median $compose
        compose_p95_us  = Get-Percentile $compose 95
        present_us      = Get-Median $present
        present_p95_us  = Get-Percentile $present 95
        meiga_cpu_pct   = $group['terminal'].cpu_pct
        meiga_ctx_per_s = $group['terminal'].ctx_per_s
        meiga_mb        = $group['terminal'].private_mb_max
        dwm_cpu_pct     = $group['dwm'].cpu_pct
        rdp_cpu_pct     = $group['termservice'].cpu_pct
    }
}

$results = foreach ($bench in $modes) {
    Write-Host "Measuring $bench frames: $Warmup s of warm-up and $Duration s of measurement..."
    Measure-Mode $bench
}

$results | Format-Table mode, width, height, fps, kpixels, compose_us, compose_p95_us, present_us, present_p95_us -AutoSize | Out-Host
$results | Format-Table mode, meiga_cpu_pct, meiga_ctx_per_s, meiga_mb, dwm_cpu_pct, rdp_cpu_pct -AutoSize | Out-Host

# The CSV uses the invariant culture: with the Galician or Spanish culture,
# Export-Csv would write 1,5 instead of 1.5.
$file = Join-Path $OutDir "frames-$stamp-$Label.csv"
$results | ForEach-Object {
    $text = [ordered]@{}
    foreach ($property in $_.PSObject.Properties) {
        $value = $property.Value
        $text[$property.Name] = if ($value -is [IFormattable]) { $value.ToString($null, $invariant) } else { $value }
    }
    [pscustomobject]$text
} | Export-Csv -LiteralPath $file -NoTypeInformation -Encoding utf8NoBOM
Write-Host 'kpixels: thousands of pixels presented by a frame. Times in microseconds: median and 95th percentile.'
Write-Host 'CPU in % of one core: meiga, dwm.exe and the Remote Desktop service (rdp).'
Write-Host "Saved $file"
