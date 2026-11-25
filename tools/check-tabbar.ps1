<#
.SYNOPSIS
Checks the tab bar of meiga.exe with the mouse: the button that opens a tab,
the close buttons, the clicks on the tabs, and what each change draws.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and, in its first tab, a
PowerShell script that does one step at a time, when this check tells it to,
and reports it in a file. The check posts mouse messages to the bar, at the
places of its layout, which it computes as meiga does: a row of tabs 32
pixels high at a scale of 100 %, below the band where the window is
resized, tabs of up to 240, the buttons of the window on its right, and so
on (ADR 0020 and 0027). It checks that:

- the terminal is below the bar: a row that changes is presented in its
  place there, as high as a cell, and the rows fill the rest of the window;
- while a program gives titles as fast as it can, in the tab that is shown
  or in a hidden one, the window takes them at most once per frame
  interval: its thread wakes up at most 1000 times per second (it counts its
  context switches, with tools/perf/ProcessSnapshot.cs), and the frames draw
  only the title of the tab in the bar. The window takes the last title of
  the tab that it shows, and keeps it while a hidden tab gives titles. Then
  the program gives that same title as fast as it can: the bound holds too,
  and nothing is drawn, since nothing changes; in a hidden tab, since 3.1,
  the window does not even wake up, at most 20 times per second, as the
  same title is not a change for the terminal (ADR 0029). The bound needs
  the ConPTY of meiga next to meiga.exe, which gives the window each title:
  the console of Windows gives them together, at its own pace, so without
  it the check warns that the bound is not really checked;
- the pointer on the button + draws only the button, and so does its press;
  when the left button goes up on it, a tab with the default shell opens,
  and is shown with one full frame;
- the pointer on a tab draws only that tab;
- a click on a tab shows it with one full frame, and the window takes its
  title;
- a press on the close button of a tab that goes up outside the button
  closes nothing, and draws only in the bar;
- while the middle button is down on a tab, Ctrl+Tab shows another tab and
  the bar keeps the mouse: the window still has it captured, until the
  button goes up, outside the tab, which closes nothing;
- the middle button closes the tab under it, and its programs end; the bar
  is drawn again, and nothing else;
- a reset of the screen (ESC c) keeps the title of the tab, and draws only
  the terminal; the titles that the program saved, before the reset and
  after it, come back when it gives them again; and a title of more than
  1000 characters is ignored, and draws nothing in the bar (ADR 0029);
- while nothing changes, nothing is drawn;
- the close button of the last tab closes the window, and meiga.exe ends
  with exit code 0.

Windows gives the window mouse input of its own, such as the moves at the
place of the pointer. So the check moves the pointer out of the window, if
it is over it, and posts its messages while the thread of the window is
suspended: once the thread runs again, it reads the posted messages before
any input (see check-mouse.ps1). The pointer is never over the window, so
Windows tells the window that it left as soon as the window follows its
leave: the check does not count on the highlights staying. It types
Ctrl+Tab as check-tabs.ps1 does: meanwhile, it also sets the key state of
the thread of the window, which it shares.

It needs PowerShell 7 (pwsh) in the PATH, for the script in the terminal.
The new tab runs the default profile of meiga, which is PowerShell 7 if
meiga finds it, and loads the profile of PowerShell: the check waits until
its shell stops using the processor, but a profile that draws something
seconds after the prompt can make the check fail.

If a check fails, the script ends meiga.exe and keeps the frame log and the
reports of the script.

.PARAMETER Path
The executable to check. Defaults to target\release\meiga.exe.

.PARAMETER TimeoutSeconds
How long to wait for the window, for each step, and for the processes to
end.
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
# The console of Windows gives the titles of a program together, at its own
# pace: only the ConPTY that meiga ships gives the window each one.
if (-not (Test-Path -LiteralPath (Join-Path (Split-Path -Parent $Path) 'conpty.dll') -PathType Leaf)) {
    Write-Warning "There is no conpty.dll next to $exe, so the console of Windows gives the titles together, and the bound on the wake-ups of the window is not really checked. tools\package.ps1 copies the ConPTY of meiga there."
}

