<#
.SYNOPSIS
Checks the title bar of meiga: its frame, the parts of the window for Windows,
its buttons and its menu.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and an APPDATA of its
own, whose config.toml opens its tabs with Light: cmd, with a prompt that only
hides the cursor, so that nothing is drawn while nothing changes. It checks
that:

- the bar of meiga goes up to the top of the window, over a band that resizes
  it, and Windows keeps its frame on the other sides;
- WM_NCHITTEST, the message with which Windows asks a window what each point
  of it is, gives the top edge and its corner on the band, the caption on the
  free parts of the bar, the buttons to minimize, maximize and close the
  window along all their height, the client area on the tabs, on the buttons
  of the bar and on the terminal, and the frame of Windows around them: with
  the window restored, maximized, as small as it can be, with the tabs
  scrolling in the strip, and while a menu of meiga is open;
- the pointer on the buttons of the window draws only them;
- the right button on the bar opens the menu of the window, with the commands
  that the state of the window allows, and Close in bold, as Windows has it,
  and nothing is drawn while it is open; Esc closes it, and Down and Enter
  choose its commands;
- the buttons maximize, restore, minimize and close the window, and a double
  click on the bar maximizes and restores it; maximized, the client area is
  the work area of the monitor; minimized, nothing is drawn;
- the window cannot be narrower than the bar with one tab between the buttons
  that scroll the strip, nor shorter than the bar;
- losing and getting the focus draws, on the bar, only the buttons of the
  window;
- the button to close it ends meiga.exe with exit code 0, and the programs of
  its tabs.

The check posts the messages of the mouse and of the keys to the window. It
moves the pointer of the system out of the window, where it can, and puts it
back at the end.

If a check fails, the script ends meiga.exe and the programs of its tabs, and
keeps its folder, with the frame log.

.PARAMETER Path
The executable to check. Defaults to target\release\meiga.exe.

.PARAMETER TimeoutSeconds
How long to wait for the window, for each program to start and to end, and
for each frame.
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
if (-not $Path) { $Path = Join-Path $PSScriptRoot '..\target\release\meiga.exe' }
$Path = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path)
if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "$Path does not exist. Build it first with: cargo build --release"
}
$exe = Split-Path -Leaf $Path

