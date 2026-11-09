//! The keyboard of a window: its key messages, as whole key events.

use std::cell::Cell;

use meiga_core::input::{ControlKeys, KeyEvent};
use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardLayout, GetKeyboardState, MAPVK_VSC_TO_VK_EX, MapVirtualKeyW, ToUnicodeEx,
    VK_CAPITAL, VK_F4, VK_LCONTROL, VK_LMENU, VK_LWIN, VK_NUMLOCK, VK_PROCESSKEY, VK_RCONTROL,
    VK_RMENU, VK_RWIN, VK_SCROLL, VK_SHIFT, VK_SPACE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MSG, PM_NOREMOVE, PM_REMOVE, PeekMessageW, WM_CHAR, WM_DEADCHAR, WM_KEYDOWN, WM_KEYUP,
    WM_SYSCHAR, WM_SYSDEADCHAR, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

/// The keyboard messages: the keys that go down and up, and their
/// characters.
pub(crate) const fn is_keyboard_message(message: u32) -> bool {
    matches!(
        message,
        WM_KEYDOWN
            | WM_KEYUP
            | WM_SYSKEYDOWN
            | WM_SYSKEYUP
            | WM_CHAR
            | WM_DEADCHAR
            | WM_SYSCHAR
            | WM_SYSDEADCHAR
    )
}

/// Who handles a keyboard message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Destination {
    /// The program: the events of its keys go to the handler of the window,
    /// now or when their characters arrive.
    Program,
    /// Windows (`DefWindowProcW`): Alt+F4 closes the window, and Alt+Space
    /// opens its menu.
    Windows,
}

/// Turns the keyboard messages of a window into key events, as the console
/// of Windows does.
///
/// Windows sends a message when a key goes down or up and, if it writes a
/// character, `TranslateMessage` posts another message with it. The message
/// loop calls `TranslateMessage` before it gives the key message to the
/// window, so the character is already in the queue, and it is the next
/// message that the window gets: the event of the key waits for it.
///
/// A program can also post key messages to the window, as automation tools
/// do. Then the next key message can come before the character of the
/// first key, which is the first character in the queue: the event of the
/// first key takes it from there, and goes first.
#[derive(Default)]
pub(crate) struct Keyboard {
    waiting: Cell<Option<Waiting>>,
}

/// The event of a key that waits for its character.
#[derive(Clone, Copy)]
struct Waiting {
    event: KeyEvent,
    /// Whether the character comes in `WM_SYSCHAR` or `WM_SYSDEADCHAR`,
    /// because Alt is down.
    system: bool,
}

impl Keyboard {
    /// A keyboard message ([`is_keyboard_message`]) of `window`. The events
    /// that it gives, none, one or two, go to `send`.
    pub(crate) fn message(
        &self,
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        send: &mut dyn FnMut(KeyEvent),
    ) -> Destination {
        self.handle(&Windows(window), message, wparam, lparam, send)
    }

    fn handle(
        &self,
        system: &impl System,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        send: &mut dyn FnMut(KeyEvent),
    ) -> Destination {
        // The virtual-key code, or the UTF-16 unit of the character, is in
        // the low 16 bits; the details of the key, in the low 32 bits.
        let (value, bits) = (wparam as u16, KeyBits(lparam as u32));
        match message {
            WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {
                self.key(system, message, value, bits, send)
            }
            _ => self.character(system, message, value, bits, send),
        }
    }