if (-not ('Meiga.Tools.TabBarCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name TabBarCheck -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)]
public struct RECT { public int Left, Top, Right, Bottom; }

[StructLayout(LayoutKind.Sequential)]
public struct POINT { public int X, Y; }

[StructLayout(LayoutKind.Sequential)]
public struct MSG { public IntPtr hwnd; public uint message; public UIntPtr wParam; public IntPtr lParam; public uint time; public int x, y; }

[StructLayout(LayoutKind.Sequential)]
public struct GUITHREADINFO
{
    public int cbSize;
    public uint flags;
    public IntPtr hwndActive, hwndFocus, hwndCapture, hwndMenuOwner, hwndMoveSize, hwndCaret;
    public RECT rcCaret;
}

[DllImport("user32.dll")]
public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);

[DllImport("kernel32.dll")]
public static extern uint GetCurrentThreadId();

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool PeekMessageW(out MSG message, IntPtr window, uint first, uint last, uint remove);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool AttachThreadInput(uint thread, uint to, [MarshalAs(UnmanagedType.Bool)] bool attach);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetKeyboardState(byte[] state);

[DllImport("user32.dll")]
public static extern IntPtr GetKeyboardLayout(uint thread);

[DllImport("user32.dll")]
public static extern uint MapVirtualKeyExW(uint code, uint type, IntPtr layout);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool GetGUIThreadInfo(uint thread, ref GUITHREADINFO info);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetClientRect(IntPtr window, out RECT rect);

[DllImport("user32.dll")]
public static extern uint GetDpiForWindow(IntPtr window);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetWindowRect(IntPtr window, out RECT rect);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsZoomed(IntPtr window);

[DllImport("user32.dll")]
public static extern int GetSystemMetricsForDpi(int index, uint dpi);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetCursorPos(out POINT point);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetCursorPos(int x, int y);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ScreenToClient(IntPtr window, ref POINT point);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ClientToScreen(IntPtr window, ref POINT point);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool PostMessageW(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);

[DllImport("user32.dll")]
public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

[DllImport("kernel32.dll", SetLastError = true)]
static extern IntPtr OpenThread(uint access, [MarshalAs(UnmanagedType.Bool)] bool inherit, uint id);

[DllImport("kernel32.dll")]
static extern int GetThreadPriority(IntPtr thread);

[DllImport("kernel32.dll")]
static extern int GetThreadDescription(IntPtr thread, out IntPtr description);

[DllImport("kernel32.dll")]
static extern IntPtr LocalFree(IntPtr memory);

[DllImport("kernel32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool CloseHandle(IntPtr handle);

[DllImport("kernel32.dll", SetLastError = true)]
static extern uint SuspendThread(IntPtr thread);

[DllImport("kernel32.dll")]
static extern uint ResumeThread(IntPtr thread);

[DllImport("kernel32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool GetThreadContext(IntPtr thread, IntPtr context);

// Posts messages to a window: each one is three numbers, the message, its
// wParam and its lParam. The thread of the window is suspended meanwhile,
// so it reads all of them before the input that Windows makes: a thread
// reads the posted messages first. If `state` is not null, the thread of
// the window gets it as its key state, which this thread shares: the key
// state changes as the thread reads its input, so it is set while the
// thread is suspended, for the keys of the messages.
public static void PostAll(IntPtr window, uint thread, long[] messages, byte[] state)
{
    // THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT
    IntPtr handle = OpenThread(0x0002 | 0x0008, false, thread);
    if (handle == IntPtr.Zero) throw new System.ComponentModel.Win32Exception();
    // A CONTEXT of x64: 1232 bytes, aligned to 16.
    IntPtr memory = Marshal.AllocHGlobal(1232 + 16);
    try
    {
        if (SuspendThread(handle) == uint.MaxValue) throw new System.ComponentModel.Win32Exception();
        try
        {
            // SuspendThread does not wait until the thread stops; reading its
            // context does. ContextFlags: CONTEXT_CONTROL of x64.
            IntPtr context = new IntPtr((memory.ToInt64() + 15) & ~15L);
            Marshal.WriteInt32(context, 0x30, 0x00100001);
            if (!GetThreadContext(handle, context)) throw new System.ComponentModel.Win32Exception();
            if (state != null && !SetKeyboardState(state)) throw new System.ComponentModel.Win32Exception();
            for (int i = 0; i + 2 < messages.Length; i += 3)
            {
                if (!PostMessageW(window, (uint)messages[i], new UIntPtr((ulong)messages[i + 1]), new IntPtr(messages[i + 2])))
                    throw new System.ComponentModel.Win32Exception();
            }
        }
        finally { ResumeThread(handle); }
    }
    finally
    {
        Marshal.FreeHGlobal(memory);
        CloseHandle(handle);
    }
}

// The window that has the mouse captured in the thread `thread`, or zero.
public static IntPtr CaptureOf(uint thread)
{
    GUITHREADINFO info = new GUITHREADINFO();
    info.cbSize = Marshal.SizeOf(typeof(GUITHREADINFO));
    if (!GetGUIThreadInfo(thread, ref info)) throw new System.ComponentModel.Win32Exception();
    return info.hwndCapture;
}

// The priority of the thread of a process whose name is `name`, or null if
// the process has none, or has ended.
public static int? PriorityOf(int process, string name)
{
    System.Diagnostics.Process target;
    try { target = System.Diagnostics.Process.GetProcessById(process); }
    catch (ArgumentException) { return null; }
    using (target)
    {
        System.Diagnostics.ProcessThreadCollection threads;
        try { threads = target.Threads; }
        catch (InvalidOperationException) { return null; }
        foreach (System.Diagnostics.ProcessThread thread in threads)
        {
            // THREAD_QUERY_LIMITED_INFORMATION
            IntPtr handle = OpenThread(0x0800, false, (uint)thread.Id);
            if (handle == IntPtr.Zero) continue;
            try
            {
                IntPtr text;
                if (GetThreadDescription(handle, out text) < 0) continue;
                string description;
                try { description = Marshal.PtrToStringUni(text); }
                finally { LocalFree(text); }
                if (description == name) return GetThreadPriority(handle);
            }
            finally { CloseHandle(handle); }
        }
    }
    return null;
}
'@
}
$api = [Meiga.Tools.TabBarCheck]
# The process and thread counters of Windows, as the measurements read them.
if (-not ('Meiga.Perf.ProcessSnapshot' -as [type])) {
    Add-Type -Path (Join-Path $PSScriptRoot 'perf\ProcessSnapshot.cs')
}

# How many times per second the thread of the window can wake up while a
# program gives titles as fast as it can. The window takes them at most
# once per frame interval, which is some dozens of times per second; taking
# each one would be thousands.
$MAX_WAKES = 1000

# And while the program of a hidden tab gives the title that its tab already
# has: since 3.1 it is not a change for the terminal, and the output of a
# hidden tab does not wake the window up again once it did, so nothing does.
# Until 3.1, each title woke it up, once per frame interval.
$MAX_HIDDEN_SAME_WAKES = 20

$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_MBUTTONDOWN = 0x0207
$WM_MBUTTONUP = 0x0208
$MK_LBUTTON = 0x0001
$MK_MBUTTON = 0x0010
$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$MAPVK_VK_TO_VSC = 0
# The keys that Ctrl sets down: Ctrl and the left Ctrl.
$ctrlKeys = 0x11, 0xA2
$THREAD_PRIORITY_NORMAL = 0
$THREAD_PRIORITY_BELOW_NORMAL = -1
# The console of each tab: the one of the ConPTY of meiga, or the one of
# Windows if the ConPTY is not next to meiga.exe.
$consoles = 'OpenConsole.exe', 'conhost.exe'

# The script of the first tab. Each time the check sets the event
# <Name>-go, it does its next step and reports it in $Log, and sets
# <Name>-done.
$program = @'
param([Parameter(Mandatory)][string]$Name, [Parameter(Mandatory)][string]$Log)
$ErrorActionPreference = 'Stop'
$go = [Threading.EventWaitHandle]::OpenExisting("$Name-go")
$done = [Threading.EventWaitHandle]::OpenExisting("$Name-done")
$e = [char]27
$out = [Console]::Out
# Titles as fast as the program can write them, for two seconds: other ones
# and then $Last or, with -Same, $Last each time. Returns how many.
function Write-Titles([string]$Last, [switch]$Same) {
    $end = [DateTime]::UtcNow.AddSeconds(2)
    $count = 0
    $title = "$e]0;$Last$e\"
    if ($Same) {
        while ([DateTime]::UtcNow -lt $end) {
            $out.Write($title)
            $count++
        }
        return $count
    }
    while ([DateTime]::UtcNow -lt $end) {
        $out.Write("$e]0;spin $count$e\")
        $count++
    }
    $out.Write($title)
    $count
}
$steps = @(
    # Hide the cursor, so that it never blinks, give the tab a title, and
    # tell the process and the rows of the screen.
    { $out.Write("$e[?25l$e]0;meiga-tabbar-1$e\$e[2J$e[H"); "ready $PID $([Console]::WindowHeight)" },
    # Text in the fifth row.
    { $out.Write("$e[5;3Hrow 5"); 'row' },
    # Titles while the tab is shown, and then the last one again and again.
    { "titles $(Write-Titles 'meiga-tabbar-1')" },
    { "titles $(Write-Titles 'meiga-tabbar-1' -Same)" },
    # Titles while it is hidden, another one, and then that one again and
    # again.
    { "titles $(Write-Titles 'meiga-tabbar-renamed')" },
    { "titles $(Write-Titles 'meiga-tabbar-renamed' -Same)" },
    # A title, and then that one saved and another one; a reset of the
    # screen, which shows the cursor again: it is hidden again, and text
    # written. Two steps for the first title: the console of Windows gives
    # the titles together, later, but passes CSI 22 t at once.
    { $out.Write("$e]0;meiga-tabbar-before$e\"); 'title' },
    { $out.Write("$e[22t$e]0;meiga-tabbar-reset$e\"); 'saved' },
    { $out.Write("${e}c$e[?25l$e[2;3Hreset"); 'reset' },
    # The title saved, and another one; then the saved ones, one at a time.
    { $out.Write("$e[22t$e]0;meiga-tabbar-saved$e\"); 'saved' },
    { $out.Write("$e[23t"); 'restored' },
    { $out.Write("$e[23t"); 'restored' },
    # A title of 1001 characters, and text in the third row.
    { $out.Write("$e]0;$('x' * 1001)$e\$e[3;3Hlong"); 'long' }
)
foreach ($step in $steps) {
    if (-not $go.WaitOne(60000)) { exit 1 }
    [IO.File]::AppendAllText($Log, "$(& $step)`n")
    $null = $done.Set()
}
# Until the tab closes, which ends this script.
$null = $go.WaitOne()
'@

$name = "meiga-check-tabbar-$PID"
$temp = [IO.Path]::GetTempPath()
$log = Join-Path $temp "$name.csv"
$reports = Join-Path $temp "$name.txt"
$script = Join-Path $temp "$name.ps1"
Set-Content -LiteralPath $script -Value $program -Encoding utf8NoBOM
# meiga reads its configuration from %APPDATA%\meiga\config.toml: an empty
# folder instead gives it its default values, whatever the user has there.
# The programs in its tabs get that folder too.
$appdata = Join-Path ([IO.Path]::GetTempPath()) "$name-appdata"
$null = New-Item -ItemType Directory -Force -Path $appdata
Remove-Item -LiteralPath $reports -ErrorAction SilentlyContinue
$go = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::AutoReset, "$name-go")
$done = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::AutoReset, "$name-done")

# As per monitor (v2), the same as meiga, the size of the window is in pixels.
$previousDpi = $api::SetThreadDpiAwarenessContext([IntPtr]::new(-4))

# Waits until $Condition is true, or throws what $Failure says.
function Wait-For([scriptblock]$Condition, [scriptblock]$Failure) {
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while (-not (& $Condition)) {
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode): $(& $Failure)"
        }
        if ([DateTime]::UtcNow -gt $deadline) { throw (& $Failure) }
        Start-Sleep -Milliseconds 20
    }
}

# The lines of the frame log, as objects. meiga writes each line at once.
function Get-Log {
    if (-not (Test-Path -LiteralPath $log)) { return }
    Get-Content -LiteralPath $log | ConvertFrom-Csv
}

# The frames after the first $From lines of the log.
function Get-Frames([int]$From) {
    @(Get-Log | Select-Object -Skip $From | Where-Object kind -EQ 'frame')
}

# Waits until meiga has written nothing in its log for $Milliseconds.
function Wait-Quiet([int]$Milliseconds, [int]$Seconds = $TimeoutSeconds) {
    $deadline = [DateTime]::UtcNow.AddSeconds($Seconds)
    $count = -1
    $since = [DateTime]::UtcNow
    while ($true) {
        $lines = @(Get-Log).Count
        if ($lines -ne $count) {
            $count = $lines
            $since = [DateTime]::UtcNow
        }
        elseif (([DateTime]::UtcNow - $since).TotalMilliseconds -ge $Milliseconds) {
            return
        }
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) while the check waited for it to stop drawing."
        }
        if ([DateTime]::UtcNow -gt $deadline) {
            throw "$exe kept drawing for $Seconds s."
        }
        Start-Sleep -Milliseconds 50
    }
}

function Test-Full($Line) {
    [long]$Line.pixels -eq [long]$Line.width * [long]$Line.height -and [long]$Line.pixels -gt 0
}

function Show-Lines($Lines) {
    $Lines | Format-Table frame, time_ms, kind, cause, width, height, rects, pixels, areas -AutoSize | Out-Host
}

# A rectangle, from its left and top, included, to its right and bottom,
# excluded.
function New-Rect([int]$Left, [int]$Top, [int]$Right, [int]$Bottom) {
    [pscustomobject]@{ Left = $Left; Top = $Top; Right = $Right; Bottom = $Bottom }
}

function Show-Rect($Rect) {
    "($($Rect.Left), $($Rect.Top)) to ($($Rect.Right), $($Rect.Bottom))"
}

# The areas that the frames of $Lines presented: "x y w h;x y w h".
function Get-Areas($Lines) {
    foreach ($line in $Lines) {
        foreach ($area in ($line.areas -split ';' | Where-Object { $_ })) {
            $x, $y, $w, $h = [int[]]($area -split ' ')
            New-Rect $x $y ($x + $w) ($y + $h)
        }
    }
}

# Waits until meiga stops drawing, and returns the frames after the first
# $From lines of the log, which have to be some, all of them inside $Rect;
# otherwise throws $What.
function Assert-Inside([int]$From, $Rect, [string]$What) {
    Wait-Quiet 400
    $frames = @(Get-Frames $From)
    $outside = @(Get-Areas $frames | Where-Object {
            $_.Left -lt $Rect.Left -or $_.Top -lt $Rect.Top -or $_.Right -gt $Rect.Right -or $_.Bottom -gt $Rect.Bottom
        })
    if ($frames.Count -eq 0 -or $outside.Count -gt 0) {
        Show-Lines $frames
        throw "$What`: the window did not draw only inside $(Show-Rect $Rect)."
    }
    $frames
}

# The pixels that $Frames presented.
function Measure-Pixels($Frames) {
    ($Frames | Measure-Object -Property pixels -Sum).Sum
}

# How many times per second the thread of the window woke up since the
# snapshot $Before: its context switches.
function Measure-Wakes($Before) {
    $after = [Meiga.Perf.ProcessSnapshot]::Take()
    $from = $Before.Processes[$process.Id].ContextSwitches[[long]$thread]
    $to = $after.Processes[$process.Id].ContextSwitches[[long]$thread]
    [Math]::Round(([long]$to - [long]$from) / $after.SecondsSince($Before))
}

# Tells the script of the first tab to give titles as fast as it can, for
# two seconds, and checks that the window took them at most once per frame
# interval: it did not wake up more than $MAX_WAKES times per second, and
# drew only $Title, where the bar has the title of the tab. Returns how many
# titles the script gave, and how many times per second the window woke up.
function Assert-Titles($Title, [string]$What) {
    $start = @(Get-Log).Count
    $before = [Meiga.Perf.ProcessSnapshot]::Take()
    $count = [int]((Invoke-Step "give titles $What") -split ' ')[1]
    $wakes = Measure-Wakes $before
    if ($wakes -gt $MAX_WAKES) {
        throw "While the program of $What gave $count titles in two seconds, the window woke up $wakes times per second, more than $MAX_WAKES."
    }
    $frames = Assert-Inside $start $Title "The titles of $What"
    [pscustomobject]@{ Titles = $count; Wakes = $wakes; Frames = $frames.Count }
}

# Tells the script of the first tab to give the title that its tab already
# has, as fast as it can, for two seconds, and checks that the window did
# not wake up more than $MAX_WAKES times per second, and drew nothing: a
# title that does not change changes nothing. Returns how many times the
# script gave it, and how many times per second the window woke up.
function Assert-SameTitle([string]$What) {
    $start = @(Get-Log).Count
    $before = [Meiga.Perf.ProcessSnapshot]::Take()
    $count = [int]((Invoke-Step "give the same title $What") -split ' ')[1]
    $wakes = Measure-Wakes $before
    if ($wakes -gt $MAX_WAKES) {
        throw "While the program of $What gave the same title $count times in two seconds, the window woke up $wakes times per second, more than $MAX_WAKES."
    }
    Wait-Quiet 400
    $frames = @(Get-Frames $start)
    if ($frames.Count -gt 0) {
        Show-Lines $frames
        throw "The window drew while the program of $What gave the same title."
    }
    [pscustomobject]@{ Titles = $count; Wakes = $wakes }
}

# The reports of the script of the first tab.
function Get-Reports {
    if (-not (Test-Path -LiteralPath $reports)) { return @() }
    @(Get-Content -LiteralPath $reports)
}

# Tells the script of the first tab to do its next step, waits until it has
# done it, and returns its report.
function Invoke-Step([string]$What) {
    $null = $go.Set()
    if (-not $done.WaitOne($TimeoutSeconds * 1000)) {
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) before the script of the first tab could $What."
        }
        throw "The script of the first tab did not $What within $TimeoutSeconds s."
    }
    @(Get-Reports)[-1]
}

# The programs of the tabs: the processes that meiga started, but for the
# consoles. A process that ended before meiga started may have had a
# process with the same number as its parent.
function Get-Shells {
    @(Get-CimInstance Win32_Process -Filter "ParentProcessId = $($process.Id)" |
            Where-Object { $_.Name -notin $consoles -and $_.CreationDate -ge $started })
}

# The consoles of the tabs.
function Get-Consoles {
    @(Get-CimInstance Win32_Process -Filter "ParentProcessId = $($process.Id)" |
            Where-Object { $_.Name -in $consoles -and $_.CreationDate -ge $started })
}

# The priority of the reader thread of the tab whose program is $Program,
# or $null if it has none.
function Get-Priority([int]$Program) {
    $api::PriorityOf($process.Id, "conpty reader $Program")
}

# Waits until the tab of $Shown is the one with the reader of normal
# priority, and the ones of $Hidden have the lower one.
function Wait-Shown([int]$Shown, [int[]]$Hidden, [string]$What) {
    Wait-For {
        (Get-Priority $Shown) -eq $THREAD_PRIORITY_NORMAL -and
        @($Hidden | Where-Object { (Get-Priority $_) -ne $THREAD_PRIORITY_BELOW_NORMAL }).Count -eq 0
    } {
        $all = @($Shown) + $Hidden | ForEach-Object { "$_ $(Get-Priority $_)" }
        "$What`: the priorities of the readers are not the expected ones (process and priority: $($all -join ', '))."
    }
}

# Waits until the process $Id has ended, also after meiga.
function Wait-Ended([int]$Id, [string]$What) {
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while (Get-Process -Id $Id -ErrorAction Ignore) {
        if ([DateTime]::UtcNow -gt $deadline) {
            throw "$What was still running after $TimeoutSeconds s."
        }
        Start-Sleep -Milliseconds 20
    }
}

# Waits until the process $Id has used no processor time for a second: a
# shell that starts has loaded its profile and written its prompt.
function Wait-Settled([int]$Id, [string]$What) {
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds + 10)
    $last = -1
    $since = [DateTime]::UtcNow
    while ($true) {
        $program = Get-Process -Id $Id -ErrorAction Ignore
        if (-not $program) { throw "$What ended." }
        $time = $program.TotalProcessorTime.Ticks
        if ($time -ne $last) {
            $last = $time
            $since = [DateTime]::UtcNow
        }
        elseif (([DateTime]::UtcNow - $since).TotalMilliseconds -ge 1000) {
            return
        }
        if ([DateTime]::UtcNow -gt $deadline) {
            throw "$What kept running for $($TimeoutSeconds + 10) s."
        }
        Start-Sleep -Milliseconds 100
    }
}

function Get-Title {
    $process.Refresh()
    $process.MainWindowTitle
}

# If the pointer is over the window, moves it out, above the window: the
# moves that Windows makes at its place would reach the bar.
function Move-Pointer {
    $point = [Meiga.Tools.TabBarCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.TabBarCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = [Meiga.Tools.TabBarCheck+RECT]::new()
    if (-not $api::ScreenToClient($window, [ref]$inside) -or -not $api::GetClientRect($window, [ref]$rect) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = [Meiga.Tools.TabBarCheck+POINT]::new()
    $title.X = [Math]::Floor($rect.Right / 2)
    $title.Y = -8
    $null = $api::ClientToScreen($window, [ref]$title)
    if (-not $api::SetCursorPos($title.X, $title.Y)) {
        throw "Could not move the pointer out of the window: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# The band above the row of tabs at $Dpi, where the pointer resizes the
# window, as meiga computes it (see crates/meiga-win/src/window.rs): as high
# as the frame that Windows gives a window to resize it, while the window
# has the title bar of meiga and is not maximized, and none otherwise. With
# the title bar of meiga, the client area goes up to the top of the window.
function Get-Band([uint32]$Dpi) {
    if ($api::IsZoomed($window)) { return 0 }
    $rect = [Meiga.Tools.TabBarCheck+RECT]::new()
    $corner = [Meiga.Tools.TabBarCheck+POINT]::new()
    if (-not $api::GetWindowRect($window, [ref]$rect) -or -not $api::ClientToScreen($window, [ref]$corner)) {
        throw "Could not read where the window of $exe is."
    }
    if ($corner.Y -ne $rect.Top) { return 0 }
    # SM_CXPADDEDBORDER and SM_CYSIZEFRAME.
    $api::GetSystemMetricsForDpi(92, $Dpi) + $api::GetSystemMetricsForDpi(33, $Dpi)
}

# The layout of the bar with $Count tabs, as meiga computes it (see
# crates/meiga-core/src/ui/tabbar.rs): its sizes at the scale of the window,
# rounded and at least one pixel. The band where the window is resized goes
# above the row of tabs, and the tabs and the button + start a gap below it.
# On the right of the bar go the three buttons of the window, and the room
# to drag it.
function Get-Bar([int]$Count) {
    $client = [Meiga.Tools.TabBarCheck+RECT]::new()
    if (-not $api::GetClientRect($window, [ref]$client)) {
        throw "Could not read the size of the window of $exe."
    }
    $dpi = $api::GetDpiForWindow($window)
    $scale = { param([int]$Pixels) [int][Math]::Max(1, [Math]::Floor(($Pixels * $dpi + 48) / 96)) }
    $band = Get-Band $dpi
    $row = & $scale 32
    $height = $band + $row
    $gap = & $scale 4
    $top = $band + $gap
    $padding = & $scale 8
    $closeSide = & $scale 24
    $newSide = & $scale 28
    $profilesSide = & $scale 24
    $fixed = 3 * $gap + $newSide + $profilesSide + (& $scale 45) + 3 * (& $scale 46)
    $width = [int][Math]::Min((& $scale 240), [Math]::Floor([Math]::Max(0, $client.Right - $fixed) / $Count))
    $tabs = foreach ($index in 0..($Count - 1)) {
        $left = $gap + $index * $width
        $right = $left + $width
        $close = New-Rect ($right - $gap - $closeSide) ($top + [Math]::Floor(($row - $gap - $closeSide) / 2)) ($right - $gap) 0
        $close.Bottom = $close.Top + $closeSide
        [pscustomobject]@{
            Rect = New-Rect $left $top $right $height
            # Its whole column of the row of tabs, and the separator on its
            # right.
            Area = New-Rect $left $band ($right + 1) $height
            Title = New-Rect ($left + $padding) $top $close.Left $height
            Close = $close
        }
    }
    $newLeft = $gap + $Count * $width + $gap
    [pscustomobject]@{
        Rect = New-Rect 0 0 $client.Right $height
        Height = $height
        Band = $band
        Tabs = @($tabs)
        New = New-Rect $newLeft $top ($newLeft + $newSide) ($top + $newSide)
        Client = $client
    }
}

# The middle of a rectangle.
function Get-Middle($Rect) {
    [pscustomobject]@{
        X = [Math]::Floor(($Rect.Left + $Rect.Right) / 2)
        Y = [Math]::Floor(($Rect.Top + $Rect.Bottom) / 2)
    }
}

# A point of a tab, on its title, away from its close button.
function Get-TabPoint($Tab) {
    [pscustomobject]@{ X = $Tab.Title.Left + 4; Y = [Math]::Floor(($Tab.Rect.Top + $Tab.Rect.Bottom) / 2) }
}

# A mouse message at a point: the message, its wParam and its lParam.
function Get-Mouse([int]$Message, [int]$Keys, $Point) {
    $Message, $Keys, (([int]$Point.X -band 0xFFFF) -bor (([int]$Point.Y -band 0xFFFF) -shl 16))
}

# Posts mouse messages at once: see PostAll.
function Send-Mouse([long[]]$Messages) {
    $api::PostAll($window, $thread, $Messages, $null)
}

# The window that has the mouse captured in the thread of the window of
# meiga, or zero.
function Get-Capture {
    $api::CaptureOf($thread)
}

# Shares the input of this thread with the one of the window, whose key
# state Send-Ctrl sets: SetKeyboardState sets the key state of the thread
# that calls it. This thread needs a message queue, which PeekMessageW
# creates.
function Join-Input {
    $message = [Meiga.Tools.TabBarCheck+MSG]::new()
    $null = $api::PeekMessageW([ref]$message, [IntPtr]::Zero, 0, 0, 0)
    if (-not $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $true)) {
        throw "AttachThreadInput failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
    $script:attached = $true
}

# Stops sharing the input with the thread of the window, with no key down.
function Split-Input {
    if ($script:attached) {
        $null = $api::SetKeyboardState([byte[]]::new(256))
        $null = $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $false)
        $script:attached = $false
    }
}

# Types Ctrl+$Key, such as Ctrl+Tab: posts the messages of $Key going down
# and up, with Ctrl down and no other key (see PostAll). The input has to be
# shared with the thread of the window: see Join-Input.
function Send-Ctrl([string]$Key) {
    $code = [int][ConsoleKey]$Key
    $state = [byte[]]::new(256)
    foreach ($down in $ctrlKeys) { $state[$down] = 0x80 }
    $scan = $api::MapVirtualKeyExW($code, $MAPVK_VK_TO_VSC, $api::GetKeyboardLayout($thread))
    [long]$bits = 1 -bor ($scan -shl 16)
    # Going up: the key was down (bit 30), and goes up (bit 31).
    $api::PostAll($window, $thread, [long[]]@($WM_KEYDOWN, $code, $bits, $WM_KEYUP, $code, ($bits -bor 0xC0000000L)), $state)
}

# Moves the pointer below the bar, which takes any highlight off the bar,
# and waits until meiga stops drawing.
function Clear-Highlight($Bar) {
    Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 ([pscustomobject]@{ X = 10; Y = $Bar.Height + 100 }))
    Wait-Quiet 400
}

$info = [Diagnostics.ProcessStartInfo]::new($Path)
$info.UseShellExecute = $false
$info.Environment['APPDATA'] = $appdata
foreach ($argument in '--trace-frames', $log, 'pwsh', '-NoLogo', '-NoProfile', '-NonInteractive',
    '-ExecutionPolicy', 'Bypass', '-File', $script, '-Name', $name, '-Log', $reports) {
    $info.ArgumentList.Add($argument)
}
$process = $null
$pointer = $null
$attached = $false
$passed = $false
try {
    $process = [Diagnostics.Process]::Start($info)
    $started = $process.StartTime
    $idle = try { $process.WaitForInputIdle($TimeoutSeconds * 1000) } catch { $false }
    if (-not $idle) {
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) before it opened its window."
        }
        throw "$exe did not open its window within $TimeoutSeconds s."
    }
    $process.Refresh()
    $window = $process.MainWindowHandle
    $processId = [uint32]0
    $thread = $api::GetWindowThreadProcessId($window, [ref]$processId)

    # 1. The first tab, alone.
    $ready = (Invoke-Step 'start') -split ' '
    $first = [int]$ready[1]
    Wait-For { (Get-Title) -eq 'meiga-tabbar-1' } { "The title of the window is '$(Get-Title)' instead of 'meiga-tabbar-1'." }
    Wait-Shown $first @() 'With one tab'
    Move-Pointer
    Wait-Quiet 400
    $bar = Get-Bar 1
    Write-Host "The bar is $($bar.Height) pixels high, $($bar.Band) of them above the tabs, and its tabs $($bar.Tabs[0].Rect.Right - $bar.Tabs[0].Rect.Left) wide."

    # 2. The terminal is below the bar: a row that changes is presented in
    # its place there. The text is in the fifth row, so what is drawn is as
    # high as a cell, four cells below the bar; and the rows of the screen
    # fill the rest of the window, with less than a row to spare. Dividing
    # the room by the rows would not do: with as many pixels to spare as
    # rows, it gives a cell a pixel higher.
    $rows = [int]$ready[2]
    $room = $bar.Client.Bottom - $bar.Height
    $start = @(Get-Log).Count
    $null = Invoke-Step 'write in the fifth row'
    $frames = Assert-Inside $start (New-Rect 0 $bar.Height $bar.Rect.Right $bar.Client.Bottom) 'Text in the fifth row'
    $areas = @(Get-Areas $frames)
    $top = ($areas | Measure-Object -Property Top -Minimum).Minimum
    $cellHeight = ($areas | Measure-Object -Property Bottom -Maximum).Maximum - $top
    if ($cellHeight -lt 1 -or $top -ne $bar.Height + 4 * $cellHeight -or $rows * $cellHeight -gt $room -or ($rows + 1) * $cellHeight -le $room) {
        Show-Lines $frames
        throw "Text in the fifth row: the window drew it from $top to $($top + $cellHeight) pixels below the top, which is not the fifth of $rows rows that fill the $room pixels below the bar, $($bar.Height) pixels high."
    }
    Write-Host "OK: the terminal is below the bar: the fifth row, which changes, is presented $top pixels below the top of the window ($($frames.Count) frames)."

    # 3. Titles as fast as the program can give them, in the tab that is
    # shown: the last one goes to the window. Then that one again and again.
    $titles = Assert-Titles $bar.Tabs[0].Title 'the tab that is shown'
    Wait-For { (Get-Title) -eq 'meiga-tabbar-1' } `
    { "After the titles of the tab that is shown, the title of the window is '$(Get-Title)' instead of 'meiga-tabbar-1'." }
    $same = Assert-SameTitle 'the tab that is shown'
    Write-Host "OK: while the program of the tab that is shown gives $($titles.Titles) titles in two seconds, the window wakes up $($titles.Wakes) times per second, takes the last one, and draws only the title of its tab ($($titles.Frames) frames); while it gives that title again $($same.Titles) times, the window wakes up $($same.Wakes) times per second, and draws nothing."

    # 4. The button +: the pointer, the press, and the release on it.
    $plus = Get-Middle $bar.New
    $start = @(Get-Log).Count
    Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 $plus)
    $null = Assert-Inside $start $bar.New 'The pointer on the button +'
    $start = @(Get-Log).Count
    Send-Mouse (Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $plus)
    $null = Assert-Inside $start $bar.New 'The press on the button +'
    if (@(Get-Shells).Count -ne 1) {
        throw 'The button + opened a tab when the left button went down on it.'
    }
    $start = @(Get-Log).Count
    Send-Mouse (Get-Mouse $WM_LBUTTONUP 0 $plus)
    Wait-For { @(Get-Shells).Count -eq 2 } { 'The button + did not open a tab with a program.' }
    $opened = Get-Shells | Where-Object { [int]$_.ProcessId -ne $first } | Select-Object -First 1
    $second = [int]$opened.ProcessId
    Wait-Shown $second @($first) 'After the button +'
    Wait-For { @(Get-Frames $start).Count -gt 0 } { 'The window drew nothing after the button +.' }
    $frame = (Get-Frames $start)[0]
    if (-not (Test-Full $frame)) {
        Show-Lines (Get-Frames $start)
        throw 'The first frame after the button + does not draw the whole window.'
    }
    Write-Host "OK: the pointer on the button +, and its press, draw only the button; when the left button goes up on it, a tab opens with the default shell ($($opened.Name)), shown with a full frame."

    # 5. The new tab has written its prompt, and nothing is drawn for a
    # second: its cursor, if it blinks, has stopped.
    Wait-Settled $second 'The program of the new tab'
    Wait-Quiet 1000 ($TimeoutSeconds + 10)
    $bar = Get-Bar 2

    # 6. The pointer on the hidden tab.
    Clear-Highlight $bar
    $start = @(Get-Log).Count
    Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 (Get-TabPoint $bar.Tabs[0]))
    $frames = Assert-Inside $start $bar.Tabs[0].Area 'The pointer on a tab'
    Write-Host "OK: the pointer on a tab draws only that tab ($($frames.Count) frames, $(Measure-Pixels $frames) pixels)."

    # 7. Titles as fast as the program can give them, in the hidden tab, and
    # then the last one again and again.
    Clear-Highlight $bar
    $title = Get-Title
    $titles = Assert-Titles $bar.Tabs[0].Title 'a hidden tab'
    $same = Assert-SameTitle 'a hidden tab'
    if ($same.Wakes -gt $MAX_HIDDEN_SAME_WAKES) {
        throw "While the program of a hidden tab gave the same title $($same.Titles) times in two seconds, the window woke up $($same.Wakes) times per second, more than $($MAX_HIDDEN_SAME_WAKES), though the same title again is not a change."
    }
    if ((Get-Title) -ne $title) {
        throw "The title of a hidden tab reached the window: '$(Get-Title)'."
    }
    Write-Host "OK: while the program of a hidden tab gives $($titles.Titles) titles in two seconds, the window wakes up $($titles.Wakes) times per second, draws only the title of its tab in the bar ($($titles.Frames) frames), and keeps the title of the tab that it shows; while it gives that title again $($same.Titles) times, the window wakes up $($same.Wakes) times per second, and draws nothing."

    # 8. A click on the first tab shows it.
    $point = Get-TabPoint $bar.Tabs[0]
    $start = @(Get-Log).Count
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $point
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $point
        Get-Mouse $WM_LBUTTONUP 0 $point
    )
    Wait-Shown $first @($second) 'After a click on the first tab'
    Wait-For { (Get-Title) -eq 'meiga-tabbar-renamed' } `
    { "The window did not take the title of the tab that it shows: it is '$(Get-Title)' instead of 'meiga-tabbar-renamed'." }
    Wait-Quiet 400
    $full = @(Get-Frames $start | Where-Object { Test-Full $_ })
    if ($full.Count -ne 1) {
        Show-Lines (Get-Frames $start)
        throw "A click on a tab made $($full.Count) full frames instead of 1."
    }
    Write-Host 'OK: a click on a tab shows it with one full frame, and the window takes its title.'

    # 9. A press on the close button of the second tab, which goes up below
    # the bar.
    $close = Get-Middle $bar.Tabs[1].Close
    $away = [pscustomobject]@{ X = $close.X; Y = $bar.Height + 100 }
    $start = @(Get-Log).Count
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $close
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $close
        Get-Mouse $WM_MOUSEMOVE $MK_LBUTTON $away
        Get-Mouse $WM_LBUTTONUP 0 $away
    )
    $null = Assert-Inside $start $bar.Rect 'A press on a close button that goes up outside it'
    if (-not (Get-Process -Id $second -ErrorAction Ignore) -or @(Get-Shells).Count -ne 2) {
        throw 'A press on a close button that went up outside it closed the tab.'
    }
    Write-Host 'OK: a press on a close button that goes up outside it closes nothing, and draws only in the bar.'

    # 10. A middle press on the second tab, and Ctrl+Tab before it goes up,
    # which shows that tab: the bar keeps the mouse until the button goes
    # up, outside the tab. Then Ctrl+Tab shows the first tab again.
    Join-Input
    $point = Get-TabPoint $bar.Tabs[1]
    $away = [pscustomobject]@{ X = $point.X; Y = $bar.Height + 100 }
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $point
        Get-Mouse $WM_MBUTTONDOWN $MK_MBUTTON $point
    )
    Wait-For { (Get-Capture) -eq $window } { 'The window did not capture the mouse when the middle button went down on a tab.' }
    Send-Ctrl 'Tab'
    Wait-Shown $second @($first) 'After Ctrl+Tab with the middle button down on a tab'
    if ((Get-Capture) -ne $window) {
        throw 'The window let the mouse go when Ctrl+Tab showed another tab while the middle button was down on a tab.'
    }
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE $MK_MBUTTON $away
        Get-Mouse $WM_MBUTTONUP 0 $away
    )
    Wait-For { (Get-Capture) -eq [IntPtr]::Zero } { 'The window kept the mouse after the middle button went up.' }
    if (-not (Get-Process -Id $second -ErrorAction Ignore) -or @(Get-Shells).Count -ne 2) {
        throw 'The middle button closed a tab when it went up outside it.'
    }
    Send-Ctrl 'Tab'
    Wait-Shown $first @($second) 'After Ctrl+Tab back to the first tab'
    Split-Input
    Wait-Quiet 400
    Write-Host 'OK: while the middle button is down on a tab, Ctrl+Tab shows another tab, and the bar keeps the mouse until the button goes up: outside the tab, it closes nothing.'

    # 11. The middle button closes the second tab.
    $point = Get-TabPoint $bar.Tabs[1]
    $start = @(Get-Log).Count
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $point
        Get-Mouse $WM_MBUTTONDOWN $MK_MBUTTON $point
        Get-Mouse $WM_MBUTTONUP 0 $point
    )
    Wait-Ended $second 'The program of the tab that the middle button closed'
    Wait-For { (Get-Consoles).Count -eq 1 } { "With one tab left, $exe does not have one console." }
    Wait-Shown $first @() 'After the middle button closed the second tab'
    $frames = Assert-Inside $start $bar.Rect 'The middle button on a tab'
    Write-Host "OK: the middle button closes the tab under it, and its programs end; only the bar is drawn again ($($frames.Count) frames)."

    # 12. A reset of the screen (ESC c) keeps the title of the tab, and the
    # titles that its program saved (ADR 0029): the bar is not drawn, and
    # the titles saved after the reset and before it come back. A title of
    # more than 1000 characters is ignored.
    $bar = Get-Bar 1
    $terminal = New-Rect 0 $bar.Height $bar.Rect.Right $bar.Client.Bottom
    $null = Invoke-Step 'give a title before the reset'
    Wait-For { (Get-Title) -eq 'meiga-tabbar-before' } { "The title of the window is '$(Get-Title)' instead of 'meiga-tabbar-before'." }
    $null = Invoke-Step 'save the title and give another one before the reset'
    Wait-For { (Get-Title) -eq 'meiga-tabbar-reset' } { "The title of the window is '$(Get-Title)' instead of 'meiga-tabbar-reset'." }
    Wait-Quiet 400
    $start = @(Get-Log).Count
    $null = Invoke-Step 'reset the screen'
    $null = Assert-Inside $start $terminal 'A reset of the screen'
    if ((Get-Title) -ne 'meiga-tabbar-reset') {
        throw "After a reset of the screen, the title of the window is '$(Get-Title)' instead of 'meiga-tabbar-reset'."
    }
    $null = Invoke-Step 'save the title and give another one'
    Wait-For { (Get-Title) -eq 'meiga-tabbar-saved' } { "After the reset, the title of the window is '$(Get-Title)' instead of 'meiga-tabbar-saved'." }
    $null = Invoke-Step 'give the title saved after the reset again'
    Wait-For { (Get-Title) -eq 'meiga-tabbar-reset' } { "The title saved after the reset, and given again, is '$(Get-Title)' instead of 'meiga-tabbar-reset'." }
    $null = Invoke-Step 'give the title saved before the reset again'
    Wait-For { (Get-Title) -eq 'meiga-tabbar-before' } { "The title saved before the reset, and given again, is '$(Get-Title)' instead of 'meiga-tabbar-before'." }
    Wait-Quiet 400
    $start = @(Get-Log).Count
    $null = Invoke-Step 'give a title of 1001 characters'
    $null = Assert-Inside $start $terminal 'A title of 1001 characters, and text in the third row'
    if ((Get-Title) -ne 'meiga-tabbar-before') {
        throw "A title of 1001 characters reached the window: its title is $((Get-Title).Length) characters long."
    }
    Write-Host 'OK: a reset of the screen keeps the title of the tab, and draws only the terminal; the titles saved after it and before it come back; and a title of 1001 characters is ignored.'

    # 13. Nothing changes: nothing is drawn.
    $start = @(Get-Log).Count
    Start-Sleep -Seconds 1
    $lines = @(Get-Log | Select-Object -Skip $start)
    if ($lines.Count -gt 0) {
        Show-Lines $lines
        throw 'The window drew while nothing changed.'
    }
    Write-Host 'OK: while nothing changes, nothing is drawn (1 s).'

    # 14. The close button of the last tab closes the window.
    $close = Get-Middle (Get-Bar 1).Tabs[0].Close
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $close
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $close
        Get-Mouse $WM_LBUTTONUP 0 $close
    )
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after the close button of its last tab."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Wait-Ended $first 'The program of the first tab'
    Write-Host 'OK: the close button of the last tab closes the window, and it ends with exit code 0.'
    $passed = $true
}
finally {
    Split-Input
    if ($process) {
        if (-not $process.HasExited) {
            $process.Kill()
        }
        $process.Dispose()
    }
    if ($null -ne $pointer) {
        $null = $api::SetCursorPos($pointer.X, $pointer.Y)
    }
    $null = $api::SetThreadDpiAwarenessContext($previousDpi)
    $go.Dispose()
    $done.Dispose()
    Remove-Item -LiteralPath $script -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $appdata -Recurse -Force -ErrorAction SilentlyContinue
    if ($passed) {
        Remove-Item -LiteralPath $log, $reports -ErrorAction SilentlyContinue
    }
    else {
        foreach ($file in $log, $reports) {
            if (Test-Path -LiteralPath $file) { Write-Host "Kept $file" }
        }
    }
}
