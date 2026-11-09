<#
.SYNOPSIS
Checks that meiga.exe presents only what changes, groups changes into frames
and draws nothing while it is idle or minimized.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and, in its terminal, a
PowerShell script that writes to the screen one step at a time, when this
check tells it to. The script hides the cursor first, so that it never
blinks. Then the check reads the log and checks that:

- the first paint covers the whole window;
- while one row changes, each frame presents only that row;
- an idle window draws nothing;
- 40 changes written at once make at most 3 frames, because the changes are
  grouped into frames, at most one per interval;
- another background colour (OSC 11), which changes every pixel, makes one
  full frame;
- a minimized window draws nothing, even when the output changes;
- while it is minimized, a synchronized update (DECSET 2026) that the
  program never ends still ends at its deadline: the title written in it
  reaches the window;
- restoring the window paints all of it;
- when the window grows, meiga composes the new size and paints it;
- it ends with exit code 0 when its window closes.

The check tells the script when to write with two named events. It needs
PowerShell 7 (pwsh) in the PATH, for the script in the terminal.

If a check fails, the script ends meiga.exe and keeps the frame log.

.PARAMETER Path
The executable to check. Defaults to target\release\meiga.exe.

.PARAMETER TimeoutSeconds
How long to wait for the window, for each step, and for the process to end.
#>
[CmdletBinding()]
param(
    [string]$Path,

    [ValidateRange(1, 600)]
    [int]$TimeoutSeconds = 10
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
    $Path = Join-Path $PSScriptRoot '..\target\release\meiga.exe'
}
$Path = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path)
if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "$Path does not exist. Build it first with: cargo build --release"
}
$exe = Split-Path -Leaf $Path

