<#
.SYNOPSIS
Checks the editing of the name of a tab of meiga.exe (ADR 0031): how it
opens, what it draws, the keys and the clicks that it keeps from the
program, and the name that the window and the copies of the tab take.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and, in its first tab, a
PowerShell script that does one step at a time, when this check tells it to,
and reports it in a file. The copies of the tab run the same script, with the
same arguments: they only give their tab a title, tell it in a file of their
own, and wait until the check ends them, or their tab closes.
The check posts mouse and key messages to the window, at the places of the
layout of the bar, which it computes as meiga does (ADR 0020), and the
messages of the focus. It checks that:

- a double click on the title of the tab opens the field of its name there:
  the frames draw only its tab and, while Windows lets the caret blink, it
  blinks there; without the focus it does not, and the editing goes on, and
  with the focus again it blinks again;
- the letters typed and Enter give the tab its name, which the window takes
  as its title; the field closes, and nothing blinks any more; the program
  gets none of those keys, neither going down nor going up;
- Enter on "Rename tab", which the Menu key highlights first in the menu of
  the tab, opens the field: its repeat and Enter going up do nothing, and X
  and Enter give the tab another name;
- Ctrl+Shift+R opens the field too; meanwhile Ctrl+Shift+T opens no tab, and
  Esc closes the field and leaves the name as it was. The program gets Ctrl
  and Shift going up, which went down for it before the field opened, and
  none of the other keys;
- while the left button is down in the field, the window keeps the mouse, and
  then lets it go; a click puts the caret, at the end of the text, and a
  click outside the field, on the terminal, keeps the name;
- the titles that the program gives meanwhile do not change the title of the
  window, nor the name; a name of only spaces gives the tab the title of its
  program back, and Enter without changing the text does not keep it as a
  name: the next title of the program is the title of the window;
- "Duplicate tab" gives the copy the name of the tab, which the window keeps
  as its title when the program of the copy gives its tab a title; a double
  click on the first tab, which is hidden then, shows it and edits its name;
- when the program of the tab whose name is edited ends, its tab closes, and
  the editing with it: the program of the tab that is shown then gets the
  keys again. Ctrl+Shift+W closes the last tab, and meiga.exe ends with exit
  code 0.

As check-menu.ps1 does, the check moves the pointer out of the window, posts
its messages while the thread of the window is suspended, and does not count
on the highlights that the pointer makes staying. It types each key with the
key state that it needs, which it shares with the thread of the window (see
check-tabs.ps1). The program of the first tab hides its cursor, so that only
the caret of the field blinks.

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

