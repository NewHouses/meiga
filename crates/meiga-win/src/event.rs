//! The events that the window delivers to its handler.

#![forbid(unsafe_code)]

use std::error::Error as StdError;

use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::input::KeyEvent;
use meiga_core::mouse::{Button, Modifiers};
use windows_sys::Win32::Foundation::{LPARAM, WPARAM};
use windows_sys::Win32::Graphics::Gdi::HDC;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SIZE_MINIMIZED, WM_APP, WM_CAPTURECHANGED, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_SETFOCUS, WM_SIZE,
};

use crate::window::Window;

/// The message of [`Waker`](crate::window::Waker): the first one that
/// Windows leaves to the programs for their own windows.
pub(crate) const WM_WAKE: u32 = WM_APP;

// The flags of the buttons and of the modifier keys in the `wParam` of the
// mouse messages. They are in the `Win32_System_SystemServices` feature of
// `windows-sys`, which would bring thousands of declarations for them.
const MK_LBUTTON: WPARAM = 0x0001;
const MK_RBUTTON: WPARAM = 0x0002;
const MK_SHIFT: WPARAM = 0x0004;
const MK_CONTROL: WPARAM = 0x0008;
const MK_MBUTTON: WPARAM = 0x0010;

/// The message of [`Window::track_mouse_leave`]. It is in the
/// `Win32_UI_Controls` feature, for the same reason.
const WM_MOUSELEAVE: u32 = 0x02a3;

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
    Resized { size: Size, minimized: bool },
    /// The window moved to a monitor with another scale, or the scale of
    /// its monitor changed: it has `dpi` dots per inch now. Its new size, if
    /// it changes, comes right after, with [`Event::Resized`].
    DpiChanged { dpi: u32 },
    /// The user started to move or resize the window with the mouse or the
    /// keyboard. Until [`Event::SizeMoveEnded`], Windows runs its own message
    /// loop.
    SizeMoveStarted,
    /// The user finished moving or resizing the window.
    SizeMoveEnded,
    /// A button of the mouse went down, with the modifier keys that were
    /// down.
    MouseDown {
        button: Button,
        at: Point,
        modifiers: Modifiers,
    },
    MouseUp {
        button: Button,
        at: Point,
        modifiers: Modifiers,
    },
    /// The mouse moved, with the first of its buttons that is down, if
    /// any: the left one, the middle one or the right one.
    MouseMoved {
        at: Point,
        button: Option<Button>,
        modifiers: Modifiers,
    },
    /// The wheel turned `delta` 120ths of a notch: forward, or to the right
    /// if it is `horizontal`, when it is positive. `at` is where the
    /// pointer is.
    Wheel {
        at: Point,
        delta: i16,
        horizontal: bool,
        modifiers: Modifiers,
    },
    /// The window no longer receives all the mouse input
    /// ([`Window::capture_mouse`]), for example because another window took
    /// it.
    CaptureLost,
    /// The pointer left the client area, once after each
    /// [`Window::track_mouse_leave`].
    MouseLeft,
    /// A key went down or up, with the character that it writes, if any.
    /// Alt+F4 and Alt+Space never come as events: Windows handles them, to
    /// close the window and to open its menu.
    Key(KeyEvent),
    /// The window gained or lost the keyboard focus.
    Focus { focused: bool },
    /// The frame timer ([`Window::start_frame_timer`]) is due.
    FrameTimer,
    /// Another thread woke the window up with its
    /// [`Waker`](crate::window::Waker).
    Wake,
}

