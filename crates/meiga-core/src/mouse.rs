//! The mouse: the cell under the pointer, the clicks and the turns of the
//! wheel as meiga counts them, and what it sends to a program that asked
//! for the mouse.
//!
//! A program asks for the clicks (`?1000`), also for the moves while a
//! button is down (`?1002`), or for every move (`?1003`), and for the
//! format of the reports: the one of the first xterm, or the same in UTF-8
//! (`?1005`), or SGR (`?1006`). The ConPTY gives the reports to the
//! programs of WSL as they are, so meiga writes them in the format that
//! the program asked for. To the programs of the console, the ConPTY asks
//! for SGR itself, and turns the reports into mouse events of Windows.

use std::time::{Duration, Instant};

use crate::geometry::{GridPoint, GridSize, Point, Size};
use crate::input::{ControlKeys, KeyEvent, push_decimal};
use crate::term::{Modes, Side};

/// What Windows counts as a whole notch of the wheel (`WHEEL_DELTA`).
pub const WHEEL_DELTA: i32 = 120;

/// The biggest column or row, from 0, that fits in a report in the format
/// of the first xterm: a byte with 33 more than it, without going over
/// 127. The ConPTY reads the input as UTF-8, and a byte from 128 alone
/// would reach the program as another character.
const MAX_X10: u16 = 127 - 33;

/// The biggest column or row, from 0, that fits in a report in UTF-8
/// (`?1005`): a character of up to two bytes, with 33 more than it.
const MAX_UTF8: u16 = 0x7ff - 33;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Left,
    Middle,
    Right,
}

/// The modifier keys that were down with a mouse event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

/// Where a notch of the wheel goes: forward is up, and a tilt to the right
/// is right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wheel {
    Up,
    Down,
    Left,
    Right,
}

/// What the mouse did, as a program receives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseAction {
    Press(Button),
    Release(Button),
    /// It moved to another cell, with a button down or with none.
    Move(Option<Button>),
    /// The wheel turned a notch.
    Wheel(Wheel),
}

/// Whether the program asked for the mouse, so that the clicks go to it
/// instead of selecting text.
pub const fn is_tracked(modes: &Modes) -> bool {
    modes.mouse_clicks || modes.mouse_drag || modes.mouse_motion
}

/// Whether the program asked for `action`.
pub const fn wants(action: MouseAction, modes: &Modes) -> bool {
    match action {
        MouseAction::Press(_) | MouseAction::Release(_) | MouseAction::Wheel(_) => {
            is_tracked(modes)
        }
        MouseAction::Move(Some(_)) => modes.mouse_drag || modes.mouse_motion,
        MouseAction::Move(None) => modes.mouse_motion,
    }
}

/// Adds to `out` the report of `action` on the cell `at`, with
/// `modifiers`, in the format that the program asked for. Returns whether
/// it added it: not if the program did not ask for `action`, nor if the
/// cell does not fit in the format.
pub fn report(
    action: MouseAction,
    at: GridPoint,
    modifiers: Modifiers,
    modes: &Modes,
    out: &mut Vec<u8>,
) -> bool {
    if !wants(action, modes) {
        return false;
    }
    let button = |button: Button| match button {
        Button::Left => 0,
        Button::Middle => 1,
        Button::Right => 2,
    };
    let mut code: u8 = match action {
        MouseAction::Press(pressed) => button(pressed),
        // Without SGR, a release does not say which button it was.
        MouseAction::Release(released) if modes.sgr_mouse => button(released),
        MouseAction::Release(_) => 3,
        MouseAction::Move(down) => 32 + down.map_or(3, button),
        MouseAction::Wheel(Wheel::Up) => 64,
        MouseAction::Wheel(Wheel::Down) => 65,
        MouseAction::Wheel(Wheel::Left) => 66,
        MouseAction::Wheel(Wheel::Right) => 67,
    };
    for (down, value) in [
        (modifiers.shift, 4),
        (modifiers.alt, 8),
        (modifiers.ctrl, 16),
    ] {
        if down {
            code += value;
        }
    }
    if modes.sgr_mouse {
        out.extend_from_slice(b"\x1b[<");
        push_decimal(out, u16::from(code));
        out.push(b';');
        push_decimal(out, at.column.saturating_add(1));
        out.push(b';');
        push_decimal(out, at.row.saturating_add(1));
        out.push(if matches!(action, MouseAction::Release(_)) {
            b'm'
        } else {
            b'M'
        });
        return true;
    }
    let max = if modes.utf8_mouse { MAX_UTF8 } else { MAX_X10 };
    if at.column > max || at.row > max {
        return false;
    }
    out.extend_from_slice(b"\x1b[M");
    // At most 67 + 4 + 8 + 16 + 32 = 127: it is always one byte.
    out.push(32 + code);
    for value in [at.column, at.row] {
        let value = u32::from(value) + 33;
        match char::from_u32(value) {
            Some(character) if value >= 0x80 => {
                let mut utf8 = [0; 4];
                out.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
            }
            // Up to 127, the byte, in both formats.
            _ => out.push(value as u8),
        }
    }
    true
}

