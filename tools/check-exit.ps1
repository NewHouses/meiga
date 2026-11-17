<#
.SYNOPSIS
Checks what meiga.exe does when the program of a tab ends: a clean end, with
exit code 0, closes the tab, and the last one the window; any other end, or
a program that cannot start, leaves the tab, which tells it, and then Enter
starts the program again, and Ctrl+D closes the tab.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and an APPDATA of its
own, whose config.toml has two profiles, Probe, the default one, and Later,
which run the same PowerShell script, Later in a folder that is not there at
first; and hides the shells that meiga finds. The script of each tab writes
its folder and the size of its console in a file of its own, and in another
one the keys that its program gets; and it ends when the check tells it to,
with an exit code, after some text or none.

The check reads the lines of the terminal with a triple click, which copies
them to the clipboard, and types the keys as check-shortcuts.ps1 does. The
keys that nothing can show go with a move of the pointer onto a button of
the bar, after them: once the window draws the button, it has taken them.
It checks that:

- when the program of a hidden tab ends with exit code 0, its tab closes,
  and only the bar is drawn;
- when the program of the tab that is shown ends with exit code 0, its tab
  closes, and the one on its left is shown, with one full frame;
- when the program of the last tab ends with exit code 3, after writing
  "bye", the tab stays, with "bye", the line of the exit code in a row of its
  own below it, and below that what can be done then; meiga goes on;
- then the keys that are not shortcuts do nothing, and draw nothing: the
  selection stays; and Ctrl+Enter and AltGr+D neither start the program
  again nor close the tab;
- Enter, which repeats once, starts the program again in the same tab, with
  the same command line and folder, and shows it with one full frame; the new
  program gets neither the repeat of Enter nor Enter going up; and what the
  last program wrote goes to the history, where Ctrl+Shift+Home shows it;
- while the program of a tab runs, Restart tab is disabled in the menu of
  the tab: Shift+F10 opens it with Duplicate tab highlighted, and Down goes
  to Close tab, which Enter chooses;
- when the program ends with exit code 1 at the start of a row, the line of
  the exit code goes in that row;
- when the program of a hidden tab ends with exit code 4, its tab stays, and
  nothing is drawn; on the menu that the right button opens on it, Down and
  Down highlight Restart tab, which Enter chooses: the program starts again
  in the hidden tab, with the lower priority of a hidden tab from the start,
  and the tab is not shown;
- Ctrl+D closes the tab that is shown, whose program ended, and shows the
  next one, whose program gets neither D nor Ctrl going up;
- Ctrl+Shift+2 opens Later, whose folder is not there, and its tab tells why
  its program could not start, and what can be done then; Enter tries
  again, with one full frame, and once the folder is there, starts it there;
- while a message box is open, the programs of a hidden tab and of the tab
  that is shown end with exit code 0, and once the box closes, both tabs
  close, and the last one closes the window: meiga.exe ends with exit code
  0, and the consoles end too;
- when the program of the command line of meiga.exe ends with exit code 0,
  meiga.exe ends with exit code 0 too.

It needs PowerShell 7 (pwsh) in the PATH, for the script of the tabs. It
uses the clipboard, and puts back the text that it had.

If a check fails, the script ends meiga.exe and keeps its folder, with the
frame log and the files of the tabs.

.PARAMETER Path
The executable to check. Defaults to target\release\meiga.exe.

.PARAMETER TimeoutSeconds
How long to wait for the window, for each program to start and to end, for
each line of the terminal, and for each frame.
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

