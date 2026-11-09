<#
.SYNOPSIS
Checks that the keys typed in meiga.exe reach the program in its terminal as
PowerShell reads them.

.DESCRIPTION
Starts meiga.exe with a PowerShell script in its terminal that reads keys
with [Console]::ReadKey, as PSReadLine does, and writes each one to a file.
Then it types keys in the window of meiga.exe, which does not need the
focus: it shares the key state of the thread of the window
(AttachThreadInput), sets the modifier keys that each key needs
(SetKeyboardState), and posts the messages of the key to the window, as
Windows does with the keys that the user types. meiga.exe calls
TranslateMessage for them, which writes the characters with the keyboard
layout of its thread.

What a key writes depends on the layout. The check knows two: Spanish, with
its dead keys (´ and ¨) and AltGr, and US. For each key, it checks the
character, the key and the modifier keys that the program reads, and at the
end that meiga.exe ends with exit code 0 when its window closes.

It needs PowerShell 7 (pwsh) in the PATH, for the script in the terminal.

.PARAMETER Layout
The keyboard layout to type with: Spanish or US. Without it, the layout of
the window of meiga.exe, if it is one of them. With another layout, the
check changes the one of the window, and Windows changes it for the whole
session, unless each window can have its own (in Settings, Time & language,
Typing, Advanced keyboard settings): do not type in other windows until the
check ends, which puts it back.

.PARAMETER Path
The executable to check. Defaults to target\release\meiga.exe.

