//! The keys that meiga sends to the program.
//!
//! Windows describes each key that goes down or up with a key event: which
//! key it is, where it is on the keyboard, the character that it writes and
//! the modifier keys that were down. The ConPTY asks the terminal for them
//! whole, with win32-input-mode (`?9001`), and gives them to the programs
//! of the console as they are: so PSReadLine can tell Shift+Enter from
//! Enter. Until the ConPTY asks for them, or if it turns the mode off,
//! meiga sends what a VT terminal sends: the character of the key, or the
//! sequence of a key without one, such as an arrow.

use std::ops::{BitOr, BitOrAssign};

use crate::term::Modes;

const ESC: u8 = 0x1b;

/// The modifier keys and the lock keys of a key event, with the bits of the
/// `dwControlKeyState` of the key events of Windows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ControlKeys(u16);

impl ControlKeys {
    pub const RIGHT_ALT: Self = Self(0x0001);
    pub const LEFT_ALT: Self = Self(0x0002);
    pub const RIGHT_CTRL: Self = Self(0x0004);
    pub const LEFT_CTRL: Self = Self(0x0008);
    pub const SHIFT: Self = Self(0x0010);
    pub const NUM_LOCK: Self = Self(0x0020);
    pub const SCROLL_LOCK: Self = Self(0x0040);
    pub const CAPS_LOCK: Self = Self(0x0080);
    /// An extended key: one of the keys that the first PC keyboards did
    /// not have, such as the arrows, Insert, Delete, Home, End, Page Up and
    /// Page Down outside the numeric keypad, the right Ctrl and Alt, and the
    /// Enter and the / of the keypad.
    pub const ENHANCED: Self = Self(0x0100);

    pub const fn empty() -> Self {
        Self(0)
    }

    /// The bits, as Windows gives them.
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// Whether it has all the keys of `other`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether an Alt is down, the left one or the right one.
    pub const fn alt(self) -> bool {
        self.0 & (Self::LEFT_ALT.0 | Self::RIGHT_ALT.0) != 0
    }

    /// Whether a Ctrl is down.
    pub const fn ctrl(self) -> bool {
        self.0 & (Self::LEFT_CTRL.0 | Self::RIGHT_CTRL.0) != 0
    }

    pub const fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }
}

impl BitOr for ControlKeys {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for ControlKeys {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

/// A key that went down or up, as Windows describes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// The virtual-key code (`VK_…`), which says what the key is in the
    /// layout of the keyboard, or 0 for a character without a key.
    pub key: u16,
    /// The scan code, which says where the key is on the keyboard.
    pub scan: u16,
    /// The character that the key writes, as a UTF-16 code unit, or 0. A
    /// character outside the Basic Multilingual Plane, as most emoji, comes
    /// in two events, one with each half (surrogate).
    pub character: u16,
    /// Whether the key went down. Each key that goes down also goes up,
    /// with another event.
    pub down: bool,
    pub controls: ControlKeys,
    /// How many times it went down: more than once when the keyboard
    /// repeated it faster than the window took it.
    pub repeat: u16,
}

/// Turns key events into the bytes that the program reads.
///
/// It keeps the first half of a character outside the Basic Multilingual
/// Plane until the second one arrives, for the VT encoding.
#[derive(Debug, Default)]
pub struct KeyEncoder {
    high_surrogate: Option<u16>,
}

impl KeyEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds to `out` what the program reads for `event`, with the input
    /// modes that it turned on.
    pub fn encode(&mut self, event: &KeyEvent, modes: &Modes, out: &mut Vec<u8>) {
        if modes.win32_input {
            // The ConPTY joins the two halves of a character by itself.
            self.high_surrogate = None;
            push_win32(event, out);
            return;
        }
        // A VT terminal only sends something when a key goes down.
        if !event.down {
            return;
        }
        let start = out.len();
        self.push_vt(event, modes, out);
        let end = out.len();
        for _ in 1..event.repeat {
            out.extend_from_within(start..end);
        }
    }

