//! The main window, its frame timer and the message loop.

use std::cell::{Cell, RefCell};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use std::{io, mem, ptr};

use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::mouse::{Button, ClickSettings, Modifiers};
use meiga_core::ui::titlebar::{Hit, TitleBar, min_client};
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_CLASS_ALREADY_EXISTS, HANDLE, HWND, LPARAM, LRESULT, POINT, RECT,
    WAIT_FAILED, WAIT_OBJECT_0, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetMonitorInfoW, InvalidateRect, MONITOR_DEFAULTTONEAREST, MONITORINFO,
    MonitorFromWindow, PAINTSTRUCT, ScreenToClient, UpdateWindow, ValidateRect,
};
use windows_sys::Win32::System::LibraryLoader::{
    GetModuleHandleW, GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};
use windows_sys::Win32::System::Threading::{
    CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CancelWaitableTimer, CreateWaitableTimerExW, INFINITE,
    SetWaitableTimer, TIMER_ALL_ACCESS,
};
use windows_sys::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetCapture, GetDoubleClickTime, GetKeyState, ReleaseCapture, SetCapture, TME_LEAVE,
    TME_NONCLIENT, TRACKMOUSEEVENT, TrackMouseEvent, VK_CONTROL, VK_MENU, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    EnableMenuItem, GWLP_USERDATA, GetCaretBlinkTime, GetClientRect, GetCursorPos,
    GetForegroundWindow, GetSystemMenu, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect,
    HCURSOR, HTCAPTION, HTCLIENT, HTCLOSE, HTMAXBUTTON, HTMINBUTTON, HTTOP, HTTOPLEFT, IDC_ARROW,
    IDC_IBEAM, IDOK, IsIconic, IsWindow, IsZoomed, KillTimer, LoadCursorW, MB_DEFBUTTON2,
    MB_ICONERROR, MB_ICONWARNING, MB_OK, MB_OKCANCEL, MF_BYCOMMAND, MF_ENABLED, MF_GRAYED,
    MINMAXINFO, MSG, MWMO_INPUTAVAILABLE, MessageBoxW, MsgWaitForMultipleObjectsEx, PM_REMOVE,
    PeekMessageW, PostMessageW, PostQuitMessage, QS_ALLINPUT, RegisterClassExW, SC_CLOSE,
    SC_MAXIMIZE, SC_MINIMIZE, SC_MOVE, SC_RESTORE, SC_SIZE, SM_CARETBLINKINGENABLED,
    SM_CXDOUBLECLK, SM_CXPADDEDBORDER, SM_CYDOUBLECLK, SM_CYSIZEFRAME, SM_REMOTESESSION,
    SPI_GETCARETTIMEOUT, SPI_GETCARETWIDTH, SPI_GETWHEELSCROLLCHARS, SPI_GETWHEELSCROLLLINES,
    SW_SHOWDEFAULT, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SetCursor, SetMenuDefaultItem, SetTimer, SetWindowLongPtrW, SetWindowPos, SetWindowTextW,
    ShowWindow, SystemParametersInfoW, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu,
    TranslateMessage, USER_DEFAULT_SCREEN_DPI, USER_TIMER_MAXIMUM, USER_TIMER_MINIMUM, WM_CLOSE,
    WM_DESTROY, WM_DPICHANGED, WM_ENTERMENULOOP, WM_ENTERSIZEMOVE, WM_ERASEBKGND, WM_EXITMENULOOP,
    WM_EXITSIZEMOVE, WM_GETMINMAXINFO, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEMOVE, WM_NCCALCSIZE, WM_NCHITTEST, WM_NCMBUTTONDBLCLK, WM_NCMOUSELEAVE, WM_NCMOUSEMOVE,
    WM_PAINT, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETCURSOR, WM_SYSCOMMAND, WM_TIMER,
    WNDCLASSEXW, WS_OVERLAPPEDWINDOW, WindowFromPoint,
};

use crate::event::{
    Event, Handler, HandlerError, NonClient, WM_MOUSELEAVE, WM_WAKE, decode, decode_nonclient,
    point,
};
use crate::keyboard::{Destination, Keyboard, is_keyboard_message};
use crate::{Error, wide};

/// Name of the window class of meiga windows.
const CLASS_NAME: &str = "meiga";

/// Identifier of the window timer that replaces the frame timer while
/// Windows runs its own message loop.
const FRAME_TIMER_ID: usize = 1;

/// The longest time that a caret can stay shown, and then hidden, while it
/// blinks, in milliseconds. Windows Terminal does not make it blink with
/// more: Windows lets the user choose between 200 and 1 200 ms, and
/// `GetCaretBlinkTime` gives `INFINITE` when the caret does not blink.
const MAX_BLINK_MS: u32 = 10_000;

/// How long the caret of Windows blinks before it stops, if Windows does not
/// say: its default value.
const DEFAULT_CARET_TIMEOUT: Duration = Duration::from_secs(5);

/// Where the flag of the title bar of meiga ([`WindowOptions::title_bar`])
/// is, in the extra bytes of each window of the class: the window procedure
/// reads it also before [`Window::run`], while Windows creates the window
/// and asks for its frame.
const CUSTOM_FRAME: i32 = 0;

/// How far from its left side the top edge of a window resizes it also to
/// the left, in pixels at any scale: as on a window of Windows 11 with the
/// title bar of Windows, which does not scale it.
const TOP_LEFT_CORNER: i32 = 19;

/// The pixels that a maximized window leaves free on an edge of its monitor
/// with a taskbar that hides itself, so that the pointer can show it, as in
/// Windows Terminal.
const AUTOHIDE_TASKBAR: i32 = 2;

/// How a new window looks.
pub struct WindowOptions<'a> {
    /// The text of the window, which the taskbar shows, and the title bar of
    /// Windows, if the window has it.
    pub title: &'a str,
    /// Whether the window has the title bar of meiga: its client area goes
    /// up to the top of the window, where the title bar of Windows would
    /// be, and the handler draws it, and tells the window what each part of
    /// it is with [`Window::set_title_bar`]. The window keeps the frame of
    /// Windows on its other sides, which resizes it. Without it, the window
    /// has the frame and the title bar of Windows.
    pub title_bar: bool,
}

/// The shape of the mouse pointer over the client area of the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pointer {
    /// The arrow: the clicks go to the program in the terminal.
    Arrow,
    /// The I-beam of text: the clicks select text.
    Text,
}

/// A top-level window.
///
/// It belongs to the thread that creates it. It lives until the user closes
/// it, which ends [`Window::run`], or until it is dropped. It paints nothing
/// by itself: its [`Handler`] paints every pixel.
pub struct Window {
    handle: HWND,
    timer: FrameTimer,
    arrow: HCURSOR,
    text: HCURSOR,
    /// The pointer over the client area.
    pointer: Cell<HCURSOR>,
    /// Whether the window has the title bar of meiga.
    custom: bool,
    /// What each part of the title bar of meiga is, from the handler.
    title_bar: Cell<Option<TitleBar>>,
    /// What the pointer was over for Windows in the last mouse message: its
    /// answer to `WM_NCHITTEST`, `HTCLIENT` over the client area.
    hit: Cell<u32>,
    /// Whether the window watches for the pointer to leave it.
    leave: Cell<Leave>,
    /// Whether the window asked Windows again to tell when the pointer
    /// leaves, after it went from a part of the window to another, since the
    /// pointer last moved.
    retracked: Cell<bool>,
}

/// Whether the window watches for the pointer to leave it, for
/// [`Event::MouseLeft`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Leave {
    /// It does not: the handler did not ask, or the event already came.
    Off,
    /// Windows tells it when the pointer leaves the part of the window where
    /// it is: the client area, or the title bar of meiga or its buttons,
    /// which are not of the client area for Windows.
    Armed,
    /// The pointer went from one of those parts to another, which are all
    /// of the handler: the window asks Windows again when the pointer moves.
    Rearm,
}

