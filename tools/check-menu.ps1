<#
.SYNOPSIS
Checks the menu of a tab of meiga.exe, with the mouse and with the keyboard:
where it opens, what each change draws, what its items do, and the keys that
it keeps from the program.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and, in its first tab, a
PowerShell script that does one step at a time, when this check tells it to,
and reports it in a file. The copies of the tab run the same script, with the
same arguments: they only give their tab a title, tell it in a file of their
own, and wait until their tab closes.
The check posts mouse and key messages to the window, at the places of the
layouts of the bar and of the menu, which it computes as meiga does (ADR 0020
and ADR 0021): only the width of the menu, which depends on the font, comes
from the frame that opens it. It checks that:

- the Menu key opens the menu of the tab that is shown, below its tab, and
  the frame draws only the menu; while it is open, nothing is drawn when
  nothing changes, nor when the program writes under it, and what it writes
  across its side is drawn; Esc closes it, also with a button down in it,
  and then the window lets the mouse go, and the frame draws again what it
  covered;
- F10 alone goes to the program, and Shift+F10 opens the menu. While it is
  open, Down moves the highlight, which draws only the highlights of the
  two items, and X does nothing. The program gets none of those keys,
  neither going down nor going up, but it gets Shift going up, which went
  down for it before the menu opened; and Ctrl, which goes down for it
  before the right button opens the menu, and repeats while it is open:
  the program gets it going up;
- Enter chooses the highlighted item: "Duplicate tab", the first one, opens
  a tab with the same command line, which is shown with one full frame;
- the right button opens the menu of the tab where it goes up, at the
  pointer, without showing the tab; the pointer on an item draws only its
  highlight; a click on "Duplicate tab" puts the copy right after the tab,
  and not at the end, and the window lets the mouse go;
- a click on a disabled item, "Restart tab" while the program of the tab
  runs, or "Close tabs to the right" on the last tab, does nothing, and the
  pointer on it draws nothing; a press outside the menu closes it and does
  nothing else: on another tab, it does not show it;
- "Close tabs to the right", on the tab in the middle, closes the last one,
  and the first one stays; its program ends, and the tab of the menu is
  shown, with one full frame, because the one that was shown closed;
- the menu closes when the window changes size, and when it is minimized:
  then Esc, which would close it, reaches the program;
- "Close tab", on a tab that is hidden, closes it, and draws only the bar
  and what the menu covered: the tab that is shown stays. "Close other
  tabs", on the last tab, closes the first one, on its left: its program
  ends, and the tab of the menu is shown, with one full frame;
- with one tab, the arrows skip the separator and the disabled items, and
  go round, and "Close tab" closes the window: meiga.exe ends with exit
  code 0.

As check-tabbar.ps1 does, the check moves the pointer out of the window,
posts its messages while the thread of the window is suspended, and does not
count on the highlights that the pointer makes staying. It types each key
with the key state that it needs, which it shares with the thread of the
window (see check-tabs.ps1).

It needs PowerShell 7 (pwsh) in the PATH, for the script in the terminal.

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