    /// What a VT terminal sends when the key of `event` goes down once.
    fn push_vt(&mut self, event: &KeyEvent, modes: &Modes, out: &mut Vec<u8>) {
        // The first half of a character only waits for the second one.
        let high_surrogate = self.high_surrogate.take();
        let controls = event.controls;
        let (alt, ctrl) = (controls.alt(), controls.ctrl());
        // The parameter of the modifiers in the sequences of xterm: 1, plus
        // 1 for Shift, 2 for Alt and 4 for Ctrl.
        let modifiers = 1 + u16::from(controls.shift()) + 2 * u16::from(alt) + 4 * u16::from(ctrl);
        // Alt adds ESC before the keys that have no parameter for it.
        let with_alt = |out: &mut Vec<u8>, bytes: &[u8]| {
            if alt {
                out.push(ESC);
            }
            out.extend_from_slice(bytes);
        };
        match event.key {
            vk::UP => push_cursor_key(out, b'A', modifiers, modes.app_cursor),
            vk::DOWN => push_cursor_key(out, b'B', modifiers, modes.app_cursor),
            vk::RIGHT => push_cursor_key(out, b'C', modifiers, modes.app_cursor),
            vk::LEFT => push_cursor_key(out, b'D', modifiers, modes.app_cursor),
            vk::HOME => push_cursor_key(out, b'H', modifiers, modes.app_cursor),
            vk::END => push_cursor_key(out, b'F', modifiers, modes.app_cursor),
            vk::INSERT => push_tilde_key(out, 2, modifiers),
            vk::DELETE => push_tilde_key(out, 3, modifiers),
            vk::PRIOR => push_tilde_key(out, 5, modifiers),
            vk::NEXT => push_tilde_key(out, 6, modifiers),
            // F1 to F4 always go as in the application cursor mode.
            vk::F1..=vk::F4 => {
                let last = b'P' + (event.key - vk::F1) as u8;
                push_cursor_key(out, last, modifiers, true);
            }
            vk::F5..=vk::F12 => {
                let number = F5_TO_F12[usize::from(event.key - vk::F5)];
                push_tilde_key(out, number, modifiers);
            }
            // Backspace sends DEL, and Ctrl+Backspace, BS.
            vk::BACK => with_alt(out, if ctrl { b"\x08" } else { b"\x7f" }),
            vk::TAB if controls.shift() => out.extend_from_slice(b"\x1b[Z"),
            vk::TAB => with_alt(out, b"\t"),
            vk::RETURN => with_alt(
                out,
                if ctrl {
                    b"\n"
                } else if modes.line_feed_new_line {
                    b"\r\n"
                } else {
                    b"\r"
                },
            ),
            vk::ESCAPE => with_alt(out, &[ESC]),
            vk::PAUSE => out.push(0x1a),
            // Ctrl+Break.
            vk::CANCEL => out.push(0x03),
            _ => self.push_vt_character(event, high_surrogate, out),
        }
    }

    /// The character of the key, or the control code of Ctrl with a key
    /// that has none.
    fn push_vt_character(
        &mut self,
        event: &KeyEvent,
        high_surrogate: Option<u16>,
        out: &mut Vec<u8>,
    ) {
        let (alt, ctrl) = (event.controls.alt(), event.controls.ctrl());
        let unit = event.character;
        // Windows writes a space for Ctrl+Space, and nothing for some keys
        // with Ctrl, such as Ctrl+2: a VT terminal sends control codes.
        if ctrl && (unit == 0 || unit == u16::from(b' ')) {
            if let Some(code) = control_code(event.key) {
                if alt {
                    out.push(ESC);
                }
                out.push(code);
            }
            return;
        }
        let character = match unit {
            0 => return,
            0xd800..=0xdbff => {
                self.high_surrogate = Some(unit);
                return;
            }
            0xdc00..=0xdfff => {
                // A second half without its first one is lost.
                let Some(high) = high_surrogate else {
                    return;
                };
                let code = 0x10000 + (u32::from(high - 0xd800) << 10) + u32::from(unit - 0xdc00);
                char::from_u32(code)
            }
            _ => char::from_u32(u32::from(unit)),
        };
        let Some(character) = character else {
            return;
        };
        // AltGr writes its characters with Ctrl and Alt down: they go as
        // they are. With Alt, a VT terminal sends ESC first.
        let alt_gr = ctrl && alt && character > ' ' && character != '\x7f';
        if alt && !alt_gr {
            out.push(ESC);
        }
        let mut utf8 = [0; 4];
        out.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
    }
}

