//! The main window, its frame timer and the message loop.

use std::cell::{Cell, RefCell};
use std::ptr;
use std::time::{Duration, Instant};

use meiga_core::geometry::{Rect, Size};
use windows_sys::Win32::Foundation::{
    CloseHandle, HANDLE, HWND, LPARAM, LRESULT, RECT, WAIT_FAILED, WAIT_OBJECT_0, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT, UpdateWindow, ValidateRect,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{
    CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CancelWaitableTimer, CreateWaitableTimerExW, INFINITE,
    SetWaitableTimer, TIMER_ALL_ACCESS,
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetCapture, ReleaseCapture, SetCapture};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GWLP_USERDATA,
    GetClientRect, GetSystemMetrics, GetWindowLongPtrW, IDC_ARROW, IsWindow, KillTimer,
    LoadCursorW, MB_ICONERROR, MB_OK, MSG, MWMO_INPUTAVAILABLE, MessageBoxW,
    MsgWaitForMultipleObjectsEx, PM_REMOVE, PeekMessageW, PostQuitMessage, QS_ALLINPUT,
    RegisterClassExW, SM_REMOTESESSION, SW_SHOWDEFAULT, SWP_NOACTIVATE, SWP_NOZORDER, SetTimer,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, USER_DEFAULT_SCREEN_DPI,
    USER_TIMER_MAXIMUM, USER_TIMER_MINIMUM, WM_DESTROY, WM_DPICHANGED, WM_ENTERMENULOOP,
    WM_ENTERSIZEMOVE, WM_ERASEBKGND, WM_EXITMENULOOP, WM_EXITSIZEMOVE, WM_PAINT, WM_QUIT, WM_TIMER,
    WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

use crate::Error;
use crate::event::{Event, Handler, HandlerError, decode};

/// Name of the window class of meiga windows.
const CLASS_NAME: &str = "meiga";

/// Identifier of the window timer that replaces the frame timer while
/// Windows runs its own message loop.
const FRAME_TIMER_ID: usize = 1;

/// How a new window looks.
pub struct WindowOptions<'a> {
    /// Text of the title bar.
    pub title: &'a str,
}

/// A top-level window.
///
/// It belongs to the thread that creates it. It lives until the user closes
/// it, which ends [`Window::run`], or until it is dropped. It paints nothing
/// by itself: its [`Handler`] paints every pixel.
pub struct Window {
    handle: HWND,
    timer: FrameTimer,
}

impl Window {
    /// Registers the window class and creates the window, still hidden.
    ///
    /// Call it once per process: a window class can be registered only once.
    pub fn create(options: &WindowOptions<'_>) -> Result<Self, Error> {
        let timer = FrameTimer::new()?;

        // SAFETY: a null name returns the handle of the executable of this
        // process, which stays loaded while the process runs.
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        if instance.is_null() {
            return Err(Error::last("GetModuleHandleW"));
        }

        // SAFETY: IDC_ARROW is a predefined system cursor, which is loaded
        // without an instance handle.
        let cursor = unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) };
        if cursor.is_null() {
            return Err(Error::last("LoadCursorW"));
        }

        // No background brush: the handler paints the whole client area in
        // WM_PAINT, so erasing it first would only paint it twice.
        let class_name = wide(CLASS_NAME);
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            hCursor: cursor,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };
        // SAFETY: `class` is fully initialised and its strings are
        // null-terminated and outlive the call, which copies them.
        if unsafe { RegisterClassExW(&class) } == 0 {
            return Err(Error::last("RegisterClassExW"));
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
        Ok(Self { handle, timer })
    }

    /// The Windows handle of the window, for the functions that draw in it.
    pub const fn handle(&self) -> HWND {
        self.handle
    }

    /// The size of the client area: the window without its title bar and
    /// borders.
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
        // SAFETY: GetDpiForWindow accepts any handle, and returns 0 for one
        // that is not a window.
        match unsafe { GetDpiForWindow(self.handle) } {
            0 => USER_DEFAULT_SCREEN_DPI,
            dpi => dpi,
        }
    }

    /// Shows the window, paints it, and delivers its events to `handler`
    /// until the window closes.
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

/// Whether this program runs in a remote session (Remote Desktop). It can
/// change while it runs, for example when the user connects from another
/// computer.
pub fn is_remote_session() -> bool {
    // SAFETY: GetSystemMetrics has no preconditions.
    unsafe { GetSystemMetrics(SM_REMOTESESSION) != 0 }
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
    match message {
        // The window moved to a monitor with another scale: use the size and
        // position that Windows suggests, so it keeps its physical size.
        WM_DPICHANGED => {
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
        _ => {}
    }

    // SAFETY: GWLP_USERDATA holds 0 or a pointer to the State of the running
    // `Window::run`, which removes it before the State is dropped. The State
    // is only used through shared references.
    let state = unsafe { (GetWindowLongPtrW(window, GWLP_USERDATA) as *const State<'_>).as_ref() };
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
        _ => match decode(message, wparam, lparam) {
            Some(event) => state.dispatch(event),
            // SAFETY: the arguments are the ones that Windows passed to this
            // window procedure.
            None => return unsafe { DefWindowProcW(window, message, wparam, lparam) },
        },
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

/// Converts `text` to the null-terminated UTF-16 that the `W` functions of
/// Windows expect.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
