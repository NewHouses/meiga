<#
.SYNOPSIS
Creates the big file that the bigfile load scenario prints.

.DESCRIPTION
Writes .perf\bigfile.txt: about 50 MiB of text that looks like a long log,
with what makes a terminal work hardest:

- colours of the 16-colour, 256-colour and 24-bit palettes, bold and dim;
- accented letters, Greek and Cyrillic;
- CJK characters and emoji, which take two cells;
- box drawing characters and 24-bit colour bars of block characters;
- empty lines and lines longer than 80 columns, which wrap.

It builds 2000 different lines and repeats them until the file reaches the
size. The lines come from a fixed sequence of pseudo-random numbers, so the
same size always gives the same file, byte by byte, on any machine: every
terminal and every day print exactly the same thing. The script prints the
SHA-256 of the file to compare it.

.PARAMETER Path
File to create. Defaults to .perf\bigfile.txt, which git ignores.

.PARAMETER SizeMB
Size of the file, in MiB. The file ends at the end of the block of 2000 lines
that reaches the size, so it is a little bigger.

.EXAMPLE
pwsh -File .\tools\perf\gen-bigfile.ps1
#>
[CmdletBinding()]
param(
    [string]$Path,

    [ValidateRange(1, 1024)]
    [int]$SizeMB = 50
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
    $Path = Join-Path $PSScriptRoot '../../.perf/bigfile.txt'
}
$Path = [IO.Path]::GetFullPath($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path))
$invariant = [cultureinfo]::InvariantCulture

# A linear congruential generator, with the constants of the C standard. Its
# own code, and not System.Random, keeps the sequence the same in every
# version of .NET.
$seed = 0L
function Get-Number([int]$Count) {
    $script:seed = ($script:seed * 1103515245 + 12345) % 2147483648
    [int](($script:seed -shr 16) % $Count)
}

function Get-Word {
    $words = $script:words
    $words[(Get-Number $words.Count)]
}

# Text of $Count random characters from the code points $First to
# $First + $Range - 1.
function Get-Characters([int]$First, [int]$Range, [int]$Count) {
    -join (1..$Count | ForEach-Object { [char]::ConvertFromUtf32($First + (Get-Number $Range)) })
}

$words = @(
    'frame', 'glyph', 'cache', 'resize', 'scroll', 'damage', 'cursor', 'buffer', 'thread', 'wakeup', 'parse', 'flush',
    'tab', 'group', 'prompt', 'session', 'restore', 'path', 'width', 'height', 'row', 'cell', 'pipe', 'read', 'write',
    'render', 'present', 'atlas', 'font', 'shell', 'profile', 'config', 'input', 'key', 'mouse', 'select', 'copy',
    'paste', 'title', 'bell', 'exit', 'spawn', 'console', 'conpty', 'timer', 'message', 'window', 'dpi', 'scale'
)
$modules = 'conpty', 'render', 'term', 'shell', 'session', 'explorer', 'config', 'input'
# Name and colour of each log level: 16-colour palette, dim and bold.
$levels = @(
    @('INFO ', '32'), @('INFO ', '32'), @('INFO ', '32'), @('DEBUG', '2'), @('DEBUG', '2'), @('WARN ', '33'),
    @('ERROR', '1;31'), @('TRACE', '90')
)
# Lower-case accented letters, from U+00E0 to U+00FF, without U+00F7 (a
# division sign).
$accented = @(0xE0..0xF6) + @(0xF8..0xFF) | ForEach-Object { [char]$_ }
$vowels = 'aeiou'
$consonants = 'bcdfgklmnprstvxz'