/// What tells the program that the window gained or lost the focus: nothing
/// if it did not ask for it (`?1004`).
pub fn focus_sequence(focused: bool, modes: &Modes) -> &'static [u8] {
    match (modes.focus_events, focused) {
        (false, _) => b"",
        (true, true) => b"\x1b[I",
        (true, false) => b"\x1b[O",
    }
}

/// The virtual-key codes of Windows that the VT encoding treats apart.
mod vk {
    pub const CANCEL: u16 = 0x03;
    pub const BACK: u16 = 0x08;
    pub const TAB: u16 = 0x09;
    pub const RETURN: u16 = 0x0d;
    pub const PAUSE: u16 = 0x13;
    pub const ESCAPE: u16 = 0x1b;
    pub const SPACE: u16 = 0x20;
    pub const PRIOR: u16 = 0x21;
    pub const NEXT: u16 = 0x22;
    pub const END: u16 = 0x23;
    pub const HOME: u16 = 0x24;
    pub const LEFT: u16 = 0x25;
    pub const UP: u16 = 0x26;
    pub const RIGHT: u16 = 0x27;
    pub const DOWN: u16 = 0x28;
    pub const INSERT: u16 = 0x2d;
    pub const DELETE: u16 = 0x2e;
    pub const F1: u16 = 0x70;
    pub const F4: u16 = 0x73;
    pub const F5: u16 = 0x74;
    pub const F12: u16 = 0x7b;
}

/// The numbers of F5 to F12 in their sequences, `ESC [ n ~`, which skip
/// some numbers for historical reasons.
const F5_TO_F12: [u16; 8] = [15, 17, 18, 19, 20, 21, 23, 24];

/// The win32-input-mode sequence of `event`:
/// `ESC [ Vk ; Sc ; Uc ; Kd ; Cs ; Rc _`, with all its numbers, as Windows
/// Terminal sends it.
fn push_win32(event: &KeyEvent, out: &mut Vec<u8>) {
    out.extend_from_slice(b"\x1b[");
    for number in [
        event.key,
        event.scan,
        event.character,
        u16::from(event.down),
        event.controls.bits(),
    ] {
        push_decimal(out, number);
        out.push(b';');
    }
    // The ConPTY stores a 0 as it is, and a program could lose the key.
    push_decimal(out, event.repeat.max(1));
    out.push(b'_');
}

/// An arrow, Home, End or F1 to F4: `ESC [ X`, or `ESC O X` in the
/// application cursor mode, and `ESC [ 1 ; m X` with modifiers.
fn push_cursor_key(out: &mut Vec<u8>, last: u8, modifiers: u16, application: bool) {
    if modifiers > 1 {
        out.extend_from_slice(b"\x1b[1;");
        push_decimal(out, modifiers);
    } else if application {
        out.extend_from_slice(b"\x1bO");
    } else {
        out.extend_from_slice(b"\x1b[");
    }
    out.push(last);
}

/// Insert, Delete, Page Up, Page Down and F5 to F12: `ESC [ n ~`, and
/// `ESC [ n ; m ~` with modifiers.
fn push_tilde_key(out: &mut Vec<u8>, number: u16, modifiers: u16) {
    out.extend_from_slice(b"\x1b[");
    push_decimal(out, number);
    if modifiers > 1 {
        out.push(b';');
        push_decimal(out, modifiers);
    }
    out.push(b'~');
}

/// The control code of a VT terminal for Ctrl and `key`, for the keys that
/// have none with Ctrl in Windows.
fn control_code(key: u16) -> Option<u8> {
    Some(match key {
        // Ctrl+Space and Ctrl+2, which is Ctrl+@: NUL.
        vk::SPACE | 0x32 => 0x00,
        // Ctrl+3 to Ctrl+7, which are Ctrl+[, \, ], ^ and _: ESC to US.
        0x33..=0x37 => (key - 0x33) as u8 + ESC,
        // Ctrl+8: DEL.
        0x38 => 0x7f,
        // Ctrl+A to Ctrl+Z, when the layout gives no character with them,
        // as with Ctrl+Alt: 1 to 26.
        0x41..=0x5a => (key - 0x40) as u8,
        _ => return None,
    })
}

