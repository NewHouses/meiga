<#
.SYNOPSIS
Checks the load scenarios, in PowerShell and in bash.

.DESCRIPTION
Runs each load scenario as run-scenario.ps1 does, with a temporary run folder,
and with its output redirected to a file, and checks:

- spinner: it says that it has started, draws about 10 frames per second,
  hides the cursor and shows it again, and when the file stop appears, it
  ends and deletes its start marker.
- ai-stream: at 1000 words per second and without thinking, the text that it
  leaves, replayed through a small line emulator, is exactly the conversation
  in ai-stream.txt followed by its first prompt again, and the status area is
  gone.
- bigfile: with a file of 1 MiB from gen-bigfile.ps1, it prints the file once
  on its own and, under run-scenario.ps1, once for each go-<n> file, byte for
  byte, writing the seconds into finished-<n>.

The bash scenarios need bash 5: on Windows they run in WSL, and elsewhere in
the bash of the system. Without it, the bash checks are skipped with a
message. CI runs the PowerShell checks on Windows and the bash checks on
Linux.

.PARAMETER Shell
Shells whose scenarios are checked: pwsh, bash or both.

.EXAMPLE
pwsh -File .\tools\perf\test-scenarios.ps1
#>
[CmdletBinding()]
param(
    [ValidateSet('pwsh', 'bash')]
    [string[]]$Shell = @('pwsh', 'bash')
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

$utf8 = [Text.UTF8Encoding]::new($false)
$invariant = [cultureinfo]::InvariantCulture
$scenarios = Join-Path $PSScriptRoot 'scenarios'
$pwsh = (Get-Process -Id $PID).Path
$work = Join-Path ([IO.Path]::GetTempPath()) "meiga-test-scenarios-$PID"
$checks = [Collections.Generic.List[object]]::new()

function Add-Check([string]$Check, [string]$Expected, $Value, [bool]$Passed) {
    $checks.Add([pscustomobject]@{
            Check    = $Check
            Expected = $Expected
            Measured = $Value
            Result   = if ($Passed) { 'ok' } else { 'FAILED' }
        })
}

# Finds bash 5: in WSL on Windows, and in the path elsewhere. Returns an empty
# string when it is there, and why not when it is not.
function Find-Bash {
    if ($IsWindows) {
        if (-not (Get-Command wsl.exe -ErrorAction Ignore)) {
            return 'WSL is not installed'
        }
        # Without this, wsl.exe writes its own messages in UTF-16.
        $env:WSL_UTF8 = '1'
        $distributions = & wsl.exe --list --quiet 2>$null
        if ($LASTEXITCODE -ne 0 -or -not ($distributions -join '').Trim()) {
            return 'WSL has no Linux distribution'
        }
        $version = & wsl.exe -e bash -c 'echo ${BASH_VERSINFO[0]}' 2>$null
    }
    else {
        if (-not (Get-Command bash -ErrorAction Ignore)) {
            return 'bash is not installed'
        }
        $version = & bash -c 'echo ${BASH_VERSINFO[0]}' 2>$null
    }
    $major = 0
    if (-not [int]::TryParse("$version".Trim(), [ref]$major) -or $major -lt 5) {
        return "bash is older than 5 ($version)"
    }
    ''
}

# A path as the bash of the scenarios sees it.
function ConvertTo-BashPath([string]$Path) {
    if (-not $IsWindows) {
        return $Path
    }
    "$(& wsl.exe -e wslpath -a $Path)".Trim()
}

# Start-Process joins its arguments with spaces, so an argument with spaces
# needs quotes.
function ConvertTo-Argument([string]$Value) {
    if ($Value -notmatch '[\s"]') {
        return $Value
    }
    '"' + ($Value -replace '"', '\"') + '"'
}

# A new run folder; with -Controlled, as run-scenario.ps1 leaves it.
function New-RunDirectory([string]$Name, [switch]$Controlled) {
    $directory = Join-Path $work $Name
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    if ($Controlled) {
        New-Item -ItemType File -Path (Join-Path $directory 'controller') | Out-Null
    }
    $directory
}

# Starts a scenario with its output redirected to the file "output" of its run
# folder.
function Start-ScenarioProcess([string]$ShellName, [string]$Name, [string[]]$Arguments, [string]$RunDirectory) {
    $env:MEIGA_PERF_RUN = $RunDirectory
    if ($ShellName -eq 'pwsh') {
        $file = $pwsh
        $list = @('-NoProfile', '-NonInteractive', '-File', (Join-Path $scenarios "$Name.ps1")) + $Arguments
    }
    elseif ($IsWindows) {
        $file = 'wsl.exe'
        $list = @('-e', 'bash', (ConvertTo-BashPath (Join-Path $scenarios "$Name.sh"))) + $Arguments
    }
    else {
        $file = 'bash'
        $list = @((Join-Path $scenarios "$Name.sh")) + $Arguments
    }
    $output = Join-Path $RunDirectory 'output'
    $process = Start-Process $file -ArgumentList ($list | ForEach-Object { ConvertTo-Argument $_ }) -NoNewWindow -PassThru `
        -RedirectStandardOutput $output -RedirectStandardError "$output.err"
    # Without the handle, ExitCode is empty once the process has ended.
    $null = $process.Handle
    [pscustomobject]@{
        Process = $process
        Output  = $output
    }
}

function Wait-Condition([scriptblock]$Condition, [double]$Seconds) {
    $clock = [Diagnostics.Stopwatch]::StartNew()
    while (-not (& $Condition)) {
        if ($clock.Elapsed.TotalSeconds -ge $Seconds) {
            return $false
        }
        Start-Sleep -Milliseconds 50
    }
    $true
}

function Get-Marker([string]$RunDirectory) {
    Get-ChildItem -LiteralPath $RunDirectory -Filter 'started-*'
}

# Waits for the end of a scenario after the file stop, and checks how it
# ended. Returns the output.
function Complete-Scenario($Scenario, [string]$Label, [string]$RunDirectory, [double]$Seconds = 5) {
    $exited = $Scenario.Process.WaitForExit([int]($Seconds * 1000))
    if (-not $exited) {
        $Scenario.Process.Kill()
        $Scenario.Process.WaitForExit()
    }
    Add-Check "${Label}: ends" "in $Seconds s, with code 0" $(if ($exited) { $Scenario.Process.ExitCode } else { 'still running' }) `
        ($exited -and $Scenario.Process.ExitCode -eq 0)
    Add-Check "${Label}: start marker deleted" '0' @(Get-Marker $RunDirectory).Count (@(Get-Marker $RunDirectory).Count -eq 0)
    # WSL may add its own notices.
    $errors = @(Get-Content -LiteralPath "$($Scenario.Output).err" | Where-Object { $_ -notmatch '^wsl: ' })
    Add-Check "${Label}: no errors" 'empty' ($errors -join ' ') ($errors.Count -eq 0)
    [IO.File]::ReadAllText($Scenario.Output, $utf8)
}

# Replays text through a minimal terminal that knows only what the scenarios
# use: text, carriage return, line feed as a new line, cursor up (ESC [ n A)
# and erase to the end of the screen or of the line (ESC [ J, ESC [ K).
# Colours and other sequences are ignored, and every character takes one cell.
# Returns the lines of the screen and of the scrollback.
function Invoke-Screen([string]$Text) {
    $lines = [Collections.Generic.List[Text.StringBuilder]]::new()
    $lines.Add([Text.StringBuilder]::new())
    $row = 0
    $column = 0
    foreach ($match in [regex]::Matches($Text, "`e\[[?0-9;]*[A-Za-z]|`r|`n|[^`e`r`n]+")) {
        $value = $match.Value
        $line = $lines[$row]
        if ($value -eq "`r") {
            $column = 0
        }
        elseif ($value -eq "`n") {
            $row++
            $column = 0
            if ($row -eq $lines.Count) {
                $lines.Add([Text.StringBuilder]::new())
            }
        }
        elseif ($value[0] -eq [char]27) {
            $parameter = $value.Substring(2, $value.Length - 3)
            switch -CaseSensitive ($value[-1]) {
                'A' {
                    $row = [math]::Max(0, $row - $(if ($parameter) { [int]$parameter } else { 1 }))
                }
                'J' {
                    $line.Length = [math]::Min($column, $line.Length)
                    $lines.RemoveRange($row + 1, $lines.Count - $row - 1)
                }
                'K' {
                    $line.Length = [math]::Min($column, $line.Length)
                }
            }
        }
        else {
            if ($line.Length -lt $column) {
                [void]$line.Append(' ', $column - $line.Length)
            }
            [void]$line.Remove($column, [math]::Min($value.Length, $line.Length - $column))
            [void]$line.Insert($column, $value)
            $column += $value.Length
        }
    }
    $lines | ForEach-Object { $_.ToString().TrimEnd() }
}

# The text that ai-stream leaves on the screen for one loop of the
# conversation: each prompt, a blank line, the answer without the blank lines
# around it, and a blank line.
function Get-Conversation {
    $exchanges = [Collections.Generic.List[object]]::new()
    foreach ($line in [IO.File]::ReadAllLines((Join-Path $scenarios 'ai-stream.txt'))) {
        $line = ($line -replace '\\e\[[0-9;]*m', '').TrimEnd()
        if ($line.StartsWith('> ')) {
            $exchanges.Add([pscustomobject]@{
                    Prompt = $line.Substring(2)
                    Lines  = [Collections.Generic.List[string]]::new()
                })
        }
        elseif ($exchanges.Count -gt 0) {
            $exchanges[-1].Lines.Add($line)
        }
    }
    foreach ($exchange in $exchanges) {
        $lines = $exchange.Lines
        while ($lines.Count -gt 0 -and $lines[0] -eq '') {
            $lines.RemoveAt(0)
        }
        while ($lines.Count -gt 0 -and $lines[-1] -eq '') {
            $lines.RemoveAt($lines.Count - 1)
        }
        "`u{276F} $($exchange.Prompt)"
        ''
        $lines
        ''
    }
}

function Test-Spinner([string]$ShellName) {
    $label = "spinner ($ShellName)"
    $run = New-RunDirectory "spinner-$ShellName" -Controlled
    $scenario = Start-ScenarioProcess $ShellName spinner @() $run
    try {
        $started = Wait-Condition { @(Get-Marker $run).Count -gt 0 } 30
        Add-Check "${label}: start marker" 'created' $started $started
        if (-not $started) {
            return
        }
        $clock = [Diagnostics.Stopwatch]::StartNew()
        Start-Sleep -Seconds 3
        New-Item -ItemType File -Path (Join-Path $run 'stop') | Out-Null
        $output = Complete-Scenario $scenario $label $run
        $seconds = $clock.Elapsed.TotalSeconds
        $rate = [math]::Round(([regex]::Matches($output, "`r")).Count / $seconds, 1)
        Add-Check "${label}: frames per second" '7-12' $rate ($rate -ge 7 -and $rate -le 12)
        Add-Check "${label}: cursor" 'hidden, then shown' $output.Length `
            ($output.StartsWith("`e[?25l") -and $output.EndsWith("`e[?25h"))
        Add-Check "${label}: UTF-8 spinner" "`u{280B} Thinking" $output.Length `
            ($output.Contains("`u{280B}`e[0m `e[1mThinking"))
    }
    finally {
        if (-not $scenario.Process.HasExited) {
            $scenario.Process.Kill()
        }
    }
}

function Test-AiStream([string]$ShellName) {
    $label = "ai-stream ($ShellName)"
    $run = New-RunDirectory "ai-stream-$ShellName" -Controlled
    $arguments = if ($ShellName -eq 'pwsh') { '-WordsPerSecond', '1000', '-ThinkSeconds', '0' } else { '1000', '0' }
    $scenario = Start-ScenarioProcess $ShellName ai-stream $arguments $run
    try {
        $started = Wait-Condition { @(Get-Marker $run).Count -gt 0 } 30
        Add-Check "${label}: start marker" 'created' $started $started
        if (-not $started) {
            return
        }
        # One loop takes about 3 s at this speed.
        Start-Sleep -Seconds 5
        New-Item -ItemType File -Path (Join-Path $run 'stop') | Out-Null
        $output = Complete-Scenario $scenario $label $run
        $screen = @(Invoke-Screen $output)
        $expected = @(Get-Conversation)
        $difference = -1
        for ($index = 0; $index -lt $expected.Count; $index++) {
            if ($index -ge $screen.Count -or $screen[$index] -cne $expected[$index]) {
                $difference = $index
                break
            }
        }
        $measured = if ($difference -lt 0) { "$($expected.Count) lines" } else { "line $($difference + 1) differs" }
        Add-Check "${label}: conversation" "$($expected.Count) lines" $measured ($difference -lt 0)
        $again = $screen.Count -gt $expected.Count -and $screen[$expected.Count] -ceq $expected[0]
        Add-Check "${label}: starts again" 'first prompt' $again $again
        $status = @($screen | Where-Object { $_.Contains('ctrl+c to stop') }).Count
        Add-Check "${label}: status area erased" '0 lines' $status ($status -eq 0)
        Add-Check "${label}: cursor" 'hidden, then shown' $output.Length `
            ($output.StartsWith("`e[?25l") -and $output.EndsWith("`e[?25h"))
    }
    finally {
        if (-not $scenario.Process.HasExited) {
            $scenario.Process.Kill()
        }
    }
}

function Test-BigFile([string]$ShellName, [string]$File, [string]$Text) {
    $label = "bigfile ($ShellName)"
    $path = if ($ShellName -eq 'pwsh') { '-Path', $File } else { @(ConvertTo-BashPath $File) }
    $megabytes = ([IO.FileInfo]$File).Length / 1MB

    # On its own: it prints the file once, says how long it took, and ends.
    $run = New-RunDirectory "bigfile-$ShellName-alone"
    $scenario = Start-ScenarioProcess $ShellName bigfile $path $run
    try {
        $output = Complete-Scenario $scenario "$label on its own" $run 30
        $expected = [string]::Format($invariant, 'Printed {0:0.0} MB in ', $megabytes)
        $passed = $output.StartsWith($Text, 'Ordinal') -and $output.Length -gt $Text.Length -and
            $output.Substring($Text.Length) -match "^$([regex]::Escape($expected))\d+\.\d{3} s\.`n`e\[0m\z"
        Add-Check "${label} on its own: output" 'the file and the time' $output.Substring([math]::Min($Text.Length, $output.Length)).Trim() $passed
    }
    finally {
        if (-not $scenario.Process.HasExited) {
            $scenario.Process.Kill()
        }
    }

    # Under run-scenario.ps1: once per go-<n>, until stop.
    $run = New-RunDirectory "bigfile-$ShellName" -Controlled
    $scenario = Start-ScenarioProcess $ShellName bigfile $path $run
    try {
        $started = Wait-Condition { @(Get-Marker $run).Count -gt 0 } 30
        Add-Check "${label}: start marker" 'created' $started $started
        if (-not $started) {
            return
        }
        $expected = [Text.StringBuilder]::new()
        foreach ($number in 1, 2) {
            New-Item -ItemType File -Path (Join-Path $run "go-$number") | Out-Null
            $finished = Join-Path $run "finished-$number"
            $done = Wait-Condition { Test-Path -LiteralPath $finished } 30
            $seconds = if ($done) { (Get-Content -LiteralPath $finished -Raw).Trim() } else { 'missing' }
            $value = 0.0
            $valid = [double]::TryParse($seconds, [Globalization.NumberStyles]::Float, $invariant, [ref]$value) -and $value -gt 0
            Add-Check "${label}: finished-$number" 'seconds' $seconds $valid
            [void]$expected.Append($Text).Append([string]::Format($invariant, "Run {0}: printed {1:0.0} MB in {2} s.`n", $number, $megabytes, $seconds))
        }
        New-Item -ItemType File -Path (Join-Path $run 'stop') | Out-Null
        $output = Complete-Scenario $scenario $label $run
        [void]$expected.Append("`e[0m")
        Add-Check "${label}: output" 'the file twice, byte for byte' $output.Length ($output -ceq $expected.ToString())
    }
    finally {
        if (-not $scenario.Process.HasExited) {
            $scenario.Process.Kill()
        }
    }
}

$shells = [Collections.Generic.List[string]]::new()
if ($Shell -contains 'pwsh') {
    $shells.Add('pwsh')
}
if ($Shell -contains 'bash') {
    $missing = Find-Bash
    if ($missing) {
        Write-Warning "Skipping the bash scenarios: $missing."
    }
    else {
        $shells.Add('bash')
        # WSL takes MEIGA_PERF_RUN from Windows, translating the path.
        $env:WSLENV = (@($env:WSLENV, 'MEIGA_PERF_RUN/p') | Where-Object { $_ }) -join ':'
    }
}

New-Item -ItemType Directory -Force -Path $work | Out-Null
try {
    $bigfile = Join-Path $work 'bigfile.txt'
    & $pwsh -NoProfile -File (Join-Path $PSScriptRoot 'gen-bigfile.ps1') -Path $bigfile -SizeMB 1 | Out-Null
    $bigText = [IO.File]::ReadAllText($bigfile, $utf8)
    foreach ($name in $shells) {
        Write-Host "Checking the $name scenarios..."
        Test-Spinner $name
        Test-AiStream $name
        Test-BigFile $name $bigfile $bigText
    }
}
finally {
    Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}

$checks | Format-Table -AutoSize | Out-Host
$failed = @($checks | Where-Object Result -NE 'ok').Count
if ($checks.Count -eq 0) {
    throw 'No scenario was checked.'
}
if ($failed -gt 0) {
    throw "$failed of $($checks.Count) checks failed."
}
Write-Host "All $($checks.Count) checks passed."
