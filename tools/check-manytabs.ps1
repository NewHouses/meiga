<#
.SYNOPSIS
Checks that meiga.exe keeps 30 tabs usable: the tabs keep a narrowest width,
the bar scrolls them with the wheel and with its buttons, the mouse can close
one tab after another without moving, and a hidden tab tells its activity,
its bell and the end of its program.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and an APPDATA of its
own, whose config.toml has two profiles, Light, which runs cmd with a hidden
cursor, and Probe, the default one, which runs a PowerShell script that
writes the text that the check gives it, and ends when the check tells it
to; and hides the shells that meiga finds. The window gets a client area of
983 x 400 pixels at a scale of 100 %, so that 7 tabs fit in the bar, beside
its buttons, the room to drag the window and the buttons of the window, and
8 do not.

The check computes the layout of the bar as meiga does (see
crates/meiga-core/src/ui/tabbar.rs), and follows the tab that is shown by
the priority of the reader thread of each session. It posts the clicks, the
wheel and the keys to the window, as check-tabbar.ps1 does, and reads what
the window draws in its frame log. It checks that:

- with 7 tabs, they shrink alike, and fit in the bar, with the button +
  after them: the pointer on the last tab, on the button +, and on the first
  tab draws only that part;
- an eighth tab does not fit: the tabs keep their narrowest width, and the
  strip, between the buttons that scroll it, shows the new one, which is
  shown; the buttons + and of the profiles go on the right of the bar;
- with 30 tabs, each one runs its own program, nothing overlaps, and the
  strip shows the last one, which ends before the button that scrolls the
  strip to the right; that button cannot scroll it further, and the pointer
  on it, and a click on it, draw nothing;
- the wheel on the bar scrolls a tab for each step, towards the first one
  forward or to the left, and towards the last one backward or to the right,
  and keeps what is left of a step for the next turn;
- a click on a button that scrolls the strip scrolls it until the first tab
  that it hides on that side, even in part, is all in view;
- while the left button stays down on a button that scrolls the strip, with
  the pointer on it, the strip scrolls another tab after 250 ms, and then
  after each 100 ms. If the check cannot move the pointer of the system onto
  the button, as with the client of the Remote Desktop minimized, it checks
  instead that the strip scrolls once, and no more once the pointer leaves
  the button;
- each of the 30 tabs can be shown with a click on its title, once the wheel
  brings it into view;
- when the close button or the middle button closes a tab, the others keep
  their width and their place while the pointer stays on the bar, and the
  next close button is under the pointer; once the pointer leaves the bar,
  they get their width again, and the strip shows the tab that is shown;
- a tab that the keyboard closes, with the pointer on the bar, lets the
  others get their width at once; then a tab that the mouse closes makes
  them keep it;
- the last tab, closed with the mouse, lets the others get their width at
  once, so the close button of the new last one comes under the pointer;
- showing a tab with the keyboard, also round the bar, scrolls the strip so
  that it is in view; the wheel can then move the tab that is shown out of
  view, where it stays until the tab is chosen again: Ctrl+1 and Ctrl+9, and
  a click on the part of it that is in view, scroll the strip until it is all
  in view, and Ctrl+9 on the tab all in view draws nothing; Shift+F10 also
  scrolls the strip until the tab is in view, and opens its menu below it;
- while a tab is hidden, its program writes, which draws only the title of
  its tab, once, and rings the bell, which draws it once more; showing the
  tab clears both, and a bell while it is shown draws nothing; the end of
  its program with exit code 3, while it is hidden, draws its title once
  more (ADR 0025);
- it ends with exit code 0 when its window closes, and the programs of its
  tabs and their consoles end too.

It needs PowerShell 7 (pwsh) in the PATH, for the script of Probe.

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
if (-not $Path) {
    $Path = Join-Path $PSScriptRoot '..\target\release\meiga.exe'
}
$Path = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path)
if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "$Path does not exist. Build it first with: cargo build --release"
}
$exe = Split-Path -Leaf $Path

if (-not ('Meiga.Tools.ManyTabsCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name ManyTabsCheck -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)]
public struct RECT { public int Left, Top, Right, Bottom; }

[StructLayout(LayoutKind.Sequential)]
public struct POINT { public int X, Y; }

[StructLayout(LayoutKind.Sequential)]
public struct MSG { public IntPtr hwnd; public uint message; public UIntPtr wParam; public IntPtr lParam; public uint time; public int x, y; }

public delegate bool EnumProc(IntPtr window, IntPtr parameter);

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

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetClientRect(IntPtr window, out RECT rect);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetWindowRect(IntPtr window, out RECT rect);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);

[DllImport("user32.dll")]
public static extern uint GetDpiForWindow(IntPtr window);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsZoomed(IntPtr window);

[DllImport("user32.dll")]
public static extern int GetSystemMetricsForDpi(int index, uint dpi);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetCursorPos(out POINT point);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetCursorPos(int x, int y);

[DllImport("user32.dll")]
public static extern IntPtr WindowFromPoint(POINT point);

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

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool EnumWindows(EnumProc callback, IntPtr parameter);

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern int GetClassNameW(IntPtr window, System.Text.StringBuilder name, int count);

[DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
static extern uint GetLongPathNameW(string path, System.Text.StringBuilder longPath, uint length);

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
// reads the posted messages first. Unless `state` is null, the thread of
// the window gets it as its key state, which this thread shares: the key
// state changes as the thread reads its input, so it is set while the
// thread is suspended, for the keys of the messages. The messages of the
// mouse leave it as it is.
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

// The priorities of the threads of a process whose names start with
// `prefix`, by name, or null if the process has ended.
public static System.Collections.Generic.Dictionary<string, int> PrioritiesOf(int process, string prefix)
{
    System.Diagnostics.Process target;
    try { target = System.Diagnostics.Process.GetProcessById(process); }
    catch (ArgumentException) { return null; }
    var found = new System.Collections.Generic.Dictionary<string, int>();
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
                if (description != null && description.StartsWith(prefix, StringComparison.Ordinal))
                    found[description] = GetThreadPriority(handle);
            }
            finally { CloseHandle(handle); }
        }
    }
    return found;
}

static string ClassOf(IntPtr window)
{
    var name = new System.Text.StringBuilder(256);
    GetClassNameW(window, name, name.Capacity);
    return name.ToString();
}

// The top-level window of the class `name` of the process `process`, also a
// hidden one, or zero.
public static IntPtr WindowOf(int process, string name)
{
    IntPtr found = IntPtr.Zero;
    EnumWindows((window, parameter) =>
    {
        uint owner;
        GetWindowThreadProcessId(window, out owner);
        if (owner != (uint)process || ClassOf(window) != name) return true;
        found = window;
        return false;
    }, IntPtr.Zero);
    return found;
}

// The long form of the path of a folder that exists: with the long names of
// the folders that have a short one too, as RUNNER~1.
public static string LongPathOf(string path)
{
    var longPath = new System.Text.StringBuilder(32768);
    uint length = GetLongPathNameW(path, longPath, (uint)longPath.Capacity);
    if (length == 0 || length >= longPath.Capacity) throw new System.ComponentModel.Win32Exception();
    return longPath.ToString();
}
'@
}
$api = [Meiga.Tools.ManyTabsCheck]

