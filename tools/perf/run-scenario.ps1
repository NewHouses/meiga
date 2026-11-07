<#
.SYNOPSIS
Measures a terminal while it runs a load scenario: several windows in a row,
with a summary and a verdict on how stable the results are.

.DESCRIPTION
Run it in a terminal other than the one that you measure. It:

1. prepares the run folder, .perf\run\, with the file "controller", which
   tells the scenarios that this script coordinates them;
2. shows the command that starts the scenario in the terminal that you
   measure, in PowerShell or in WSL, and waits until -Instances copies of the
   scenario have started, for example one per tab;
3. waits -Warmup seconds and measures -Runs windows in a row with
   measure.ps1, and after each window prints the CPU and the cycles of the
   terminal. For bigfile, each window lasts what printing the file takes,
   plus 1 s;
4. creates the file "stop", which ends the scenarios, and prints the
   summary: for each group of processes, the mean, the lowest and the highest
   CPU, the CPU time, the cycles, the wakeups, the memory and the threads; for
   bigfile, also the time that printing took;
5. saves the summary as CSV in .perf\, next to the CSV of each window.

The results are stable when the cycles of the terminal vary between windows
at most 10 % of their mean, or at most one million per second when that is
more; for bigfile, when the time of printing varies at most 10 %. Cycles, and
not CPU time, because Windows charges CPU time in clock ticks of 15.6 ms,
which is too coarse for the small amounts that a terminal uses.

idle runs no scenario: it measures the terminal as you leave it, for example
with 5 tabs open, after -Delay seconds to bring it to the front.

.PARAMETER Scenario
idle, spinner, ai-stream or bigfile.

.PARAMETER Name
Executable name of the terminal, as in measure.ps1: warp, WindowsTerminal,
meiga...

.PARAMETER Id
Process ids of the terminal, instead of -Name.

.PARAMETER Shell
Shell of the terminal where the scenario runs: pwsh or wsl. It only changes
the command that the script shows.

.PARAMETER Instances
How many copies of the scenario to wait for, for example 5 for a spinner in
5 tabs. bigfile runs a single copy.

.PARAMETER Runs
Number of measurement windows.

.PARAMETER Duration
Seconds of each window. For bigfile, the longest window, which is 600 s if
you do not give it: the window ends when the file has been printed.

.PARAMETER Warmup
Seconds between the start of the last copy of the scenario and the first
window.

.PARAMETER Delay
For idle, seconds before the first window.

.PARAMETER Label
Name of the measurement, for the CSV files. Defaults to the terminal and the
scenario, such as WindowsTerminal-spinner.

.PARAMETER OutDir
Folder for the CSV files. Defaults to .perf\ in the repository.

.EXAMPLE
pwsh -File .\tools\perf\run-scenario.ps1 spinner -Name WindowsTerminal

.EXAMPLE
pwsh -File .\tools\perf\run-scenario.ps1 ai-stream -Name warp -Shell wsl

.EXAMPLE
pwsh -File .\tools\perf\run-scenario.ps1 spinner -Name warp -Instances 5 -Label warp-spinner-5-tabs
#>
[CmdletBinding(DefaultParameterSetName = 'Name')]
param(
    [Parameter(Mandatory, Position = 0)]
    [ValidateSet('idle', 'spinner', 'ai-stream', 'bigfile')]
    [string]$Scenario,

    [Parameter(Mandatory, ParameterSetName = 'Name', Position = 1)]
    [string]$Name,

    [Parameter(Mandatory, ParameterSetName = 'Id')]
    [int[]]$Id,

    [ValidateSet('pwsh', 'wsl')]
    [string]$Shell = 'pwsh',

    [ValidateRange(1, 50)]
    [int]$Instances = 1,

    [ValidateRange(1, 20)]
    [int]$Runs = 3,

    [ValidateRange(1, 3600)]
    [int]$Duration = 60,

    [ValidateRange(0, 600)]
    [int]$Warmup = 5,

    [ValidateRange(0, 600)]
    [int]$Delay = 10,

    [ValidatePattern('^[\w.-]+$')]
    [string]$Label,

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
Import-Module (Join-Path $PSScriptRoot 'scenarios/Scenario.psm1') -Function Get-RunDirectory -Force

$invariant = [cultureinfo]::InvariantCulture
$measure = Join-Path $PSScriptRoot 'measure.ps1'
# Not in param(): there, Windows PowerShell 5.1 leaves $PSScriptRoot empty and
# fails before the check above.
if (-not $OutDir) {
    $OutDir = Join-Path $PSScriptRoot '../../.perf'
}
$OutDir = [IO.Path]::GetFullPath($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutDir))
$runDirectory = Get-RunDirectory
$started = Get-Date

