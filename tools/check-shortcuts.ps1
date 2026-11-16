<#
.SYNOPSIS
Checks the keyboard shortcuts of meiga.exe: the ones that it has, the ones
that config.toml adds, changes and frees, and the keys that it leaves to the
program.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and an APPDATA of its
own, whose config.toml has two profiles, Probe, the default one, and Probe 2,
which run the same PowerShell script, and hides the shells that meiga finds:
the menu of the profiles has only those two. Its table [shortcuts] frees
Ctrl+6, gives Ctrl+Alt+2 the second tab, Ctrl+8 the last tab instead of the
eighth, and Alt+O and Alt+R close_other_tabs and close_tabs_to_the_right,
two actions without a shortcut of their own; and it has four errors: a
chord with two keys, an action that does not exist, profile_0, and Alt+O
again, after its first line. The script of each tab writes in a file of its
own the keys that its program gets.

The check types the keys in the window as check-profiles.ps1 does: it posts
their messages while the thread of the window is suspended, with the key
state that they need, which it shares with that thread. A chord goes as a
keyboard sends it: its modifier keys go down, its key goes down and up, and
its modifier keys go up; with Alt and without Ctrl, as system keys, but for
Alt going up, which Windows sends as a key that is not a system one after
another key. After the keys whose effect it cannot see, it types Z, and
waits until the program of the tab that is shown gets it: then the window
has taken all the keys before it. It checks that:

- a message box tells the errors of [shortcuts], each with its line and its
  column, and that meiga ignored them;
- Ctrl+T and Ctrl+W reach the program, and so does Ctrl+6, which config.toml
  frees: no tab opens nor closes;
- Ctrl+Shift+T opens one tab, also while T repeats;
- AltGr+2, which Windows gives as the left Ctrl and the right Alt, reaches
  the program, while Ctrl+Alt+2, with the left Alt, shows the second tab, as
  config.toml says;
- Ctrl+Shift+D opens a copy of the tab that is shown right after it, and
  shows it; Ctrl+3, Ctrl+1 and Ctrl+2 show the tabs at those places, and
  Ctrl+9 the last one, each with one full frame; and Ctrl+4, with three
  tabs, while the copy, in the middle, is shown, draws nothing, and its 4
  does not reach the program;
- Ctrl+PageDown and Ctrl+PageUp show the next and the previous tab, round
  the bar, and PageUp goes on while it repeats; and PageDown, down for the
  program before Ctrl, goes to it while it repeats as Ctrl+PageDown and
  when it goes up, and shows no tab: a key that went down for the program
  is its own until it goes up;
- Ctrl+Shift+2 and Ctrl+Shift+1 open Probe 2 and Probe, the profiles at
  those places of the menu, at the end, where Ctrl+4 and Ctrl+5 show them;
  and Ctrl+Shift+3, with two profiles, opens nothing, and its 3 does not
  reach the program;
- Ctrl+8 shows the last of five tabs, as config.toml says;
- Alt+R, with the third of five tabs shown, closes the two on its right,
  and the two on its left stay; Alt+O, with the one in the middle shown,
  closes the other two, on both sides, as its first line in config.toml
  says; their programs end, and the program of the tab gets Alt, and not R
  nor O;
- Ctrl+Shift+W closes the last tab, and with it the window: meiga.exe ends
  with exit code 0, and the program of the tab and the consoles end too.

It needs PowerShell 7 (pwsh) in the PATH, for the script of the tabs.

If a check fails, the script ends meiga.exe and keeps its folder, with the
frame log and the files of the keys.

.PARAMETER Path
The executable to check. Defaults to target\release\meiga.exe.

.PARAMETER TimeoutSeconds
How long to wait for the window, for the message box, for each chord, and
for the programs of the tabs to end.
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