$WM_CLOSE = 0x0010
$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_SYSKEYDOWN = 0x0104
$WM_SYSKEYUP = 0x0105
$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_MBUTTONDOWN = 0x0207
$WM_MBUTTONUP = 0x0208
$WM_MOUSEWHEEL = 0x020A
$WM_MOUSEHWHEEL = 0x020E
$MK_LBUTTON = 0x0001
$MK_MBUTTON = 0x0010
$WHEEL_DELTA = 120
$SWP_NOMOVE = 0x0002
$SWP_NOZORDER = 0x0004
$SWP_NOACTIVATE = 0x0010
$MAPVK_VK_TO_VSC = 0
$VK_CONTROL = 0x11
$VK_MENU = 0x12
$VK_F10 = 0x79
$THREAD_PRIORITY_NORMAL = 0
$THREAD_PRIORITY_BELOW_NORMAL = -1
# The name of the reader thread of a session, before the number of its
# program (see crates/meiga-win/src/conpty.rs).
$readerPrefix = 'conpty reader '
# The console of each tab: the one of the ConPTY of meiga, or the one of
# Windows if the ConPTY is not next to meiga.exe.
$consoles = 'OpenConsole.exe', 'conhost.exe'
# The modifier keys of the chords: the keys whose messages go down and up,
# each one with whether it is an extended key, and the keys that they set
# down in the key state.
$chordKeys = @{
    Ctrl = , @{ Code = 0x11; Extended = $false; Down = 0x11, 0xA2 }
    Shift = , @{ Code = 0x10; Extended = $false; Down = 0x10, 0xA0 }
    Alt = , @{ Code = 0x12; Extended = $false; Down = 0x12, 0xA4 }
}
# The keys that a keyboard sends as extended ones: Page Up, Page Down, End,
# Home, the arrows, Insert and Delete.
$extendedKeys = @(0x21..0x28) + @(0x2D, 0x2E)

# The script of the tabs of Probe. It hides the cursor, so that it never
# blinks, and then writes the text that the check gives it, each time, and a
# file with how many times it did, until the check tells it how to end: with
# an exit code, after some text or none.
$program = @'
param([Parameter(Mandatory)][string]$Work)
$ErrorActionPreference = 'Stop'
[Console]::Out.Write("$([char]27)[?25l")
[Console]::Out.Flush()
# The check sets it once it has written how this script ends.
$stop = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, "meiga-check-manytabs-$PID")
# And this one once it has written some text for this script to write.
$write = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::AutoReset, "meiga-check-manytabs-write-$PID")
$written = 0
[IO.File]::WriteAllText((Join-Path $Work "ready-$PID.txt"), '')
while ($true) {
    if ([Threading.WaitHandle]::WaitAny([Threading.WaitHandle[]]@($stop, $write)) -eq 0) {
        # Its exit code, and the text to write before, if there is one.
        $end = [IO.File]::ReadAllLines((Join-Path $Work "exit-$PID.txt"))
        if ($end.Count -gt 1) {
            [Console]::Out.Write($end[1])
            [Console]::Out.Flush()
        }
        exit [int]$end[0]
    }
    [Console]::Out.Write([IO.File]::ReadAllText((Join-Path $Work "write-$PID.txt")))
    [Console]::Out.Flush()
    $written++
    [IO.File]::WriteAllText((Join-Path $Work "written-$PID.txt"), "$written")
}
'@

$name = "meiga-check-manytabs-$PID"
# All the files of the check, with the APPDATA of meiga. With the long names
# of the folders, which .NET gives as the current one.
$work = Join-Path ($api::LongPathOf([IO.Path]::GetTempPath())) $name
$appdata = Join-Path $work 'appdata'
$config = Join-Path $appdata 'meiga\config.toml'
$script = Join-Path $work 'probe.ps1'
$log = Join-Path $work 'frames.csv'
Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
$null = New-Item -ItemType Directory -Force -Path (Split-Path $config)
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText($script, $program, $utf8)

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

# The configuration: Light, the first profile of the menu, which Ctrl+Shift+1
# opens, runs cmd with a prompt that only hides the cursor, so that its tabs
# cost little; Probe, the default profile, runs the script above; and the
# shells that meiga finds, hidden.
$lightCommand = '%SystemRoot%\System32\cmd.exe /d /q /k prompt $E[?25l'
$probeCommand = 'pwsh -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%MEIGA_CHECK_WORK%\probe.ps1" -Work "%MEIGA_CHECK_WORK%"'
$configLines = @(
    '# Written by tools/check-manytabs.ps1.'
    'default_profile = "Probe"'
    ''
    '[[profiles]]'
    'name = "Light"'
    "command = '$lightCommand'"
    "directory = '%MEIGA_CHECK_WORK%'"
    ''
    '[[profiles]]'
    'name = "Probe"'
    "command = '$probeCommand'"
    "directory = '%MEIGA_CHECK_WORK%'"
    foreach ($shell in $hidden) {
        ''
        '[[profiles]]'
        "name = $(ConvertTo-TomlString $shell)"
        'hidden = true'
    }
)
[IO.File]::WriteAllText($config, ($configLines -join "`n") + "`n", $utf8)

# What the tabs of Light and of Probe run.
$lightLine = $lightCommand.Replace('%SystemRoot%', $env:SystemRoot)
$probeLine = $probeCommand.Replace('%MEIGA_CHECK_WORK%', $work)

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

# The priorities of the reader threads of the sessions of meiga, by name,
# or null if meiga has ended.
function Get-Readers {
    $api::PrioritiesOf($process.Id, $readerPrefix)
}

