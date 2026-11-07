//! The main window and the message loop.

use std::ptr;

use meiga_core::color::Rgb;
use windows_sys::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{CreateSolidBrush, DeleteObject, UpdateWindow};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, IDC_ARROW,
    LoadCursorW, MB_ICONERROR, MB_OK, MSG, MessageBoxW, PostQuitMessage, RegisterClassExW,
    SW_SHOWDEFAULT, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos, ShowWindow, TranslateMessage,
    WM_DESTROY, WM_DPICHANGED, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

use crate::Error;

/// Name of the window class of meiga windows.
const CLASS_NAME: &str = "meiga";

/// How a new window looks.
pub struct WindowOptions<'a> {
    /// Text of the title bar.
    pub title: &'a str,
    /// Colour that fills the client area.
    pub background: Rgb,
}

/// A top-level window.
///
/// The window belongs to the thread that creates it and lives until the user
/// closes it, which ends [`run_message_loop`].
pub struct Window {
    handle: HWND,
}

impl Window {
    /// Registers the window class and creates the window, still hidden.
    ///
    /// Call it once per process: a window class can be registered only once.
    pub fn create(options: &WindowOptions<'_>) -> Result<Self, Error> {
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

        // The class owns the brush: the system paints the background with it
        // (WM_ERASEBKGND) and deletes it when the class is unregistered, at
        // the latest when the process ends.
        // SAFETY: CreateSolidBrush has no preconditions.
        let background = unsafe { CreateSolidBrush(colorref(options.background)) };
        if background.is_null() {
            return Err(Error::last("CreateSolidBrush"));
        }

        let class_name = wide(CLASS_NAME);
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            hCursor: cursor,
            hbrBackground: background,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };
        // SAFETY: `class` is fully initialised and its strings are
        // null-terminated and outlive the call, which copies them.
        if unsafe { RegisterClassExW(&class) } == 0 {
            let error = Error::last("RegisterClassExW");
            // SAFETY: the brush was created above and, as the registration
            // failed, nothing else owns or uses it.
            unsafe { DeleteObject(background) };
            return Err(error);
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
        Ok(Self { handle })
    }

    /// Shows the window and paints it before returning.
    pub fn show(&self) -> Result<(), Error> {
        // SW_SHOWDEFAULT honours how the launcher asked to start the program
        // (minimised or maximised from a shortcut, for example). The return
        // value is the previous visibility, not an error.
        // SAFETY: `self.handle` is a window of this thread.
        unsafe { ShowWindow(self.handle, SW_SHOWDEFAULT) };
        // SAFETY: as above.
        if unsafe { UpdateWindow(self.handle) } == 0 {
            return Err(Error::last("UpdateWindow"));
        }
        Ok(())
    }
}

/// Dispatches the messages of the calling thread until a window closes.
///
/// While there are no messages, the thread sleeps inside `GetMessageW`, so an
/// idle window uses no CPU.
pub fn run_message_loop() -> Result<(), Error> {
    let mut message = MSG::default();
    loop {
        // SAFETY: `message` is a valid MSG for GetMessageW to fill in.
        match unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) } {
            // WM_QUIT, posted when the window is destroyed.
            0 => return Ok(()),
            -1 => return Err(Error::last("GetMessageW")),
            _ => {
                // SAFETY: `message` holds the message that GetMessageW has
                // just retrieved.
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
    }
}

/// Shows an error message and waits until the user closes it.
pub fn show_error(title: &str, text: &str) {
    let title = wide(title);
    let text = wide(text);
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
            0
        }
        WM_DESTROY => {
            // SAFETY: PostQuitMessage has no preconditions.
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => {
            // SAFETY: the arguments are the ones that Windows passed to this
            // window procedure.
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
    }
}

/// Converts a colour to a GDI `COLORREF`, which stores it as `0x00BBGGRR`.
const fn colorref(color: Rgb) -> COLORREF {
    color.r as u32 | (color.g as u32) << 8 | (color.b as u32) << 16
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
    fn colorref_stores_blue_green_red() {
        assert_eq!(colorref(Rgb::new(0x12, 0x34, 0x56)), 0x0056_3412);
    }
}