impl Window {
    /// Registers the window class, unless this process did, and creates the
    /// window, still hidden.
    pub fn create(options: &WindowOptions<'_>) -> Result<Self, Error> {
        let timer = FrameTimer::new()?;

        // SAFETY: a null name returns the handle of the executable of this
        // process, which stays loaded while the process runs.
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        if instance.is_null() {
            return Err(Error::last("GetModuleHandleW"));
        }

        // SAFETY: both are predefined system cursors, which are loaded
        // without an instance handle, and are never destroyed.
        let (arrow, text) = unsafe {
            (
                LoadCursorW(ptr::null_mut(), IDC_ARROW),
                LoadCursorW(ptr::null_mut(), IDC_IBEAM),
            )
        };
        if arrow.is_null() || text.is_null() {
            return Err(Error::last("LoadCursorW"));
        }

        // No background brush: the handler paints the whole client area in
        // WM_PAINT, so erasing it first would only paint it twice.
        let class_name = wide(CLASS_NAME);
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_proc),
            // The flag of the title bar of meiga (CUSTOM_FRAME).
            cbWndExtra: size_of::<isize>() as i32,
            hInstance: instance,
            hCursor: arrow,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };
        // SAFETY: `class` is fully initialised and its strings are
        // null-terminated and outlive the call, which copies them.
        if unsafe { RegisterClassExW(&class) } == 0 {
            // Another window of this process registered it: the same class.
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(ERROR_CLASS_ALREADY_EXISTS as i32) {
                return Err(Error::new("RegisterClassExW", error));
            }
        }

        let title = wide(options.title);
        // SAFETY: the class is registered, both strings are null-terminated
        // and outlive the call, and the window has no parent, menu or
        // creation data.
        let handle = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null(),
            )
        };
        if handle.is_null() {
            return Err(Error::last("CreateWindowExW"));
        }
        let custom = options.title_bar && own_title_bar(handle);
        Ok(Self {
            handle,
            timer,
            arrow,
            text,
            pointer: Cell::new(arrow),
            custom,
            title_bar: Cell::new(None),
            hit: Cell::new(HTCLIENT),
            leave: Cell::new(Leave::Off),
            retracked: Cell::new(false),
        })
    }

    /// The Windows handle of the window, for the functions that draw in it.
    pub const fn handle(&self) -> HWND {
        self.handle
    }

    /// The size of the client area: the window without the frame and the
    /// title bar of Windows.
    pub fn client_size(&self) -> Result<Size, Error> {
        let mut rect = RECT::default();
        // SAFETY: `rect` is a valid RECT for GetClientRect to fill in.
        if unsafe { GetClientRect(self.handle, &mut rect) } == 0 {
            return Err(Error::last("GetClientRect"));
        }
        // The client rectangle starts at (0, 0).
        Ok(Size::new(
            rect.right.max(0) as u32,
            rect.bottom.max(0) as u32,
        ))
    }

    /// The dots per inch of the monitor of the window: 96 at a scale of
    /// 100 %, 120 at 125 % and 144 at 150 %.
    pub fn dpi(&self) -> u32 {
        window_dpi(self.handle)
    }

    /// Whether the window is minimized.
    pub fn is_minimized(&self) -> bool {
        // SAFETY: IsIconic accepts any handle.
        unsafe { IsIconic(self.handle) != 0 }
    }

    /// Whether the window is maximized.
    pub fn is_maximized(&self) -> bool {
        // SAFETY: IsZoomed accepts any handle.
        unsafe { IsZoomed(self.handle) != 0 }
    }

    /// Whether the window has the title bar of meiga
    /// ([`WindowOptions::title_bar`]).
    pub const fn has_title_bar(&self) -> bool {
        self.custom
    }

    /// How many pixels at the top of the client area, at `dpi`, resize the
    /// window from its top edge: as many as its frame, while it has the
    /// title bar of meiga and is not maximized, or else none.
    pub fn resize_band(&self, dpi: u32) -> u32 {
        if self.custom && !self.is_maximized() {
            frame(dpi)
        } else {
            0
        }
    }

    /// Tells the window what each part of its title bar is, from the next
    /// time that Windows asks: what drags the window, what resizes it, and
    /// its buttons. Without the title bar of meiga, Windows never asks.
    pub fn set_title_bar(&self, bar: TitleBar) {
        self.title_bar.set(Some(bar));
    }

    /// Minimizes the window, as the button of its title bar does, once the
    /// handler returns.
    pub fn minimize(&self) {
        self.command(SC_MINIMIZE);
    }

    /// Maximizes the window, or restores it if it is maximized, as the button
    /// of its title bar does, once the handler returns.
    pub fn toggle_maximize(&self) {
        self.command(if self.is_maximized() {
            SC_RESTORE
        } else {
            SC_MAXIMIZE
        });
    }

    /// Shows the window, paints it, tells `handler` with [`Event::Shown`],
    /// and delivers its events to it until the window closes.
    ///
    /// While nothing happens, the thread sleeps inside
    /// `MsgWaitForMultipleObjectsEx`, waiting for a message or for the frame
    /// timer, so an idle window uses no CPU. If the handler returns an error,
    /// the window is destroyed, and the error is returned.
    pub fn run(&self, handler: &mut dyn Handler) -> Result<(), HandlerError> {
        let state = State {
            window: self,
            handler: RefCell::new(handler),
            failure: RefCell::new(None),
            keyboard: Keyboard::default(),
        };
        let _installed = Installed::new(self.handle, &state);

        // SW_SHOWDEFAULT honours how the launcher asked to start the program
        // (minimised or maximised from a shortcut, for example). The return
        // value is the previous visibility, not an error.
        // SAFETY: `self.handle` is a window of this thread.
        unsafe { ShowWindow(self.handle, SW_SHOWDEFAULT) };
        // Paint it before waiting for messages, so that the first frame is
        // on the screen as soon as possible.
        // SAFETY: as above.
        let result = if unsafe { UpdateWindow(self.handle) } == 0 {
            Err(Error::last("UpdateWindow"))
        } else {
            state.dispatch(Event::Shown);
            self.pump(&state)
        };
        if let Some(failure) = state.failure.take() {
            return Err(failure);
        }
        result.map_err(Into::into)
    }

    /// Asks for [`Event::FrameTimer`] after `delay`, instead of the one asked
    /// before, if any.
    pub fn start_frame_timer(&self, delay: Duration) -> Result<(), Error> {
        let due_time = due_time(delay);
        // SAFETY: the timer handle is valid while `self` exists, and
        // `due_time` outlives the call. Without a period or a completion
        // routine, the timer is signalled once. Setting it again also
        // clears the signal of the previous time, if it was not waited for.
        let started =
            unsafe { SetWaitableTimer(self.timer.handle, &due_time, 0, None, ptr::null(), 0) };
        if started == 0 {
            return Err(Error::last("SetWaitableTimer"));
        }
        let now = Instant::now();
        self.timer
            .due
            .set(Some(now.checked_add(delay).unwrap_or(now)));
        if self.timer.modal.get() {
            self.start_window_timer(delay)?;
        }
        Ok(())
    }

    /// Cancels the frame timer, if it was started.
    pub fn stop_frame_timer(&self) {
        if self.timer.due.take().is_some() {
            // SAFETY: the timer handle is valid while `self` exists, and
            // KillTimer accepts a timer that does not exist.
            unsafe {
                CancelWaitableTimer(self.timer.handle);
                KillTimer(self.handle, FRAME_TIMER_ID);
            }
        }
    }

    /// Sends all the mouse input to this window, also when the mouse leaves
    /// it, until [`Window::release_mouse`]. For dragging.
    pub fn capture_mouse(&self) {
        // SAFETY: the window is of this thread. The return value is the
        // window that had the capture before.
        unsafe { SetCapture(self.handle) };
    }

    /// Ends [`Window::capture_mouse`].
    pub fn release_mouse(&self) {
        // SAFETY: GetCapture and ReleaseCapture have no preconditions.
        unsafe {
            if GetCapture() == self.handle {
                ReleaseCapture();
            }
        }
    }

    /// Asks for [`Event::MouseLeft`] when the pointer leaves the client
    /// area, once. With the title bar of meiga, which is of the client area
    /// for the handler, also its buttons, it comes when the pointer leaves
    /// the window, or goes to the frame that resizes it.
    pub fn track_mouse_leave(&self) {
        self.leave.set(Leave::Armed);
        self.track(self.hit.get());
    }

    /// Asks Windows to tell when the pointer leaves the part of the window
    /// where it is for Windows, `hit`: the client area, or another part. If
    /// it is elsewhere, Windows tells it at once.
    fn track(&self, hit: u32) {
        let mut flags = TME_LEAVE;
        if hit != HTCLIENT {
            flags |= TME_NONCLIENT;
        }
        let mut track = TRACKMOUSEEVENT {
            cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: flags,
            hwndTrack: self.handle,
            dwHoverTime: 0,
        };
        // SAFETY: `track` is a valid TRACKMOUSEEVENT for a window of this
        // thread. If it fails, the event does not come, and the pointer
        // only looks over the window for longer.
        unsafe { TrackMouseEvent(&mut track) };
    }

    /// The pointer moved over the window: if it went from a part of the
    /// window to another, asks Windows again to tell when it leaves, now
    /// that the window knows where it is. Not while the window has the
    /// mouse: the handler asks again when it lets the mouse go.
    fn rearm(&self) {
        // SAFETY: GetCapture has no preconditions.
        if self.leave.get() == Leave::Rearm && unsafe { GetCapture() } != self.handle {
            self.leave.set(Leave::Armed);
            self.track(self.hit.get());
        }
    }

    /// Windows says that the pointer left the client area, or another part
    /// of the window.
    fn mouse_left(&self, state: &State<'_>) {
        match self.leave.get() {
            Leave::Off => {}
            // It went between the client area and the title bar of meiga,
            // which are both of the handler. The move that took it there
            // comes after this message, which Windows posted: until then,
            // the window does not know where it is for Windows. It asks again
            // once, which is right if the move already came, and else waits
            // for the move: asking with the wrong part, Windows would tell
            // it at once, again and again, before the move.
            _ if self.custom && self.pointer_inside() => {
                if self.retracked.replace(true) {
                    self.leave.set(Leave::Rearm);
                } else {
                    self.leave.set(Leave::Armed);
                    self.track(self.hit.get());
                }
            }
            _ => {
                self.leave.set(Leave::Off);
                state.dispatch(Event::MouseLeft);
            }
        }
    }

    /// Whether the pointer is over the client area of the window, and not
    /// over another window in front of it. If Windows cannot tell where the
    /// pointer is, as while the client of Remote Desktop is minimized, it
    /// is not.
    fn pointer_inside(&self) -> bool {
        let mut cursor = POINT::default();
        // SAFETY: `cursor` is a valid POINT for GetCursorPos to fill in.
        if unsafe { GetCursorPos(&mut cursor) } == 0 {
            return false;
        }
        // SAFETY: WindowFromPoint has no preconditions.
        (unsafe { WindowFromPoint(cursor) } == self.handle)
            && self.in_client_area(self.client_point(Point::new(cursor.x, cursor.y)))
    }

    /// Whether `at`, a point from the top-left corner of the client area,
    /// is in it.
    fn in_client_area(&self, at: Point) -> bool {
        self.client_size()
            .is_ok_and(|size| Rect::from_size(size).contains_point(at))
    }

    /// Whether the window is the one that the user is using: the
    /// foreground window. A window that a program starts in the background
    /// can have the focus of its thread without being it.
    pub fn is_foreground(&self) -> bool {
        // SAFETY: GetForegroundWindow has no preconditions.
        unsafe { GetForegroundWindow() == self.handle }
    }

    /// Asks Windows for a `WM_PAINT` of `area`, or of the whole window.
    pub fn invalidate(&self, area: Option<Rect>) {
        let rect = area.map(|area| RECT {
            left: area.left,
            top: area.top,
            right: area.right,
            bottom: area.bottom,
        });
        let rect = rect.as_ref().map_or(ptr::null(), ptr::from_ref);
        // SAFETY: the RECT, if there is one, outlives the call. FALSE: do not
        // erase the background first.
        unsafe { InvalidateRect(self.handle, rect, 0) };
    }

    /// Changes the text of the title bar, which the taskbar also shows.
    pub fn set_title(&self, title: &str) {
        let title = wide(title);
        // SAFETY: the window is of this thread, and the string is
        // null-terminated and outlives the call. Windows sends WM_SETTEXT to
        // the window, which DefWindowProcW handles without the handler, so
        // it can be called from the handler.
        unsafe { SetWindowTextW(self.handle, title.as_ptr()) };
    }

    /// Closes the window as Alt+F4 does, once the handler returns: the
    /// message loop of [`Window::run`] ends.
    pub fn close(&self) {
        // SAFETY: PostMessageW accepts any handle, and the message has no
        // parameters. DefWindowProcW destroys the window when it gets it.
        unsafe { PostMessageW(self.handle, WM_CLOSE, 0, 0) };
    }

    /// Changes the pointer over the client area, from the next time that
    /// the mouse moves.
    pub fn set_pointer(&self, pointer: Pointer) {
        self.pointer.set(match pointer {
            Pointer::Arrow => self.arrow,
            Pointer::Text => self.text,
        });
    }

    /// Asks the user to confirm `text` in a message box with OK and Cancel,
    /// in front of the window, with Cancel chosen. Returns whether the user
    /// chose OK.
    ///
    /// While the box is open, the window keeps receiving messages, but its
    /// events do not reach the handler, which is waiting for this call: a
    /// [`Event::Wake`] is lost, for example, or a new size or scale of the
    /// window.
    pub fn confirm(&self, title: &str, text: &str) -> bool {
        let title = wide(title);
        let text = wide(text);
        // SAFETY: the window is of this thread, and both strings are
        // null-terminated and outlive the call.
        let chosen = unsafe {
            MessageBoxW(
                self.handle,
                text.as_ptr(),
                title.as_ptr(),
                MB_OKCANCEL | MB_ICONWARNING | MB_DEFBUTTON2,
            )
        };
        chosen == IDOK
    }

    /// Shows `text` in a message box with OK, in front of the window, and
    /// waits until the user closes it. As with [`Window::confirm`], the
    /// events of the window do not reach the handler meanwhile.
    pub fn alert(&self, title: &str, text: &str) {
        let title = wide(title);
        let text = wide(text);
        // SAFETY: the window is of this thread, and both strings are
        // null-terminated and outlive the call.
        unsafe {
            MessageBoxW(
                self.handle,
                text.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONWARNING,
            )
        };
    }

    /// What wakes this window up from another thread.
    pub fn waker(&self) -> Waker {
        Waker(self.handle as usize)
    }

    /// Dispatches the messages of this thread until WM_QUIT. Between them,
    /// it sleeps until another message arrives or the frame timer is due.
    fn pump(&self, state: &State<'_>) -> Result<(), Error> {
        let mut message = MSG::default();
        loop {
            // SAFETY: `message` is a valid MSG for PeekMessageW to fill in.
            while unsafe { PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if message.message == WM_QUIT {
                    return Ok(());
                }
                // SAFETY: `message` holds the message that PeekMessageW has
                // just retrieved.
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            // SAFETY: the array has one handle, valid while `self` exists.
            // MWMO_INPUTAVAILABLE also wakes for input that arrived before
            // the call.
            let woke = unsafe {
                MsgWaitForMultipleObjectsEx(
                    1,
                    &self.timer.handle,
                    INFINITE,
                    QS_ALLINPUT,
                    MWMO_INPUTAVAILABLE,
                )
            };
            if woke == WAIT_OBJECT_0 {
                self.frame_timer_due(state);
            } else if woke == WAIT_FAILED {
                return Err(Error::last("MsgWaitForMultipleObjectsEx"));
            }
        }
    }

    /// The frame timer, or the window timer that replaces it, is due.
    fn frame_timer_due(&self, state: &State<'_>) {
        // The other of the two timers already delivered it, or it was
        // stopped after it was signalled: CancelWaitableTimer does not clear
        // the signal.
        if self.timer.due.take().is_none() {
            return;
        }
        // SAFETY: the timer handle is valid while `self` exists, and
        // KillTimer accepts a timer that does not exist.
        unsafe {
            CancelWaitableTimer(self.timer.handle);
            KillTimer(self.handle, FRAME_TIMER_ID);
        }
        state.dispatch(Event::FrameTimer);
    }

    /// Windows starts a message loop of its own, which does not wait for the
    /// frame timer: a window timer replaces it until the loop ends.
    fn enter_modal_loop(&self) {
        self.timer.modal.set(true);
        if let Some(due) = self.timer.due.get() {
            // If it fails, the frame waits until the loop of Windows ends.
            let _ = self.start_window_timer(due.saturating_duration_since(Instant::now()));
        }
    }

    fn leave_modal_loop(&self) {
        self.timer.modal.set(false);
        // SAFETY: KillTimer accepts a timer that does not exist.
        unsafe { KillTimer(self.handle, FRAME_TIMER_ID) };
    }

    fn start_window_timer(&self, delay: Duration) -> Result<(), Error> {
        // SAFETY: the window is of this thread, and without a callback the
        // timer posts WM_TIMER.
        if unsafe { SetTimer(self.handle, FRAME_TIMER_ID, timer_millis(delay), None) } == 0 {
            return Err(Error::last("SetTimer"));
        }
        Ok(())
    }

    /// The point of the client area at `on_screen`, a point from the
    /// top-left corner of the screen.
    fn client_point(&self, on_screen: Point) -> Point {
        let mut point = POINT {
            x: on_screen.x,
            y: on_screen.y,
        };
        // SAFETY: the window is of this thread, and `point` is a valid
        // POINT for ScreenToClient to change. If it fails, the point stays.
        unsafe { ScreenToClient(self.handle, &mut point) };
        Point::new(point.x, point.y)
    }

    /// Asks Windows for a command of the menu of the window, such as
    /// `SC_MINIMIZE`, once the handler returns.
    fn command(&self, command: u32) {
        // SAFETY: PostMessageW accepts any handle. DefWindowProcW does the
        // command when it gets the message.
        unsafe { PostMessageW(self.handle, WM_SYSCOMMAND, command as WPARAM, 0) };
    }

    /// What the point `on_screen` of the client area is for Windows, with
    /// the title bar of meiga: its answer to `WM_NCHITTEST`.
    fn hit_test(&self, on_screen: Point) -> u32 {
        let Some(bar) = self.title_bar.get() else {
            return HTCLIENT;
        };
        let at = self.client_point(on_screen);
        // The frame on the left of the client area: the top edge resizes the
        // window also to the left as far from the side of the window as on a
        // window with the title bar of Windows.
        let mut rect = RECT::default();
        // SAFETY: `rect` is a valid RECT for GetWindowRect to fill in.
        let left = if unsafe { GetWindowRect(self.handle, &mut rect) } == 0 {
            0
        } else {
            on_screen.x - at.x - rect.left
        };
        match bar.hit(at, TOP_LEFT_CORNER - left) {
            Hit::Client => HTCLIENT,
            Hit::Caption => HTCAPTION,
            Hit::Top => HTTOP,
            Hit::TopLeft => HTTOPLEFT,
            Hit::Minimize => HTMINBUTTON,
            Hit::Maximize => HTMAXBUTTON,
            Hit::Close => HTCLOSE,
        }
    }

    /// A mouse message of a part of the window that is not of its client
    /// area for Windows, with the title bar of meiga: the title bar, its
    /// buttons, the band that resizes the window from its top edge and the
    /// frame. Returns whether Windows does what it does with the message
    /// too: drag the window, resize it, or maximize it with a double click.
    fn nonclient_mouse(&self, state: &State<'_>, mouse: NonClient) -> bool {
        self.hit.set(mouse.hit);
        let button = matches!(mouse.hit, HTMINBUTTON | HTMAXBUTTON | HTCLOSE);
        match mouse.event {
            Event::MouseMoved { at, modifiers, .. } => {
                self.retracked.set(false);
                let at = self.client_point(at);
                if self.in_client_area(at) {
                    self.rearm();
                    state.dispatch(Event::MouseMoved {
                        at,
                        button: None,
                        modifiers,
                    });
                } else if self.leave.get() != Leave::Off {
                    // The frame, which resizes the window, is not of the
                    // handler.
                    self.leave.set(Leave::Off);
                    state.dispatch(Event::MouseLeft);
                }
                // Over its buttons, Windows would draw its own ones: the
                // handler draws them. Windows opens the snap layouts of
                // Windows 11 below the button to maximize anyway, because
                // of WM_NCHITTEST.
                !button
            }
            Event::MouseDown {
                button: pressed,
                at,
                modifiers,
            } if button => {
                state.dispatch(Event::MouseDown {
                    button: pressed,
                    at: self.client_point(at),
                    modifiers,
                });
                false
            }
            Event::MouseUp {
                button: released,
                at,
                modifiers,
            } if button => {
                state.dispatch(Event::MouseUp {
                    button: released,
                    at: self.client_point(at),
                    modifiers,
                });
                false
            }
            // Windows does nothing with the right button on the title bar of
            // meiga: the window opens its menu.
            Event::MouseUp {
                button: Button::Right,
                at,
                ..
            } if mouse.hit == HTCAPTION => {
                self.system_menu(at);
                false
            }
            _ => true,
        }
    }

    /// Opens the menu of the window at `at`, a point from the top-left
    /// corner of the screen, as the title bar of Windows does with the right
    /// button, and does the command that the user chooses.
    ///
    /// The menu runs a message loop of Windows, whose messages reach the
    /// window procedure, and the handler, until it closes.
    fn system_menu(&self, at: Point) {
        // SAFETY: the window is of this thread. FALSE: the menu of the
        // window, and not a copy of the default one.
        let menu = unsafe { GetSystemMenu(self.handle, 0) };
        if menu.is_null() {
            return;
        }
        let maximized = self.is_maximized();
        for (command, enabled) in [
            (SC_RESTORE, maximized),
            (SC_MOVE, !maximized),
            (SC_SIZE, !maximized),
            (SC_MINIMIZE, true),
            (SC_MAXIMIZE, !maximized),
            (SC_CLOSE, true),
        ] {
            let state = if enabled { MF_ENABLED } else { MF_GRAYED };
            // SAFETY: the menu is the one of the window, which lives as long
            // as it. It returns the state before, or -1 without the item.
            unsafe { EnableMenuItem(menu, command, MF_BYCOMMAND | state) };
        }
        // SAFETY: as above. Close is the default item, in bold, as in the
        // menu that Windows opens with Alt+Space.
        unsafe { SetMenuDefaultItem(menu, SC_CLOSE, 0) };
        // SAFETY: the menu and the window are of this thread, and there is no
        // rectangle. TPM_RETURNCMD: it returns the command that the user
        // chose, or 0, instead of sending it.
        let command = unsafe {
            TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                at.x,
                at.y,
                0,
                self.handle,
                ptr::null(),
            )
        };
        if command > 0 {
            self.command(command as u32);
        }
    }

    /// WM_PAINT: the handler paints the area that Windows asks for.
    fn paint(&self, state: &State<'_>) {
        let mut paint = PAINTSTRUCT::default();
        // SAFETY: the window is of this thread, and `paint` is a valid
        // PAINTSTRUCT for BeginPaint to fill in.
        let dc = unsafe { BeginPaint(self.handle, &mut paint) };
        if dc.is_null() {
            // Without a device context nothing can be painted. Validate the
            // window anyway: while it has an area to paint, Windows keeps
            // sending WM_PAINT, and the loop would never sleep.
            // SAFETY: the window is of this thread, and null means all of it.
            unsafe { ValidateRect(self.handle, ptr::null()) };
            return;
        }
        let area = Rect::new(
            paint.rcPaint.left,
            paint.rcPaint.top,
            paint.rcPaint.right,
            paint.rcPaint.bottom,
        );
        let result = state.deliver(|handler, window| handler.paint(window, dc, area));
        // SAFETY: ends the BeginPaint above, with the same PAINTSTRUCT.
        unsafe { EndPaint(self.handle, &paint) };
        state.check(result);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // The window is usually destroyed already, when the user closed it.
        // SAFETY: IsWindow accepts any handle, and the window is of this
        // thread, because a Window cannot move to another one.
        unsafe {
            if IsWindow(self.handle) != 0 {
                DestroyWindow(self.handle);
            }
        }
    }
}

