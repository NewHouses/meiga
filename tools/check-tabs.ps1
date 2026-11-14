<#
.SYNOPSIS
Checks the tabs of meiga.exe: opening, showing and closing them with their
shortcuts, and what a hidden tab does and does not do.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames) and, in its first tab, a
PowerShell script that writes to its screen one step at a time, when this
check tells it to, and reports what it sees in a file. The check types the
shortcuts of the tabs in the window of meiga.exe, as check-keyboard.ps1
types its keys: Ctrl+Shift+T opens a tab with the default shell, Ctrl+Tab
and Ctrl+Shift+Tab show the next and the previous tab, and Ctrl+Shift+W
closes the tab that is shown. It checks that:

- a new tab runs the default shell, and is shown with one full frame;
- the reader thread of each tab has the normal priority while its tab is
  shown, and below normal while it is hidden, which tells which tab is
  shown;
- while a tab is hidden, the output of its program draws nothing, and its
  program still gets the answers to its questions, also when it leaves a
  synchronized update (DECSET 2026) open: the window ends it at its
  deadline;
- the title of a hidden tab, and the text that it copies to the clipboard
  (OSC 52), do not reach the window; the title does when the tab is shown,
  and the text never, while the text that the tab copies once shown does. A
  program only copies text while the window is in the foreground, as in the
  CI: when it is not, the check only shows that no tab copies any;
- a hidden tab keeps its grid while the window changes size, and gets the
  one of the window when it is shown;
- Ctrl+Tab and Ctrl+Shift+Tab go round the tabs;
- Ctrl+Shift+W closes the tab that is shown, and its programs end; the tab
  on its right is shown or, if it was the last one, the one on its left;
- closing the last tab closes the window, and meiga.exe ends with exit code
  0.

The key state of a thread changes as it reads its input, and Windows also
gives the window input of its own (see check-mouse.ps1). So the check types
each shortcut while the thread of the window is suspended: it sets the key
state and posts the keys, and once the thread runs again, it reads the
posted keys before any input.

Before the output of the hidden tab, the check waits until the window draws
nothing for a second: until the cursor of the shown tab stops blinking, if
Windows lets it blink. With the settings of Windows by default, the cursor
changes about every half a second while it blinks, and stops blinking 5 s
after the screen last changed.

It needs PowerShell 7 (pwsh) in the PATH, for the script in the terminal,
which is also the default shell of the new tabs. The new tabs load the
profile of PowerShell: one that draws something seconds after the prompt
can make the check fail. The check uses the clipboard: if it had text, the
check puts it back at the end.

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

if (-not ('Meiga.Tools.TabsCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name TabsCheck -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)]
public struct MSG { public IntPtr hwnd; public uint message; public UIntPtr wParam; public IntPtr lParam; public uint time; public int x, y; }

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
public static extern IntPtr GetForegroundWindow();

[DllImport("user32.dll")]
public static extern uint MapVirtualKeyExW(uint code, uint type, IntPtr layout);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);

[DllImport("user32.dll")]
public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

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

// Types a key in the window: sets the key state of its thread, which this
// thread shares, to `state`, and posts the messages of the key `code` going
// down and up, with the lParams `down` and `up`. The thread of the window is
// suspended meanwhile: the key state changes as the thread reads its input,
// and Windows gives the window input of its own, such as mouse moves. Once
// the thread runs again, it reads the posted messages before any input.
public static void TypeKey(IntPtr window, uint thread, byte[] state, uint code, long down, long up)
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
            // WM_KEYDOWN and WM_KEYUP
            if (!PostMessageW(window, 0x0100, new UIntPtr(code), new IntPtr(down)) ||
                !PostMessageW(window, 0x0101, new UIntPtr(code), new IntPtr(up)))
                throw new System.ComponentModel.Win32Exception();
        }
        finally { ResumeThread(handle); }
    }
    finally
    {
        Marshal.FreeHGlobal(memory);
        CloseHandle(handle);
    }
}