/// The event of a window message, or `None` if it is not one of them. The
/// keyboard messages are not: they become events in the window, which asks
/// Windows more about them. `alt` says whether Alt is down, which the mouse
/// messages do not say. The point of [`Event::Wheel`] is from the top-left
/// corner of the screen: the window makes it of its client area.
pub(crate) fn decode(
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    alt: impl FnOnce() -> bool,
) -> Option<Event> {
    let at = || point(lparam);
    let modifiers = |alt: bool| Modifiers {
        shift: wparam & MK_SHIFT != 0,
        alt,
        ctrl: wparam & MK_CONTROL != 0,
    };
    Some(match message {
        WM_SIZE => Event::Resized {
            size: Size::new(u32::from(low_word(lparam)), u32::from(high_word(lparam))),
            minimized: wparam == SIZE_MINIMIZED as WPARAM,
        },
        WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN => Event::MouseDown {
            button: match message {
                WM_LBUTTONDOWN => Button::Left,
                WM_MBUTTONDOWN => Button::Middle,
                _ => Button::Right,
            },
            at: at(),
            modifiers: modifiers(alt()),
        },
        WM_LBUTTONUP | WM_MBUTTONUP | WM_RBUTTONUP => Event::MouseUp {
            button: match message {
                WM_LBUTTONUP => Button::Left,
                WM_MBUTTONUP => Button::Middle,
                _ => Button::Right,
            },
            at: at(),
            modifiers: modifiers(alt()),
        },
        WM_MOUSEMOVE => Event::MouseMoved {
            at: at(),
            button: [
                (MK_LBUTTON, Button::Left),
                (MK_MBUTTON, Button::Middle),
                (MK_RBUTTON, Button::Right),
            ]
            .into_iter()
            .find_map(|(flag, button)| (wparam & flag != 0).then_some(button)),
            modifiers: modifiers(alt()),
        },
        // The flags are in the low word of wParam, and the turn, signed, in
        // the high one.
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL => Event::Wheel {
            at: at(),
            delta: (wparam >> 16) as u16 as i16,
            horizontal: message == WM_MOUSEHWHEEL,
            modifiers: modifiers(alt()),
        },
        WM_CAPTURECHANGED => Event::CaptureLost,
        WM_MOUSELEAVE => Event::MouseLeft,
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

    fn decode(message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<Event> {
        super::decode(message, wparam, lparam, || false)
    }

    const NONE: Modifiers = Modifiers {
        shift: false,
        alt: false,
        ctrl: false,
    };

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
                button: Button::Left,
                at: Point::new(-3, 40),
                modifiers: NONE,
            })
        );
        assert_eq!(
            decode(WM_MOUSEMOVE, 0, xy(10, -20)),
            Some(Event::MouseMoved {
                at: Point::new(10, -20),
                button: None,
                modifiers: NONE,
            })
        );
    }

    #[test]
    fn mouse_messages_say_the_buttons_and_the_modifier_keys() {
        assert_eq!(
            decode(WM_MBUTTONUP, MK_SHIFT | MK_CONTROL, xy(1, 2)),
            Some(Event::MouseUp {
                button: Button::Middle,
                at: Point::new(1, 2),
                modifiers: Modifiers {
                    shift: true,
                    alt: false,
                    ctrl: true,
                },
            })
        );
        assert_eq!(
            super::decode(WM_RBUTTONDOWN, MK_RBUTTON, xy(1, 2), || true),
            Some(Event::MouseDown {
                button: Button::Right,
                at: Point::new(1, 2),
                modifiers: Modifiers { alt: true, ..NONE },
            })
        );
        // While moving, the first button that is down.
        for (wparam, button) in [
            (MK_RBUTTON | MK_MBUTTON, Some(Button::Middle)),
            (MK_RBUTTON | MK_LBUTTON, Some(Button::Left)),
            (MK_RBUTTON | MK_SHIFT, Some(Button::Right)),
            (MK_SHIFT, None),
        ] {
            assert!(
                matches!(
                    decode(WM_MOUSEMOVE, wparam, xy(1, 2)),
                    Some(Event::MouseMoved { button: moved, .. }) if moved == button
                ),
                "{wparam:#x}"
            );
        }
    }

    #[test]
    fn the_wheel_turns_forward_or_back() {
        let turn = |delta: i16| (delta as u16 as WPARAM) << 16;
        assert_eq!(
            decode(WM_MOUSEWHEEL, turn(-240) | MK_CONTROL, xy(500, 300)),
            Some(Event::Wheel {
                at: Point::new(500, 300),
                delta: -240,
                horizontal: false,
                modifiers: Modifiers { ctrl: true, ..NONE },
            })
        );
        assert_eq!(
            decode(WM_MOUSEHWHEEL, turn(30), xy(-5, 7)),
            Some(Event::Wheel {
                at: Point::new(-5, 7),
                delta: 30,
                horizontal: true,
                modifiers: NONE,
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