/// Gives `window` the title bar of meiga: its client area goes up to its
/// top, and it keeps the frame of Windows on its other sides. Returns
/// whether it has it.
fn own_title_bar(window: HWND) -> bool {
    // SAFETY: the window is of this thread, and its class has the extra
    // bytes of the flag. It returns the flag before, 0.
    unsafe { SetWindowLongPtrW(window, CUSTOM_FRAME, 1) };
    // Windows asks for the frame again, with WM_NCCALCSIZE, now with the
    // flag.
    // SAFETY: the window is of this thread. It keeps its place, size and
    // order.
    let changed = unsafe {
        SetWindowPos(
            window,
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    } != 0;
    if !changed {
        // SAFETY: as above.
        unsafe { SetWindowLongPtrW(window, CUSTOM_FRAME, 0) };
    }
    changed && custom_frame(window)
}

/// Whether `window`, of the class of meiga, has the title bar of meiga.
fn custom_frame(window: HWND) -> bool {
    // SAFETY: the windows of the class have the extra bytes of the flag,
    // which are 0 until `own_title_bar` sets them.
    unsafe { GetWindowLongPtrW(window, CUSTOM_FRAME) != 0 }
}

/// The dots per inch of the monitor of `window`.
fn window_dpi(window: HWND) -> u32 {
    // SAFETY: GetDpiForWindow accepts any handle, and returns 0 for one that
    // is not a window.
    match unsafe { GetDpiForWindow(window) } {
        0 => USER_DEFAULT_SCREEN_DPI,
        dpi => dpi,
    }
}

/// The frame of Windows on each side of a window at `dpi`, which resizes it,
/// in pixels: 8 at a scale of 100 %, 9 at 125 %, 11 at 150 % and 13 at
/// 200 %. Most of it is invisible.
fn frame(dpi: u32) -> u32 {
    // SAFETY: GetSystemMetricsForDpi has no preconditions, and gives 0 for a
    // metric that it does not know.
    let metric = |index| unsafe { GetSystemMetricsForDpi(index, dpi) };
    u32::try_from(metric(SM_CXPADDEDBORDER).saturating_add(metric(SM_CYSIZEFRAME))).unwrap_or(0)
}

/// What `SHAppBarMessage` takes: the `APPBARDATA` of shellapi.h, which
/// windows-sys has in a feature that meiga does not use.
#[repr(C)]
struct AppBarData {
    size: u32,
    window: HWND,
    callback_message: u32,
    edge: u32,
    rect: RECT,
    lparam: LPARAM,
}

// As in shellapi.h for x64, where its members have their natural alignment.
const _: () = assert!(size_of::<AppBarData>() == 48);

impl AppBarData {
    fn new(edge: u32, rect: RECT) -> Self {
        Self {
            size: size_of::<Self>() as u32,
            window: ptr::null_mut(),
            callback_message: 0,
            edge,
            rect,
            lparam: 0,
        }
    }
}

/// `SHAppBarMessage`.
type AppBarFn = unsafe extern "system" fn(u32, *mut AppBarData) -> usize;

/// The messages of `SHAppBarMessage` that say whether the taskbar hides
/// itself, and whether it does on an edge of a monitor.
const ABM_GETSTATE: u32 = 4;
const ABM_GETAUTOHIDEBAREX: u32 = 11;

/// What `ABM_GETSTATE` gives when the taskbar hides itself.
const ABS_AUTOHIDE: usize = 1;

/// The edges of a monitor.
const ABE_LEFT: u32 = 0;
const ABE_TOP: u32 = 1;
const ABE_RIGHT: u32 = 2;
const ABE_BOTTOM: u32 = 3;

/// `SHAppBarMessage`, in `shell32.dll`, which meiga loads only the first
/// time that a window is maximized, because it takes about a millisecond.
/// `None` if it cannot.
fn app_bar_message() -> Option<AppBarFn> {
    static FUNCTION: OnceLock<Option<AppBarFn>> = OnceLock::new();
    *FUNCTION.get_or_init(|| {
        let name = wide("shell32.dll");
        // SAFETY: the name is null-terminated, and the flag looks for the
        // library, and the ones that it needs, only in System32.
        let module =
            unsafe { LoadLibraryExW(name.as_ptr(), ptr::null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32) };
        if module.is_null() {
            return None;
        }
        // The module is never freed: the function points into it.
        // SAFETY: the module is loaded, and the name is null-terminated.
        let function = unsafe { GetProcAddress(module, c"SHAppBarMessage".as_ptr().cast()) }?;
        // SAFETY: shellapi.h declares SHAppBarMessage with this signature.
        Some(unsafe { mem::transmute::<unsafe extern "system" fn() -> isize, AppBarFn>(function) })
    })
}

/// Maximized, a window covers the whole work area of its monitor, which is
/// the whole monitor on the edges with a taskbar that hides itself: leaves
/// a few pixels of `client` free there, so that the pointer can show the
/// taskbar, as in Windows Terminal.
fn leave_room_for_hidden_taskbars(window: HWND, client: &mut RECT) {
    let Some(app_bar_message) = app_bar_message() else {
        return;
    };
    let mut query = AppBarData::new(0, RECT::default());
    // SAFETY: the function is SHAppBarMessage, and `query` is a valid
    // APPBARDATA for it, which outlives the call. Explorer answers, and the
    // messages that this thread gets meanwhile reach the window procedure.
    if unsafe { app_bar_message(ABM_GETSTATE, &mut query) } & ABS_AUTOHIDE == 0 {
        return;
    }
    // SAFETY: MonitorFromWindow accepts any handle, and always gives a
    // monitor with MONITOR_DEFAULTTONEAREST.
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is a valid MONITORINFO for GetMonitorInfoW to fill in.
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return;
    }
    let hides_on = |edge| {
        let mut query = AppBarData::new(edge, info.rcMonitor);
        // SAFETY: as above. It gives the taskbar that hides itself on that
        // edge of the monitor, or 0.
        unsafe { app_bar_message(ABM_GETAUTOHIDEBAREX, &mut query) != 0 }
    };
    if hides_on(ABE_LEFT) {
        client.left += AUTOHIDE_TASKBAR;
    }
    if hides_on(ABE_TOP) {
        client.top += AUTOHIDE_TASKBAR;
    }
    if hides_on(ABE_RIGHT) {
        client.right -= AUTOHIDE_TASKBAR;
    }
    if hides_on(ABE_BOTTOM) {
        client.bottom -= AUTOHIDE_TASKBAR;
    }
}

