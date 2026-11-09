<#
.SYNOPSIS
Checks the mouse of meiga.exe: the selection, the clipboard, and the mouse
of the programs in its terminal.

.DESCRIPTION
Starts meiga.exe three times, each time with a PowerShell script in its
terminal, and posts mouse and key messages to its window, as Windows does
with the ones of the user. The window does not need the focus.

1. Text. The script writes known text, and writes to a file each key that
   it reads. The check selects text with one, two and three clicks and by
   dragging, and checks what reaches the clipboard. It checks Ctrl+C,
   Ctrl+Shift+C and Esc with a selection and without one; the right click,
   which pastes; Ctrl+V and Ctrl+Shift+V with more than one line, which ask
   first in a message box; and the text that a program copies to the
   clipboard (OSC 52), which only changes it while the window has the
   focus.
2. Mouse events. The script asks the console for the mouse, as the programs
   for Windows do, and writes the mouse events that it reads. The check
   moves the mouse, clicks, turns the wheel and drags; and drags with Shift,
   which selects text instead.
3. Terminal input. The script reads its input as a terminal sends it, as
   the programs of WSL do, and asks for the mouse in SGR. The check clicks;
   the script asks for the format of the first xterm, and the check clicks
   again; the script turns the alternate screen on, without the mouse, and
   the check turns the wheel, which sends arrows; and the script asks for
   bracketed paste, and the check pastes more than one line.

Each time, it checks that meiga.exe ends with exit code 0 when its window
closes.

Windows also sends the window mouse moves of its own, at the place of the
pointer: for example, when the window takes the mouse while a button is
down. So the messages of each click, drag or key go at once, while the
thread of the window is suspended, and, if the pointer is over the
terminal, the check moves it to the title bar of the window, and puts it
back at the end.

The check uses the clipboard: if it had text, the check puts it back at the
end, but anything else, such as an image, is lost. Do not use the mouse nor
the keyboard while it runs. It needs PowerShell 7 (pwsh) in the PATH, for
the scripts in the terminal.

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