    /// A key went down or up.
    fn key(
        &self,
        system: &impl System,
        message: u32,
        key: u16,
        bits: KeyBits,
        send: &mut dyn FnMut(KeyEvent),
    ) -> Destination {
        if let Some(waiting) = self.waiting.take() {
            let character = system.take_character(waiting.system);
            send(KeyEvent {
                character,
                ..waiting.event
            });
        }
        let keys = system.key_state();
        let controls = control_keys(&keys, bits.extended());
        if matches!(message, WM_SYSKEYDOWN | WM_SYSKEYUP) && for_windows(key, controls) {
            return Destination::Windows;
        }
        // The Windows keys are for the shortcuts of Windows, as in Windows
        // Terminal, and the keys that an IME takes, for the IME.
        if matches!(key, 0 | VK_LWIN | VK_RWIN | VK_PROCESSKEY) || key >= 0xff {
            return Destination::Program;
        }
        let mut event = KeyEvent {
            key,
            scan: bits.scan(),
            character: 0,
            down: !bits.up(),
            controls,
            repeat: bits.repeat(),
        };
        if let Some(system) = system.posted_character() {
            self.waiting.set(Some(Waiting { event, system }));
            return Destination::Program;
        }
        // A key that goes up writes nothing, but its event says what it
        // writes, as in Windows Terminal.
        if !event.down {
            event.character = system.character(key, event.scan, &keys);
        }
        send(event);
        Destination::Program
    }

    /// The character of a key.
    fn character(
        &self,
        system: &impl System,
        message: u32,
        unit: u16,
        bits: KeyBits,
        send: &mut dyn FnMut(KeyEvent),
    ) -> Destination {
        // The space of Alt+Space opens the menu of the window, as its key.
        if message == WM_SYSCHAR && unit == u16::from(b' ') {
            let controls = control_keys(&system.key_state(), bits.extended());
            if for_windows(VK_SPACE, controls) {
                return Destination::Windows;
            }
        }
        // A dead key writes nothing yet: its character goes with the next
        // key, as in the console of Windows.
        let dead = matches!(message, WM_DEADCHAR | WM_SYSDEADCHAR);
        let character = if dead { 0 } else { unit };
        if let Some(waiting) = self.waiting.take() {
            send(KeyEvent {
                character,
                ..waiting.event
            });
        } else if !dead {
            // A character without a key waiting for it: the second one of a
            // dead key and a key that do not join (´ and x write ´x), or one
            // from an IME. The key comes from the scan code, if it has one.
            let scan = bits.scan();
            let key = if scan == 0 {
                0
            } else {
                system.key_at(scan, bits.extended())
            };
            send(KeyEvent {
                key,
                scan,
                character,
                down: !bits.up(),
                controls: control_keys(&system.key_state(), bits.extended()),
                repeat: bits.repeat(),
            });
        }
        Destination::Program
    }
}

/// Whether Windows handles a key with Alt: Alt+F4 closes the window, and
/// Alt+Space opens its menu, as in Windows Terminal. Alt alone, F10 and the
/// other keys with Alt go to the program.
fn for_windows(key: u16, controls: ControlKeys) -> bool {
    controls.alt() && !controls.ctrl() && matches!(key, VK_F4 | VK_SPACE)
}

/// The modifier and lock keys of a key message.
fn control_keys(keys: &KeyState, extended: bool) -> ControlKeys {
    let mut controls = ControlKeys::empty();
    for (key, control) in [
        (VK_RMENU, ControlKeys::RIGHT_ALT),
        (VK_LMENU, ControlKeys::LEFT_ALT),
        (VK_RCONTROL, ControlKeys::RIGHT_CTRL),
        (VK_LCONTROL, ControlKeys::LEFT_CTRL),
        (VK_SHIFT, ControlKeys::SHIFT),
    ] {
        if keys.down(key) {
            controls |= control;
        }
    }
    for (key, control) in [
        (VK_NUMLOCK, ControlKeys::NUM_LOCK),
        (VK_SCROLL, ControlKeys::SCROLL_LOCK),
        (VK_CAPITAL, ControlKeys::CAPS_LOCK),
    ] {
        if keys.toggled(key) {
            controls |= control;
        }
    }
    if extended {
        controls |= ControlKeys::ENHANCED;
    }
    controls
}

/// The `lParam` of a key or character message.
#[derive(Clone, Copy)]
struct KeyBits(u32);

impl KeyBits {
    /// How many times the key went down: 1, unless the keyboard repeated it
    /// faster than the window took it.
    fn repeat(self) -> u16 {
        (self.0 as u16).max(1)
    }

    fn scan(self) -> u16 {
        ((self.0 >> 16) & 0xff) as u16
    }