/// Whether this program runs in a remote session (Remote Desktop). It can
/// change while it runs, for example when the user connects from another
/// computer.
pub fn is_remote_session() -> bool {
    // SAFETY: GetSystemMetrics has no preconditions.
    unsafe { GetSystemMetrics(SM_REMOTESESSION) != 0 }
}

/// Wakes the message loop of a window up from any thread: the window gives
/// [`Event::Wake`] to its handler, with the number that the call gave, for
/// example to say who woke it up.
///
/// Each call posts a message to the window, which waits in its queue until
/// the loop takes it, and the queue of a thread holds 10 000 at most:
/// whoever calls it should post only a few with the same number while they
/// wait, for example one for each kind of change. Once the window is
/// destroyed, it does nothing.
#[derive(Clone, Debug)]
pub struct Waker(
    /// The handle of the window, as a number, which can go to other threads.
    usize,
);

// The number of a wake-up goes whole in the wParam of its message: meiga is
// only built for x64, where it has 64 bits.
const _: () = assert!(size_of::<WPARAM>() == size_of::<u64>());

impl Waker {
    pub fn wake(&self, number: u64) {
        let number = number as WPARAM;
        // SAFETY: PostMessageW accepts any handle, and fails without effects
        // if the window no longer exists.
        unsafe { PostMessageW(self.0 as HWND, WM_WAKE, number, 0) };
    }
}