/// The cell of a grid of `grid` cells of `cell` pixels under the point
/// `at`, in pixels from the top-left corner of the grid, and the side of
/// the cell where it is. A point outside the grid gives the nearest cell:
/// on the right of the grid, its right side.
pub fn cell_at(at: Point, cell: Size, grid: GridSize) -> (GridPoint, Side) {
    let width = i64::from(cell.width.max(1));
    let height = i64::from(cell.height.max(1));
    let (x, y) = (i64::from(at.x), i64::from(at.y));
    let column = (x.max(0) / width).min(i64::from(grid.columns.max(1)) - 1);
    let row = (y.max(0) / height).min(i64::from(grid.rows.max(1)) - 1);
    let side = if x >= width * i64::from(grid.columns) || x.max(0) % width > width / 2 {
        Side::Right
    } else {
        Side::Left
    };
    // Both are below the columns and the rows of the grid.
    (GridPoint::new(column as u16, row as u16), side)
}

/// When Windows takes two clicks as a double click: the double-click time
/// (`GetDoubleClickTime`), and the size of the rectangle around the first
/// click where the second one has to be (`SM_CXDOUBLECLK` and
/// `SM_CYDOUBLECLK`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClickSettings {
    pub time: Duration,
    pub width: u32,
    pub height: u32,
}

impl Default for ClickSettings {
    /// The ones of Windows by default.
    fn default() -> Self {
        Self {
            time: Duration::from_millis(500),
            width: 4,
            height: 4,
        }
    }
}

/// Counts the clicks of a button: the second one in a row is a double
/// click, and the third one a triple click.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clicks {
    last: Option<LastClick>,
}

#[derive(Clone, Copy, Debug)]
struct LastClick {
    button: Button,
    at: Point,
    when: Instant,
    count: u8,
}

impl Clicks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Counts a press of `button` at `at` and `now`: 1, or one more than
    /// the last one if it was of the same button, inside the double-click
    /// time and near it. After 3 comes 1 again.
    pub fn press(
        &mut self,
        button: Button,
        at: Point,
        now: Instant,
        settings: &ClickSettings,
    ) -> u8 {
        let count = match self.last {
            Some(last)
                if last.button == button
                    && now.saturating_duration_since(last.when) <= settings.time
                    && at.x.abs_diff(last.at.x) <= settings.width / 2
                    && at.y.abs_diff(last.at.y) <= settings.height / 2 =>
            {
                last.count % 3 + 1
            }
            _ => 1,
        };
        self.last = Some(LastClick {
            button,
            at,
            when: now,
            count,
        });
        count
    }

    /// Forgets the last click: the next one is a single click.
    pub fn reset(&mut self) {
        self.last = None;
    }
}

/// Adds up the turns of the wheel until they make whole notches: some
/// mice and touchpads turn it by less than a notch at a time.
#[derive(Clone, Copy, Debug, Default)]
pub struct WheelTurns {
    vertical: i32,
    horizontal: i32,
}