if (-not ('Meiga.Tools.ExitCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name ExitCheck -MemberDefinition @'
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
public static extern uint GetDpiForWindow(IntPtr window);

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
public static extern bool IsWindowVisible(IntPtr window);

[DllImport("user32.dll")]
public static extern IntPtr GetWindow(IntPtr window, uint command);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool EnumWindows(EnumProc callback, IntPtr parameter);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool EnumThreadWindows(uint thread, EnumProc callback, IntPtr parameter);

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern int GetClassNameW(IntPtr window, System.Text.StringBuilder name, int count);

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
static extern uint GetDlgItemTextW(IntPtr dialog, int id, System.Text.StringBuilder text, int count);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool OpenClipboard(IntPtr owner);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool EmptyClipboard();

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool CloseClipboard();

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

// The text of a message box: the one of its control 0xFFFF.
public static string MessageOf(IntPtr dialog)
{
    var text = new System.Text.StringBuilder(16384);
    GetDlgItemTextW(dialog, 0xFFFF, text, text.Capacity);
    return text.ToString();
}

// Empties the clipboard. Returns whether it could.
public static bool ClearClipboard()
{
    if (!OpenClipboard(IntPtr.Zero)) return false;
    try { return EmptyClipboard(); }
    finally { CloseClipboard(); }
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
$api = [Meiga.Tools.ExitCheck]

$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_SYSKEYDOWN = 0x0104
$WM_SYSKEYUP = 0x0105
$WM_COMMAND = 0x0111
$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_RBUTTONDOWN = 0x0204
$WM_RBUTTONUP = 0x0205
$MK_LBUTTON = 0x0001
$MK_RBUTTON = 0x0002
$MAPVK_VK_TO_VSC = 0
$GW_OWNER = 4
$IDCANCEL = 2
$VK_CONTROL = 0x11
$VK_MENU = 0x12
$VK_Z = 0x5A
$VK_F10 = 0x79
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
# The keys that a keyboard sends as extended ones: Page Up, Page Down, End,
# Home, the arrows, Insert and Delete.
$extendedKeys = @(0x21..0x28) + @(0x2D, 0x2E)

# The script of the tabs of Probe and Later. It writes a line with its
# number, its folder and the size of its console, and then the keys that its
# program gets, until the check tells it how to end: with an exit code, after
# some text or none.
$program = @'
param([Parameter(Mandatory)][string]$Work)
$ErrorActionPreference = 'Stop'
# The cursor is hidden, so that it never blinks; then a line that tells the
# screen of this program from the ones of the programs before it in its tab.
[Console]::Out.Write("$([char]27)[?25lprobe-$PID`r`n")
Add-Type -Namespace MeigaCheck -Name Input -MemberDefinition @"
[DllImport("kernel32.dll")] public static extern System.IntPtr GetStdHandle(int handle);
[DllImport("kernel32.dll")] public static extern bool ReadConsoleInputW(System.IntPtr handle, [System.Runtime.InteropServices.Out] byte[] records, uint count, out uint read);
[DllImport("kernel32.dll")] public static extern uint WaitForMultipleObjects(uint count, System.IntPtr[] handles, bool all, uint milliseconds);
"@
$in = [MeigaCheck.Input]::GetStdHandle(-10)
# The check sets it once it has written how this script ends.
$stop = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, "meiga-check-exit-$PID")
# The folder of the program, and the size of its console: the grid of the
# terminal.
[IO.File]::WriteAllText((Join-Path $Work "info-$PID.txt"), "$([Environment]::CurrentDirectory)`n$([Console]::WindowWidth) $([Console]::WindowHeight)`n")
# Up to 64 INPUT_RECORD of 20 bytes: the type of the event in the first two
# and, for a key, whether it goes down from byte 4, and its virtual-key code
# from byte 10.
$records = [byte[]]::new(20 * 64)
# The virtual-key code of each key, with + when it goes down and - when it
# goes up, in a file that the check reads while this script writes it. Once
# the file is there, the script reads the keys.
$file = [IO.FileStream]::new((Join-Path $Work "keys-$PID.txt"), [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::ReadWrite)
$handles = [IntPtr[]]@($in, $stop.SafeWaitHandle.DangerousGetHandle())
while ($true) {
    $signaled = [MeigaCheck.Input]::WaitForMultipleObjects(2, $handles, $false, [uint32]::MaxValue)
    if ($signaled -eq 1) {
        # Its exit code, and the text to write before, if there is one.
        $end = [IO.File]::ReadAllLines((Join-Path $Work "exit-$PID.txt"))
        if ($end.Count -gt 1) { [Console]::Out.Write($end[1]) }
        exit [int]$end[0]
    }
    if ($signaled -ne 0) { exit 99 }
    $read = 0
    if (-not [MeigaCheck.Input]::ReadConsoleInputW($in, $records, 64, [ref]$read)) { exit 98 }
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

$name = "meiga-check-exit-$PID"
# All the files of the check, with the APPDATA of meiga. With the long names
# of the folders, which .NET gives as the current one.
$work = Join-Path ($api::LongPathOf([IO.Path]::GetTempPath())) $name
$appdata = Join-Path $work 'appdata'
$config = Join-Path $appdata 'meiga\config.toml'
$script = Join-Path $work 'probe.ps1'
$log = Join-Path $work 'frames.csv'
# The folder of Later, which the check makes once its tab tells that it is
# not there.
$later = Join-Path $work 'later'
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

# The configuration: Probe, the default profile, and Later, which runs the
# same script in a folder that is not there yet; and the shells that meiga
# finds, hidden.
$probeCommand = 'pwsh -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%MEIGA_CHECK_WORK%\probe.ps1" -Work "%MEIGA_CHECK_WORK%"'
$configLines = @(
    '# Written by tools/check-exit.ps1.'
    'default_profile = "Probe"'
    ''
    '[[profiles]]'
    'name = "Probe"'
    "command = '$probeCommand'"
    "directory = '%MEIGA_CHECK_WORK%'"
    ''
    '[[profiles]]'
    'name = "Later"'
    "command = '$probeCommand'"
    "directory = '%MEIGA_CHECK_WORK%\later'"
    foreach ($shell in $hidden) {
        ''
        '[[profiles]]'
        "name = $(ConvertTo-TomlString $shell)"
        'hidden = true'
    }
)
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText($config, ($configLines -join "`n") + "`n", $utf8)

# What the tabs of Probe and of Later run.
$probeLine = $probeCommand.Replace('%MEIGA_CHECK_WORK%', $work)
# What a tab shows below the line that tells how its program ended, or why it
# could not start (see crates/meiga/src/session.rs).
$hint = 'You can now close this tab with Ctrl+D, or press Enter to restart.'

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

# The areas that the frames of $Lines presented: "x y w h;x y w h".
function Get-Areas($Lines) {
    foreach ($line in $Lines) {
        foreach ($area in ($line.areas -split ';' | Where-Object { $_ })) {
            $x, $y, $w, $h = [int[]]($area -split ' ')
            New-Rect $x $y ($x + $w) ($y + $h)
        }
    }
}

# Waits until meiga draws after the first $From lines of the log, and stops
# drawing; then checks that all it drew is inside $Rect; otherwise throws
# $What.
function Assert-Inside([int]$From, $Rect, [string]$What) {
    Wait-For { @(Get-Frames $From).Count -gt 0 } { "$What`: the window drew nothing." }
    Wait-Quiet 400
    $frames = @(Get-Frames $From)
    $outside = @(Get-Areas $frames | Where-Object { -not (Test-Inside $_ $Rect) })
    if ($outside.Count -gt 0) {
        Show-Lines $frames
        throw "$What`: the window did not draw only inside $(Show-Rect $Rect)."
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

# Waits until a tab starts a program that is not one of $Known, which runs
# the command line of Probe, and returns its number once its script reads
# its keys.
function Wait-NewShell([int[]]$Known, [string]$What) {
    $new = { @(Get-Shells | Where-Object { [int]$_.ProcessId -notin $Known }) }
    Wait-For { @(& $new).Count -gt 0 } { "$What did not start a program." }
    $shells = @(& $new)
    if ($shells.Count -ne 1) {
        throw "$What started $($shells.Count) programs, with the command lines: $(($shells | ForEach-Object CommandLine) -join '; ')."
    }
    if ($shells[0].CommandLine -cne $probeLine) {
        throw "$What started '$($shells[0].CommandLine)' instead of '$probeLine'."
    }
    $id = [int]$shells[0].ProcessId
    Wait-For { Test-Path -LiteralPath (Get-KeyFile $id) } { "$What started a program whose script did not start." }
    $id
}

# The priority of the reader thread of the tab whose program is $Program,
# or $null if it has none.
function Get-Priority([int]$Program) {
    $api::PriorityOf($process.Id, "conpty reader $Program")
}

# Waits until the tab of $Shown is the one with the reader of normal
# priority, and the ones of $Hidden have the lower one. With $Shown 0, the
# tab that is shown has no program.
function Wait-Shown([int]$Shown, [int[]]$Hidden, [string]$What) {
    Wait-For {
        ($Shown -eq 0 -or (Get-Priority $Shown) -eq $THREAD_PRIORITY_NORMAL) -and
        @($Hidden | Where-Object { (Get-Priority $_) -ne $THREAD_PRIORITY_BELOW_NORMAL }).Count -eq 0
    } {
        $all = @($Shown | Where-Object { $_ -ne 0 }) + $Hidden | ForEach-Object { "$_ $(Get-Priority $_)" }
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

# Tells the script of the tab of $Id to end with exit code $Code, after
# writing $Text, if it is not empty, without a new line.
function Stop-Probe([int]$Id, [int]$Code, [string]$Text = '') {
    $end = if ($Text) { "$Code`n$Text`n" } else { "$Code`n" }
    [IO.File]::WriteAllText((Join-Path $work "exit-$Id.txt"), $end, $utf8)
    $stop = [Threading.EventWaitHandle]::OpenExisting("meiga-check-exit-$Id")
    try { $null = $stop.Set() }
    finally { $stop.Dispose() }
}

# Waits until the program of the tab of $Id has ended, and the reader thread
# of its session too, whose last work is to give the terminal how the
# program ended, and to post the wake-up that tells it to the window: what
# the check posts to the window from then on comes after it. The files of
# the script go, so that a program that gets the same number later can
# create them.
function Wait-Gone([int]$Id, [string]$What) {
    Wait-Ended $Id $What
    Wait-For { $null -eq (Get-Priority $Id) } { "$What ended, and the reader thread of its session did not." }
    foreach ($file in "keys-$Id.txt", "info-$Id.txt", "exit-$Id.txt") {
        Remove-Item -LiteralPath (Join-Path $work $file) -ErrorAction SilentlyContinue
    }
}

# The folder where the script of the tab of $Id runs, and the size of its
# console, which it wrote when it started.
function Get-Info([int]$Id) {
    $folder, $size = [IO.File]::ReadAllLines((Join-Path $work "info-$Id.txt"))
    $columns, $rows = [int[]]($size -split ' ')
    [pscustomobject]@{ Folder = $folder; Columns = $columns; Rows = $rows }
}

# Checks that the program of the tab of $Id runs in $Folder.
function Assert-Folder([int]$Id, [string]$Folder, [string]$What) {
    $actual = (Get-Info $Id).Folder
    # Folders do not distinguish upper and lower case.
    if ($actual.TrimEnd('\') -ne $Folder.TrimEnd('\')) {
        throw "$What`: the program runs in $actual instead of $Folder."
    }
}

# The client area of the window.
function Get-Client {
    $client = [Meiga.Tools.ExitCheck+RECT]::new()
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

# The size of the grid, which the script of the tab of $Id wrote, and of its
# cells: the grid fills the client area of the window below the tab bar, 32
# pixels high at a scale of 100 %, but for the strip of the scroll bar, 12
# pixels, and less than a cell.
function Measure-Cells([int]$Id) {
    $info = Get-Info $Id
    $client = Get-Client
    $strip = Get-Scaled 12
    $script:barHeight = Get-Scaled 32
    $script:grid = [pscustomobject]@{ Columns = $info.Columns; Rows = $info.Rows }
    $script:cell = [pscustomobject]@{
        Width = [int][Math]::Floor(($client.Right - $strip) / $info.Columns)
        Height = [int][Math]::Floor(($client.Bottom - $barHeight) / $info.Rows)
    }
}

# The pixel in the middle of a cell of the terminal.
function Get-X([int]$Column) { $Column * $cell.Width + [Math]::Floor($cell.Width / 2) }
function Get-Y([int]$Row) { $barHeight + $Row * $cell.Height + [Math]::Floor($cell.Height / 2) }

# The layout of the bar with $Count tabs, as meiga computes it (see
# crates/meiga-core/src/ui/tabbar.rs): the bar, its tabs, and its buttons +
# and of the profiles.
function Get-Bar([int]$Count) {
    $client = Get-Client
    $height = Get-Scaled 32
    $gap = Get-Scaled 4
    $padding = Get-Scaled 8
    $newSide = Get-Scaled 28
    $profilesSide = Get-Scaled 24
    $width = [int][Math]::Min((Get-Scaled 240), [Math]::Floor([Math]::Max(0, $client.Right - 3 * $gap - $newSide - $profilesSide) / $Count))
    $tabs = @(foreach ($index in 0..($Count - 1)) {
            $left = $gap + $index * $width
            [pscustomobject]@{ Rect = New-Rect $left $gap ($left + $width) $height; TitleLeft = $left + $padding }
        })
    $newLeft = $gap + $Count * $width + $gap
    [pscustomobject]@{
        Rect = New-Rect 0 0 $client.Right $height
        Tabs = $tabs
        New = New-Rect $newLeft $gap ($newLeft + $newSide) ($gap + $newSide)
        Profiles = New-Rect ($newLeft + $newSide) $gap ($newLeft + $newSide + $profilesSide) ($gap + $newSide)
    }
}

# The pixel in the middle of $Rect.
function Get-Middle($Rect) {
    [pscustomobject]@{ X = [int][Math]::Floor(($Rect.Left + $Rect.Right) / 2); Y = [int][Math]::Floor(($Rect.Top + $Rect.Bottom) / 2) }
}

# A point of a tab, on its title, away from its close button.
function Get-TabPoint($Tab) {
    [pscustomobject]@{ X = $Tab.TitleLeft + 4; Y = (Get-Middle $Tab.Rect).Y }
}

# A mouse message at a point: the message, its wParam and its lParam.
function Get-Mouse([int]$Message, [int]$Keys, $Point) {
    $Message, $Keys, (([int]$Point.X -band 0xFFFF) -bor (([int]$Point.Y -band 0xFFFF) -shl 16))
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

# The text of the clipboard: nothing if it has none, or if another program
# keeps it open.
function Get-Text {
    try { Get-Clipboard -Raw } catch { $null }
}

# Puts $Text in the clipboard. Another program can keep it open for a moment,
# such as the clipboard history of Windows: then, it tries again.
function Set-Text([string]$Text) {
    Wait-For { try { Set-Clipboard -Value $Text } catch { $null }; (Get-Text) -ceq $Text } `
    { "Could not put '$(Show-Text $Text)' in the clipboard." }
}

# Text with its control characters in sight.
function Show-Text([string]$Text) {
    $Text -replace "`e", '\e' -replace "`r", '\r' -replace "`n", '\n'
}

# Reads the line at the row $Row of the view, with a triple click on its
# second column, which copies the whole line, with its rows joined if it
# wraps; and checks that it is $Start or, with $End, that it starts with
# $Start and ends with $End. Returns it. The click happens again while the
# line is not there yet, as the output of a program that has just started.
function Assert-Line([int]$Row, [string]$Start, [string]$End, [string]$What) {
    $ends = $PSBoundParameters.ContainsKey('End')
    $point = [pscustomobject]@{ X = Get-X 1; Y = Get-Y $Row }
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while ($true) {
        $script:marks++
        Set-Text "meiga-check-exit-$PID-$($script:marks)"
        Send-Mouse @(for ($click = 0; $click -lt 3; $click++) {
                Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $point
                Get-Mouse $WM_LBUTTONUP 0 $point
            })
        # The second click copies a word, and the third one the line; a click
        # one second later is a click again.
        $again = [DateTime]::UtcNow.AddSeconds(1)
        while ([DateTime]::UtcNow -lt $again) {
            $text = Get-Text
            $found = if ($null -eq $text) { $false }
            elseif ($ends) { $text.StartsWith($Start, 'Ordinal') -and $text.EndsWith($End, 'Ordinal') }
            else { $text -ceq $Start }
            if ($found) { return $text }
            Start-Sleep -Milliseconds 20
        }
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) while the check read the row $Row of the terminal."
        }
        if ([DateTime]::UtcNow -gt $deadline) {
            $expected = if ($ends) { "a line that starts with '$Start' and ends with '$End'" } else { "'$Start'" }
            throw "$What, the row $Row of the terminal has '$(Show-Text $text)' instead of $expected."
        }
    }
}

# Waits for the message box of meiga that asks before pasting, owned by its
# window, and returns it.
function Wait-Dialog([string]$What) {
    Wait-For { @($api::DialogsOf($thread)).Count -gt 0 } { "$What did not open a message box." }
    $dialogs = @($api::DialogsOf($thread))
    if ($dialogs.Count -ne 1) {
        throw "$What opened $($dialogs.Count) message boxes instead of one."
    }
    $dialog = $dialogs[0]
    $owner = $api::GetWindow($dialog, $GW_OWNER)
    if ($owner -ne $window) {
        throw "The owner of the message box is the window $owner, instead of the one of $exe, $window."
    }
    $text = $api::MessageOf($dialog)
    if (-not $text.StartsWith('You are about to paste 2 lines,', 'Ordinal')) {
        throw "$What opened a message box that says: $text"
    }
    $dialog
}

# If the pointer is over the window, moves it out, above the window: the
# moves that Windows makes at its place would reach the bar, which would draw.
function Move-Pointer {
    $point = [Meiga.Tools.ExitCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.ExitCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = [Meiga.Tools.ExitCheck+RECT]::new()
    if (-not $api::ScreenToClient($window, [ref]$inside) -or -not $api::GetClientRect($window, [ref]$rect) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = [Meiga.Tools.ExitCheck+POINT]::new()
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
    $message = [Meiga.Tools.ExitCheck+MSG]::new()
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
# that stays down. F10 is a system key also without Alt, as Windows sends it.
function Get-Key([int]$Code, [bool]$Down, [bool]$System = $false, [bool]$Extended = $false, [bool]$Repeat = $false) {
    $scan = $api::MapVirtualKeyExW($Code, $MAPVK_VK_TO_VSC, $layout)
    [long]$bits = 1 -bor ($scan -shl 16)
    if ($Extended) { $bits = $bits -bor 0x01000000L }
    if ($System) { $bits = $bits -bor 0x20000000L }
    $asSystem = $System -or $Code -eq $VK_F10
    if ($Down) {
        # The key was down (bit 30).
        if ($Repeat) { $bits = $bits -bor 0x40000000L }
        $message = if ($asSystem) { $WM_SYSKEYDOWN } else { $WM_KEYDOWN }
    }
    else {
        # The key was down (bit 30), and goes up (bit 31).
        $bits = $bits -bor 0xC0000000L
        $message = if ($asSystem) { $WM_SYSKEYUP } else { $WM_KEYUP }
    }
    $message, $Code, $bits
}

# The key messages of typing $Chord, such as "Ctrl+Shift+T", "AltGr+D",
# "Enter" or "Shift+F10", with the names of System.ConsoleKey for the keys
# that are not a letter nor a digit, and the keys that are down meanwhile:
# its modifier keys go down, its key goes down, again $Repeats times, and
# up, and its modifier keys go up, the other way round. With Alt and without
# Ctrl, they are system keys, as Windows sends them, but for Alt going up:
# after another key, Windows sends it as a key that is not a system one, and
# then it does not open the menu of the window.
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
    $extended = $code -in $extendedKeys
    $reversed = @(for ($index = $modifiers.Count - 1; $index -ge 0; $index--) { $modifiers[$index] })
    $messages = @(
        foreach ($modifier in $modifiers) { Get-Key $modifier.Code $true $system $modifier.Extended }
        Get-Key $code $true $system $extended
        for ($repeat = 0; $repeat -lt $Repeats; $repeat++) { Get-Key $code $true $system $extended $true }
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

# Types $Chord, with $Repeats repeats of its key (see Get-Chord).
function Send-Chord([string]$Chord, [int]$Repeats = 0) {
    $typed = Get-Chord $Chord $Repeats
    Send-Keys $typed.Messages $typed.Down
}

# Types the keys of $Messages, with the keys of $Down down, which nothing
# shows, and moves the pointer onto a button of the bar with $Count tabs
# after them, in the same batch: the button + and the one of the profiles,
# in turn, so that what the pointer highlights always changes. Once the
# window draws that button, it has taken all the keys before it; then it
# has to stop drawing.
function Send-Synced([long[]]$Messages, [int[]]$Down, [int]$Count, [string]$What) {
    $bar = Get-Bar $Count
    $script:onNew = -not $script:onNew
    $button = if ($script:onNew) { $bar.New } else { $bar.Profiles }
    $from = @(Get-Log).Count
    Send-Keys ($Messages + (Get-Mouse $WM_MOUSEMOVE 0 (Get-Middle $button))) $Down
    Wait-For { @(Get-Areas (Get-Frames $from) | Where-Object { Test-Overlap $_ $button }).Count -gt 0 } {
        "After $What, the window did not draw the button of the bar under the pointer."
    }
    Wait-Quiet 400
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

# Types Z for the program of the tab of $Id, which is shown, and returns the
# keys that it got before Z, after the first $From characters of its file.
function Get-KeysBefore([int]$Id, [int]$From) {
    Send-Keys (@(Get-Key $VK_Z $true) + @(Get-Key $VK_Z $false))
    Wait-Mark $Id $From 'Z'
}

# Types $Chord, which opens a tab whose program runs the command line of
# Probe, and shows it, while the tabs of $Known are hidden. Returns the
# number of its program once it has got Z after the chord: then the window
# has taken all its keys.
function Open-Tab([string]$Chord, [int[]]$Known) {
    Send-Chord $Chord
    $id = Wait-NewShell $Known $Chord
    Wait-Shown $id $Known "After $Chord"
    $null = Get-KeysBefore $id 0
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
$window = [IntPtr]::Zero
$thread = [uint32]0
$layout = [IntPtr]::Zero
$dpi = 96
$grid = $null
$cell = $null
$barHeight = 0
$pointer = $null
$attached = $false
$onNew = $false
$marks = 0
$passed = $false
$saved = Get-Text
try {
    $process = [Diagnostics.Process]::Start($info)
    $started = $process.StartTime

    # 1. The first tab runs Probe, the default profile, in its folder; and
    # Ctrl+Shift+T opens two more, so that the third one is shown.
    Wait-For { $api::WindowOf($process.Id, 'meiga') -ne [IntPtr]::Zero } { "$exe did not open its window." }
    $window = $api::WindowOf($process.Id, 'meiga')
    $processId = [uint32]0
    $thread = $api::GetWindowThreadProcessId($window, [ref]$processId)
    $layout = $api::GetKeyboardLayout($thread)
    $dpi = $api::GetDpiForWindow($window)
    $first = Wait-NewShell @() 'The first tab'
    Wait-Shown $first @() 'With one tab'
    Assert-Folder $first $work 'The first tab'
    Measure-Cells $first
    Join-Input
    Move-Pointer
    $second = Open-Tab 'Ctrl+Shift+T' @($first)
    $third = Open-Tab 'Ctrl+Shift+T' @($first, $second)
    Write-Host "The grid has $($grid.Columns) x $($grid.Rows) cells of $($cell.Width) x $($cell.Height) pixels."

    # 2. The program of the first tab, which is hidden, ends with exit code
    # 0: its tab closes, and only the bar is drawn.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Stop-Probe $first 0
    Wait-Gone $first 'The program of the first tab'
    Assert-Inside $start (Get-Bar 2).Rect 'When the program of a hidden tab ended with exit code 0'
    Wait-Shown $third @($second) 'After the first tab closed'
    Write-Host 'OK: when the program of a hidden tab ends with exit code 0, its tab closes, and only the bar is drawn.'

    # 3. The program of the tab that is shown, the last one, ends with exit
    # code 0: its tab closes, and the one on its left is shown.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Stop-Probe $third 0
    Wait-Gone $third 'The program of the tab that was shown'
    Wait-OneFull $start 'The end of the program of the tab that was shown, with exit code 0,'
    Wait-Shown $second @() 'After the tab that was shown closed'
    Write-Host 'OK: when the program of the tab that is shown ends with exit code 0, its tab closes, and the one on its left is shown, with one full frame.'

    # 4. The program of the last tab writes bye, and ends with exit code 3:
    # the tab stays, and tells how it ended, in a row of its own, and below
    # it what can be done then.
    Stop-Probe $second 3 'bye'
    Wait-Gone $second 'The program of the last tab'
    $code3 = '[process exited with code 3 (0x00000003)]'
    $what = 'After the program of the last tab wrote bye and ended with exit code 3'
    $null = Assert-Line 0 "probe-$second" -What $what
    $null = Assert-Line 1 'bye' -What $what
    $null = Assert-Line 2 $code3 -What $what
    $null = Assert-Line 3 $hint -What $what
    if (@(Get-Shells).Count -ne 0) {
        throw "$what, $exe has $(@(Get-Shells).Count) programs instead of none."
    }
    Write-Host "OK: when the program of the last tab ends with exit code 3 after writing 'bye', the tab stays, with '$code3' in the row below 'bye', and '$hint' below it; meiga goes on."

    # 5. Without a program, the keys that are not shortcuts do nothing, and
    # draw nothing: the hint stays selected, after its triple click. Neither
    # Ctrl+Enter nor AltGr+D starts the program again, nor closes the tab.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    $plain = @(foreach ($key in 'A', 'Spacebar', 'DownArrow', 'Tab', 'Backspace', 'F5', 'D') { (Get-Chord $key).Messages })
    Send-Synced $plain @() 1 'A, Space, Down, Tab, Backspace, F5 and D'
    foreach ($chord in 'Ctrl+Enter', 'AltGr+D') {
        $typed = Get-Chord $chord
        Send-Synced $typed.Messages $typed.Down 1 $chord
    }
    Assert-Inside $start (Get-Bar 1).Rect 'The keys that are not shortcuts, without a program,'
    if (@(Get-Shells).Count -ne 0) {
        throw "Without a program, the keys that are not shortcuts started $(@(Get-Shells).Count) programs."
    }
    Write-Host 'OK: then the keys that are not shortcuts do nothing: only the button of the bar under the pointer is drawn, and the selection stays; Ctrl+Enter and AltGr+D neither start the program again nor close the tab.'

    # 6. Enter, which repeats once, starts the program again in the same tab
    # and shows it, and the new program gets neither the repeat nor Enter
    # going up. What the last program wrote went to the history, where
    # Ctrl+Shift+Home shows it.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Chord 'Enter' 1
    $again = Wait-NewShell @() 'Enter, without a program,'
    Wait-Shown $again @() 'After Enter'
    $keys = Get-KeysBefore $again 0
    if ($keys) {
        throw "Enter started the program again, which got the keys '$keys' before Z, instead of none: the repeat of Enter, or Enter going up."
    }
    Assert-Folder $again $work 'After Enter'
    Wait-OneFull $start 'Enter, which started the program again,'
    $null = Assert-Line 0 "probe-$again" -What 'Once Enter started the program again'
    Wait-Quiet 400
    $from = @(Get-Log).Count
    Send-Chord 'Ctrl+Shift+Home'
    Wait-For { @(Get-Frames $from).Count -gt 0 } { 'Ctrl+Shift+Home drew nothing: the view did not move to the history.' }
    Wait-Quiet 400
    $what = 'In the history, once Enter started the program again'
    $null = Assert-Line 0 "probe-$second" -What $what
    $null = Assert-Line 2 $code3 -What $what
    $from = @(Get-Log).Count
    Send-Chord 'Ctrl+Shift+End'
    Wait-For { @(Get-Frames $from).Count -gt 0 } { 'Ctrl+Shift+End drew nothing: the view did not move back.' }
    Wait-Quiet 400
    Write-Host 'OK: Enter starts the program again in its tab, with the same command line and folder, and shows it with one full frame, and the new program gets neither the repeat of Enter nor Enter going up; what the last program wrote goes to the history.'

    # 7. While the program of a tab runs, Restart tab is disabled in its
    # menu: Shift+F10 opens it with Duplicate tab highlighted, and Down goes
    # to Close tab, past Restart tab and the separator.
    $fourth = Open-Tab 'Ctrl+Shift+T' @($again)
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Chord 'Shift+F10'
    Wait-For { @(Get-Frames $start).Count -gt 0 } { 'Shift+F10 did not open the menu of the tab.' }
    Wait-Quiet 400
    Send-Keys (@((Get-Chord 'DownArrow').Messages) + @((Get-Chord 'Enter').Messages))
    Wait-Gone $fourth 'After Shift+F10, Down and Enter, on the menu of a tab whose program runs, the program'
    Wait-Shown $again @() 'After Close tab'
    Write-Host 'OK: while the program of a tab runs, Restart tab is disabled in the menu of the tab: Shift+F10 and Down highlight Close tab, which Enter chooses.'

    # 8. The program ends with exit code 1 at the start of a row, below the
    # line of its number: its line goes in that row.
    Stop-Probe $again 1
    Wait-Gone $again 'The program that Enter started'
    $code1 = '[process exited with code 1 (0x00000001)]'
    $what = 'After the program ended with exit code 1 at the start of a row'
    $null = Assert-Line 1 $code1 -What $what
    $null = Assert-Line 2 $hint -What $what
    Write-Host "OK: when the program ends with exit code 1 at the start of a row, '$code1' goes in that row."

    # 9. The first tab, whose program ended, is shown again, and the program
    # of the second one, which is hidden then, ends with exit code 4: its tab
    # stays, and nothing is drawn. On the menu that the right button opens on
    # it, nothing is highlighted: Down goes to Duplicate tab, and Down again
    # to Restart tab, which starts the program again in the tab, without
    # showing it. Nothing wakes the window up for the new program: its
    # screen does not, since the window does not copy the screen of a hidden
    # tab, and the end of the last program already woke it up for it; nor
    # does a title, which the script does not give. So its reader has the
    # lower priority of a hidden tab from the start, and not after a wake-up.
    $fifth = Open-Tab 'Ctrl+Shift+T' @()
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Chord 'Ctrl+1'
    Wait-Shown 0 @($fifth) 'After Ctrl+1'
    Wait-OneFull $start 'Ctrl+1, which showed the first tab,'
    $start = @(Get-Log).Count
    Stop-Probe $fifth 4
    Wait-Gone $fifth 'The program of the hidden tab'
    Wait-Quiet 400
    $frames = @(Get-Frames $start)
    if ($frames.Count -gt 0) {
        Show-Lines $frames
        throw "When the program of a hidden tab ended with exit code 4, the window drew $($frames.Count) frames instead of none."
    }
    $point = Get-TabPoint (Get-Bar 2).Tabs[1]
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $point
        Get-Mouse $WM_RBUTTONDOWN $MK_RBUTTON $point
        Get-Mouse $WM_RBUTTONUP 0 $point
    )
    Wait-For { @(Get-Frames $start).Count -gt 0 } { 'The right button did not open the menu of the tab.' }
    Wait-Quiet 400
    Send-Keys (@((Get-Chord 'DownArrow').Messages) + @((Get-Chord 'DownArrow').Messages) + @((Get-Chord 'Enter').Messages))
    $behind = Wait-NewShell @() 'Down, Down and Enter, on the menu of a hidden tab whose program ended,'
    Wait-Shown 0 @($behind) 'After Restart tab, on a hidden tab'
    Assert-Folder $behind $work 'After Restart tab, on a hidden tab'
    Wait-Quiet 400
    $full = @(Get-Frames $start | Where-Object { Test-Full $_ })
    if ($full.Count -gt 0) {
        Show-Lines (Get-Frames $start)
        throw "Restart tab, on a hidden tab, made $($full.Count) full frames: it showed the tab."
    }
    Write-Host 'OK: when the program of a hidden tab ends with exit code 4, its tab stays, and nothing is drawn; on the menu that the right button opens on it, Down and Down highlight Restart tab, which Enter chooses: the program starts again in the tab, in its folder, with the lower priority of a hidden tab, and the tab is not shown.'

    # 10. Ctrl+D closes the tab that is shown, whose program ended, and
    # shows the next one, whose program gets neither D nor Ctrl going up:
    # meiga took them when they went down.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    $from = (Get-KeyText $behind).Length
    Send-Chord 'Ctrl+D'
    Wait-Shown $behind @() 'After Ctrl+D'
    Wait-OneFull $start 'Ctrl+D, which closed the tab whose program ended,'
    $keys = Get-KeysBefore $behind $from
    if ($keys) {
        throw "Ctrl+D closed the tab whose program ended, and the program of the next tab got the keys '$keys' before Z, instead of none."
    }
    if (@(Get-Shells).Count -ne 1) {
        throw "After Ctrl+D, $exe has $(@(Get-Shells).Count) programs instead of one."
    }
    Write-Host 'OK: Ctrl+D closes the tab that is shown, whose program ended, and shows the next one, with one full frame; its program gets neither D nor Ctrl going up.'

    # 11. Ctrl+Shift+2 opens Later, whose folder is not there: its tab
    # tells why its program could not start, and below it what can be done
    # then. Enter tries again, and once the folder is there, starts it.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Chord 'Ctrl+Shift+2'
    Wait-OneFull $start 'Ctrl+Shift+2, which opened Later,'
    if (@(Get-Shells).Count -ne 1) {
        throw "Ctrl+Shift+2 opened Later, whose folder is not there, and $exe has $(@(Get-Shells).Count) programs instead of one."
    }
    # The rest is the error of Windows, in its language.
    $failure = Assert-Line 0 ('[could not start `' + $probeLine + '` in `' + $later + '`: ') ']' 'The tab of Later'
    # The line wraps into the rows that it needs, and the hint follows it.
    $hintRow = [int][Math]::Ceiling($failure.Length / $grid.Columns)
    $null = Assert-Line $hintRow $hint -What 'The tab of Later'
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Chord 'Enter'
    Wait-OneFull $start 'Enter, which tried to start Later again,'
    if (@(Get-Shells).Count -ne 1) {
        throw "Enter tried to start Later again, whose folder is not there, and $exe has $(@(Get-Shells).Count) programs instead of one."
    }
    $null = Assert-Line 0 $failure -What 'Once Enter tried to start Later again'
    $null = New-Item -ItemType Directory -Path $later
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Chord 'Enter'
    $sixth = Wait-NewShell @($behind) 'Enter, once the folder of Later was there,'
    Wait-Shown $sixth @($behind) 'After Enter started Later'
    $keys = Get-KeysBefore $sixth 0
    if ($keys) {
        throw "Enter started Later, whose program got the keys '$keys' before Z, instead of none."
    }
    Assert-Folder $sixth $later 'Later'
    Wait-OneFull $start 'Enter, which started Later,'
    Write-Host 'OK: Ctrl+Shift+2 opens Later, whose folder is not there, and its tab tells why its program could not start, and what can be done then; Enter tries again, with one full frame, and once the folder is there, starts the program in it.'

    # 12. Ctrl+V, with two lines in the clipboard, opens the message box that
    # asks before pasting. While it is open, the programs of both tabs end
    # with exit code 0, and the window does not get their wake-ups: once it
    # closes, both tabs close, the hidden one and the one that is shown, and
    # the last one closes the window.
    Set-Text "echo un`r`necho dous"
    Wait-Quiet 400
    Send-Chord 'Ctrl+V'
    $dialog = Wait-Dialog 'Ctrl+V, with two lines in the clipboard,'
    Stop-Probe $behind 0
    Stop-Probe $sixth 0
    Wait-Gone $behind 'While the message box was open, the program of the hidden tab'
    Wait-Gone $sixth 'While the message box was open, the program of the tab that was shown'
    if (-not $api::PostMessageW($dialog, $WM_COMMAND, [UIntPtr]::new([uint64]$IDCANCEL), [IntPtr]::Zero)) {
        throw "Could not press Cancel on the message box: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after the message box closed, once the programs of its two tabs had ended with exit code 0."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Wait-ConsolesEnded
    Write-Host 'OK: while a message box is open, the programs of a hidden tab and of the tab that is shown end with exit code 0, and once it closes, both tabs close: the last one closes the window, which ends with exit code 0, and the consoles end too.'

    # 13. The program of the command line of meiga ends with exit code 0:
    # its tab, the only one, closes, and the window too.
    Split-Input
    $process.Dispose()
    $process = $null
    $log = Join-Path $work 'frames-command.csv'
    $info = [Diagnostics.ProcessStartInfo]::new($Path)
    $info.UseShellExecute = $false
    $info.WorkingDirectory = $work
    $info.Environment['APPDATA'] = $appdata
    foreach ($argument in '--trace-frames', $log, 'cmd', '/c', 'exit', '0') {
        $info.ArgumentList.Add($argument)
    }
    $process = [Diagnostics.Process]::Start($info)
    $started = $process.StartTime
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after it started cmd /c exit 0."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0, once cmd /c exit 0 ended."
    }
    if (@(Get-Log | Where-Object kind -In 'compose', 'paint', 'frame').Count -eq 0) {
        throw "$exe ended with exit code 0 without drawing its window."
    }
    Wait-ConsolesEnded
    Write-Host 'OK: when the program of its command line ends with exit code 0, meiga.exe ends with exit code 0 too, and its console ends.'
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
    # The text that the clipboard had, or nothing.
    for ($try = 0; $try -lt 20; $try++) {
        $restored = if ($null -ne $saved) {
            try { Set-Clipboard -Value $saved; (Get-Text) -ceq $saved } catch { $false }
        }
        else {
            $api::ClearClipboard()
        }
        if ($restored) { break }
        Start-Sleep -Milliseconds 50
    }
    if ($passed) {
        Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $work) {
        Write-Host "Kept $work"
    }
}