/// How the user wants the text caret, which a terminal also uses for its
/// cursor. The settings can change while meiga runs: for example, Remote
/// Desktop turns blinking off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaretSettings {
    /// How long the caret stays shown, and then hidden, while it blinks, or
    /// `None` if it does not blink. It blinks, as in Windows Terminal, if
    /// `SM_CARETBLINKINGENABLED` says so and `GetCaretBlinkTime` gives a
    /// time of at most 10 s.
    pub blink: Option<Duration>,
    /// How long it blinks before it stops, shown, until there is activity
    /// again: `SPI_GETCARETTIMEOUT`, 5 s by default.
    pub timeout: Duration,
    /// Its width in pixels at a scale of 100 %, `SPI_GETCARETWIDTH`: the
    /// "text cursor thickness" of the accessibility settings, 1 by default.
    pub width: u32,
}

/// The current settings of the caret.
pub fn caret_settings() -> CaretSettings {
    // SAFETY: neither function has preconditions. GetSystemMetrics takes the
    // index as an i32, which the constant fits in.
    let (enabled, blink) = unsafe {
        (
            GetSystemMetrics(SM_CARETBLINKINGENABLED as i32),
            GetCaretBlinkTime(),
        )
    };
    let blink = (enabled != 0 && (1..=MAX_BLINK_MS).contains(&blink))
        .then(|| Duration::from_millis(u64::from(blink)));
    let timeout = system_parameter(SPI_GETCARETTIMEOUT)
        .filter(|&ms| ms > 0)
        .map_or(DEFAULT_CARET_TIMEOUT, |ms| {
            Duration::from_millis(u64::from(ms))
        });
    let width = system_parameter(SPI_GETCARETWIDTH)
        .filter(|&width| width > 0)
        .unwrap_or(1);
    CaretSettings {
        blink,
        timeout,
        width,
    }
}

