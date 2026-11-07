<#
.SYNOPSIS
Measures the CPU, memory, threads and wakeups of a terminal and of the parts
of Windows that put it on the screen.

.DESCRIPTION
Takes a snapshot of every process and thread at the start of the window, every
-SampleInterval seconds and at the end, and reports these groups:

- terminal:    the terminal: the -Name or -Id processes and their children with
               the same executable name.
- console:     the console hosts (OpenConsole.exe, conhost.exe) started by the
               terminal, usually one per tab.
- shells:      everything else started by the terminal: the shells and the
               programs they run. It is the workload, not the cost of the
               terminal.
- dwm:         the Desktop Window Manager of this session, which composes the
               windows on the screen.
- termservice: the Remote Desktop service.
- wudfhost:    the user-mode driver hosts, which include the display driver of
               Remote Desktop.
- harness:     this script, to show what the measurement itself costs.
- system:      the whole machine, without this script.

CPU is a percentage of one core: 100 means one core busy all the time, and 200
two cores. Context switches count how many times the threads were put on a
processor, that is, how many times they woke up.

Nothing is printed during the window, so that the measurement does not make
any terminal draw. The window starts 2 s after the message that announces it,
when the start-up work of the script has finished. The results are printed at
the end and saved as CSV in .perf\, which git ignores because they describe
this machine. Numbers in the CSV always use a dot as the decimal separator.

.PARAMETER Name
Executable name of the terminal, with or without .exe: warp, WindowsTerminal,
meiga...

.PARAMETER Id
Process ids of the terminal, instead of -Name.

.PARAMETER Duration
Length of the measurement window, in seconds.

.PARAMETER SampleInterval
Seconds between snapshots inside the window. Memory and threads are averaged
over the snapshots, and a process that exits loses the CPU that it used after
the last snapshot in which it appeared.

.PARAMETER Delay
Seconds to wait before starting, for example to prepare the scenario.

.PARAMETER Label
Name of the scenario. It goes into the CSV and into its file name.

.PARAMETER OutDir
Folder for the CSV file. Defaults to .perf\ in the repository.

.PARAMETER StartedFile
File to create when the window starts, so that another script, such as a
load scenario, can start at the same time.

.PARAMETER StopFile
File that ends the window early when it appears. The script looks for it
every 0.1 s. -Duration is then the longest window.

.PARAMETER Quiet
Do not print the messages and the table. Warnings are still printed.

.PARAMETER PassThru
Also return the rows as objects, for other scripts.

.EXAMPLE
.\tools\perf\measure.ps1 -Name WindowsTerminal -Duration 60 -Label idle-5-tabs
#>
[CmdletBinding(DefaultParameterSetName = 'Name')]
param(
    [Parameter(Mandatory, ParameterSetName = 'Name', Position = 0)]
    [string]$Name,

    [Parameter(Mandatory, ParameterSetName = 'Id')]
    [int[]]$Id,

    [ValidateRange(1, 86400)]
    [int]$Duration = 60,

    [ValidateRange(1, 3600)]
    [int]$SampleInterval = 5,

    [ValidateRange(0, 3600)]
    [int]$Delay = 0,

    [ValidatePattern('^[\w.-]+$')]
    [string]$Label = 'manual',

    [string]$OutDir = (Join-Path $PSScriptRoot '..\..\.perf'),

    [string]$StartedFile,

    [string]$StopFile,

    [switch]$Quiet,

    [switch]$PassThru
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

if (-not [Environment]::Is64BitProcess) {
    throw 'Run this script in 64-bit PowerShell: it reads 64-bit Windows structures.'
}
if (-not ('Meiga.Perf.Tracker' -as [type])) {
    Add-Type -Path (Join-Path $PSScriptRoot 'ProcessSnapshot.cs')
}

$OutDir = [IO.Path]::GetFullPath($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutDir))
if ($StartedFile) {
    $StartedFile = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($StartedFile)
}
if ($StopFile) {
    $StopFile = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($StopFile)
}
$invariant = [cultureinfo]::InvariantCulture
$groupNames = [Meiga.Perf.Tracker]::GroupNames
$cpuCount = [Meiga.Perf.ProcessSnapshot]::ProcessorCount
$termService = Get-CimInstance Win32_Service -Filter "Name = 'TermService'"
$termServiceId = if ($termService) { [int]$termService.ProcessId } else { 0 }