if (-not ('Meiga.Tools.MouseCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name MouseCheck -MemberDefinition @'
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

[DllImport("user32.dll", CharSet = CharSet.Unicode)]
public static extern IntPtr FindWindowW(string className, string title);

[DllImport("user32.dll")]
public static extern IntPtr GetWindow(IntPtr window, uint command);

[DllImport("user32.dll")]
public static extern IntPtr GetForegroundWindow();

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
$api = [Meiga.Tools.MouseCheck]

$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_COMMAND = 0x0111
$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$WM_RBUTTONDOWN = 0x0204
$WM_RBUTTONUP = 0x0205
$WM_MOUSEWHEEL = 0x020A
$MK_LBUTTON = 0x0001
$MK_RBUTTON = 0x0002
$MK_SHIFT = 0x0004
$WHEEL_DELTA = 120
$WHEEL_PAGESCROLL = [uint32]::MaxValue
$SPI_GETWHEELSCROLLLINES = 0x0068
$MAPVK_VK_TO_VSC = 0
$GW_OWNER = 4
$IDOK = 1
$IDCANCEL = 2

# The keys that each modifier sets down: Shift and the left Shift, and Ctrl
# and the left Ctrl.
$modifierKeys = @{
    Shift = 0x10, 0xA0
    Ctrl = 0x11, 0xA2
}

# The script that runs in the terminal, in one of the three phases. It
# writes to the log the size of the terminal and what it reads, and sets the
# title of the window when it is ready: the title goes after what it wrote
# before, so, when the window shows it, meiga has read the rest.
$program = @'
param(
    [Parameter(Mandatory)][string]$Log,
    [Parameter(Mandatory)][ValidateSet('Text', 'Events', 'Terminal')][string]$Phase
)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
Add-Type -Namespace MeigaCheck -Name Console -MemberDefinition @"
[DllImport("kernel32.dll")] public static extern System.IntPtr GetStdHandle(int handle);
[DllImport("kernel32.dll")] public static extern bool SetConsoleMode(System.IntPtr handle, uint mode);
[DllImport("kernel32.dll")] public static extern bool ReadConsoleInputW(System.IntPtr handle, [System.Runtime.InteropServices.Out] byte[] records, uint count, out uint read);
"@
function Write-Log([string]$Line) { [IO.File]::AppendAllText($Log, "$Line`n") }
function Set-Title([string]$Title) { [Console]::Write("`e]0;$Title`a") }
$in = [MeigaCheck.Console]::GetStdHandle(-10)
# Up to 64 INPUT_RECORD of 20 bytes: the type of the event in the first two,
# and the event from byte 4.
$records = [byte[]]::new(20 * 64)
# The offsets of the records that the console gives.
function Read-Records {
    $read = 0
    $null = [MeigaCheck.Console]::ReadConsoleInputW($in, $records, 64, [ref]$read)
    for ($at = 0; $at -lt 20 * $read; $at += 20) { $at }
}
Write-Log "size $([Console]::WindowWidth) $([Console]::WindowHeight)"
switch ($Phase) {
    'Text' {
        # Ctrl+C is a key, as in PSReadLine.
        [Console]::TreatControlCAsInput = $true
        [Console]::Write("ola mundo C:\x\f.rs:12:5 (ok, si)`r`n")
        [Console]::Write("`u{E1}`u{E9}`u{ED}`u{F3}`u{FA} `u{F1} `u{20AC} `u{4E2D}`u{6587} fin`r`n")
        Set-Title 'meiga-mouse-text'
        while ($true) {
            $key = [Console]::ReadKey($true)
            Write-Log ('U+{0:X4} {1} {2}' -f [int]$key.KeyChar, $key.Key, [int]$key.Modifiers)
            # Q: the program copies text to the clipboard (OSC 52).
            if ($key.Key -eq 'Q') {
                $text = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("OSC 52 `u{2713}"))
                [Console]::Write("`e]52;c;$text`a")
                Set-Title 'meiga-mouse-osc52'
            }
        }
    }
    'Events' {
        [Console]::Write('meiga')
        # The mouse, without the quick edit of the console:
        # ENABLE_MOUSE_INPUT and ENABLE_EXTENDED_FLAGS.
        $null = [MeigaCheck.Console]::SetConsoleMode($in, 0x90)
        Set-Title 'meiga-mouse-events'
        while ($true) {
            foreach ($at in Read-Records) {
                # MOUSE_EVENT: the cell, the buttons, the direction of the
                # wheel and the flags.
                if ([BitConverter]::ToUInt16($records, $at) -eq 2) {
                    Write-Log ('mouse {0} {1} 0x{2:X} {3} 0x{4:X}' -f [BitConverter]::ToInt16($records, $at + 4),
                        [BitConverter]::ToInt16($records, $at + 6), [BitConverter]::ToUInt16($records, $at + 8),
                        [Math]::Sign([BitConverter]::ToInt16($records, $at + 10)), [BitConverter]::ToUInt32($records, $at + 16))
                }
            }
        }
    }
    'Terminal' {
        # The input as a terminal sends it, the mouse too:
        # ENABLE_VIRTUAL_TERMINAL_INPUT and ENABLE_EXTENDED_FLAGS. Then, the
        # mouse in SGR.
        $null = [MeigaCheck.Console]::SetConsoleMode($in, 0x280)
        [Console]::Write("`e[?1000;1006h")
        Set-Title 'meiga-mouse-sgr'
        while ($true) {
            # The characters of the keys that go down.
            $units = @(foreach ($at in Read-Records) {
                    if ([BitConverter]::ToUInt16($records, $at) -eq 1 -and [BitConverter]::ToInt32($records, $at + 4) -ne 0) {
                        $unit = [BitConverter]::ToUInt16($records, $at + 14)
                        if ($unit -ne 0) { $unit }
                    }
                })
            if ($units.Count -eq 0) { continue }
            Write-Log ('in ' + (($units | ForEach-Object { '{0:X2}' -f $_ }) -join ' '))
            switch (-join [char[]]$units) {
                # X: the mouse in the format of the first xterm.
                'x' { [Console]::Write("`e[?1006l"); Set-Title 'meiga-mouse-x10' }
                # A: the alternate screen, without the mouse.
                'a' { [Console]::Write("`e[?1000l`e[?1049h"); Set-Title 'meiga-mouse-alternate' }
                # B: bracketed paste.
                'b' { [Console]::Write("`e[?2004h"); Set-Title 'meiga-mouse-bracketed' }
            }
        }
    }
}
'@

$name = "meiga-check-mouse-$PID"
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
        if ([DateTime]::UtcNow -gt $deadline) { throw (& $Failure) }
        Start-Sleep -Milliseconds 20
    }
}