    fn extended(self) -> bool {
        self.0 & (1 << 24) != 0
    }

    /// Whether the key goes up: its transition state.
    fn up(self) -> bool {
        self.0 & (1 << 31) != 0
    }
}

/// The state of every key, as of the message that the window handles:
/// `GetKeyboardState`.
struct KeyState([u8; 256]);

impl KeyState {
    fn down(&self, key: u16) -> bool {
        self.0[usize::from(key & 0xff)] & 0x80 != 0
    }

    /// For the lock keys, whether they are on.
    fn toggled(&self, key: u16) -> bool {
        self.0[usize::from(key & 0xff)] & 0x01 != 0
    }
}

/// What the keyboard asks Windows. The tests answer for it.
trait System {
    fn key_state(&self) -> KeyState;

    /// Whether a character is waiting in the queue: `Some(true)` if it comes
    /// in `WM_SYSCHAR` or `WM_SYSDEADCHAR`, and `Some(false)` in `WM_CHAR`
    /// or `WM_DEADCHAR`.
    fn posted_character(&self) -> Option<bool>;

    /// Takes from the queue the first character of `WM_SYSCHAR` and
    /// `WM_SYSDEADCHAR` if `system`, or of `WM_CHAR` and `WM_DEADCHAR`: its
    /// UTF-16 unit, or 0 for a dead key or if there is none.
    fn take_character(&self, system: bool) -> u16;

    /// What a key writes, with the modifier and lock keys of `keys`, or 0.
    fn character(&self, key: u16, scan: u16, keys: &KeyState) -> u16;

    /// The virtual-key code of the key at a scan code, or 0.
    fn key_at(&self, scan: u16, extended: bool) -> u16;
}

/// The character messages of the keys with Alt, or of the others.
const fn character_messages(system: bool) -> (u32, u32) {
    if system {
        (WM_SYSCHAR, WM_SYSDEADCHAR)
    } else {
        (WM_CHAR, WM_DEADCHAR)
    }
}

/// Windows, for the messages of a window.
struct Windows(HWND);

impl Windows {
    /// The first character message of the queue between `first` and
    /// `last`, taken out of it with `PM_REMOVE`, or left with `PM_NOREMOVE`.
    fn peek(&self, (first, last): (u32, u32), remove: u32) -> Option<MSG> {
        let mut message = MSG::default();
        // SAFETY: `message` is a valid MSG for PeekMessageW to fill in. The
        // range only has character messages, which are posted, not input.
        let found = unsafe { PeekMessageW(&mut message, self.0, first, last, remove) } != 0;
        found.then_some(message)
    }
}

impl System for Windows {
    fn key_state(&self) -> KeyState {
        let mut keys = KeyState([0; 256]);
        // SAFETY: GetKeyboardState fills in the 256 bytes of the array. If it
        // fails, they stay 0: every key is up.
        unsafe { GetKeyboardState(keys.0.as_mut_ptr()) };
        keys
    }

    fn posted_character(&self) -> Option<bool> {
        [false, true]
            .into_iter()
            .find(|&system| self.peek(character_messages(system), PM_NOREMOVE).is_some())
    }

    fn take_character(&self, system: bool) -> u16 {
        let (first, last) = character_messages(system);
        match self.peek((first, last), PM_REMOVE) {
            // The UTF-16 unit is in the low 16 bits.
            Some(message) if message.message == first => message.wParam as u16,
            _ => 0,
        }
    }

    fn character(&self, key: u16, scan: u16, keys: &KeyState) -> u16 {
        let mut buffer = [0_u16; 2];
        // SAFETY: the state has the 256 bytes of every key, and the buffer
        // has room for the 2 units that it says. Bit 0 of the flags: as if a
        // menu were open, so that the keys of the keypad with Alt do not
        // count for a character code. Bit 2: the state of the keyboard does
        // not change, as a dead key that went down before (Windows 10 1607
        // and later). GetKeyboardLayout(0): the layout of this thread.
        let written = unsafe {
            ToUnicodeEx(
                u32::from(key),
                u32::from(scan),
                keys.0.as_ptr(),
                buffer.as_mut_ptr(),
                2,
                0b101,
                GetKeyboardLayout(0),
            )
        };
        // -1: a dead key. 2: a dead key went down before, and the key does
        // not join with it: it is not the character of this key alone.
        if matches!(written, 1 | -1) {
            buffer[0]
        } else {
            0
        }
    }