impl WheelTurns {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a turn of `delta`, in 120ths of a notch, forward or to the
    /// right when it is positive, and returns the whole notches that it
    /// completes. A turn in the other direction starts again from zero.
    pub fn add(&mut self, delta: i16, horizontal: bool) -> i32 {
        let total = if horizontal {
            &mut self.horizontal
        } else {
            &mut self.vertical
        };
        let delta = i32::from(delta);
        if (*total > 0 && delta < 0) || (*total < 0 && delta > 0) {
            *total = 0;
        }
        *total += delta;
        let notches = *total / WHEEL_DELTA;
        *total -= notches * WHEEL_DELTA;
        notches
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The arrow key of `wheel`, as it goes down and up, for the programs on
/// the alternate screen that did not ask for the mouse (`?1007`): they
/// scroll with the arrows.
pub const fn arrow_keys(wheel: Wheel) -> [KeyEvent; 2] {
    // The virtual-key code and the scan code of each arrow, outside the
    // numeric keypad.
    let (key, scan) = match wheel {
        Wheel::Up => (0x26, 0x48),
        Wheel::Down => (0x28, 0x50),
        Wheel::Left => (0x25, 0x4b),
        Wheel::Right => (0x27, 0x4d),
    };
    let down = KeyEvent {
        key,
        scan,
        character: 0,
        down: true,
        controls: ControlKeys::ENHANCED,
        repeat: 1,
    };
    let mut up = down;
    up.down = false;
    [down, up]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modes(change: impl FnOnce(&mut Modes)) -> Modes {
        let mut modes = Modes::default();
        change(&mut modes);
        modes
    }

    fn reported(
        action: MouseAction,
        at: GridPoint,
        modifiers: Modifiers,
        modes: &Modes,
    ) -> Vec<u8> {
        let mut out = Vec::new();
        let added = report(action, at, modifiers, modes, &mut out);
        assert_eq!(added, !out.is_empty());
        out
    }

    const NONE: Modifiers = Modifiers {
        shift: false,
        alt: false,
        ctrl: false,
    };

    #[test]
    fn the_program_gets_only_what_it_asked_for() {
        let at = GridPoint::new(0, 0);
        let press = MouseAction::Press(Button::Left);
        let drag = MouseAction::Move(Some(Button::Left));
        let hover = MouseAction::Move(None);
        let none = Modes::default();
        assert!(!is_tracked(&none));
        assert_eq!(reported(press, at, NONE, &none), b"");
        let clicks = modes(|modes| modes.mouse_clicks = true);
        assert!(is_tracked(&clicks));
        assert_eq!(reported(press, at, NONE, &clicks), b"\x1b[M !!");
        assert_eq!(reported(drag, at, NONE, &clicks), b"");
        let drags = modes(|modes| modes.mouse_drag = true);
        assert_eq!(reported(drag, at, NONE, &drags), b"\x1b[M@!!");
        assert_eq!(reported(hover, at, NONE, &drags), b"");
        let motion = modes(|modes| modes.mouse_motion = true);
        assert_eq!(reported(drag, at, NONE, &motion), b"\x1b[M@!!");
        assert_eq!(reported(hover, at, NONE, &motion), b"\x1b[MC!!");
    }

    #[test]
    fn sgr_reports_say_the_button_and_the_cell_from_1() {
        let sgr = modes(|modes| {
            modes.mouse_motion = true;
            modes.sgr_mouse = true;
        });
        let at = GridPoint::new(119, 29);
        let cases: [(MouseAction, Modifiers, &[u8]); 9] = [
            (MouseAction::Press(Button::Left), NONE, b"\x1b[<0;120;30M"),
            (MouseAction::Press(Button::Middle), NONE, b"\x1b[<1;120;30M"),
            (
                MouseAction::Release(Button::Right),
                NONE,
                b"\x1b[<2;120;30m",
            ),
            (
                MouseAction::Move(Some(Button::Left)),
                NONE,
                b"\x1b[<32;120;30M",
            ),
            (MouseAction::Move(None), NONE, b"\x1b[<35;120;30M"),
            (MouseAction::Wheel(Wheel::Up), NONE, b"\x1b[<64;120;30M"),
            (MouseAction::Wheel(Wheel::Right), NONE, b"\x1b[<67;120;30M"),
            (
                MouseAction::Press(Button::Left),
                Modifiers {
                    shift: true,
                    alt: true,
                    ctrl: true,
                },
                b"\x1b[<28;120;30M",
            ),
            (
                MouseAction::Wheel(Wheel::Down),
                Modifiers { ctrl: true, ..NONE },
                b"\x1b[<81;120;30M",
            ),
        ];
        for (action, modifiers, expected) in cases {
            assert_eq!(
                reported(action, at, modifiers, &sgr),
                expected,
                "{action:?} {modifiers:?}"
            );
        }
        // SGR has no limit.
        let far = GridPoint::new(999, 300);
        assert_eq!(
            reported(MouseAction::Press(Button::Left), far, NONE, &sgr),
            b"\x1b[<0;1000;301M"
        );
    }

    #[test]
    fn the_first_format_does_not_say_which_button_went_up_and_has_a_limit() {
        let x10 = modes(|modes| modes.mouse_clicks = true);
        let release = MouseAction::Release(Button::Right);
        assert_eq!(
            reported(release, GridPoint::new(2, 3), NONE, &x10),
            b"\x1b[M##$"
        );
        // Column 94, from 0: the byte 127.
        assert_eq!(
            reported(release, GridPoint::new(94, 0), NONE, &x10),
            b"\x1b[M#\x7f!"
        );
        assert_eq!(reported(release, GridPoint::new(95, 0), NONE, &x10), b"");
        assert_eq!(reported(release, GridPoint::new(0, 95), NONE, &x10), b"");
    }

    #[test]
    fn utf8_reports_reach_further() {
        let utf8 = modes(|modes| {
            modes.mouse_clicks = true;
            modes.utf8_mouse = true;
        });
        let press = MouseAction::Press(Button::Left);
        assert_eq!(
            reported(press, GridPoint::new(94, 0), NONE, &utf8),
            b"\x1b[M \x7f!"
        );
        // Column 95, from 0: the character 128, in two bytes.
        assert_eq!(
            reported(press, GridPoint::new(95, 0), NONE, &utf8),
            "\x1b[M \u{80}!".as_bytes()
        );
        assert_eq!(
            reported(press, GridPoint::new(MAX_UTF8, 0), NONE, &utf8),
            "\x1b[M \u{7ff}!".as_bytes()
        );
        assert_eq!(
            reported(press, GridPoint::new(MAX_UTF8 + 1, 0), NONE, &utf8),
            b""
        );
    }

    #[test]
    fn the_cell_under_the_pointer_and_its_side() {
        let cell = Size::new(10, 20);
        let grid = GridSize::new(8, 4);
        let at = |x, y| cell_at(Point::new(x, y), cell, grid);
        assert_eq!(at(0, 0), (GridPoint::new(0, 0), Side::Left));
        // The left half goes up to the middle, included.
        assert_eq!(at(5, 19), (GridPoint::new(0, 0), Side::Left));
        assert_eq!(at(6, 20), (GridPoint::new(0, 1), Side::Right));
        assert_eq!(at(79, 79), (GridPoint::new(7, 3), Side::Right));
        // Outside the grid, the nearest cell.
        assert_eq!(at(-30, -5), (GridPoint::new(0, 0), Side::Left));
        assert_eq!(at(80, 15), (GridPoint::new(7, 0), Side::Right));
        assert_eq!(at(500, 500), (GridPoint::new(7, 3), Side::Right));
        assert_eq!(at(31, 500), (GridPoint::new(3, 3), Side::Left));
    }

    #[test]
    fn clicks_in_a_row_count_up_to_three() {
        let settings = ClickSettings::default();
        let mut clicks = Clicks::new();
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        let here = Point::new(100, 50);
        assert_eq!(clicks.press(Button::Left, here, at(0), &settings), 1);
        assert_eq!(
            clicks.press(Button::Left, Point::new(102, 48), at(400), &settings),
            2
        );
        assert_eq!(clicks.press(Button::Left, here, at(800), &settings), 3);
        assert_eq!(clicks.press(Button::Left, here, at(1200), &settings), 1);
        // Too late.
        assert_eq!(clicks.press(Button::Left, here, at(1701), &settings), 1);
        // Too far.
        assert_eq!(
            clicks.press(Button::Left, Point::new(103, 50), at(1800), &settings),
            1
        );
        // Another button.
        assert_eq!(
            clicks.press(Button::Right, Point::new(103, 50), at(1900), &settings),
            1
        );
        assert_eq!(
            clicks.press(Button::Right, Point::new(103, 50), at(2000), &settings),
            2
        );
        clicks.reset();
        assert_eq!(
            clicks.press(Button::Right, Point::new(103, 50), at(2100), &settings),
            1
        );
    }

    #[test]
    fn the_wheel_adds_up_whole_notches() {
        let mut turns = WheelTurns::new();
        assert_eq!(turns.add(120, false), 1);
        assert_eq!(turns.add(-360, false), -3);
        // A touchpad: a notch in parts.
        assert_eq!(turns.add(50, false), 0);
        assert_eq!(turns.add(50, false), 0);
        assert_eq!(turns.add(50, false), 1);
        // The 30 that were left go when it turns back.
        assert_eq!(turns.add(-100, false), 0);
        assert_eq!(turns.add(-20, false), -1);
        // Vertical and horizontal apart.
        assert_eq!(turns.add(60, true), 0);
        assert_eq!(turns.add(60, false), 0);
        assert_eq!(turns.add(60, true), 1);
        turns.reset();
        assert_eq!(turns.add(60, false), 0);
    }

    #[test]
    fn the_wheel_on_the_alternate_screen_sends_arrows() {
        let [down, up] = arrow_keys(Wheel::Up);
        assert_eq!((down.key, down.scan, down.down), (0x26, 0x48, true));
        assert_eq!((up.key, up.scan, up.down), (0x26, 0x48, false));
        assert_eq!(down.controls, ControlKeys::ENHANCED);
        let mut encoder = crate::input::KeyEncoder::new();
        let mut out = Vec::new();
        encoder.encode(&arrow_keys(Wheel::Down)[0], &Modes::default(), &mut out);
        assert_eq!(out, b"\x1b[B");
        out.clear();
        let app_cursor = modes(|modes| modes.app_cursor = true);
        encoder.encode(&arrow_keys(Wheel::Down)[0], &app_cursor, &mut out);
        assert_eq!(out, b"\x1bOB");
    }
}