/// Adds `value` in decimal, without allocating.
fn push_decimal(out: &mut Vec<u8>, value: u16) {
    let mut digits = [0; 5];
    let mut start = digits.len();
    let mut rest = value;
    loop {
        start -= 1;
        digits[start] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    out.extend_from_slice(&digits[start..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: ControlKeys = ControlKeys::empty();
    const SHIFT: ControlKeys = ControlKeys::SHIFT;
    const ALT: ControlKeys = ControlKeys::LEFT_ALT;
    const CTRL: ControlKeys = ControlKeys::LEFT_CTRL;
    /// AltGr: Windows adds a left Ctrl to the right Alt.
    const ALT_GR: ControlKeys = ControlKeys(ControlKeys::LEFT_CTRL.0 | ControlKeys::RIGHT_ALT.0);
    const ENHANCED: ControlKeys = ControlKeys::ENHANCED;

    fn down(key: u16, scan: u16, character: u16, controls: ControlKeys) -> KeyEvent {
        KeyEvent {
            key,
            scan,
            character,
            down: true,
            controls,
            repeat: 1,
        }
    }

    fn up(key: u16, scan: u16, character: u16, controls: ControlKeys) -> KeyEvent {
        KeyEvent {
            down: false,
            ..down(key, scan, character, controls)
        }
    }

    /// A key that goes down, as the VT encoding reads it: without a scan
    /// code.
    fn key(key: u16, character: char, controls: ControlKeys) -> KeyEvent {
        down(key, 0, character as u16, controls)
    }

    fn encode(events: &[KeyEvent], modes: Modes) -> Vec<u8> {
        let mut encoder = KeyEncoder::new();
        let mut out = Vec::new();
        for event in events {
            encoder.encode(event, &modes, &mut out);
        }
        out
    }

    fn vt(events: &[KeyEvent]) -> Vec<u8> {
        encode(events, Modes::default())
    }

    fn win32(event: KeyEvent) -> String {
        let modes = Modes {
            win32_input: true,
            ..Modes::default()
        };
        String::from_utf8(encode(&[event], modes)).unwrap()
    }

    #[test]
    fn win32_input_mode_sends_the_whole_key_events() {
        // Key events that Windows Terminal sent with the Spanish layout.
        assert_eq!(
            win32(down(0x41, 30, 'a' as u16, NONE)),
            "\x1b[65;30;97;1;0;1_"
        );
        assert_eq!(
            win32(up(0x41, 30, 'a' as u16, NONE)),
            "\x1b[65;30;97;0;0;1_"
        );
        // AltGr+2 writes @.
        assert_eq!(
            win32(down(0x32, 3, '@' as u16, ALT_GR)),
            "\x1b[50;3;64;1;9;1_"
        );
        // Shift+Enter, which a VT terminal cannot tell from Enter.
        assert_eq!(win32(down(0x0d, 28, 13, SHIFT)), "\x1b[13;28;13;1;16;1_");
        assert_eq!(win32(down(0x25, 75, 0, ENHANCED)), "\x1b[37;75;0;1;256;1_");
        // A dead key goes down without a character.
        assert_eq!(win32(down(0xde, 40, 0, NONE)), "\x1b[222;40;0;1;0;1_");
        // Even keys that a VT terminal does not send.
        assert_eq!(win32(down(0x10, 42, 0, SHIFT)), "\x1b[16;42;0;1;16;1_");
        let repeated = KeyEvent {
            repeat: 3,
            ..down(0x41, 30, 'a' as u16, NONE)
        };
        assert_eq!(win32(repeated), "\x1b[65;30;97;1;0;3_");
        let largest = KeyEvent {
            repeat: 0,
            ..down(u16::MAX, u16::MAX, u16::MAX, ControlKeys(u16::MAX))
        };
        assert_eq!(win32(largest), "\x1b[65535;65535;65535;1;65535;1_");
    }

    #[test]
    fn vt_sends_the_characters_in_utf8_when_keys_go_down() {
        assert_eq!(vt(&[key(0x41, 'a', NONE)]), b"a");
        assert_eq!(vt(&[key(0xc0, 'ñ', NONE)]), "ñ".as_bytes());
        assert_eq!(vt(&[key(0x45, '€', ALT_GR)]), "€".as_bytes());
        // The dead key writes nothing, and the next key its character.
        let dead = [key(0xde, '\0', NONE), key(0x41, 'á', NONE)];
        assert_eq!(vt(&dead), "á".as_bytes());
        assert_eq!(vt(&[up(0x41, 30, 'a' as u16, NONE)]), b"");
        assert_eq!(vt(&[key(0x10, '\0', SHIFT)]), b"");
    }

    #[test]
    fn vt_joins_the_two_halves_of_a_character() {
        // 😀 is U+1F600: D83D DE00 in UTF-16.
        let first = down(0xe7, 0, 0xd83d, NONE);
        let second = down(0xe7, 0, 0xde00, NONE);
        assert_eq!(vt(&[first]), b"");
        assert_eq!(vt(&[first, second]), "😀".as_bytes());
        // The halves go down and up as two keys.
        let first_up = KeyEvent {
            down: false,
            ..first
        };
        assert_eq!(vt(&[first, first_up, second]), "😀".as_bytes());
        let with_alt = [
            KeyEvent {
                controls: ALT,
                ..first
            },
            KeyEvent {
                controls: ALT,
                ..second
            },
        ];
        assert_eq!(vt(&with_alt), "\x1b😀".as_bytes());
        // A half without the other one is lost.
        assert_eq!(vt(&[second]), b"");
        assert_eq!(vt(&[first, key(0x41, 'a', NONE), second]), b"a");
    }

    #[test]
    fn vt_cursor_keys_follow_the_application_mode_and_the_modifiers() {
        assert_eq!(vt(&[key(0x26, '\0', ENHANCED)]), b"\x1b[A");
        assert_eq!(vt(&[key(0x24, '\0', NONE)]), b"\x1b[H");
        let application = Modes {
            app_cursor: true,
            ..Modes::default()
        };
        assert_eq!(encode(&[key(0x26, '\0', NONE)], application), b"\x1bOA");
        assert_eq!(encode(&[key(0x23, '\0', NONE)], application), b"\x1bOF");
        assert_eq!(vt(&[key(0x25, '\0', CTRL)]), b"\x1b[1;5D");
        assert_eq!(vt(&[key(0x27, '\0', SHIFT | ALT)]), b"\x1b[1;4C");
        assert_eq!(encode(&[key(0x28, '\0', CTRL)], application), b"\x1b[1;5B");
    }

    #[test]
    fn vt_editing_and_function_keys_send_their_sequences() {
        assert_eq!(vt(&[key(0x2d, '\0', NONE)]), b"\x1b[2~");
        assert_eq!(vt(&[key(0x2e, '\0', NONE)]), b"\x1b[3~");
        assert_eq!(vt(&[key(0x2e, '\0', CTRL)]), b"\x1b[3;5~");
        assert_eq!(vt(&[key(0x21, '\0', NONE)]), b"\x1b[5~");
        assert_eq!(vt(&[key(0x22, '\0', SHIFT)]), b"\x1b[6;2~");
        assert_eq!(vt(&[key(0x70, '\0', NONE)]), b"\x1bOP");
        assert_eq!(vt(&[key(0x73, '\0', NONE)]), b"\x1bOS");
        assert_eq!(vt(&[key(0x70, '\0', SHIFT)]), b"\x1b[1;2P");
        assert_eq!(vt(&[key(0x74, '\0', NONE)]), b"\x1b[15~");
        assert_eq!(vt(&[key(0x79, '\0', NONE)]), b"\x1b[21~");
        assert_eq!(vt(&[key(0x7b, '\0', CTRL)]), b"\x1b[24;5~");
    }

    #[test]
    fn vt_backspace_tab_enter_and_escape_send_control_codes() {
        // Windows writes BS for Backspace, and a VT terminal sends DEL.
        assert_eq!(vt(&[key(0x08, '\x08', NONE)]), b"\x7f");
        assert_eq!(vt(&[key(0x08, '\x7f', CTRL)]), b"\x08");
        assert_eq!(vt(&[key(0x08, '\x08', ALT)]), b"\x1b\x7f");
        assert_eq!(vt(&[key(0x09, '\t', NONE)]), b"\t");
        assert_eq!(vt(&[key(0x09, '\t', SHIFT)]), b"\x1b[Z");
        assert_eq!(vt(&[key(0x0d, '\r', NONE)]), b"\r");
        assert_eq!(vt(&[key(0x0d, '\r', SHIFT)]), b"\r");
        assert_eq!(vt(&[key(0x0d, '\n', CTRL)]), b"\n");
        assert_eq!(vt(&[key(0x0d, '\r', ALT)]), b"\x1b\r");
        let line_feed = Modes {
            line_feed_new_line: true,
            ..Modes::default()
        };
        assert_eq!(encode(&[key(0x0d, '\r', NONE)], line_feed), b"\r\n");
        assert_eq!(vt(&[key(0x1b, '\x1b', NONE)]), b"\x1b");
        assert_eq!(vt(&[key(0x1b, '\x1b', ALT)]), b"\x1b\x1b");
        assert_eq!(vt(&[key(0x13, '\0', NONE)]), b"\x1a");
        assert_eq!(vt(&[key(0x03, '\x03', CTRL)]), b"\x03");
    }

    #[test]
    fn vt_ctrl_sends_control_codes() {
        // Windows already gives the control code of Ctrl+letter.
        assert_eq!(vt(&[key(0x43, '\x03', CTRL)]), b"\x03");
        assert_eq!(vt(&[key(0x20, ' ', CTRL)]), b"\0");
        assert_eq!(vt(&[key(0x20, '\0', CTRL)]), b"\0");
        assert_eq!(vt(&[key(0x32, '\0', CTRL)]), b"\0");
        assert_eq!(vt(&[key(0x33, '\0', CTRL)]), b"\x1b");
        assert_eq!(vt(&[key(0x37, '\0', CTRL)]), b"\x1f");
        assert_eq!(vt(&[key(0x38, '\0', CTRL)]), b"\x7f");
        // Ctrl+Alt+A, without a character in the layout.
        assert_eq!(vt(&[key(0x41, '\0', CTRL | ALT)]), b"\x1b\x01");
        assert_eq!(vt(&[key(0x41, '\x01', CTRL | ALT)]), b"\x1b\x01");
        // A key without a control code sends nothing.
        assert_eq!(vt(&[key(0xc0, '\0', CTRL)]), b"");
    }

    #[test]
    fn vt_alt_sends_escape_first_but_altgr_does_not() {
        assert_eq!(vt(&[key(0x41, 'a', ALT)]), b"\x1ba");
        assert_eq!(vt(&[key(0x32, '@', ALT_GR)]), b"@");
        // Ctrl+Alt also writes the characters of AltGr.
        assert_eq!(vt(&[key(0x32, '@', CTRL | ALT)]), b"@");
        assert_eq!(vt(&[key(0xba, '[', ALT_GR)]), b"[");
    }

    #[test]
    fn vt_repeats_the_keys_that_repeated() {
        let repeated = |event: KeyEvent| KeyEvent { repeat: 3, ..event };
        assert_eq!(vt(&[repeated(key(0x41, 'a', NONE))]), b"aaa");
        assert_eq!(
            vt(&[repeated(key(0x25, '\0', NONE))]),
            b"\x1b[D\x1b[D\x1b[D"
        );
        let none = KeyEvent {
            repeat: 0,
            ..key(0x41, 'a', NONE)
        };
        assert_eq!(vt(&[none]), b"a");
    }

    #[test]
    fn focus_events_only_go_if_the_program_asks_for_them() {
        let asked = Modes {
            focus_events: true,
            ..Modes::default()
        };
        assert_eq!(focus_sequence(true, &asked), b"\x1b[I");
        assert_eq!(focus_sequence(false, &asked), b"\x1b[O");
        assert_eq!(focus_sequence(true, &Modes::default()), b"");
    }
}
