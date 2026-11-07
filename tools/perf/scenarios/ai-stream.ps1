<#
.SYNOPSIS
Load scenario: imitates an AI assistant that answers in a terminal.

.DESCRIPTION
Plays the conversation in ai-stream.txt in a loop, the way that command-line
AI tools draw it. For each prompt, it shows the prompt, "thinks" for 2 s and
then writes the answer at about 40 words per second.

Below the text there is a status area of 3 lines: a spinner with the time and
the number of words, a rule and a hint. The screen changes 10 times per
second. Each frame goes back to the start of the last line of text, erases
from there to the end of the screen, writes that line again with the new
words and draws the status area again. The words of each frame are those due
at that moment, so a frame that comes late catches up.

ai-stream.txt holds the answers as an AI tool shows them, with colours: lines
that start with "> " are prompts, the rest is the answer to the prompt above,
and \e stands for the escape character. The spaces between words are kept, so
tables stay aligned. Its lines fit in 80 columns, because a line that wraps
would break the redraw.

It runs until Ctrl+C or, under run-scenario.ps1, until the measurement ends.
ai-stream.sh is the same scenario for bash, and it writes the same bytes.

.PARAMETER WordsPerSecond
Speed of the answers.

.PARAMETER ThinkSeconds
Seconds of "thinking" before each answer.

.EXAMPLE
pwsh -NoProfile -File .\tools\perf\scenarios\ai-stream.ps1
#>
[CmdletBinding()]
param(
    [ValidateRange(1, 100000)]
    [int]$WordsPerSecond = 40,

    [ValidateRange(0, 60)]
    [int]$ThinkSeconds = 2
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

# Reads the conversation. Each answer is a list of tokens: a word with what
# goes before it, that is, the spaces before it and, at the start of a line,
# the line breaks. Blank lines at the start and at the end of an answer are
# left out, because the scenario draws its own.
function Read-Conversation([string]$Path) {
    $exchanges = [Collections.Generic.List[object]]::new()
    $current = $null
    $breaks = 0
    foreach ($line in [IO.File]::ReadAllLines($Path)) {
        $line = $line.Replace('\e', "`e")
        if ($line.StartsWith('> ')) {
            $current = [pscustomobject]@{
                Prompt = $line.Substring(2)
                Tokens = [Collections.Generic.List[string]]::new()
            }
            $exchanges.Add($current)
            $breaks = 0
            continue
        }
        if (-not $current) {
            continue
        }
        $words = [regex]::Matches($line, ' *[^ ]+')
        if ($words.Count -eq 0) {
            $breaks++
            continue
        }
        $prefix = if ($current.Tokens.Count -eq 0) { '' } else { "`n" * ($breaks + 1) }
        $breaks = 0
        foreach ($word in $words) {
            $current.Tokens.Add($prefix + $word.Value)
            $prefix = ''
        }
    }
    if ($exchanges.Count -eq 0) {
        throw "$Path has no prompts, that is, lines that start with '> '."
    }
    $exchanges
}

$exchanges = @(Read-Conversation (Join-Path $PSScriptRoot 'ai-stream.txt'))
# The scripts are kept in ASCII, so characters outside it go as escapes: the
# braille glyphs of the spinner, the arrow of the prompt, an ellipsis, a middle
# dot and a box drawing line.
$glyphs = "`u{280B}`u{2819}`u{2839}`u{2838}`u{283C}`u{2834}`u{2826}`u{2827}`u{2807}`u{280F}"
$arrow = "`u{276F}"
$ellipsis = "`u{2026}"
$dot = "`u{00B7}"
$rule = "`u{2500}" * 60
$hint = "`e[2mctrl+c to stop $dot simulated AI answer`e[0m"

# prompt: show the next prompt; thinking: before the answer; streaming: while
# writing it; answered: the answer has finished.
$phase = 'prompt'
$exchange = 0
$exchangeStart = 0.0
$emitted = 0
# The last line of text, which every frame writes again.
$partial = ''
$endText = ''

Start-Scenario ai-stream
try {
    for ($frame = 0; -not (Test-ScenarioStop); $frame++) {
        $now = Get-ScenarioTime
        $out = [Text.StringBuilder]::new()
        if ($frame -gt 0) {
            # Back to the start of the last line of text, 3 lines above the
            # end of the status area, and erase down to the end of the screen.
            [void]$out.Append("`r`e[3A`e[J")
        }
        [void]$out.Append($partial)
        if ($phase -eq 'answered') {
            $exchange = ($exchange + 1) % $exchanges.Count
            $phase = 'prompt'
        }
        $tokens = $exchanges[$exchange].Tokens
        if ($phase -eq 'prompt') {
            [void]$out.Append("`e[1;36m$arrow`e[0m `e[1m$($exchanges[$exchange].Prompt)`e[0m`n`n")
            $phase = 'thinking'
            $exchangeStart = $now
            $emitted = 0
        }
        elseif ($now -ge $exchangeStart + $ThinkSeconds) {
            $phase = 'streaming'
            $due = [math]::Min($tokens.Count, [math]::Floor(($now - $exchangeStart - $ThinkSeconds) * $WordsPerSecond))
            for (; $emitted -lt $due; $emitted++) {
                $token = $tokens[$emitted]
                [void]$out.Append($token)
                $newline = $token.LastIndexOf("`n")
                $partial = if ($newline -ge 0) { $token.Substring($newline + 1) } else { $partial + $token }
            }
            if ($emitted -eq $tokens.Count) {
                # End the last line and leave a blank line before the next
                # prompt.
                [void]$out.Append("`n`n")
                $partial = ''
                $phase = 'answered'
            }
        }
        $seconds = [math]::Floor($now - $exchangeStart)
        $state = if ($phase -eq 'thinking') {
            "Thinking$ellipsis`e[0m `e[2m($seconds s)"
        }
        else {
            "Generating$ellipsis`e[0m `e[2m($seconds s $dot $emitted words)"
        }
        $spinner = "`e[38;5;$(16 + $frame % 216)m$($glyphs[$frame % 10])`e[0m"
        [void]$out.Append("`e[0m`n$spinner `e[1m$state`e[0m`n`e[2m$rule`e[0m`n$hint")
        Write-Console $out.ToString()
        $endText = "`r`e[3A`e[J$partial`e[0m`n"
        Wait-NextFrame 0.1
    }
}
finally {
    # Erase the status area and leave the cursor below the text.
    Stop-Scenario $endText
}
