<#
.SYNOPSIS
Checks the resizing, the history and the zoom of meiga.exe.

.DESCRIPTION
Starts meiga.exe with a PowerShell script in its terminal, changes the size
of its window, and posts mouse and key messages to it, as Windows does with
the ones of the user. The window does not need the focus.

1. Resizing. The script writes lines of many lengths, which wrap. The check
   makes the window narrower and shorter, wider, and narrower and taller.
   Each time, the program has to get the grid that fills the window without
   the strip of the scroll bar, and the screen of meiga has to show the
   same rows as the buffer of the console, which ConPTY reflows too: the
   check selects the whole screen, and compares the text that reaches the
   clipboard with the rows that the script reads from the console. Then it
   types a key, which the script writes where its cursor is, and compares
   again. On the alternate screen, the primary one gets the new size when
   the program goes back to it, as in the console. And if the window changes
   size while the message box that asks before pasting is open, the program
   gets the new grid when the box closes.
2. The history. The script writes 100 short lines. The check moves the view
   with the wheel; with Shift+Page Up, Ctrl+Shift+Up, Ctrl+Shift+Home and
   Ctrl+Shift+End; and with the scroll bar, dragging its thumb and clicking
   the strip under it; and checks the first row that the view shows. A key
   for the program, and pasting, bring the view back to the screen; and a
   selection that goes above the window moves the view on its own.
3. The zoom. Ctrl and +, and Ctrl and the wheel, make the cells bigger and
   the grid smaller, which the program gets; Ctrl and -, and Ctrl+0, bring
   the grid back.

At the end, it checks that meiga.exe ends with exit code 0 when its window
closes.

As in check-mouse.ps1, the messages of each action go at once, while the
thread of the window is suspended, and the pointer goes to the title bar of
the window if it is over the terminal. The check uses the clipboard: if it
had text, the check puts it back at the end, but anything else, such as an
image, is lost. Do not use the mouse nor the keyboard while it runs. It
needs PowerShell 7 (pwsh) in the PATH, for the script in the terminal.

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