if ($Scenario -eq 'bigfile' -and $Instances -ne 1) {
    throw 'bigfile runs a single copy: use -Instances 1.'
}
if ($Scenario -eq 'idle' -and $Instances -ne 1) {
    throw 'idle runs no scenario: open the tabs that you want and leave out -Instances.'
}

# Check the terminal before asking anything of the user.
if ($PSCmdlet.ParameterSetName -eq 'Id') {
    $target = @{ Id = $Id }
    foreach ($processId in $Id) {
        if (-not (Get-Process -Id $processId -ErrorAction Ignore)) {
            throw "There is no process with id $processId."
        }
    }
    $terminalLabel = "pid$($Id[0])"
}
else {
    $Name = $Name -replace '\.exe$', ''
    $target = @{ Name = $Name }
    if (-not (Get-Process -Name $Name -ErrorAction Ignore)) {
        throw "No process named $Name.exe is running. Open the terminal first."
    }
    $terminalLabel = $Name
}
if (-not $Label) {
    $Label = "$terminalLabel-$Scenario"
    if ($Instances -gt 1) {
        $Label += "-$Instances"
    }
    if ($Shell -eq 'wsl' -and $Scenario -ne 'idle') {
        $Label += '-wsl'
    }
}

# The command that starts the scenario in the terminal that is measured.
$command = $null
if ($Scenario -ne 'idle') {
    if ($Scenario -eq 'bigfile') {
        $bigfile = Join-Path $PSScriptRoot '../../.perf/bigfile.txt'
        if (-not (Test-Path -LiteralPath $bigfile)) {
            Write-Host 'Creating .perf\bigfile.txt, which bigfile prints...'
            & (Join-Path $PSScriptRoot 'gen-bigfile.ps1') -Path $bigfile
        }
    }
    if ($Shell -eq 'pwsh') {
        $script = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "scenarios/$Scenario.ps1"))
        $command = "pwsh -NoProfile -File `"$script`""
    }
    else {
        $script = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "scenarios/$Scenario.sh"))
        $linuxPath = "$(& wsl.exe -e wslpath -a $script 2>$null)".Trim()
        if ($LASTEXITCODE -ne 0 -or -not $linuxPath) {
            throw 'WSL does not answer: check that wsl.exe opens your Linux distribution.'
        }
        $command = "bash '$($linuxPath -replace "'", "'\''")'"
    }
}

# A clean run folder. The file "controller" makes the scenarios say that they
# have started and wait for "stop".
New-Item -ItemType Directory -Force -Path $runDirectory | Out-Null
Get-ChildItem -LiteralPath $runDirectory -File | Remove-Item -Force
New-Item -ItemType File -Path (Join-Path $runDirectory 'controller') | Out-Null

function Get-StartedCount {
    @(Get-ChildItem -LiteralPath $runDirectory -Filter "started-$Scenario-*").Count
}

$windows = [Collections.Generic.List[object]]::new()
try {
    if ($Scenario -eq 'idle') {
        Write-Host "Leave the terminal as you want to measure it, for example with 5 tabs open, and bring it to the front."
        Write-Host "Measuring starts in $Delay s."
        Start-Sleep -Seconds $Delay
    }
    else {
        $copies = if ($Instances -gt 1) { "in $Instances tabs" } else { 'once' }
        Write-Host "In the terminal that you measure, run this $($copies):"
        Write-Host ''
        Write-Host "    $command" -ForegroundColor Cyan
        Write-Host ''
        Write-Host 'Waiting for the scenario. Press Ctrl+C to cancel.'
        $count = -1
        while ($count -lt $Instances) {
            $now = Get-StartedCount
            if ($now -ne $count) {
                $count = $now
                Write-Host "Started: $count of $Instances"
            }
            Start-Sleep -Milliseconds 250
        }
        Write-Host "Leave the terminal as you want to measure it, for example with the right tab in front. Measuring starts in $Warmup s."
        Start-Sleep -Seconds $Warmup
    }
    Write-Host "Do not touch anything until the end: $Runs windows, then the summary."

    for ($run = 1; $run -le $Runs; $run++) {
        $parameters = @{
            Duration = $Duration
            Label    = "$Label-$run"
            OutDir   = $OutDir
            Quiet    = $true
            PassThru = $true
        } + $target
        $finished = Join-Path $runDirectory "finished-$run"
        if ($Scenario -eq 'bigfile') {
            $parameters.Duration = if ($PSBoundParameters.ContainsKey('Duration')) { $Duration } else { 600 }
            $parameters.SampleInterval = 1
            $parameters.StartedFile = Join-Path $runDirectory "go-$run"
            $parameters.StopFile = $finished
        }
        $rows = @(& $measure @parameters)
        $terminal = $rows | Where-Object { $_.kind -eq 'group' -and $_.group -eq 'terminal' }
        $window = [pscustomobject]@{
            Rows  = $rows
            Print = $null
        }
        $line = [string]::Format($invariant, 'Window {0} of {1}: the terminal used {2:0.00} % of one core and {3:0.00} million cycles per second',
            $run, $Runs, $terminal.cpu_pct, $terminal.mcycles_per_s)
        if ($Scenario -eq 'bigfile') {
            if (-not (Test-Path -LiteralPath $finished)) {
                throw "bigfile did not finish within $($parameters.Duration) s. Give a longer -Duration."
            }
            $window.Print = [double]::Parse((Get-Content -LiteralPath $finished -Raw).Trim(), $invariant)
            $line += [string]::Format($invariant, ', {0:0.000} s of CPU, while printing took {1:0.000} s', $terminal.cpu_s, $window.Print)
        }
        Write-Host "$line."
        $windows.Add($window)
    }
}
finally {
    # End the scenarios and wait a little for them to delete their markers.
    New-Item -ItemType File -Force -Path (Join-Path $runDirectory 'stop') | Out-Null
    $clock = [Diagnostics.Stopwatch]::StartNew()
    while ((Get-StartedCount) -gt 0 -and $clock.Elapsed.TotalSeconds -lt 5) {
        Start-Sleep -Milliseconds 250
    }
    $left = Get-StartedCount
    if ($left -gt 0) {
        Write-Warning "$left copies of the scenario are still running: stop them with Ctrl+C."
    }
    Remove-Item -LiteralPath (Join-Path $runDirectory 'controller') -ErrorAction Ignore
}

function Get-Statistic([object[]]$Values, [string]$Kind, [int]$Digits) {
    $numbers = @($Values | Where-Object { $null -ne $_ })
    if ($numbers.Count -eq 0) {
        return $null
    }
    $measured = $numbers | Measure-Object -Average -Minimum -Maximum
    $value = [math]::Round($measured.$Kind, $Digits)
    if ($Digits -eq 0) {
        return [long]$value
    }
    $value
}

$prints = @($windows | ForEach-Object Print | Where-Object { $null -ne $_ })
$groupNames = @($windows[0].Rows | Where-Object kind -EQ 'group' | ForEach-Object group)
$summary = foreach ($group in $groupNames) {
    $values = @($windows | ForEach-Object { $_.Rows | Where-Object { $_.kind -eq 'group' -and $_.group -eq $group } })
    [pscustomobject][ordered]@{
        scenario          = $Scenario
        label             = $Label
        shell             = if ($Scenario -eq 'idle') { $null } else { $Shell }
        instances         = if ($Scenario -eq 'idle') { $null } else { $Instances }
        runs              = $windows.Count
        group             = $group
        processes         = Get-Statistic $values.processes Maximum 0
        cpu_pct           = Get-Statistic $values.cpu_pct Average 2
        cpu_pct_min       = Get-Statistic $values.cpu_pct Minimum 2
        cpu_pct_max       = Get-Statistic $values.cpu_pct Maximum 2
        cpu_s             = Get-Statistic $values.cpu_s Average 3
        mcycles_per_s     = Get-Statistic $values.mcycles_per_s Average 2
        mcycles_per_s_min = Get-Statistic $values.mcycles_per_s Minimum 2
        mcycles_per_s_max = Get-Statistic $values.mcycles_per_s Maximum 2
        ctx_per_s         = Get-Statistic $values.ctx_per_s Average 2
        private_mb_max    = Get-Statistic $values.private_mb_max Maximum 1
        threads_max       = Get-Statistic $values.threads_max Maximum 0
        window_s          = Get-Statistic $values.seconds Average 2
        print_s           = Get-Statistic $prints Average 3
        print_s_min       = Get-Statistic $prints Minimum 3
        print_s_max       = Get-Statistic $prints Maximum 3
    }
}

Write-Host ''
Write-Host "Mean of the $($windows.Count) windows; CPU in % of one core, CPU time in seconds per window, cycles in millions per second:"
$summary | Format-Table group, processes, cpu_pct, cpu_pct_min, cpu_pct_max, cpu_s, mcycles_per_s, ctx_per_s, private_mb_max, threads_max -AutoSize |
    Out-Host

$terminal = $summary | Where-Object group -EQ 'terminal'
if ($Scenario -eq 'bigfile') {
    $spread = $terminal.print_s_max - $terminal.print_s_min
    $limit = 0.1 * $terminal.print_s
    Write-Host ([string]::Format($invariant, 'Printing took {0:0.000} s on average, from {1:0.000} to {2:0.000} s.',
            $terminal.print_s, $terminal.print_s_min, $terminal.print_s_max))
    $what = [string]::Format($invariant, 'the time of printing varied {0:0.000} s between windows (limit {1:0.000} s)', $spread, $limit)
}
else {
    # Cycles, and not CPU time: Windows charges CPU time in clock ticks of
    # 15.6 ms, which is too coarse for the small amounts of a terminal.
    $spread = $terminal.mcycles_per_s_max - $terminal.mcycles_per_s_min
    $limit = [math]::Max(0.1 * $terminal.mcycles_per_s, 1)
    $what = [string]::Format($invariant, 'the cycles of the terminal varied {0:0.00} million per second between windows (limit {1:0.00})',
        $spread, $limit)
}
if ($windows.Count -lt 2) {
    Write-Host 'With a single window there is no verdict on stability.'
}
elseif ($spread -le $limit) {
    Write-Host "Stable: $what." -ForegroundColor Green
}
else {
    Write-Host "Not stable: $what. Measure again with the machine quieter, or with longer windows." -ForegroundColor Yellow
}

# The CSV uses the invariant culture, as in measure.ps1.
$file = Join-Path $OutDir ('scenario-{0}-{1}.csv' -f $started.ToString('yyyyMMdd-HHmmss', $invariant), $Label)
$summary | ForEach-Object {
    $text = [ordered]@{}
    foreach ($property in $_.PSObject.Properties) {
        $value = $property.Value
        $text[$property.Name] = if ($value -is [IFormattable]) { $value.ToString($null, $invariant) } else { $value }
    }
    [pscustomobject]$text
} | Export-Csv -LiteralPath $file -NoTypeInformation -Encoding utf8NoBOM
Write-Host "Saved $file"