# The priority of the reader thread of the tab whose program is $Program,
# or $null if it has none.
function Get-Priority([int]$Program) {
    $readers = Get-Readers
    $priority = 0
    if ($null -eq $readers -or -not $readers.TryGetValue("$readerPrefix$Program", [ref]$priority)) { return $null }
    $priority
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

# How many times the script of the tab of $Id has written the text that the
# check gave it.
function Get-Written([int]$Id) {
    try { [int][IO.File]::ReadAllText((Join-Path $work "written-$Id.txt")) } catch { 0 }
}

# Tells the script of the tab of $Id to write $Text, without a new line, and
# waits until it has.
function Write-Probe([int]$Id, [string]$Text) {
    $before = Get-Written $Id
    [IO.File]::WriteAllText((Join-Path $work "write-$Id.txt"), $Text, $utf8)
    $write = [Threading.EventWaitHandle]::OpenExisting("meiga-check-manytabs-write-$Id")
    try { $null = $write.Set() }
    finally { $write.Dispose() }
    Wait-For { (Get-Written $Id) -gt $before } { 'The script of a tab did not write its text.' }
}

# Tells the script of the tab of $Id to end with exit code $Code.
function Stop-Probe([int]$Id, [int]$Code) {
    [IO.File]::WriteAllText((Join-Path $work "exit-$Id.txt"), "$Code`n", $utf8)
    $stop = [Threading.EventWaitHandle]::OpenExisting("meiga-check-manytabs-$Id")
    try { $null = $stop.Set() }
    finally { $stop.Dispose() }
}

# Waits until the program of the tab of $Id has ended, and the reader thread
# of its session too, whose last work is to post the wake-up that tells the
# window that the program ended, which it ignores if it closed the tab
# already: what the check posts to the window from then on comes after it.
# The files of its script go too, so that the check does not take them for
# the ones of a program that gets the same number later.
function Wait-Gone([int]$Id, [string]$What) {
    Wait-Ended $Id $What
    Wait-For { $null -eq (Get-Priority $Id) } { "$What ended, and the reader thread of its session did not." }
    foreach ($file in "ready-$Id.txt", "exit-$Id.txt", "write-$Id.txt", "written-$Id.txt") {
        Remove-Item -LiteralPath (Join-Path $work $file) -ErrorAction SilentlyContinue
    }
}

# The client area of the window.
function Get-Client {
    $client = [Meiga.Tools.ManyTabsCheck+RECT]::new()
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

# Gives the window a client area of $Width x $Height pixels.
function Resize-Window([int]$Width, [int]$Height) {
    $client = Get-Client
    $outer = [Meiga.Tools.ManyTabsCheck+RECT]::new()
    if (-not $api::GetWindowRect($window, [ref]$outer)) {
        throw "Could not read the size of the window of $exe."
    }
    $outerWidth = $Width + ($outer.Right - $outer.Left - $client.Right)
    $outerHeight = $Height + ($outer.Bottom - $outer.Top - $client.Bottom)
    if (-not $api::SetWindowPos($window, [IntPtr]::Zero, 0, 0, $outerWidth, $outerHeight, $SWP_NOMOVE -bor $SWP_NOZORDER -bor $SWP_NOACTIVATE)) {
        throw "SetWindowPos failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
    $client = Get-Client
    if ($client.Right -ne $Width -or $client.Bottom -ne $Height) {
        throw "The window of $exe has a client area of $($client.Right) x $($client.Bottom) pixels instead of $Width x $Height."
    }
}

# The pixel in the middle of $Rect.
function Get-Middle($Rect) {
    [pscustomobject]@{ X = [int][Math]::Floor(($Rect.Left + $Rect.Right) / 2); Y = [int][Math]::Floor(($Rect.Top + $Rect.Bottom) / 2) }
}

# $Point of the client area on the screen.
function Get-OnScreen($Point) {
    $screen = [Meiga.Tools.ManyTabsCheck+POINT]::new()
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

# A message of the wheel, $Delta, at a point of the client area: the
# message, its wParam and its lParam, which has the point on the screen, as
# Windows gives it.
function Get-Wheel([int]$Message, [int]$Delta, $Point) {
    $screen = Get-OnScreen $Point
    $Message, (([long]$Delta -band 0xFFFF) -shl 16), (([int]$screen.X -band 0xFFFF) -bor (([int]$screen.Y -band 0xFFFF) -shl 16))
}

# Posts $Messages to the window at once, with the state of the keys of
# $State, if it is not null: see PostAll. If meiga has ended, it says so,
# instead of the error of its thread, which is not there any more.
function Send-Posted([long[]]$Messages, [byte[]]$State) {
    try {
        $api::PostAll($window, $thread, $Messages, $State)
    }
    catch {
        if ($process.WaitForExit(2000)) {
            throw "$exe ended with exit code $($process.ExitCode), and the check could not post its keys or its clicks to its window."
        }
        throw
    }
}

# Posts mouse messages at once.
function Send-Mouse([long[]]$Messages) {
    Send-Posted $Messages $null
}

# If the pointer is over the window, moves it out, above the window: the
# moves that Windows makes at its place would reach the bar, which would draw.
function Move-Pointer {
    $point = [Meiga.Tools.ManyTabsCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.ManyTabsCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = [Meiga.Tools.ManyTabsCheck+RECT]::new()
    if (-not $api::ScreenToClient($window, [ref]$inside) -or -not $api::GetClientRect($window, [ref]$rect) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = Get-OnScreen ([pscustomobject]@{ X = [Math]::Floor($rect.Right / 2); Y = -8 })
    if (-not $api::SetCursorPos($title.X, $title.Y)) {
        throw "Could not move the pointer out of the window: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# Moves the pointer of the system onto $Point of the client area, and
# returns whether it is there, on the window. With the client of the Remote
# Desktop minimized, Windows does not let a program read nor move it.
function Set-RealPointer($Point) {
    $now = [Meiga.Tools.ManyTabsCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$now)) { return $false }
    if ($null -eq $script:pointer) { $script:pointer = $now }
    $screen = Get-OnScreen $Point
    if (-not $api::SetCursorPos($screen.X, $screen.Y)) { return $false }
    $at = [Meiga.Tools.ManyTabsCheck+POINT]::new()
    $api::GetCursorPos([ref]$at) -and $at.X -eq $screen.X -and $at.Y -eq $screen.Y -and
    $api::WindowFromPoint($screen) -eq $window
}

# Shares the input of this thread with the one of the window, whose key
# state Send-Keys sets: SetKeyboardState sets the key state of the thread
# that calls it. This thread needs a message queue, which PeekMessageW
# creates.
function Join-Input {
    $message = [Meiga.Tools.ManyTabsCheck+MSG]::new()
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
# lParam, with the scan code of the key; as a system key if $System, with
# the context code, Alt down; and with the bit of the extended keys if
# $Extended. F10 is a system key also without Alt, as Windows sends it.
function Get-Key([int]$Code, [bool]$Down, [bool]$System = $false, [bool]$Extended = $false) {
    $scan = $api::MapVirtualKeyExW($Code, $MAPVK_VK_TO_VSC, $layout)
    [long]$bits = 1 -bor ($scan -shl 16)
    if ($Extended) { $bits = $bits -bor 0x01000000L }
    if ($System) { $bits = $bits -bor 0x20000000L }
    $asSystem = $System -or $Code -eq $VK_F10
    if ($Down) {
        $message = if ($asSystem) { $WM_SYSKEYDOWN } else { $WM_KEYDOWN }
    }
    else {
        # The key was down (bit 30), and goes up (bit 31).
        $bits = $bits -bor 0xC0000000L
        $message = if ($asSystem) { $WM_SYSKEYUP } else { $WM_KEYUP }
    }
    $message, $Code, $bits
}

# The key messages of typing $Chord, such as "Ctrl+Shift+T", "Ctrl+1" or
# "Shift+F10", with the names of System.ConsoleKey for the keys that are not
# a letter nor a digit, and the keys that are down meanwhile: its modifier
# keys go down, its key goes down and up, and its modifier keys go up, the
# other way round. With Alt and without Ctrl, they are system keys, as
# Windows sends them, but for Alt going up: after another key, Windows sends
# it as a key that is not a system one, and then it does not open the menu
# of the window.
function Get-Chord([string]$Chord) {
    $parts = $Chord -split '\+'
    $modifiers = @(foreach ($part in ($parts | Select-Object -SkipLast 1)) {
            if (-not $chordKeys.ContainsKey($part)) { throw "$part, in $Chord, is not a modifier key of the check." }
            $chordKeys[$part]
        })
    $down = @($modifiers | ForEach-Object { $_.Down })
    $system = $down -contains $VK_MENU -and $down -notcontains $VK_CONTROL
    $key = $parts[-1]
    $code = if ($key -cmatch '^[0-9A-Z]$') { [int][char]$key } else { [int][ConsoleKey]$key }
    $extended = $code -in $extendedKeys
    $reversed = @(for ($index = $modifiers.Count - 1; $index -ge 0; $index--) { $modifiers[$index] })
    $messages = @(
        foreach ($modifier in $modifiers) { Get-Key $modifier.Code $true $system $modifier.Extended }
        Get-Key $code $true $system $extended
        Get-Key $code $false $system $extended
        foreach ($modifier in $reversed) { Get-Key $modifier.Code $false ($system -and $modifier.Code -ne $VK_MENU) $modifier.Extended }
    )
    [pscustomobject]@{ Messages = [long[]]$messages; Down = [int[]]$down }
}

# Posts key messages at once, with the keys of $Down down and no other one
# (see PostAll). The input has to be shared with the thread of the window:
# see Join-Input.
function Send-Keys([long[]]$Messages, [int[]]$Down = @()) {
    $state = [byte[]]::new(256)
    foreach ($key in $Down) { $state[$key] = 0x80 }
    Send-Posted $Messages $state
}

# Types $Chord (see Get-Chord).
function Send-Chord([string]$Chord) {
    $typed = Get-Chord $Chord
    Send-Keys $typed.Messages $typed.Down
}

# The bar, as meiga lays it out (see crates/meiga-core/src/ui/tabbar.rs): its
# sizes, once the window is known, with the band above its row of tabs, where
# the window is resized; the width of its tabs, whether they do not fit, and
# how far the strip scrolls them; what is left of the steps of the wheel (see
# crates/meiga/src/tabbar.rs); and its tabs, by the number of their programs,
# with the one that is shown.
$barWidth = 0
$band = 0
$rowHeight = 0
$barHeight = 0
$gap = 0
$padding = 0
$closeSide = 0
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
$barRest = 0
$tabs = [Collections.Generic.List[int]]::new()
$active = 0

# The band above the row of tabs, where the pointer resizes the window, as
# meiga computes it (see crates/meiga-win/src/window.rs): as high as the frame
# that Windows gives a window to resize it, while the window has the title bar
# of meiga and is not maximized, and none otherwise. With the title bar of
# meiga, the client area goes up to the top of the window.
function Get-Band {
    if ($api::IsZoomed($window)) { return 0 }
    $rect = [Meiga.Tools.ManyTabsCheck+RECT]::new()
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

# What the pointer draws when it comes onto the tab at $Index, or leaves it:
# its column of the row of tabs, with the separator on its right, in the
# strip.
function Get-TabArea([int]$Index) {
    $tab = Get-TabRect $Index
    Get-Intersection (New-Rect $tab.Left $band ($tab.Right + 1) $barHeight) (Get-Strip)
}

# The close button of the tab at $Index, or $null if it is too narrow.
function Get-CloseRect([int]$Index) {
    $tab = Get-TabRect $Index
    $right = $tab.Right - $gap
    $left = $right - $closeSide
    if ($left -lt $tab.Left + $gap) { return $null }
    $top = $tab.Top + [Math]::Floor(($rowHeight - $gap - $closeSide) / 2)
    New-Rect $left $top $right ($top + $closeSide)
}

function Get-TitleRect([int]$Index) {
    $tab = Get-TabRect $Index
    $left = $tab.Left + $padding
    $close = Get-CloseRect $Index
    $right = if ($null -ne $close) { $close.Left } else { $tab.Right - $padding }
    New-Rect $left $tab.Top ([Math]::Max($right, $left)) $tab.Bottom
}

# What the title of the tab at $Index draws when it changes: its room, in the
# strip.
function Get-LabelArea([int]$Index) {
    Get-Intersection (Get-TitleRect $Index) (Get-Strip)
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

# Scrolls the strip to $Target, or as far as it can. Returns whether it
# scrolled.
function Set-BarScroll([long]$Target) {
    $scroll = [Math]::Min([Math]::Max(0, $Target), (Get-MaxScroll))
    $moved = $scroll -ne $barScroll
    $script:barScroll = [int]$scroll
    $moved
}

# Scrolls the strip as a button does, towards $Side: until the first tab
# that it hides on that side, even in part, is all in view.
function Step-Bar([string]$Side) {
    if ($Side -eq 'Left') {
        $target = [Math]::Max(0, [Math]::Ceiling($barScroll / $barTabWidth) - 1) * $barTabWidth
    }
    else {
        $hidden = [Math]::Floor(($barScroll + (Get-Viewport)) / $barTabWidth)
        $target = [Math]::Max(0, ($hidden + 1) * $barTabWidth - (Get-Viewport))
    }
    Set-BarScroll $target
}

# Scrolls the strip as little as it can so that the tab at $Index is all in
# view, as meiga does before a frame once a tab is shown, and at once when the
# tab that is shown is chosen again or before it opens the menu of a tab with
# the keyboard. Returns whether it scrolled.
function Show-BarTab([int]$Index) {
    if (-not $barOverflow -or $Index -ge $tabs.Count) { return $false }
    $left = $Index * $barTabWidth
    $right = $left + $barTabWidth
    $viewport = Get-Viewport
    if ($left -lt $barScroll) { return Set-BarScroll $left }
    if ($right -gt $barScroll + $viewport) { return Set-BarScroll ([Math]::Min($right - $viewport, $left)) }
    $false
}

# The wheel turns $Delta 120ths of a step, forward or, $Horizontal, to the
# right, if it is positive: a tab for each step, towards the first one
# forward, and what is left of a step stays for the next turn.
function Add-Wheel([int]$Delta, [bool]$Horizontal) {
    if (-not $barOverflow) {
        $script:barRest = 0
        return
    }
    $towardsLast = if ($Horizontal) { $Delta } else { -$Delta }
    [long]$turn = $barRest + [long]$towardsLast * $barTabWidth
    # As in Rust, the division truncates, and the rest has the sign of the
    # turn.
    [long]$pixels = [Math]::Truncate($turn / $WHEEL_DELTA)
    $script:barRest = $turn - $pixels * $WHEEL_DELTA
    $null = Set-BarScroll ($barScroll + $pixels)
}

# Whether the tab at $Index is all in view.
function Test-AllInView([int]$Index) {
    if (-not $barOverflow) { return $true }
    $left = $Index * $barTabWidth
    $left -ge $barScroll -and $left + $barTabWidth -le $barScroll + (Get-Viewport)
}

# The first tab that is all in view and not shown.
function Get-OtherTab {
    for ($index = 0; $index -lt $tabs.Count; $index++) {
        if ($index -ne $active -and (Test-AllInView $index)) { return $index }
    }
    throw 'The check found no tab all in view but the one that is shown.'
}

# How many steps of the wheel forward, or backward if they are negative,
# bring the tab at $Index all in view.
function Get-StepsTo([int]$Index) {
    if ($barRest -ne 0) { throw 'The check counts the steps of the wheel without a rest.' }
    $left = $Index * $barTabWidth
    $right = $left + $barTabWidth
    if ($left -lt $barScroll) { return [int][Math]::Ceiling(($barScroll - $left) / $barTabWidth) }
    $end = $barScroll + (Get-Viewport)
    if ($right -gt $end) { return -1 * [int][Math]::Ceiling(($right - $end) / $barTabWidth) }
    0
}

# The tab whose close button is at $Point, as meiga finds it, or -1.
function Get-CloseAt($Point) {
    $across = { param($Rect) $Point.X -ge $Rect.Left -and $Point.X -lt $Rect.Right }
    if ($Point.X -lt 0 -or $Point.X -ge $barWidth -or $Point.Y -lt 0 -or $Point.Y -ge $barHeight) { return -1 }
    # The buttons of the window take all the height of the bar, and the band
    # is above the tabs and the other buttons.
    if ((& $across (Get-CaptionsRect)) -or $Point.Y -lt $band) { return -1 }
    if ((& $across (Get-NewRect)) -or (& $across (Get-ProfilesRect))) { return -1 }
    if ($barOverflow -and ((& $across (Get-ScrollRect 'Left')) -or (& $across (Get-ScrollRect 'Right')))) { return -1 }
    $fromStart = $Point.X - (Get-Start)
    if (-not (& $across (Get-Strip)) -or $fromStart -lt 0) { return -1 }
    $index = [int][Math]::Floor($fromStart / $barTabWidth)
    if ($index -ge $tabs.Count) { return -1 }
    $close = Get-CloseRect $index
    if ($null -eq $close -or $Point.Y -lt $close.Top -or $Point.Y -ge $close.Bottom -or -not (& $across $close)) { return -1 }
    $index
}

# The tab at $Index closes, and the one that is shown is the one on its
# right or, after the last one, the one on its left. The others keep their
# width and their place, as when the mouse closes it with the pointer on the
# bar: Update-Fit gives them their width again.
function Remove-Tab([int]$Index) {
    $tabs.RemoveAt($Index)
    if ($Index -lt $active -or ($Index -eq $active -and $Index -eq $tabs.Count)) {
        $script:active--
    }
}

# Waits until a tab opens and is shown: meiga has a reader thread for a
# program that is not one of the tabs, with the normal priority of the tab
# that is shown, and the readers of the tabs have the lower one. Returns the
# number of its program.
function Wait-NewReader([string]$What) {
    $script:newReader = 0
    Wait-For {
        $readers = Get-Readers
        if ($null -eq $readers) { return $false }
        $fresh = @($readers.Keys | Where-Object { [int]$_.Substring($readerPrefix.Length) -notin $tabs })
        if ($fresh.Count -gt 1) { throw "$What opened $($fresh.Count) tabs instead of one." }
        if ($fresh.Count -ne 1 -or $readers[$fresh[0]] -ne $THREAD_PRIORITY_NORMAL) { return $false }
        $script:newReader = [int]$fresh[0].Substring($readerPrefix.Length)
        @($tabs | Where-Object { $readers["$readerPrefix$_"] -ne $THREAD_PRIORITY_BELOW_NORMAL }).Count -eq 0
    } { "$What did not open a tab that is shown." }
    $script:newReader
}

# Opens a tab of Light, the first profile, with Ctrl+Shift+1: at the end,
# and shown, so that the strip shows it before the next frame.
function Open-Light {
    Send-Chord 'Ctrl+Shift+1'
    $tabs.Add((Wait-NewReader 'Ctrl+Shift+1'))
    $script:active = $tabs.Count - 1
    Update-Fit
    $null = Show-BarTab $active
}

# Waits until the tab that is shown is the one of the model: the reader of
# its session has the normal priority, and the ones of all the other tabs,
# and only them, the lower one.
function Wait-Active([string]$What) {
    Wait-For {
        $readers = Get-Readers
        if ($null -eq $readers -or $readers.Count -ne $tabs.Count) { return $false }
        for ($index = 0; $index -lt $tabs.Count; $index++) {
            $expected = if ($index -eq $active) { $THREAD_PRIORITY_NORMAL } else { $THREAD_PRIORITY_BELOW_NORMAL }
            $priority = 0
            if (-not $readers.TryGetValue("$readerPrefix$($tabs[$index])", [ref]$priority) -or $priority -ne $expected) {
                return $false
            }
        }
        $true
    } { "$What`: the tab at $($active + 1) of $($tabs.Count) is not the one that is shown, or another tab is." }
}

# Moves the pointer to $Point, and checks that the window draws only $Area,
# once or twice: the pointer of the system is not on the window, so Windows
# tells it at once that the pointer left, and what it highlighted goes.
function Assert-Hover($Point, $Area, [string]$What) {
    Wait-Quiet 400
    $from = Get-Mark
    Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 $Point)
    Wait-For { @(Get-Frames $from).Count -gt 0 } { "The pointer on $What drew nothing." }
    Wait-Quiet 400
    $frames = @(Get-Frames $from)
    $other = @(Get-Areas $frames | Where-Object { -not (Test-Same $_ $Area) })
    if ($frames.Count -gt 2 -or $other.Count -gt 0) {
        Show-Lines $frames
        throw "The pointer on $What did not draw only $(Show-Rect $Area), once or twice."
    }
}

# A click of the left button at $Point: the pointer comes, and the button
# goes down and up.
function Get-Click($Point) {
    (Get-Mouse $WM_MOUSEMOVE 0 $Point) + (Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $Point) + (Get-Mouse $WM_LBUTTONUP 0 $Point)
}

# Checks where the strip is: the pointer on the title of the tab at $Index,
# which is not the one that is shown, draws its column of the bar, which
# depends on the scroll. Then a click there shows that tab, which is all in
# view, so the strip stays; and the check waits for its full frame, before
# which the strip brings the tab that is shown into view.
function Assert-Scroll([int]$Index, [string]$What) {
    if ($Index -eq $active) { throw "The check expects a tab that is not shown, $What." }
    $point = Get-Middle (Get-TitleRect $Index)
    Assert-Hover $point (Get-TabArea $Index) "the title of the tab at $($Index + 1) of $($tabs.Count), $What,"
    $from = Get-Mark
    Send-Mouse (Get-Click $point)
    $script:active = $Index
    Wait-Active "A click on the title of the tab at $($Index + 1), $What"
    Wait-OneFull $from "A click on the title of the tab at $($Index + 1), $What,"
}

# The frames after the first $From lines of the log that draw the whole bar:
# the strip scrolled, or the tabs changed.
function Get-BarFrames([int]$From) {
    $bar = New-Rect 0 0 $barWidth $barHeight
    @(Get-Frames $From | Where-Object { @(Get-Areas $_ | Where-Object { Test-Inside $bar $_ }).Count -gt 0 })
}

# Posts $Messages, which scroll the strip or change the tabs, once meiga
# stops drawing what came before, and waits until the window draws the whole
# bar.
function Send-ToBar([long[]]$Messages, [string]$What) {
    Wait-Quiet 400
    $from = Get-Mark
    Send-Mouse $Messages
    Wait-For { @(Get-BarFrames $from).Count -gt 0 } { "$What did not draw the bar again." }
}

# Does $Action, and checks that the window draws only $Area, once.
function Assert-Drawn($Area, [scriptblock]$Action, [string]$What) {
    Wait-Quiet 400
    $from = Get-Mark
    & $Action
    Wait-For { @(Get-Frames $from).Count -gt 0 } { "$What drew nothing." }
    Wait-Quiet 400
    $frames = @(Get-Frames $from)
    if ($frames.Count -ne 1 -or @(Get-Areas $frames | Where-Object { -not (Test-Same $_ $Area) }).Count -gt 0) {
        Show-Lines $frames
        throw "$What did not draw only $(Show-Rect $Area), once."
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

# Does $Action, which chooses again the tab that is shown, while the strip
# leaves it out of view, at least in part: the strip scrolls at once so that
# the tab is all in view, as when another tab is shown. Checks that the
# window draws the whole bar, and nothing out of it: the tab that is shown is
# the same.
function Assert-Reveal([scriptblock]$Action, [string]$What) {
    if (-not (Show-BarTab $active)) {
        throw 'The check expects the tab that is shown out of view, at least in part.'
    }
    Wait-Quiet 400
    $from = Get-Mark
    & $Action
    Wait-For { @(Get-BarFrames $from).Count -gt 0 } {
        Show-Lines (Get-Frames $from)
        "$What did not draw the whole bar: the strip did not scroll so that the tab is all in view."
    }
    Wait-Quiet 400
    $frames = @(Get-Frames $from)
    $bar = New-Rect 0 0 $barWidth $barHeight
    if (@($frames | Where-Object { Test-Full $_ }).Count -gt 0 -or @(Get-Areas $frames | Where-Object { -not (Test-Inside $_ $bar) }).Count -gt 0) {
        Show-Lines $frames
        throw "$What drew out of the bar: it did not leave the tab shown."
    }
}

# Types $Chord, which shows the tab at $Index, once meiga stops drawing what
# came before, and waits until it is shown, with one full frame: before it,
# the strip scrolls so that the tab is in view. Only the tab that is shown at
# the frame comes into view: two tabs shown before a frame would scroll the
# strip once.
function Show-Tab([string]$Chord, [int]$Index) {
    Wait-Quiet 400
    $from = Get-Mark
    Send-Chord $Chord
    $script:active = $Index
    $null = Show-BarTab $Index
    Wait-Active $Chord
    Wait-OneFull $from $Chord
}

# The command line of meiga: its frame log, and no program, so that its first
# tab runs the default profile, Probe.
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
$layout = [IntPtr]::Zero
$dpi = 96
$pointer = $null
$attached = $false
$newReader = 0
$passed = $false
try {
    $process = [Diagnostics.Process]::Start($info)
    $started = $process.StartTime
    Wait-For { $api::WindowOf($process.Id, 'meiga') -ne [IntPtr]::Zero } { "$exe did not open its window." }
    $window = $api::WindowOf($process.Id, 'meiga')
    $processId = [uint32]0
    $thread = $api::GetWindowThreadProcessId($window, [ref]$processId)
    $layout = $api::GetKeyboardLayout($thread)
    $dpi = $api::GetDpiForWindow($window)
    $barWidth = Get-Scaled 983
    $band = Get-Band
    $rowHeight = Get-Scaled 32
    $barHeight = $band + $rowHeight
    $gap = Get-Scaled 4
    $padding = Get-Scaled 8
    $closeSide = Get-Scaled 24
    $newSide = Get-Scaled 28
    $profilesSide = Get-Scaled 24
    $scrollSide = Get-Scaled 24
    $dragSide = Get-Scaled 45
    $captionSide = Get-Scaled 46
    $maxTab = Get-Scaled 240
    $minTab = Get-Scaled 100
    $fits = { param($Count) [Math]::Floor((Get-Room) / $Count) -ge $minTab }
    if (-not (& $fits 7) -or (& $fits 8)) {
        throw "At $dpi DPI, the check needs a bar where 7 tabs fit and 8 do not."
    }
    Resize-Window $barWidth (Get-Scaled 400)

    # The first tab runs Probe, the default profile.
    $first = Wait-NewReader 'The first tab'
    $line = (Get-CimInstance Win32_Process -Filter "ProcessId = $first").CommandLine
    if ($line -cne $probeLine) {
        throw "The first tab runs '$line' instead of '$probeLine'."
    }
    Wait-For { Test-Path -LiteralPath (Join-Path $work "ready-$first.txt") } { 'The script of the first tab did not start.' }
    $tabs.Add($first)
    $active = 0
    Update-Fit
    Join-Input
    Move-Pointer
    Wait-Settled @($first) 'The first tab'
    Write-Host "The bar is $barHeight pixels high, $band of them above the tabs, and its tabs from $minTab to $maxTab wide: 7 of them fit in the window, $barWidth pixels wide, and 8 do not."

    # 1. Six tabs of Light more: the seven tabs fit, alike.
    for ($count = 1; $count -lt 7; $count++) { Open-Light }
    Wait-Settled $tabs.ToArray() 'The first 7 tabs'
    Assert-Hover (Get-Middle (Get-TitleRect 6)) (Get-TabArea 6) 'the title of the last tab'
    Assert-Hover (Get-Middle (Get-NewRect)) (Get-NewRect) 'the button +'
    Assert-Hover (Get-Middle (Get-TitleRect 0)) (Get-TabArea 0) 'the title of the first tab'
    Write-Host 'OK: with 7 tabs, they shrink alike, and fit in the bar, with the button + after them: the pointer on the last tab, on the button +, and on the first tab draws only that part.'

    # 2. An eighth tab does not fit: the tabs keep the narrowest width, and
    # the strip scrolls to show the new one.
    Open-Light
    Wait-Settled @($tabs[$tabs.Count - 1]) 'The eighth tab'
    Assert-Hover (Get-Middle (Get-ScrollRect 'Left')) (Get-ScrollRect 'Left') 'the button that scrolls the strip to the left'
    Assert-Hover (Get-Middle (Get-NewRect)) (Get-NewRect) 'the button + on the right of the bar'
    Assert-Hover (Get-Middle (Get-ProfilesRect)) (Get-ProfilesRect) 'the button of the profiles'
    Assert-Scroll (Get-OtherTab) 'with eight tabs'
    Write-Host 'OK: an eighth tab does not fit: the tabs keep their narrowest width, and the strip, between the buttons that scroll it, shows the new one, which is shown; the buttons + and of the profiles go on the right of the bar.'

    # 3. Up to 30 tabs, each one with its own program.
    while ($tabs.Count -lt 30) { Open-Light }
    Wait-Settled $tabs.ToArray() 'The 30 tabs'
    $shells = @(Get-Shells)
    $ids = @($shells | ForEach-Object { [int]$_.ProcessId } | Sort-Object)
    if (($ids -join ' ') -ne (@($tabs | Sort-Object) -join ' ')) {
        throw "The 30 tabs run the programs $($ids -join ', '), instead of the ones of their readers, $(@($tabs | Sort-Object) -join ', ')."
    }
    foreach ($shell in $shells) {
        $expected = if ([int]$shell.ProcessId -eq $first) { $probeLine } else { $lightLine }
        if ($shell.CommandLine -cne $expected) {
            throw "A tab runs '$($shell.CommandLine)' instead of '$expected'."
        }
    }
    $last = $tabs.Count - 1
    if (-not (Test-AllInView $last) -or (Get-TabArea $last).Right -gt (Get-ScrollRect 'Right').Left) {
        throw 'The check expects the last tab all in view, before the button that scrolls the strip to the right.'
    }
    # The strip is at its end: the button that scrolls it to the right
    # cannot, so it looks the same under the pointer, and a click on it does
    # nothing.
    Assert-NothingDrawn { Send-Mouse (Get-Click (Get-Middle (Get-ScrollRect 'Right'))) } 'The pointer on the button that scrolls the strip to the right, at its end, and a click on it,'
    Assert-Hover (Get-Middle (Get-TitleRect $last)) (Get-TabArea $last) 'the title of the last tab'
    Assert-Scroll (Get-OtherTab) 'with 30 tabs'
    Write-Host 'OK: with 30 tabs, each one runs its own program, nothing overlaps, and the strip shows the last one, which ends before the button that scrolls the strip to the right; that button cannot scroll it further, and the pointer on it, and a click on it, draw nothing.'

    # 4. The wheel, on the strip: a tab for each step, and the rest of a step
    # for the next turn. Two steps to the right and one to the left: if they
    # were the same, a horizontal wheel that goes the wrong way, or does
    # nothing, would leave the strip where it has to be.
    $at = Get-Middle (Get-Strip)
    $turns = @(
        @{ Delta = 120; Horizontal = $false }
        @{ Delta = 480; Horizontal = $false }
        @{ Delta = 240; Horizontal = $true }
        @{ Delta = -120; Horizontal = $false }
        @{ Delta = -120; Horizontal = $true }
        @{ Delta = 50; Horizontal = $false }
        @{ Delta = 70; Horizontal = $false }
    )
    $messages = @(foreach ($turn in $turns) {
            $message = if ($turn.Horizontal) { $WM_MOUSEHWHEEL } else { $WM_MOUSEWHEEL }
            Get-Wheel $message $turn.Delta $at
        })
    Send-ToBar $messages 'The wheel'
    foreach ($turn in $turns) { Add-Wheel $turn.Delta $turn.Horizontal }
    Assert-Scroll (Get-OtherTab) 'after the wheel'
    Write-Host 'OK: the wheel on the bar scrolls a tab for each step, towards the first one forward or to the left, and towards the last one backward or to the right, and keeps what is left of a step for the next turn.'

    # 5. The buttons that scroll the strip, with a click.
    foreach ($side in 'Left', 'Left', 'Right') {
        Send-ToBar (Get-Click (Get-Middle (Get-ScrollRect $side))) "A click on the button that scrolls the strip to the $($side.ToLowerInvariant())"
        $null = Step-Bar $side
        Assert-Scroll (Get-OtherTab) "after a click on the button that scrolls the strip to the $($side.ToLowerInvariant())"
    }
    Write-Host 'OK: a click on a button that scrolls the strip scrolls it until the first tab that it hides on that side, even in part, is all in view.'

    # 6. The left button held down on the button that scrolls the strip to the
    # left, with the pointer of the system on it, if the check can put it
    # there.
    $button = Get-ScrollRect 'Left'
    $arrow = Get-Middle $button
    Wait-Quiet 400
    $from = Get-Mark
    if (Set-RealPointer $arrow) {
        Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 $arrow)
        Wait-For { @(Get-Areas (Get-Frames $from) | Where-Object { Test-Overlap $_ $button }).Count -gt 0 } {
            'The pointer on the button that scrolls the strip to the left drew nothing.'
        }
        Wait-Quiet 400
        $from = Get-Mark
        Send-Mouse (Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $arrow)
        Start-Sleep -Milliseconds 1000
        Send-Mouse (Get-Mouse $WM_LBUTTONUP 0 $arrow)
        Move-Pointer
        Wait-Quiet 400
        $scrolls = @(Get-BarFrames $from)
        $times = @($scrolls | ForEach-Object { [double]$_.time_ms })
        $gaps = @(for ($index = 1; $index -lt $times.Count; $index++) { $times[$index] - $times[$index - 1] })
        $early = @(for ($index = 0; $index -lt $gaps.Count; $index++) {
                if ($gaps[$index] -lt $(if ($index -eq 0) { 249.9 } else { 99.9 })) { $index }
            })
        if ($scrolls.Count -lt 3 -or $early.Count -gt 0) {
            Show-Lines $scrolls
            $count = if ($scrolls.Count -eq 1) { 'once' } else { "$($scrolls.Count) times" }
            $after = if ($gaps.Count -gt 0) { ', after ' + (($gaps | ForEach-Object { '{0:0.0} ms' -f $_ }) -join ', ') } else { '' }
            throw "While the left button stayed down for a second on the button that scrolls the strip to the left, the strip scrolled $count$($after): it has to scroll at once, 250 ms later, and then every 100 ms."
        }
        $ok = "OK: while the left button stays down on a button that scrolls the strip, with the pointer on it, the strip scrolls another tab after 250 ms, and then after each 100 ms: $($scrolls.Count) tabs in a second."
    }
    else {
        Send-Mouse ((Get-Mouse $WM_MOUSEMOVE 0 $arrow) + (Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $arrow))
        Start-Sleep -Milliseconds 1000
        Send-Mouse (Get-Mouse $WM_LBUTTONUP 0 $arrow)
        Wait-Quiet 400
        $scrolls = @(Get-BarFrames $from)
        if ($scrolls.Count -ne 1) {
            Show-Lines $scrolls
            throw "The left button down for a second on the button that scrolls the strip to the left, with the pointer that left at once, scrolled the strip $($scrolls.Count) times instead of once."
        }
        $ok = 'OK: the left button down on a button that scrolls the strip scrolls it once, and no more once the pointer leaves the button; the check could not move the pointer of the system onto the button to hold it there.'
    }
    foreach ($scroll in $scrolls) { $null = Step-Bar 'Left' }
    Assert-Scroll (Get-OtherTab) 'after the button held down'
    Write-Host $ok

    # 7. Each of the 30 tabs, with a click on its title, once the wheel
    # brings it all into view, if it is not: the wheel and the click go
    # together.
    $at = Get-Middle (Get-Strip)
    for ($index = 0; $index -lt $tabs.Count; $index++) {
        $messages = @()
        $steps = Get-StepsTo $index
        if ($steps -ne 0) {
            $messages += Get-Wheel $WM_MOUSEWHEEL ($steps * $WHEEL_DELTA) $at
            Add-Wheel ($steps * $WHEEL_DELTA) $false
        }
        $messages += Get-Click (Get-Middle (Get-TitleRect $index))
        Send-Mouse $messages
        $active = $index
        Wait-Active "A click on the title of the tab at $($index + 1) of $($tabs.Count)"
    }
    Write-Host 'OK: each of the 30 tabs can be shown with a click on its title, once the wheel brings it into view.'

    # 8. The close button, and then the middle button, at the same place,
    # close two tabs in the middle of the strip: the second one is the tab
    # that came onto the place of the first one, which kept its width and
    # the strip, though the strip could not scroll so far with a tab less.
    $closing = [int][Math]::Floor(($barScroll + (Get-Viewport) / 2) / $barTabWidth)
    if ($closing -lt 1 -or $closing + 1 -ge $active -or -not (Test-AllInView $closing) -or $barScroll -ne (Get-MaxScroll)) {
        throw 'The check expects two tabs all in view in the middle of the strip, after the first tab and before the one that is shown, and the strip scrolled as far as it goes.'
    }
    $point = Get-Middle (Get-CloseRect $closing)
    Assert-Hover (Get-Middle (Get-TitleRect $closing)) (Get-TabArea $closing) "the title of the tab at $($closing + 1) of $($tabs.Count), once a click showed the last one,"
    $gone = @($tabs[$closing], $tabs[$closing + 1])
    foreach ($id in $gone) {
        if ((Get-CloseAt $point) -ne $closing) {
            throw 'The check expects the close button of the tab on the right under the pointer, once a tab closes.'
        }
        Remove-Tab $closing
    }
    $messages = (Get-Click $point) + (Get-Mouse $WM_MOUSEMOVE 0 $point) +
    (Get-Mouse $WM_MBUTTONDOWN $MK_MBUTTON $point) + (Get-Mouse $WM_MBUTTONUP 0 $point)
    Send-Mouse $messages
    foreach ($id in $gone) { Wait-Gone $id 'The program of a tab that the mouse closed' }
    Wait-Active 'Once the mouse closed two tabs'
    Update-Fit
    $null = Show-BarTab $active
    $shown = $closing - 1
    if (-not (Test-AllInView $shown) -or $barScroll -ne (Get-MaxScroll)) {
        throw 'The check expects the tab on the left of the closed ones all in view, and the strip scrolled as far as it goes.'
    }
    Assert-Scroll $shown 'once the pointer left the bar after two tabs closed'
    Write-Host 'OK: when the close button or the middle button closes a tab, the others keep their width and their place while the pointer stays on the bar, and the next close button is under the pointer; once the pointer leaves the bar, they get their width again, and the strip shows the tab that is shown.'

    # 9. With the pointer on the close button of the tab that is shown,
    # Ctrl+Shift+W closes it: the strip scrolls a tab less at once, and the
    # close button of the tab on its left comes under the pointer. A click
    # closes that one, and then the others keep their place: a click on the
    # title of the third tab from the end, where it is then, shows that tab,
    # and not the one on its left.
    $point = Get-Middle (Get-CloseRect $shown)
    if ((Get-CloseAt $point) -ne $shown) {
        throw 'The check expects the close button of the tab that is shown under the pointer.'
    }
    $keyboard = $tabs[$shown]
    Remove-Tab $shown
    Update-Fit
    if (Show-BarTab $active) {
        throw 'The check expects the tab on the right of the closed one all in view.'
    }
    $under = Get-CloseAt $point
    if ($under -ne $shown - 1) {
        throw 'The check expects the close button of the tab on the left of the closed one under the pointer, once the tabs get their width.'
    }
    $mouse = $tabs[$under]
    Remove-Tab $under
    $target = $tabs.Count - 3
    if (-not (Test-AllInView $target) -or $target -eq $active) {
        throw 'The check expects the third tab from the end all in view, and not shown.'
    }
    $typed = Get-Chord 'Ctrl+Shift+W'
    $messages = (Get-Mouse $WM_MOUSEMOVE 0 $point) + $typed.Messages +
    (Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $point) + (Get-Mouse $WM_LBUTTONUP 0 $point) +
    (Get-Click (Get-Middle (Get-TitleRect $target)))
    Send-Keys $messages $typed.Down
    $active = $target
    Wait-Gone $keyboard 'The program of the tab that Ctrl+Shift+W closed'
    Wait-Gone $mouse 'The program of the tab that the close button closed then'
    Wait-Active 'A click on a title, once Ctrl+Shift+W and then the close button closed two tabs,'
    Update-Fit
    if ((Show-BarTab $active) -or $barScroll -ne (Get-MaxScroll)) {
        throw 'The check expects the tab that is shown all in view, and the strip scrolled as far as it goes.'
    }
    Assert-Hover (Get-Middle (Get-TitleRect $target)) (Get-TabArea $target) "the title of the tab at $($target + 1) of $($tabs.Count), once the pointer left the bar,"
    Write-Host 'OK: a tab that the keyboard closes, with the pointer on the bar, lets the others get their width at once; then a tab that the mouse closes makes them keep it.'

    # 10. Two clicks on the close button of the last tab close it, and then
    # the one before it, whose close button comes under the pointer.
    $last = $tabs.Count - 1
    $point = Get-Middle (Get-CloseRect $last)
    if ((Get-CloseAt $point) -ne $last) {
        throw 'The check expects the close button of the last tab under the pointer.'
    }
    $gone = @($tabs[$last], $tabs[$last - 1])
    foreach ($id in $gone) {
        if ((Get-CloseAt $point) -ne $tabs.Count - 1) {
            throw 'The check expects the close button of the new last tab under the pointer, once the last one closes.'
        }
        Remove-Tab ($tabs.Count - 1)
        Update-Fit
        if (Show-BarTab $active) {
            throw 'The check expects the tab that is shown all in view, once the last tab closes.'
        }
    }
    $click = (Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $point) + (Get-Mouse $WM_LBUTTONUP 0 $point)
    Send-Mouse ((Get-Mouse $WM_MOUSEMOVE 0 $point) + $click + $click)
    foreach ($id in $gone) { Wait-Gone $id 'The program of a last tab that the mouse closed' }
    Wait-Active 'Once the mouse closed the last tab twice'
    Assert-Scroll (Get-OtherTab) 'once the mouse closed the last tab twice'
    Write-Host 'OK: the last tab, closed with the mouse, lets the others get their width at once, so the close button of the new last one comes under the pointer.'

    # 11. The keyboard shows a tab, and the strip scrolls so that it is in
    # view: Ctrl+1 the first one, and Ctrl+Shift+Tab, round the bar, the last
    # one. The wheel can move the tab that is shown out of view, and the strip
    # stays there until that tab is chosen again (ADR 0025, decision 5): Ctrl+1
    # and Ctrl+9 bring the first and the last tab into view again, though
    # Ctrl+9 draws nothing while the last tab is all in view; Shift+F10 scrolls
    # the strip too, and opens the menu of the tab below it; and a click on the
    # part of the tab that is in view, on the right of the strip this time,
    # brings all of it into view. After each of them, a click on another tab,
    # or the wheel and a click, find the strip where it has to be.
    $at = Get-Middle (Get-Strip)
    Show-Tab 'Ctrl+1' 0
    Assert-Hover (Get-Middle (Get-TitleRect 0)) (Get-TabArea 0) 'the title of the first tab, once Ctrl+1 showed it,'
    Send-ToBar (Get-Wheel $WM_MOUSEWHEEL (-$WHEEL_DELTA) $at) 'The wheel backward'
    Add-Wheel (-$WHEEL_DELTA) $false
    Assert-Reveal { Send-Chord 'Ctrl+1' } 'Ctrl+1, on the first tab, which is shown,'
    Assert-Scroll (Get-OtherTab) 'once Ctrl+1 brought the first tab into view'
    Show-Tab 'Ctrl+Shift+Tab' 0
    $last = $tabs.Count - 1
    Show-Tab 'Ctrl+Shift+Tab' $last
    Assert-Hover (Get-Middle (Get-TitleRect $last)) (Get-TabArea $last) 'the title of the last tab, once Ctrl+Shift+Tab showed it from the first one,'
    Assert-NothingDrawn { Send-Chord 'Ctrl+9' } 'Ctrl+9, on the last tab, which is shown and all in view,'
    Send-ToBar (Get-Wheel $WM_MOUSEWHEEL (2 * $WHEEL_DELTA) $at) 'The wheel forward'
    Add-Wheel (2 * $WHEEL_DELTA) $false
    Assert-Reveal { Send-Chord 'Ctrl+9' } 'Ctrl+9, on the last tab, which is shown,'
    $messages = Get-Wheel $WM_MOUSEWHEEL (2 * $WHEEL_DELTA) $at
    Add-Wheel (2 * $WHEEL_DELTA) $false
    $target = Get-OtherTab
    $messages += Get-Click (Get-Middle (Get-TitleRect $target))
    Send-Mouse $messages
    $active = $target
    Wait-Active 'A click on the title of a tab, once the wheel scrolled the strip,'
    Send-ToBar (Get-Wheel $WM_MOUSEWHEEL (-3 * $WHEEL_DELTA) $at) 'The wheel backward'
    Add-Wheel (-3 * $WHEEL_DELTA) $false
    $area = Get-TabArea $target
    if ($area.Right -gt $area.Left) {
        throw 'The check expects the tab that is shown out of view, once the wheel scrolls the strip backward.'
    }
    Wait-Quiet 400
    $from = Get-Mark
    Send-Chord 'Shift+F10'
    if (-not (Show-BarTab $target)) {
        throw 'The check expects the strip to scroll so that the tab that is shown is in view.'
    }
    $tab = Get-TabRect $target
    $bar = New-Rect 0 0 $barWidth $barHeight
    Wait-For {
        $areas = @(Get-Areas (Get-Frames $from))
        @($areas | Where-Object { Test-Same $_ $bar }).Count -gt 0 -and
        @($areas | Where-Object { $_.Left -eq $tab.Left -and $_.Top -eq $tab.Bottom }).Count -gt 0
    } {
        Show-Lines (Get-Frames $from)
        "Shift+F10 did not draw the whole bar and a menu from $(Show-Rect $tab) down: it did not scroll the strip until the tab that is shown is in view, or did not open its menu below it."
    }
    $menu = @(Get-Areas (Get-Frames $from) | Where-Object { $_.Left -eq $tab.Left -and $_.Top -eq $tab.Bottom })[0]
    Wait-Quiet 400
    $from = Get-Mark
    Send-Chord 'Escape'
    Wait-For { @(Get-Areas (Get-Frames $from) | Where-Object { Test-Inside $menu $_ }).Count -gt 0 } { 'Esc did not close the menu.' }
    Assert-Scroll (Get-OtherTab) 'once Shift+F10 scrolled the strip'
    # The tab that the click showed, on the right of the strip, where only
    # part of it is in view, with its title.
    Send-ToBar (Get-Wheel $WM_MOUSEWHEEL (5 * $WHEEL_DELTA) $at) 'The wheel forward'
    Add-Wheel (5 * $WHEEL_DELTA) $false
    $label = Get-LabelArea $active
    if ((Test-AllInView $active) -or $label.Right -le $label.Left) {
        throw 'The check expects the tab that is shown partly in view, with part of its title, once the wheel scrolls the strip forward.'
    }
    Assert-Reveal { Send-Mouse (Get-Click (Get-Middle $label)) } 'A click on the title of the tab that is shown, partly in view,'
    Assert-Scroll (Get-OtherTab) 'once a click on the tab that is shown brought it all into view'
    Write-Host 'OK: showing a tab with the keyboard, also round the bar, scrolls the strip so that it is in view; the wheel can then move the tab that is shown out of view, where it stays until the tab is chosen again: Ctrl+1 and Ctrl+9, and a click on the part of it that is in view, scroll the strip until it is all in view, and Ctrl+9 on the tab all in view draws nothing; Shift+F10 also scrolls the strip until the tab is in view, and opens its menu below it.'

    # 12. Probe, the first tab, while it is hidden: its program writes, rings
    # the bell, and ends with exit code 3.
    Show-Tab 'Ctrl+1' 0
    Show-Tab 'Ctrl+2' 1
    $probe = $tabs[0]
    $label = Get-LabelArea 0
    $bell = [string][char]7
    Assert-Drawn $label { Write-Probe $probe 'x' } 'The program of a hidden tab, which wrote,'
    Assert-NothingDrawn { Write-Probe $probe 'y' } 'The program of a hidden tab, which wrote again,'
    Assert-Drawn $label { Write-Probe $probe $bell } 'The bell of a hidden tab'
    Assert-NothingDrawn { Write-Probe $probe $bell } 'The bell of a hidden tab, which rang again,'
    Show-Tab 'Ctrl+1' 0
    Assert-NothingDrawn { Write-Probe $probe $bell } 'The bell of the tab that is shown'
    Show-Tab 'Ctrl+2' 1
    Assert-Drawn $label { Write-Probe $probe 'z' } 'The program of a hidden tab, which wrote once the tab had been shown,'
    Assert-Drawn $label { Stop-Probe $probe 3 } 'The end of the program of a hidden tab, with exit code 3,'
    Wait-Gone $probe 'The program of the hidden tab, which ended with exit code 3,'
    Write-Host 'OK: while a tab is hidden, its program writes, which draws only the title of its tab, once, and rings the bell, which draws it once more; showing the tab clears both, and a bell while it is shown draws nothing; the end of its program with exit code 3, while it is hidden, draws its title once more.'

    # 13. The window closes, with the tab of Probe, whose program ended, and
    # 23 tabs of Light.
    $shells = @(Get-Shells)
    $tabConsoles = @(Get-Consoles)
    Split-Input
    if (-not $api::PostMessageW($window, $WM_CLOSE, [UIntPtr]::Zero, [IntPtr]::Zero)) {
        throw "Could not close the window of ${exe}: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
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
    Write-Host 'OK: it ends with exit code 0 when its window closes, and the programs of its tabs and their consoles end too.'
    $passed = $true
}
finally {
    Split-Input
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
