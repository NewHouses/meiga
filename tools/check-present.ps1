<#
.SYNOPSIS
Checks that meiga.exe presents only what changes, groups changes into frames
and draws nothing while it is idle or minimized.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames), sends mouse events to
its window as the user would, and checks in the log that:

- the first paint covers the whole window;
- while a rectangle is drawn with the mouse, each frame presents only part of
  the window, inside that rectangle;
- an idle window draws nothing;
- 40 mouse moves sent at once make at most 3 frames, because the changes are
  grouped into frames, at most one per interval;
- the right button, which clears the window, makes a full frame;
- a minimized window draws nothing, and restoring it paints all of it;
- when the window grows, meiga composes the new size and paints it;
- it ends with exit code 0 when its window closes.

The events are posted to the window (PostMessage), so the real mouse is not
used. Even so, do not click or type while the check runs: a click or the Esc
key in the window would change what it draws.

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
public static extern bool PostMessageW(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);

[DllImport("user32.dll")]
public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ShowWindowAsync(IntPtr window, int command);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetClientRect(IntPtr window, out RECT rect);
'@
}

$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_RBUTTONDOWN = 0x0204
$WM_RBUTTONUP = 0x0205
$MK_LBUTTON = 0x0001
$MK_RBUTTON = 0x0002
$SW_MINIMIZE = 6
$SW_RESTORE = 9
# SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS
$SWP_RESIZE_ONLY = 0x0002 -bor 0x0004 -bor 0x0010 -bor 0x4000

# Windows scales the coordinates of the mouse messages that a thread sends
# with its own DPI awareness. As per monitor (v2), the same as meiga, they
# are pixels of the window, as in its log.
$previousDpi = [Meiga.Tools.PresentCheck]::SetThreadDpiAwarenessContext([IntPtr]::new(-4))

$log = Join-Path ([IO.Path]::GetTempPath()) "meiga-check-present-$PID.csv"
$info = [Diagnostics.ProcessStartInfo]::new($Path)
$info.UseShellExecute = $false
$info.ArgumentList.Add('--trace-frames')
$info.ArgumentList.Add($log)
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

    function Send-Mouse([int]$Message, [int]$Buttons, [int]$X, [int]$Y) {
        $position = [IntPtr](($Y -shl 16) -bor ($X -band 0xFFFF))
        if (-not [Meiga.Tools.PresentCheck]::PostMessageW($window, $Message, [IntPtr]$Buttons, $position)) {
            throw "Could not send a mouse message to the window of $exe."
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

    # 1. The first paint.
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

    # 2. A rectangle drawn with the mouse, from (40, 40) to (240, 160).
    $start = @(Get-Log).Count
    Send-Mouse $WM_LBUTTONDOWN $MK_LBUTTON 40 40
    for ($step = 1; $step -le 10; $step++) {
        Start-Sleep -Milliseconds 15
        Send-Mouse $WM_MOUSEMOVE $MK_LBUTTON (40 + 20 * $step) (40 + 12 * $step)
    }
    Send-Mouse $WM_LBUTTONUP 0 240 160
    $lines = @(Wait-Quiet $start)
    $frames = @($lines | Where-Object kind -EQ 'frame')
    $outside = @($frames | Where-Object {
            (Test-Full $_) -or @(Get-Areas $_ | Where-Object {
                    $_.Left -lt 40 -or $_.Top -lt 40 -or $_.Right -gt 241 -or $_.Bottom -gt 161
                }).Count -gt 0
        })
    if ($frames.Count -lt 2 -or $outside.Count -gt 0) {
        Show-Lines $lines
        throw 'While drawing the rectangle, the frames did not present only parts of it.'
    }
    $pixels = ($frames | Measure-Object -Property pixels -Sum).Sum
    Write-Host "OK: drawing a rectangle made $($frames.Count) frames, all inside it ($pixels pixels in total)."

    # 3. Nothing changes: nothing is drawn.
    $start = @(Get-Log).Count
    Start-Sleep -Seconds 1
    $lines = @(Get-Log | Select-Object -Skip $start)
    if ($lines.Count -gt 0) {
        Show-Lines $lines
        throw 'The window drew while nothing changed.'
    }
    Write-Host 'OK: while nothing changes, it draws nothing (1 s).'

    # 4. A burst of 40 moves, sent at once, in a second rectangle.
    $start = @(Get-Log).Count
    Send-Mouse $WM_LBUTTONDOWN $MK_LBUTTON 300 40
    for ($step = 1; $step -le 40; $step++) {
        Send-Mouse $WM_MOUSEMOVE $MK_LBUTTON (300 + 2 * $step) (40 + 2 * $step)
    }
    Send-Mouse $WM_LBUTTONUP 0 380 120
    $frames = @(Wait-Quiet $start | Where-Object kind -EQ 'frame')
    if ($frames.Count -lt 1 -or $frames.Count -gt 3) {
        Show-Lines $frames
        throw "40 moves sent at once made $($frames.Count) frames instead of 1 to 3."
    }
    Write-Host "OK: 40 moves sent at once made $($frames.Count) frames."

    # 5. The right button clears the window.
    $start = @(Get-Log).Count
    Send-Mouse $WM_RBUTTONDOWN $MK_RBUTTON 50 50
    Send-Mouse $WM_RBUTTONUP 0 50 50
    $frames = @(Wait-Quiet $start | Where-Object kind -EQ 'frame')
    if ($frames.Count -ne 1 -or -not (Test-Full $frames[0])) {
        Show-Lines $frames
        throw 'Clearing the window did not make one full frame.'
    }
    Write-Host 'OK: clearing the window makes one full frame.'

    # 6. Minimized, it draws nothing, even if the content changes.
    $null = [Meiga.Tools.PresentCheck]::ShowWindowAsync($window, $SW_MINIMIZE)
    $null = Wait-Quiet 0
    $start = @(Get-Log).Count
    Send-Mouse $WM_LBUTTONDOWN $MK_LBUTTON 60 200
    Send-Mouse $WM_MOUSEMOVE $MK_LBUTTON 160 260
    Send-Mouse $WM_LBUTTONUP 0 160 260
    Start-Sleep -Milliseconds 300
    $lines = @(Wait-Quiet $start)
    if ($lines.Count -gt 0) {
        Show-Lines $lines
        throw 'The window drew while it was minimized.'
    }
    Write-Host 'OK: minimized, it draws nothing.'

    # 7. Restored, all of it is painted, with what changed meanwhile.
    $start = @(Get-Log).Count
    $null = [Meiga.Tools.PresentCheck]::ShowWindowAsync($window, $SW_RESTORE)
    $lines = @(Wait-Quiet $start)
    $paint = @($lines | Where-Object { $_.kind -eq 'paint' -and (Test-Full $_) })
    if ($paint.Count -lt 1) {
        Show-Lines $lines
        throw 'Restoring the window did not paint all of it.'
    }
    Write-Host 'OK: restoring the window paints all of it.'

    # 8. Smaller and then bigger.
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
    if ($passed) {
        Remove-Item -LiteralPath $log -ErrorAction SilentlyContinue
    }
    elseif (Test-Path -LiteralPath $log) {
        Write-Host "The frame log is in $log"
    }
}
