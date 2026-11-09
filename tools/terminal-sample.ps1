<#
.SYNOPSIS
Writes a sample of what a terminal shows, to check it by eye.

.DESCRIPTION
Writes the 16 colours of the palette, as text and as background; the 256
colours; a truecolor gradient; the styles (bold, italic, dim, inverse,
hidden, strikeout and five kinds of underline); wide characters between
brackets, which must fall on their two cells; accented letters and box
drawing. It also sets the title of the window to "meiga sample".

Run it in the terminal that you check or, while meiga has no keyboard, as
the program of meiga:

    .\target\release\meiga.exe pwsh -NoProfile -File tools\terminal-sample.ps1

.PARAMETER Wait
Seconds to wait at the end, before the script ends.
#>
[CmdletBinding()]
param(
    [ValidateRange(0, 86400)]
    [int]$Wait = 0
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

$e = [char]27
$sample = [Text.StringBuilder]::new()
function Add-Text([string]$Text) {
    $null = $sample.Append($Text)
}

Add-Text "$e]0;meiga sample$e\"

Add-Text "Text colours (30-37 and 90-97):`r`n"
foreach ($first in 30, 90) {
    foreach ($code in $first..($first + 7)) {
        Add-Text "$e[${code}m $code $e[0m"
    }
    Add-Text "`r`n"
}
Add-Text "Background colours (40-47 and 100-107):`r`n"
foreach ($first in 40, 100) {
    foreach ($code in $first..($first + 7)) {
        Add-Text "$e[${code}m $code $e[0m"
    }
    Add-Text "`r`n"
}

Add-Text "256 colours: the 6 x 6 x 6 cube and the 24 greys:`r`n"
foreach ($row in 0..5) {
    foreach ($column in 0..35) {
        Add-Text "$e[48;5;$(16 + 36 * $row + $column)m  "
    }
    Add-Text "$e[0m`r`n"
}
foreach ($grey in 232..255) {
    Add-Text "$e[48;5;${grey}m   "
}
Add-Text "$e[0m`r`n"

Add-Text "Truecolor:`r`n"
foreach ($step in 0..71) {
    $red = [int](255 * $step / 71)
    Add-Text "$e[48;2;$red;80;$(255 - $red)m "
}
Add-Text "$e[0m`r`n"

Add-Text "Styles: $e[1mbold$e[0m $e[3mitalic$e[0m $e[1;3mbold italic$e[0m $e[2mdim$e[0m $e[7minverse$e[0m [$e[8mhidden$e[0m] $e[9mstrikeout$e[0m`r`n"
Add-Text "Underlines: $e[4msingle$e[0m $e[4:2mdouble$e[0m $e[4:3mcurly$e[0m $e[4:4mdotted$e[0m $e[4:5mdashed$e[0m $e[1;4;31mbold red$e[0m`r`n"
Add-Text "Wide: [`u{4E2D}`u{6587}] [`u{65E5}`u{672C}`u{8A9E}] [`u{D55C}`u{AD6D}`u{C5B4}] [`u{1F600}]  Latin: `u{E1} `u{E9} `u{ED} `u{F3} `u{FA} `u{F1} `u{E7} `u{20AC}`r`n"
Add-Text "Box: `u{250C}`u{2500}`u{252C}`u{2500}`u{2510} `u{2554}`u{2550}`u{2557} `u{2591}`u{2592}`u{2593}`u{2588}`r`n"

# The console writes UTF-8 only if it is asked to, and the code page belongs
# to the console, which outlives this script: put the previous one back.
$encoding = [Console]::OutputEncoding
try {
    [Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
    [Console]::Out.Write($sample.ToString())
    [Console]::Out.Flush()
}
finally {
    [Console]::OutputEncoding = $encoding
}
if ($Wait -gt 0) {
    Start-Sleep -Seconds $Wait
}