/// A system parameter that Windows gives as a `DWORD`, or `None` if it
/// fails, for example because the version of Windows does not have it.
fn system_parameter(action: u32) -> Option<u32> {
    let mut value = 0_u32;
    // SAFETY: for these actions, pvParam points to a DWORD that receives the
    // value. 0: the user profile does not change.
    let read = unsafe { SystemParametersInfoW(action, 0, ptr::from_mut(&mut value).cast(), 0) };
    (read != 0).then_some(value)
}

/// What `SPI_GETWHEELSCROLLLINES` gives when a notch of the wheel scrolls
/// a page (`WHEEL_PAGESCROLL`).
const WHEEL_PAGE: u32 = u32::MAX;

/// How the user wants the mouse. The settings can change while meiga
/// runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseSettings {
    /// When two clicks are a double click.
    pub clicks: ClickSettings,
    /// The lines that a notch of the wheel scrolls, 3 by default, or
    /// `None` if it scrolls a page.
    pub wheel_lines: Option<u32>,
    /// The characters that a notch of the wheel to a side scrolls, 3 by
    /// default.
    pub wheel_chars: u32,
}

/// The current settings of the mouse.
pub fn mouse_settings() -> MouseSettings {
    let defaults = ClickSettings::default();
    // SAFETY: neither function has preconditions.
    let (time, width, height) = unsafe {
        (
            GetDoubleClickTime(),
            GetSystemMetrics(SM_CXDOUBLECLK),
            GetSystemMetrics(SM_CYDOUBLECLK),
        )
    };
    let clicks = ClickSettings {
        time: Some(time)
            .filter(|&ms| ms > 0)
            .map_or(defaults.time, |ms| Duration::from_millis(u64::from(ms))),
        width: u32::try_from(width).unwrap_or(defaults.width),
        height: u32::try_from(height).unwrap_or(defaults.height),
    };
    let wheel_lines = match system_parameter(SPI_GETWHEELSCROLLLINES) {
        Some(WHEEL_PAGE) => None,
        Some(lines) => Some(lines),
        None => Some(3),
    };
    MouseSettings {
        clicks,
        wheel_lines,
        wheel_chars: system_parameter(SPI_GETWHEELSCROLLCHARS).unwrap_or(3),
    }
}

/// Shows an error message and waits until the user closes it.
pub fn show_error(title: &str, text: &str) {
    let title = wide(title);
    let text = wide(text);
    // A WM_QUIT waiting in the queue, posted when the window was destroyed,
    // would close the message box at once: its message loop ends when it
    // finds one. Take it out while the box is shown, and post it again
    // after.
    let mut quit = MSG::default();
    // SAFETY: `quit` is a valid MSG for PeekMessageW to fill in.
    let pending =
        unsafe { PeekMessageW(&mut quit, ptr::null_mut(), WM_QUIT, WM_QUIT, PM_REMOVE) } != 0;
    // SAFETY: both strings are null-terminated and outlive the call, and the
    // message box has no owner window.
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        )
    };
    if pending {
        // SAFETY: PostQuitMessage has no preconditions. The exit code of
        // WM_QUIT is in its wParam.
        unsafe { PostQuitMessage(quit.wParam as i32) };
    }
}

/// The timer of the next frame.
///
/// It is a high-resolution waitable timer, which the message loop waits for
/// together with the messages. A window timer (`SetTimer`) only fires at the
/// ticks of the system clock, every 15.6 ms by default, and would turn 30
/// frames per second into 21. While Windows runs a message loop of its own
/// (when the user moves or resizes the window, or opens its menu), the
/// waitable timer is not waited for, and a window timer replaces it.
struct FrameTimer {
    handle: HANDLE,
    /// When it is due, while it is started.
    due: Cell<Option<Instant>>,
    /// Whether a message loop of Windows is running.
    modal: Cell<bool>,
}

impl FrameTimer {
    fn new() -> Result<Self, Error> {
        let create = |flags| {
            // SAFETY: no security attributes and no name. It returns null on
            // failure.
            unsafe { CreateWaitableTimerExW(ptr::null(), ptr::null(), flags, TIMER_ALL_ACCESS) }
        };
        let mut handle = create(CREATE_WAITABLE_TIMER_HIGH_RESOLUTION);
        if handle.is_null() {
            // Windows 10 has high-resolution timers since version 1803.
            handle = create(0);
        }
        if handle.is_null() {
            return Err(Error::last("CreateWaitableTimerExW"));
        }
        Ok(Self {
            handle,
            due: Cell::new(None),
            modal: Cell::new(false),
        })
    }
}

impl Drop for FrameTimer {
    fn drop(&mut self) {
        // SAFETY: the handle was created in `new`, and nothing else owns it.
        unsafe { CloseHandle(self.handle) };
    }
}

/// What the window procedure needs while [`Window::run`] runs.
struct State<'a> {
    window: &'a Window,
    handler: RefCell<&'a mut dyn Handler>,
    /// The first error of the handler.
    failure: RefCell<Option<HandlerError>>,
    keyboard: Keyboard,
}

impl State<'_> {
    fn dispatch(&self, event: Event) {
        let result = self.deliver(|handler, window| handler.event(window, event));
        self.check(result);
    }

    /// Calls the handler, unless it cannot take more calls.
    fn deliver(
        &self,
        call: impl FnOnce(&mut dyn Handler, &Window) -> Result<(), HandlerError>,
    ) -> Result<(), HandlerError> {
        // After an error, the window is being destroyed: nothing else reaches
        // the handler.
        if self.failure.borrow().is_some() {
            return Ok(());
        }
        // The handler is already running: it called a function that sent a
        // message to the window and waits for it (see `Handler`). The message
        // is not delivered.
        let Ok(mut handler) = self.handler.try_borrow_mut() else {
            return Ok(());
        };
        call(&mut **handler, self.window)
    }

    /// If the handler failed, keeps its error for [`Window::run`] and
    /// destroys the window, which ends the message loop.
    fn check(&self, result: Result<(), HandlerError>) {
        if let Err(error) = result {
            *self.failure.borrow_mut() = Some(error);
            // SAFETY: the window is of this thread. Destroying it posts
            // WM_QUIT (see WM_DESTROY), which ends the message loop.
            unsafe { DestroyWindow(self.window.handle) };
        }
    }
}

/// Stores the state of [`Window::run`] in the window for the window
/// procedure, and removes it when dropped.
struct Installed(HWND);

impl Installed {
    fn new(window: HWND, state: &State<'_>) -> Self {
        // SAFETY: the window is of this thread. The state outlives this
        // guard, which removes the pointer when it is dropped.
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, ptr::from_ref(state) as isize) };
        Self(window)
    }
}

impl Drop for Installed {
    fn drop(&mut self) {
        // If the loop failed, the window still exists, and its messages must
        // no longer reach the state, which is about to be dropped.
        // SAFETY: SetWindowLongPtrW fails without effects for a window that
        // no longer exists.
        unsafe { SetWindowLongPtrW(self.0, GWLP_USERDATA, 0) };
    }
}