if (-not ('Meiga.Tools.RenameCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name RenameCheck -MemberDefinition @'
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
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsZoomed(IntPtr window);

[DllImport("user32.dll")]
public static extern int GetSystemMetrics(int index);

[DllImport("user32.dll")]
public static extern int GetSystemMetricsForDpi(int index, uint dpi);

[DllImport("user32.dll")]
public static extern uint GetDpiForWindow(IntPtr window);

[DllImport("user32.dll")]
public static extern uint GetCaretBlinkTime();

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
$api = [Meiga.Tools.RenameCheck]

$WM_SETFOCUS = 0x0007
$WM_KILLFOCUS = 0x0008
$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$MK_LBUTTON = 0x0001
$MAPVK_VK_TO_VSC = 0
$SM_CARETBLINKINGENABLED = 0x2002
$VK_RETURN = 0x0D
$VK_SHIFT = 0x10
$VK_CONTROL = 0x11
$VK_ESCAPE = 0x1B
$VK_SPACE = 0x20
$VK_DOWN = 0x28
$VK_R = 0x52
$VK_T = 0x54
$VK_W = 0x57
$VK_X = 0x58
$VK_Z = 0x5A
$VK_APPS = 0x5D
$VK_LSHIFT = 0xA0
$VK_LCONTROL = 0xA2
# The keys that Ctrl and Shift set down: Ctrl, Shift, and their left ones.
$ctrlShiftKeys = $VK_CONTROL, $VK_LCONTROL, $VK_SHIFT, $VK_LSHIFT
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
# own, and waits until the check sets the event <Name>-end, or its tab
# closes: then it ends with exit code 0.
$created = $false
$first = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, "$Name-first", [ref]$created)
if (-not $created) {
    $out.Write("$e[?25l$e]0;meiga-rename-copy$e\")
    [IO.File]::WriteAllText("$Log.$PID", 'copy')
    $null = [Threading.EventWaitHandle]::OpenExisting("$Name-end").WaitOne()
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
    { $out.Write("$e[?25l$e]0;meiga-rename-1$e\$e[2J$e[H"); "ready $PID" },
    { "keys $(Read-Keys)" },
    { "keys $(Read-Keys)" },
    { "keys $(Read-Keys)" },
    { "keys $(Read-Keys)" },
    { $out.Write("$e]0;meiga-rename-2$e\"); 'retitled' },
    { $out.Write("$e]0;meiga-rename-3$e\"); 'retitled' },
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

$name = "meiga-check-rename-$PID"
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
$endCopies = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, "$name-end")

# Whether the caret blinks, as meiga reads it (see caret_settings in
# crates/meiga-win/src/window.rs), fast enough for this check: then it
# changes at least once in $blinkWait milliseconds.
$blinkTime = $api::GetCaretBlinkTime()
$blinks = $api::GetSystemMetrics($SM_CARETBLINKINGENABLED) -ne 0 -and $blinkTime -ge 1 -and $blinkTime -le 1000
$blinkWait = if ($blinks) { 2 * $blinkTime + 200 } else { 1200 }

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

# Waits until meiga has written nothing in its log for $Milliseconds. While
# a name is edited, its caret blinks: then it waits between two changes, or
# until the caret stops blinking.
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

# The areas that the frames of $Lines presented: "x y w h;x y w h".
function Get-Areas($Lines) {
    foreach ($line in $Lines) {
        foreach ($area in ($line.areas -split ';' | Where-Object { $_ })) {
            $x, $y, $w, $h = [int[]]($area -split ' ')
            New-Rect $x $y ($x + $w) ($y + $h)
        }
    }
}

# Checks that the frames after the first $From lines of the log, at least
# $Least of them, drew only inside $Rect; otherwise throws $What.
function Assert-Inside([int]$From, $Rect, [int]$Least, [string]$What) {
    $frames = @(Get-Frames $From)
    $outside = @(Get-Areas $frames | Where-Object { -not (Test-Inside $_ $Rect) })
    if ($frames.Count -lt $Least -or $outside.Count -gt 0) {
        Show-Lines $frames
        throw "$What`: the window did not draw $Least frames or more, only inside $(Show-Rect $Rect)."
    }
}

# Checks that the window drew nothing after the first $From lines of the
# log; otherwise throws $What.
function Assert-Nothing([int]$From, [string]$What) {
    $lines = @(Get-Log | Select-Object -Skip $From)
    if ($lines.Count -gt 0) {
        Show-Lines $lines
        throw "$What`: the window drew."
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

# The keys that reached the program of the first tab, until Z went up, which
# the check types now; they have to be $Expected, or it throws $What.
function Assert-Keys([string]$Expected, [string]$What) {
    Send-Press $VK_Z
    $keys = (Invoke-Step 'read the keys') -replace '^keys ', ''
    if ($keys -ne $Expected) {
        throw "$What, the program got the keys '$keys' instead of '$Expected'."
    }
}

# The programs of the tabs: the processes that meiga started, but for the
# consoles. A process that ended before meiga started may have had a
# process with the same number as its parent.
function Get-Shells {
    @(Get-CimInstance Win32_Process -Filter "ParentProcessId = $($process.Id)" |
            Where-Object { $_.Name -notin $consoles -and $_.CreationDate -ge $started })
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

# Waits until meiga stops drawing, and checks that the title of the window
# is still $Title: it did not take another one.
function Assert-Title([string]$Title, [string]$What) {
    Wait-Quiet 400
    if ((Get-Title) -ne $Title) {
        throw "$What`: the title of the window is '$(Get-Title)' instead of '$Title'."
    }
}

# If the pointer is over the window, moves it out, above the window: the
# moves that Windows makes at its place would reach the bar.
function Move-Pointer {
    $point = [Meiga.Tools.RenameCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.RenameCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = [Meiga.Tools.RenameCheck+RECT]::new()
    if (-not $api::ScreenToClient($window, [ref]$inside) -or -not $api::GetClientRect($window, [ref]$rect) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = [Meiga.Tools.RenameCheck+POINT]::new()
    $title.X = [Math]::Floor($rect.Right / 2)
    $title.Y = -8
    $null = $api::ClientToScreen($window, [ref]$title)
    if (-not $api::SetCursorPos($title.X, $title.Y)) {
        throw "Could not move the pointer out of the window: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# The client area of the window.
function Get-Client {
    $client = [Meiga.Tools.RenameCheck+RECT]::new()
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

# The band above the row of tabs, where the pointer resizes the window, as
# meiga computes it (see crates/meiga-win/src/window.rs): as high as the
# frame that Windows gives a window to resize it, while the window has the
# title bar of meiga and is not maximized, and none otherwise. With the
# title bar of meiga, the client area goes up to the top of the window.
function Get-Band {
    if ($api::IsZoomed($window)) { return 0 }
    $rect = [Meiga.Tools.RenameCheck+RECT]::new()
    $corner = [Meiga.Tools.RenameCheck+POINT]::new()
    if (-not $api::GetWindowRect($window, [ref]$rect) -or -not $api::ClientToScreen($window, [ref]$corner)) {
        throw "Could not read where the window of $exe is."
    }
    if ($corner.Y -ne $rect.Top) { return 0 }
    # SM_CXPADDEDBORDER and SM_CYSIZEFRAME.
    $api::GetSystemMetricsForDpi(92, $dpi) + $api::GetSystemMetricsForDpi(33, $dpi)
}

# The layout of the bar with $Count tabs, as meiga computes it (see
# crates/meiga-core/src/ui/tabbar.rs): the band where the window is resized
# above the row of tabs, the tabs a gap below it, and on the right of the
# bar the buttons of the window and the room to drag it. Each tab with its
# area, from the top of the bar to its bottom, and one pixel more on its
# right, where a change of the tab draws (see TabBar::draw).
function Get-Bar([int]$Count) {
    $client = Get-Client
    $band = Get-Band
    $row = Get-Scaled 32
    $height = $band + $row
    $gap = Get-Scaled 4
    $top = $band + $gap
    $padding = Get-Scaled 8
    $closeSide = Get-Scaled 24
    $newSide = Get-Scaled 28
    $profilesSide = Get-Scaled 24
    $fixed = 3 * $gap + $newSide + $profilesSide + (Get-Scaled 45) + 3 * (Get-Scaled 46)
    $width = [int][Math]::Min((Get-Scaled 240), [Math]::Floor([Math]::Max(0, $client.Right - $fixed) / $Count))
    $tabs = foreach ($index in 0..($Count - 1)) {
        $left = $gap + $index * $width
        $right = $left + $width
        $close = New-Rect ($right - $gap - $closeSide) 0 ($right - $gap) 0
        [pscustomobject]@{
            Rect = New-Rect $left $top $right $height
            Area = New-Rect $left 0 ($right + 1) $height
            Title = New-Rect ($left + $padding) $top $close.Left $height
        }
    }
    [pscustomobject]@{
        Rect = New-Rect 0 0 $client.Right $height
        Height = $height
        Tabs = @($tabs)
    }
}

# A point of a tab, on its title, near its left side: on the first cell of
# the field of its name.
function Get-TabPoint($Tab) {
    [pscustomobject]@{ X = $Tab.Title.Left + 4; Y = [Math]::Floor(($Tab.Rect.Top + $Tab.Rect.Bottom) / 2) }
}

# A mouse message at a point: the message, its wParam and its lParam.
function Get-Mouse([int]$Message, [int]$Keys, $Point) {
    $Message, $Keys, (([int]$Point.X -band 0xFFFF) -bor (([int]$Point.Y -band 0xFFFF) -shl 16))
}

# Posts messages at once, without a key state: see PostAll.
function Send-Posted([long[]]$Messages) {
    $api::PostAll($window, $thread, $Messages, $null)
}

# A click of the left button at $Point: the pointer moves there, and the
# button goes down and up.
function Send-Click($Point) {
    Send-Posted @(
        Get-Mouse $WM_MOUSEMOVE 0 $Point
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $Point
        Get-Mouse $WM_LBUTTONUP 0 $Point
    )
}

# A double click of the left button at $Point: the pointer moves there, and
# the button goes down and up twice, at once, well inside the time of a
# double click.
function Send-DoubleClick($Point) {
    Send-Posted @(
        Get-Mouse $WM_MOUSEMOVE 0 $Point
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $Point
        Get-Mouse $WM_LBUTTONUP 0 $Point
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $Point
        Get-Mouse $WM_LBUTTONUP 0 $Point
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
    $message = [Meiga.Tools.RenameCheck+MSG]::new()
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
# stays down.
function Get-Key([int]$Code, [bool]$Down, [bool]$Repeat) {
    $scan = $api::MapVirtualKeyExW($Code, $MAPVK_VK_TO_VSC, $api::GetKeyboardLayout($thread))
    [long]$bits = 1 -bor ($scan -shl 16)
    if ($Down) {
        # The key was down (bit 30).
        if ($Repeat) { $bits = $bits -bor 0x40000000L }
        $message = $WM_KEYDOWN
    }
    else {
        # The key was down (bit 30), and goes up (bit 31).
        $bits = $bits -bor 0xC0000000L
        $message = $WM_KEYUP
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

# Types $Text, of lowercase letters and spaces, a key for each character.
function Send-Text([string]$Text) {
    foreach ($character in $Text.ToCharArray()) {
        Send-Press ([int][char]::ToUpperInvariant($character))
    }
}

# Ctrl+Shift and $Code: Ctrl and Shift go down for the program, if the keys
# go to it, and up after $Code.
function Send-CtrlShift([int]$Code) {
    Send-Keys @(
        Get-Key $VK_CONTROL $true
        Get-Key $VK_SHIFT $true
        Get-Key $Code $true
        Get-Key $Code $false
    ) $ctrlShiftKeys
    Send-Keys @(Get-Key $VK_SHIFT $false; Get-Key $VK_CONTROL $false)
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
    Join-Input

    # 1. The first tab, alone, in a window that has the focus, as far as
    # meiga knows: the check posts it the message of the focus, whatever
    # window the user has in the foreground.
    $first = [int]((Invoke-Step 'start') -split ' ')[1]
    Wait-Title 'meiga-rename-1' 'With one tab'
    Wait-Shown $first @() 'With one tab'
    Move-Pointer
    Send-Posted @($WM_SETFOCUS, 0, 0)
    Wait-Quiet 400
    $bar = Get-Bar 1
    $tab = $bar.Tabs[0]

    # 2. A double click on the title of the tab opens the field of its name,
    # which draws only in its tab, where its caret blinks, if Windows lets it
    # blink. Without the focus, it does not blink, and with it, it blinks
    # again.
    $start = @(Get-Log).Count
    Send-DoubleClick (Get-TabPoint $tab)
    Start-Sleep -Milliseconds $blinkWait
    $least = if ($blinks) { 2 } else { 1 }
    Assert-Inside $start $tab.Area $least 'A double click on the tab, and the caret of the field'
    if ((Get-Capture) -ne [IntPtr]::Zero) {
        throw 'After a double click on the tab, the window kept the mouse.'
    }
    Send-Posted @($WM_KILLFOCUS, 0, 0)
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Start-Sleep -Milliseconds $blinkWait
    Assert-Nothing $start 'Without the focus, while a name was edited'
    $start = @(Get-Log).Count
    Send-Posted @($WM_SETFOCUS, 0, 0)
    Start-Sleep -Milliseconds $blinkWait
    $frames = @(Get-Frames $start)
    $inTab = @($frames | Where-Object { @(Get-Areas $_ | Where-Object { -not (Test-Inside $_ $tab.Area) }).Count -eq 0 })
    if ($blinks -and $inTab.Count -lt 2) {
        Show-Lines $frames
        throw 'With the focus again, the caret of the field did not blink.'
    }
    $what = if ($blinks) { "blinks there, every $blinkTime ms" } else { 'is there, and Windows does not let it blink' }
    Write-Host "OK: a double click on the title of the tab opens the field of its name, which draws only in its tab, and keeps nothing of the mouse; its caret $what; without the focus nothing is drawn meanwhile, and with the focus again the field is still there."

    # 3. The letters and Enter give the tab its name, which the window takes
    # as its title; then nothing blinks any more. The program gets none of
    # those keys.
    Send-Text 'build'
    Send-Press $VK_RETURN
    Wait-Title 'build' 'After build and Enter in the field'
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Start-Sleep -Milliseconds $blinkWait
    Assert-Nothing $start 'After Enter in the field'
    Assert-Keys '90+ 90-' 'After build and Enter in the field'
    Write-Host 'OK: the letters typed and Enter give the tab its name, which the window takes as its title; the field closes, and nothing blinks any more; the program gets none of those keys.'

    # 4. The Menu key opens the menu of the tab, with Rename tab highlighted
    # first, which Enter chooses: the field opens while Enter is down, and
    # its repeat and Enter going up do nothing; X replaces the name, which
    # is selected, and Enter keeps it.
    Send-Press $VK_APPS
    Wait-Quiet 400
    Send-Keys (Get-Key $VK_RETURN $true)
    Wait-Quiet 400
    Send-Keys (Get-Key $VK_RETURN $true $true)
    Send-Keys (Get-Key $VK_RETURN $false)
    Assert-Title 'build' 'After Enter on Rename tab, its repeat, and Enter going up'
    Send-Press $VK_X
    Send-Press $VK_RETURN
    Wait-Title 'x' 'After Rename tab, x and Enter'
    Assert-Keys '90+ 90-' 'After Rename tab, x and Enter'
    Write-Host 'OK: Enter on Rename tab, which the Menu key highlights first, opens the field: its repeat and Enter going up do nothing, and x and Enter give the tab its name; the program gets none of those keys.'

    # 5. Ctrl+Shift+R opens the field; meanwhile Ctrl+Shift+T opens no tab,
    # and Esc closes the field and leaves the name. The program gets Ctrl
    # and Shift going up, which went down for it before the field opened.
    Send-CtrlShift $VK_R
    Wait-Quiet 400
    Send-CtrlShift $VK_T
    Start-Sleep -Milliseconds 1000
    if (@(Get-Shells).Count -ne 1) {
        throw 'Ctrl+Shift+T opened a tab while a name was edited.'
    }
    Send-Press $VK_ESCAPE
    Assert-Title 'x' 'After Ctrl+Shift+R, Ctrl+Shift+T and Esc'
    Assert-Keys '17+ 16+ 16- 17- 90+ 90-' 'After Ctrl+Shift+R, Ctrl+Shift+T and Esc'
    # Esc closed the field: the next keys go to the program.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Start-Sleep -Milliseconds $blinkWait
    Assert-Nothing $start 'After Esc in the field'
    Write-Host 'OK: Ctrl+Shift+R opens the field, where Ctrl+Shift+T does nothing, and Esc closes it and leaves the name; the program gets Ctrl and Shift going up, which went down for it before, and none of the other keys.'

    # 6. A double click opens the field again, with all the name selected.
    # While the left button is down in it, the window keeps the mouse; a
    # click at the end of the title puts the caret after the text, and Y
    # goes there. A click on the terminal keeps the name.
    Send-DoubleClick (Get-TabPoint $tab)
    Wait-Quiet 400
    $last = [pscustomobject]@{ X = $tab.Title.Right - 2; Y = (Get-TabPoint $tab).Y }
    Send-Posted @(
        Get-Mouse $WM_MOUSEMOVE 0 $last
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $last
    )
    Wait-For { (Get-Capture) -ne [IntPtr]::Zero } { 'The left button down in the field did not capture the mouse.' }
    Send-Posted (Get-Mouse $WM_LBUTTONUP 0 $last)
    Wait-For { (Get-Capture) -eq [IntPtr]::Zero } { 'The left button went up in the field, and the window kept the mouse.' }
    Send-Text 'y'
    Wait-Quiet 400
    Send-Click ([pscustomobject]@{ X = $tab.Title.Left + 4; Y = $bar.Height + 40 })
    Wait-Title 'xy' 'After a click at the end of the field, y, and a click on the terminal'
    Assert-Keys '90+ 90-' 'After the clicks in the field, y, and the click on the terminal'
    Write-Host 'OK: while the left button is down in the field, the window keeps the mouse, and then lets it go; a click puts the caret at the end of the text, and a click on the terminal keeps the name.'

    # 7. A title that the program gives meanwhile changes neither the title
    # of the window nor the name. A name of only spaces gives the tab the
    # title of its program back. Enter without changing the text does not
    # keep it as a name: the next title of the program is the title of the
    # window.
    $null = Invoke-Step 'give another title'
    Assert-Title 'xy' 'After the program gave another title to the tab with a name'
    Send-CtrlShift $VK_R
    Wait-Quiet 400
    Send-Text '   '
    Send-Press $VK_RETURN
    Wait-Title 'meiga-rename-2' 'After a name of only spaces'
    Send-CtrlShift $VK_R
    Wait-Quiet 400
    Send-Press $VK_RETURN
    Wait-Quiet 400
    $null = Invoke-Step 'give a third title'
    Wait-Title 'meiga-rename-3' 'After Enter without changing the text, the program gave another title'
    Assert-Keys '17+ 16+ 16- 17- 17+ 16+ 16- 17- 90+ 90-' 'After Ctrl+Shift+R twice, three spaces and Enter, and Enter'
    Write-Host 'OK: the titles that the program gives meanwhile change neither the title of the window nor the name; a name of only spaces gives the tab the title of its program back, and Enter without changing the text does not keep it as a name.'

    # 8. Duplicate tab, the second item of the menu, gives the copy the name
    # of the tab, which the window keeps when the program of the copy gives
    # its title. A double click on the first tab, which is hidden then, shows
    # it and edits its name.
    Send-CtrlShift $VK_R
    Wait-Quiet 400
    Send-Text 'dup'
    Send-Press $VK_RETURN
    Wait-Title 'dup' 'After dup and Enter in the field'
    Send-Press $VK_APPS
    Wait-Quiet 400
    Send-Press $VK_DOWN
    Send-Press $VK_RETURN
    $copy = Wait-Copy @($first) 'Duplicate tab'
    Wait-Shown $copy @($first) 'After Duplicate tab'
    Wait-For { Test-Path -LiteralPath "$reports.$copy" } { 'After Duplicate tab, the program of the copy did not start.' }
    Assert-Title 'dup' 'After the program of the copy, which has the name of its tab, gave its title'
    $bar = Get-Bar 2
    Move-Pointer
    Send-DoubleClick (Get-TabPoint $bar.Tabs[0])
    Wait-Shown $first @($copy) 'After a double click on the first tab'
    Wait-Quiet 400
    Send-Text 'one'
    Send-Press $VK_RETURN
    Wait-Title 'one' 'After a double click on the first tab, one and Enter'
    Assert-Keys '17+ 16+ 16- 17- 90+ 90-' 'After Ctrl+Shift+R, dup and Enter, Duplicate tab, and one and Enter on the first tab'
    Write-Host 'OK: Duplicate tab gives the copy the name of the tab, which the window keeps when the program of the copy gives its title; a double click on a hidden tab shows it and edits its name.'

    # 9. The field of the name of the copy, which a click shows, closes when
    # its program ends, and its tab with it: the program of the first tab,
    # which is shown then, gets the keys again.
    Send-Click (Get-TabPoint $bar.Tabs[1])
    Wait-Shown $copy @($first) 'After a click on the copy'
    Wait-Title 'dup' 'After a click on the copy'
    Send-CtrlShift $VK_R
    Wait-Quiet 400
    Send-Text 'zz'
    $null = $endCopies.Set()
    Wait-Ended $copy 'The program of the copy, after the check ended it,'
    Wait-Shown $first @() 'After the tab of the copy closed'
    Wait-Title 'one' 'After the tab of the copy closed'
    Assert-Keys '90+ 90-' 'After the tab whose name was edited closed'
    Write-Host 'OK: when the program of the tab whose name is edited ends, its tab closes, and the editing with it: the program of the tab that is shown then gets the keys again.'

    # 10. Ctrl+Shift+W closes the last tab, and the window.
    Send-CtrlShift $VK_W
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after Ctrl+Shift+W on its last tab."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Wait-Ended $first 'The program of the last tab'
    Write-Host 'OK: Ctrl+Shift+W closes the last tab and the window, and meiga ends with exit code 0.'
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
    $endCopies.Dispose()
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