if (-not ('Meiga.Tools.MenuCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name MenuCheck -MemberDefinition @'
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
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetWindowRect(IntPtr window, out RECT rect);

[DllImport("user32.dll")]
public static extern uint GetDpiForWindow(IntPtr window);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ShowWindowAsync(IntPtr window, int command);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsIconic(IntPtr window);

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
$api = [Meiga.Tools.MenuCheck]

$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_SYSKEYDOWN = 0x0104
$WM_SYSKEYUP = 0x0105
$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_RBUTTONDOWN = 0x0204
$WM_RBUTTONUP = 0x0205
$MK_LBUTTON = 0x0001
$MK_RBUTTON = 0x0002
$MAPVK_VK_TO_VSC = 0
$VK_RETURN = 0x0D
$VK_SHIFT = 0x10
$VK_CONTROL = 0x11
$VK_ESCAPE = 0x1B
$VK_UP = 0x26
$VK_DOWN = 0x28
$VK_X = 0x58
$VK_Z = 0x5A
$VK_APPS = 0x5D
$VK_F10 = 0x79
$VK_LSHIFT = 0xA0
$VK_LCONTROL = 0xA2
# The keys that Shift sets down: Shift and the left Shift. And the ones of
# Ctrl.
$shiftKeys = $VK_SHIFT, $VK_LSHIFT
$ctrlKeys = $VK_CONTROL, $VK_LCONTROL
$SW_MINIMIZE = 6
$SW_RESTORE = 9
# SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS
$SWP_RESIZE_ONLY = 0x0002 -bor 0x0004 -bor 0x0010 -bor 0x4000
$THREAD_PRIORITY_NORMAL = 0
$THREAD_PRIORITY_BELOW_NORMAL = -1
# The console of each tab: the one of the ConPTY of meiga, or the one of
# Windows if the ConPTY is not next to meiga.exe.
$consoles = 'OpenConsole.exe', 'conhost.exe'

# The script of the first tab. Each time the check sets the event
# <Name>-go, it does its next step and reports it in $Log, and sets
# <Name>-done. Its copies run it with the same arguments.
$program = @'
param([Parameter(Mandatory)][string]$Name, [Parameter(Mandatory)][string]$Log)
$ErrorActionPreference = 'Stop'
$e = [char]27
$out = [Console]::Out
# A copy of the first tab gives its tab a title, tells it in a file of its
# own, and waits until its tab closes.
$created = $false
$first = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, "$Name-first", [ref]$created)
if (-not $created) {
    $out.Write("$e[?25l$e]0;meiga-menu-copy$e\")
    [IO.File]::WriteAllText("$Log.$PID", 'copy')
    $null = $first.WaitOne()
    exit
}
$go = [Threading.EventWaitHandle]::OpenExisting("$Name-go")
$done = [Threading.EventWaitHandle]::OpenExisting("$Name-done")
Add-Type -Namespace MeigaCheck -Name Console -MemberDefinition @"
[DllImport("kernel32.dll")] public static extern System.IntPtr GetStdHandle(int handle);
[DllImport("kernel32.dll")] public static extern bool ReadConsoleInputW(System.IntPtr handle, [System.Runtime.InteropServices.Out] byte[] records, uint count, out uint read);
"@
$in = [MeigaCheck.Console]::GetStdHandle(-10)
# Up to 64 INPUT_RECORD of 20 bytes: the type of the event in the first two
# and, for a key, whether it goes down from byte 4, and its virtual-key code
# from byte 10.
$records = [byte[]]::new(20 * 64)
# The keys that reach the program until Z goes up: the virtual-key code of
# each one, with + when it goes down and - when it goes up.
function Read-Keys {
    $keys = [Collections.Generic.List[string]]::new()
    while ($true) {
        $read = 0
        $null = [MeigaCheck.Console]::ReadConsoleInputW($in, $records, 64, [ref]$read)
        for ($at = 0; $at -lt 20 * $read; $at += 20) {
            if ([BitConverter]::ToUInt16($records, $at) -ne 1) { continue }
            $down = [BitConverter]::ToInt32($records, $at + 4) -ne 0
            $key = [BitConverter]::ToUInt16($records, $at + 10)
            $keys.Add("$key$(if ($down) { '+' } else { '-' })")
            if ($key -eq 0x5A -and -not $down) { return $keys -join ' ' }
        }
    }
}
$steps = @(
    # Hide the cursor, so that it never blinks, give the tab a title, and
    # tell the process.
    { $out.Write("$e[?25l$e]0;meiga-menu-1$e\$e[2J$e[H"); "ready $PID" },
    # Write on the third row, from the third column: under the menu that
    # opens below the first tab.
    { $out.Write("$e[3;3Hhidden"); 'wrote' },
    # And on the fourth one, from the third column, across its right side.
    { $out.Write("$e[4;3H" + ('across' * 6)); 'wrote' },
    { "keys $(Read-Keys)" },
    { "keys $(Read-Keys)" },
    { "keys $(Read-Keys)" },
    { "keys $(Read-Keys)" }
)
foreach ($step in $steps) {
    if (-not $go.WaitOne(60000)) { exit 1 }
    [IO.File]::AppendAllText($Log, "$(& $step)`n")
    $null = $done.Set()
}
# Until the tab closes, which ends this script.
$null = $go.WaitOne()
'@

$name = "meiga-check-menu-$PID"
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
function Wait-Quiet([int]$Milliseconds) {
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
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
            throw "$exe kept drawing for $TimeoutSeconds s."
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

# Whether $Inner is inside $Outer.
function Test-Inside($Inner, $Outer) {
    $Inner.Left -ge $Outer.Left -and $Inner.Top -ge $Outer.Top -and
    $Inner.Right -le $Outer.Right -and $Inner.Bottom -le $Outer.Bottom
}

# The smallest rectangle around $Rects.
function Get-Union($Rects) {
    $all = @($Rects)
    New-Rect ($all | Measure-Object -Property Left -Minimum).Minimum ($all | Measure-Object -Property Top -Minimum).Minimum `
    ($all | Measure-Object -Property Right -Maximum).Maximum ($all | Measure-Object -Property Bottom -Maximum).Maximum
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
    $outside = @(Get-Areas $frames | Where-Object { -not (Test-Inside $_ $Rect) })
    if ($frames.Count -eq 0 -or $outside.Count -gt 0) {
        Show-Lines $frames
        throw "$What`: the window did not draw only inside $(Show-Rect $Rect)."
    }
    $frames
}

# Waits until meiga stops drawing, and checks that the frames after the first
# $From lines of the log presented $Rect, and nothing else; otherwise throws
# $What.
function Assert-Only([int]$From, $Rect, [string]$What) {
    Wait-Quiet 400
    $frames = @(Get-Frames $From)
    $areas = @(Get-Areas $frames)
    if ($areas.Count -ne 1 -or (Show-Rect $areas[0]) -ne (Show-Rect $Rect)) {
        Show-Lines $frames
        throw "$What`: the window did not draw only $(Show-Rect $Rect)."
    }
}

# Waits until meiga stops drawing, and checks that it drew nothing after the
# first $From lines of the log; otherwise throws $What.
function Assert-Nothing([int]$From, [string]$What) {
    Wait-Quiet 400
    $lines = @(Get-Log | Select-Object -Skip $From)
    if ($lines.Count -gt 0) {
        Show-Lines $lines
        throw "$What`: the window drew."
    }
}

# Waits until meiga stops drawing, and checks that it drew one full frame
# after the first $From lines of the log; otherwise throws $What.
function Assert-OneFull([int]$From, [string]$What) {
    Wait-Quiet 400
    $full = @(Get-Frames $From | Where-Object { Test-Full $_ })
    if ($full.Count -ne 1) {
        Show-Lines (Get-Frames $From)
        throw "$What made $($full.Count) full frames instead of 1."
    }
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

# Waits until a tab opens with a program that is not one of $Known, and
# checks that it runs the same command line as the first tab: it is a copy.
# Returns its process.
function Wait-Copy([int[]]$Known, [string]$What) {
    $new = { @(Get-Shells | Where-Object { [int]$_.ProcessId -notin $Known }) }
    Wait-For { @(& $new).Count -eq 1 } { "$What did not open a tab with a program." }
    $copy = @(& $new)[0]
    $original = @(Get-Shells | Where-Object { [int]$_.ProcessId -eq $first })[0].CommandLine
    if ($copy.CommandLine -ne $original) {
        throw "$What opened a tab with the command line '$($copy.CommandLine)' instead of '$original'."
    }
    [int]$copy.ProcessId
}

# Waits until the program of the copy $Id, in the tab that is shown, has
# given its tab its title, and the window has taken it and drawn it.
function Wait-CopyReady([int]$Id, [string]$What) {
    Wait-For { Test-Path -LiteralPath "$reports.$Id" } { "$What`: the program of the copy did not start." }
    Wait-Title 'meiga-menu-copy' $What
    Wait-Quiet 400
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

function Get-Title {
    $process.Refresh()
    $process.MainWindowTitle
}

# Waits until the title of the window is $Title.
function Wait-Title([string]$Title, [string]$What) {
    Wait-For { (Get-Title) -eq $Title } { "$What`: the title of the window is '$(Get-Title)' instead of '$Title'." }
}

# If the pointer is over the window, moves it out, above the window: the
# moves that Windows makes at its place would reach the bar.
function Move-Pointer {
    $point = [Meiga.Tools.MenuCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.MenuCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = [Meiga.Tools.MenuCheck+RECT]::new()
    if (-not $api::ScreenToClient($window, [ref]$inside) -or -not $api::GetClientRect($window, [ref]$rect) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = [Meiga.Tools.MenuCheck+POINT]::new()
    $title.X = [Math]::Floor($rect.Right / 2)
    $title.Y = -8
    $null = $api::ClientToScreen($window, [ref]$title)
    if (-not $api::SetCursorPos($title.X, $title.Y)) {
        throw "Could not move the pointer out of the window: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# The client area of the window.
function Get-Client {
    $client = [Meiga.Tools.MenuCheck+RECT]::new()
    if (-not $api::GetClientRect($window, [ref]$client)) {
        throw "Could not read the size of the window of $exe."
    }
    $client
}

# `$Pixels` at a scale of 100 %, at the scale of the window, as meiga does:
# rounded, and at least one.
function Get-Scaled([int]$Pixels) {
    [int][Math]::Max(1, [Math]::Floor(($Pixels * $dpi + 48) / 96))
}

# The layout of the bar with $Count tabs, as meiga computes it (see
# crates/meiga-core/src/ui/tabbar.rs).
function Get-Bar([int]$Count) {
    $client = Get-Client
    $height = Get-Scaled 32
    $gap = Get-Scaled 4
    $padding = Get-Scaled 8
    $closeSide = Get-Scaled 24
    $newSide = Get-Scaled 28
    $profilesSide = Get-Scaled 24
    $width = [int][Math]::Min((Get-Scaled 240), [Math]::Floor([Math]::Max(0, $client.Right - 3 * $gap - $newSide - $profilesSide) / $Count))
    $tabs = foreach ($index in 0..($Count - 1)) {
        $left = $gap + $index * $width
        $right = $left + $width
        $close = New-Rect ($right - $gap - $closeSide) 0 ($right - $gap) 0
        [pscustomobject]@{
            Rect = New-Rect $left $gap $right $height
            Title = New-Rect ($left + $padding) $gap $close.Left $height
        }
    }
    [pscustomobject]@{
        Rect = New-Rect 0 0 $client.Right $height
        Height = $height
        Tabs = @($tabs)
    }
}

# The sizes of the menu of a tab, as meiga computes them (see
# crates/meiga-core/src/ui/menu.rs): five items and a separator, between a
# border with some space inside it. Its width depends on the font, and
# comes from the frame that opens it.
function Get-MenuSizes {
    $border = Get-Scaled 1
    $padding = Get-Scaled 2
    $side = Get-Scaled 4
    $gap = Get-Scaled 2
    $item = (Get-Scaled 28) + 2 * $gap
    $separator = (Get-Scaled 1) + 2 * (Get-Scaled 1)
    [pscustomobject]@{
        Border = $border
        Padding = $padding
        Side = $side
        Gap = $gap
        Item = $item
        Separator = $separator
        Height = 2 * ($border + $padding) + 5 * $item + $separator
        # Beside the text: the border, the space between it and the
        # highlight, and the one between the highlight and the text, on
        # each side.
        Sides = 2 * ($border + $side + (Get-Scaled 11))
    }
}

# The menu of a tab, as meiga places it: with its top-left corner at $Left,
# $Top or, where it does not fit, its right side at $OtherRight or its
# bottom at $OtherBottom; and never outside the client area.
function Get-Menu([int]$Left, [int]$Top, [int]$OtherRight, [int]$OtherBottom) {
    $client = Get-Client
    $width = [Math]::Min($menuWidth, $client.Right)
    $height = $sizes.Height
    $x = if ($Left + $width -gt $client.Right) { $OtherRight - $width } else { $Left }
    $y = if ($Top + $height -gt $client.Bottom) { $OtherBottom - $height } else { $Top }
    $x = [Math]::Max(0, [Math]::Min($x, $client.Right - $width))
    $y = [Math]::Max(0, [Math]::Min($y, $client.Bottom - $height))
    New-Rect $x $y ($x + $width) ($y + $height)
}

# The menu of a tab that the right button opens at $Point.
function Get-MenuAt($Point) {
    Get-Menu $Point.X $Point.Y $Point.X $Point.Y
}

# The entries of $Menu: Duplicate tab, Restart tab, the separator, Close
# tab, Close other tabs and Close tabs to the right. Each one with a point
# in the middle of it, X and Y, and, for an item, its highlight, without the
# space on its sides and above and below it. Restart tab is disabled while
# the program of the tab runs, as in all this check.
function Get-Entries($Menu) {
    $top = $Menu.Top + $sizes.Border + $sizes.Padding
    $left = $Menu.Left + $sizes.Border + $sizes.Side
    $right = $Menu.Right - $sizes.Border - $sizes.Side
    $x = [Math]::Floor(($Menu.Left + $Menu.Right) / 2)
    foreach ($height in $sizes.Item, $sizes.Item, $sizes.Separator, $sizes.Item, $sizes.Item, $sizes.Item) {
        [pscustomobject]@{
            X = $x
            Y = $top + [Math]::Floor($height / 2)
            Highlight = New-Rect $left ($top + $sizes.Gap) $right ($top + $height - $sizes.Gap)
        }
        $top += $height
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

# A click of $Button at $Point: the pointer moves there, and the button goes
# down and up.
function Send-Click($Point, [string]$Button = 'Left') {
    $down, $up, $keys = if ($Button -eq 'Right') {
        $WM_RBUTTONDOWN, $WM_RBUTTONUP, $MK_RBUTTON
    }
    else {
        $WM_LBUTTONDOWN, $WM_LBUTTONUP, $MK_LBUTTON
    }
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $Point
        Get-Mouse $down $keys $Point
        Get-Mouse $up 0 $Point
    )
}

# The window that has the mouse captured in the thread of the window of
# meiga, or zero.
function Get-Capture {
    $api::CaptureOf($thread)
}

# Shares the input of this thread with the one of the window, whose key
# state Send-Keys sets: SetKeyboardState sets the key state of the thread
# that calls it. This thread needs a message queue, which PeekMessageW
# creates.
function Join-Input {
    $message = [Meiga.Tools.MenuCheck+MSG]::new()
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

# The message of $Code going down, or up: the message, its wParam and its
# lParam. With $Repeat, it goes down again, as Windows repeats a key that
# stays down. F10 goes as a system key, as Windows sends it.
function Get-Key([int]$Code, [bool]$Down, [bool]$Repeat) {
    $scan = $api::MapVirtualKeyExW($Code, $MAPVK_VK_TO_VSC, $api::GetKeyboardLayout($thread))
    [long]$bits = 1 -bor ($scan -shl 16)
    if ($Down) {
        # The key was down (bit 30).
        if ($Repeat) { $bits = $bits -bor 0x40000000L }
        $message = if ($Code -eq $VK_F10) { $WM_SYSKEYDOWN } else { $WM_KEYDOWN }
    }
    else {
        # The key was down (bit 30), and goes up (bit 31).
        $bits = $bits -bor 0xC0000000L
        $message = if ($Code -eq $VK_F10) { $WM_SYSKEYUP } else { $WM_KEYUP }
    }
    $message, $Code, $bits
}

# Posts key messages at once, with the keys of $Down down and no other one
# (see PostAll). The input has to be shared with the thread of the window:
# see Join-Input.
function Send-Keys([long[]]$Messages, [int[]]$Down = @()) {
    $state = [byte[]]::new(256)
    foreach ($key in $Down) { $state[$key] = 0x80 }
    $api::PostAll($window, $thread, $Messages, $state)
}

# Types $Code: it goes down and up, with the keys of $Down down.
function Send-Press([int]$Code, [int[]]$Down = @()) {
    Send-Keys @((Get-Key $Code $true) + (Get-Key $Code $false)) $Down
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
    $dpi = $api::GetDpiForWindow($window)
    $sizes = Get-MenuSizes
    Join-Input

    # 1. The first tab, alone.
    $first = [int]((Invoke-Step 'start') -split ' ')[1]
    Wait-Title 'meiga-menu-1' 'With one tab'
    Wait-Shown $first @() 'With one tab'
    Move-Pointer
    Wait-Quiet 400
    $bar = Get-Bar 1

    # 2. The Menu key opens the menu of the tab that is shown, below its
    # tab. Its longest item, Close tabs to the right, has 23 characters.
    # While it is open, what the program writes under it is not drawn, and
    # what it writes across its side is. Esc closes it, also with a button
    # down in it: then the window lets the mouse go.
    $start = @(Get-Log).Count
    Send-Press $VK_APPS
    Wait-Quiet 400
    $frames = @(Get-Frames $start)
    $areas = @(Get-Areas $frames)
    if ($areas.Count -ne 1) {
        Show-Lines $frames
        throw "The Menu key drew $($areas.Count) areas instead of the menu."
    }
    $menuWidth = $areas[0].Right - $areas[0].Left
    $cell = ($menuWidth - $sizes.Sides) / 23
    $tab = $bar.Tabs[0].Rect
    $menu = Get-Menu $tab.Left $tab.Bottom $tab.Right $tab.Top
    if ((Show-Rect $areas[0]) -ne (Show-Rect $menu) -or $cell -lt 1 -or $cell -ne [Math]::Floor($cell)) {
        throw "The Menu key drew $(Show-Rect $areas[0]), and the menu has to be $(Show-Rect $menu): below the tab, $($sizes.Height) pixels high, and as wide as 23 cells and $($sizes.Sides) pixels."
    }
    $start = @(Get-Log).Count
    Start-Sleep -Seconds 1
    Assert-Nothing $start 'While the menu was open and nothing changed'
    $start = @(Get-Log).Count
    $null = Invoke-Step 'write under the menu'
    Assert-Nothing $start 'While the menu was open, the program wrote under it'
    $start = @(Get-Log).Count
    $null = Invoke-Step 'write across the side of the menu'
    Wait-For { @(Get-Frames $start).Count -gt 0 } { 'While the menu was open, the program wrote across its right side, and the window drew nothing.' }
    Wait-Quiet 400
    $frames = @(Get-Frames $start)
    $union = Get-Union @(Get-Areas $frames)
    if ($union.Right -le $menu.Right) {
        Show-Lines $frames
        throw "While the menu was open, the program wrote across its right side, and the window drew only $(Show-Rect $union), and nothing on the right of the menu, $(Show-Rect $menu)."
    }
    $entries = @(Get-Entries $menu)
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $entries[3]
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $entries[3]
    )
    Wait-For { (Get-Capture) -ne [IntPtr]::Zero } { 'A press on an item of the menu did not capture the mouse.' }
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Press $VK_ESCAPE
    Assert-Only $start $menu 'Esc on the menu, with a button down in it'
    if ((Get-Capture) -ne [IntPtr]::Zero) {
        throw 'Esc closed the menu while a button was down in it, and the window kept the mouse.'
    }
    Send-Mouse (Get-Mouse $WM_LBUTTONUP 0 $entries[3])
    Write-Host "OK: the Menu key opens the menu of the tab that is shown, below its tab ($menuWidth x $($sizes.Height) pixels, with cells $cell pixels wide), and draws only the menu; while it is open, nothing is drawn when nothing changes (1 s), nor when the program writes under it, and what it writes across its side is drawn; Esc closes it, also with a button down in it, and then the window lets the mouse go, and draws again what it covered."

    # 3. F10 alone goes to the program. Then Shift goes down for the
    # program, F10 opens the menu, and Shift goes up. While the menu is
    # open, Down, X and Esc are for it. Then Ctrl goes down for the
    # program, and the right button opens the menu of the tab: Ctrl repeats,
    # which does nothing, and goes up for the program. Esc closes the menu,
    # and Z goes to the program.
    Send-Press $VK_F10
    Send-Press $VK_Z
    $keys = (Invoke-Step 'read the keys') -replace '^keys ', ''
    if ($keys -ne '121+ 121- 90+ 90-') {
        throw "The program got the keys '$keys' instead of '121+ 121- 90+ 90-': F10 alone, and Z."
    }
    $start = @(Get-Log).Count
    Send-Keys @(Get-Key $VK_SHIFT $true; Get-Key $VK_F10 $true; Get-Key $VK_F10 $false) $shiftKeys
    Assert-Only $start $menu 'Shift+F10'
    Send-Keys (Get-Key $VK_SHIFT $false)
    $start = @(Get-Log).Count
    Send-Press $VK_DOWN
    $null = Assert-Inside $start (Get-Union @($entries[0].Highlight, $entries[3].Highlight)) 'Down on the menu'
    $start = @(Get-Log).Count
    Send-Press $VK_X
    Assert-Nothing $start 'X on the menu'
    $start = @(Get-Log).Count
    Send-Press $VK_ESCAPE
    Assert-Only $start $menu 'Esc on the menu'
    $point = Get-TabPoint $bar.Tabs[0]
    Send-Keys (Get-Key $VK_CONTROL $true) $ctrlKeys
    Send-Click $point 'Right'
    Wait-Quiet 400
    # The repeat does the same with any key state: nothing has to wait
    # until the window takes it.
    Send-Keys (Get-Key $VK_CONTROL $true $true) $ctrlKeys
    Send-Keys (Get-Key $VK_CONTROL $false)
    $start = @(Get-Log).Count
    Send-Press $VK_ESCAPE
    Assert-Only $start (Get-MenuAt $point) 'Esc on the menu of the right button'
    Send-Press $VK_Z
    $keys = (Invoke-Step 'read the keys') -replace '^keys ', ''
    if ($keys -ne '16+ 16- 17+ 17- 90+ 90-') {
        throw "The program got the keys '$keys' instead of '16+ 16- 17+ 17- 90+ 90-': Shift and Ctrl, which went down for it before the menu opened, and Z, after the menu closed."
    }
    Write-Host 'OK: F10 alone goes to the program, and Shift+F10 opens the menu; while it is open, Down draws only the highlights of the two items, and X draws nothing; the program gets none of those keys, neither going down nor going up, and it gets Shift going up, which went down for it before, and Ctrl going up, which went down for it before the right button opened the menu, and repeated while it was open.'

    # 4. Enter on the first item, which the keyboard highlights: Duplicate
    # tab.
    Send-Press $VK_APPS
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Press $VK_RETURN
    $copy1 = Wait-Copy @($first) 'Enter on Duplicate tab'
    Wait-Shown $copy1 @($first) 'After Enter on Duplicate tab'
    Wait-CopyReady $copy1 'After Enter on Duplicate tab'
    Assert-OneFull $start 'Enter on Duplicate tab'
    Write-Host 'OK: Enter chooses the item that is highlighted, Duplicate tab when the keyboard opens the menu, which opens a tab with the same command line, shown with one full frame.'

    # 5. The right button on the first tab, which is hidden: its menu opens
    # at the pointer, without showing the tab, and the pointer on its items
    # draws only their highlights. Its copy goes right after it.
    $bar = Get-Bar 2
    $point = Get-TabPoint $bar.Tabs[0]
    $menu = Get-MenuAt $point
    $start = @(Get-Log).Count
    Send-Click $point 'Right'
    Wait-Quiet 400
    $areas = @(Get-Areas (Get-Frames $start))
    $reach = New-Rect 0 0 $bar.Rect.Right $menu.Bottom
    if ($areas.Count -eq 0 -or -not (Test-Inside $menu (Get-Union $areas)) -or
        @($areas | Where-Object { -not (Test-Inside $_ $reach) }).Count -gt 0) {
        Show-Lines (Get-Frames $start)
        throw "The right button on a tab did not draw its menu at $(Show-Rect $menu), and only there and in the bar."
    }
    if ((Get-Priority $first) -ne $THREAD_PRIORITY_BELOW_NORMAL -or (Get-Title) -ne 'meiga-menu-copy') {
        throw 'The right button showed the tab whose menu it opened.'
    }
    $entries = @(Get-Entries $menu)
    $start = @(Get-Log).Count
    Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 $entries[0])
    $null = Assert-Inside $start $entries[0].Highlight 'The pointer on Duplicate tab'
    $start = @(Get-Log).Count
    Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 $entries[3])
    $null = Assert-Inside $start $entries[3].Highlight 'The pointer on Close tab'
    Send-Click $entries[0]
    $copy2 = Wait-Copy @($first, $copy1) 'A click on Duplicate tab'
    Wait-Shown $copy2 @($first, $copy1) 'After a click on Duplicate tab'
    Wait-For { (Get-Capture) -eq [IntPtr]::Zero } { 'The window kept the mouse after a click on an item of the menu.' }
    Wait-CopyReady $copy2 'After a click on Duplicate tab'
    $bar = Get-Bar 3
    Send-Click (Get-TabPoint $bar.Tabs[2])
    Wait-Shown $copy1 @($first, $copy2) 'After a click on the last tab'
    Wait-Quiet 400
    Write-Host 'OK: the right button opens the menu of the tab where it goes up, at the pointer, without showing the tab, and the pointer on an item draws only its highlight; a click on Duplicate tab puts the copy right after the tab, shows it, and lets the mouse go.'

    # 6. The menu of the last tab: Restart tab, while its program runs, and
    # Close tabs to the right are disabled. The pointer on them highlights
    # nothing, and a click on them does nothing. Then a press on another tab
    # closes the menu, and does not show the tab.
    $point = Get-TabPoint $bar.Tabs[2]
    $menu = Get-MenuAt $point
    Send-Click $point 'Right'
    Wait-Quiet 400
    $entries = @(Get-Entries $menu)
    foreach ($disabled in $entries[1], $entries[5]) {
        $start = @(Get-Log).Count
        Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 $disabled)
        Assert-Nothing $start 'The pointer on a disabled item'
        Send-Click $disabled
        Wait-Quiet 400
    }
    if (@(Get-Shells).Count -ne 3) {
        throw 'A click on Restart tab, disabled while the program of the tab ran, or on Close tabs to the right, disabled on the last tab, closed a tab or started a program.'
    }
    $start = @(Get-Log).Count
    Send-Click (Get-TabPoint $bar.Tabs[1])
    Wait-Quiet 400
    $frames = @(Get-Frames $start)
    $areas = @(Get-Areas $frames)
    if ($areas.Count -eq 0 -or -not (Test-Inside $menu (Get-Union $areas)) -or
        @($frames | Where-Object { Test-Full $_ }).Count -gt 0) {
        Show-Lines $frames
        throw "The press that closed the menu did not draw again only what the menu covered, $(Show-Rect $menu), and the bar."
    }
    if ((Get-Priority $copy1) -ne $THREAD_PRIORITY_NORMAL -or (Get-Priority $copy2) -ne $THREAD_PRIORITY_BELOW_NORMAL) {
        throw 'A press on another tab showed it: the menu was not open any more, or did not take the press.'
    }
    Write-Host 'OK: the pointer on a disabled item, Restart tab while the program of the tab runs, or Close tabs to the right on the last tab, draws nothing, and a click on it does nothing; a press outside the menu closes it and does nothing else: on another tab, it does not show it.'

    # 7. Close tabs to the right, on the tab in the middle, which is hidden:
    # the last tab, which is shown, closes, and the tab of the menu is
    # shown. The first one stays.
    $point = Get-TabPoint $bar.Tabs[1]
    $entries = @(Get-Entries (Get-MenuAt $point))
    Send-Click $point 'Right'
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Click $entries[5]
    Wait-Ended $copy1 'The program of the last tab, after Close tabs to the right'
    Wait-For { @(Get-Consoles).Count -eq 2 } { "With two tabs left, $exe does not have two consoles." }
    Wait-Shown $copy2 @($first) 'After Close tabs to the right'
    Assert-OneFull $start 'Close tabs to the right'
    Write-Host 'OK: Close tabs to the right closes the tabs on the right of the tab, and their programs end; the tab of the menu is shown, with one full frame, when the one that was shown closes.'

    # 8. The menu closes when the window changes size: Esc, which would
    # close it, reaches the program then. The program of the first tab
    # reads the keys: a click shows it again.
    $bar = Get-Bar 2
    Send-Click (Get-TabPoint $bar.Tabs[0])
    Wait-Shown $first @($copy2) 'After a click on the first tab'
    Wait-Title 'meiga-menu-1' 'After a click on the first tab'
    Wait-Quiet 400
    Send-Press $VK_APPS
    Wait-Quiet 400
    $rect = [Meiga.Tools.MenuCheck+RECT]::new()
    if (-not $api::GetWindowRect($window, [ref]$rect)) {
        throw "Could not read the size of the window of $exe."
    }
    $start = @(Get-Log).Count
    $width = $rect.Right - $rect.Left - 80
    $height = $rect.Bottom - $rect.Top - 60
    if (-not $api::SetWindowPos($window, [IntPtr]::Zero, 0, 0, $width, $height, $SWP_RESIZE_ONLY)) {
        throw 'SetWindowPos could not change the size of the window.'
    }
    Wait-For { @(Get-Log | Select-Object -Skip $start | Where-Object kind -EQ 'compose').Count -gt 0 } `
    { 'The window was not composed at its new size.' }
    Wait-Quiet 400
    Send-Press $VK_ESCAPE
    Send-Press $VK_Z
    $keys = (Invoke-Step 'read the keys') -replace '^keys ', ''
    if ($keys -ne '27+ 27- 90+ 90-') {
        throw "After the window changed size, the program got the keys '$keys' instead of '27+ 27- 90+ 90-': the menu did not close."
    }
    Write-Host 'OK: the menu closes when the window changes size: then Esc reaches the program.'

    # 9. The same when the window is minimized.
    Send-Press $VK_APPS
    Wait-Quiet 400
    $null = $api::ShowWindowAsync($window, $SW_MINIMIZE)
    Wait-For { $api::IsIconic($window) } { 'The window was not minimized.' }
    $start = @(Get-Log).Count
    $null = $api::ShowWindowAsync($window, $SW_RESTORE)
    Wait-For { -not $api::IsIconic($window) -and @(Get-Log | Select-Object -Skip $start | Where-Object kind -EQ 'paint').Count -gt 0 } `
    { 'The window was not restored.' }
    Wait-Quiet 400
    Send-Press $VK_ESCAPE
    Send-Press $VK_Z
    $keys = (Invoke-Step 'read the keys') -replace '^keys ', ''
    if ($keys -ne '27+ 27- 90+ 90-') {
        throw "After the window was minimized and restored, the program got the keys '$keys' instead of '27+ 27- 90+ 90-': the menu did not close."
    }
    Write-Host 'OK: the menu closes when the window is minimized: once it is restored, Esc reaches the program.'

    # 10. Close tab, on a tab that is hidden: only it closes, and the one
    # that is shown stays. The keyboard duplicates the first tab, and a
    # click shows the first tab again. Then Close other tabs, on the copy,
    # which is hidden: the first tab, on its left, closes, and the copy is
    # shown.
    Send-Press $VK_APPS
    Wait-Quiet 400
    Send-Press $VK_RETURN
    $copy3 = Wait-Copy @($first, $copy2) 'Enter on Duplicate tab, on the first tab'
    Wait-Shown $copy3 @($first, $copy2) 'After Enter on Duplicate tab, on the first tab'
    Wait-CopyReady $copy3 'After Enter on Duplicate tab, on the first tab'
    $bar = Get-Bar 3
    Send-Click (Get-TabPoint $bar.Tabs[0])
    Wait-Shown $first @($copy3, $copy2) 'After a click on the first tab'
    Wait-Title 'meiga-menu-1' 'After a click on the first tab'
    Wait-Quiet 400
    $point = Get-TabPoint $bar.Tabs[1]
    $menu = Get-MenuAt $point
    $entries = @(Get-Entries $menu)
    Send-Click $point 'Right'
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Click $entries[3]
    Wait-Ended $copy3 'The program of the tab of the menu, after Close tab'
    Wait-For { @(Get-Consoles).Count -eq 2 } { "With two tabs left, $exe does not have two consoles." }
    Wait-Shown $first @($copy2) 'After Close tab'
    Wait-Quiet 400
    $frames = @(Get-Frames $start)
    $areas = @(Get-Areas $frames)
    $reach = New-Rect 0 0 $bar.Rect.Right $menu.Bottom
    if ($areas.Count -eq 0 -or -not (Test-Inside $menu (Get-Union $areas)) -or
        @($areas | Where-Object { -not (Test-Inside $_ $reach) }).Count -gt 0) {
        Show-Lines $frames
        throw "Close tab, on a tab that is hidden, did not draw again only what the menu covered, $(Show-Rect $menu), and the bar."
    }
    $bar = Get-Bar 2
    $point = Get-TabPoint $bar.Tabs[1]
    $entries = @(Get-Entries (Get-MenuAt $point))
    Send-Click $point 'Right'
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Click $entries[4]
    Wait-Ended $first 'The program of the first tab, after Close other tabs'
    Wait-For { @(Get-Consoles).Count -eq 1 } { "With one tab left, $exe does not have one console." }
    Wait-Shown $copy2 @() 'After Close other tabs'
    Wait-Title 'meiga-menu-copy' 'After Close other tabs'
    Assert-OneFull $start 'Close other tabs'
    Write-Host 'OK: Close tab, on a tab that is hidden, closes it, and its program ends, and draws only the bar and what the menu covered: the tab that is shown stays; Close other tabs closes all the tabs but the one of the menu, also on its left, and their programs end; the tab of the menu is shown, with one full frame.'

    # 11. With one tab whose program runs, Restart tab, Close other tabs and
    # Close tabs to the right are disabled: Down goes from Duplicate tab,
    # over Restart tab and the separator, to Close tab, and from it, over
    # the disabled items, round to Duplicate tab; then Up goes round, over
    # the disabled items, to Close tab. Arrows that stopped on the separator
    # or on the disabled items, or did not go round, would end on another
    # entry: the separator, Restart tab, or Duplicate tab. Enter on Close
    # tab closes the window.
    Send-Keys @(Get-Key $VK_F10 $true; Get-Key $VK_F10 $false) $shiftKeys
    Wait-Quiet 400
    foreach ($key in $VK_DOWN, $VK_DOWN, $VK_UP) {
        Send-Press $key
    }
    Send-Press $VK_RETURN
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after Close tab on its last tab."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Wait-Ended $copy2 'The program of the last tab'
    Write-Host 'OK: with one tab, the arrows skip the separator and the disabled items, and go round; Close tab closes the window, and it ends with exit code 0.'
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
    Remove-Item -Path "$reports.*" -ErrorAction SilentlyContinue
    if ($passed) {
        Remove-Item -LiteralPath $log, $reports -ErrorAction SilentlyContinue
    }
    else {
        foreach ($file in $log, $reports) {
            if (Test-Path -LiteralPath $file) { Write-Host "Kept $file" }
        }
    }
}
