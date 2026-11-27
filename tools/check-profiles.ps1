<#
.SYNOPSIS
Checks the profiles and the configuration of meiga.exe: the message box with
the errors of config.toml, the profile that its first tab runs, the
environment of its programs, and the menu of the profiles, with the keyboard
and with the mouse.

.DESCRIPTION
Starts meiga.exe with its frame log (--trace-frames), in a folder of its own,
and with an APPDATA of its own, whose config.toml makes Probe the default
profile: a PowerShell script that runs in another folder, whose name has a
letter that is not ASCII. The file also hides Command Prompt, and has two
errors: a key that meiga does not know, and a value that is neither true nor
false. meiga gets the variables of Windows Terminal, and other values for the
ones that it sets for its programs. The first tab runs the script, which
does one step at a time, when this check tells it to, and reports it in a
file; the other tabs of Probe only report their folder and their variables
in a file of their own, and wait until their tab closes.

The check finds the shells that are installed as meiga does (see
crates/meiga-win/src/profiles.rs): in the folders where their installers put
them, and in the registry. It posts mouse and key messages to the window, at
the places of the layouts of the bar and of the menu, which it computes as
meiga does (ADR 0020, 0021 and 0022): only the width of the menu, which
depends on the font, comes from the frame that opens it. It checks that:

- once the window is on the screen and painted, a message box in front of
  it, owned by it, with the title meiga and one button, tells the errors of
  the file, each with its line and its column, and that meiga ignored them;
  meanwhile, the program of the first tab runs. When the box closes, the
  window paints all of it again, and draws what the program wrote meanwhile,
  and what it writes then: only in the terminal and, if the program gave a
  title, in the first tab. PowerShell gives one while it starts as an
  administrator: it puts "Administrator: " before the title of its console;
- the first tab runs Probe, with the variables of its command line and of
  its folder expanded, in its folder. Its program gets TERM_PROGRAM=meiga,
  TERM_PROGRAM_VERSION with the version of meiga, and COLORTERM=truecolor,
  instead of the values that meiga had; WSLENV with them, and without the
  variables of Windows Terminal; and neither WT_SESSION nor WT_PROFILE_ID;
- Ctrl+Shift+Space opens the menu of the profiles below the button + and
  the one of the profiles, and the frame draws only the menu: the profiles
  that the check finds, but Command Prompt, and Probe at the end, as wide as
  the longest name. No program starts, nor while Space repeats. Down draws
  only the highlights of the first two items, and Esc closes the menu. The
  program gets Ctrl and Shift, which went down for it, and no other key;
- the pointer on the button of the profiles, and its press, draw only the
  button; when the left button goes up on it, the menu opens, and only the
  button and the menu are drawn, and the window lets the mouse go. The
  pointer on an item draws only its highlight, and a click on Probe opens a
  tab at the end, shown with one full frame, whose program runs the command
  line of Probe in its folder;
- Ctrl+Shift+Space, Down and Enter open each profile that the check finds,
  whose program runs the command line that the check expects, in the home
  folder of the user: WSL with --cd "~", in the one of Linux. Ctrl+Shift+W
  closes it, and its program ends;
- the button + opens a tab with Probe, the default profile, and Duplicate
  tab copies it, with the same command line and the same folder;
- meiga.exe ends with exit code 0 when its window closes;
- with a config.toml that is not TOML, the message box says where it is
  wrong, and that meiga ignored the file: the first tab runs the default
  shell, in the home folder, and meiga.exe ends with exit code 0 when its
  window closes.

Opening each profile starts its shell, as a tab of meiga does: the ones of
the user load their profiles, and WSL may have to start. The check waits
until each one is idle, with the programs that it started, and closes it
then: the bash.exe of Git Bash is a launcher, which waits for the shell that
it starts, while that shell starts git for its prompt; and a program that
starts as its console closes keeps the console from closing, so the
session ends its programs 10 s later (ADR 0011). A start-up file that
changes the folder of its shell, as a cd in the .bashrc of Git Bash, can
make the check fail, if it runs before the check reads the folder.

As check-menu.ps1 does, the check moves the pointer out of the window, posts
its messages while the thread of the window is suspended, and types each key
with the key state that it needs, which it shares with the thread of the
window (see check-tabs.ps1). It reads the command line of each program from
WMI, and its folder from the memory of its process.

It needs PowerShell 7 (pwsh) in the PATH, for the script in the terminal.

If a check fails, the script ends meiga.exe and keeps its folder, with the
frame log and the reports of the script.

.PARAMETER Path
The executable to check. Defaults to target\release\meiga.exe.

.PARAMETER TimeoutSeconds
How long to wait for the window, for the message box, for each step, and
for the programs of the check to end. The shells of the user get three
times as long to end, and at least 30 s; and so does meiga, once its window
closes, as it waits for their sessions.
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

# The version of meiga, which its programs get in TERM_PROGRAM_VERSION.
$manifest = Get-Content -LiteralPath (Join-Path $PSScriptRoot '..\Cargo.toml') -Raw
if ($manifest -notmatch '(?ms)^\[workspace\.package\]\s*$.*?^version\s*=\s*"([^"]+)"') {
    throw 'Could not find the version of meiga in Cargo.toml.'
}
$version = $Matches[1]

if (-not ('Meiga.Tools.ProfilesCheck' -as [type])) {
    Add-Type -Namespace Meiga.Tools -Name ProfilesCheck -MemberDefinition @'
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

[StructLayout(LayoutKind.Sequential)]
struct PROCESS_BASIC_INFORMATION
{
    public IntPtr ExitStatus, PebBaseAddress, AffinityMask, BasePriority, UniqueProcessId, InheritedFromUniqueProcessId;
}

[StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
struct PROCESSENTRY32W
{
    public uint dwSize, cntUsage, th32ProcessID;
    public IntPtr th32DefaultHeapID;
    public uint th32ModuleID, cntThreads, th32ParentProcessID;
    public int pcPriClassBase;
    public uint dwFlags;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)] public string szExeFile;
}

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