// Empties the clipboard. Returns whether it could.
public static bool ClearClipboard()
{
    if (!OpenClipboard(IntPtr.Zero)) return false;
    try { return EmptyClipboard(); }
    finally { CloseClipboard(); }
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
$api = [Meiga.Tools.TabsCheck]

$THREAD_PRIORITY_NORMAL = 0
$THREAD_PRIORITY_BELOW_NORMAL = -1
# The console of each tab: the one of the ConPTY of meiga, or the one of
# Windows if the ConPTY is not next to meiga.exe.
$consoles = 'OpenConsole.exe', 'conhost.exe'
$MAPVK_VK_TO_VSC = 0
# SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS
$SWP_RESIZE_ONLY = 0x0002 -bor 0x0004 -bor 0x0010 -bor 0x4000
# The keys that each modifier sets down: Shift and the left Shift, and Ctrl
# and the left Ctrl.
$modifierKeys = @{
    Shift = 0x10, 0xA0
    Ctrl = 0x11, 0xA2
}

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
function Get-Size { '{0}x{1}' -f [Console]::WindowWidth, [Console]::WindowHeight }
# The answer to a question, which comes as keys, up to $Last or for
# $Milliseconds at most, and how long it took.
function Read-Answer([char]$Last, [int]$Milliseconds) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $answer = ''
    while ($watch.ElapsedMilliseconds -lt $Milliseconds) {
        if ([Console]::KeyAvailable) {
            $char = [Console]::ReadKey($true).KeyChar
            $answer += $char
            if ($char -eq $Last) { break }
        }
        else { Start-Sleep -Milliseconds 5 }
    }
    '{0} {1}' -f ($answer -replace $e, '\e'), $watch.ElapsedMilliseconds
}
$hidden = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('meiga-tabs-hidden'))
$shown = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('meiga-tabs-shown'))
$steps = @(
    # Hide the cursor, so that it never blinks, give the tab a title, and
    # tell the process and the size.
    { $out.Write("$e[?25l$e]0;meiga-tabs-1$e\$e[2J$e[H"); "ready $PID $(Get-Size)" },
    # Output for a second.
    { foreach ($i in 1..20) { $out.Write("$e[$($i + 1);3Hline $i"); Start-Sleep -Milliseconds 50 }; 'burst' },
    # A question in a synchronized update that is never ended.
    { $out.Write("$e[?2026h$e[c"); "sync $(Read-Answer 'c' 5000)" },
    # A title, and text for the clipboard.
    { $out.Write("$e]0;meiga-tabs-hidden$e\$e]52;c;$hidden$e\"); 'title' },
    # The size, which it also writes on its screen, twice.
    { $size = Get-Size; $out.Write("$e[1;1Hsize $size"); "size $size" },
    { $size = Get-Size; $out.Write("$e[1;1Hsize $size"); "size $size" },
    # Other text for the clipboard.
    { $out.Write("$e]52;c;$shown$e\"); 'copy' }
)
foreach ($step in $steps) {
    if (-not $go.WaitOne(60000)) { exit 1 }
    [IO.File]::AppendAllText($Log, "$(& $step)`n")
    $null = $done.Set()
}
# Until the tab closes, which ends this script.
$null = $go.WaitOne()
'@

$name = "meiga-check-tabs-$PID"
$temp = [IO.Path]::GetTempPath()
$log = Join-Path $temp "$name.csv"
$reports = Join-Path $temp "$name.txt"
$script = Join-Path $temp "$name.ps1"
Set-Content -LiteralPath $script -Value $program -Encoding utf8NoBOM
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
    $Lines | Format-Table frame, time_ms, kind, cause, width, height, rects, pixels -AutoSize | Out-Host
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

function Get-Title {
    $process.Refresh()
    $process.MainWindowTitle
}

# The text of the clipboard: nothing if it has none, or if another program
# keeps it open.
function Get-Text {
    try { Get-Clipboard -Raw } catch { $null }
}

# Types a shortcut, such as "Ctrl+Shift+T": sets its modifier keys down, and
# no other key, and posts the messages of the key going down and up (see
# TypeKey).
function Send-Key([string]$Key) {
    $parts = $Key -split '\+'
    $code = [int][ConsoleKey]$parts[-1]
    $state = [byte[]]::new(256)
    foreach ($modifier in ($parts | Select-Object -SkipLast 1)) {
        foreach ($down in $modifierKeys[$modifier]) { $state[$down] = 0x80 }
    }
    $scan = $api::MapVirtualKeyExW($code, $MAPVK_VK_TO_VSC, $keyboard)
    [long]$bits = 1 -bor ($scan -shl 16)
    # Going up: the key was down (bit 30), and goes up (bit 31).
    $api::TypeKey($window, $thread, $state, $code, $bits, ($bits -bor 0xC0000000L))
}

# Opens a tab with Ctrl+Shift+T, and returns the process of its program once
# its tab is shown, with the tabs of $Hidden hidden.
function Open-Tab([int[]]$Hidden) {
    $before = @(Get-Shells | ForEach-Object ProcessId)
    Send-Key 'Ctrl+Shift+T'
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while (-not ($opened = Get-Shells | Where-Object { $_.ProcessId -notin $before } | Select-Object -First 1)) {
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) after Ctrl+Shift+T."
        }
        if ([DateTime]::UtcNow -gt $deadline) {
            throw "Ctrl+Shift+T did not open a tab with a program within $TimeoutSeconds s."
        }
        Start-Sleep -Milliseconds 20
    }
    Wait-Shown $opened.ProcessId $Hidden 'After Ctrl+Shift+T'
    $opened
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

$info = [Diagnostics.ProcessStartInfo]::new($Path)
$info.UseShellExecute = $false
foreach ($argument in '--trace-frames', $log, 'pwsh', '-NoLogo', '-NoProfile', '-NonInteractive',
    '-ExecutionPolicy', 'Bypass', '-File', $script, '-Name', $name, '-Log', $reports) {
    $info.ArgumentList.Add($argument)
}
$process = $null
$attached = $false
$saved = Get-Text
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
    $keyboard = $api::GetKeyboardLayout($thread)

    # 1. The first tab, alone.
    $ready = Invoke-Step 'start'
    $first, $firstSize = ($ready -split ' ')[1, 2]
    $first = [int]$first
    Wait-For { (Get-Title) -eq 'meiga-tabs-1' } { "The title of the window is '$(Get-Title)' instead of 'meiga-tabs-1'." }
    Wait-Shown $first @() 'With one tab'

    # SetKeyboardState sets the key state of the thread that calls it: share
    # it with the thread of the window. This thread needs a message queue,
    # which PeekMessageW creates.
    $message = [Meiga.Tools.TabsCheck+MSG]::new()
    $null = $api::PeekMessageW([ref]$message, [IntPtr]::Zero, 0, 0, 0)
    $attached = $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $true)
    if (-not $attached) {
        throw "AttachThreadInput failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }

    # 2. A second tab, with the default shell.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    $second = Open-Tab @($first)
    Wait-For { @(Get-Frames $start).Count -gt 0 } { 'The window drew nothing after Ctrl+Shift+T.' }
    $frame = (Get-Frames $start)[0]
    if (-not (Test-Full $frame)) {
        Show-Lines (Get-Frames $start)
        throw 'The first frame after Ctrl+Shift+T does not draw the whole window.'
    }
    if ((Get-Title) -eq 'meiga-tabs-1') {
        throw 'The window kept the title of the first tab after Ctrl+Shift+T.'
    }
    Write-Host "OK: Ctrl+Shift+T opens a tab with the default shell ($($second.Name)), and shows it with a full frame; the reader of the hidden tab gets a lower priority."
    $second = [int]$second.ProcessId

    # 3. Nothing is drawn for a second: the cursor of the new tab, if it
    # blinks, has stopped.
    Wait-Quiet 1000 ($TimeoutSeconds + 10)

    # 4. The hidden tab writes for a second.
    $start = @(Get-Log).Count
    $null = Invoke-Step 'write while it is hidden'
    Start-Sleep -Milliseconds 300
    $frames = @(Get-Frames $start)
    if ($frames.Count -gt 0) {
        Show-Lines $frames
        throw 'The window drew while the output of a hidden tab changed.'
    }
    Write-Host 'OK: while a tab is hidden, the output of its program draws nothing.'

    # 5. A question in a synchronized update that the program never ends.
    $start = @(Get-Log).Count
    $sync = Invoke-Step 'ask in a synchronized update while it is hidden'
    $answer, $milliseconds = ($sync -split ' ')[1, 2]
    if ($answer -ne '\e[?6c' -or [int]$milliseconds -gt 1000) {
        throw "A hidden tab that left a synchronized update open did not get the answer to its question in time: '$sync'."
    }
    Start-Sleep -Milliseconds 200
    $frames = @(Get-Frames $start)
    if ($frames.Count -gt 0) {
        Show-Lines $frames
        throw 'The window drew when it ended the synchronized update of a hidden tab.'
    }
    Write-Host "OK: a hidden tab gets the answers to its questions, also when it leaves a synchronized update open, which ends at its deadline ($milliseconds ms), without drawing."

    # 6. A title, and text for the clipboard, while it is hidden.
    $sentinel = "meiga-check-tabs-$PID"
    Wait-For { try { Set-Clipboard -Value $sentinel } catch { $null }; (Get-Text) -ceq $sentinel } `
    { "Could not put '$sentinel' in the clipboard." }
    $title = Get-Title
    $null = Invoke-Step 'write a title and copy text while it is hidden'
    Start-Sleep -Milliseconds 300
    if ((Get-Title) -ne $title) {
        throw "The title of a hidden tab reached the window: '$(Get-Title)'."
    }
    if ((Get-Text) -cne $sentinel) {
        throw "A hidden tab copied text to the clipboard: '$(Get-Text)'."
    }

    # 7. The window changes size: the hidden tab keeps its grid.
    $start = @(Get-Log).Count
    if (-not $api::SetWindowPos($window, [IntPtr]::Zero, 0, 0, 640, 480, $SWP_RESIZE_ONLY)) {
        throw 'SetWindowPos could not change the size of the window.'
    }
    Wait-For { @(Get-Log | Select-Object -Skip $start | Where-Object kind -EQ 'compose').Count -gt 0 } `
    { 'The window was not composed at its new size.' }
    Wait-Quiet 400
    $hiddenSize = ((Invoke-Step 'tell its size while it is hidden') -split ' ')[1]
    if ($hiddenSize -ne $firstSize) {
        throw "A hidden tab got another grid while the window changed size: $hiddenSize instead of $firstSize."
    }

    # 8. Ctrl+Tab: the first tab again.
    $start = @(Get-Log).Count
    Send-Key 'Ctrl+Tab'
    Wait-Shown $first @($second) 'After Ctrl+Tab'
    Wait-For { (Get-Title) -eq 'meiga-tabs-hidden' } `
    { "The window did not take the title of the tab that it shows: it is '$(Get-Title)' instead of 'meiga-tabs-hidden'." }
    Wait-Quiet 400
    $full = @(Get-Frames $start | Where-Object { Test-Full $_ })
    if ($full.Count -ne 1) {
        Show-Lines (Get-Frames $start)
        throw "Showing another tab made $($full.Count) full frames instead of 1."
    }
    # A program can only copy text while the window is in the foreground,
    # which has to stay so, or not, until the end of this step.
    $foreground = $api::GetForegroundWindow() -eq $window
    $moved = {
        if (($api::GetForegroundWindow() -eq $window) -ne $foreground) {
            ' The window went in or out of the foreground meanwhile: run the check again without using the mouse or the keyboard.'
        }
    }
    $shownSize = ((Invoke-Step 'tell its size once it is shown') -split ' ')[1]
    if ($shownSize -eq $firstSize) {
        throw "Once shown, the tab still has the grid that it had before the window changed size: $shownSize."
    }
    # What it writes now wakes the window up for it, which would take the
    # text for the clipboard, if it were still waiting.
    Start-Sleep -Milliseconds 300
    if ((Get-Text) -cne $sentinel) {
        throw "The text that a tab copied to the clipboard while it was hidden reached it once it was shown: '$(Get-Text)'."
    }
    # The text that it copies now does, if the window can take it: then the
    # window could have taken the other one too.
    $null = Invoke-Step 'copy text once it is shown'
    if ($foreground) {
        Wait-For { (Get-Text) -ceq 'meiga-tabs-shown' } `
        { "The text that a tab copied to the clipboard once it was shown did not reach it: it has '$(Get-Text)'.$(& $moved)" }
    }
    else {
        Start-Sleep -Milliseconds 300
        if ((Get-Text) -cne $sentinel) {
            throw "A tab copied text to the clipboard while the window was not in the foreground: '$(Get-Text)'.$(& $moved)"
        }
    }
    Write-Host "OK: Ctrl+Tab shows the other tab with one full frame and gives it the grid of the window ($firstSize -> $shownSize), which it did not get while it was hidden."
    if ($foreground) {
        Write-Host 'OK: the title of a hidden tab reaches the window once the tab is shown; the text that it copied to the clipboard never does, and the text that it copies once shown does.'
    }
    else {
        Write-Host 'OK: the title of a hidden tab reaches the window once the tab is shown; the window is not in the foreground, so no tab copies text to the clipboard, hidden or shown.'
    }

    # 9. A third tab, and around the tabs.
    $third = [int](Open-Tab @($first, $second)).ProcessId
    Send-Key 'Ctrl+Tab'
    Wait-Shown $first @($second, $third) 'Ctrl+Tab from the last tab'
    Send-Key 'Ctrl+Shift+Tab'
    Wait-Shown $third @($first, $second) 'Ctrl+Shift+Tab from the first tab'
    Send-Key 'Ctrl+Shift+Tab'
    Wait-Shown $second @($first, $third) 'Ctrl+Shift+Tab'
    Write-Host 'OK: Ctrl+Tab and Ctrl+Shift+Tab go round the tabs: 3 -> 1 -> 3 -> 2.'

    # 10. Closing the tabs.
    Send-Key 'Ctrl+Shift+W'
    Wait-Ended $second 'The program of the closed tab'
    Wait-Shown $third @($first) 'After closing the second tab'
    Send-Key 'Ctrl+Shift+W'
    Wait-Ended $third 'The program of the closed tab'
    Wait-Shown $first @() 'After closing the last tab of the bar'
    Wait-For { (Get-Consoles).Count -eq 1 } { "With one tab left, $exe does not have one console." }
    Write-Host 'OK: Ctrl+Shift+W closes the tab that is shown, and its programs end; then the tab on its right is shown or, after the last one, the one on its left.'

    Send-Key 'Ctrl+Shift+W'
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        throw "$exe was still running $TimeoutSeconds s after its last tab was closed."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Wait-Ended $first 'The program of the first tab'
    Write-Host 'OK: closing the last tab closes the window, and it ends with exit code 0.'
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
    $go.Dispose()
    $done.Dispose()
    Remove-Item -LiteralPath $script -ErrorAction SilentlyContinue
    if ($passed) {
        Remove-Item -LiteralPath $log, $reports -ErrorAction SilentlyContinue
    }
    else {
        foreach ($file in $log, $reports) {
            if (Test-Path -LiteralPath $file) { Write-Host "Kept $file" }
        }
    }
}