# Find the terminal.
$snapshot = [Meiga.Perf.ProcessSnapshot]::Take()
$self = $snapshot.Processes[$PID]
if ($PSCmdlet.ParameterSetName -eq 'Name') {
    $exe = if ($Name -match '\.exe$') { $Name } else { "$Name.exe" }
    $named = @($snapshot.Processes.Values | Where-Object { $_.Name -eq $exe -and $_.Id -ne $PID })
    # The roots are the processes with that name whose parent is not one of them.
    $roots = @($named | Where-Object {
            $child = $_
            -not ($named | Where-Object { $_.Id -eq $child.ParentId -and $_.CreateTime -le $child.CreateTime })
        })
    $target = $exe
}
else {
    foreach ($missing in @($Id | Where-Object { -not $snapshot.Processes.ContainsKey($_) })) {
        Write-Warning "There is no process with id $missing."
    }
    $roots = @($Id | Where-Object { $_ -ne $PID -and $snapshot.Processes.ContainsKey($_) } |
            ForEach-Object { $snapshot.Processes[$_] })
    $target = "process $($Id -join ', ')"
}
if ($roots.Count -eq 0) {
    throw "No running process matches $target."
}

$tracker = [Meiga.Perf.Tracker]::new($self.SessionId, $termServiceId)
$tracker.AddHarness($self)
foreach ($root in $roots) {
    $tracker.AddTerminal($root)
}

if ($Delay -gt 0) {
    if (-not $Quiet) {
        Write-Host "Starting in $Delay s..."
    }
    Start-Sleep -Seconds $Delay
}
if (-not $Quiet) {
    Write-Host "Measuring $target for $Duration s. Leave this window alone until it finishes."
}

# Start-up work goes on in the background for a moment after it ends: .NET
# optimizes the code that has just run, and that can cost this script more than
# 10 % of a core. Run the code of the window once and wait, so that it does not
# count. The wait also lets the terminal draw the line above.
[Meiga.Perf.Tracker]::new($self.SessionId, $termServiceId).Add($snapshot)
Start-Sleep -Seconds 2

$first = [Meiga.Perf.ProcessSnapshot]::Take()
$run = [datetime]::Now
$tracker.Add($first)
if ($StartedFile) {
    New-Item -ItemType File -Force -Path $StartedFile | Out-Null
}

# Waits until the next snapshot. With -StopFile, it looks for the file every
# 0.1 s and returns $true as soon as it appears.
function Wait-Snapshot([double]$seconds) {
    $deadline = [Diagnostics.Stopwatch]::GetTimestamp() + $seconds * [Diagnostics.Stopwatch]::Frequency
    while ($true) {
        if ($StopFile -and (Test-Path -LiteralPath $StopFile)) {
            return $true
        }
        $left = ($deadline - [Diagnostics.Stopwatch]::GetTimestamp()) / [Diagnostics.Stopwatch]::Frequency
        if ($left -le 0) {
            return $false
        }
        $step = if ($StopFile) { [math]::Min($left, 0.1) } else { $left }
        Start-Sleep -Milliseconds ([int][math]::Ceiling($step * 1000))
    }
}

do {
    $elapsed = ([Diagnostics.Stopwatch]::GetTimestamp() - $first.Timestamp) / [Diagnostics.Stopwatch]::Frequency
    $stopped = Wait-Snapshot ([math]::Min($SampleInterval, $Duration - $elapsed))
    $current = [Meiga.Perf.ProcessSnapshot]::Take()
    $tracker.Add($current)
} while (-not $stopped -and $current.SecondsSince($first) -lt $Duration)
$seconds = $current.SecondsSince($first)

# CPU, cycles and context switches of each process and group.
$usage = @{}
foreach ($group in $groupNames) {
    $usage[$group] = @{ Cpu = 0.0; User = 0.0; Kernel = 0.0; Cycles = 0.0; Switches = 0.0; Processes = 0; Exited = 0 }
}
$processUsage = @{}
foreach ($entry in $tracker.Processes.GetEnumerator()) {
    $start = $entry.Value.Start
    $end = $entry.Value.Last
    $user = $end.UserTime - $(if ($start) { $start.UserTime } else { 0 })
    $kernel = $end.KernelTime - $(if ($start) { $start.KernelTime } else { 0 })
    $one = @{
        Cpu       = [double]($user + $kernel)
        User      = [double]$user
        Kernel    = [double]$kernel
        Cycles    = [double]$end.CycleTime - $(if ($start) { [double]$start.CycleTime } else { 0.0 })
        Switches  = [double]$entry.Value.Switches
        Processes = 1
        Exited    = [int]($entry.Value.LastIndex -lt $tracker.Index)
    }
    $processUsage[$entry.Key] = $one
    $sum = $usage[$entry.Value.Group]
    foreach ($field in 'Cpu', 'User', 'Kernel', 'Cycles', 'Switches', 'Processes', 'Exited') {
        $sum[$field] += $one[$field]
    }
}