/// Handles the messages of meiga windows.
unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: GWLP_USERDATA holds 0 or a pointer to the State of the running
    // `Window::run`, which removes it before the State is dropped. The State
    // is only used through shared references.
    let state = unsafe { (GetWindowLongPtrW(window, GWLP_USERDATA) as *const State<'_>).as_ref() };
    match message {
        // The window moved to a monitor with another scale, or the scale of
        // its monitor changed. The handler learns it first, to have its font
        // ready for the size that comes next. The window takes the size and
        // position that Windows suggests, so it keeps its physical size.
        WM_DPICHANGED => {
            if let Some(state) = state {
                // Both words of wParam have the new DPI: horizontal and
                // vertical, which are the same.
                let dpi = u32::from(wparam as u16);
                state.dispatch(Event::DpiChanged { dpi });
            }
            // SAFETY: for WM_DPICHANGED, `lparam` points to a RECT that is
            // valid while the message is handled.
            let suggested = unsafe { &*(lparam as *const RECT) };
            // SAFETY: `window` is the window that receives the message.
            unsafe {
                SetWindowPos(
                    window,
                    ptr::null_mut(),
                    suggested.left,
                    suggested.top,
                    suggested.right - suggested.left,
                    suggested.bottom - suggested.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )
            };
            return 0;
        }
        // The handler paints every pixel in WM_PAINT: say that the background
        // is erased, so that Windows does not paint it first.
        WM_ERASEBKGND => return 1,
        WM_DESTROY => {
            // SAFETY: PostQuitMessage has no preconditions.
            unsafe { PostQuitMessage(0) };
            return 0;
        }
        // With the title bar of meiga, the client area goes up to the top of
        // the window, and Windows keeps its frame on the other sides.
        WM_NCCALCSIZE if custom_frame(window) => {
            // For WM_NCCALCSIZE, `lparam` points to the rectangle of the
            // window, alone or as the first member of a NCCALCSIZE_PARAMS,
            // and DefWindowProcW leaves the client area in it.
            let rect = lparam as *mut RECT;
            // SAFETY: as above: it is valid while the message is handled.
            let top = unsafe { (*rect).top };
            // SAFETY: the arguments are the ones that Windows passed to this
            // window procedure.
            let result = unsafe { DefWindowProcW(window, message, wparam, lparam) };
            // SAFETY: as above.
            let mut client = unsafe { *rect };
            client.top = top;
            // Maximized, the window goes beyond its monitor as far as its
            // frame on each side: its client area starts below the frame, at
            // the top of the monitor, as with the title bar of Windows.
            // SAFETY: IsZoomed accepts any handle.
            if unsafe { IsZoomed(window) } != 0 {
                client.top += frame(window_dpi(window)) as i32;
                leave_room_for_hidden_taskbars(window, &mut client);
            }
            // SAFETY: as above.
            unsafe { *rect = client };
            return result;
        }
        // The smallest window has room for the narrowest title bar of meiga,
        // with the frame of Windows on its sides and below it.
        WM_GETMINMAXINFO if custom_frame(window) => {
            let dpi = window_dpi(window);
            let band = frame(dpi);
            let min = min_client(dpi, band);
            let pixels = |value: u32| i32::try_from(value).unwrap_or(i32::MAX);
            // SAFETY: for WM_GETMINMAXINFO, `lparam` points to a MINMAXINFO
            // that is valid while the message is handled.
            let info = unsafe { &mut *(lparam as *mut MINMAXINFO) };
            info.ptMinTrackSize.x = info
                .ptMinTrackSize
                .x
                .max(pixels(min.width.saturating_add(band.saturating_mul(2))));
            info.ptMinTrackSize.y = info
                .ptMinTrackSize
                .y
                .max(pixels(min.height.saturating_add(band)));
            return 0;
        }
        _ => {}
    }

    let Some(state) = state else {
        // SAFETY: the arguments are the ones that Windows passed to this
        // window procedure.
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    };
    let this = state.window;
    match message {
        WM_PAINT => this.paint(state),
        WM_TIMER if wparam == FRAME_TIMER_ID => this.frame_timer_due(state),
        WM_ENTERSIZEMOVE => {
            this.enter_modal_loop();
            state.dispatch(Event::SizeMoveStarted);
        }
        WM_EXITSIZEMOVE => {
            this.leave_modal_loop();
            state.dispatch(Event::SizeMoveEnded);
        }
        WM_ENTERMENULOOP => this.enter_modal_loop(),
        WM_EXITMENULOOP => this.leave_modal_loop(),
        // Over the client area, the pointer that the handler chose.
        WM_SETCURSOR if (lparam & 0xffff) as u32 == HTCLIENT => {
            // SAFETY: the cursor is a system one, loaded in Window::create.
            unsafe { SetCursor(this.pointer.get()) };
            return 1;
        }
        // With the title bar of meiga, what a point of the client area is for
        // Windows: the title bar drags the window, its top edge resizes it,
        // and its buttons are the ones of the window, so that the one to
        // maximize opens the snap layouts of Windows 11.
        WM_NCHITTEST if this.custom => {
            // The frame of Windows, on the sides and below. The answer stays
            // a LRESULT, because HTERROR is negative.
            // SAFETY: the arguments are the ones that Windows passed to this
            // window procedure.
            let default = unsafe { DefWindowProcW(window, message, wparam, lparam) };
            if default != HTCLIENT as LRESULT {
                return default;
            }
            return this.hit_test(point(lparam)) as LRESULT;
        }
        WM_NCMOUSEMOVE..=WM_NCMBUTTONDBLCLK if this.custom => {
            // SAFETY: GetKeyState has no preconditions. The high bit of the
            // state of a key says that it is down, as the thread got the
            // message.
            let down = |key: u16| unsafe { GetKeyState(i32::from(key)) } < 0;
            let modifiers = || Modifiers {
                shift: down(VK_SHIFT),
                alt: down(VK_MENU),
                ctrl: down(VK_CONTROL),
            };
            if let Some(mouse) = decode_nonclient(message, wparam, lparam, modifiers)
                && !this.nonclient_mouse(state, mouse)
            {
                return 0;
            }
            // SAFETY: the arguments are the ones that Windows passed to this
            // window procedure.
            return unsafe { DefWindowProcW(window, message, wparam, lparam) };
        }
        WM_MOUSELEAVE => this.mouse_left(state),
        WM_NCMOUSELEAVE if this.custom => {
            this.mouse_left(state);
            // SAFETY: the arguments are the ones that Windows passed to this
            // window procedure.
            return unsafe { DefWindowProcW(window, message, wparam, lparam) };
        }
        _ if is_keyboard_message(message) => {
            let mut send = |key| state.dispatch(Event::Key(key));
            let destination = state
                .keyboard
                .message(window, message, wparam, lparam, &mut send);
            if destination == Destination::Windows {
                // SAFETY: the arguments are the ones that Windows passed to
                // this window procedure.
                return unsafe { DefWindowProcW(window, message, wparam, lparam) };
            }
        }
        _ => {
            match message {
                // The pointer is over the client area for Windows, or the
                // window has the mouse.
                WM_MOUSEMOVE => {
                    this.hit.set(HTCLIENT);
                    this.retracked.set(false);
                    this.rearm();
                }
                WM_LBUTTONDOWN | WM_LBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_RBUTTONDOWN
                | WM_RBUTTONUP => this.hit.set(HTCLIENT),
                _ => {}
            }
            // The mouse messages do not say whether Alt is down. The high
            // bit of the state of the key says it, as the thread got the
            // message.
            // SAFETY: GetKeyState has no preconditions.
            let alt = || unsafe { GetKeyState(i32::from(VK_MENU)) } < 0;
            match decode(message, wparam, lparam, alt) {
                Some(Event::Wheel {
                    at,
                    delta,
                    horizontal,
                    modifiers,
                }) => state.dispatch(Event::Wheel {
                    at: this.client_point(at),
                    delta,
                    horizontal,
                    modifiers,
                }),
                Some(event) => state.dispatch(event),
                // SAFETY: the arguments are the ones that Windows passed to
                // this window procedure.
                None => return unsafe { DefWindowProcW(window, message, wparam, lparam) },
            }
        }
    }
    0
}

/// The due time of a waitable timer `delay` from now: negative, because it
/// is relative, in units of 100 ns, and at least one.
fn due_time(delay: Duration) -> i64 {
    -i64::try_from(delay.as_nanos().div_ceil(100))
        .unwrap_or(i64::MAX)
        .max(1)
}

/// The milliseconds of a window timer for `delay`, rounded up, within the
/// limits of `SetTimer`.
fn timer_millis(delay: Duration) -> u32 {
    u32::try_from(delay.as_nanos().div_ceil(1_000_000))
        .unwrap_or(USER_TIMER_MAXIMUM)
        .clamp(USER_TIMER_MINIMUM, USER_TIMER_MAXIMUM)
}

#[cfg(test)]
mod tests {
    use meiga_core::ui::tabbar::{CaptionButton, Layout};
    use windows_sys::Win32::Graphics::Gdi::{ClientToScreen, HDC};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HTBOTTOM, HTLEFT, SendMessageW, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP,
    };

    use super::*;

    /// Keeps the events that reach it.
    #[derive(Default)]
    struct Recorder(Vec<Event>);

    impl Handler for Recorder {
        fn event(&mut self, _window: &Window, event: Event) -> Result<(), HandlerError> {
            self.0.push(event);
            Ok(())
        }

        fn paint(&mut self, _window: &Window, _dc: HDC, _area: Rect) -> Result<(), HandlerError> {
            Ok(())
        }
    }

    /// The rectangle of the window on the screen.
    fn window_rect(window: &Window) -> RECT {
        let mut rect = RECT::default();
        // SAFETY: `rect` is a valid RECT for GetWindowRect to fill in.
        unsafe { GetWindowRect(window.handle(), &mut rect) };
        rect
    }

    /// Gives the window `width` × `height` pixels, or as many as it can have.
    fn resize(window: &Window, width: i32, height: i32) {
        // SAFETY: the window is of this thread. It keeps its place and order.
        unsafe {
            SetWindowPos(
                window.handle(),
                ptr::null_mut(),
                0,
                0,
                width,
                height,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    /// Sends a mouse message of the frame to the window, at `at`, a point of
    /// its client area.
    fn send(window: &Window, message: u32, hit: u32, at: Point) -> LRESULT {
        let mut point = POINT { x: at.x, y: at.y };
        // SAFETY: the window is of this thread, and `point` is a valid POINT
        // for ClientToScreen to change.
        unsafe { ClientToScreen(window.handle(), &mut point) };
        // The screen point in two signed words, as Windows sends it.
        let lparam = (point.x as u16 as LPARAM) | ((point.y as u16 as LPARAM) << 16);
        // SAFETY: the window is of this thread, and the message is handled
        // before the call returns.
        unsafe { SendMessageW(window.handle(), message, hit as WPARAM, lparam) }
    }

    fn center(rect: Rect) -> Point {
        Point::new(
            rect.left + (rect.right - rect.left) / 2,
            rect.top + (rect.bottom - rect.top) / 2,
        )
    }

    // Each test that creates a window registers the class, unless the other
    // one did.
    #[test]
    fn a_new_scale_reaches_the_handler_before_the_new_size() {
        let window = Window::create(&WindowOptions {
            title: "meiga",
            title_bar: false,
        })
        .unwrap();
        let mut recorder = Recorder::default();
        {
            let state = State {
                window: &window,
                handler: RefCell::new(&mut recorder),
                failure: RefCell::new(None),
                keyboard: Keyboard::default(),
            };
            let _installed = Installed::new(window.handle(), &state);
            let suggested = RECT {
                left: 40,
                top: 30,
                right: 817,
                bottom: 585,
            };
            // SAFETY: the window is of this thread, and the rectangle
            // outlives the call, which handles the message.
            unsafe {
                SendMessageW(
                    window.handle(),
                    WM_DPICHANGED,
                    144 | (144 << 16),
                    ptr::from_ref(&suggested) as LPARAM,
                )
            };
        }
        let rect = window_rect(&window);
        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (40, 30, 817, 585)
        );
        let resized = Event::Resized {
            size: window.client_size().unwrap(),
            minimized: false,
            maximized: false,
        };
        let events = &recorder.0;
        assert_eq!(events.first(), Some(&Event::DpiChanged { dpi: 144 }));
        assert!(events.contains(&resized), "{events:?}");
    }

    #[test]
    fn the_title_bar_of_meiga_tells_windows_what_each_of_its_parts_is() {
        let window = Window::create(&WindowOptions {
            title: "meiga",
            title_bar: true,
        })
        .unwrap();
        assert!(window.has_title_bar());
        resize(&window, 800, 600);
        // The test is not aware of the DPI: 96.
        let dpi = window.dpi();
        let band = frame(dpi);
        let side = i32::try_from(band).unwrap();
        assert!(side > 0);
        assert_eq!(window.resize_band(dpi), band);

        // The client area goes up to the top of the window, and the frame of
        // Windows stays on the other sides.
        let rect = window_rect(&window);
        assert_eq!(
            window.client_point(Point::new(rect.left, rect.top)),
            Point::new(-side, 0)
        );
        let size = window.client_size().unwrap();
        assert_eq!(
            (
                i32::try_from(size.width).unwrap() + 2 * side,
                i32::try_from(size.height).unwrap() + side
            ),
            (rect.right - rect.left, rect.bottom - rect.top)
        );

        let mut recorder = Recorder::default();
        let layout = Layout::new(dpi, size.width, 3, band);
        let close = center(layout.caption(CaptionButton::Close));
        let left_frame = Point::new(-side / 2, 100);
        {
            let state = State {
                window: &window,
                handler: RefCell::new(&mut recorder),
                failure: RefCell::new(None),
                keyboard: Keyboard::default(),
            };
            let _installed = Installed::new(window.handle(), &state);
            window.set_title_bar(TitleBar::new(layout, false));
            let captions = layout.captions();
            let corner = TOP_LEFT_CORNER - side;
            let bottom = i32::try_from(size.height).unwrap();
            for (at, expected) in [
                (center(layout.caption(CaptionButton::Minimize)), HTMINBUTTON),
                (center(layout.caption(CaptionButton::Maximize)), HTMAXBUTTON),
                (close, HTCLOSE),
                (center(layout.tab(0)), HTCLIENT),
                (Point::new(captions.left - 1, side + 5), HTCAPTION),
                (Point::new(captions.left - 1, 0), HTTOP),
                (Point::new(corner, 0), HTTOP),
                (Point::new(corner - 1, 0), HTTOPLEFT),
                (left_frame, HTLEFT),
                (Point::new(100, bottom + side / 2), HTBOTTOM),
            ] {
                let hit = send(&window, WM_NCHITTEST, 0, at);
                assert_eq!(hit, expected as LRESULT, "{at:?}");
            }

            // The handler draws the buttons of the window, and gets their
            // clicks.
            send(&window, WM_NCLBUTTONDOWN, HTCLOSE, close);
            send(&window, WM_NCLBUTTONUP, HTCLOSE, close);
            // The frame, which resizes the window, is not of the handler.
            window.track_mouse_leave();
            send(&window, WM_NCMOUSEMOVE, HTLEFT, left_frame);
        }
        let modifiers = Modifiers::default();
        assert_eq!(
            recorder.0,
            [
                Event::MouseDown {
                    button: Button::Left,
                    at: close,
                    modifiers,
                },
                Event::MouseUp {
                    button: Button::Left,
                    at: close,
                    modifiers,
                },
                Event::MouseLeft,
            ]
        );

        // The smallest window has room for the narrowest title bar.
        resize(&window, 100, 10);
        let rect = window_rect(&window);
        let min = min_client(dpi, band);
        assert_eq!(
            (rect.right - rect.left, rect.bottom - rect.top),
            (
                i32::try_from(min.width + 2 * band).unwrap(),
                i32::try_from(min.height + band).unwrap()
            )
        );
    }

    #[test]
    fn waitable_timers_count_in_hundreds_of_nanoseconds_from_now() {
        assert_eq!(due_time(Duration::from_micros(33_334)), -333_340);
        assert_eq!(due_time(Duration::from_nanos(150)), -2);
        assert_eq!(due_time(Duration::ZERO), -1);
    }

    #[test]
    fn window_timers_round_up_within_their_limits() {
        assert_eq!(timer_millis(Duration::from_micros(33_334)), 34);
        assert_eq!(timer_millis(Duration::from_millis(2)), USER_TIMER_MINIMUM);
        assert_eq!(timer_millis(Duration::MAX), USER_TIMER_MAXIMUM);
    }
}