$block = [Text.StringBuilder]::new()
for ($line = 0; $line -lt 2000; $line++) {
    # Each line starts its own sequence, so a change in one kind of line does
    # not change the others.
    $seed = ($line * 2654435761L + 12345) % 2147483648
    $kind = Get-Number 20
    if ($kind -lt 12) {
        # A log line: time, level, module and a message.
        $time = [datetime]::new(2026, 1, 15, 10, 0, 0).AddMilliseconds($line * 37)
        $level = $levels[(Get-Number $levels.Count)]
        $module = $modules[(Get-Number $modules.Count)]
        $message = -join (1..(3 + (Get-Number 12)) | ForEach-Object { ' ' + (Get-Word) })
        [void]$block.Append("`e[2m$($time.ToString('yyyy-MM-dd HH:mm:ss.fff', $invariant))`e[0m `e[$($level[1])m$($level[0])`e[0m ")
        [void]$block.Append("`e[38;5;$(17 + (Get-Number 214))m$module`e[0m:$message")
        if ((Get-Number 3) -eq 0) {
            [void]$block.Append(" `e[1m$(Get-Number 100000)`e[0m ms")
        }
    }
    elseif ($kind -lt 14) {
        # Words with accented letters, then Greek (U+03B1 to U+03C9) and
        # Cyrillic (U+0430 to U+044F) words.
        $parts = foreach ($index in 1..(4 + (Get-Number 8))) {
            -join (1..(2 + (Get-Number 4)) | ForEach-Object {
                    $consonants[(Get-Number $consonants.Length)]
                    if ((Get-Number 3) -eq 0) { $accented[(Get-Number $accented.Count)] } else { $vowels[(Get-Number $vowels.Length)] }
                })
        }
        [void]$block.Append("`e[3m$($parts -join ' ')`e[0m $(Get-Characters 0x3B1 25 (3 + (Get-Number 6))) $(Get-Characters 0x430 32 (3 + (Get-Number 8)))")
    }
    elseif ($kind -eq 14) {
        # CJK ideographs, hiragana and hangul, two cells each.
        [void]$block.Append("`e[36m$(Get-Characters 0x4E00 20000 (5 + (Get-Number 30)))`e[0m $(Get-Characters 0x3041 86 (3 + (Get-Number 8))) $(Get-Characters 0xAC00 11172 (2 + (Get-Number 8)))")
    }
    elseif ($kind -eq 15) {
        # Words and emoji, from U+1F600 to U+1F64F, two cells each.
        $parts = foreach ($index in 1..(3 + (Get-Number 10))) {
            if ((Get-Number 2) -eq 0) { Get-Characters 0x1F600 80 1 } else { Get-Word }
        }
        [void]$block.Append($parts -join ' ')
    }
    elseif ($kind -eq 16) {
        # A bar of 64 full blocks with a 24-bit colour gradient: one colour
        # sequence per cell.
        $start = Get-Number 256
        foreach ($cell in 0..63) {
            $red = ($start + $cell * 4) % 256
            [void]$block.Append("`e[38;2;$red;$(255 - $red);$(($red * 3) % 256)m`u{2588}")
        }
        [void]$block.Append("`e[0m")
    }
    elseif ($kind -eq 17) {
        # A table row with box drawing characters and 256-colour backgrounds.
        $cells = foreach ($index in 1..4) {
            "`e[48;5;$(232 + (Get-Number 24))m $((Get-Word).PadRight(10)) `e[0m"
        }
        [void]$block.Append("`u{2502}" + ($cells -join "`u{2502}") + "`u{2502}")
    }
    elseif ($kind -eq 18) {
        # A long line, which wraps in an 80-column window.
        [void]$block.Append(-join (1..(30 + (Get-Number 20)) | ForEach-Object { (Get-Word) + ' ' }))
    }
    [void]$block.Append("`n")
}

$bytes = [Text.UTF8Encoding]::new($false).GetBytes($block.ToString())
$repeats = [int][math]::Ceiling($SizeMB * 1MB / $bytes.Length)
New-Item -ItemType Directory -Force -Path (Split-Path $Path) | Out-Null
$stream = [IO.File]::Create($Path)
try {
    for ($index = 0; $index -lt $repeats; $index++) {
        $stream.Write($bytes, 0, $bytes.Length)
    }
}
finally {
    $stream.Dispose()
}

$hash = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
$size = (Get-Item -LiteralPath $Path).Length
Write-Host ([string]::Format($invariant, 'Created {0}: {1:N0} bytes, 2000 lines repeated {2} times.', $Path, $size, $repeats))
Write-Host "SHA-256: $hash"
