<#
.SYNOPSIS
Load scenario: prints a big file as fast as the terminal takes it.

.DESCRIPTION
Imitates printing a long log or the output of a big build, the heaviest case
for a terminal. It reads the whole file into memory first and then writes it
to the console in pieces of 64 KiB, without the formatting of PowerShell, so
that the time is that of the terminal, and not that of the disk or of
PowerShell. The console makes the script wait while the terminal is behind,
so the time includes reading and drawing everything but the last few pieces.

On its own, it prints the file once and says how long it took. Under
run-scenario.ps1, it prints the file once per measurement window: it waits
until the file go-<n> appears in the run folder, prints the file, waits 1 s
more so that the terminal finishes drawing, and writes the seconds that
printing took into finished-<n>, which ends the window. It ends when the file
stop appears.

gen-bigfile.ps1 creates the file. bigfile.sh is the same scenario for bash.

.PARAMETER Path
File to print. Defaults to .perf\bigfile.txt, which gen-bigfile.ps1 creates.

.EXAMPLE
pwsh -NoProfile -File .\tools\perf\scenarios\bigfile.ps1
#>
[CmdletBinding()]
param(
    [string]$Path
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
Import-Module (Join-Path $PSScriptRoot 'Scenario.psm1') -Force

# Not in param(): there, Windows PowerShell 5.1 leaves $PSScriptRoot empty and
# fails before the check above.
if (-not $Path) {
    $Path = Join-Path $PSScriptRoot '../../../.perf/bigfile.txt'
}
$Path = [IO.Path]::GetFullPath($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path))
if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    $generator = Resolve-Path -Relative (Join-Path $PSScriptRoot '../gen-bigfile.ps1')
    throw "$Path does not exist. Create it with: pwsh -File $generator"
}
$bytes = [IO.File]::ReadAllBytes($Path)
$invariant = [cultureinfo]::InvariantCulture
$megabytes = $bytes.Length / 1MB

# Prints the whole file and returns the seconds that it took.
function Write-BigFile {
    $clock = [Diagnostics.Stopwatch]::StartNew()
    for ($offset = 0; $offset -lt $bytes.Length; $offset += 64KB) {
        Write-ConsoleBytes $bytes $offset ([math]::Min(64KB, $bytes.Length - $offset))
    }
    $clock.Elapsed.TotalSeconds
}

Start-Scenario bigfile -KeepCursor
try {
    if (-not (Test-ScenarioControlled)) {
        $seconds = Write-BigFile
        Write-Console ([string]::Format($invariant, "Printed {0:0.0} MB in {1:0.000} s.`n", $megabytes, $seconds))
        return
    }
    $runDirectory = Get-RunDirectory
    $stop = Join-Path $runDirectory 'stop'
    for ($run = 1; ; $run++) {
        $go = Join-Path $runDirectory "go-$run"
        while (-not (Test-Path -LiteralPath $go)) {
            if (Test-Path -LiteralPath $stop) {
                return
            }
            Start-Sleep -Milliseconds 100
        }
        $seconds = Write-BigFile
        # Let the terminal draw what it still has before the window ends.
        Start-Sleep -Seconds 1
        [IO.File]::WriteAllText((Join-Path $runDirectory "finished-$run"), $seconds.ToString('0.000', $invariant))
        Write-Console ([string]::Format($invariant, "Run {0}: printed {1:0.0} MB in {2:0.000} s.`n", $run, $megabytes, $seconds))
    }
}
finally {
    Stop-Scenario
}