    fn key_at(&self, scan: u16, extended: bool) -> u16 {
        // The scan codes of the extended keys start with E0.
        let code = u32::from(scan) | if extended { 0xe000 } else { 0 };
        // SAFETY: MapVirtualKeyW has no preconditions, and returns 0 if no
        // key has the scan code.
        let key = unsafe { MapVirtualKeyW(code, MAPVK_VSC_TO_VK_EX) };
        u16::try_from(key).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;

    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        VK_CONTROL, VK_F10, VK_LEFT, VK_MENU, VK_OEM_7,
    };

    use super::*;

    /// Windows, with the keys that the test presses, and a queue with the
    /// character messages that `TranslateMessage` posts.
    #[derive(Default)]
    struct Fake {
        /// The keys that are down, and the lock keys that are on.
        down: Vec<u16>,
        toggled: Vec<u16>,
        queue: RefCell<VecDeque<(u32, u16)>>,
        /// What every key writes when it goes up.
        character: u16,
        /// The key of every scan code.
        key_at: u16,
    }

    impl Fake {
        /// `TranslateMessage` posts a character message.
        fn post(&self, message: u32, unit: u16) {
            self.queue.borrow_mut().push_back((message, unit));
        }

        /// The message loop takes the next character message, if any.
        fn next(&self) -> Option<(u32, u16)> {
            self.queue.borrow_mut().pop_front()
        }
    }

    impl System for Fake {
        fn key_state(&self) -> KeyState {
            let mut keys = KeyState([0; 256]);
            for &key in &self.down {
                keys.0[usize::from(key)] |= 0x80;
            }
            for &key in &self.toggled {
                keys.0[usize::from(key)] |= 0x01;
            }
            keys
        }

        fn posted_character(&self) -> Option<bool> {
            let queue = self.queue.borrow();
            [false, true].into_iter().find(|&system| {
                let (first, last) = character_messages(system);
                queue
                    .iter()
                    .any(|&(message, _)| (first..=last).contains(&message))
            })
        }

        fn take_character(&self, system: bool) -> u16 {
            let (first, last) = character_messages(system);
            let mut queue = self.queue.borrow_mut();
            let Some(index) = queue
                .iter()
                .position(|&(message, _)| (first..=last).contains(&message))
            else {
                return 0;
            };
            match queue.remove(index) {
                Some((message, unit)) if message == first => unit,
                _ => 0,
            }
        }

        fn character(&self, _key: u16, _scan: u16, _keys: &KeyState) -> u16 {
            self.character
        }

        fn key_at(&self, _scan: u16, _extended: bool) -> u16 {
            self.key_at
        }
    }

    /// A keyboard, with the events that it sends.
    #[derive(Default)]
    struct Test {
        keyboard: Keyboard,
        sent: Vec<KeyEvent>,
    }

    impl Test {
        fn handle(
            &mut self,
            fake: &Fake,
            message: u32,
            wparam: u16,
            lparam: LPARAM,
        ) -> Destination {
            let sent = &mut self.sent;
            self.keyboard
                .handle(fake, message, WPARAM::from(wparam), lparam, &mut |event| {
                    sent.push(event);
                })
        }

        /// The message loop gives the window the next character message.
        fn next_character(&mut self, fake: &Fake, lparam: LPARAM) {
            let (message, unit) = fake.next().unwrap();
            self.handle(fake, message, unit, lparam);
        }

        fn take(&mut self) -> Vec<KeyEvent> {
            std::mem::take(&mut self.sent)
        }
    }

    /// The `lParam` of a key message, or of the character of a key that
    /// goes down, for a key at `scan`.
    fn down(scan: u16) -> LPARAM {
        1 | ((scan as LPARAM) << 16)
    }

