<#
.SYNOPSIS
Load scenario: a spinner with colours that redraws one line 10 times per
second.

.DESCRIPTION
Imitates the progress indicator of command-line tools, such as an AI agent
that is thinking. Each frame rewrites the same line: it goes back to the start
of the line, draws a braille glyph in a colour that changes in every frame, a
word in bold and the seconds in dim text, and erases the rest of the line.

It runs until Ctrl+C or, under run-scenario.ps1, until the measurement ends.
spinner.sh is the same scenario for bash.

.PARAMETER Hz
Frames per second.

.EXAMPLE
pwsh -NoProfile -File .\tools\perf\scenarios\spinner.ps1
#>
[CmdletBinding()]
param(
    [ValidateRange(1, 100)]
    [int]$Hz = 10
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

# The ten braille glyphs of the usual spinner, from U+280B to U+280F. The
# scripts are kept in ASCII, so characters outside it go as escapes.
$glyphs = "`u{280B}`u{2819}`u{2839}`u{2838}`u{283C}`u{2834}`u{2826}`u{2827}`u{2807}`u{280F}"

Start-Scenario spinner
try {
    for ($frame = 0; -not (Test-ScenarioStop); $frame++) {
        # 16 to 231 are the 216 colours of the 6x6x6 cube of the 256-colour
        # palette.
        $color = 16 + $frame % 216
        $seconds = [math]::Floor((Get-ScenarioTime))
        Write-Console "`r`e[38;5;${color}m$($glyphs[$frame % 10])`e[0m `e[1mThinking`e[0m `e[2m($seconds s)`e[0m`e[K"
        Wait-NextFrame (1 / $Hz)
    }
}
finally {
    Stop-Scenario "`n"
}