if (-not ('Meiga.Tools.PresentCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name PresentCheck -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)]
public struct RECT { public int Left, Top, Right, Bottom; }

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ShowWindowAsync(IntPtr window, int command);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetClientRect(IntPtr window, out RECT rect);

[DllImport("user32.dll")]
public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
'@
}

$SW_MINIMIZE = 6
$SW_RESTORE = 9
# SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS
$SWP_RESIZE_ONLY = 0x0002 -bor 0x0004 -bor 0x0010 -bor 0x4000

# The script that runs in the terminal. Each time the check sets the event
# <Name>-go, it writes the next step and sets <Name>-done.
$program = @'
param([Parameter(Mandatory)][string]$Name)
$ErrorActionPreference = 'Stop'
$go = [Threading.EventWaitHandle]::OpenExisting("$Name-go")
$done = [Threading.EventWaitHandle]::OpenExisting("$Name-done")
$e = [char]27
$out = [Console]::Out
$steps = @(
    # Hide the cursor, so that it never blinks, and clear the screen.
    { $out.Write("$e[?25l$e[2J$e[H") },
    # Row 5, five times, slowly.
    { foreach ($i in 1..5) { $out.Write("$e[5;3Hrow 5, $i of 5"); Start-Sleep -Milliseconds 80 } },
    # Row 8, 40 times at once.
    { foreach ($i in 1..40) { $out.Write("$e[8;3Hrow 8, $i of 40") } },
    # Another background colour, which changes every pixel.
    { $out.Write("$e]11;#202830$e\") },
    # Rows 10 to 14, while the window is minimized.
    { foreach ($i in 10..14) { $out.Write("$e[$i;3Hrow $i"); Start-Sleep -Milliseconds 50 } },
    # Still minimized, a synchronized update that is never ended, with a
    # title in it.
    { $out.Write("$e[?2026h$e]0;meiga-sync$e\") }
)
foreach ($step in $steps) {
    if (-not $go.WaitOne(60000)) { exit 1 }
    & $step
    $null = $done.Set()
}
# Until the window closes, which ends this script.
$null = $go.WaitOne()
'@

$name = "meiga-check-present-$PID"
$log = Join-Path ([IO.Path]::GetTempPath()) "$name.csv"
$script = Join-Path ([IO.Path]::GetTempPath()) "$name.ps1"
Set-Content -LiteralPath $script -Value $program -Encoding utf8NoBOM
$go = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::AutoReset, "$name-go")
$done = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::AutoReset, "$name-done")

# As per monitor (v2), the same as meiga, the size of the window is in pixels,
# as in its log.
$previousDpi = [Meiga.Tools.PresentCheck]::SetThreadDpiAwarenessContext([IntPtr]::new(-4))

$info = [Diagnostics.ProcessStartInfo]::new($Path)
$info.UseShellExecute = $false
foreach ($argument in '--trace-frames', $log, 'pwsh', '-NoLogo', '-NoProfile', '-NonInteractive',
    '-ExecutionPolicy', 'Bypass', '-File', $script, '-Name', $name) {
    $info.ArgumentList.Add($argument)
}
$process = $null
$passed = $false
try {
    $process = [Diagnostics.Process]::Start($info)
    $idle = try { $process.WaitForInputIdle($TimeoutSeconds * 1000) } catch { $false }
    if (-not $idle) {
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) before it opened its window."
        }
        throw "$exe did not open its window within $TimeoutSeconds s."
    }
    $process.Refresh()
    $window = $process.MainWindowHandle

    # The lines of the log, as objects. meiga writes each line at once.
    function Get-Log {
        if (-not (Test-Path -LiteralPath $log)) { return }
        Get-Content -LiteralPath $log | ConvertFrom-Csv
    }

    # Waits until meiga has written nothing for 0.4 s, more than 10 frame
    # intervals, and returns the lines that it wrote after the first $From.
    function Wait-Quiet([int]$From) {
        $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
        $count = -1
        $since = [DateTime]::UtcNow
        while ($true) {
            $lines = @(Get-Log)
            if ($lines.Count -ne $count) {
                $count = $lines.Count
                $since = [DateTime]::UtcNow
            }
            elseif (([DateTime]::UtcNow - $since).TotalMilliseconds -ge 400) {
                return $lines | Select-Object -Skip $From
            }
            if ([DateTime]::UtcNow -gt $deadline) {
                throw "$exe kept writing frames for $TimeoutSeconds s."
            }
            Start-Sleep -Milliseconds 50
        }
    }

    # Tells the script in the terminal to write its next step, and waits
    # until it has written it.
    function Invoke-Step([string]$What) {
        $null = $go.Set()
        if (-not $done.WaitOne($TimeoutSeconds * 1000)) {
            if ($process.HasExited) {
                throw "$exe ended with exit code $($process.ExitCode) before the script in its terminal could $What."
            }
            throw "The script in the terminal did not $What within $TimeoutSeconds s."
        }
    }

    function Get-ClientSize {
        $rect = [Meiga.Tools.PresentCheck+RECT]::new()
        if (-not [Meiga.Tools.PresentCheck]::GetClientRect($window, [ref]$rect)) {
            throw "Could not read the size of the window of $exe."
        }
        [pscustomobject]@{ Width = $rect.Right - $rect.Left; Height = $rect.Bottom - $rect.Top }
    }

    # The rectangles of the areas column: "x y w h;x y w h".
    function Get-Areas($Line) {
        foreach ($area in ($Line.areas -split ';' | Where-Object { $_ })) {
            $x, $y, $w, $h = [int[]]($area -split ' ')
            [pscustomobject]@{ Left = $x; Top = $y; Right = $x + $w; Bottom = $y + $h }
        }
    }

    function Test-Full($Line) {
        [long]$Line.pixels -eq [long]$Line.width * [long]$Line.height -and [long]$Line.pixels -gt 0
    }

    function Show-Lines($Lines) {
        $Lines | Format-Table frame, time_ms, kind, cause, width, height, rects, pixels, areas -AutoSize | Out-Host
    }

    # 1. The first paint, before the script in the terminal starts. Then the
    # script hides the cursor and clears the screen.
    Invoke-Step 'start'
    $lines = @(Wait-Quiet 0)
    $paint = @($lines | Where-Object kind -EQ 'paint') | Select-Object -First 1
    if (-not $paint -or -not (Test-Full $paint)) {
        Show-Lines $lines
        throw 'The first paint does not cover the whole window.'
    }
    $width = [int]$paint.width
    $height = [int]$paint.height
    if ($width -lt 400 -or $height -lt 300) {
        throw "The window is too small for the check: $width x $height pixels."
    }
    Write-Host "OK: the first paint covers the whole window ($width x $height pixels)."

    # 2. One row changes five times, slowly: every frame is inside that row.
    $start = @(Get-Log).Count
    Invoke-Step 'write one row'
    $lines = @(Wait-Quiet $start)
    $frames = @($lines | Where-Object kind -EQ 'frame')
    $areas = @($frames | ForEach-Object { Get-Areas $_ })
    $tops = @($areas | ForEach-Object Top | Sort-Object -Unique)
    $bottoms = @($areas | ForEach-Object Bottom | Sort-Object -Unique)
    if ($frames.Count -lt 2 -or $tops.Count -ne 1 -or $bottoms.Count -ne 1 -or 10 * ($bottoms[0] - $tops[0]) -gt $height) {
        Show-Lines $lines
        throw 'While one row changed, the frames did not present only that row.'
    }
    $pixels = ($frames | Measure-Object -Property pixels -Sum).Sum
    Write-Host "OK: a row that changed 5 times made $($frames.Count) frames, all inside it ($pixels pixels in total)."

    # 3. Nothing changes: nothing is drawn.
    $start = @(Get-Log).Count
    Start-Sleep -Seconds 1
    $lines = @(Get-Log | Select-Object -Skip $start)
    if ($lines.Count -gt 0) {
        Show-Lines $lines
        throw 'The window drew while nothing changed.'
    }
    Write-Host 'OK: while nothing changes, it draws nothing (1 s).'

    # 4. 40 changes of another row, written at once.
    $start = @(Get-Log).Count
    Invoke-Step 'write 40 changes'
    $frames = @(Wait-Quiet $start | Where-Object kind -EQ 'frame')
    if ($frames.Count -lt 1 -or $frames.Count -gt 3) {
        Show-Lines $frames
        throw "40 changes written at once made $($frames.Count) frames instead of 1 to 3."
    }
    Write-Host "OK: 40 changes written at once made $($frames.Count) frames."

    # 5. Another background colour changes every pixel, margins included.
    $start = @(Get-Log).Count
    Invoke-Step 'change the background colour'
    $frames = @(Wait-Quiet $start | Where-Object kind -EQ 'frame')
    if ($frames.Count -ne 1 -or -not (Test-Full $frames[0])) {
        Show-Lines $frames
        throw 'Another background colour did not make one full frame.'
    }
    Write-Host 'OK: another background colour makes one full frame.'

    # 6. Minimized, it draws nothing, even if the output changes.
    $null = [Meiga.Tools.PresentCheck]::ShowWindowAsync($window, $SW_MINIMIZE)
    $null = Wait-Quiet 0
    $start = @(Get-Log).Count
    Invoke-Step 'write while the window is minimized'
    Start-Sleep -Milliseconds 300
    $lines = @(Wait-Quiet $start)
    if ($lines.Count -gt 0) {
        Show-Lines $lines
        throw 'The window drew while it was minimized.'
    }
    Write-Host 'OK: minimized, it draws nothing, even when the output changes.'

    # 7. Still minimized, a synchronized update that the program does not
    # end ends at its deadline: the title written in it reaches the window,
    # and nothing is drawn.
    $start = @(Get-Log).Count
    Invoke-Step 'open a synchronized update while the window is minimized'
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    $process.Refresh()
    while ($process.MainWindowTitle -ne 'meiga-sync') {
        if ([DateTime]::UtcNow -gt $deadline) {
            throw "Minimized, a synchronized update that the program did not end did not end at its deadline: the title of the window is still '$($process.MainWindowTitle)'."
        }
        Start-Sleep -Milliseconds 50
        $process.Refresh()
    }
    $lines = @(Get-Log | Select-Object -Skip $start)
    if ($lines.Count -gt 0) {
        Show-Lines $lines
        throw 'The window drew while it was minimized.'
    }
    Write-Host 'OK: minimized, a synchronized update that the program leaves open ends at its deadline.'

    # 8. Restored, all of it is painted, with what changed meanwhile.
    $start = @(Get-Log).Count
    $null = [Meiga.Tools.PresentCheck]::ShowWindowAsync($window, $SW_RESTORE)
    $lines = @(Wait-Quiet $start)
    $paint = @($lines | Where-Object { $_.kind -eq 'paint' -and (Test-Full $_) })
    if ($paint.Count -lt 1) {
        Show-Lines $lines
        throw 'Restoring the window did not paint all of it.'
    }
    Write-Host 'OK: restoring the window paints all of it.'

    # 9. Smaller and then bigger.
    $start = @(Get-Log).Count
    $null = [Meiga.Tools.PresentCheck]::SetWindowPos($window, [IntPtr]::Zero, 0, 0, 500, 400, $SWP_RESIZE_ONLY)
    $null = Wait-Quiet $start
    $start = @(Get-Log).Count
    $null = [Meiga.Tools.PresentCheck]::SetWindowPos($window, [IntPtr]::Zero, 0, 0, 800, 600, $SWP_RESIZE_ONLY)
    $lines = @(Wait-Quiet $start)
    $size = Get-ClientSize
    $compose = @($lines | Where-Object kind -EQ 'compose') | Select-Object -Last 1
    $paint = @($lines | Where-Object kind -EQ 'paint')
    if (-not $compose -or [int]$compose.width -ne $size.Width -or [int]$compose.height -ne $size.Height -or $paint.Count -lt 1) {
        Show-Lines $lines
        throw "When the window grew to $($size.Width) x $($size.Height) pixels, it was not composed and painted at that size."
    }
    Write-Host "OK: when the window grows ($($size.Width) x $($size.Height) pixels), it is composed and painted at the new size."

    # CloseMainWindow sends WM_CLOSE, as the X button does.
    if (-not $process.CloseMainWindow()) {
        throw "Could not ask the window of $exe to close."
    }
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after its window was asked to close."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Write-Host 'OK: it ends with exit code 0 when its window closes.'
    $passed = $true
}
finally {
    $null = [Meiga.Tools.PresentCheck]::SetThreadDpiAwarenessContext($previousDpi)
    if ($process) {
        if (-not $process.HasExited) {
            $process.Kill()
        }
        $process.Dispose()
    }
    $go.Dispose()
    $done.Dispose()
    Remove-Item -LiteralPath $script -ErrorAction SilentlyContinue
    if ($passed) {
        Remove-Item -LiteralPath $log -ErrorAction SilentlyContinue
    }
    elseif (Test-Path -LiteralPath $log) {
        Write-Host "The frame log is in $log"
    }
}