    /// The `lParam` of a key that goes up: the key was down, and goes up.
    fn up(scan: u16) -> LPARAM {
        down(scan) | (1 << 30) | (1 << 31)
    }

    const EXTENDED: LPARAM = 1 << 24;

    fn event(key: u16, scan: u16, character: u16, down: bool) -> KeyEvent {
        KeyEvent {
            key,
            scan,
            character,
            down,
            controls: ControlKeys::empty(),
            repeat: 1,
        }
    }

    #[test]
    fn a_key_that_writes_a_character_waits_for_it() {
        let mut test = Test::default();
        let fake = Fake {
            character: 0x61,
            ..Fake::default()
        };
        fake.post(WM_CHAR, 0x61);
        test.handle(&fake, WM_KEYDOWN, 0x41, down(30));
        assert_eq!(test.take(), []);
        test.next_character(&fake, down(30));
        assert_eq!(test.take(), [event(0x41, 30, 0x61, true)]);
        // Going up, it says what it writes, without a character message.
        test.handle(&fake, WM_KEYUP, 0x41, up(30));
        assert_eq!(test.take(), [event(0x41, 30, 0x61, false)]);
    }

    #[test]
    fn keys_without_a_character_go_at_once() {
        let mut test = Test::default();
        test.handle(&Fake::default(), WM_KEYDOWN, VK_LEFT, down(75) | EXTENDED);
        // When the keyboard repeats it faster than the window takes it.
        let repeated = (down(75) & !0xffff) | 3;
        test.handle(&Fake::default(), WM_KEYDOWN, VK_LEFT, repeated);
        let left = KeyEvent {
            controls: ControlKeys::ENHANCED,
            ..event(VK_LEFT, 75, 0, true)
        };
        assert_eq!(
            test.take(),
            [
                left,
                KeyEvent {
                    repeat: 3,
                    controls: ControlKeys::empty(),
                    ..left
                }
            ]
        );
    }

    #[test]
    fn posted_key_messages_keep_the_order_of_their_keys() {
        // A program posts the messages of two keys at once, so the
        // characters come after all of them.
        let mut test = Test::default();
        let fake = Fake::default();
        fake.post(WM_CHAR, 0x61);
        test.handle(&fake, WM_KEYDOWN, 0x41, down(30));
        test.handle(&fake, WM_KEYUP, 0x41, up(30));
        fake.post(WM_CHAR, 0x62);
        test.handle(&fake, WM_KEYDOWN, 0x42, down(48));
        test.handle(&fake, WM_KEYUP, 0x42, up(48));
        assert_eq!(fake.next(), None);
        assert_eq!(
            test.take(),
            [
                event(0x41, 30, 0x61, true),
                event(0x41, 30, 0, false),
                event(0x42, 48, 0x62, true),
                event(0x42, 48, 0, false),
            ]
        );
    }

    #[test]
    fn a_dead_key_goes_down_without_its_character() {
        let mut test = Test::default();
        let fake = Fake::default();
        fake.post(WM_DEADCHAR, 0xb4);
        test.handle(&fake, WM_KEYDOWN, VK_OEM_7, down(40));
        test.next_character(&fake, down(40));
        assert_eq!(test.take(), [event(VK_OEM_7, 40, 0, true)]);
        // A dead character always comes after its key.
        test.handle(&fake, WM_DEADCHAR, 0xb4, down(40));
        assert_eq!(test.take(), []);
    }

    #[test]
    fn a_character_without_its_key_takes_the_key_of_its_scan_code() {
        let mut test = Test::default();
        // ´ and x write ´x: the key of x waits for the first character, and
        // the second one comes alone.
        let x = Fake {
            key_at: 0x58,
            ..Fake::default()
        };
        x.post(WM_CHAR, 0xb4);
        x.post(WM_CHAR, 0x78);
        test.handle(&x, WM_KEYDOWN, 0x58, down(45));
        test.next_character(&x, down(45));
        test.next_character(&x, down(45));
        // A character of an IME has no scan code, and no key.
        test.handle(&Fake::default(), WM_CHAR, 0x4e2d, 1);
        assert_eq!(
            test.take(),
            [
                event(0x58, 45, 0xb4, true),
                event(0x58, 45, 0x78, true),
                event(0, 0, 0x4e2d, true),
            ]
        );
    }