if (-not ('Meiga.Tools.ResizeCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name ResizeCheck -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)]
public struct MSG { public IntPtr hwnd; public uint message; public UIntPtr wParam; public IntPtr lParam; public uint time; public int x, y; }

[StructLayout(LayoutKind.Sequential)]
public struct RECT { public int Left, Top, Right, Bottom; }

[StructLayout(LayoutKind.Sequential)]
public struct POINT { public int X, Y; }

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

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool PostMessageW(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);

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

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
public static extern IntPtr FindWindowW(string className, string title);

[DllImport("user32.dll")]
public static extern IntPtr GetWindow(IntPtr window, uint command);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ClientToScreen(IntPtr window, ref POINT point);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ScreenToClient(IntPtr window, ref POINT point);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetCursorPos(out POINT point);

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetCursorPos(int x, int y);

[DllImport("user32.dll")]
public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SystemParametersInfoW(uint action, uint parameter, out uint value, uint winIni);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool OpenClipboard(IntPtr owner);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool EmptyClipboard();

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool CloseClipboard();

[DllImport("kernel32.dll", SetLastError = true)]
public static extern IntPtr OpenThread(uint access, [MarshalAs(UnmanagedType.Bool)] bool inherit, uint thread);

[DllImport("kernel32.dll", SetLastError = true)]
public static extern uint SuspendThread(IntPtr thread);

[DllImport("kernel32.dll")]
public static extern uint ResumeThread(IntPtr thread);

[DllImport("kernel32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool CloseHandle(IntPtr handle);

// Posts messages to a window, each one as three numbers: the message, its
// wParam and its lParam. The thread of the window is suspended meanwhile,
// so it reads all of them before the input that Windows makes: a thread
// reads the posted messages first.
public static void PostAll(IntPtr window, uint thread, long[] messages)
{
    const uint THREAD_SUSPEND_RESUME = 0x0002;
    IntPtr handle = OpenThread(THREAD_SUSPEND_RESUME, false, thread);
    if (handle == IntPtr.Zero) throw new System.ComponentModel.Win32Exception();
    try
    {
        if (SuspendThread(handle) == uint.MaxValue) throw new System.ComponentModel.Win32Exception();
        try
        {
            for (int i = 0; i + 2 < messages.Length; i += 3)
            {
                if (!PostMessageW(window, (uint)messages[i], new UIntPtr((ulong)messages[i + 1]), new IntPtr(messages[i + 2])))
                    throw new System.ComponentModel.Win32Exception();
            }
        }
        finally { ResumeThread(handle); }
    }
    finally { CloseHandle(handle); }
}

// Empties the clipboard. Returns whether it could.
public static bool ClearClipboard()
{
    if (!OpenClipboard(IntPtr.Zero)) return false;
    try { return EmptyClipboard(); }
    finally { CloseClipboard(); }
}
'@
}
$api = [Meiga.Tools.ResizeCheck]

$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_COMMAND = 0x0111
$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_MOUSEWHEEL = 0x020A
$MK_LBUTTON = 0x0001
$MK_CONTROL = 0x0008
$WHEEL_DELTA = 120
$WHEEL_PAGESCROLL = [uint32]::MaxValue
$SPI_GETWHEELSCROLLLINES = 0x0068
$MAPVK_VK_TO_VSC = 0
$GW_OWNER = 4
$IDCANCEL = 2
$SWP_NOMOVE = 0x0002
$SWP_NOZORDER = 0x0004
$SWP_NOACTIVATE = 0x0010

# The keys that each modifier sets down: Shift and the left Shift, and Ctrl
# and the left Ctrl.
$modifierKeys = @{
    Shift = 0x10, 0xA0
    Ctrl = 0x11, 0xA2
}

# The script that runs in the terminal. It writes to the log the size of
# the terminal, the keys that it reads, and, when the size changes and after
# some keys, the rows of the console and where its cursor is. Its lines end
# with |, so the rows that do not end with it go on in the next one.
$program = @'
param([Parameter(Mandatory)][string]$Log)
$ErrorActionPreference = 'Stop'
# An error goes to the log, where the check finds it.
trap {
    [IO.File]::AppendAllText($Log, "error $($_.Exception.Message) (line $($_.InvocationInfo.ScriptLineNumber))`n")
    break
}
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
Add-Type -Namespace MeigaCheck -Name Console -MemberDefinition @"
[DllImport("kernel32.dll")] public static extern System.IntPtr GetStdHandle(int handle);
[DllImport("kernel32.dll")] public static extern bool SetConsoleMode(System.IntPtr handle, uint mode);
[DllImport("kernel32.dll")] public static extern bool ReadConsoleInputW(System.IntPtr handle, [System.Runtime.InteropServices.Out] byte[] records, uint count, out uint read);
[DllImport("kernel32.dll", CharSet = System.Runtime.InteropServices.CharSet.Unicode)] public static extern bool ReadConsoleOutputCharacterW(System.IntPtr handle, [System.Runtime.InteropServices.Out] char[] text, uint length, int at, out uint read);
"@
function Write-Log([string]$Line) { [IO.File]::AppendAllText($Log, "$Line`n") }
function Set-Title([string]$Title) { [Console]::Write("`e]0;$Title`a") }
$in = [MeigaCheck.Console]::GetStdHandle(-10)
$out = [MeigaCheck.Console]::GetStdHandle(-11)
# The keys and the changes of size, without the mouse, which stays for
# meiga: ENABLE_WINDOW_INPUT and ENABLE_EXTENDED_FLAGS.
$null = [MeigaCheck.Console]::SetConsoleMode($in, 0x88)
# Up to 64 INPUT_RECORD of 20 bytes: the type of the event in the first two,
# and the event from byte 4.
$records = [byte[]]::new(20 * 64)
$screens = 0
# Writes to the log the size of the screen, its rows and where the cursor
# is, and then sets a title with how many times it did.
function Write-Screen {
    $width = [Console]::WindowWidth
    $height = [Console]::WindowHeight
    $top = [Console]::WindowTop
    $text = [char[]]::new($width)
    Write-Log "size $width $height"
    for ($row = 0; $row -lt $height; $row++) {
        $read = 0
        $null = [MeigaCheck.Console]::ReadConsoleOutputCharacterW($out, $text, $width, ($top + $row) -shl 16, [ref]$read)
        Write-Log ("row $row " + [string]::new($text, 0, $read).TrimEnd(' '))
    }
    Write-Log "cursor $([Console]::CursorLeft) $([Console]::CursorTop - $top)"
    $script:screens++
    Set-Title "meiga-resize-$screens"
}
Write-Log "size $([Console]::WindowWidth) $([Console]::WindowHeight)"
Set-Title 'meiga-resize-ready'
while ($true) {
    $read = 0
    $null = [MeigaCheck.Console]::ReadConsoleInputW($in, $records, 64, [ref]$read)
    for ($at = 0; $at -lt 20 * $read; $at += 20) {
        $type = [BitConverter]::ToUInt16($records, $at)
        # WINDOW_BUFFER_SIZE_EVENT
        if ($type -eq 4) { Write-Screen }
        # KEY_EVENT, of a key that goes down: its virtual-key code.
        if ($type -ne 1 -or [BitConverter]::ToInt32($records, $at + 4) -eq 0) { continue }
        $key = [BitConverter]::ToUInt16($records, $at + 10)
        Write-Log "key $key"
        switch ($key) {
            # L: lines of many lengths, and the start of another one, where
            # the cursor stays.
            0x4C {
                $chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
                for ($line = 1; $line -le 30; $line++) {
                    $length = ($line * 37) % 150 + 6
                    $body = -join @(for ($i = 0; $i -lt $length - 4; $i++) { $chars[($i + $line) % $chars.Length] })
                    [Console]::Write(('{0:D2} {1}|' -f $line, $body) + "`r`n")
                }
                [Console]::Write('> ready|')
                Write-Screen
            }
            # W: two characters where the cursor is.
            0x57 { [Console]::Write('w|'); Write-Screen }
            # A: the alternate screen. B: back to the primary one.
            0x41 { [Console]::Write("`e[?1049h`e[Halt|"); Write-Screen }
            0x42 { [Console]::Write("`e[?1049l"); Write-Screen }
            # H: 100 short lines.
            0x48 {
                [Console]::Write("`r`n")
                for ($line = 1; $line -le 100; $line++) { [Console]::Write(('h{0:D3}|' -f $line) + "`r`n") }
                [Console]::Write('> ready|')
                Write-Screen
            }
        }
    }
}
'@

$name = "meiga-check-resize-$PID"
$log = Join-Path ([IO.Path]::GetTempPath()) "$name.txt"
$script = Join-Path ([IO.Path]::GetTempPath()) "$name.ps1"
Set-Content -LiteralPath $script -Value $program -Encoding utf8NoBOM

# The lines that the script wrote. It appends them while the check reads.
function Get-Lines {
    if (-not (Test-Path -LiteralPath $log)) { return @() }
    $stream = [IO.File]::Open($log, 'Open', 'Read', 'ReadWrite')
    try { $text = [IO.StreamReader]::new($stream).ReadToEnd() }
    finally { $stream.Dispose() }
    # Only the whole lines.
    @($text.Substring(0, $text.LastIndexOf("`n") + 1) -split "`n" | Where-Object { $_ })
}

# Waits until $Done gives true. If it does not in time, fails with the
# message that $Failure gives then.
function Wait-For([scriptblock]$Done, [scriptblock]$Failure) {
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while (-not (& $Done)) {
        if ($process -and $process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) during the check."
        }
        if ([DateTime]::UtcNow -gt $deadline) {
            $failed = @(Get-Lines | Where-Object { $_.StartsWith('error ') })
            if ($failed) { throw "The script in the terminal failed: $($failed[0].Substring(6))" }
            throw (& $Failure)
        }
        Start-Sleep -Milliseconds 20
    }
}

function Wait-Title([string]$Title) {
    Wait-For { $process.Refresh(); $process.MainWindowTitle -eq $Title } `
    { "The title of the window of $exe did not become '$Title': it is '$($process.MainWindowTitle)'." }
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

# How many times the script wrote the screen.
function Get-Screens { @(Get-Lines | Where-Object { $_.StartsWith('cursor ') }).Count }

# The last screen that the script wrote whole: its size, and its rows. Nothing
# if it did not write any.
function Read-Screen {
    $lines = @(Get-Lines)
    $start = $lines.Count - 1
    while ($start -ge 0 -and -not $lines[$start].StartsWith('cursor ')) { $start-- }
    while ($start -ge 0 -and -not $lines[$start].StartsWith('size ')) { $start-- }
    if ($start -lt 0) { return $null }
    $size = $lines[$start] -split ' '
    $rows = [int]$size[2]
    [pscustomobject]@{
        Columns = [int]$size[1]
        Rows = $rows
        Text = @(for ($row = 1; $row -le $rows; $row++) { $lines[$start + $row] -replace '^row \d+ ', '' })
    }
}

# Waits until the script writes the screen after the $Before time, and
# returns it. With $Columns and $Rows, waits for a screen of that size.
function Wait-Screen([int]$Before, [string]$What, [int]$Columns = 0, [int]$Rows = 0) {
    Wait-For {
        if ((Get-Screens) -le $Before) { return $false }
        $screen = Read-Screen
        $Columns -eq 0 -or ($screen.Columns -eq $Columns -and $screen.Rows -eq $Rows)
    } {
        $screen = Read-Screen
        if ((Get-Screens) -le $Before) { "$What`: the program did not write its screen." }
        else { "$What`: the program has a grid of $($screen.Columns) x $($screen.Rows) cells, and it has to be $Columns x $Rows." }
    }
    $script:grid = Read-Screen
    $grid
}

# The size of the cells, from the first size of the grid that the script
# wrote: the grid fills the client area of the window but for the strip of
# the scroll bar, 12 pixels at a scale of 100 %, and less than a cell.
function Measure-Cells {
    $size = @(Get-Lines)[0] -split ' '
    $client = Get-Client
    $script:strip = [Math]::Max(1, [Math]::Floor((12 * $api::GetDpiForWindow($window) + 48) / 96))
    $script:cell = [pscustomobject]@{
        Width = [Math]::Floor(($client.Right - $strip) / [int]$size[1])
        Height = [Math]::Floor($client.Bottom / [int]$size[2])
    }
}

function Get-Client {
    $rect = [Meiga.Tools.ResizeCheck+RECT]::new()
    if (-not $api::GetClientRect($window, [ref]$rect)) {
        throw "Could not read the size of the window of $exe."
    }
    $rect
}

# Changes the size of the window, so that the grid that fills it has
# $Columns x $Rows cells, and half a cell to spare.
function Resize-Window([int]$Columns, [int]$Rows) {
    $client = Get-Client
    $outer = [Meiga.Tools.ResizeCheck+RECT]::new()
    $null = $api::GetWindowRect($window, [ref]$outer)
    $width = $Columns * $cell.Width + $strip + [Math]::Floor($cell.Width / 2) + ($outer.Right - $outer.Left - $client.Right)
    $height = $Rows * $cell.Height + [Math]::Floor($cell.Height / 2) + ($outer.Bottom - $outer.Top - $client.Bottom)
    if (-not $api::SetWindowPos($window, [IntPtr]::Zero, 0, 0, $width, $height, $SWP_NOMOVE -bor $SWP_NOZORDER -bor $SWP_NOACTIVATE)) {
        throw "SetWindowPos failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
    Move-Pointer
}

# Changes the size of the window as Resize-Window, and waits until the
# program gets the grid.
function Set-Grid([int]$Columns, [int]$Rows, [string]$What) {
    $before = Get-Screens
    Resize-Window $Columns $Rows
    $null = Wait-Screen $before $What $Columns $Rows
}

# The message box of meiga: a dialog with its title, owned by its window.
function Get-Dialog {
    $dialog = $api::FindWindowW('#32770', 'meiga')
    if ($dialog -ne [IntPtr]::Zero -and $api::GetWindow($dialog, $GW_OWNER) -eq $window) { $dialog }
    else { [IntPtr]::Zero }
}

function Wait-Dialog {
    Wait-For { (Get-Dialog) -ne [IntPtr]::Zero } { "$exe did not ask before pasting more than one line." }
}

# Presses $Button in the message box, and waits until it closes.
function Close-Dialog([int]$Button) {
    if (-not $api::PostMessageW((Get-Dialog), $WM_COMMAND, [UIntPtr]::new([uint64]$Button), [IntPtr]::Zero)) {
        throw "PostMessageW failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
    Wait-For { (Get-Dialog) -eq [IntPtr]::Zero } { 'The message box did not close.' }
}

# A pixel of a cell: near its left edge, near its right edge, or in its
# middle.
function Get-X([int]$Column, [string]$Side = 'Middle') {
    switch ($Side) {
        'Left' { $Column * $cell.Width + 1 }
        'Right' { ($Column + 1) * $cell.Width - 2 }
        default { $Column * $cell.Width + [Math]::Floor($cell.Width / 2) }
    }
}
function Get-Y([int]$Row) { $Row * $cell.Height + [Math]::Floor($cell.Height / 2) }

# Posts messages to the window at once: see PostAll.
function Send-Messages([long[]]$Messages) {
    $api::PostAll($window, $thread, $Messages)
}

# A mouse message at a pixel: the message, its wParam and its lParam.
function Get-Mouse([int]$Message, [int]$Keys, [int]$X, [int]$Y) {
    $Message, $Keys, (($X -band 0xFFFF) -bor (($Y -band 0xFFFF) -shl 16))
}

# Turns the wheel over the middle of the grid, forward with a positive
# number of notches, with the modifier keys of $Keys. Its message gives the
# place on the screen.
function Send-Wheel([int]$Notches, [int]$Keys = 0) {
    $point = [Meiga.Tools.ResizeCheck+POINT]::new()
    $point.X = Get-X ([Math]::Floor($grid.Columns / 2))
    $point.Y = Get-Y ([Math]::Floor($grid.Rows / 2))
    $null = $api::ClientToScreen($window, [ref]$point)
    $turn = ([long](($Notches * $WHEEL_DELTA) -band 0xFFFF) -shl 16) -bor $Keys
    Send-Messages @($WM_MOUSEWHEEL, $turn, (($point.X -band 0xFFFF) -bor (($point.Y -band 0xFFFF) -shl 16)))
}

# Sets down the modifier keys of $Modifiers, such as "Ctrl" and "Shift", and
# no other key, in the key state that the thread shares with the window.
function Set-Modifiers([string[]]$Modifiers) {
    $state = [byte[]]::new(256)
    foreach ($modifier in @($Modifiers)) {
        foreach ($down in $modifierKeys[$modifier]) { $state[$down] = 0x80 }
    }
    if (-not $api::SetKeyboardState($state)) {
        throw "SetKeyboardState failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# The message of a key, such as "V", going down, or up with -Up: the
# message, its wParam and its lParam.
function Get-Key([string]$Key, [switch]$Up) {
    $code = [int][ConsoleKey]$Key
    $scan = $api::MapVirtualKeyExW($code, $MAPVK_VK_TO_VSC, $api::GetKeyboardLayout($thread))
    [long]$bits = 1 -bor ($scan -shl 16)
    # Going up: the key was down (bit 30), and goes up (bit 31).
    if ($Up) { $WM_KEYUP, $code, ($bits -bor 0xC0000000L) }
    else { $WM_KEYDOWN, $code, $bits }
}

# Types a key, such as "Ctrl+Shift+Home": sets its modifier keys down, and
# posts the messages of the key going down and up.
function Send-Key([string]$Key) {
    $parts = $Key -split '\+'
    Set-Modifiers @($parts | Select-Object -SkipLast 1)
    Send-Messages @((Get-Key $parts[-1]) + (Get-Key $parts[-1] -Up))
}

# Types a key for the program, and waits until the script reads it.
function Send-ProgramKey([string]$Key) {
    $count = @(Get-Lines).Count
    Send-Key $Key
    $code = [int][ConsoleKey]$Key
    Wait-For { @(Get-Lines | Select-Object -Skip $count) -contains "key $code" } `
    { "The program did not read the $Key that the check typed." }
}

# Selects from the left edge of a cell to the right edge of another one,
# and returns the text that meiga copies. The press goes to the top or to the
# bottom of the first cell, by turns: two presses at the same place, inside
# the double-click time, would be a double or a triple click.
function Copy-Cells([int]$FromColumn, [int]$FromRow, [int]$ToColumn, [int]$ToRow, [string]$What) {
    Wait-For { $api::ClearClipboard() } { 'Could not empty the clipboard.' }
    $script:presses++
    $from = if ($presses % 2) { $FromRow * $cell.Height + 2 } else { ($FromRow + 1) * $cell.Height - 3 }
    $x = Get-X $ToColumn 'Right'
    $y = Get-Y $ToRow
    Send-Messages @(
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON (Get-X $FromColumn 'Left') $from
        Get-Mouse $WM_MOUSEMOVE $MK_LBUTTON $x $y
        Get-Mouse $WM_LBUTTONUP 0 $x $y
    )
    Wait-Copied "$What`: $exe did not copy the selection."
}

# Waits until the clipboard has text, and returns it: reading it once more
# could fail, while another program opens it.
function Wait-Copied([string]$Message) {
    Wait-For { ($script:copiedText = Get-Text) } { $Message }
    $script:copiedText
}

# The text of a row of the view.
function Read-Row([int]$Row, [string]$What) {
    Copy-Cells 0 $Row ($grid.Columns - 1) $Row $What
}

# The number of the short line of a row of the view.
function Read-Number([int]$Row, [string]$What) {
    $text = Read-Row $Row $What
    if ($text -notmatch '^h(\d{3})\|$') { throw "$What`: the row $Row of the view is '$(Show-Text $text)'." }
    [int]$Matches[1]
}

function Assert-Number([int]$Expected, [string]$What) {
    $number = Read-Number 0 $What
    if ($number -ne $Expected) {
        throw "$What`: the first row of the view is the line h$('{0:D3}' -f $number), and it has to be h$('{0:D3}' -f $Expected)."
    }
}

# The text that meiga copies when the whole screen is selected, from the
# rows of the console: each row, with the end of a line after the ones that
# do not go on in the next one. alacritty_terminal leaves out the end of the
# last row, and meiga one more.
function Get-Expected([string[]]$Rows) {
    $text = -join @(foreach ($row in $Rows) {
            if ($row -and -not $row.EndsWith('|')) { $row } else { "$row`r`n" }
        })
    foreach ($end in 1, 2) {
        if ($text.EndsWith("`r`n")) { $text = $text.Substring(0, $text.Length - 2) }
    }
    $text
}

# Checks that the screen of meiga shows the rows of the last screen that the
# script wrote.
function Compare-Screen([string]$What) {
    $screen = Read-Screen
    $expected = Get-Expected $screen.Text
    $copied = Copy-Cells 0 0 ($screen.Columns - 1) ($screen.Rows - 1) $What
    if ($copied -cne $expected) {
        $mine = $copied -split "`r`n"
        $theirs = $expected -split "`r`n"
        $line = 0
        while ($line -lt $mine.Count -and $line -lt $theirs.Count -and $mine[$line] -ceq $theirs[$line]) { $line++ }
        $inMeiga = if ($line -lt $mine.Count) { "'$($mine[$line])'" } else { 'missing' }
        $inConsole = if ($line -lt $theirs.Count) { "'$($theirs[$line])'" } else { 'missing' }
        throw "$What`: the screen of $exe is not the one of the console. Their line $($line + 1) is $inMeiga in meiga and $inConsole in the console."
    }
}

# If the pointer is over the terminal, moves it to the title bar of the
# window: the moves that Windows makes at its place would reach meiga.
function Move-Pointer {
    $point = [Meiga.Tools.ResizeCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.ResizeCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = Get-Client
    if (-not $api::ScreenToClient($window, [ref]$inside) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = [Meiga.Tools.ResizeCheck+POINT]::new()
    $title.X = [Math]::Floor($rect.Right / 2)
    $title.Y = -8
    $null = $api::ClientToScreen($window, [ref]$title)
    if (-not $api::SetCursorPos($title.X, $title.Y)) {
        throw "Could not move the pointer out of the window: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# Starts meiga.exe with the script in its terminal, and waits until it is
# ready.
function Start-Meiga {
    Remove-Item -LiteralPath $log -ErrorAction SilentlyContinue
    $info = [Diagnostics.ProcessStartInfo]::new($Path)
    $info.UseShellExecute = $false
    foreach ($argument in 'pwsh', '-NoLogo', '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
        '-File', $script, '-Log', $log) {
        $info.ArgumentList.Add($argument)
    }
    $script:process = [Diagnostics.Process]::Start($info)
    $idle = try { $process.WaitForInputIdle($TimeoutSeconds * 1000) } catch { $false }
    if (-not $idle) {
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) before it opened its window."
        }
        throw "$exe did not open its window within $TimeoutSeconds s."
    }
    $process.Refresh()
    $script:window = $process.MainWindowHandle
    $processId = [uint32]0
    $script:thread = $api::GetWindowThreadProcessId($window, [ref]$processId)
    Move-Pointer
    Wait-Title 'meiga-resize-ready'
    Measure-Cells
}

# SetKeyboardState sets the key state of the thread that calls it: share it
# with the thread of the window. This thread needs a message queue, which
# PeekMessageW creates.
function Connect-Keyboard {
    $message = [Meiga.Tools.ResizeCheck+MSG]::new()
    $null = $api::PeekMessageW([ref]$message, [IntPtr]::Zero, 0, 0, 0)
    $script:attached = $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $true)
    if (-not $attached) {
        throw "AttachThreadInput failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# Closes the window, as its X button does, and checks how meiga.exe ends.
function Stop-Meiga {
    if ($attached) {
        $null = $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $false)
        $script:attached = $false
    }
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
    $process.Dispose()
    $script:process = $null
}

# As per monitor (v2), as meiga, this thread sees the pixels of the window as
# they are on the screen.
$previousDpi = $api::SetThreadDpiAwarenessContext([IntPtr]::new(-4))
$process = $null
$window = [IntPtr]::Zero
$thread = [uint32]0
$attached = $false
$pointer = $null
$grid = $null
$cell = $null
$presses = 0
$strip = 0
$passed = $false
$saved = Get-Text
try {
    Start-Meiga
    Connect-Keyboard
    Write-Host "The cells have $($cell.Width) x $($cell.Height) pixels, and the strip of the scroll bar $strip."

    # 1. Resizing.
    Set-Grid 70 20 'The first size'
    $before = Get-Screens
    Send-Key 'L'
    $null = Wait-Screen $before 'The lines'
    Compare-Screen 'The lines of many lengths'
    foreach ($size in @(45, 14), @(90, 18), @(60, 24)) {
        $what = "At $($size[0]) x $($size[1]) cells"
        Set-Grid $size[0] $size[1] $what
        Compare-Screen $what
        $before = Get-Screens
        Send-Key 'W'
        $null = Wait-Screen $before $what
        Compare-Screen "$what, after writing where the cursor is"
    }
    Write-Host 'OK: the program gets the grid that fills the window, and the screen has the rows of the console when the window gets narrower, shorter, wider and taller.'

    $before = Get-Screens
    Send-Key 'A'
    $null = Wait-Screen $before 'The alternate screen'
    Set-Grid 50 16 'On the alternate screen'
    Compare-Screen 'The alternate screen, at 50 x 16 cells'
    $before = Get-Screens
    Send-Key 'B'
    $null = Wait-Screen $before 'Back to the primary screen' 50 16
    Compare-Screen 'The primary screen, after the alternate one changed size'
    Write-Host 'OK: on the alternate screen, the primary one gets the new size when the program goes back to it.'

    # While the message box that asks before pasting more than one line is
    # open, the window changes size. Its events do not reach meiga, which
    # waits for the box: the program gets the new grid when the box closes.
    Set-Text "un`r`ndous"
    $before = Get-Screens
    Send-Key 'Ctrl+V'
    Wait-Dialog
    Resize-Window 66 15
    Close-Dialog $IDCANCEL
    $null = Wait-Screen $before 'After the message box' 66 15
    Compare-Screen 'The screen, after the window changed size while the message box was open'
    Write-Host 'OK: if the window changes size while a message box is open, the program gets the grid that fills it when the box closes.'

    # 2. The history. At the bottom, the view shows the last $rows - 1 short
    # lines, and the line of the cursor.
    Set-Grid 60 20 'Before the history'
    $before = Get-Screens
    Send-Key 'H'
    $null = Wait-Screen $before 'The short lines'
    $rows = $grid.Rows
    $last = 101 - $rows + 1
    Assert-Number $last 'At the bottom'

    $lines = [uint32]0
    $null = $api::SystemParametersInfoW($SPI_GETWHEELSCROLLLINES, 0, [ref]$lines, 0)
    if ($lines -eq $WHEEL_PAGESCROLL) { $lines = [uint32]$rows }
    Send-Wheel 1
    $first = $last - [int]$lines
    Assert-Number $first 'A notch of the wheel'
    Send-Key 'Shift+PageUp'
    $first -= $rows
    Assert-Number $first 'Shift+Page Up'
    Send-Key 'Ctrl+Shift+UpArrow'
    $first -= 1
    Assert-Number $first 'Ctrl+Shift+Up'
    Send-Key 'Ctrl+Shift+Home'
    $top = Read-Row 0 'Ctrl+Shift+Home'
    if (-not $top.StartsWith('01 ')) {
        throw "Ctrl+Shift+Home: the first row of the view is '$(Show-Text $top)' instead of the first line of the history."
    }
    Send-Key 'Ctrl+Shift+End'
    Assert-Number $last 'Ctrl+Shift+End'
    Write-Host "OK: the wheel ($lines rows), Shift+Page Up, Ctrl+Shift+Up, Ctrl+Shift+Home and Ctrl+Shift+End move the view of the history."

    # The thumb, at the bottom of the strip, up to the top; and a click on
    # the strip under it, a page down.
    $client = Get-Client
    $x = $client.Right - [Math]::Floor($strip / 2) - 1
    $bottom = $client.Bottom - [Math]::Max(1, [Math]::Floor((2 * $api::GetDpiForWindow($window) + 48) / 96)) - 2
    Send-Messages @(
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $x $bottom
        Get-Mouse $WM_MOUSEMOVE $MK_LBUTTON $x (-1000)
        Get-Mouse $WM_LBUTTONUP 0 $x (-1000)
    )
    $top = Read-Row 0 'Dragging the thumb to the top'
    if (-not $top.StartsWith('01 ')) {
        throw "Dragging the thumb to the top: the first row of the view is '$(Show-Text $top)' instead of the first line of the history."
    }
    Send-Key 'Ctrl+Shift+End'
    Send-Key 'Shift+PageUp'
    Send-Key 'Shift+PageUp'
    Assert-Number ($last - 2 * $rows) 'Two pages up'
    Send-Messages @(
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $x ($client.Bottom - 1)
        Get-Mouse $WM_LBUTTONUP 0 $x ($client.Bottom - 1)
    )
    Assert-Number ($last - $rows) 'A click on the strip under the thumb'
    Write-Host 'OK: dragging the thumb of the scroll bar moves the view, and a click on the strip under it moves it a page down.'

    # A key for the program, and pasting, bring the view back to the screen.
    Send-Key 'Shift+PageUp'
    Send-ProgramKey 'N'
    Assert-Number $last 'A key for the program'
    Set-Text 'x'
    Send-Key 'Shift+PageUp'
    Send-Key 'Ctrl+V'
    Wait-For { @(Get-Lines)[-1] -eq "key $([int][ConsoleKey]::X)" } { 'The program did not get the text that the check pasted.' }
    Assert-Number $last 'Pasting'
    Write-Host 'OK: a key for the program, and pasting, bring the view back to the screen.'

    # A selection from the third row to above the window, where the pointer
    # stays for half a second: the view moves on its own, and the selection
    # grows with it. The pointer goes there too, for the moves that Windows
    # makes at its place when the window takes the mouse.
    $above = [Meiga.Tools.ResizeCheck+POINT]::new()
    $above.X = Get-X 0 'Left'
    $above.Y = -3 * $cell.Height
    $screen = [Meiga.Tools.ResizeCheck+POINT]::new()
    $screen.X = $above.X
    $screen.Y = $above.Y
    $null = $api::ClientToScreen($window, [ref]$screen)
    if ($null -eq $pointer) {
        $current = [Meiga.Tools.ResizeCheck+POINT]::new()
        if ($api::GetCursorPos([ref]$current)) { $pointer = $current }
    }
    $null = $api::SetCursorPos($screen.X, $screen.Y)
    Set-Modifiers @()
    Send-Messages @(
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON (Get-X 3) (Get-Y 2)
        Get-Mouse $WM_MOUSEMOVE $MK_LBUTTON $above.X $above.Y
    )
    Start-Sleep -Milliseconds 500
    Wait-For { $api::ClearClipboard() } { 'Could not empty the clipboard.' }
    Send-Messages (Get-Mouse $WM_LBUTTONUP 0 $above.X $above.Y)
    $first = (Wait-Copied "A selection above the window: $exe did not copy it.") -split "`r`n" | Select-Object -First 1
    if ($first -match '^h(\d{3})\|' -and [int]$Matches[1] -ge $last - 5) {
        throw "A selection above the window: the view did not move on its own, and the selection starts with '$first'."
    }
    Send-ProgramKey 'N'
    Write-Host "OK: a selection above the window moves the view on its own, up to the line '$first'."

    # 3. The zoom.
    $columns = $grid.Columns
    $before = Get-Screens
    Send-Key 'Ctrl+OemPlus'
    $null = Wait-Screen $before 'Ctrl and +'
    if ($grid.Columns -ge $columns -or $grid.Rows -gt $rows) {
        throw "Ctrl and +: the grid has $($grid.Columns) x $($grid.Rows) cells, and it has to be smaller than $columns x $rows."
    }
    $before = Get-Screens
    Send-Key 'Ctrl+OemMinus'
    $null = Wait-Screen $before 'Ctrl and -' $columns $rows
    $before = Get-Screens
    Set-Modifiers 'Ctrl'
    Send-Wheel 1 $MK_CONTROL
    $null = Wait-Screen $before 'Ctrl and the wheel'
    if ($grid.Columns -ge $columns) {
        throw "Ctrl and the wheel: the grid has $($grid.Columns) x $($grid.Rows) cells, and it has to be smaller than $columns x $rows."
    }
    $before = Get-Screens
    Send-Key 'Ctrl+D0'
    $null = Wait-Screen $before 'Ctrl+0' $columns $rows
    Write-Host 'OK: Ctrl and +, and Ctrl and the wheel, make the cells bigger and the grid smaller, which the program gets; Ctrl and -, and Ctrl+0, bring the grid back.'

    Stop-Meiga
    $passed = $true
}
finally {
    if ($attached) {
        $null = $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $false)
    }
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
    Remove-Item -LiteralPath $script -ErrorAction SilentlyContinue
    if ($passed) {
        Remove-Item -LiteralPath $log -ErrorAction SilentlyContinue
    }
    elseif (Test-Path -LiteralPath $log) {
        Write-Host "What the program wrote is in $log"
    }
}