if (-not ('Meiga.Tools.ShortcutsCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name ShortcutsCheck -MemberDefinition @'
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

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsWindow(IntPtr window);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsWindowVisible(IntPtr window);

[DllImport("user32.dll")]
public static extern IntPtr GetWindow(IntPtr window, uint command);

[DllImport("user32.dll")]
public static extern int GetDlgCtrlID(IntPtr window);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool EnumWindows(EnumProc callback, IntPtr parameter);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool EnumThreadWindows(uint thread, EnumProc callback, IntPtr parameter);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool EnumChildWindows(IntPtr parent, EnumProc callback, IntPtr parameter);

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern int GetClassNameW(IntPtr window, System.Text.StringBuilder name, int count);

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern int GetWindowTextW(IntPtr window, System.Text.StringBuilder text, int count);

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern uint GetDlgItemTextW(IntPtr dialog, int id, System.Text.StringBuilder text, int count);

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
// reads the posted messages first. The thread of the window gets `state` as
// its key state, which this thread shares: the key state changes as the
// thread reads its input, so it is set while the thread is suspended, for
// the keys of the messages.
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
            if (!SetKeyboardState(state)) throw new System.ComponentModel.Win32Exception();
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

static string ClassOf(IntPtr window)
{
    var name = new System.Text.StringBuilder(256);
    GetClassNameW(window, name, name.Capacity);
    return name.ToString();
}

// The text of a window: the title of a top-level one.
public static string TextOf(IntPtr window)
{
    var text = new System.Text.StringBuilder(1024);
    GetWindowTextW(window, text, text.Capacity);
    return text.ToString();
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

// The visible dialog boxes of the thread `thread`, as a message box.
public static IntPtr[] DialogsOf(uint thread)
{
    var dialogs = new System.Collections.Generic.List<IntPtr>();
    EnumThreadWindows(thread, (window, parameter) =>
    {
        if (ClassOf(window) == "#32770" && IsWindowVisible(window)) dialogs.Add(window);
        return true;
    }, IntPtr.Zero);
    return dialogs.ToArray();
}

// The buttons of the dialog box `dialog`.
public static IntPtr[] ButtonsOf(IntPtr dialog)
{
    var buttons = new System.Collections.Generic.List<IntPtr>();
    EnumChildWindows(dialog, (window, parameter) =>
    {
        if (ClassOf(window) == "Button") buttons.Add(window);
        return true;
    }, IntPtr.Zero);
    return buttons.ToArray();
}

// The text of a message box: the one of its control 0xFFFF.
public static string MessageOf(IntPtr dialog)
{
    var text = new System.Text.StringBuilder(16384);
    GetDlgItemTextW(dialog, 0xFFFF, text, text.Capacity);
    return text.ToString();
}
'@
}
$api = [Meiga.Tools.ShortcutsCheck]

$WM_COMMAND = 0x0111
$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_SYSKEYDOWN = 0x0104
$WM_SYSKEYUP = 0x0105
$MAPVK_VK_TO_VSC = 0
$GW_OWNER = 4
$VK_CONTROL = 0x11
$VK_MENU = 0x12
$VK_PRIOR = 0x21
$VK_NEXT = 0x22
$VK_Z = 0x5A
$VK_LCONTROL = 0xA2
$THREAD_PRIORITY_NORMAL = 0
$THREAD_PRIORITY_BELOW_NORMAL = -1
# The console of each tab: the one of the ConPTY of meiga, or the one of
# Windows if the ConPTY is not next to meiga.exe.
$consoles = 'OpenConsole.exe', 'conhost.exe'
# The modifier keys of the chords: the keys whose messages go down and up,
# each one with whether it is an extended key, and the keys that they set
# down in the key state. AltGr is the left Ctrl and the right Alt, as Windows
# gives it.
$chordKeys = @{
    Ctrl = , @{ Code = 0x11; Extended = $false; Down = 0x11, 0xA2 }
    Shift = , @{ Code = 0x10; Extended = $false; Down = 0x10, 0xA0 }
    Alt = , @{ Code = 0x12; Extended = $false; Down = 0x12, 0xA4 }
    AltGr = @{ Code = 0x11; Extended = $false; Down = 0x11, 0xA2 }, @{ Code = 0x12; Extended = $true; Down = 0x12, 0xA5 }
}

# The script of the tabs of Probe and Probe 2: it writes in a file of its own
# the keys that its program gets, until its tab closes.
$program = @'
param([Parameter(Mandatory)][string]$Keys, [switch]$Second)
$ErrorActionPreference = 'Stop'
# The cursor is hidden, so that it never blinks.
[Console]::Out.Write("$([char]27)[?25l")
Add-Type -Namespace MeigaCheck -Name Input -MemberDefinition @"
[DllImport("kernel32.dll")] public static extern System.IntPtr GetStdHandle(int handle);
[DllImport("kernel32.dll")] public static extern bool ReadConsoleInputW(System.IntPtr handle, [System.Runtime.InteropServices.Out] byte[] records, uint count, out uint read);
"@
$in = [MeigaCheck.Input]::GetStdHandle(-10)
# Up to 64 INPUT_RECORD of 20 bytes: the type of the event in the first two
# and, for a key, whether it goes down from byte 4, and its virtual-key code
# from byte 10.
$records = [byte[]]::new(20 * 64)
# The virtual-key code of each key, with + when it goes down and - when it
# goes up, in a file that the check reads while this script writes it. Once
# the file is there, the script reads the keys.
$file = [IO.FileStream]::new((Join-Path $Keys "keys-$PID.txt"), [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::ReadWrite)
while ($true) {
    $read = 0
    if (-not [MeigaCheck.Input]::ReadConsoleInputW($in, $records, 64, [ref]$read)) { exit 1 }
    $text = [Text.StringBuilder]::new()
    for ($at = 0; $at -lt 20 * $read; $at += 20) {
        if ([BitConverter]::ToUInt16($records, $at) -ne 1) { continue }
        $down = [BitConverter]::ToInt32($records, $at + 4) -ne 0
        $key = [BitConverter]::ToUInt16($records, $at + 10)
        $null = $text.Append("$key$(if ($down) { '+' } else { '-' }) ")
    }
    if ($text.Length -gt 0) {
        $bytes = [Text.Encoding]::ASCII.GetBytes($text.ToString())
        $file.Write($bytes, 0, $bytes.Length)
        $file.Flush()
    }
}
'@

$name = "meiga-check-shortcuts-$PID"
# All the files of the check, with the APPDATA of meiga.
$work = Join-Path ([IO.Path]::GetTempPath()) $name
$appdata = Join-Path $work 'appdata'
$config = Join-Path $appdata 'meiga\config.toml'
$script = Join-Path $work 'probe.ps1'
$log = Join-Path $work 'frames.csv'
Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
$null = New-Item -ItemType Directory -Force -Path (Split-Path $config)
Set-Content -LiteralPath $script -Value $program -Encoding utf8NoBOM

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

# The configuration: Probe, the default profile, and Probe 2, which runs the
# same script with another command line; the shells that meiga finds, hidden;
# and the shortcuts, with four errors, which meiga ignores.
$probeCommand = 'pwsh -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%MEIGA_CHECK_WORK%\probe.ps1" -Keys "%MEIGA_CHECK_WORK%"'
$configLines = @(
    '# Written by tools/check-shortcuts.ps1.'
    'default_profile = "Probe"'
    ''
    '[[profiles]]'
    'name = "Probe"'
    "command = '$probeCommand'"
    "directory = '%MEIGA_CHECK_WORK%'"
    ''
    '[[profiles]]'
    'name = "Probe 2"'
    "command = '$probeCommand -Second'"
    "directory = '%MEIGA_CHECK_WORK%'"
    foreach ($shell in $hidden) {
        ''
        '[[profiles]]'
        "name = $(ConvertTo-TomlString $shell)"
        'hidden = true'
    }
    ''
    '[shortcuts]'
    '"ctrl+6" = "none"'
    '"ctrl+alt+2" = "tab_2"'
    '"ctrl+8" = "last_tab"'
    '"alt+o" = "close_other_tabs"'
    '"alt+r" = "close_tabs_to_the_right"'
    '"ctrl+t+w" = "new_tab"'
    '"ctrl+shift+e" = "open_explorer"'
    '"ctrl+shift+0" = "profile_0"'
    '"Alt+O" = "close_tab"'
)
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText($config, ($configLines -join "`n") + "`n", $utf8)

# The number of the line $Line of the file, from 1.
function Get-LineNumber([string]$Line) {
    [Array]::IndexOf($configLines, $Line) + 1
}

# An error of the file, as meiga tells it: the line of $Line, and the column
# of $At in it, counted from 1.
function Get-Problem([string]$Line, [string]$At, [string]$Message) {
    'Line {0}, column {1}: {2}' -f (Get-LineNumber $Line), ($Line.IndexOf($At) + 1), $Message
}
$problems = @(
    Get-Problem '"ctrl+t+w" = "new_tab"' '"ctrl+t+w"' '`ctrl+t+w` is not a shortcut: it has more than one key'
    Get-Problem '"ctrl+shift+e" = "open_explorer"' '"open_explorer"' '`open_explorer` is not an action'
    Get-Problem '"ctrl+shift+0" = "profile_0"' '"profile_0"' '`profile_0` is not an action: the tabs and the profiles count from 1'
    Get-Problem '"Alt+O" = "close_tab"' '"Alt+O"' ('`Alt+O` repeats the shortcut of line {0}' -f (Get-LineNumber '"alt+o" = "close_other_tabs"'))
)

# What the tabs of Probe and of Probe 2 run.
$probeLine = $probeCommand.Replace('%MEIGA_CHECK_WORK%', $work)
$probe2Line = "$probeLine -Second"

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

# Checks that meiga drew nothing after the first $From lines of the log, a
# while after the keys of $What.
function Assert-NothingDrawn([int]$From, [string]$What) {
    Start-Sleep -Milliseconds 200
    $frames = @(Get-Frames $From)
    if ($frames.Count -gt 0) {
        Show-Lines $frames
        throw "$What drew $($frames.Count) frames, and it has to draw nothing."
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

# Waits until a tab opens with a program that is not one of $Known, and
# returns its process, from WMI.
function Wait-NewShell([int[]]$Known, [string]$What) {
    $new = { @(Get-Shells | Where-Object { [int]$_.ProcessId -notin $Known }) }
    Wait-For { @(& $new).Count -gt 0 } { "$What did not open a tab with a program." }
    $shells = @(& $new)
    if ($shells.Count -ne 1) {
        throw "$What opened $($shells.Count) tabs, with the command lines: $(($shells | ForEach-Object CommandLine) -join '; ')."
    }
    $shells[0]
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

# Waits until the consoles of the tabs have ended, once meiga has.
function Wait-ConsolesEnded {
    foreach ($console in Get-Consoles) {
        Wait-Ended ([int]$console.ProcessId) "The console $($console.Name) of a tab"
    }
}

# Waits for the message box of meiga, and checks that it is in front of its
# window, owned by it, with the title meiga and one button. Returns it, with
# its button and its text, with each line ending in a line feed.
function Get-Dialog {
    Wait-For { @($api::DialogsOf($thread)).Count -gt 0 } { 'The window did not show a message box.' }
    $dialogs = @($api::DialogsOf($thread))
    if ($dialogs.Count -ne 1) {
        throw "The window showed $($dialogs.Count) message boxes instead of one."
    }
    $dialog = $dialogs[0]
    $owner = $api::GetWindow($dialog, $GW_OWNER)
    if ($owner -ne $window) {
        throw "The owner of the message box is the window $owner, instead of the one of $exe, $window."
    }
    $title = $api::TextOf($dialog)
    if ($title -cne 'meiga') {
        throw "The message box has the title '$title' instead of 'meiga'."
    }
    $buttons = @($api::ButtonsOf($dialog))
    if ($buttons.Count -ne 1) {
        throw "The message box has $($buttons.Count) buttons instead of one."
    }
    [pscustomobject]@{
        Window = $dialog
        Button = $buttons[0]
        Text = $api::MessageOf($dialog).Replace("`r`n", "`n")
    }
}

# Presses the button of the message box $Dialog, as its keyboard or its
# mouse does, and waits until the box closes.
function Close-Dialog($Dialog) {
    $id = $api::GetDlgCtrlID($Dialog.Button)
    if (-not $api::PostMessageW($Dialog.Window, $WM_COMMAND, [UIntPtr]::new([uint64]$id), $Dialog.Button)) {
        throw "Could not press the button of the message box: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
    Wait-For { -not $api::IsWindow($Dialog.Window) } { 'The message box did not close when its button was pressed.' }
}

# If the pointer is over the window, moves it out, above the window: the
# moves that Windows makes at its place would reach the bar, which would draw.
function Move-Pointer {
    $point = [Meiga.Tools.ShortcutsCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.ShortcutsCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = [Meiga.Tools.ShortcutsCheck+RECT]::new()
    if (-not $api::ScreenToClient($window, [ref]$inside) -or -not $api::GetClientRect($window, [ref]$rect) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = [Meiga.Tools.ShortcutsCheck+POINT]::new()
    $title.X = [Math]::Floor($rect.Right / 2)
    $title.Y = -8
    $null = $api::ClientToScreen($window, [ref]$title)
    if (-not $api::SetCursorPos($title.X, $title.Y)) {
        throw "Could not move the pointer out of the window: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# Shares the input of this thread with the one of the window, whose key
# state Send-Keys sets: SetKeyboardState sets the key state of the thread
# that calls it. This thread needs a message queue, which PeekMessageW
# creates.
function Join-Input {
    $message = [Meiga.Tools.ShortcutsCheck+MSG]::new()
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
# $Extended. With $Repeat, the key goes down again, as Windows repeats a key
# that stays down.
function Get-Key([int]$Code, [bool]$Down, [bool]$System = $false, [bool]$Extended = $false, [bool]$Repeat = $false) {
    $scan = $api::MapVirtualKeyExW($Code, $MAPVK_VK_TO_VSC, $layout)
    [long]$bits = 1 -bor ($scan -shl 16)
    if ($Extended) { $bits = $bits -bor 0x01000000L }
    if ($System) { $bits = $bits -bor 0x20000000L }
    if ($Down) {
        # The key was down (bit 30).
        if ($Repeat) { $bits = $bits -bor 0x40000000L }
        $message = if ($System) { $WM_SYSKEYDOWN } else { $WM_KEYDOWN }
    }
    else {
        # The key was down (bit 30), and goes up (bit 31).
        $bits = $bits -bor 0xC0000000L
        $message = if ($System) { $WM_SYSKEYUP } else { $WM_KEYUP }
    }
    $message, $Code, $bits
}

# The key messages of typing $Chord, such as "Ctrl+Shift+T", "Alt+O",
# "AltGr+2" or "Ctrl+PageDown", and the keys that are down meanwhile: its
# modifier keys go down, its key goes down, again $Repeats times, and up, and
# its modifier keys go up, the other way round. With Alt and without Ctrl,
# they are system keys, as Windows sends them, but for Alt going up: after
# another key, Windows sends it as a key that is not a system one, and then
# it does not open the menu of the window.
function Get-Chord([string]$Chord, [int]$Repeats = 0) {
    $parts = $Chord -split '\+'
    $modifiers = @(foreach ($part in ($parts | Select-Object -SkipLast 1)) {
            if (-not $chordKeys.ContainsKey($part)) { throw "$part, in $Chord, is not a modifier key of the check." }
            $chordKeys[$part]
        })
    $down = @($modifiers | ForEach-Object { $_.Down })
    $system = $down -contains $VK_MENU -and $down -notcontains $VK_CONTROL
    $key = $parts[-1]
    $code = if ($key -cmatch '^[0-9A-Z]$') { [int][char]$key } else { [int][ConsoleKey]$key }
    $extended = $code -in $VK_PRIOR, $VK_NEXT
    $reversed = @(for ($index = $modifiers.Count - 1; $index -ge 0; $index--) { $modifiers[$index] })
    $messages = @(
        foreach ($modifier in $modifiers) { Get-Key $modifier.Code $true $system $modifier.Extended }
        Get-Key $code $true $system $extended
        for ($repeat = 0; $repeat -lt $Repeats; $repeat++) { Get-Key $code $true $system $extended $true }
        Get-Key $code $false $system $extended
        foreach ($modifier in $reversed) { Get-Key $modifier.Code $false ($system -and $modifier.Code -ne $VK_MENU) $modifier.Extended }
    )
    [pscustomobject]@{ Messages = [long[]]$messages; Down = [int[]]$down; System = $system }
}

# Posts key messages at once, with the keys of $Down down and no other one
# (see PostAll). The input has to be shared with the thread of the window:
# see Join-Input.
function Send-Keys([long[]]$Messages, [int[]]$Down = @()) {
    $state = [byte[]]::new(256)
    foreach ($key in $Down) { $state[$key] = 0x80 }
    $api::PostAll($window, $thread, $Messages, $state)
}

# Types $Chord, with $Repeats repeats of its key (see Get-Chord) and, with
# $Mark, Z after it, with the same modifier keys down: none of the chords of
# Z is a shortcut, and the program gets Z once the window has taken all the
# keys before it.
function Send-Chord([string]$Chord, [int]$Repeats = 0, [switch]$Mark) {
    $typed = Get-Chord $Chord $Repeats
    $messages = $typed.Messages
    if ($Mark) {
        $messages += @(Get-Key $VK_Z $true $typed.System) + @(Get-Key $VK_Z $false $typed.System)
    }
    Send-Keys $messages $typed.Down
}

# The file where the program of the tab of $Id writes the keys that it gets.
function Get-KeyFile([int]$Id) {
    Join-Path $work "keys-$Id.txt"
}

# The keys that the program of the tab of $Id has got: the text of its file,
# which it may be writing.
function Get-KeyText([int]$Id) {
    $file = Get-KeyFile $Id
    if (-not (Test-Path -LiteralPath $file)) { return '' }
    $stream = [IO.FileStream]::new($file, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
    try { [IO.StreamReader]::new($stream).ReadToEnd() }
    finally { $stream.Dispose() }
}

# Waits until the program of the tab of $Id gets Z after the first $From
# characters of its file, and returns the keys that it got before Z.
function Wait-Mark([int]$Id, [int]$From, [string]$What) {
    Wait-For { (Get-KeyText $Id).Substring($From).EndsWith('90+ 90- ', [StringComparison]::Ordinal) } {
        "$What`: the program of the tab did not get the Z after it: it got '$((Get-KeyText $Id).Substring($From).Trim())'."
    }
    $text = (Get-KeyText $Id).Substring($From)
    $text.Substring(0, $text.Length - '90+ 90- '.Length).Trim()
}

# Types Z for the program of the tab of $Id, which is shown, and waits until
# it gets it: then it has got all the keys before it. Returns where its file
# ends.
function Sync-Keys([int]$Id) {
    $from = (Get-KeyText $Id).Length
    Send-Keys (@(Get-Key $VK_Z $true) + @(Get-Key $VK_Z $false))
    $null = Wait-Mark $Id $from 'Z'
    (Get-KeyText $Id).Length
}

# Types $Chord, and Z after it, for the program of the tab of $Id, which is
# shown, and checks that the program gets $Expected before Z: the keys that
# meiga does not take.
function Assert-Keys([int]$Id, [string]$Chord, [string]$Expected) {
    $from = Sync-Keys $Id
    Send-Chord $Chord -Mark
    $got = Wait-Mark $Id $from $Chord
    if ($got -cne $Expected) {
        throw "With $Chord, the program got the keys '$got' instead of '$Expected'."
    }
}

# Types $Chord, with $Repeats repeats of its key, and Z after it, which shows
# the tab of $Id, and hides the ones of $Hidden: once the program of the tab
# gets Z, checks that the window drew one full frame, if the key did not
# repeat.
function Show-Tab([string]$Chord, [int]$Id, [int[]]$Hidden, [int]$Repeats = 0) {
    Wait-Quiet 400
    $start = @(Get-Log).Count
    $from = (Get-KeyText $Id).Length
    Send-Chord $Chord $Repeats -Mark
    Wait-Shown $Id $Hidden "After $Chord"
    $null = Wait-Mark $Id $from $Chord
    if ($Repeats -eq 0) { Assert-OneFull $start $Chord }
}

# Types $Chord, with $Repeats repeats of its key, which opens a tab whose
# program runs $Command, and shows it, while the tabs of $Known are hidden.
# Returns the process of its program, once it reads its keys, and has got Z
# after the chord: then the window has taken all its keys.
function Open-Tab([string]$Chord, [string]$Command, [int[]]$Known, [int]$Repeats = 0) {
    Send-Chord $Chord $Repeats
    $shell = Wait-NewShell $Known $Chord
    if ($shell.CommandLine -cne $Command) {
        throw "$Chord started '$($shell.CommandLine)' instead of '$Command'."
    }
    $id = [int]$shell.ProcessId
    Wait-For { Test-Path -LiteralPath (Get-KeyFile $id) } { "After $Chord, the script of the new tab did not start." }
    Wait-Shown $id $Known "After $Chord"
    $null = Sync-Keys $id
    $id
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
$pointer = $null
$attached = $false
$passed = $false
try {
    $process = [Diagnostics.Process]::Start($info)
    $started = $process.StartTime

    # 1. A message box tells the errors of [shortcuts]: each one with its
    # line and its column.
    Wait-For { $api::WindowOf($process.Id, 'meiga') -ne [IntPtr]::Zero } { "$exe did not open its window." }
    $window = $api::WindowOf($process.Id, 'meiga')
    $processId = [uint32]0
    $thread = $api::GetWindowThreadProcessId($window, [ref]$processId)
    $layout = $api::GetKeyboardLayout($thread)
    $dialog = Get-Dialog
    $expected = "There are errors in $($config):`n`n$($problems -join "`n")`n`nmeiga ignored them, and uses its default values instead."
    if ($dialog.Text -cne $expected) {
        throw "The message box says:`n$($dialog.Text)`n... instead of:`n$expected"
    }
    Move-Pointer
    Close-Dialog $dialog
    Write-Host "OK: a message box tells the $($problems.Count) errors of [shortcuts] in config.toml, each with its line and its column, and that meiga ignored them."

    # 2. With one tab, Ctrl+T and Ctrl+W reach the program, and so does
    # Ctrl+6, which config.toml frees.
    Wait-For { @(Get-Shells).Count -gt 0 } { 'The first tab did not start its program.' }
    $shells = @(Get-Shells)
    if ($shells.Count -ne 1 -or $shells[0].CommandLine -cne $probeLine) {
        throw "With one tab, the programs of $exe are $(($shells | ForEach-Object { "$($_.ProcessId): $($_.CommandLine)" }) -join '; '), instead of the script of Probe: $probeLine."
    }
    $first = [int]$shells[0].ProcessId
    Wait-For { Test-Path -LiteralPath (Get-KeyFile $first) } { 'The script of the first tab did not start.' }
    Wait-Shown $first @() 'With one tab'
    Join-Input
    Assert-Keys $first 'Ctrl+T' '17+ 84+ 84- 17-'
    Assert-Keys $first 'Ctrl+W' '17+ 87+ 87- 17-'
    Assert-Keys $first 'Ctrl+6' '17+ 54+ 54- 17-'
    if (@(Get-Shells).Count -ne 1) {
        throw "Ctrl+T, Ctrl+W or Ctrl+6 opened a tab: $exe has $(@(Get-Shells).Count) programs."
    }
    Wait-Shown $first @() 'After Ctrl+T, Ctrl+W and Ctrl+6'
    Write-Host 'OK: Ctrl+T and Ctrl+W reach the program, and so does Ctrl+6, which config.toml frees; no tab opens nor closes.'

    # 3. Ctrl+Shift+T opens one tab, also while T repeats: twice, so that
    # the second repeat finds T still taken after the first one.
    $second = Open-Tab 'Ctrl+Shift+T' $probeLine @($first) -Repeats 2
    Wait-Quiet 400
    if (@(Get-Shells).Count -ne 2) {
        throw "Ctrl+Shift+T, while T repeated twice, opened $(@(Get-Shells).Count - 1) tabs instead of one."
    }
    Write-Host 'OK: Ctrl+Shift+T opens one tab, also while T repeats twice.'

    # 4. AltGr+2 reaches the program, while Ctrl+Alt+2 shows the second tab,
    # as config.toml says.
    Show-Tab 'Ctrl+1' $first @($second)
    Assert-Keys $first 'AltGr+2' '17+ 18+ 50+ 50- 18- 17-'
    Wait-Shown $first @($second) 'After AltGr+2'
    Show-Tab 'Ctrl+Alt+2' $second @($first)
    Write-Host 'OK: AltGr+2, the left Ctrl and the right Alt, reaches the program, while Ctrl+Alt+2, with the left Alt, shows the second tab, as config.toml says.'

    # 5. Ctrl+Shift+D puts a copy of the first tab after it, and shows it;
    # Ctrl+3, Ctrl+1 and Ctrl+2 show the tabs at those places, and Ctrl+9
    # the last one. Ctrl+4, with three tabs, shows none: neither the last
    # one nor the first one, while the copy, in the middle, is shown.
    Show-Tab 'Ctrl+1' $first @($second)
    $copy = Open-Tab 'Ctrl+Shift+D' $probeLine @($first, $second)
    Show-Tab 'Ctrl+3' $second @($first, $copy)
    Show-Tab 'Ctrl+1' $first @($second, $copy)
    Show-Tab 'Ctrl+9' $second @($first, $copy)
    Show-Tab 'Ctrl+2' $copy @($first, $second)
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Assert-Keys $copy 'Ctrl+4' '17+ 17-'
    Assert-NothingDrawn $start 'Ctrl+4, with three tabs,'
    Wait-Shown $copy @($first, $second) 'After Ctrl+4'
    Write-Host 'OK: Ctrl+Shift+D opens a copy of the tab right after it, and shows it; Ctrl+3, Ctrl+1 and Ctrl+2 show the tabs at those places, and Ctrl+9 the last one, each with one full frame; Ctrl+4, with three tabs, draws nothing, and its 4 does not reach the program.'

    # 6. Ctrl+PageDown and Ctrl+PageUp go round the tabs: from the copy to
    # the second tab, the last one, and then to the first one; and back to
    # the second one. While PageUp repeats, it goes on: to the copy, and to
    # the first tab.
    Show-Tab 'Ctrl+PageDown' $second @($first, $copy)
    Show-Tab 'Ctrl+PageDown' $first @($second, $copy)
    Show-Tab 'Ctrl+PageUp' $second @($first, $copy)
    Show-Tab 'Ctrl+PageUp' $first @($second, $copy) -Repeats 1
    # A key that went down for the program is its own until it goes up:
    # PageDown goes down for the program of the first tab and, with Ctrl
    # down then, it repeats as Ctrl+PageDown, and goes up. The program gets
    # all of them, and no tab is shown.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    $from = Sync-Keys $first
    Send-Keys (@(Get-Key $VK_NEXT $true $false $true) + @(Get-Key $VK_Z $true) + @(Get-Key $VK_Z $false))
    $got = Wait-Mark $first $from 'PageDown'
    if ($got -cne '34+') {
        throw "With PageDown, the program got the keys '$got' instead of '34+'."
    }
    $from = (Get-KeyText $first).Length
    $held = @(
        Get-Key $VK_CONTROL $true
        Get-Key $VK_NEXT $true $false $true $true
        Get-Key $VK_NEXT $false $false $true
        Get-Key $VK_CONTROL $false
        Get-Key $VK_Z $true
        Get-Key $VK_Z $false
    )
    Send-Keys $held @($VK_CONTROL, $VK_LCONTROL, $VK_NEXT)
    $got = Wait-Mark $first $from 'With Ctrl, while PageDown was down'
    if ($got -cne '17+ 34+ 34- 17-') {
        throw "With Ctrl, while PageDown was down for it, the program got the keys '$got' instead of '17+ 34+ 34- 17-'."
    }
    Assert-NothingDrawn $start 'Ctrl, while PageDown was down for the program,'
    Wait-Shown $first @($second, $copy) 'After Ctrl, while PageDown was down'
    Write-Host 'OK: Ctrl+PageDown and Ctrl+PageUp show the next and the previous tab, round the bar, and PageUp goes on while it repeats; PageDown, down for the program before Ctrl, goes to it while it repeats and when it goes up, and shows no tab.'

    # 7. Ctrl+Shift+2 and Ctrl+Shift+1 open the second and the first profile
    # of the menu at the end, where Ctrl+4 and Ctrl+5 show them; with two
    # profiles, Ctrl+Shift+3 opens nothing.
    $fourth = Open-Tab 'Ctrl+Shift+2' $probe2Line @($first, $copy, $second)
    $fifth = Open-Tab 'Ctrl+Shift+1' $probeLine @($first, $copy, $second, $fourth)
    $others = @($first, $copy, $second, $fourth)
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Assert-Keys $fifth 'Ctrl+Shift+3' '17+ 16+ 16- 17-'
    Assert-NothingDrawn $start 'Ctrl+Shift+3, with two profiles,'
    if (@(Get-Shells).Count -ne 5) {
        throw "Ctrl+Shift+3, with two profiles, opened a tab: $exe has $(@(Get-Shells).Count) programs."
    }
    Wait-Shown $fifth $others 'After Ctrl+Shift+3'
    Show-Tab 'Ctrl+4' $fourth @($first, $copy, $second, $fifth)
    Show-Tab 'Ctrl+5' $fifth $others
    Write-Host 'OK: Ctrl+Shift+2 and Ctrl+Shift+1 open Probe 2 and Probe, the profiles at those places of the menu, at the end; Ctrl+Shift+3, with two profiles, opens nothing, and its 3 does not reach the program.'

    # 8. Ctrl+8 shows the last of five tabs, as config.toml says.
    Show-Tab 'Ctrl+1' $first @($copy, $second, $fourth, $fifth)
    Show-Tab 'Ctrl+8' $fifth $others
    Write-Host 'OK: Ctrl+8 shows the last of five tabs, as config.toml says, instead of the eighth.'

    # 9. Alt+R closes the tabs on the right of the third tab, as config.toml
    # says, and the two on its left stay: Ctrl+2 shows the copy, in the
    # middle. Then Alt+O closes the other tabs, on both sides, as the first
    # line of its chord says.
    Show-Tab 'Ctrl+3' $second @($first, $copy, $fourth, $fifth)
    Assert-Keys $second 'Alt+R' '18+ 18-'
    foreach ($right in $fourth, $fifth) {
        Wait-Ended $right 'The program of a tab that Alt+R closed'
    }
    Wait-Shown $second @($first, $copy) 'After Alt+R'
    Show-Tab 'Ctrl+2' $copy @($first, $second)
    Assert-Keys $copy 'Alt+O' '18+ 18-'
    foreach ($other in $first, $second) {
        Wait-Ended $other 'The program of a tab that Alt+O closed'
    }
    Wait-Shown $copy @() 'After Alt+O'
    Wait-For { @(Get-Shells).Count -eq 1 -and @(Get-Consoles).Count -eq 1 } {
        "After Alt+O, $exe has $(@(Get-Shells).Count) programs and $(@(Get-Consoles).Count) consoles instead of one."
    }
    Write-Host 'OK: Alt+R closes the tabs on the right of the tab that is shown, and the ones on its left stay; Alt+O closes the other tabs, on both sides, as its first line in config.toml says; their programs end, and the program of the tab gets Alt, and not R nor O.'

    # 10. Ctrl+Shift+W closes the last tab, and the window.
    Send-Chord 'Ctrl+Shift+W'
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after Ctrl+Shift+W closed its last tab."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Wait-Ended $copy 'The program of the last tab'
    Wait-ConsolesEnded
    Write-Host 'OK: Ctrl+Shift+W closes the last tab, and the window: it ends with exit code 0, and the program of the tab and the consoles end too.'
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
    if ($passed) {
        Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $work) {
        Write-Host "Kept $work"
    }
}