    #[test]
    fn the_character_of_alt_and_the_keypad_goes_when_alt_goes_up() {
        // Alt+0225 writes á when Alt goes up, as .NET expects.
        let mut test = Test::default();
        let fake = Fake::default();
        fake.post(WM_CHAR, 0xe1);
        test.handle(&fake, WM_KEYUP, VK_MENU, up(56));
        test.next_character(&fake, up(56));
        assert_eq!(test.take(), [event(VK_MENU, 56, 0xe1, false)]);
    }

    #[test]
    fn the_control_keys_come_from_the_state_of_the_keys() {
        let mut test = Test::default();
        // AltGr+2 with the Spanish layout, with Caps Lock and Num Lock on.
        // Windows sets Ctrl and Alt, and their left or right key.
        let alt_gr = Fake {
            down: vec![VK_CONTROL, VK_LCONTROL, VK_MENU, VK_RMENU],
            toggled: vec![VK_CAPITAL, VK_NUMLOCK],
            ..Fake::default()
        };
        test.handle(&alt_gr, WM_KEYDOWN, 0x32, down(3));
        let others = Fake {
            down: vec![VK_LMENU, VK_RCONTROL, VK_SHIFT],
            toggled: vec![VK_SCROLL],
            ..Fake::default()
        };
        test.handle(&others, WM_KEYDOWN, VK_LEFT, down(75) | EXTENDED);
        let controls: Vec<_> = test.take().iter().map(|event| event.controls).collect();
        assert_eq!(
            controls,
            [
                ControlKeys::LEFT_CTRL
                    | ControlKeys::RIGHT_ALT
                    | ControlKeys::CAPS_LOCK
                    | ControlKeys::NUM_LOCK,
                ControlKeys::LEFT_ALT
                    | ControlKeys::RIGHT_CTRL
                    | ControlKeys::SHIFT
                    | ControlKeys::SCROLL_LOCK
                    | ControlKeys::ENHANCED,
            ]
        );
    }

    #[test]
    fn alt_f4_and_alt_space_go_to_windows() {
        let mut test = Test::default();
        let alt = Fake {
            down: vec![VK_MENU, VK_LMENU],
            ..Fake::default()
        };
        let to_windows = Destination::Windows;
        assert_eq!(
            test.handle(&alt, WM_SYSKEYDOWN, VK_F4, down(62)),
            to_windows
        );
        assert_eq!(
            test.handle(&alt, WM_SYSKEYDOWN, VK_SPACE, down(57)),
            to_windows
        );
        assert_eq!(
            test.handle(&alt, WM_SYSCHAR, VK_SPACE, down(57)),
            to_windows
        );
        assert_eq!(test.take(), []);
        // Alt alone, F10, and Ctrl+Alt+F4 go to the program.
        let ctrl_alt = Fake {
            down: vec![VK_CONTROL, VK_LCONTROL, VK_MENU, VK_LMENU],
            ..Fake::default()
        };
        for (fake, key, scan) in [
            (&alt, VK_MENU, 56),
            (&Fake::default(), VK_F10, 68),
            (&ctrl_alt, VK_F4, 62),
        ] {
            assert_eq!(
                test.handle(fake, WM_SYSKEYDOWN, key, down(scan)),
                Destination::Program
            );
        }
        let keys: Vec<_> = test.take().iter().map(|event| event.key).collect();
        assert_eq!(keys, [VK_MENU, VK_F10, VK_F4]);
    }

    #[test]
    fn the_windows_keys_and_the_keys_of_an_ime_do_not_go_to_the_program() {
        let mut test = Test::default();
        for key in [VK_LWIN, VK_RWIN, VK_PROCESSKEY] {
            test.handle(&Fake::default(), WM_KEYDOWN, key, down(0));
        }
        assert_eq!(test.take(), []);
    }
}