# The whole machine was busy all the time that its idle process did not run.
# This script is left out, as in the memory of the group.
$idle = $current.Processes[0].KernelTime - $first.Processes[0].KernelTime
$usage.system = @{
    Cpu       = [math]::Max(0.0, $cpuCount * $seconds * 1e7 - $idle - $usage.harness.Cpu)
    User      = $null
    Kernel    = $null
    Cycles    = $null
    Switches  = [double]($tracker.SystemSwitches - $usage.harness.Switches)
    Processes = $current.Processes.Count - 2
    Exited    = $null
}

function Get-Percent($time) {
    if ($null -eq $time) { return $null }
    [math]::Round($time / 1e5 / $seconds, 2)
}

function Get-Rate($count, [double]$unit = 1) {
    if ($null -eq $count) { return $null }
    [math]::Round($count / $unit / $seconds, 2)
}

$runText = $run.ToString('yyyy-MM-ddTHH:mm:ss', $invariant)
$remote = [Meiga.Perf.ProcessSnapshot]::IsRemoteSession

function New-Row([string]$kind, [string]$group, $processId, [string]$processName, $use, $memory) {
    $samples = if ($memory) { $memory.Samples } else { 0 }
    [pscustomobject][ordered]@{
        run                = $runText
        label              = $Label
        remote             = $remote
        cpus               = $cpuCount
        kind               = $kind
        group              = $group
        pid                = $processId
        name               = $processName
        processes          = $use.Processes
        exited             = $use.Exited
        seconds            = [math]::Round($seconds, 2)
        cpu_s              = [math]::Round($use.Cpu / 1e7, 3)
        cpu_pct            = Get-Percent $use.Cpu
        user_pct           = Get-Percent $use.User
        kernel_pct         = Get-Percent $use.Kernel
        mcycles_per_s      = Get-Rate $use.Cycles 1e6
        ctx_switches       = $use.Switches
        ctx_per_s          = Get-Rate $use.Switches
        private_mb_avg     = if ($samples) { [math]::Round($memory.PrivateSum / $samples / 1MB, 1) } else { $null }
        private_mb_max     = if ($samples) { [math]::Round($memory.PrivateMax / 1MB, 1) } else { $null }
        working_set_mb_avg = if ($samples) { [math]::Round($memory.WorkingSetSum / $samples / 1MB, 1) } else { $null }
        threads_avg        = if ($samples) { [math]::Round($memory.ThreadsSum / $samples, 1) } else { $null }
        threads_max        = if ($samples) { $memory.ThreadsMax } else { $null }
        samples            = $samples
    }
}

$rows = [System.Collections.Generic.List[object]]::new()
foreach ($group in $groupNames) {
    $memory = if ($tracker.GroupMemory.ContainsKey($group)) { $tracker.GroupMemory[$group] } else { $null }
    $rows.Add((New-Row 'group' $group $null $null $usage[$group] $memory))
}
$ordered = $tracker.Processes.GetEnumerator() |
    Sort-Object { $groupNames.IndexOf($_.Value.Group) }, { $_.Value.Last.Name }, { $_.Value.Last.Id }
foreach ($entry in $ordered) {
    $p = $entry.Value.Last
    $rows.Add((New-Row 'process' $entry.Value.Group $p.Id $p.Name $processUsage[$entry.Key] $entry.Value.Memory))
}

# The CSV uses the invariant culture: with the Galician or Spanish culture,
# Export-Csv would write 1,5 instead of 1.5.
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$file = Join-Path $OutDir ('measure-{0}-{1}.csv' -f $run.ToString('yyyyMMdd-HHmmss', $invariant), $Label)
$rows | ForEach-Object {
    $text = [ordered]@{}
    foreach ($property in $_.PSObject.Properties) {
        $value = $property.Value
        $text[$property.Name] = if ($value -is [IFormattable]) { $value.ToString($null, $invariant) } else { $value }
    }
    [pscustomobject]$text
} | Export-Csv -LiteralPath $file -NoTypeInformation -Encoding utf8NoBOM

if (-not $Quiet) {
    $rows | Where-Object kind -EQ 'group' |
        Format-Table group, processes, cpu_pct, mcycles_per_s, ctx_per_s, private_mb_avg, private_mb_max, threads_max -AutoSize |
        Out-Host
}
foreach ($group in 'terminal', 'console') {
    if ($usage[$group].Exited -gt 0) {
        Write-Warning "$($usage[$group].Exited) $group processes exited during the window: the CPU that they used after the last snapshot is not counted."
    }
}
if (-not $Quiet) {
    Write-Host "CPU in % of one core over $([math]::Round($seconds, 1)) s; Task Manager shows it divided by $cpuCount."
    Write-Host "Saved $file"
}

if ($PassThru) {
    $rows
}