[DllImport("user32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool GetGUIThreadInfo(uint thread, ref GUITHREADINFO info);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetClientRect(IntPtr window, out RECT rect);

[DllImport("user32.dll")]
public static extern uint GetDpiForWindow(IntPtr window);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool GetWindowRect(IntPtr window, out RECT rect);

[DllImport("user32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool IsZoomed(IntPtr window);

[DllImport("user32.dll")]
public static extern int GetSystemMetricsForDpi(int index, uint dpi);

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

[DllImport("kernel32.dll", SetLastError = true)]
static extern IntPtr OpenProcess(uint access, [MarshalAs(UnmanagedType.Bool)] bool inherit, int id);

[DllImport("kernel32.dll")]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool GetProcessTimes(IntPtr process, out long creation, out long exit, out long kernel, out long user);

[DllImport("kernel32.dll")]
static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);

[DllImport("kernel32.dll", SetLastError = true)]
static extern IntPtr CreateToolhelp32Snapshot(uint flags, uint process);

[DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool Process32FirstW(IntPtr snapshot, ref PROCESSENTRY32W entry);

[DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool Process32NextW(IntPtr snapshot, ref PROCESSENTRY32W entry);

[DllImport("ntdll.dll")]
static extern int NtQueryInformationProcess(IntPtr process, int type, ref PROCESS_BASIC_INFORMATION information, int size, out int returned);

[DllImport("kernel32.dll", SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
static extern bool ReadProcessMemory(IntPtr process, IntPtr address, byte[] buffer, IntPtr size, out IntPtr read);

[DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
static extern uint GetFileAttributesW(string path);

[DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
static extern uint GetLongPathNameW(string path, System.Text.StringBuilder longPath, uint size);

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

// The current folder of the process `process`, or null if it cannot be read:
// the one of its PEB, which Windows sets when it starts the process, with a
// backslash at its end. In the PEB of x64, the parameters of the process are
// at 0x20; and in them, the folder, a UNICODE_STRING at 0x38: its length in
// bytes, and the address of its characters at 0x40.
public static string FolderOf(int process)
{
    // PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ
    IntPtr handle = OpenProcess(0x1000 | 0x0010, false, process);
    if (handle == IntPtr.Zero) return null;
    try
    {
        var information = new PROCESS_BASIC_INFORMATION();
        int returned;
        if (NtQueryInformationProcess(handle, 0, ref information, Marshal.SizeOf(information), out returned) != 0) return null;
        byte[] address = new byte[8];
        IntPtr read;
        if (!ReadProcessMemory(handle, information.PebBaseAddress + 0x20, address, new IntPtr(8), out read)) return null;
        IntPtr parameters = new IntPtr(BitConverter.ToInt64(address, 0));
        byte[] folder = new byte[16];
        if (!ReadProcessMemory(handle, parameters + 0x38, folder, new IntPtr(16), out read)) return null;
        int length = BitConverter.ToUInt16(folder, 0);
        IntPtr characters = new IntPtr(BitConverter.ToInt64(folder, 8));
        byte[] text = new byte[length];
        if (length > 0 && !ReadProcessMemory(handle, characters, text, new IntPtr(length), out read)) return null;
        return System.Text.Encoding.Unicode.GetString(text);
    }
    finally { CloseHandle(handle); }
}

// Whether `path` is a file that Windows finds, as meiga tests it: also an
// alias of a program of the Microsoft Store.
public static bool IsFile(string path)
{
    uint attributes = GetFileAttributesW(path);
    // INVALID_FILE_ATTRIBUTES, and FILE_ATTRIBUTE_DIRECTORY.
    return attributes != 0xFFFFFFFF && (attributes & 0x10) == 0;
}

// The process `process` and the ones under it, the programs that it started
// and the ones that they started, as a text that changes when one of them
// starts or ends, or uses processor time: the number of each one and its
// time, in ticks of 100 ns. A process keeps the number of its parent after
// the parent ends, and Windows can give that number to another process:
// only the ones that started after their parent count. Null if `process`
// has ended.
public static string TreeOf(int process)
{
    // TH32CS_SNAPPROCESS
    IntPtr snapshot = CreateToolhelp32Snapshot(0x2, 0);
    if (snapshot == new IntPtr(-1)) throw new System.ComponentModel.Win32Exception();
    var children = new System.Collections.Generic.Dictionary<uint, System.Collections.Generic.List<uint>>();
    try
    {
        var entry = new PROCESSENTRY32W();
        entry.dwSize = (uint)Marshal.SizeOf(typeof(PROCESSENTRY32W));
        for (bool more = Process32FirstW(snapshot, ref entry); more; more = Process32NextW(snapshot, ref entry))
        {
            System.Collections.Generic.List<uint> list;
            if (!children.TryGetValue(entry.th32ParentProcessID, out list))
            {
                list = new System.Collections.Generic.List<uint>();
                children[entry.th32ParentProcessID] = list;
            }
            list.Add(entry.th32ProcessID);
        }
    }
    finally { CloseHandle(snapshot); }
    long created, used;
    if (!TimesOf((uint)process, out created, out used)) return null;
    var tree = new System.Collections.Generic.SortedDictionary<uint, long>();
    tree[(uint)process] = used;
    var pending = new System.Collections.Generic.Queue<Tuple<uint, long>>();
    pending.Enqueue(Tuple.Create((uint)process, created));
    while (pending.Count > 0)
    {
        var parent = pending.Dequeue();
        System.Collections.Generic.List<uint> list;
        if (!children.TryGetValue(parent.Item1, out list)) continue;
        foreach (uint child in list)
        {
            long childCreated, childUsed;
            if (tree.ContainsKey(child) || !TimesOf(child, out childCreated, out childUsed) || childCreated < parent.Item2) continue;
            tree[child] = childUsed;
            pending.Enqueue(Tuple.Create(child, childCreated));
        }
    }
    var text = new System.Text.StringBuilder();
    foreach (var pair in tree) text.Append(pair.Key).Append(' ').Append(pair.Value).Append(';');
    return text.ToString();
}

// When the process `process` started, and the processor time that it has
// used, in ticks of 100 ns; false if it has ended, or cannot be opened.
static bool TimesOf(uint process, out long created, out long used)
{
    created = used = 0;
    // PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE
    IntPtr handle = OpenProcess(0x1000 | 0x100000, false, (int)process);
    if (handle == IntPtr.Zero) return false;
    try
    {
        // A process that has ended, but that somebody holds open, is not
        // WAIT_TIMEOUT.
        if (WaitForSingleObject(handle, 0) != 0x102) return false;
        long exited, kernel, user;
        if (!GetProcessTimes(handle, out created, out exited, out kernel, out user)) return false;
        used = kernel + user;
        return true;
    }
    finally { CloseHandle(handle); }
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
$api = [Meiga.Tools.ProfilesCheck]

$WM_CLOSE = 0x0010
$WM_COMMAND = 0x0111
$WM_KEYDOWN = 0x0100
$WM_KEYUP = 0x0101
$WM_SYSKEYDOWN = 0x0104
$WM_SYSKEYUP = 0x0105
$WM_MOUSEMOVE = 0x0200
$WM_LBUTTONDOWN = 0x0201
$WM_LBUTTONUP = 0x0202
$MK_LBUTTON = 0x0001
$MAPVK_VK_TO_VSC = 0
$GW_OWNER = 4
$VK_RETURN = 0x0D
$VK_SHIFT = 0x10
$VK_CONTROL = 0x11
$VK_ESCAPE = 0x1B
$VK_SPACE = 0x20
$VK_DOWN = 0x28
$VK_W = 0x57
$VK_Z = 0x5A
$VK_APPS = 0x5D
$VK_F10 = 0x79
$VK_LSHIFT = 0xA0
$VK_LCONTROL = 0xA2
# The keys that Shift sets down: Shift and the left Shift. And the ones of
# Ctrl.
$shiftKeys = $VK_SHIFT, $VK_LSHIFT
$ctrlKeys = $VK_CONTROL, $VK_LCONTROL
$THREAD_PRIORITY_NORMAL = 0
$THREAD_PRIORITY_BELOW_NORMAL = -1
# The console of each tab: the one of the ConPTY of meiga, or the one of
# Windows if the ConPTY is not next to meiga.exe.
$consoles = 'OpenConsole.exe', 'conhost.exe'

# The script of the tabs of Probe. The first one writes on its screen, and
# reports its process and some of its variables in $Log; then, each time the
# check sets the event <Name>-go, it does its next step, reports it, and
# sets <Name>-done. The other ones report the same in a file of their own,
# and wait until their tab closes.
$program = @'
param([Parameter(Mandatory)][string]$Name, [Parameter(Mandatory)][string]$Log)
$ErrorActionPreference = 'Stop'
$e = [char]27
$out = [Console]::Out
# The cursor is hidden, so that it never blinks.
$out.Write("$e[?25l")
$report = [ordered]@{ pid = $PID }
foreach ($variable in 'TERM_PROGRAM', 'TERM_PROGRAM_VERSION', 'COLORTERM', 'WSLENV', 'WT_SESSION', 'WT_PROFILE_ID', 'APPDATA') {
    $report[$variable] = [Environment]::GetEnvironmentVariable($variable)
}
$json = ConvertTo-Json $report -Compress
$created = $false
$first = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, "$Name-first", [ref]$created)
if (-not $created) {
    [IO.File]::WriteAllText("$Log.$PID", $json)
    $null = $first.WaitOne()
    exit
}
$go = [Threading.EventWaitHandle]::OpenExisting("$Name-go")
$done = [Threading.EventWaitHandle]::OpenExisting("$Name-done")
# While the message box of meiga is open: the window draws it once the box
# closes.
$out.Write("$e[2J$e[Hmeiga-check-profiles: written while the message box is open")
[IO.File]::AppendAllText($Log, "ready $json`n")
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
    { $out.Write("$e[3;1Hmeiga-check-profiles: written after the message box"); 'wrote' },
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

$name = "meiga-check-profiles-$PID"
# All the files of the check, with the APPDATA of meiga, and the folder of
# Probe, whose name has a letter that is not ASCII: an n with a tilde. With
# the long names of the folders, which .NET gives as the current one.
$work = Join-Path ($api::LongPathOf([IO.Path]::GetTempPath())) $name
$appdata = Join-Path $work 'appdata'
$config = Join-Path $appdata 'meiga\config.toml'
$probeFolder = Join-Path $work "the folder $([char]0x00F1)"
$script = Join-Path $work 'probe.ps1'
$reports = Join-Path $work 'reports.txt'
$log = Join-Path $work 'frames.csv'
Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
$null = New-Item -ItemType Directory -Force -Path (Split-Path $config), $probeFolder
Set-Content -LiteralPath $script -Value $program -Encoding utf8NoBOM

# The configuration: Probe, with a variable of meiga in its command line and
# in its folder, is the default profile; Command Prompt is hidden; and two
# errors, which meiga ignores: a key that it does not know, and a value that
# is neither true nor false, whose profile stays visible.
$probeLine = 'pwsh -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%MEIGA_CHECK_WORK%\probe.ps1" -Name {0} -Log "%MEIGA_CHECK_WORK%\reports.txt"' -f $name
$configLines = @(
    '# Written by tools/check-profiles.ps1.'
    'default_profile = "Probe"'
    'colour = "red"'
    ''
    '[[profiles]]'
    'name = "Probe"'
    "command = '$probeLine'"
    "directory = '%MEIGA_CHECK_WORK%\$(Split-Path -Leaf $probeFolder)'"
    ''
    '[[profiles]]'
    'name = "Command Prompt"'
    'hidden = true'
    ''
    '[[profiles]]'
    'name = "Windows PowerShell"'
    'hidden = "yes"'
)
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText($config, ($configLines -join "`n") + "`n", $utf8)

# An error of the file, as meiga tells it: the line, and the column of $At
# in it, counted from 1.
function Get-Problem([string]$Line, [string]$At, [string]$Message) {
    'Line {0}, column {1}: {2}' -f ([Array]::IndexOf($configLines, $Line) + 1), ($Line.IndexOf($At) + 1), $Message
}
$problems = @(
    Get-Problem 'colour = "red"' 'colour' 'unknown key `colour`'
    Get-Problem 'hidden = "yes"' '"yes"' '`hidden` has to be true or false, not a string'
)

# What a tab of Probe runs: its command line, with the variable expanded, in
# its folder.
$probeProfile = [pscustomobject]@{
    Name = 'Probe'
    Command = $probeLine.Replace('%MEIGA_CHECK_WORK%', $work)
    Folder = $probeFolder
}

# What meiga gets: its APPDATA and the folder of the check, the variables of
# Windows Terminal, and other values for the ones of a terminal, as in a
# terminal that Windows Terminal or VS Code starts. WSLENV passes some of them
# to WSL, with their flags.
$given = [ordered]@{
    APPDATA = $appdata
    MEIGA_CHECK_WORK = $work
    WT_SESSION = '0a2c4f1e-6b3d-4e5f-8a7b-9c0d1e2f3a4b'
    WT_PROFILE_ID = '{61c54bbd-c2c6-5271-96e7-009a87ff44bf}'
    TERM_PROGRAM = 'vscode'
    TERM_PROGRAM_VERSION = '1.0.0'
    COLORTERM = '24bit'
    WSLENV = 'WT_SESSION:COLORTERM/u:WT_PROFILE_ID/p:MEIGA_CHECK_WORK/p'
}
# And what its programs get instead.
$expectedVariables = [ordered]@{
    TERM_PROGRAM = 'meiga'
    TERM_PROGRAM_VERSION = $version
    COLORTERM = 'truecolor'
    WSLENV = 'COLORTERM/u:MEIGA_CHECK_WORK/p:TERM_PROGRAM:TERM_PROGRAM_VERSION'
    WT_SESSION = $null
    WT_PROFILE_ID = $null
    APPDATA = $appdata
}

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

# The point in the middle of $Rect.
function Get-Center($Rect) {
    [pscustomobject]@{
        X = [int][Math]::Floor(($Rect.Left + $Rect.Right) / 2)
        Y = [int][Math]::Floor(($Rect.Top + $Rect.Bottom) / 2)
    }
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
# $From lines of the log, which have to be some, drew only inside the
# terminal, where the program of the first tab writes; and inside the first
# tab too, if the window no longer has the title $Title: then the program
# gave one, as PowerShell does while it starts as an administrator, which
# puts "Administrator: " before the title of its console. Otherwise throws
# $What.
function Assert-Written([int]$From, [string]$Title, [string]$What) {
    Wait-Quiet 400
    $frames = @(Get-Frames $From)
    # After the frames: the window already has the title that they drew.
    $allowed = @($terminal)
    if ($api::TextOf($window) -cne $Title) { $allowed += $bar.FirstTab }
    $outside = @(foreach ($area in @(Get-Areas $frames)) {
            if (@($allowed | Where-Object { Test-Inside $area $_ }).Count -eq 0) { $area }
        })
    if ($frames.Count -eq 0 -or $outside.Count -gt 0) {
        Show-Lines $frames
        throw "$What`: the window did not draw only inside $(($allowed | ForEach-Object { Show-Rect $_ }) -join ' and ')."
    }
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

# Waits until a frame after the first $From lines of the log presents the
# menu $Menu: then the window has taken the keys that opened it.
function Wait-Menu([int]$From, $Menu, [string]$What) {
    Wait-For { @(Get-Areas (Get-Frames $From) | Where-Object { Test-Inside $Menu $_ }).Count -gt 0 } {
        "$What did not open the menu of the profiles at $(Show-Rect $Menu)."
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

# Checks that the program of a tab of Probe got the variables of a terminal,
# and not the ones of meiga: $Report, as its script tells them.
function Assert-Variables($Report, [string]$What) {
    foreach ($variable in $expectedVariables.Keys) {
        $value = $Report.$variable
        $expected = $expectedVariables[$variable]
        if ($value -cne $expected) {
            $show = { param($Text) if ($null -eq $Text) { 'none' } else { "'$Text'" } }
            throw "$What got $(& $show $value) as $variable, instead of $(& $show $expected)."
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

# Checks that $Shell, a process from WMI, runs the command line of $Launch,
# which a tab of a profile runs, in its folder.
function Assert-Launch($Shell, $Launch, [string]$What) {
    if ($Shell.CommandLine -cne $Launch.Command) {
        throw "$What started '$($Shell.CommandLine)' instead of '$($Launch.Command)', the command line of $($Launch.Name)."
    }
    $folder = $api::FolderOf([int]$Shell.ProcessId)
    if ($null -eq $folder) {
        throw "$What started a program whose folder cannot be read: it may have ended."
    }
    # Folders do not distinguish upper and lower case.
    if ($folder.TrimEnd('\') -ne $Launch.Folder.TrimEnd('\')) {
        throw "$What started its program in $folder instead of $($Launch.Folder), the folder of $($Launch.Name)."
    }
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

# Waits until the process $Id, and the ones under it, have used no processor
# time for a second, and no other one has started or ended, or for $Seconds
# at most: a shell of the user has started, and its profile may keep
# working. A launcher, as the bash.exe of Git Bash, waits for the shell that
# it starts, which may still start programs, as git for its prompt; and a
# program that starts as its console closes keeps the console from closing,
# so the session ends its programs 10 s later (ADR 0011).
function Wait-Settled([int]$Id, [int]$Seconds) {
    $deadline = [DateTime]::UtcNow.AddSeconds($Seconds)
    $last = $null
    $since = [DateTime]::UtcNow
    while ([DateTime]::UtcNow -lt $deadline) {
        $tree = $api::TreeOf($Id)
        if ($null -eq $tree) { return }
        if ($tree -cne $last) {
            $last = $tree
            $since = [DateTime]::UtcNow
        }
        elseif (([DateTime]::UtcNow - $since).TotalMilliseconds -ge 1000) {
            return
        }
        Start-Sleep -Milliseconds 100
    }
}

# Waits until the process $Id has ended, also after meiga, for $Seconds at
# most.
function Wait-Ended([int]$Id, [string]$What, [int]$Seconds = $TimeoutSeconds) {
    $deadline = [DateTime]::UtcNow.AddSeconds($Seconds)
    while (Get-Process -Id $Id -ErrorAction Ignore) {
        if ([DateTime]::UtcNow -gt $deadline) {
            throw "$What was still running after $Seconds s."
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
# moves that Windows makes at its place would reach the bar.
function Move-Pointer {
    $point = [Meiga.Tools.ProfilesCheck+POINT]::new()
    if (-not $api::GetCursorPos([ref]$point)) { return }
    $inside = [Meiga.Tools.ProfilesCheck+POINT]::new()
    $inside.X = $point.X
    $inside.Y = $point.Y
    $rect = [Meiga.Tools.ProfilesCheck+RECT]::new()
    if (-not $api::ScreenToClient($window, [ref]$inside) -or -not $api::GetClientRect($window, [ref]$rect) -or
        $inside.X -lt 0 -or $inside.Y -lt 0 -or $inside.X -ge $rect.Right -or $inside.Y -ge $rect.Bottom) {
        return
    }
    if ($null -eq $script:pointer) { $script:pointer = $point }
    $title = [Meiga.Tools.ProfilesCheck+POINT]::new()
    $title.X = [Math]::Floor($rect.Right / 2)
    $title.Y = -8
    $null = $api::ClientToScreen($window, [ref]$title)
    if (-not $api::SetCursorPos($title.X, $title.Y)) {
        throw "Could not move the pointer out of the window: error $([Runtime.InteropServices.Marshal]::GetLastWin32Error())."
    }
}

# The client area of the window.
function Get-Client {
    $client = [Meiga.Tools.ProfilesCheck+RECT]::new()
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
    $rect = [Meiga.Tools.ProfilesCheck+RECT]::new()
    $corner = [Meiga.Tools.ProfilesCheck+POINT]::new()
    if (-not $api::GetWindowRect($window, [ref]$rect) -or -not $api::ClientToScreen($window, [ref]$corner)) {
        throw "Could not read where the window of $exe is."
    }
    if ($corner.Y -ne $rect.Top) { return 0 }
    # SM_CXPADDEDBORDER and SM_CYSIZEFRAME.
    $api::GetSystemMetricsForDpi(92, $dpi) + $api::GetSystemMetricsForDpi(33, $dpi)
}

# The layout of the bar with $Count tabs, as meiga computes it (see
# crates/meiga-core/src/ui/tabbar.rs): its height, with the band where the
# window is resized above the row of tabs, the first tab, the button + after
# the last tab, the one of the profiles joined to it, and both of them,
# which the menu of the profiles opens below. On the right of the bar go the
# buttons of the window, and the room to drag it.
function Get-Bar([int]$Count) {
    $client = Get-Client
    $band = Get-Band
    $height = $band + (Get-Scaled 32)
    $gap = Get-Scaled 4
    $top = $band + $gap
    $newSide = Get-Scaled 28
    $profilesSide = Get-Scaled 24
    $fixed = 3 * $gap + $newSide + $profilesSide + (Get-Scaled 45) + 3 * (Get-Scaled 46)
    $width = [int][Math]::Min((Get-Scaled 240), [Math]::Floor([Math]::Max(0, $client.Right - $fixed) / $Count))
    $left = $gap + $Count * $width + $gap
    $new = New-Rect $left $top ($left + $newSide) ($top + $newSide)
    $profiles = New-Rect $new.Right $new.Top ($new.Right + $profilesSide) $new.Bottom
    [pscustomobject]@{
        Height = $height
        FirstTab = New-Rect $gap $top ($gap + $width) $height
        New = $new
        Profiles = $profiles
        Buttons = Get-Union @($new, $profiles)
    }
}

# The sizes of a menu of $Items items, as meiga computes them (see
# crates/meiga-core/src/ui/menu.rs): between a border with some space inside
# it. Its width depends on the font, and comes from the frame that opens it.
function Get-MenuSizes([int]$Items) {
    $border = Get-Scaled 1
    $padding = Get-Scaled 2
    $side = Get-Scaled 4
    $gap = Get-Scaled 2
    $item = (Get-Scaled 28) + 2 * $gap
    [pscustomobject]@{
        Items = $Items
        Border = $border
        Padding = $padding
        Side = $side
        Gap = $gap
        Item = $item
        Height = 2 * ($border + $padding) + $Items * $item
        # Beside the text: the border, the space between it and the
        # highlight, and the one between the highlight and the text, on
        # each side.
        Sides = 2 * ($border + $side + (Get-Scaled 11))
    }
}

# The menu, as meiga places it: with its top-left corner at $Left, $Top or,
# where it does not fit, its right side at $OtherRight or its bottom at
# $OtherBottom; and never outside the client area.
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

# The items of $Menu, each one with a point in the middle of it, X and Y,
# and its highlight, without the space on its sides and above and below it.
function Get-Entries($Menu) {
    $top = $Menu.Top + $sizes.Border + $sizes.Padding
    $left = $Menu.Left + $sizes.Border + $sizes.Side
    $right = $Menu.Right - $sizes.Border - $sizes.Side
    $x = [Math]::Floor(($Menu.Left + $Menu.Right) / 2)
    foreach ($index in 1..$sizes.Items) {
        [pscustomobject]@{
            X = $x
            Y = $top + [Math]::Floor($sizes.Item / 2)
            Highlight = New-Rect $left ($top + $sizes.Gap) $right ($top + $sizes.Item - $sizes.Gap)
        }
        $top += $sizes.Item
    }
}

# A mouse message at a point: the message, its wParam and its lParam.
function Get-Mouse([int]$Message, [int]$Keys, $Point) {
    $Message, $Keys, (([int]$Point.X -band 0xFFFF) -bor (([int]$Point.Y -band 0xFFFF) -shl 16))
}

# Posts mouse messages at once: see PostAll.
function Send-Mouse([long[]]$Messages) {
    $api::PostAll($window, $thread, $Messages, $null)
}

# A click of the left button at $Point: the pointer moves there, and the
# button goes down and up.
function Send-Click($Point) {
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $Point
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
    $message = [Meiga.Tools.ProfilesCheck+MSG]::new()
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

# Ctrl and Shift go down, and each key of $Codes goes down and up, with
# both of them down. They stay down until Send-ModifiersUp, which changes
# the key state: before it, the window has to take the keys, which read it.
function Send-Shortcut([int[]]$Codes) {
    $messages = @(Get-Key $VK_CONTROL $true; Get-Key $VK_SHIFT $true)
    foreach ($code in $Codes) {
        $messages += @(Get-Key $code $true) + @(Get-Key $code $false)
    }
    Send-Keys $messages ($ctrlKeys + $shiftKeys)
}

# Shift and Ctrl go up.
function Send-ModifiersUp {
    Send-Keys @(Get-Key $VK_SHIFT $false; Get-Key $VK_CONTROL $false)
}

# The items of $Items in the order of $Before, which tells whether its first
# argument goes before its second, as meiga sorts them: an insertion sort,
# which keeps the order of the ones that are alike.
function Get-Sorted([object[]]$Items, [scriptblock]$Before) {
    $sorted = [Collections.Generic.List[object]]::new()
    foreach ($item in $Items) {
        $at = $sorted.Count
        while ($at -gt 0 -and (& $Before $item $sorted[$at - 1])) { $at-- }
        $sorted.Insert($at, $item)
    }
    $sorted.ToArray()
}

# The folders of the versions of PowerShell among $Names, the newest first,
# as meiga orders them: the ones named with a number, as 7, or, with
# $Preview, with a number and -preview, as 7-preview.
function Get-Versions([string[]]$Names, [bool]$Preview) {
    $versions = @(foreach ($folder in $Names) {
            $number = $folder
            if ($Preview) {
                if (-not $folder.EndsWith('-preview', [StringComparison]::Ordinal)) { continue }
                $number = $folder.Substring(0, $folder.Length - '-preview'.Length)
            }
            $value = [uint32]0
            if ($number -cmatch '^[0-9]+$' -and [uint32]::TryParse($number, [ref]$value)) {
                [pscustomobject]@{ Number = $value; Folder = $folder }
            }
        })
    $newest = {
        param($First, $Second)
        $First.Number -gt $Second.Number -or
        ($First.Number -eq $Second.Number -and [string]::CompareOrdinal($First.Folder, $Second.Folder) -gt 0)
    }
    Get-Sorted $versions $newest | ForEach-Object Folder
}

# The first PowerShell 7 that is installed, as meiga finds it (see
# crates/meiga-win/src/profiles.rs), or nothing.
function Get-PowerShell {
    $programFiles = [Environment]::GetEnvironmentVariable('ProgramFiles')
    $local = [Environment]::GetEnvironmentVariable('LOCALAPPDATA')
    $user = [Environment]::GetEnvironmentVariable('USERPROFILE')
    $installed = "$programFiles\PowerShell"
    # Only folders, and not links to them.
    $folders = @()
    if ($programFiles) {
        $folders = @(Get-ChildItem -LiteralPath $installed -Directory -Force -ErrorAction Ignore |
                Where-Object { -not $_.LinkType } | ForEach-Object Name)
    }
    $paths = @(
        Get-Versions $folders $false | ForEach-Object { "$installed\$_\pwsh.exe" }
        if ($local) { "$local\Microsoft\WindowsApps\Microsoft.PowerShell_8wekyb3d8bbwe\pwsh.exe" }
        if ($user) { "$user\scoop\apps\pwsh\current\pwsh.exe"; "$user\.dotnet\tools\pwsh.exe" }
        Get-Versions $folders $true | ForEach-Object { "$installed\$_\pwsh.exe" }
        if ($local) { "$local\Microsoft\WindowsApps\Microsoft.PowerShellPreview_8wekyb3d8bbwe\pwsh.exe" }
    )
    $paths | Where-Object { $api::IsFile($_) } | Select-Object -First 1
}

# The value $Value of the key $Key of the registry, if it is a string, with
# the variables that it may have expanded, or $null.
function Get-RegistryText($Key, [string]$Value) {
    if ($Key.GetValueNames() -notcontains $Value) { return $null }
    if ($Key.GetValueKind($Value) -notin 'String', 'ExpandString') { return $null }
    [string]$Key.GetValue($Value)
}

# The names of the distributions of WSL of the user, as meiga orders them:
# the default one first, and then the others by name, without
# distinguishing case; without the ones that are not installed, nor the
# engines of Docker Desktop and Rancher Desktop.
function Get-Distributions {
    $lxss = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Lxss')
    if (-not $lxss) { return }
    try {
        $default = Get-RegistryText $lxss 'DefaultDistribution'
        $installed = @(foreach ($subkey in $lxss.GetSubKeyNames()) {
                $key = $lxss.OpenSubKey($subkey)
                if (-not $key) { continue }
                try {
                    $distribution = Get-RegistryText $key 'DistributionName'
                    $installedState = $key.GetValueNames() -contains 'State' -and $key.GetValueKind('State') -eq 'DWord' -and
                    [int]$key.GetValue('State') -eq 1
                    $engine = $null -ne $distribution -and ($distribution.StartsWith('docker-desktop', [StringComparison]::OrdinalIgnoreCase) -or
                        $distribution.StartsWith('rancher-desktop', [StringComparison]::OrdinalIgnoreCase))
                    if ($null -ne $distribution -and $installedState -and -not $engine) {
                        [pscustomobject]@{
                            Name = $distribution
                            IsDefault = $null -ne $default -and [string]::Equals($subkey, $default, [StringComparison]::OrdinalIgnoreCase)
                        }
                    }
                }
                finally { $key.Dispose() }
            })
    }
    finally { $lxss.Dispose() }
    $order = {
        param($First, $Second)
        if ($First.IsDefault -ne $Second.IsDefault) { return $First.IsDefault }
        [string]::CompareOrdinal($First.Name.ToLowerInvariant(), $Second.Name.ToLowerInvariant()) -lt 0
    }
    Get-Sorted $installed $order | ForEach-Object Name
}

# The bash.exe of Git for Windows, as meiga finds it, or nothing: in the
# folder where the registry says that it is installed, for all the users or
# for this one, or in the folder of scoop.
function Get-GitBash {
    $paths = @(
        foreach ($root in [Microsoft.Win32.Registry]::LocalMachine, [Microsoft.Win32.Registry]::CurrentUser) {
            $key = $root.OpenSubKey('SOFTWARE\GitForWindows')
            if ($key) {
                try {
                    $folder = Get-RegistryText $key 'InstallPath'
                    if ($null -ne $folder) { "$($folder.TrimEnd('\'))\bin\bash.exe" }
                }
                finally { $key.Dispose() }
            }
        }
        $user = [Environment]::GetEnvironmentVariable('USERPROFILE')
        if ($user) { "$user\scoop\apps\git\current\bin\bash.exe" }
    )
    $paths | Where-Object { $api::IsFile($_) } | Select-Object -First 1
}

# $Text as an argument of a command line, as meiga quotes it (see
# crates/meiga-core/src/shell.rs): in quotes if it is empty or has spaces or
# tabs; a quote, and the backslashes before it, escaped; and the
# backslashes before the closing quote, doubled.
function Get-Argument([string]$Text) {
    $quote = $Text -eq '' -or $Text -match '[ \t]'
    $line = [Text.StringBuilder]::new()
    if ($quote) { $null = $line.Append([char]'"') }
    $backslashes = 0
    foreach ($character in $Text.ToCharArray()) {
        if ($character -eq [char]'\') { $backslashes++ }
        elseif ($character -eq [char]'"') {
            $null = $line.Append([char]'\', $backslashes + 1)
            $backslashes = 0
        }
        else { $backslashes = 0 }
        $null = $line.Append($character)
    }
    if ($quote) { $null = $line.Append([char]'\', $backslashes).Append([char]'"') }
    $line.ToString()
}

# The command line of $Program with $Arguments, as meiga writes it: the
# program in quotes only if it is empty or has spaces or tabs.
function Get-CommandLine([string]$Program, [string[]]$Arguments) {
    $line = if ($Program -eq '' -or $Program -match '[ \t]') { "`"$Program`"" } else { $Program }
    foreach ($argument in $Arguments) { $line += ' ' + (Get-Argument $argument) }
    $line
}

# The command line that a tab runs for $Command, if it is wsl.exe from the
# folder of the system, $System, and its folder is the home one, as meiga
# writes it (see crates/meiga-core/src/profile.rs): with --cd "~". Nothing
# for other programs, and for a wsl.exe that has --cd, or ~ as an argument.
function Get-WslCommand([string]$Command, [string]$System) {
    if ($Command.Length -lt 1) { return }
    # The program ends at the first space or quote after its first
    # character, which can be the opening quote.
    $start = if ($Command[0] -eq [char]'"') { 1 } else { 0 }
    $end = $Command.IndexOfAny([char[]]' "', 1)
    if ($end -lt 0) { $end = $Command.Length }
    $program = $Command.Substring($start, $end - $start)
    $slash = $program.LastIndexOfAny([char[]]'\/')
    $file = $program.Substring($slash + 1)
    if ($file -ne 'wsl' -and $file -ne 'wsl.exe') { return }
    if ($slash -ge 0 -and $program.Substring(0, $slash) -ne $System) { return }
    $arguments = if ($end + 1 -lt $Command.Length) { $Command.Substring($end + 1) } else { '' }
    if ($arguments.Contains('--cd')) { return }
    $tilde = $arguments.IndexOf([char]'~')
    if ($tilde -ge 0 -and ($tilde + 1 -eq $arguments.Length -or $arguments[$tilde + 1] -eq [char]' ')) { return }
    "`"$program`" --cd `"~`" $arguments"
}

# The profiles that meiga finds, in the order of its menu, without a
# configuration, each one with its name, the command line that its tab
# runs, and its folder, the home one (see crates/meiga-win/src/profiles.rs):
# PowerShell, Windows PowerShell, Command Prompt, the distributions of WSL
# and Git Bash, when their programs are there, each name only once.
function Get-Detected {
    $system = [Environment]::SystemDirectory
    $userFolder = [Environment]::GetEnvironmentVariable('USERPROFILE')
    # Each one: its name, its program, and its arguments.
    $found = [Collections.Generic.List[object]]::new()
    $pwsh = Get-PowerShell
    if ($pwsh) { $found.Add(@('PowerShell', $pwsh, @())) }
    $windowsPowerShell = "$system\WindowsPowerShell\v1.0\powershell.exe"
    if ($api::IsFile($windowsPowerShell)) { $found.Add(@('Windows PowerShell', $windowsPowerShell, @())) }
    $cmd = "$system\cmd.exe"
    if ($api::IsFile($cmd)) { $found.Add(@('Command Prompt', $cmd, @())) }
    $wsl = "$system\wsl.exe"
    if ($api::IsFile($wsl)) {
        foreach ($distribution in Get-Distributions) { $found.Add(@($distribution, $wsl, @('-d', $distribution))) }
    }
    $bash = Get-GitBash
    if ($bash) { $found.Add(@('Git Bash', $bash, @('-i', '-l'))) }
    $names = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($candidate in $found) {
        $profileName, $file, $arguments = $candidate
        if (-not $names.Add($profileName)) { continue }
        $command = Get-CommandLine $file $arguments
        $wslCommand = Get-WslCommand $command $system
        [pscustomobject]@{
            Name = $profileName
            Command = if ($wslCommand) { $wslCommand } else { $command }
            Folder = $userFolder
        }
    }
}

# The command line of meiga: its frame log, and no program, so that its first
# tab runs the default profile. Its folder is the one of the check, which no
# profile has: a tab that started there would not start in its own.
$info = [Diagnostics.ProcessStartInfo]::new($Path)
$info.UseShellExecute = $false
$info.WorkingDirectory = $work
foreach ($variable in $given.Keys) { $info.Environment[$variable] = $given[$variable] }
$info.ArgumentList.Add('--trace-frames')
$info.ArgumentList.Add($log)
$process = $null
$pointer = $null
$attached = $false
$passed = $false
# How long the shells of the user get to end, once their tabs close; and
# meiga, once its window closes, as it waits for their sessions before it
# ends: a session that gets stuck ends its programs once 10 s pass without
# one of them ending (ADR 0011 and ADR 0026).
$shellSeconds = [Math]::Max(30, 3 * $TimeoutSeconds)
try {
    # The profiles that meiga finds, and the ones of its menu with this
    # config.toml: without Command Prompt, and with Probe at the end, as wide
    # as the longest name.
    $detected = @(Get-Detected)
    $listed = @($detected | Where-Object Name -CNE 'Command Prompt') + $probeProfile
    $columns = ($listed | ForEach-Object { $_.Name.Length } | Measure-Object -Maximum).Maximum

    $process = [Diagnostics.Process]::Start($info)
    $started = $process.StartTime

    # 1. Once the window is on the screen and painted, a message box tells
    # the errors of config.toml. Meanwhile, the program of the first tab
    # writes on its screen, which the window draws once the box closes.
    Wait-For { $api::WindowOf($process.Id, 'meiga') -ne [IntPtr]::Zero } { "$exe did not open its window." }
    $window = $api::WindowOf($process.Id, 'meiga')
    $processId = [uint32]0
    $thread = $api::GetWindowThreadProcessId($window, [ref]$processId)
    $dialog = Get-Dialog
    if (-not $api::IsWindowVisible($window)) {
        throw "The message box opened before the window of $exe was on the screen."
    }
    if (@(Get-Log | Where-Object kind -EQ 'paint').Count -eq 0) {
        throw "The message box opened before the window of $exe was painted."
    }
    $expected = "There are errors in $($config):`n`n$($problems -join "`n")`n`nmeiga ignored them, and uses its default values instead."
    if ($dialog.Text -cne $expected) {
        throw "The message box says:`n$($dialog.Text)`n... instead of:`n$expected"
    }
    Wait-For { @(Get-Reports | Where-Object { $_.StartsWith('ready ') }).Count -gt 0 } {
        'While the message box was open, the program of the first tab did not write on its screen.'
    }
    # Time for the reader of the tab to read what the program wrote, while
    # the box is open.
    Start-Sleep -Milliseconds 500
    $dpi = $api::GetDpiForWindow($window)
    $client = Get-Client
    $bar = Get-Bar 1
    $terminal = New-Rect 0 $bar.Height $client.Right $client.Bottom
    Move-Pointer
    # The title of the window, which does not change while the box is open.
    $titleBefore = $api::TextOf($window)
    $start = @(Get-Log).Count
    Close-Dialog $dialog
    Wait-For { @(Get-Log | Select-Object -Skip $start | Where-Object { $_.kind -eq 'paint' -and (Test-Full $_) }).Count -gt 0 } {
        'When the message box closed, the window did not paint all of it.'
    }
    Assert-Written $start $titleBefore 'When the message box closed, what the program wrote meanwhile'
    $start = @(Get-Log).Count
    $null = Invoke-Step 'write after the message box'
    Assert-Written $start $titleBefore 'What the program wrote after the message box'
    $titleNow = $api::TextOf($window)
    $titled = if ($titleNow -cne $titleBefore) { "; its tab shows the title that the program gave, '$titleNow'" } else { '' }
    Write-Host "OK: once the window is on the screen and painted, a message box owned by it, with the title meiga and one button, tells the $($problems.Count) errors of config.toml, each with its line and its column, and that meiga ignored them; when it closes, the window paints all of it again, and draws what the program of the first tab wrote meanwhile, and what it writes then$titled."

    # 2. The first tab runs Probe, in its folder, with the environment of a
    # terminal.
    $report = @(Get-Reports | Where-Object { $_.StartsWith('ready ') })[0].Substring(6) | ConvertFrom-Json
    $first = [int]$report.pid
    $shells = @(Get-Shells)
    if ($shells.Count -ne 1 -or [int]$shells[0].ProcessId -ne $first) {
        throw "With one tab, the programs of $exe are $(($shells | ForEach-Object { "$($_.ProcessId): $($_.CommandLine)" }) -join '; '), instead of the script of Probe, $first."
    }
    Assert-Launch $shells[0] $probeProfile 'The first tab'
    Assert-Variables $report 'The program of the first tab'
    Wait-Shown $first @() 'With one tab'
    Write-Host "OK: the first tab runs Probe, the default profile of config.toml, with the variables of its command line and of its folder expanded, in its folder, whose name is not ASCII; its program gets TERM_PROGRAM=meiga, TERM_PROGRAM_VERSION=$version, COLORTERM=truecolor and WSLENV=$($expectedVariables.WSLENV), and neither WT_SESSION nor WT_PROFILE_ID."

    # 3. Ctrl+Shift+Space opens the menu of the profiles below the buttons,
    # with its first item highlighted. Ctrl and Shift go down for the
    # program, before the menu opens; Space repeats, which does nothing, and
    # goes up. Then Shift and Ctrl go up for the program, Down moves the
    # highlight, and Esc closes the menu. Z goes to the program.
    Join-Input
    $sizes = Get-MenuSizes $listed.Count
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Keys @(
        Get-Key $VK_CONTROL $true
        Get-Key $VK_SHIFT $true
        Get-Key $VK_SPACE $true
        Get-Key $VK_SPACE $true $true
        Get-Key $VK_SPACE $false
    ) ($ctrlKeys + $shiftKeys)
    Wait-Quiet 400
    $frames = @(Get-Frames $start)
    $areas = @(Get-Areas $frames)
    if ($areas.Count -ne 1) {
        Show-Lines $frames
        throw "Ctrl+Shift+Space drew $($areas.Count) areas instead of the menu of the profiles."
    }
    $menuWidth = $areas[0].Right - $areas[0].Left
    $cell = ($menuWidth - $sizes.Sides) / $columns
    $menu = Get-Menu $bar.Buttons.Left $bar.Buttons.Bottom $bar.Buttons.Right $bar.Buttons.Top
    if ((Show-Rect $areas[0]) -ne (Show-Rect $menu) -or $cell -lt 1 -or $cell -ne [Math]::Floor($cell)) {
        throw "Ctrl+Shift+Space drew $(Show-Rect $areas[0]), and the menu of the profiles has to be $(Show-Rect $menu): below the buttons + and of the profiles, $($sizes.Height) pixels high, for $($listed.Count) profiles, and as wide as $columns cells and $($sizes.Sides) pixels."
    }
    if (@(Get-Shells).Count -ne 1) {
        throw 'Ctrl+Shift+Space, or its repeat, opened a tab.'
    }
    Send-ModifiersUp
    $entries = @(Get-Entries $menu)
    $start = @(Get-Log).Count
    Send-Press $VK_DOWN
    $null = Assert-Inside $start (Get-Union @($entries[0].Highlight, $entries[1].Highlight)) 'Down on the menu of the profiles'
    $start = @(Get-Log).Count
    Send-Press $VK_ESCAPE
    Assert-Only $start $menu 'Esc on the menu of the profiles'
    Send-Press $VK_Z
    $keys = (Invoke-Step 'read the keys') -replace '^keys ', ''
    if ($keys -ne '17+ 16+ 16- 17- 90+ 90-') {
        throw "The program got the keys '$keys' instead of '17+ 16+ 16- 17- 90+ 90-': Ctrl and Shift, which went down for it before the menu opened, and Z, after the menu closed."
    }
    if (@(Get-Shells).Count -ne 1) {
        throw 'The keys on the menu of the profiles opened a tab.'
    }
    Write-Host "OK: Ctrl+Shift+Space opens the menu of the profiles below the buttons + and of the profiles ($menuWidth x $($sizes.Height) pixels, with cells $cell pixels wide), and draws only the menu: $(($listed | ForEach-Object Name) -join ', '); while Space repeats, nothing opens; Down draws only the highlights of the first two items, and Esc closes the menu; the program gets Ctrl and Shift, which went down for it, and no other key of the menu."

    # 4. The button of the profiles, with the mouse: the pointer on it, and
    # its press, draw only the button, which takes the mouse; when the left
    # button goes up on it, the menu opens below the buttons, and the window
    # lets the mouse go. A click on Probe opens a tab with it.
    $point = Get-Center $bar.Profiles
    $start = @(Get-Log).Count
    Send-Mouse @(
        Get-Mouse $WM_MOUSEMOVE 0 $point
        Get-Mouse $WM_LBUTTONDOWN $MK_LBUTTON $point
    )
    Wait-For { (Get-Capture) -ne [IntPtr]::Zero } { 'A press on the button of the profiles did not capture the mouse.' }
    $null = Assert-Inside $start $bar.Profiles 'The pointer on the button of the profiles, and its press'
    $start = @(Get-Log).Count
    Send-Mouse (Get-Mouse $WM_LBUTTONUP 0 $point)
    Wait-Menu $start $menu 'The button of the profiles'
    Wait-Quiet 400
    $frames = @(Get-Frames $start)
    $drawn = Get-Union @(Get-Areas $frames)
    $expected = Get-Union @($bar.Profiles, $menu)
    if ((Show-Rect $drawn) -ne (Show-Rect $expected)) {
        Show-Lines $frames
        throw "The button of the profiles drew up to $(Show-Rect $drawn), instead of only itself and the menu, $(Show-Rect $expected)."
    }
    Wait-For { (Get-Capture) -eq [IntPtr]::Zero } { 'The window kept the mouse after the button of the profiles opened the menu.' }
    $start = @(Get-Log).Count
    Send-Mouse (Get-Mouse $WM_MOUSEMOVE 0 $entries[1])
    $null = Assert-Inside $start $entries[1].Highlight 'The pointer on the second item of the menu of the profiles'
    $start = @(Get-Log).Count
    Send-Click $entries[-1]
    $shell = Wait-NewShell @($first) 'A click on Probe'
    Assert-Launch $shell $probeProfile 'A click on Probe'
    $copy = [int]$shell.ProcessId
    Wait-Shown $copy @($first) 'After a click on Probe'
    Wait-For { Test-Path -LiteralPath "$reports.$copy" } { 'After a click on Probe, the script of Probe did not start.' }
    Assert-OneFull $start 'A click on Probe'
    Wait-For { (Get-Capture) -eq [IntPtr]::Zero } { 'The window kept the mouse after a click on an item of the menu of the profiles.' }
    Send-Shortcut $VK_W
    Wait-Shown $first @() 'Ctrl+Shift+W on the tab of Probe'
    Send-ModifiersUp
    Wait-Ended $copy 'The program of the tab of Probe, after Ctrl+Shift+W'
    Write-Host 'OK: the pointer on the button of the profiles, and its press, draw only the button; when the left button goes up on it, the menu opens, only the button and the menu are drawn, and the window lets the mouse go; the pointer on an item draws only its highlight, and a click on Probe opens a tab, shown with one full frame, whose program runs the command line of Probe in its folder; Ctrl+Shift+W closes it, and its program ends.'

    # 5. Ctrl+Shift+Space, Down to each profile that meiga finds, and Enter
    # open it, in a tab of its own. Once its shell is idle, or has had some
    # time, Ctrl+Shift+W closes the tab.
    $opened = @()
    for ($index = 0; $index -lt $listed.Count - 1; $index++) {
        $launch = $listed[$index]
        $what = "Enter on $($launch.Name)"
        Wait-Quiet 400
        $start = @(Get-Log).Count
        Send-Shortcut $VK_SPACE
        Wait-Menu $start $menu "Ctrl+Shift+Space, before $what,"
        $messages = @(Get-Key $VK_SHIFT $false; Get-Key $VK_CONTROL $false)
        for ($down = 0; $down -lt $index; $down++) {
            $messages += @(Get-Key $VK_DOWN $true) + @(Get-Key $VK_DOWN $false)
        }
        $messages += @(Get-Key $VK_RETURN $true) + @(Get-Key $VK_RETURN $false)
        Send-Keys $messages
        $shell = Wait-NewShell @($first) $what
        Assert-Launch $shell $launch $what
        $id = [int]$shell.ProcessId
        Wait-Shown $id @($first) "After $what"
        Wait-Settled $id 10
        Send-Shortcut $VK_W
        Wait-Shown $first @() "Ctrl+Shift+W on the tab of $($launch.Name)"
        Send-ModifiersUp
        Wait-Ended $id "The program of $($launch.Name), after Ctrl+Shift+W," $shellSeconds
        $opened += $launch.Name
    }
    Write-Host "OK: Ctrl+Shift+Space, Down and Enter open $($opened -join ', '), each in a tab whose program runs the command line that meiga finds for it, in the home folder; Ctrl+Shift+W closes it, and its program ends."

    # 6. The button + opens a tab with the default profile, Probe; and
    # Duplicate tab, on that tab, opens another one with the same command
    # line, in the same folder.
    Wait-Quiet 400
    $start = @(Get-Log).Count
    Send-Click (Get-Center $bar.New)
    $shell = Wait-NewShell @($first) 'A click on the button +'
    Assert-Launch $shell $probeProfile 'A click on the button +'
    $copy = [int]$shell.ProcessId
    Wait-Shown $copy @($first) 'After a click on the button +'
    Wait-For { Test-Path -LiteralPath "$reports.$copy" } { 'After a click on the button +, the script of Probe did not start.' }
    Assert-OneFull $start 'A click on the button +'
    # Duplicate tab is the second item, after Rename tab.
    Send-Press $VK_APPS
    Wait-Quiet 400
    Send-Press $VK_DOWN
    Send-Press $VK_RETURN
    $shell = Wait-NewShell @($first, $copy) 'Duplicate tab, on the tab of Probe'
    Assert-Launch $shell $probeProfile 'Duplicate tab, on the tab of Probe'
    $duplicate = [int]$shell.ProcessId
    Wait-Shown $duplicate @($first, $copy) 'After Duplicate tab'
    Wait-For { Test-Path -LiteralPath "$reports.$duplicate" } { 'After Duplicate tab, the script of Probe did not start.' }
    Send-Shortcut @($VK_W, $VK_W)
    Wait-Shown $first @() 'Ctrl+Shift+W twice'
    Send-ModifiersUp
    Wait-Ended $duplicate 'The program of the copy of the tab, after Ctrl+Shift+W'
    Wait-Ended $copy 'The program of the tab of the button +, after Ctrl+Shift+W'
    Write-Host 'OK: the button + opens a tab with Probe, the default profile, and Duplicate tab copies it, with the same command line, in the same folder.'

    # 7. The window closes: meiga ends, and the programs of its tabs and
    # their consoles too. The sessions of the shells of the user, which
    # their tabs closed, may still be ending.
    $null = $api::PostMessageW($window, $WM_CLOSE, [UIntPtr]::Zero, [IntPtr]::Zero)
    if (-not $process.WaitForExit($shellSeconds * 1000)) {
        throw "$exe was still running $shellSeconds s after its window closed."
    }
    if ($process.ExitCode -ne 0) {
        throw "$exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Wait-Ended $first 'The program of the first tab'
    Wait-ConsolesEnded
    Write-Host 'OK: it ends with exit code 0 when its window closes, and the programs of its tabs and their consoles end too.'

    # 8. A config.toml that is not TOML: meiga ignores all of it, and the
    # message box says so, and where the file is wrong. The first tab runs
    # the default shell, in the home folder, and not the default profile
    # that the file says.
    Split-Input
    $process.Dispose()
    $process = $null
    $brokenLines = @(
        'default_profile = "Command Prompt"'
        '[[profiles]]'
        'hidden = tru'
    )
    [IO.File]::WriteAllText($config, ($brokenLines -join "`n") + "`n", $utf8)
    $default = @($detected | Where-Object Name -CEQ 'PowerShell') + @($detected | Where-Object Name -CEQ 'Windows PowerShell') +
    $detected | Select-Object -First 1
    $log = Join-Path $work 'frames-ignored.csv'
    $info.ArgumentList[1] = $log
    $process = [Diagnostics.Process]::Start($info)
    $started = $process.StartTime
    Wait-For { $api::WindowOf($process.Id, 'meiga') -ne [IntPtr]::Zero } { "$exe did not open its window." }
    $window = $api::WindowOf($process.Id, 'meiga')
    $thread = $api::GetWindowThreadProcessId($window, [ref]$processId)
    $dialog = Get-Dialog
    # Where the value that is not true begins.
    $head = "There are errors in $($config):`n`nLine 3, column 10: "
    $tail = "`n`nmeiga ignored the file, and uses its default values."
    if (-not $dialog.Text.StartsWith($head, [StringComparison]::Ordinal) -or -not $dialog.Text.EndsWith($tail, [StringComparison]::Ordinal)) {
        throw "With a config.toml that is not TOML, the message box says:`n$($dialog.Text)`n... and it has to start with:`n$head`n... and end with:$tail"
    }
    Wait-For { @(Get-Shells).Count -gt 0 } { 'With config.toml ignored, the first tab did not start its program.' }
    $shells = @(Get-Shells)
    if ($shells.Count -ne 1) {
        throw "With config.toml ignored, $exe started $($shells.Count) programs instead of one."
    }
    Assert-Launch $shells[0] $default 'With config.toml ignored, the first tab'
    $id = [int]$shells[0].ProcessId
    Close-Dialog $dialog
    Wait-Settled $id 10
    $null = $api::PostMessageW($window, $WM_CLOSE, [UIntPtr]::Zero, [IntPtr]::Zero)
    if (-not $process.WaitForExit($shellSeconds * 1000)) {
        throw "With config.toml ignored, $exe was still running $shellSeconds s after its window closed."
    }
    if ($process.ExitCode -ne 0) {
        throw "With config.toml ignored, $exe ended with exit code $($process.ExitCode) instead of 0."
    }
    Wait-Ended $id "The program of the first tab, $($default.Name)," $shellSeconds
    Wait-ConsolesEnded
    Write-Host "OK: with a config.toml that is not TOML, the message box tells where it is wrong, and that meiga ignored the file; the first tab runs the default shell, $($default.Name), in the home folder, and it ends with exit code 0 when its window closes."
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
    if ($passed) {
        Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $work) {
        Write-Host "Kept $work"
    }
}