function Wait-Title([string]$Title) {
    Wait-For { $process.Refresh(); $process.MainWindowTitle -eq $Title } `
    { "The title of the window of $exe did not become '$Title': it is '$($process.MainWindowTitle)'." }
}

# Waits until the last thing that the script read is like $Pattern.
function Wait-Last([string]$Pattern, [string]$What) {
    Wait-For { $lines = @(Get-Lines); $lines.Count -gt 0 -and $lines[-1] -like $Pattern } `
    { "$What`: the program read $(@(Get-Lines | Select-Object -Last 1)) last." }
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

function Wait-Clipboard([string]$Expected, [string]$What) {
    Wait-For { (Get-Text) -ceq $Expected } `
    { "$What`: the clipboard has '$(Show-Text (Get-Text))' instead of '$(Show-Text $Expected)'." }
}

# Text with its control characters in sight.
function Show-Text([string]$Text) {
    $Text -replace "`e", '\e' -replace "`r", '\r' -replace "`n", '\n'
}

# The size of the grid, which the script wrote first, and of its cells: the
# grid fills the client area of the window but for less than a cell.
function Measure-Cells {
    $size = @(Get-Lines)[0] -split ' '
    $rect = [Meiga.Tools.MouseCheck+RECT]::new()
    if (-not $api::GetClientRect($window, [ref]$rect)) {
        throw "Could not read the size of the window of $exe."
    }
    $script:grid = [pscustomobject]@{ Columns = [int]$size[1]; Rows = [int]$size[2] }
    $script:cell = [pscustomobject]@{
        Width = [Math]::Floor($rect.Right / $grid.Columns)
        Height = [Math]::Floor($rect.Bottom / $grid.Rows)
    }
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

function Send-Move([int]$Column, [int]$Row) {
    Send-Messages (Get-Mouse $WM_MOUSEMOVE 0 (Get-X $Column) (Get-Y $Row))
}

# Clicks a cell $Times times, with the left or the right button and the
# modifier keys of $Keys.
function Send-Click([int]$Column, [int]$Row, [string]$Button = 'Left', [int]$Times = 1, [int]$Keys = 0) {
    $down, $up, $flag = if ($Button -eq 'Left') {
        $WM_LBUTTONDOWN, $WM_LBUTTONUP, $MK_LBUTTON
    }
    else {
        $WM_RBUTTONDOWN, $WM_RBUTTONUP, $MK_RBUTTON
    }
    $x = Get-X $Column
    $y = Get-Y $Row
    Send-Messages @(for ($i = 0; $i -lt $Times; $i++) {
            Get-Mouse $down ($Keys -bor $flag) $x $y
            Get-Mouse $up $Keys $x $y
        })
}

# Presses the left button near the left edge of a cell, moves near the right
# edge of another one, and lets the button go.
function Send-Drag([int]$FromColumn, [int]$FromRow, [int]$ToColumn, [int]$ToRow, [int]$Keys = 0) {
    $x = Get-X $ToColumn 'Right'
    $y = Get-Y $ToRow
    Send-Messages @(
        Get-Mouse $WM_LBUTTONDOWN ($Keys -bor $MK_LBUTTON) (Get-X $FromColumn 'Left') (Get-Y $FromRow)
        Get-Mouse $WM_MOUSEMOVE ($Keys -bor $MK_LBUTTON) $x $y
        Get-Mouse $WM_LBUTTONUP $Keys $x $y
    )
}

# Turns the wheel over a cell, forward with a positive number of notches.
# Its message gives the place on the screen.
function Send-Wheel([int]$Column, [int]$Row, [int]$Notches) {
    $point = [Meiga.Tools.MouseCheck+POINT]::new()
    $point.X = Get-X $Column
    $point.Y = Get-Y $Row
    $null = $api::ClientToScreen($window, [ref]$point)
    $turn = [long](($Notches * $WHEEL_DELTA) -band 0xFFFF) -shl 16
    Send-Messages @($WM_MOUSEWHEEL, $turn, (($point.X -band 0xFFFF) -bor (($point.Y -band 0xFFFF) -shl 16)))
}

# Types a key, such as "Ctrl+Shift+V": sets its modifier keys down, and posts
# the messages of the key going down and up. With -Keep, the modifier keys
# stay as they are. The key state is the one of the thread of the window, so
# a key must reach it before the next one changes the state: see
# Connect-Keyboard.
function Send-Key([string]$Key, [switch]$Keep) {
    $parts = $Key -split '\+'
    $code = [int][ConsoleKey]$parts[-1]
    if (-not $Keep) {
        $state = [byte[]]::new(256)
        foreach ($modifier in @($parts | Select-Object -SkipLast 1)) {
            foreach ($down in $modifierKeys[$modifier]) { $state[$down] = 0x80 }
        }
        if (-not $api::SetKeyboardState($state)) {
            throw "SetKeyboardState failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
        }
    }
    $scan = $api::MapVirtualKeyExW($code, $MAPVK_VK_TO_VSC, $api::GetKeyboardLayout($thread))
    [long]$bits = 1 -bor ($scan -shl 16)
    # Going up: the key was down (bit 30), and goes up (bit 31).
    Send-Messages @($WM_KEYDOWN, $code, $bits, $WM_KEYUP, $code, ($bits -bor 0xC0000000L))
}

# Does $Action, which types or pastes in the script, and then types Z with
# the modifier keys that are down: the keys reach the script in order, so,
# when it reads the Z, it has read all that $Action sent. Returns the lines
# that it wrote before the Z.
function Read-Keys([scriptblock]$Action) {
    $start = @(Get-Lines).Count
    & $Action
    Send-Key 'Z' -Keep
    $z = '^U\+[0-9A-F]{4} Z \d+$'
    Wait-For { @(Get-Lines | Select-Object -Skip $start) -match $z } `
    { "The program did not read the Z that the check typed. It read: $(@(Get-Lines | Select-Object -Skip $start) -join ' | ')." }
    $read = [Collections.Generic.List[string]]::new()
    foreach ($line in @(Get-Lines | Select-Object -Skip $start)) {
        if ($line -match $z) { break }
        $read.Add($line)
    }
    $read.ToArray()
}

# The characters of the keys of these lines of the Text phase.
function Get-Typed([string[]]$Lines) {
    -join @(foreach ($line in @($Lines)) { [char][Convert]::ToInt32($line.Substring(2, 4), 16) })
}

# What the script of the Terminal phase read from line $From, as text.
function Get-Input([int]$From) {
    -join @(foreach ($line in @(Get-Lines | Select-Object -Skip $From)) {
            foreach ($unit in @($line -split ' ' | Select-Object -Skip 1)) { [char][Convert]::ToInt32($unit, 16) }
        })
}

function Wait-Input([int]$From, [string]$Expected, [string]$What) {
    Wait-For { (Get-Input $From) -ceq $Expected } `
    { "$What`: the program read '$(Show-Text (Get-Input $From))' instead of '$(Show-Text $Expected)'." }
}

# The message box of meiga: a dialog with its title, owned by its window.
function Get-Dialog {
    $dialog = $api::FindWindowW('#32770', 'meiga')
    if ($dialog -ne [IntPtr]::Zero -and $api::GetWindow($dialog, $GW_OWNER) -eq $window) { $dialog }
    else { [IntPtr]::Zero }
}

# Waits for the message box that asks before pasting, and presses $Button.
function Close-Dialog([int]$Button) {
    Wait-For { (Get-Dialog) -ne [IntPtr]::Zero } { "$exe did not ask before pasting more than one line." }
    if (-not $api::PostMessageW((Get-Dialog), $WM_COMMAND, [UIntPtr]::new([uint64]$Button), [IntPtr]::Zero)) {
        throw "PostMessageW failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
    Wait-For { (Get-Dialog) -eq [IntPtr]::Zero } { 'The message box did not close.' }
}

# If the pointer is over the terminal, moves it to the title bar of the
# window: the moves that Windows makes at its place would reach the program.
function Move-Pointer {
    $point = [Meiga.Tools.MouseCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.MouseCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = [Meiga.Tools.MouseCheck+RECT]::new()
    if (-not $api::ScreenToClient($window, [ref]$inside) -or -not $api::GetClientRect($window, [ref]$rect) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = [Meiga.Tools.MouseCheck+POINT]::new()
    $title.X = [Math]::Floor($rect.Right / 2)
    $title.Y = -8
    $null = $api::ClientToScreen($window, [ref]$title)
    if (-not $api::SetCursorPos($title.X, $title.Y)) {
        throw "Could not move the pointer out of the window: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# Starts meiga.exe with the script of $Phase in its terminal, and waits until
# the script sets the title $Title.
function Start-Meiga([string]$Phase, [string]$Title) {
    Remove-Item -LiteralPath $log -ErrorAction SilentlyContinue
    $info = [Diagnostics.ProcessStartInfo]::new($Path)
    $info.UseShellExecute = $false
    foreach ($argument in 'pwsh', '-NoLogo', '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
        '-File', $script, '-Log', $log, '-Phase', $Phase) {
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
    Wait-Title $Title
    Measure-Cells
}

# SetKeyboardState sets the key state of the thread that calls it: share it
# with the thread of the window. This thread needs a message queue, which
# PeekMessageW creates.
function Connect-Keyboard {
    $message = [Meiga.Tools.MouseCheck+MSG]::new()
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
$passed = $false
$saved = Get-Text
try {
    # 1. Text: the selection and the clipboard.
    Start-Meiga 'Text' 'meiga-mouse-text'
    Write-Host "The grid has $($grid.Columns) x $($grid.Rows) cells of $($cell.Width) x $($cell.Height) pixels."

    Set-Text 'meiga-check'
    $focused = $api::GetForegroundWindow() -eq $window
    Connect-Keyboard
    Send-Key 'Q'
    Wait-Title 'meiga-mouse-osc52'
    if ($focused) {
        Wait-Clipboard "OSC 52 `u{2713}" 'A program copied text to the clipboard (OSC 52)'
        Write-Host 'OK: a program copies text to the clipboard (OSC 52) while the window has the focus.'
    }
    else {
        # meiga read the text to copy before the title.
        Start-Sleep -Milliseconds 300
        if ((Get-Text) -cne 'meiga-check') {
            throw "A program copied text to the clipboard (OSC 52) while the window did not have the focus: '$(Get-Text)'."
        }
        Write-Host 'OK: a program does not copy text to the clipboard (OSC 52) while the window does not have the focus.'
    }

    Send-Click 15 0 -Times 2
    Wait-Clipboard 'C:\x\f.rs:12:5' 'A double click on a path'
    Send-Click 5 0 -Times 2
    Wait-Clipboard 'mundo' 'A double click on a word'
    Send-Click 25 0 -Times 2
    Wait-Clipboard '(ok, si)' 'A double click on a bracket'
    Write-Host 'OK: a double click selects a word, a whole path, or from a bracket to the other one, and copies it.'

    $line = "`u{E1}`u{E9}`u{ED}`u{F3}`u{FA} `u{F1} `u{20AC} `u{4E2D}`u{6587} fin"
    Send-Click 2 1 -Times 3
    Wait-Clipboard $line 'A triple click'
    Write-Host 'OK: a triple click selects a whole line, with its accents and its wide characters, and copies it.'

    Send-Drag 4 0 4 1
    Wait-Clipboard ("mundo C:\x\f.rs:12:5 (ok, si)`r`n" + $line.Substring(0, 5)) 'Dragging to the next row'
    Send-Drag 11 1 12 1
    Wait-Clipboard "`u{4E2D}`u{6587}" 'Dragging from the right half of a wide character'
    Write-Host 'OK: dragging selects from a cell to another one, across rows and with whole wide characters, and copies it.'

    Send-Drag 0 0 2 0
    Wait-Clipboard 'ola' 'Dragging in a row'
    Set-Text 'meiga-check'
    $read = @(Read-Keys {
            Send-Key 'Ctrl+C'
            Wait-Clipboard 'ola' 'Ctrl+C with a selection'
            Send-Drag 4 0 8 0
            Wait-Clipboard 'mundo' 'Dragging in a row'
            Set-Text 'meiga-check'
            Send-Key 'Ctrl+Shift+C'
            Wait-Clipboard 'mundo' 'Ctrl+Shift+C with a selection'
            Send-Key 'Ctrl+C'
        })
    if (($read -join ' | ') -cne 'U+0003 C 4') {
        throw "Ctrl+C and Ctrl+Shift+C with a selection, and Ctrl+C without one, reached the program as: $($read -join ' | ')."
    }
    Write-Host 'OK: Ctrl+C and Ctrl+Shift+C copy the selection and remove it; without one, Ctrl+C reaches the program.'

    Send-Drag 0 0 2 0
    Wait-Clipboard 'ola' 'Dragging in a row'
    $read = @(Read-Keys {
            Send-Key 'Escape'
            Send-Key 'Escape'
            Wait-Last 'U+001B *' 'Esc without a selection'
            Send-Key 'Ctrl+C'
        })
    if (($read -join ' | ') -cne 'U+001B Escape 0 | U+0003 C 4') {
        throw "Esc with a selection, Esc without one, and Ctrl+C reached the program as: $($read -join ' | ')."
    }
    Write-Host 'OK: Esc removes the selection; without one, it reaches the program.'

    Send-Click 5 0 -Times 2
    Wait-Clipboard 'mundo' 'A double click on a word'
    Set-Text "eco `e[2J`u{E1}`u{20AC}  "
    $typed = Get-Typed (Read-Keys {
            Send-Click 20 3 -Button 'Right'
            Send-Key 'Ctrl+C'
        })
    if ($typed -cne "eco [2J`u{E1}`u{20AC}`u{3}") {
        throw "A right click with a selection, and then Ctrl+C, typed '$(Show-Text $typed)' instead of 'eco [2J$("`u{E1}`u{20AC}")' and U+0003."
    }
    Write-Host 'OK: a right click removes the selection and pastes the clipboard: a line without its control characters nor its blanks at the end.'

    Set-Text "un`r`ndous"
    $typed = Get-Typed (Read-Keys {
            Send-Key 'Ctrl+V'
            Close-Dialog $IDCANCEL
            Send-Key 'Ctrl+Shift+V'
            Close-Dialog $IDOK
        })
    if ($typed -cne "un`rdous") {
        throw "Ctrl+V, cancelled, and Ctrl+Shift+V pasted '$(Show-Text $typed)' instead of 'un\rdous'."
    }
    Write-Host 'OK: Ctrl+V and Ctrl+Shift+V ask before pasting more than one line: Cancel pastes nothing, and OK pastes each line with Enter.'
    Stop-Meiga

    # 2. The mouse events of a program for Windows.
    Start-Meiga 'Events' 'meiga-mouse-events'
    Set-Text 'meiga-check'
    $start = @(Get-Lines).Count
    Send-Move 5 2
    Send-Click 7 3
    Send-Click 8 1 -Button 'Right'
    Send-Wheel 2 2 1
    Send-Wheel 2 2 -1
    # With Shift, the mouse is for meiga: dragging selects text.
    Send-Drag 0 0 4 0 -Keys $MK_SHIFT
    Send-Drag 1 1 4 1
    # mouse <column> <row> <buttons> <wheel> <flags>. The buttons: left 1 and
    # right 2. The wheel: forward 1 and back -1. The flags: MOUSE_MOVED 1 and
    # MOUSE_WHEELED 4.
    $expected = @(
        'mouse 5 2 0x0 0 0x1'
        'mouse 7 3 0x1 0 0x0', 'mouse 7 3 0x0 0 0x0'
        'mouse 8 1 0x2 0 0x0', 'mouse 8 1 0x0 0 0x0'
        'mouse 2 2 0x0 1 0x4', 'mouse 2 2 0x0 -1 0x4'
        'mouse 1 1 0x1 0 0x0', 'mouse 4 1 0x1 0 0x1', 'mouse 4 1 0x0 0 0x0'
    )
    Wait-For { @(Get-Lines).Count -ge $start + $expected.Count } `
    { "The program read $(@(Get-Lines).Count - $start) mouse events instead of $($expected.Count): $(@(Get-Lines | Select-Object -Skip $start) -join ' | ')." }
    $read = @(Get-Lines | Select-Object -Skip $start)
    if (($read -join ' | ') -cne ($expected -join ' | ')) {
        throw "The program read these mouse events: $($read -join ' | '). Expected: $($expected -join ' | ')."
    }
    Write-Host 'OK: a program for Windows gets the moves, the clicks, the wheel and the drags as mouse events.'
    Wait-Clipboard 'meiga' 'Dragging with Shift'
    Write-Host 'OK: with Shift, dragging selects text and copies it, and the program does not get it.'
    Stop-Meiga

    # 3. The input of a terminal, as the programs of WSL read it.
    Start-Meiga 'Terminal' 'meiga-mouse-sgr'
    Connect-Keyboard
    $start = @(Get-Lines).Count
    Send-Click 4 2
    Wait-Input $start "`e[<0;5;3M`e[<0;5;3m" 'A click in SGR'
    Send-Key 'X'
    Wait-Title 'meiga-mouse-x10'
    $start = @(Get-Lines).Count
    Send-Click 4 2 -Button 'Right'
    Wait-Input $start "`e[M`"%#`e[M#%#" 'A right click in the format of the first xterm'
    Write-Host 'OK: a program that reads the input of a terminal gets the clicks in SGR, and in the format of the first xterm.'

    Send-Key 'A'
    Wait-Title 'meiga-mouse-alternate'
    $lines = [uint32]0
    $null = $api::SystemParametersInfoW($SPI_GETWHEELSCROLLLINES, 0, [ref]$lines, 0)
    # A notch that scrolls a page: the rows of the screen.
    if ($lines -eq $WHEEL_PAGESCROLL) { $lines = [uint32]$grid.Rows }
    $start = @(Get-Lines).Count
    Send-Wheel 4 2 1
    Wait-Input $start ("`e[A" * $lines) "A notch of the wheel, which scrolls $lines lines"
    Write-Host "OK: on the alternate screen, a program without the mouse gets $lines arrows for each notch of the wheel."

    Send-Key 'B'
    Wait-Title 'meiga-mouse-bracketed'
    Set-Text "un`r`n`e[31mdous"
    $start = @(Get-Lines).Count
    Send-Key 'Ctrl+V'
    Wait-Input $start "`e[200~un`r[31mdous`e[201~" 'Ctrl+V with bracketed paste'
    Write-Host 'OK: with bracketed paste, Ctrl+V pastes more than one line without asking, between its marks and without its control characters.'
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
        Write-Host "What the program read is in $log"
    }
}
