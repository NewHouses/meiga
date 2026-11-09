//! The events that the window delivers to its handler.

#![forbid(unsafe_code)]

use std::error::Error as StdError;

use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::input::KeyEvent;
use windows_sys::Win32::Foundation::{LPARAM, WPARAM};
use windows_sys::Win32::Graphics::Gdi::HDC;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SIZE_MINIMIZED, WM_APP, WM_CAPTURECHANGED, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETFOCUS, WM_SIZE,
};

use crate::window::Window;

/// The message of [`Waker`](crate::window::Waker): the first one that
/// Windows leaves to the programs for their own windows.
pub(crate) const WM_WAKE: u32 = WM_APP;

/// The `wParam` flag of mouse messages for a pressed left button. It is in
/// the `Win32_System_SystemServices` feature of `windows-sys`, which would
/// bring thousands of declarations for this one constant.
const MK_LBUTTON: WPARAM = 0x0001;

/// An error from the code that handles the events. It ends the message loop.
pub type HandlerError = Box<dyn StdError>;

/// The code that reacts to the events of a window.
///
/// The window calls it from its message loop, so it must return quickly. It
/// must not call functions that send messages to the window and wait for
/// them, such as `SetWindowPos` or `UpdateWindow`: the window would call the
/// handler again before it returns, and Rust does not allow that, so those
/// events would be lost.
pub trait Handler {
    /// Something happened in the window.
    fn event(&mut self, window: &Window, event: Event) -> Result<(), HandlerError>;

    /// Windows asks to paint `area` of the window in `dc`, for example
    /// because it is shown for the first time or because it grew.
    fn paint(&mut self, window: &Window, dc: HDC, area: Rect) -> Result<(), HandlerError>;
}

/// Something that happened in the window. Coordinates are in pixels, from
/// the top-left corner of its client area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The client area changed size, or the window was minimized or
    /// restored. A minimized window has size 0 × 0.
    Resized {
        size: Size,
        minimized: bool,
    },
    /// The user started to move or resize the window with the mouse or the
    /// keyboard. Until [`Event::SizeMoveEnded`], Windows runs its own message
    /// loop.
    SizeMoveStarted,
    /// The user finished moving or resizing the window.
    SizeMoveEnded,
    MouseDown {
        button: MouseButton,
        at: Point,
    },
    MouseUp {
        button: MouseButton,
        at: Point,
    },
    /// The mouse moved, with the left button pressed or not.
    MouseMoved {
        at: Point,
        left_down: bool,
    },
    /// The window no longer receives all the mouse input
    /// ([`Window::capture_mouse`]), for example because another window took
    /// it.
    CaptureLost,
    /// A key went down or up, with the character that it writes, if any.
    /// Alt+F4 and Alt+Space never come as events: Windows handles them, to
    /// close the window and to open its menu.
    Key(KeyEvent),
    /// The window gained or lost the keyboard focus.
    Focus {
        focused: bool,
    },
    /// The frame timer ([`Window::start_frame_timer`]) is due.
    FrameTimer,
    /// Another thread woke the window up with its
    /// [`Waker`](crate::window::Waker).
    Wake,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
}

/// The event of a window message, or `None` if it is not one of them. The
/// keyboard messages are not: they become events in the window, which asks
/// Windows more about them.
pub(crate) fn decode(message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<Event> {
    let at = || point(lparam);
    Some(match message {
        WM_SIZE => Event::Resized {
            size: Size::new(u32::from(low_word(lparam)), u32::from(high_word(lparam))),
            minimized: wparam == SIZE_MINIMIZED as WPARAM,
        },
        WM_LBUTTONDOWN => Event::MouseDown {
            button: MouseButton::Left,
            at: at(),
        },
        WM_LBUTTONUP => Event::MouseUp {
            button: MouseButton::Left,
            at: at(),
        },
        WM_RBUTTONDOWN => Event::MouseDown {
            button: MouseButton::Right,
            at: at(),
        },
        WM_RBUTTONUP => Event::MouseUp {
            button: MouseButton::Right,
            at: at(),
        },
        WM_MOUSEMOVE => Event::MouseMoved {
            at: at(),
            left_down: wparam & MK_LBUTTON != 0,
        },
        WM_CAPTURECHANGED => Event::CaptureLost,
        WM_SETFOCUS => Event::Focus { focused: true },
        WM_KILLFOCUS => Event::Focus { focused: false },
        WM_WAKE => Event::Wake,
        _ => return None,
    })
}

/// The point of a mouse message. Each coordinate is a signed 16-bit number,
/// because it is negative to the left of or above the window while the mouse
/// is captured.
fn point(lparam: LPARAM) -> Point {
    Point::new(
        i32::from(low_word(lparam) as i16),
        i32::from(high_word(lparam) as i16),
    )
}

const fn low_word(value: LPARAM) -> u16 {
    value as u16
}

const fn high_word(value: LPARAM) -> u16 {
    (value >> 16) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `lParam` of a mouse message for (`x`, `y`).
    fn xy(x: i16, y: i16) -> LPARAM {
        (x as u16 as LPARAM) | (y as u16 as LPARAM) << 16
    }

    #[test]
    fn size_comes_from_the_two_words_of_lparam() {
        assert_eq!(
            decode(WM_SIZE, 0, xy(1920, 1080)),
            Some(Event::Resized {
                size: Size::new(1920, 1080),
                minimized: false
            })
        );
        assert_eq!(
            decode(WM_SIZE, SIZE_MINIMIZED as WPARAM, 0),
            Some(Event::Resized {
                size: Size::new(0, 0),
                minimized: true
            })
        );
    }

    #[test]
    fn mouse_coordinates_can_be_negative() {
        assert_eq!(
            decode(WM_LBUTTONDOWN, MK_LBUTTON, xy(-3, 40)),
            Some(Event::MouseDown {
                button: MouseButton::Left,
                at: Point::new(-3, 40)
            })
        );
        assert_eq!(
            decode(WM_MOUSEMOVE, 0, xy(10, -20)),
            Some(Event::MouseMoved {
                at: Point::new(10, -20),
                left_down: false
            })
        );
        assert_eq!(
            decode(WM_MOUSEMOVE, MK_LBUTTON, xy(10, 20)),
            Some(Event::MouseMoved {
                at: Point::new(10, 20),
                left_down: true
            })
        );
    }

    #[test]
    fn focus_and_wake_messages_are_events() {
        assert_eq!(
            decode(WM_SETFOCUS, 0, 0),
            Some(Event::Focus { focused: true })
        );
        assert_eq!(
            decode(WM_KILLFOCUS, 0, 0),
            Some(Event::Focus { focused: false })
        );
        assert_eq!(decode(WM_WAKE, 0, 0), Some(Event::Wake));
    }

    #[test]
    fn other_messages_are_not_events() {
        assert_eq!(decode(0x0400, 0, 0), None);
        assert_eq!(decode(WM_WAKE + 1, 0, 0), None);
    }
}