.PARAMETER TimeoutSeconds
How long to wait for the window, for each key, and for the process to end.
#>
[CmdletBinding()]
param(
    [ValidateSet('Spanish', 'US')]
    [string]$Layout,

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

if (-not ('Meiga.Tools.KeyboardCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name KeyboardCheck -MemberDefinition @'
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
public static extern int GetKeyboardLayoutList(int count, [Out] IntPtr[] layouts);

[DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
public static extern IntPtr LoadKeyboardLayoutW(string id, uint flags);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool UnloadKeyboardLayout(IntPtr layout);

[DllImport("user32.dll")]
public static extern uint MapVirtualKeyExW(uint code, uint type, IntPtr layout);
'@
}
$api = [Meiga.Tools.KeyboardCheck]

# The layouts, by the identifier of their keyboard: the high word of a
# layout handle. The Spanish layout can go with Spanish, Galician or another
# language.
$layouts = @{ Spanish = 0x040A; US = 0x0409 }
$layoutIds = @{ Spanish = '0000040A'; US = '00000409' }

# The keys that each case presses, as "Modifier+Key", with the names of
# [ConsoleKey], which are virtual-key codes, and what the program reads for
# them: U+<character> <ConsoleKey> <ConsoleModifiers>, as numbers (Alt 1,
# Shift 2, Control 4). AltGr is Ctrl and Alt: 5. A dead key goes down
# without a character, and the next key writes both.
$common = @(
    @{ Press = 'A'; Reads = 'U+0061 A 0' }
    @{ Press = 'Shift+A'; Reads = 'U+0041 A 2' }
    @{ Press = 'Ctrl+Spacebar'; Reads = 'U+0020 Spacebar 4' }
    @{ Press = 'Enter'; Reads = 'U+000D Enter 0' }
    @{ Press = 'Shift+Enter'; Reads = 'U+000D Enter 2' }
    @{ Press = 'LeftArrow'; Reads = 'U+0000 LeftArrow 0' }
    @{ Press = 'RightArrow'; Reads = 'U+0000 RightArrow 0' }
    @{ Press = 'UpArrow'; Reads = 'U+0000 UpArrow 0' }
    @{ Press = 'DownArrow'; Reads = 'U+0000 DownArrow 0' }
    @{ Press = 'Home'; Reads = 'U+0000 Home 0' }
    @{ Press = 'End'; Reads = 'U+0000 End 0' }
    @{ Press = 'Ctrl+LeftArrow'; Reads = 'U+0000 LeftArrow 4' }
    @{ Press = 'Alt+F'; Reads = 'U+0066 F 1' }
    @{ Press = 'Ctrl+C'; Reads = 'U+0003 C 4' }
    @{ Press = 'Backspace'; Reads = 'U+0008 Backspace 0' }
    @{ Press = 'Tab'; Reads = 'U+0009 Tab 0' }
    @{ Press = 'Shift+Tab'; Reads = 'U+0009 Tab 2' }
    @{ Press = 'Escape'; Reads = 'U+001B Escape 0' }
    @{ Press = 'F5'; Reads = 'U+0000 F5 0' }
    @{ Press = 'F10'; Reads = 'U+0000 F10 0' }
)
$cases = @{
    Spanish = $common + @(
        @{ Press = 'Oem7', 'A'; Reads = 'U+0000 Oem7 0', 'U+00E1 A 0' }
        @{ Press = 'Oem7', 'E'; Reads = 'U+0000 Oem7 0', 'U+00E9 E 0' }
        @{ Press = 'Oem7', 'I'; Reads = 'U+0000 Oem7 0', 'U+00ED I 0' }
        @{ Press = 'Oem7', 'O'; Reads = 'U+0000 Oem7 0', 'U+00F3 O 0' }
        @{ Press = 'Oem7', 'U'; Reads = 'U+0000 Oem7 0', 'U+00FA U 0' }
        @{ Press = 'Shift+Oem7', 'U'; Reads = 'U+0000 Oem7 2', 'U+00FC U 0' }
        @{ Press = 'Oem7', 'X'; Reads = 'U+0000 Oem7 0', 'U+00B4 X 0', 'U+0078 X 0' }
        @{ Press = 'Oem3'; Reads = 'U+00F1 Oem3 0' }
        @{ Press = 'Oem2'; Reads = 'U+00E7 Oem2 0' }
        @{ Press = 'AltGr+D2'; Reads = 'U+0040 D2 5' }
        @{ Press = 'AltGr+D3'; Reads = 'U+0023 D3 5' }
        @{ Press = 'AltGr+E'; Reads = 'U+20AC E 5' }
        @{ Press = 'AltGr+Oem1'; Reads = 'U+005B Oem1 5' }
        @{ Press = 'AltGr+OemPlus'; Reads = 'U+005D OemPlus 5' }
        @{ Press = 'AltGr+Oem7'; Reads = 'U+007B Oem7 5' }
        @{ Press = 'AltGr+Oem2'; Reads = 'U+007D Oem2 5' }
        @{ Press = 'AltGr+Oem5'; Reads = 'U+005C Oem5 5' }
        @{ Press = 'AltGr+D1'; Reads = 'U+007C D1 5' }
    )
    US = $common + @(
        @{ Press = 'Shift+D2'; Reads = 'U+0040 D2 2' }
        @{ Press = 'Shift+D3'; Reads = 'U+0023 D3 2' }
        @{ Press = 'Oem4'; Reads = 'U+005B Oem4 0' }
        @{ Press = 'Oem6'; Reads = 'U+005D Oem6 0' }
        @{ Press = 'Shift+Oem4'; Reads = 'U+007B Oem4 2' }
        @{ Press = 'Shift+Oem6'; Reads = 'U+007D Oem6 2' }
        @{ Press = 'Oem5'; Reads = 'U+005C Oem5 0' }
        @{ Press = 'Shift+Oem5'; Reads = 'U+007C Oem5 2' }
    )
}

# The keys that each modifier sets down, as virtual-key codes: Shift and the
# left Shift; Ctrl and the left Ctrl; Alt and the left Alt; and AltGr, the
# right Alt, with the left Ctrl that Windows adds to it.
$modifierKeys = @{
    Shift = 0x10, 0xA0
    Ctrl = 0x11, 0xA2
    Alt = 0x12, 0xA4
    AltGr = 0x11, 0xA2, 0x12, 0xA5
}
# The keys that are extended keys: Page Up to Down Arrow, Insert and Delete.
$extendedKeys = @(0x21..0x28) + @(0x2D, 0x2E)

$WM_KEYDOWN = 0x0100
$WM_SYSKEYDOWN = 0x0104
$WM_INPUTLANGCHANGEREQUEST = 0x0050
$VK_F10 = 0x79
$MAPVK_VK_TO_VSC = 0

# The script that runs in the terminal: it writes each key that it reads.
# With TreatControlCAsInput, Ctrl+C is a key, as in PSReadLine.
$program = @'
param([Parameter(Mandatory)][string]$Log)
$ErrorActionPreference = 'Stop'
[Console]::TreatControlCAsInput = $true
[IO.File]::AppendAllText($Log, "ready`n")
while ($true) {
    $key = [Console]::ReadKey($true)
    $line = 'U+{0:X4} {1} {2}' -f [int]$key.KeyChar, $key.Key, [int]$key.Modifiers
    [IO.File]::AppendAllText($Log, "$line`n")
}
'@

$name = "meiga-check-keyboard-$PID"
$log = Join-Path ([IO.Path]::GetTempPath()) "$name.txt"
$script = Join-Path ([IO.Path]::GetTempPath()) "$name.ps1"
Set-Content -LiteralPath $script -Value $program -Encoding utf8NoBOM
Remove-Item -LiteralPath $log -ErrorAction SilentlyContinue

# The lines that the script wrote. It appends them while the check reads.
function Get-Lines {
    if (-not (Test-Path -LiteralPath $log)) { return @() }
    $stream = [IO.File]::Open($log, 'Open', 'Read', 'ReadWrite')
    try { $text = [IO.StreamReader]::new($stream).ReadToEnd() }
    finally { $stream.Dispose() }
    # Only the whole lines.
    @($text.Substring(0, $text.LastIndexOf("`n") + 1) -split "`n" | Where-Object { $_ })
}

# Waits until the script has written $Count lines, and returns them.
function Wait-Lines([int]$Count) {
    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while ($true) {
        $lines = @(Get-Lines)
        if ($lines.Count -ge $Count) { return $lines }
        if ($process.HasExited) {
            throw "$exe ended with exit code $($process.ExitCode) while the check typed."
        }
        if ([DateTime]::UtcNow -gt $deadline) { return $lines }
        Start-Sleep -Milliseconds 20
    }
}

function Get-LayoutName([IntPtr]$Handle) {
    $id = ($Handle.ToInt64() -shr 16) -band 0xFFFF
    $layouts.Keys | Where-Object { $layouts[$_] -eq $id } | Select-Object -First 1
}

# Types a key, such as "AltGr+Oem7": sets its modifier keys down, and posts
# the messages of the key going down and up.
function Send-Key([string]$Key) {
    $parts = $Key -split '\+'
    $code = [int][ConsoleKey]$parts[-1]
    $state = [byte[]]::new(256)
    $modifiers = if ($parts.Count -gt 1) { $parts[0..($parts.Count - 2)] } else { @() }
    foreach ($modifier in $modifiers) {
        foreach ($down in $modifierKeys[$modifier]) { $state[$down] = 0x80 }
    }
    if (-not $api::SetKeyboardState($state)) {
        throw "SetKeyboardState failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
    # With Alt and without Ctrl, and for F10, Windows sends the system key
    # messages.
    $alt = $parts -contains 'Alt'
    $message = if ($alt -or $code -eq $VK_F10) { $WM_SYSKEYDOWN } else { $WM_KEYDOWN }
    $scan = $api::MapVirtualKeyExW($code, $MAPVK_VK_TO_VSC, $keyboard)
    [long]$bits = 1 -bor ($scan -shl 16)
    if ($extendedKeys -contains $code) { $bits = $bits -bor (1 -shl 24) }
    if ($alt) { $bits = $bits -bor (1 -shl 29) }
    # Going up: the key was down (bit 30), and goes up (bit 31).
    $up = $bits -bor 0xC0000000L
    if (-not $api::PostMessageW($window, $message, [UIntPtr]::new($code), [IntPtr]::new($bits)) -or
        -not $api::PostMessageW($window, $message + 1, [UIntPtr]::new($code), [IntPtr]::new($up))) {
        throw "PostMessageW failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

$info = [Diagnostics.ProcessStartInfo]::new($Path)
$info.UseShellExecute = $false
foreach ($argument in 'pwsh', '-NoLogo', '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
    '-File', $script, '-Log', $log) {
    $info.ArgumentList.Add($argument)
}
$process = $null
$attached = $false
$previous = [IntPtr]::Zero
$loaded = [IntPtr]::Zero
$passed = $false
try {
    $process = [Diagnostics.Process]::Start($info)
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
    if (@(Wait-Lines 1).Count -lt 1) {
        throw "The script in the terminal did not start within $TimeoutSeconds s."
    }

    $keyboard = $api::GetKeyboardLayout($thread)
    if ($Layout -and (Get-LayoutName $keyboard) -ne $Layout) {
        # A layout of the session with the same keyboard, or a new one.
        $list = [IntPtr[]]::new(64)
        $count = $api::GetKeyboardLayoutList($list.Count, $list)
        $wanted = $list[0..($count - 1)] | Where-Object { (Get-LayoutName $_) -eq $Layout } | Select-Object -First 1
        if (-not $wanted) {
            $loaded = $api::LoadKeyboardLayoutW($layoutIds[$Layout], 0)
            if ($loaded -eq [IntPtr]::Zero) {
                throw "Could not load the $Layout keyboard layout: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
            }
            $wanted = $loaded
        }
        $previous = $keyboard
        $null = $api::PostMessageW($window, $WM_INPUTLANGCHANGEREQUEST, [UIntPtr]::Zero, $wanted)
        $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
        while (($keyboard = $api::GetKeyboardLayout($thread)) -ne $wanted) {
            if ([DateTime]::UtcNow -gt $deadline) {
                throw "The window of $exe did not change to the $Layout keyboard layout."
            }
            Start-Sleep -Milliseconds 20
        }
    }
    $layoutName = Get-LayoutName $keyboard
    if (-not $layoutName) {
        throw ("The window of {0} has a keyboard layout that the check does not know ({1:X8}): use -Layout Spanish or -Layout US." -f $exe, $keyboard.ToInt64())
    }
    Write-Host ("Typing with the {0} keyboard layout ({1:X8})." -f $layoutName, $keyboard.ToInt64())

    # SetKeyboardState sets the key state of the thread that calls it: share
    # it with the thread of the window. This thread needs a message queue,
    # which PeekMessageW creates.
    $message = [Meiga.Tools.KeyboardCheck+MSG]::new()
    $null = $api::PeekMessageW([ref]$message, [IntPtr]::Zero, 0, 0, 0)
    $attached = $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $true)
    if (-not $attached) {
        throw "AttachThreadInput failed: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }

    $results = foreach ($case in $cases[$layoutName]) {
        $start = @(Get-Lines).Count
        $expected = 0
        $reads = @($case.Reads)
        foreach ($key in @($case.Press)) {
            Send-Key $key
            # A dead key: wait until it is read, before the next key needs
            # other modifiers.
            $expected = [Math]::Min($expected + 1, $reads.Count)
            $null = Wait-Lines ($start + $expected)
        }
        $read = @(Wait-Lines ($start + $reads.Count) | Select-Object -Skip $start)
        [pscustomobject]@{
            Keys = @($case.Press) -join ', '
            Expected = $reads -join ' | '
            Read = $read -join ' | '
        }
    }
    $wrong = @($results | Where-Object { $_.Read -ne $_.Expected })
    if ($wrong.Count -gt 0) {
        $wrong | Format-Table -AutoSize -Wrap | Out-Host
        throw "$($wrong.Count) of $(@($results).Count) keys did not reach the program as expected."
    }
    Write-Host "OK: the $(@($results).Count) keys reached the program with their characters, keys and modifier keys."

    $null = $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $false)
    $attached = $false
    if ($previous -ne [IntPtr]::Zero) {
        $null = $api::PostMessageW($window, $WM_INPUTLANGCHANGEREQUEST, [UIntPtr]::Zero, $previous)
        $previous = [IntPtr]::Zero
    }
    # CloseMainWindow sends WM_CLOSE, as the X button does.
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
    $passed = $true
}
finally {
    if ($attached) {
        $null = $api::AttachThreadInput($api::GetCurrentThreadId(), $thread, $false)
    }
    if ($process) {
        # Put back the layout before the window goes away.
        if ($previous -ne [IntPtr]::Zero -and -not $process.HasExited) {
            $null = $api::PostMessageW($window, $WM_INPUTLANGCHANGEREQUEST, [UIntPtr]::Zero, $previous)
            Start-Sleep -Milliseconds 200
        }
        if (-not $process.HasExited) {
            $process.Kill()
        }
        $process.Dispose()
    }
    if ($loaded -ne [IntPtr]::Zero) {
        $null = $api::UnloadKeyboardLayout($loaded)
    }
    Remove-Item -LiteralPath $script -ErrorAction SilentlyContinue
    if ($passed) {
        Remove-Item -LiteralPath $log -ErrorAction SilentlyContinue
    }
    elseif (Test-Path -LiteralPath $log) {
        Write-Host "What the program read is in $log"
    }
}