if (-not ('Meiga.Tools.TitleBarCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name TitleBarCheck -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)]
public struct RECT { public int Left, Top, Right, Bottom; }

[StructLayout(LayoutKind.Sequential)]
public struct POINT { public int X, Y; }

[StructLayout(LayoutKind.Sequential)]
public struct MONITORINFO { public int Size; public RECT Monitor; public RECT Work; public uint Flags; }

[StructLayout(LayoutKind.Sequential)]
public struct APPBARDATA
{
    public uint Size;
    public IntPtr Window;
    public uint CallbackMessage;
    public uint Edge;
    public RECT Rect;
    public IntPtr Param;
}

[DllImport("user32.dll")]
public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetClientRect(IntPtr window, out RECT rect);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetWindowRect(IntPtr window, out RECT rect);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ClientToScreen(IntPtr window, ref POINT point);

[DllImport("user32.dll")]
public static extern uint GetDpiForWindow(IntPtr window);

[DllImport("user32.dll")]
public static extern int GetSystemMetricsForDpi(int index, uint dpi);

[DllImport("user32.dll")]
public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsZoomed(IntPtr window);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsIconic(IntPtr window);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool IsWindowVisible(IntPtr window);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetCursorPos(out POINT point);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetCursorPos(int x, int y);

[DllImport("user32.dll")]
public static extern IntPtr WindowFromPoint(POINT point);

[DllImport("user32.dll")]
public static extern IntPtr GetAncestor(IntPtr window, uint flags);

[DllImport("user32.dll")]
public static extern IntPtr MonitorFromPoint(POINT point, uint flags);

[DllImport("user32.dll")]
public static extern IntPtr MonitorFromWindow(IntPtr window, uint flags);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetMonitorInfoW(IntPtr monitor, ref MONITORINFO info);

[DllImport("shell32.dll")]
static extern UIntPtr SHAppBarMessage(uint message, ref APPBARDATA data);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool PostMessageW(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);

[DllImport("user32.dll", SetLastError = true)]
static extern IntPtr SendMessageTimeoutW(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam, uint flags, uint timeout, out IntPtr result);

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string className, string title);

[DllImport("user32.dll")]
public static extern int GetMenuItemCount(IntPtr menu);

[DllImport("user32.dll")]
public static extern uint GetMenuItemID(IntPtr menu, int position);

[DllImport("user32.dll")]
public static extern uint GetMenuState(IntPtr menu, uint item, uint flags);

[DllImport("user32.dll")]
public static extern uint MapVirtualKeyW(uint code, uint type);

[DllImport("kernel32.dll", SetLastError = true)]
static extern IntPtr OpenThread(uint access, [MarshalAs(UnmanagedType.Bool)] bool inherit, uint thread);

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

delegate bool EnumProc(IntPtr window, IntPtr data);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool EnumWindows(EnumProc callback, IntPtr data);

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern int GetClassNameW(IntPtr window, System.Text.StringBuilder name, int size);

[DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
static extern uint GetLongPathNameW(string path, System.Text.StringBuilder longPath, uint size);

// Posts the messages, three numbers for each one (the message, its wParam and
// its lParam), to the window, with its thread suspended: the thread gets them
// one after the other, before anything that Windows posts while it handles
// them, such as WM_MOUSELEAVE when the pointer of the system is elsewhere.
public static void PostAll(IntPtr window, uint thread, long[] messages)
{
    // THREAD_SUSPEND_RESUME and THREAD_GET_CONTEXT.
    IntPtr handle = OpenThread(0x0002 | 0x0008, false, thread);
    if (handle == IntPtr.Zero) throw new System.ComponentModel.Win32Exception();
    // A CONTEXT of x64, 1232 bytes, aligned to 16.
    IntPtr memory = Marshal.AllocHGlobal(1232 + 16);
    try
    {
        if (SuspendThread(handle) == uint.MaxValue) throw new System.ComponentModel.Win32Exception();
        try
        {
            IntPtr context = new IntPtr((memory.ToInt64() + 15) & ~15L);
            // ContextFlags: CONTEXT_CONTROL. GetThreadContext returns once the
            // thread is really suspended.
            Marshal.WriteInt32(context, 0x30, 0x00100001);
            if (!GetThreadContext(handle, context)) throw new System.ComponentModel.Win32Exception();
            for (int i = 0; i + 2 < messages.Length; i += 3)
            {
                if (!PostMessageW(window, (uint)messages[i], new UIntPtr((ulong)messages[i + 1]), new IntPtr(messages[i + 2])))
                    throw new System.ComponentModel.Win32Exception();
            }
        }
        finally
        {
            ResumeThread(handle);
        }
    }
    finally
    {
        Marshal.FreeHGlobal(memory);
        CloseHandle(handle);
    }
}

// What the window says that a point of the screen is: its answer to
// WM_NCHITTEST.
public static int Hit(IntPtr window, int x, int y)
{
    IntPtr point = new IntPtr((long)(uint)(((y & 0xFFFF) << 16) | (x & 0xFFFF)));
    IntPtr result;
    // SMTO_ABORTIFHUNG, and 2 s.
    if (SendMessageTimeoutW(window, 0x0084, UIntPtr.Zero, point, 0x0002, 2000, out result) == IntPtr.Zero)
        throw new System.ComponentModel.Win32Exception();
    return (int)result.ToInt64();
}

// The menu window (class #32768) that the thread shows, or zero.
public static IntPtr MenuOf(uint thread)
{
    IntPtr after = IntPtr.Zero;
    while (true)
    {
        after = FindWindowExW(IntPtr.Zero, after, "#32768", null);
        if (after == IntPtr.Zero) return IntPtr.Zero;
        uint process;
        if (GetWindowThreadProcessId(after, out process) == thread && IsWindowVisible(after)) return after;
    }
}

// The menu that a menu window shows (MN_GETHMENU), or zero.
public static IntPtr MenuIn(IntPtr menuWindow)
{
    IntPtr result;
    if (SendMessageTimeoutW(menuWindow, 0x01E1, UIntPtr.Zero, IntPtr.Zero, 0x0002, 2000, out result) == IntPtr.Zero)
        return IntPtr.Zero;
    return result;
}

// The edges of the monitor where a taskbar hides itself, as bits: 1 left,
// 2 top, 4 right and 8 bottom.
public static int AutoHideEdges(RECT monitor)
{
    APPBARDATA data = new APPBARDATA();
    data.Size = (uint)Marshal.SizeOf(typeof(APPBARDATA));
    // ABM_GETSTATE, and ABS_AUTOHIDE.
    if ((SHAppBarMessage(4, ref data).ToUInt64() & 1) == 0) return 0;
    int edges = 0;
    for (uint edge = 0; edge < 4; edge++)
    {
        data.Edge = edge;
        data.Rect = monitor;
        // ABM_GETAUTOHIDEBAREX: the taskbar that hides itself on that edge of
        // the monitor, if there is one.
        if (SHAppBarMessage(11, ref data) != UIntPtr.Zero) edges |= 1 << (int)edge;
    }
    return edges;
}

// The top-level window of the process with that class, or zero.
public static IntPtr WindowOf(int process, string className)
{
    IntPtr found = IntPtr.Zero;
    EnumWindows((window, data) =>
    {
        uint owner;
        GetWindowThreadProcessId(window, out owner);
        if (owner != (uint)process) return true;
        System.Text.StringBuilder name = new System.Text.StringBuilder(64);
        if (GetClassNameW(window, name, name.Capacity) > 0 && name.ToString() == className)
        {
            found = window;
            return false;
        }
        return true;
    }, IntPtr.Zero);
    return found;
}

// The path with the long names of its folders, or the path itself.
public static string LongPathOf(string path)
{
    System.Text.StringBuilder buffer = new System.Text.StringBuilder(1024);
    uint length = GetLongPathNameW(path, buffer, (uint)buffer.Capacity);
    if (length == 0 || length >= buffer.Capacity) return path;
    return buffer.ToString();
}
'@
}
$api = [Meiga.Tools.TitleBarCheck]

# The messages that the check posts to the window, and the buttons of the
# mouse in their wParam.
$WM_SETFOCUS = 0x0007
$WM_KILLFOCUS = 0x0008
$WM_NCMOUSEMOVE = 0x00A0
$WM_NCLBUTTONDOWN = 0x00A1
$WM_NCLBUTTONUP = 0x00A2
$WM_NCLBUTTONDBLCLK = 0x00A3
$WM_NCRBUTTONDOWN = 0x00A4
$WM_NCRBUTTONUP = 0x00A5
$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_SYSCOMMAND = 0x0112
$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_MBUTTONDOWN = 0x0207
$WM_MBUTTONUP = 0x0208
$MK_LBUTTON = 0x0001
$MK_MBUTTON = 0x0010

# The keys.
$VK_RETURN = 0x0D
$VK_ESCAPE = 0x1B
$VK_DOWN = 0x28
$VK_APPS = 0x5D

# What WM_NCHITTEST gives, and how the check names it.
$HTCLIENT = 1
$HTCAPTION = 2
$HTMINBUTTON = 8
$HTMAXBUTTON = 9
$HTLEFT = 10
$HTRIGHT = 11
$HTTOP = 12
$HTTOPLEFT = 13
$HTTOPRIGHT = 14
$HTBOTTOM = 15
$HTBOTTOMLEFT = 16
$HTBOTTOMRIGHT = 17
$HTCLOSE = 20
$hitNames = @{
    0 = 'nothing'
    1 = 'the client area'
    2 = 'the caption'
    8 = 'the button to minimize'
    9 = 'the button to maximize'
    10 = 'the left edge'
    11 = 'the right edge'
    12 = 'the top edge'
    13 = 'the top left corner'
    14 = 'the top right corner'
    15 = 'the bottom edge'
    16 = 'the bottom left corner'
    17 = 'the bottom right corner'
    20 = 'the button to close'
}

# The buttons of the window, from the right: what WM_NCHITTEST gives on them,
# and their names.
$captionHits = $HTCLOSE, $HTMAXBUTTON, $HTMINBUTTON
$captionNames = 'the button to close the window', 'the button to maximize the window',
    'the button to minimize the window'

# The commands of the menu of the window, and the states of its items:
# MF_GRAYED or MF_DISABLED, MF_HILITE and MF_DEFAULT.
$SC_SIZE = 0xF000
$SC_MOVE = 0xF010
$SC_MINIMIZE = 0xF020
$SC_MAXIMIZE = 0xF030
$SC_CLOSE = 0xF060
$SC_RESTORE = 0xF120
$windowCommands = [ordered]@{
    Restore = $SC_RESTORE
    Move = $SC_MOVE
    Size = $SC_SIZE
    Minimize = $SC_MINIMIZE
    Maximize = $SC_MAXIMIZE
    Close = $SC_CLOSE
}
$MF_UNABLE = 0x0003
$MF_HILITE = 0x0080
$MF_DEFAULT = 0x1000

# TOP_LEFT_CORNER of crates/meiga-win/src/window.rs: how far from the left
# side of the window the band resizes it also to the left, in pixels at any
# scale; and AUTOHIDE_TASKBAR, the pixels that a maximized window leaves free
# on an edge of its monitor where the taskbar hides itself.
$topLeftCorner = 19
$autoHideGap = 2

$consoles = 'OpenConsole.exe', 'conhost.exe'

$name = "meiga-check-titlebar-$PID"
# All the files of the check, with the APPDATA of meiga. With the long names
# of the folders, which .NET gives as the current one.
$work = Join-Path ($api::LongPathOf([IO.Path]::GetTempPath())) $name
$appdata = Join-Path $work 'appdata'
$config = Join-Path $appdata 'meiga\config.toml'
$log = Join-Path $work 'frames.csv'
Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
$null = New-Item -ItemType Directory -Force -Path (Split-Path $config)
$utf8 = [Text.UTF8Encoding]::new($false)

# The names of the shells that meiga finds (see
# crates/meiga-win/src/profiles.rs): the ones of Windows and of Git, and the
# distributions of WSL of the user. A profile of config.toml that only hides
# a shell that meiga does not find does nothing.
function Get-ShellNames {
    'PowerShell'
    'Windows PowerShell'
    'Command Prompt'
    'Git Bash'
    $lxss = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Lxss')
    if (-not $lxss) { return }
    try {
        foreach ($subkey in $lxss.GetSubKeyNames()) {
            $key = $lxss.OpenSubKey($subkey)
            if (-not $key) { continue }
            try {
                $distribution = $key.GetValue('DistributionName')
                if ($distribution -is [string] -and $distribution) { $distribution }
            }
            finally { $key.Dispose() }
        }
    }
    finally { $lxss.Dispose() }
}

# $Text as a basic string of TOML.
function ConvertTo-TomlString([string]$Text) {
    '"' + $Text.Replace('\', '\\').Replace('"', '\"') + '"'
}

$hidden = [Collections.Generic.List[string]]::new()
$names = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
foreach ($shell in Get-ShellNames) {
    if ($names.Add($shell)) { $hidden.Add($shell) }
}

# The configuration: Light, the default profile, runs cmd with a prompt that
# only hides the cursor, so that nothing is drawn while nothing changes; and
# the shells that meiga finds, hidden.
$lightCommand = '%SystemRoot%\System32\cmd.exe /d /q /k prompt $E[?25l'
$configLines = @(
    '# Written by tools/check-titlebar.ps1.'
    'default_profile = "Light"'
    ''
    '[[profiles]]'
    'name = "Light"'
    "command = '$lightCommand'"
    "directory = '%MEIGA_CHECK_WORK%'"
    foreach ($shell in $hidden) {
        ''
        '[[profiles]]'
        "name = $(ConvertTo-TomlString $shell)"
        'hidden = true'
    }
)
[IO.File]::WriteAllText($config, ($configLines -join "`n") + "`n", $utf8)

# What the tabs of Light run.
$lightLine = $lightCommand.Replace('%SystemRoot%', $env:SystemRoot)

# As per monitor (v2), the same as meiga, the size of the window is in pixels.
$previousDpi = $api::SetThreadDpiAwarenessContext([IntPtr]::new(-4))

# The bar, as meiga lays it out (see crates/meiga-core/src/ui/tabbar.rs), at
# the scale of the window: its width and its height, the band above the row
# of the tabs, the middle of that row, its gaps and its buttons, and the width
# and the scroll of its tabs, which Update-Bar reads; and the programs of the
# tabs, in their order.
$barWidth = 0
$band = 0
$rowHeight = 0
$barHeight = 0
$midY = 0
$gap = 0
$newSide = 0
$profilesSide = 0
$scrollSide = 0
$dragSide = 0
$captionSide = 0
$maxTab = 0
$minTab = 0
$barTabWidth = 0
$barOverflow = $false
$barScroll = 0
$tabs = [Collections.Generic.List[int]]::new()

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

# How many lines of the frame log there are, without its header: where the
# lines that come next start.
function Get-Mark {
    if (-not (Test-Path -LiteralPath $log)) { return 0 }
    [Math]::Max(0, @(Get-Content -LiteralPath $log).Count - 1)
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
        $lines = Get-Mark
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

# Waits until meiga draws a full frame after the first $From lines of the
# log, and checks that it drew only one once it stops drawing.
function Wait-OneFull([int]$From, [string]$What) {
    Wait-For { @(Get-Frames $From | Where-Object { Test-Full $_ }).Count -gt 0 } { "$What did not draw a full frame." }
    Assert-OneFull $From $What
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

# Whether $One and $Other have some pixel in common.
function Test-Overlap($One, $Other) {
    $One.Left -lt $Other.Right -and $Other.Left -lt $One.Right -and
    $One.Top -lt $Other.Bottom -and $Other.Top -lt $One.Bottom
}

# Whether $One and $Other are the same rectangle.
function Test-Same($One, $Other) {
    $One.Left -eq $Other.Left -and $One.Top -eq $Other.Top -and
    $One.Right -eq $Other.Right -and $One.Bottom -eq $Other.Bottom
}

# The pixels that $One and $Other have in common: an empty rectangle if none.
function Get-Intersection($One, $Other) {
    $left = [Math]::Max($One.Left, $Other.Left)
    $top = [Math]::Max($One.Top, $Other.Top)
    New-Rect $left $top ([Math]::Max($left, [Math]::Min($One.Right, $Other.Right))) ([Math]::Max($top, [Math]::Min($One.Bottom, $Other.Bottom)))
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

# Waits until the consoles of $Consoles have ended, once meiga has.
function Wait-ConsolesEnded($Consoles) {
    foreach ($console in $Consoles) {
        Wait-Ended ([int]$console.ProcessId) "The console $($console.Name) of a tab"
    }
}

# Waits until the processes of $Ids have used no processor time for a
# second: a shell that starts has written its prompt, and given its title.
function Wait-Settled([int[]]$Ids, [string]$What) {
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds + 10)
    $last = -1
    $since = [DateTime]::UtcNow
    while ($true) {
        $programs = @(Get-Process -Id $Ids -ErrorAction Ignore)
        if ($programs.Count -ne $Ids.Count) { throw "$What`: a program ended." }
        $time = ($programs | Measure-Object -Sum { $_.TotalProcessorTime.Ticks }).Sum
        if ($time -ne $last) {
            $last = $time
            $since = [DateTime]::UtcNow
        }
        elseif (([DateTime]::UtcNow - $since).TotalMilliseconds -ge 1000) {
            return
        }
        if ([DateTime]::UtcNow -gt $deadline) {
            throw "$What`: the programs kept running for $($TimeoutSeconds + 10) s."
        }
        Start-Sleep -Milliseconds 100
    }
}

# The client area of the window.
function Get-Client {
    $client = [Meiga.Tools.TitleBarCheck+RECT]::new()
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

# The pixel in the middle of $Rect.
function Get-Middle($Rect) {
    [pscustomobject]@{ X = [int][Math]::Floor(($Rect.Left + $Rect.Right) / 2); Y = [int][Math]::Floor(($Rect.Top + $Rect.Bottom) / 2) }
}

# $Point of the client area on the screen.
function Get-OnScreen($Point) {
    $screen = [Meiga.Tools.TitleBarCheck+POINT]::new()
    $screen.X = $Point.X
    $screen.Y = $Point.Y
    if (-not $api::ClientToScreen($window, [ref]$screen)) {
        throw "Could not find a point of the window of $exe on the screen."
    }
    $screen
}

# A mouse message at a point: the message, its wParam and its lParam.
function Get-Mouse([int]$Message, [int]$Keys, $Point) {
    $Message, $Keys, (([int]$Point.X -band 0xFFFF) -bor (([int]$Point.Y -band 0xFFFF) -shl 16))
}

# The band above the row of tabs, where the pointer resizes the window, as
# meiga computes it (see crates/meiga-win/src/window.rs): as high as the frame
# that Windows gives a window to resize it, while the window has the title bar
# of meiga and is not maximized, and none otherwise. With the title bar of
# meiga, the client area goes up to the top of the window.
function Get-Band {
    if ($api::IsZoomed($window)) { return 0 }
    $rect = [Meiga.Tools.TitleBarCheck+RECT]::new()
    if (-not $api::GetWindowRect($window, [ref]$rect)) {
        throw "Could not read where the window of $exe is."
    }
    if ((Get-OnScreen ([pscustomobject]@{ X = 0; Y = 0 })).Y -ne $rect.Top) { return 0 }
    # SM_CXPADDEDBORDER and SM_CYSIZEFRAME.
    $api::GetSystemMetricsForDpi(92, $dpi) + $api::GetSystemMetricsForDpi(33, $dpi)
}

# The width of the bar without what is not for the tabs: three gaps, the
# buttons that open a tab, the narrowest room to drag the window, and the
# three buttons of the window.
function Get-Room {
    [Math]::Max(0, $barWidth - 3 * $gap - $newSide - $profilesSide - $dragSide - 3 * $captionSide)
}

# The width of the strip while the tabs do not fit.
function Get-Viewport {
    [Math]::Max(0, (Get-Room) - 2 * $scrollSide)
}

function Get-MaxScroll {
    if (-not $barOverflow) { return 0 }
    [Math]::Max(0, $tabs.Count * $barTabWidth - (Get-Viewport))
}

# Gives the tabs their width, as meiga does when they open or close, but for
# the ones that close with the mouse while the pointer is on the bar.
function Update-Fit {
    $width = [Math]::Min([Math]::Floor((Get-Room) / [Math]::Max(1, $tabs.Count)), $maxTab)
    $script:barOverflow = $width -lt $minTab
    $script:barTabWidth = if ($barOverflow) { $minTab } else { [int]$width }
    $script:barScroll = [Math]::Min($barScroll, (Get-MaxScroll))
}

function Get-Strip {
    if (-not $barOverflow) { return New-Rect 0 0 $barWidth $barHeight }
    $left = $gap + $scrollSide
    New-Rect $left 0 ($left + (Get-Viewport)) $barHeight
}

# Where the first tab is, or would be.
function Get-Start {
    if ($barOverflow) { (Get-Strip).Left - $barScroll } else { $gap }
}

# The tab at $Index, from a gap below the band to the bottom of the bar.
function Get-TabRect([int]$Index) {
    $left = (Get-Start) + $Index * $barTabWidth
    New-Rect $left ($band + $gap) ($left + $barTabWidth) $barHeight
}

function Get-NewRect {
    $left = if ($barOverflow) { (Get-Strip).Right + $scrollSide + $gap } else { (Get-TabRect $tabs.Count).Left + $gap }
    $top = $band + $gap
    New-Rect $left $top ($left + $newSide) ($top + $newSide)
}

function Get-ProfilesRect {
    $new = Get-NewRect
    New-Rect $new.Right $new.Top ($new.Right + $profilesSide) $new.Bottom
}

# The button that scrolls the strip towards $Side, Left or Right.
function Get-ScrollRect([string]$Side) {
    $strip = Get-Strip
    $left = if ($Side -eq 'Left') { $strip.Left - $scrollSide } else { $strip.Right }
    $top = $band + $gap
    New-Rect $left $top ($left + $scrollSide) ($top + $newSide)
}

# The three buttons of the window together, on the right of the bar, as high
# as all of it, also over the band.
function Get-CaptionsRect {
    New-Rect ($barWidth - 3 * $captionSide) 0 $barWidth $barHeight
}

# Flags of SetWindowPos, and of GetMenuState.
$SWP_NOMOVE = 0x0002
$SWP_NOZORDER = 0x0004
$SWP_NOACTIVATE = 0x0010
$MF_BYPOSITION = 0x0400

# Posts $Messages to the window at once: see PostAll. If meiga has ended, it
# says so, instead of the error of its thread, which is not there any more.
function Send-Posted([long[]]$Messages) {
    try {
        $api::PostAll($window, $thread, $Messages)
    }
    catch {
        if ($process.WaitForExit(2000)) {
            throw "$exe ended with exit code $($process.ExitCode), and the check could not post its keys or its clicks to its window."
        }
        throw
    }
}

# The messages of $Code going down and up, as Windows sends them, with the
# scan code of the key: each message, its wParam and its lParam.
function Get-Press([int]$Code) {
    $scan = [long]$api::MapVirtualKeyW($Code, 0)
    [long]$bits = 1 -bor ($scan -shl 16)
    # Going up, the key was down (bit 30), and goes up (bit 31).
    $WM_KEYDOWN, $Code, $bits, $WM_KEYUP, $Code, ($bits -bor 0xC0000000L)
}

# Types $Code: it goes down and up.
function Send-Key([int]$Code) {
    Send-Posted (Get-Press $Code)
}

# A point of the screen as the lParam of a message.
function Get-ScreenParam($Point) {
    ([int]$Point.X -band 0xFFFF) -bor (([int]$Point.Y -band 0xFFFF) -shl 16)
}

# A point, of the screen or of the client area.
function New-Point([int]$X, [int]$Y) {
    $point = [Meiga.Tools.TitleBarCheck+POINT]::new()
    $point.X = $X
    $point.Y = $Y
    $point
}

# Where the window is on the screen, with its frame.
function Get-WindowRect {
    $rect = [Meiga.Tools.TitleBarCheck+RECT]::new()
    if (-not $api::GetWindowRect($window, [ref]$rect)) {
        throw "Could not read where the window of $exe is."
    }
    $rect
}

# Reads the bar again, as meiga lays it out for the window as it is now:
# its width, the band above the tabs, its height, the middle of the row of
# the tabs, and the width of the tabs.
function Update-Bar {
    $script:barWidth = (Get-Client).Right
    $script:band = Get-Band
    $script:barHeight = $band + $rowHeight
    $script:midY = $band + [int][Math]::Floor($rowHeight / 2)
    Update-Fit
}

# The button of the window at $Index from the right: 0 closes the window, 1
# maximizes or restores it, and 2 minimizes it.
function Get-Caption([int]$Index) {
    New-Rect ($barWidth - ($Index + 1) * $captionSide) 0 ($barWidth - $Index * $captionSide) $barHeight
}

# A point in the middle of the room to drag the window, in the middle of the
# row of the tabs: between the button of the profiles and the buttons of the
# window, which always leaves that room free, and a gap.
function Get-DragPoint {
    New-Point ([int][Math]::Floor(((Get-ProfilesRect).Right + (Get-CaptionsRect).Left) / 2)) $midY
}

# A point of the window, and what it has to be for Windows: what
# WM_NCHITTEST gives there.
function New-Hit([int]$X, [int]$Y, [int]$Hit) {
    [pscustomobject]@{ X = $X; Y = $Y; Hit = $Hit }
}

function Show-Hit([int]$Hit) {
    if ($hitNames.ContainsKey($Hit)) { $hitNames[$Hit] } else { "the code $Hit" }
}

# What the window says that $Point of the screen is.
function Get-Hit($Point) {
    try {
        $api::Hit($window, $Point.X, $Point.Y)
    }
    catch {
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) while the check asked its window what a point is."
        }
        throw
    }
}

# Asks the window what each point of $Expected is, from the top left corner
# of its client area or, with $Screen, of the screen; throws what $When says
# if any of them is not what Windows has to get.
function Assert-Hits($Expected, [string]$When, [switch]$Screen) {
    $wrong = foreach ($item in $Expected) {
        $at = if ($Screen) { New-Point $item.X $item.Y } else { Get-OnScreen $item }
        $hit = Get-Hit $at
        if ($hit -ne $item.Hit) {
            $whose = if ($Screen) { 'of the screen' } else { 'of the client area' }
            "the point ($($item.X), $($item.Y)) $whose is $(Show-Hit $hit), and not $(Show-Hit $item.Hit)"
        }
    }
    if (@($wrong).Count -gt 0) {
        throw "$When, for Windows, $(@($wrong) -join '; ')."
    }
}

# The buttons of the window along all their height, also over the band: in
# the middle of each one, at the top, in the middle of the row of the tabs and
# at the bottom of the bar; and the last pixel of the top row.
function Get-CaptionHits {
    foreach ($index in 0, 1, 2) {
        $x = (Get-Middle (Get-Caption $index)).X
        New-Hit $x 0 $captionHits[$index]
        New-Hit $x $midY $captionHits[$index]
        New-Hit $x ($barHeight - 1) $captionHits[$index]
    }
    New-Hit ($barWidth - 1) 0 $HTCLOSE
}

# Clicks the button of the window at $Index from the right, as Windows sends
# it: the pointer moves onto it, and the left button goes down and up. The
# bar acts when the left button goes up on the button that it highlights.
function Send-Caption([int]$Index) {
    $param = Get-ScreenParam (Get-OnScreen (Get-Middle (Get-Caption $Index)))
    $code = $captionHits[$Index]
    Send-Posted @($WM_NCMOUSEMOVE, $code, $param, $WM_NCLBUTTONDOWN, $code, $param, $WM_NCLBUTTONUP, $code, $param)
}

# Moves the pointer onto the button of the window at $Index from the right,
# and checks that the window draws only that button: when the pointer comes,
# and when it goes, since the pointer of the system is not there, so Windows
# tells the window at once that it left.
function Assert-CaptionHover([int]$Index) {
    $button = Get-Caption $Index
    $what = $captionNames[$Index]
    $param = Get-ScreenParam (Get-OnScreen (Get-Middle $button))
    Wait-Quiet 400
    $from = Get-Mark
    Send-Posted @($WM_NCMOUSEMOVE, $captionHits[$Index], $param)
    Wait-For { @(Get-Frames $from).Count -gt 0 } { "The pointer on $what drew nothing." }
    Wait-Quiet 400
    $frames = @(Get-Frames $from)
    $areas = @(Get-Areas $frames)
    if ($frames.Count -gt 2 -or @($areas | Where-Object { -not (Test-Same $_ $button) }).Count -gt 0) {
        Show-Lines $frames
        throw "The pointer on $what did not draw only $(Show-Rect $button), once or twice: $(@($areas | ForEach-Object { Show-Rect $_ }) -join ', ')."
    }
}

# The menu of the window that its thread shows, or zero.
function Get-WindowMenu {
    $menuWindow = $api::MenuOf($thread)
    if ($menuWindow -eq [IntPtr]::Zero) { return [IntPtr]::Zero }
    $api::MenuIn($menuWindow)
}

# Opens the menu of the window with the right button on the room to drag it,
# as Windows sends it, and returns the menu once it is on the screen.
function Open-WindowMenu {
    $param = Get-ScreenParam (Get-OnScreen (Get-DragPoint))
    Send-Posted @($WM_NCRBUTTONDOWN, $HTCAPTION, $param, $WM_NCRBUTTONUP, $HTCAPTION, $param)
    Wait-For { (Get-WindowMenu) -ne [IntPtr]::Zero } { 'The right button on the bar did not open the menu of the window.' }
    Get-WindowMenu
}

# The position of the item of $Command in $Menu, or -1.
function Get-ItemPosition([IntPtr]$Menu, [int]$Command) {
    $count = $api::GetMenuItemCount($Menu)
    for ($index = 0; $index -lt $count; $index++) {
        if ($api::GetMenuItemID($Menu, $index) -eq [uint32]$Command) { return $index }
    }
    -1
}

# Checks that the menu of the window has the commands that the window
# allows, maximized or not, enabled, the others disabled, and Close as the
# default one, in bold, as in the menu that Windows opens with Alt+Space;
# otherwise throws what $When says.
function Assert-WindowMenu([IntPtr]$Menu, [bool]$Maximized, [string]$When) {
    $enabled = @{
        Restore = $Maximized
        Move = -not $Maximized
        Size = -not $Maximized
        Minimize = $true
        Maximize = -not $Maximized
        Close = $true
    }
    $wrong = foreach ($command in $windowCommands.Keys) {
        $position = Get-ItemPosition $Menu $windowCommands[$command]
        if ($position -lt 0) {
            "it has no $command"
            continue
        }
        $state = $api::GetMenuState($Menu, [uint32]$position, $MF_BYPOSITION)
        $able = ($state -band $MF_UNABLE) -eq 0
        if ($able -ne $enabled[$command]) {
            "$command is $(if ($able) { 'enabled' } else { 'disabled' })"
        }
        $default = ($state -band $MF_DEFAULT) -ne 0
        if ($default -ne ($command -eq 'Close')) {
            "$command is $(if (-not $default) { 'not ' })the default item"
        }
    }
    if (@($wrong).Count -gt 0) {
        throw "$When, in the menu of the window, $(@($wrong) -join ', ')."
    }
}

# The position of the item that $Menu highlights, or -1.
function Get-Highlight([IntPtr]$Menu) {
    $count = $api::GetMenuItemCount($Menu)
    for ($index = 0; $index -lt $count; $index++) {
        $state = $api::GetMenuState($Menu, [uint32]$index, $MF_BYPOSITION)
        if ($state -ne [uint32]::MaxValue -and ($state -band $MF_HILITE) -ne 0) { return $index }
    }
    -1
}

# Moves the highlight of the menu of the window with Down until it is on the
# item of $Command: Down highlights the items one after the other, also the
# disabled ones, skips the separator, and goes round. The menu takes the
# highlight from an item before it gives it to the next one, so the check
# waits for another item, and not only for a change.
function Select-WindowCommand([IntPtr]$Menu, [string]$Command) {
    $target = Get-ItemPosition $Menu $windowCommands[$Command]
    if ($target -lt 0) { throw "The menu of the window has no $Command." }
    for ($press = 0; $press -le 10; $press++) {
        $before = Get-Highlight $Menu
        if ($before -eq $target) { return }
        Send-Key $VK_DOWN
        Wait-For { (Get-Highlight $Menu) -notin -1, $before } { 'Down did not move the highlight of the menu of the window.' }
    }
    throw "Down did not highlight $Command in the menu of the window."
}

# Closes the menu of the window with Esc.
function Close-WindowMenu {
    Send-Key $VK_ESCAPE
    Wait-For { $api::MenuOf($thread) -eq [IntPtr]::Zero } { 'Esc did not close the menu of the window.' }
}

# Whether $Point of the screen is on the window: the window there, or its
# top-level one (GA_ROOT), is it.
function Test-OnWindow($Point) {
    $api::GetAncestor($api::WindowFromPoint($Point), 2) -eq $window
}

# If the pointer of the system is on the window, moves it out, to the first
# place next to the window that is on a monitor: the moves that Windows makes
# at its place would reach the bar, which would draw, and keep what it
# highlights. With the client of the Remote Desktop minimized, Windows does
# not let a program read nor move the pointer, and a maximized window may
# leave no place for it: then it stays.
function Move-Pointer {
    $now = [Meiga.Tools.TitleBarCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$now) -or -not (Test-OnWindow $now)) { return }
    if ($null -eq $script:pointer) { $script:pointer = $now }
    $rect = Get-WindowRect
    $middleX = [int][Math]::Floor(($rect.Left + $rect.Right) / 2)
    $middleY = [int][Math]::Floor(($rect.Top + $rect.Bottom) / 2)
    $places = @(
        New-Point $middleX ($rect.Top - 20)
        New-Point ($rect.Left - 20) $middleY
        New-Point ($rect.Right + 20) $middleY
        New-Point $middleX ($rect.Bottom + 20)
    )
    foreach ($place in $places) {
        # MONITOR_DEFAULTTONULL: zero out of the monitors.
        if ($api::MonitorFromPoint($place, 0) -eq [IntPtr]::Zero -or (Test-OnWindow $place)) { continue }
        if ($api::SetCursorPos($place.X, $place.Y)) { return }
    }
}

# Asks Windows to make the window $Width x $Height pixels, with its frame,
# where it is. Windows makes it no smaller than meiga lets it.
function Set-WindowSize([int]$Width, [int]$Height) {
    if (-not $api::SetWindowPos($window, [IntPtr]::Zero, 0, 0, $Width, $Height, $SWP_NOMOVE -bor $SWP_NOZORDER -bor $SWP_NOACTIVATE)) {
        throw "SetWindowPos failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# Does $Action, and checks that the window draws nothing.
function Assert-NothingDrawn([scriptblock]$Action, [string]$What) {
    Wait-Quiet 400
    $from = Get-Mark
    & $Action
    Start-Sleep -Milliseconds 300
    Wait-Quiet 400
    $frames = @(Get-Frames $from)
    if ($frames.Count -gt 0) {
        Show-Lines $frames
        throw "$What drew $($frames.Count) frames instead of none."
    }
}

# Where the client area of the window is on the screen once it is maximized:
# the work area of its monitor, but for a few pixels on the edges where a
# taskbar hides itself (see leave_room_for_hidden_taskbars in
# crates/meiga-win/src/window.rs).
function Get-MaximizedClient {
    $info = [Meiga.Tools.TitleBarCheck+MONITORINFO]::new()
    $info.Size = 40
    # MONITOR_DEFAULTTONEAREST.
    if (-not $api::GetMonitorInfoW($api::MonitorFromWindow($window, 2), [ref]$info)) {
        throw "Could not read the monitor of the window of $exe."
    }
    $edges = $api::AutoHideEdges($info.Monitor)
    $left = $info.Work.Left
    $top = $info.Work.Top
    $right = $info.Work.Right
    $bottom = $info.Work.Bottom
    if ($edges -band 1) { $left += $autoHideGap }
    if ($edges -band 2) { $top += $autoHideGap }
    if ($edges -band 4) { $right -= $autoHideGap }
    if ($edges -band 8) { $bottom -= $autoHideGap }
    New-Rect $left $top $right $bottom
}

# Where the client area of the window is on the screen.
function Get-ClientOnScreen {
    $origin = Get-OnScreen (New-Point 0 0)
    $client = Get-Client
    New-Rect $origin.X $origin.Y ($origin.X + $client.Right) ($origin.Y + $client.Bottom)
}

# Waits until the window is maximized, and checks that it drew one full frame
# after the first $From lines of the log, and that its client area is the
# work area of its monitor; otherwise throws what $What says.
function Wait-Maximized([int]$From, [string]$What) {
    Wait-For { $api::IsZoomed($window) } { "$What did not maximize the window." }
    Wait-OneFull $From $What
    Update-Bar
    $expected = Get-MaximizedClient
    $actual = Get-ClientOnScreen
    if (-not (Test-Same $actual $expected)) {
        throw "$What maximized the window, and its client area is $(Show-Rect $actual) on the screen instead of $(Show-Rect $expected)."
    }
}

# Waits until the window is restored, and checks that it drew one full frame
# after the first $From lines of the log, and that it is where it was, with
# the band above the tabs; otherwise throws what $What says.
function Wait-Restored([int]$From, [string]$What) {
    Wait-For { -not $api::IsZoomed($window) -and -not $api::IsIconic($window) } { "$What did not restore the window." }
    Wait-OneFull $From $What
    Update-Bar
    $now = Get-WindowRect
    if (-not (Test-Same $now $restored) -or $band -le 0) {
        throw "$What restored the window to $(Show-Rect $now), with a band of $band pixels, instead of $(Show-Rect $restored), with one."
    }
    Move-Pointer
}

# The command line of meiga: its frame log, and no program, so that its first
# tab runs the default profile, Light.
$info = [Diagnostics.ProcessStartInfo]::new($Path)
$info.UseShellExecute = $false
$info.WorkingDirectory = $work
$info.Environment['APPDATA'] = $appdata
$info.Environment['MEIGA_CHECK_WORK'] = $work
$info.ArgumentList.Add('--trace-frames')
$info.ArgumentList.Add($log)
$process = $null
$started = [DateTime]::MinValue
$window = [IntPtr]::Zero
$thread = [uint32]0
$dpi = 96
$pointer = $null
$restored = $null
$windowMenu = [IntPtr]::Zero
$passed = $false
try {
    $process = [Diagnostics.Process]::Start($info)
    $started = $process.StartTime
    Wait-For { $api::WindowOf($process.Id, 'meiga') -ne [IntPtr]::Zero } { "$exe did not open its window." }
    $window = $api::WindowOf($process.Id, 'meiga')
    $processId = [uint32]0
    $thread = $api::GetWindowThreadProcessId($window, [ref]$processId)
    $dpi = $api::GetDpiForWindow($window)
    $rowHeight = Get-Scaled 32
    $gap = Get-Scaled 4
    $newSide = Get-Scaled 28
    $profilesSide = Get-Scaled 24
    $scrollSide = Get-Scaled 24
    $dragSide = Get-Scaled 45
    $captionSide = Get-Scaled 46
    $maxTab = Get-Scaled 240
    $minTab = Get-Scaled 100

    # The first tab runs Light, the default profile.
    Wait-For { @(Get-Shells).Count -gt 0 } { 'The first tab started no program.' }
    $first = @(Get-Shells)[0]
    if ($first.CommandLine -cne $lightLine) {
        throw "The first tab runs '$($first.CommandLine)' instead of '$lightLine'."
    }
    $tabs.Add([int]$first.ProcessId)
    Wait-Settled $tabs.ToArray() 'The first tab'
    Update-Bar
    $restored = Get-WindowRect
    Move-Pointer
    Wait-Quiet 400
    Write-Host "The bar is $barHeight pixels high, $band of them above the tabs, and the buttons of the window $captionSide wide."

    # 1. The client area, with the bar, goes up to the top of the window, and
    # Windows keeps its frame on the other sides.
    $rect = Get-WindowRect
    $origin = Get-OnScreen (New-Point 0 0)
    $client = Get-Client
    # SM_CXPADDEDBORDER, and SM_CXSIZEFRAME and SM_CYSIZEFRAME.
    $side = $api::GetSystemMetricsForDpi(92, $dpi) + $api::GetSystemMetricsForDpi(32, $dpi)
    $low = $api::GetSystemMetricsForDpi(92, $dpi) + $api::GetSystemMetricsForDpi(33, $dpi)
    $sides = "left $($origin.X - $rect.Left), top $($origin.Y - $rect.Top), right $($rect.Right - $origin.X - $client.Right) and bottom $($rect.Bottom - $origin.Y - $client.Bottom)"
    if ($origin.Y -ne $rect.Top -or $origin.X - $rect.Left -ne $side -or
        $rect.Right - $origin.X - $client.Right -ne $side -or $rect.Bottom - $origin.Y - $client.Bottom -ne $low) {
        throw "The frame of the window is $sides pixels wide, instead of left $side, top 0, right $side and bottom $low."
    }
    Write-Host "OK: the bar goes up to the top of the window, where a band of $band pixels resizes it, and Windows keeps its frame on the other sides, $side pixels wide."

    # 2. What each part of the restored window is for Windows: its frame, the
    # band and its corner, the free parts of the bar, its buttons and the
    # buttons of the window, the tab and the terminal.
    $middleX = [int][Math]::Floor(($rect.Left + $rect.Right) / 2)
    $middleY = [int][Math]::Floor(($rect.Top + $rect.Bottom) / 2)
    Assert-Hits @(
        New-Hit $rect.Left $rect.Top $HTTOPLEFT
        New-Hit ($rect.Right - 1) $rect.Top $HTTOPRIGHT
        New-Hit $rect.Left $middleY $HTLEFT
        New-Hit ($rect.Right - 1) $middleY $HTRIGHT
        New-Hit $rect.Left ($rect.Bottom - 1) $HTBOTTOMLEFT
        New-Hit $middleX ($rect.Bottom - 1) $HTBOTTOM
        New-Hit ($rect.Right - 1) ($rect.Bottom - 1) $HTBOTTOMRIGHT
    ) 'On the frame of the restored window' -Screen
    # How far from the left side of the client area the band resizes the
    # window also to the left: as far from the left side of the window as on
    # a window of Windows.
    $corner = $topLeftCorner - ($origin.X - $rect.Left)
    $tabX = (Get-Middle (Get-TabRect 0)).X
    $drag = Get-DragPoint
    Assert-Hits @(
        if ($corner -gt 0) {
            New-Hit 0 0 $HTTOPLEFT
            New-Hit ($corner - 1) ($band - 1) $HTTOPLEFT
            New-Hit $corner 0 $HTTOP
        }
        else {
            New-Hit 0 0 $HTTOP
        }
        New-Hit $tabX 0 $HTTOP
        New-Hit $tabX ($band - 1) $HTTOP
        New-Hit $drag.X 0 $HTTOP
        New-Hit $drag.X ($band - 1) $HTTOP
        New-Hit $drag.X $band $HTCAPTION
        New-Hit $drag.X $drag.Y $HTCAPTION
        New-Hit ([int][Math]::Floor($gap / 2)) $midY $HTCAPTION
        New-Hit $tabX $band $HTCLIENT
        New-Hit $tabX $midY $HTCLIENT
        New-Hit (Get-Middle (Get-NewRect)).X $midY $HTCLIENT
        New-Hit (Get-Middle (Get-ProfilesRect)).X $midY $HTCLIENT
        New-Hit $tabX ($barHeight + $rowHeight) $HTCLIENT
        Get-CaptionHits
    ) 'On the restored window'
    Write-Host 'OK: restored, for Windows, the frame resizes the window from its sides and its corners, and the band above the tabs from the top, and from the top left corner next to the left side; the free parts of the bar are the caption, the buttons of the window are those buttons along all their height, also over the band, and the tab, the buttons of the bar and the terminal are the client area.'

    # 3. The pointer on each button of the window draws only that button.
    foreach ($index in 2, 1, 0) {
        Assert-CaptionHover $index
    }
    Write-Host 'OK: the pointer on each button of the window draws only that button.'

    # 4. The right button on the free part of the bar opens the menu of the
    # window, and nothing is drawn while it is open; Esc closes it; Down and
    # Enter choose Maximize.
    Assert-NothingDrawn { $script:windowMenu = Open-WindowMenu } 'The right button on the bar, which opened the menu of the window,'
    Assert-WindowMenu $windowMenu $false 'With the window restored'
    Assert-NothingDrawn { Close-WindowMenu } 'Esc, which closed the menu of the window,'
    $windowMenu = Open-WindowMenu
    Select-WindowCommand $windowMenu 'Maximize'
    $from = Get-Mark
    Send-Key $VK_RETURN
    Wait-Maximized $from 'Enter on Maximize, in the menu of the window,'
    Write-Host 'OK: the right button on the free part of the bar opens the menu of the window, with the commands of a restored window enabled and Close in bold, and nothing is drawn while it is open; Esc closes it, and draws nothing; Down and Enter on Maximize maximize the window, with one full frame, and its client area is the work area of its monitor.'

    # 5. Maximized, the menu of the window has other commands enabled: Down
    # and Enter choose Restore.
    $windowMenu = Open-WindowMenu
    Assert-WindowMenu $windowMenu $true 'With the window maximized'
    Select-WindowCommand $windowMenu 'Restore'
    $from = Get-Mark
    Send-Key $VK_RETURN
    Wait-Restored $from 'Enter on Restore, in the menu of the window,'
    Write-Host 'OK: maximized, the menu of the window has the commands of a maximized window enabled and Close in bold, and Down and Enter on Restore restore the window where it was, with one full frame, and with the band above the tabs.'

    # 6. The button to maximize the window maximizes it: then there is no
    # band, and the top row of the bar is what is below it.
    $from = Get-Mark
    Send-Caption 1
    Wait-Maximized $from 'The button to maximize the window'
    $tabX = (Get-Middle (Get-TabRect 0)).X
    $drag = Get-DragPoint
    Assert-Hits @(
        New-Hit 0 0 $HTCAPTION
        New-Hit ($gap - 1) 0 $HTCAPTION
        New-Hit $gap 0 $HTCLIENT
        New-Hit $tabX 0 $HTCLIENT
        New-Hit (Get-Middle (Get-NewRect)).X 0 $HTCLIENT
        New-Hit $drag.X 0 $HTCAPTION
        New-Hit $drag.X $drag.Y $HTCAPTION
        Get-CaptionHits
    ) 'On the maximized window'
    Write-Host 'OK: the button to maximize the window maximizes it, with one full frame; then there is no band, and the top row of the bar is the caption on the free parts of the bar, the client area on the tab and on the buttons of the bar, and the buttons of the window on them.'

    # 7. The same button restores the window.
    $from = Get-Mark
    Send-Caption 1
    Wait-Restored $from 'The button to restore the window'
    Write-Host 'OK: the same button restores the window where it was, with one full frame.'

    # 8. A double click on the free part of the bar maximizes the window, and
    # another one restores it.
    $from = Get-Mark
    Send-Posted @($WM_NCLBUTTONDBLCLK, $HTCAPTION, (Get-ScreenParam (Get-OnScreen (Get-DragPoint))))
    Wait-Maximized $from 'A double click on the bar'
    $from = Get-Mark
    Send-Posted @($WM_NCLBUTTONDBLCLK, $HTCAPTION, (Get-ScreenParam (Get-OnScreen (Get-DragPoint))))
    Wait-Restored $from 'A double click on the bar of the maximized window'
    Write-Host 'OK: a double click on the free part of the bar maximizes the window, and another one restores it, each with one full frame.'

    # 9. While a menu of meiga is open, a click on the free part of the bar
    # only closes it: there, the bar is the client area, but for the band and
    # the buttons of the window.
    Wait-Quiet 400
    $from = Get-Mark
    Send-Key $VK_APPS
    Wait-For { @(Get-Frames $from).Count -gt 0 } { 'The Menu key did not open the menu of the tab.' }
    Wait-Quiet 400
    $drag = Get-DragPoint
    Assert-Hits @(
        if ($corner -gt 0) { New-Hit 0 0 $HTTOPLEFT } else { New-Hit 0 0 $HTTOP }
        New-Hit $drag.X 0 $HTTOP
        New-Hit $drag.X $drag.Y $HTCLIENT
        New-Hit ([int][Math]::Floor($gap / 2)) $midY $HTCLIENT
        Get-CaptionHits
    ) 'While the menu of the tab is open'
    $from = Get-Mark
    Send-Key $VK_ESCAPE
    Wait-For { @(Get-Frames $from).Count -gt 0 } { 'Esc did not close the menu of the tab.' }
    Wait-Quiet 400
    Assert-Hits @(
        New-Hit $drag.X $drag.Y $HTCAPTION
        New-Hit ([int][Math]::Floor($gap / 2)) $midY $HTCAPTION
    ) 'Once Esc closed the menu of the tab'
    Write-Host 'OK: while the menu of a tab is open, the free parts of the bar are the client area, so that a click on them only closes the menu, but for the band, and the buttons of the window are those buttons; once Esc closes it, the free parts of the bar are the caption again.'

    # 10. The window cannot be narrower than the bar with one tab between the
    # buttons that scroll the strip, nor shorter than the bar. Then, with two
    # tabs, which do not fit, what each part of the bar is.
    Wait-Quiet 400
    $from = Get-Mark
    Set-WindowSize 100 40
    Wait-OneFull $from 'The smallest size of the window'
    Update-Bar
    $narrowest = 3 * $gap + $newSide + $profilesSide + $dragSide + 3 * $captionSide + 2 * $scrollSide + $minTab
    $client = Get-Client
    if ($client.Right -ne $narrowest -or $client.Bottom -ne $barHeight) {
        throw "As small as it can be, the client area of the window is $($client.Right) x $($client.Bottom) pixels instead of $narrowest x $barHeight."
    }
    $at = Get-Middle (Get-NewRect)
    Send-Posted @(
        Get-Mouse $WM_MOUSEMOVE 0 $at
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $at
        Get-Mouse $WM_LBUTTONUP 0 $at
    )
    Wait-For { @(Get-Shells).Count -gt 1 } { 'The button + did not open a tab.' }
    $second = @(Get-Shells | Where-Object { [int]$_.ProcessId -notin $tabs })[0]
    $tabs.Add([int]$second.ProcessId)
    Wait-Settled $tabs.ToArray() 'The second tab'
    Wait-Quiet 400
    Update-Fit
    if (-not $barOverflow) {
        throw "At $dpi DPI, two tabs fit in the window as small as it can be, and the check needs them not to."
    }
    # The tab that is shown, the new one, is in view: the strip is at its end.
    $barScroll = Get-MaxScroll
    $strip = Get-Strip
    $right = Get-ScrollRect 'Right'
    $shown = Get-Intersection (Get-TabRect 1) $strip
    $drag = Get-DragPoint
    Assert-Hits @(
        New-Hit (Get-Middle (Get-ScrollRect 'Left')).X $midY $HTCLIENT
        New-Hit (Get-Middle $right).X $midY $HTCLIENT
        New-Hit (Get-Middle $shown).X $midY $HTCLIENT
        New-Hit (Get-Middle (Get-NewRect)).X $midY $HTCLIENT
        New-Hit (Get-Middle (Get-ProfilesRect)).X $midY $HTCLIENT
        New-Hit ([int][Math]::Floor($gap / 2)) $midY $HTCAPTION
        New-Hit ($right.Right + [int][Math]::Floor($gap / 2)) $midY $HTCAPTION
        New-Hit $drag.X $drag.Y $HTCAPTION
        New-Hit (Get-Middle $strip).X 0 $HTTOP
        Get-CaptionHits
    ) 'With two tabs that do not fit in the window as small as it can be'
    # The middle button closes the second tab, and the window gets its size
    # back.
    $at = Get-Middle $shown
    Send-Posted @(
        Get-Mouse $WM_MOUSEMOVE 0 $at
        Get-Mouse $WM_MBUTTONDOWN $MK_MBUTTON $at
        Get-Mouse $WM_MBUTTONUP 0 $at
    )
    Wait-Ended $tabs[1] 'The program of the second tab, which the middle button closed,'
    $tabs.RemoveAt(1)
    Update-Fit
    Wait-Quiet 400
    $from = Get-Mark
    Set-WindowSize ($restored.Right - $restored.Left) ($restored.Bottom - $restored.Top)
    Wait-OneFull $from 'Giving the window its size back'
    Update-Bar
    $now = Get-WindowRect
    if (-not (Test-Same $now $restored)) {
        throw "With its size back, the window is at $(Show-Rect $now) instead of $(Show-Rect $restored)."
    }
    Move-Pointer
    Write-Host "OK: the window cannot be narrower than the bar with one tab between the buttons that scroll the strip, nor shorter than the bar: its client area is then $narrowest x $barHeight pixels; with two tabs, which do not fit, the buttons that scroll the strip, the tab in it, and the buttons + and of the profiles are the client area, and the gaps and the room to drag the window are the caption."

    # 11. Losing and getting the focus draws only the buttons of the window.
    Wait-Quiet 400
    $from = Get-Mark
    Send-Posted @($WM_KILLFOCUS, 0, 0, $WM_SETFOCUS, 0, 0)
    Wait-For { @(Get-Frames $from).Count -gt 0 } { 'Losing and getting the focus drew nothing.' }
    Wait-Quiet 400
    $frames = @(Get-Frames $from)
    $areas = @(Get-Areas $frames)
    $captions = Get-CaptionsRect
    if (@($areas | Where-Object { -not (Test-Inside $_ $captions) }).Count -gt 0 -or
        @($areas | Where-Object { Test-Same $_ $captions }).Count -eq 0) {
        Show-Lines $frames
        throw "Losing and getting the focus did not draw only the buttons of the window, $(Show-Rect $captions): $(@($areas | ForEach-Object { Show-Rect $_ }) -join ', ')."
    }
    Write-Host 'OK: losing and getting the focus draws only the buttons of the window.'

    # 12. The button to minimize the window minimizes it, and nothing is drawn
    # while it is minimized; restored, it is painted whole.
    Send-Caption 2
    Wait-For { $api::IsIconic($window) } { 'The button to minimize the window did not minimize it.' }
    Assert-NothingDrawn { Send-Posted @($WM_KILLFOCUS, 0, 0, $WM_SETFOCUS, 0, 0) } 'While the window was minimized, losing and getting the focus'
    $from = Get-Mark
    Send-Posted @($WM_SYSCOMMAND, $SC_RESTORE, 0)
    Wait-For { -not $api::IsIconic($window) } { 'SC_RESTORE did not restore the window.' }
    Wait-For { @(Get-Log | Select-Object -Skip $from | Where-Object { $_.kind -eq 'paint' -and (Test-Full $_) }).Count -gt 0 } { 'Restoring the window did not paint all of it.' }
    Wait-Quiet 400
    Update-Bar
    $now = Get-WindowRect
    if (-not (Test-Same $now $restored)) {
        throw "Restored, the window is at $(Show-Rect $now) instead of $(Show-Rect $restored)."
    }
    Move-Pointer
    Write-Host 'OK: the button to minimize the window minimizes it, and nothing is drawn while it is minimized, also when it loses and gets the focus; restored, it is where it was, and it is painted whole.'

    # 13. The button to close the window ends meiga, and the programs of its
    # tabs.
    $shells = @(Get-Shells)
    $tabConsoles = @(Get-Consoles)
    Send-Caption 0
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after its window closed."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    foreach ($shell in $shells) {
        Wait-Ended ([int]$shell.ProcessId) "The program $($shell.Name) of a tab"
    }
    Wait-ConsolesEnded $tabConsoles
    Write-Host 'OK: the button to close the window ends meiga with exit code 0, and the programs of its tabs and their consoles end too.'
    $passed = $true
}
finally {
    if ($process) {
        if (-not $process.HasExited) {
            # With the programs of its tabs, and their consoles.
            try { $process.Kill($true) }
            catch { Write-Host "Could not end $exe and the programs of its tabs: $($_.Exception.Message)" }
        }
        $process.Dispose()
    }
    if ($null -ne $pointer) {
        $null = $api::SetCursorPos($pointer.X, $pointer.Y)
    }
    $null = $api::SetThreadDpiAwarenessContext($previousDpi)
    if ($passed) {
        Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $work) {
        Write-Host "Kept $work"
    }
}
