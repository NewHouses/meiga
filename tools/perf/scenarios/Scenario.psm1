# Shared by the PowerShell load scenarios (spinner.ps1, ai-stream.ps1 and
# bigfile.ps1): console set-up, the files that coordinate a scenario with
# run-scenario.ps1, and frame timing. common.sh does the same for the bash
# scenarios.
#
# Under run-scenario.ps1, the run folder (.perf/run/, or the folder in
# $env:MEIGA_PERF_RUN) contains a file named "controller". A scenario then
# creates "started-<scenario>-pwsh-<process id>" when it starts, deletes it
# when it ends, and ends when the file "stop" appears. Without "controller",
# it runs until Ctrl+C.

Set-StrictMode -Version Latest

$utf8 = [Text.UTF8Encoding]::new($false)
$scenario = $null

function Get-RunDirectory {
    if ($env:MEIGA_PERF_RUN) {
        return [IO.Path]::GetFullPath($env:MEIGA_PERF_RUN)
    }
    [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../.perf/run'))
}

function Start-Scenario {
    param(
        [Parameter(Mandatory)]
        [string]$Name,

        # Leave the cursor visible, as a program that only prints text does.
        [switch]$KeepCursor
    )
    $runDirectory = Get-RunDirectory
    $state = [pscustomobject]@{
        RunDirectory  = $runDirectory
        Controlled    = Test-Path -LiteralPath (Join-Path $runDirectory 'controller')
        Marker        = $null
        KeepCursor    = [bool]$KeepCursor
        Encoding      = $null
        Output        = $null
        Clock         = [Diagnostics.Stopwatch]::StartNew()
        NextFrame     = 0.0
        NextStopCheck = 0.0
    }
    $script:scenario = $state
    if (-not [Console]::IsOutputRedirected) {
        # The scenarios write UTF-8, so the console must read UTF-8 (code
        # page 65001). Stop-Scenario puts the previous code page back.
        $state.Encoding = [Console]::OutputEncoding
        [Console]::OutputEncoding = $utf8
    }
    # Writing bytes straight to the standard output skips the text layers of
    # .NET: each call is a single write to the console.
    $state.Output = [Console]::OpenStandardOutput()
    if (-not $KeepCursor) {
        # Hide the cursor while drawing, as command-line tools do.
        Write-Console "`e[?25l"
    }
    if ($state.Controlled) {
        $state.Marker = Join-Path $runDirectory "started-$Name-pwsh-$PID"
        [IO.File]::WriteAllText($state.Marker, '')
    }
}

# Writes -Text, leaves the console as it was and deletes the start marker.
# Call it in a finally block, which also runs after Ctrl+C.
function Stop-Scenario {
    param([string]$Text = '')
    $state = $script:scenario
    if (-not $state) {
        return
    }
    if ($state.Output) {
        $reset = "$Text`e[0m"
        if (-not $state.KeepCursor) {
            $reset += "`e[?25h"
        }
        Write-Console $reset
        $state.Output.Flush()
    }
    if ($state.Encoding) {
        [Console]::OutputEncoding = $state.Encoding
    }
    if ($state.Marker) {
        Remove-Item -LiteralPath $state.Marker -ErrorAction SilentlyContinue
    }
    $script:scenario = $null
}

# Whether the scenario runs under run-scenario.ps1.
function Test-ScenarioControlled {
    $script:scenario.Controlled
}

# Whether run-scenario.ps1 has asked the scenario to stop. It looks for the
# file at most once per second: in WSL, every look crosses to Windows.
function Test-ScenarioStop {
    $state = $script:scenario
    if (-not $state.Controlled) {
        return $false
    }
    $now = $state.Clock.Elapsed.TotalSeconds
    if ($now -lt $state.NextStopCheck) {
        return $false
    }
    $state.NextStopCheck = $now + 1
    Test-Path -LiteralPath (Join-Path $state.RunDirectory 'stop')
}

# Seconds since the scenario started.
function Get-ScenarioTime {
    $script:scenario.Clock.Elapsed.TotalSeconds
}

# Sleeps until the next frame. Frames are due at fixed moments since the start
# of the scenario, so the rate stays exact although every sleep lasts a little
# longer than asked. A frame that comes very late moves the next moments
# forward instead of causing a burst of frames.
function Wait-NextFrame([double]$Interval) {
    $state = $script:scenario
    $now = $state.Clock.Elapsed.TotalSeconds
    $state.NextFrame = [math]::Max($state.NextFrame + $Interval, $now)
    $milliseconds = [int][math]::Ceiling(($state.NextFrame - $now) * 1000)
    if ($milliseconds -gt 0) {
        Start-Sleep -Milliseconds $milliseconds
    }
}

# Writes text to the console as UTF-8, in a single write.
function Write-Console([string]$Text) {
    $bytes = $utf8.GetBytes($Text)
    $script:scenario.Output.Write($bytes, 0, $bytes.Length)
}

# Writes bytes to the console, in a single write.
function Write-ConsoleBytes([byte[]]$Bytes, [int]$Offset, [int]$Count) {
    $script:scenario.Output.Write($Bytes, $Offset, $Count)
}

Export-ModuleMember -Function Get-RunDirectory, Start-Scenario, Stop-Scenario, Test-ScenarioControlled, Test-ScenarioStop,
    Get-ScenarioTime, Wait-NextFrame, Write-Console, Write-ConsoleBytes
