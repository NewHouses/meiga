//! The tests of the text field, with its state written as text: `|` is the
//! caret, and `[` and `]` the ends of the selection, as in `a[bc|]d`.

use super::*;
use crate::input::{ControlKeys, KeyEvent};

const VK_BACK: u16 = 0x08;
const VK_TAB: u16 = 0x09;
const VK_RETURN: u16 = 0x0d;
const VK_SHIFT: u16 = 0x10;
const VK_MENU: u16 = 0x12;
const VK_ESCAPE: u16 = 0x1b;
const VK_PRIOR: u16 = 0x21;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_INSERT: u16 = 0x2d;
const VK_DELETE: u16 = 0x2e;
const VK_A: u16 = 0x41;
const VK_B: u16 = 0x42;
const VK_C: u16 = 0x43;
const VK_D: u16 = 0x44;
const VK_E: u16 = 0x45;
const VK_V: u16 = 0x56;
const VK_X: u16 = 0x58;
const VK_Y: u16 = 0x59;
const VK_Z: u16 = 0x5a;
const VK_NUMPAD0: u16 = 0x60;
const VK_NUMPAD1: u16 = 0x61;
const VK_NUMPAD2: u16 = 0x62;
const VK_NUMPAD4: u16 = 0x64;
const VK_F2: u16 = 0x71;
/// The key of the characters that other programs type as keys, such as
/// the emoji panel.
const VK_PACKET: u16 = 0xe7;

fn none() -> ControlKeys {
    ControlKeys::empty()
}

fn ctrl() -> ControlKeys {
    ControlKeys::LEFT_CTRL
}

fn shift() -> ControlKeys {
    ControlKeys::SHIFT
}

fn alt() -> ControlKeys {
    ControlKeys::LEFT_ALT
}

fn ctrl_shift() -> ControlKeys {
    ControlKeys::LEFT_CTRL | ControlKeys::SHIFT
}

fn alt_shift() -> ControlKeys {
    ControlKeys::LEFT_ALT | ControlKeys::SHIFT
}

/// AltGr, as Windows gives it: the left Ctrl and the right Alt.
fn alt_gr() -> ControlKeys {
    ControlKeys::LEFT_CTRL | ControlKeys::RIGHT_ALT
}

/// A key that goes down.
fn event(key: u16, character: u16, controls: ControlKeys) -> KeyEvent {
    KeyEvent {
        key,
        scan: 0,
        character,
        down: true,
        controls,
        repeat: 1,
    }
}

/// The UTF-16 code unit of `c`, which is in the Basic Multilingual Plane.
fn unit(c: char) -> u16 {
    let mut buffer = [0; 2];
    c.encode_utf16(&mut buffer)[0]
}

/// The two halves of `c`, which is outside the Basic Multilingual Plane.
fn halves(c: char) -> [u16; 2] {
    let mut buffer = [0; 2];
    c.encode_utf16(&mut buffer);
    buffer
}

/// A field that holds at most `limit` UTF-16 code units, with the text,
/// the caret and the selection of `state`.
fn make_with(state: &str, limit: usize) -> Field {
    let mut text = String::new();
    let (mut caret, mut start, mut end) = (None, None, None);
    for c in state.chars() {
        match c {
            '|' => caret = Some(text.len()),
            '[' => start = Some(text.len()),
            ']' => end = Some(text.len()),
            c => text.push(c),
        }
    }
    let mut field = Field::new(limit);
    field.set_text(&text);
    assert_eq!(field.text(), text, "{state:?} does not fit");
    let caret = caret.unwrap_or(text.len());
    field.anchor = match (start, end) {
        (Some(start), Some(end)) if caret == start => end,
        (Some(start), Some(_)) => start,
        _ => caret,
    };
    field.caret = caret;
    field.take_changes();
    field
}

fn make(state: &str) -> Field {
    make_with(state, 1000)
}

/// The text, the caret and the selection of `field`, as `field` reads them.
fn state(field: &Field) -> String {
    let text = field.text();
    let selection = field.selection();
    let mut out = String::new();
    let chars = text.char_indices().map(|(at, c)| (at, Some(c)));
    for (at, c) in chars.chain([(text.len(), None)]) {
        if !selection.is_empty() && at == selection.start {
            out.push('[');
        }
        if at == field.caret() {
            out.push('|');
        }
        if !selection.is_empty() && at == selection.end {
            out.push(']');
        }
        out.extend(c);
    }
    out
}

/// What `field` shows, written in the same way: its cells, a blank one as
/// a space and a wide one as its character, with `|` before the cell of
/// the caret, and the selected cells between `[` and `]`.
fn shown(field: &Field) -> String {
    let mut cells = Vec::new();
    let Shown { caret, selection } = field.show(&mut cells);
    let mut out = String::new();
    for index in 0..=cells.len() {
        if !selection.is_empty() && index == selection.start {
            out.push('[');
        }
        if index == caret {
            out.push('|');
        }
        if !selection.is_empty() && index == selection.end {
            out.push(']');
        }
        match cells.get(index) {
            Some(CellChar::Blank) => out.push(' '),
            Some(CellChar::Narrow(c) | CellChar::WideLeft(c)) => out.push(*c),
            Some(CellChar::WideRight(_)) | None => {}
        }
    }
    out
}

/// Presses `key` with `controls`, as a key that writes no character.
fn tap(field: &mut Field, key: u16, controls: ControlKeys) -> Reply {
    field.key(&event(key, 0, controls))
}

/// Types `text` as an IME does: each UTF-16 code unit, in a key without a
/// virtual key.
fn type_text(field: &mut Field, text: &str) {
    for unit in text.encode_utf16() {
        field.key(&event(0, unit, none()));
    }
}

/// The state of the field of `start` after `keys`.
fn after(start: &str, keys: &[(u16, ControlKeys)]) -> String {
    let mut field = make(start);
    for &(key, controls) in keys {
        tap(&mut field, key, controls);
    }
    state(&field)
}

#[test]
fn a_new_field_is_empty_and_shows_nothing_until_it_has_cells() {
    let mut field = Field::new(10);
    assert_eq!(field.text(), "");
    assert_eq!(field.limit(), 10);
    assert_eq!((field.caret(), field.selection()), (0, 0..0));
    assert_eq!(field.selected_text(), "");
    assert_eq!(shown(&field), "|");
    assert_eq!(field.take_changes(), Changes::default());
}

#[test]
fn typing_writes_at_the_caret_and_replaces_the_selection() {
    let mut field = make("ab|c");
    assert_eq!(
        field.key(&event(VK_X, u16::from(b'x'), none())),
        Reply::Taken
    );
    assert_eq!(state(&field), "abx|c");
    for start in ["a[bc|]d", "a[|bc]d"] {
        let mut field = make(start);
        type_text(&mut field, "xy");
        assert_eq!(state(&field), "axy|d", "{start}");
    }
}

#[test]
fn the_letters_of_the_commands_write_themselves_without_ctrl() {
    // With Ctrl, A, C, V, X, Y and Z are commands; alone or with Shift,
    // they are text.
    let mut field = make("|");
    for (key, letter) in [
        (VK_A, b'a'),
        (VK_C, b'c'),
        (VK_V, b'v'),
        (VK_X, b'x'),
        (VK_Y, b'y'),
        (VK_Z, b'z'),
    ] {
        let upper = letter.to_ascii_uppercase();
        assert_eq!(
            field.key(&event(key, u16::from(letter), none())),
            Reply::Taken
        );
        assert_eq!(
            field.key(&event(key, u16::from(upper), shift())),
            Reply::Taken
        );
    }
    assert_eq!(state(&field), "aAcCvVxXyYzZ|");
}

#[test]
fn altgr_writes_characters_and_alt_alone_does_not() {
    let mut field = make("|");
    // AltGr+E writes € in the Spanish and Galician layouts; AltGr+A, which
    // would select all with Ctrl alone, writes æ in some others.
    assert_eq!(field.key(&event(VK_E, unit('€'), alt_gr())), Reply::Taken);
    assert_eq!(field.key(&event(VK_A, unit('æ'), alt_gr())), Reply::Taken);
    assert_eq!(state(&field), "€æ|");
    // Alt+A is for the menus of the window.
    assert_eq!(
        field.key(&event(VK_A, u16::from(b'a'), alt())),
        Reply::Ignored
    );
    // And AltGr without a character does nothing, not even a command.
    assert_eq!(tap(&mut field, VK_A, alt_gr()), Reply::Ignored);
    assert_eq!(tap(&mut field, VK_LEFT, alt_gr()), Reply::Ignored);
    assert_eq!(state(&field), "€æ|");
}

#[test]
fn alt_writes_the_character_of_its_code_when_it_goes_up() {
    let mut field = make("|");
    // Alt+0241, on the numeric keypad: the keys of the code write nothing,
    // nor do they move the caret with Num Lock off, as Insert, ↓, ← and
    // End.
    for key in [
        VK_NUMPAD0, VK_NUMPAD2, VK_NUMPAD4, VK_NUMPAD1, VK_INSERT, VK_DOWN, VK_LEFT, VK_END,
    ] {
        assert_eq!(tap(&mut field, key, alt()), Reply::Ignored, "{key:#x}");
    }
    let up = |key, character| KeyEvent {
        down: false,
        ..event(key, character, none())
    };
    assert_eq!(field.key(&up(VK_MENU, unit('ñ'))), Reply::Taken);
    assert_eq!(state(&field), "ñ|");
    // Another key that goes up says what it writes, but writes nothing;
    // nor does Alt going up without a code.
    assert_eq!(field.key(&up(VK_A, u16::from(b'a'))), Reply::Ignored);
    assert_eq!(field.key(&up(VK_MENU, 0)), Reply::Ignored);
    assert_eq!(state(&field), "ñ|");
}

#[test]
fn control_characters_never_go_in() {
    let mut field = make("ab|");
    for (key, character, controls) in [
        (VK_RETURN, 0x0d, none()),
        (VK_RETURN, 0x0a, ctrl()),
        (VK_ESCAPE, 0x1b, none()),
        (VK_TAB, 0x09, none()),
        (VK_B, 0x02, ctrl()),
        (0, 0x7f, none()),
        (0, 0x85, none()),
        (0, 0x9b, none()),
    ] {
        let reply = field.key(&event(key, character, controls));
        assert_eq!(reply, Reply::Ignored, "{key:#x} {character:#x}");
    }
    assert_eq!(state(&field), "ab|");
    assert_eq!(field.take_changes(), Changes::default());
}

#[test]
fn characters_outside_the_basic_multilingual_plane_come_in_two_halves() {
    let mut field = make("|");
    let [high, low] = halves('😀');
    assert_eq!(field.key(&event(0, high, none())), Reply::Taken);
    assert_eq!(state(&field), "|");
    assert_eq!(field.key(&event(0, low, none())), Reply::Taken);
    assert_eq!(state(&field), "😀|");
    // A second half alone goes away; and so does a first half followed by
    // another character, or by a command.
    assert_eq!(field.key(&event(0, low, none())), Reply::Taken);
    field.key(&event(0, high, none()));
    type_text(&mut field, "a");
    field.key(&event(0, high, none()));
    tap(&mut field, VK_LEFT, none());
    field.key(&event(0, low, none()));
    assert_eq!(state(&field), "😀|a");
}

#[test]
fn characters_without_a_key_go_in_as_the_ones_of_an_ime_or_the_emoji_panel() {
    let mut field = make("|");
    type_text(&mut field, "中文");
    let [high, low] = halves('👍');
    field.key(&event(VK_PACKET, high, none()));
    field.key(&event(VK_PACKET, low, none()));
    assert_eq!(state(&field), "中文👍|");
}

#[test]
fn a_dead_key_that_does_not_join_writes_its_accent_before_the_key() {
    // ´ and Backspace: Windows writes the accent with the key, and then the
    // key with its own character, which deletes it.
    let mut field = make("ab|");
    assert_eq!(field.key(&event(VK_BACK, unit('´'), none())), Reply::Taken);
    assert_eq!(state(&field), "ab´|");
    assert_eq!(field.key(&event(VK_BACK, 0x08, none())), Reply::Taken);
    assert_eq!(state(&field), "ab|");
    // ´ and Enter, Esc or Tab: the accent goes in, and the key is for the
    // user of the field.
    for (key, own) in [(VK_RETURN, 0x0d), (VK_ESCAPE, 0x1b), (VK_TAB, 0x09)] {
        let mut field = make("ab|");
        assert_eq!(field.key(&event(key, unit('´'), none())), Reply::Taken);
        assert_eq!(field.key(&event(key, own, none())), Reply::Ignored);
        assert_eq!(state(&field), "ab´|", "{key:#x}");
    }
    // A character of the numeric keypad that comes alone, whose key comes
    // from its scan code: Home for the 7.
    let mut field = make("ab|");
    assert_eq!(
        field.key(&event(VK_HOME, u16::from(b'7'), none())),
        Reply::Taken
    );
    assert_eq!(state(&field), "ab7|");
}

#[test]
fn a_key_that_repeats_does_its_work_once_per_repeat() {
    let repeated = |key, character, controls, repeat| KeyEvent {
        repeat,
        ..event(key, character, controls)
    };
    let mut field = make("|");
    field.key(&repeated(VK_A, u16::from(b'a'), none(), 3));
    assert_eq!(state(&field), "aaa|");
    field.key(&repeated(VK_LEFT, 0, none(), 2));
    assert_eq!(state(&field), "a|aa");
    field.key(&repeated(VK_BACK, 0x08, none(), 5));
    assert_eq!(state(&field), "|aa");
    field.key(&repeated(VK_Z, 0, ctrl(), 2));
    assert_eq!(state(&field), "|");
    field.key(&repeated(VK_Y, 0, ctrl(), 2));
    assert_eq!(state(&field), "|aa");
    // A count of 0, which Windows never gives, is once.
    field.key(&repeated(VK_DELETE, 0, none(), 0));
    assert_eq!(state(&field), "|a");
}

#[test]
fn an_accent_typed_after_a_letter_joins_it() {
    let mut field = make("e|x");
    type_text(&mut field, "\u{301}");
    assert_eq!(state(&field), "e\u{301}|x");
    tap(&mut field, VK_LEFT, none());
    assert_eq!(state(&field), "|e\u{301}x");
}

#[test]
fn what_joins_the_text_after_it_leaves_the_caret_after_the_whole_cluster() {
    // A regional indicator before a flag: it makes a flag with the first
    // one of the flag.
    let mut field = make("|🇪🇸");
    type_text(&mut field, "🇫");
    assert_eq!(state(&field), "🇫🇪|🇸");
}

#[test]
fn a_character_that_does_not_fit_is_not_typed() {
    let mut field = make_with("abc|", 3);
    assert_eq!(
        field.key(&event(VK_D, u16::from(b'd'), none())),
        Reply::Taken
    );
    assert_eq!(state(&field), "abc|");
    assert_eq!(field.take_changes(), Changes::default());
    // In place of a selection, it fits.
    let mut field = make_with("a[b|]c", 3);
    type_text(&mut field, "d");
    assert_eq!(state(&field), "ad|c");
    // An emoji takes two UTF-16 code units, and ñ one.
    let mut field = make_with("ab|", 3);
    type_text(&mut field, "😀");
    assert_eq!(state(&field), "ab|");
    type_text(&mut field, "ñ");
    assert_eq!(state(&field), "abñ|");
}

#[test]
fn set_text_puts_a_single_line_cut_to_the_limit() {
    let mut field = Field::new(6);
    field.set_text("a\tb\r\nc\u{7}");
    assert_eq!(state(&field), "a b c|");
    assert_eq!(
        field.take_changes(),
        Changes {
            shown: true,
            text: true
        }
    );
    // The spaces at its ends stay.
    field.set_text(" x ");
    assert_eq!(state(&field), " x |");
    // Cut in whole clusters: e and its accent are two code units, and the
    // emoji two more.
    for text in ["abe\u{301}", "ab👍"] {
        let mut field = Field::new(3);
        field.set_text(text);
        assert_eq!(state(&field), "ab|", "{text:?}");
    }
    // The same text again, with the caret already at its end: no change.
    let mut field = make("abc|");
    field.set_text("abc");
    assert_eq!(field.take_changes(), Changes::default());
}

#[test]
fn set_text_forgets_the_steps_to_undo() {
    let mut field = make("|");
    type_text(&mut field, "abc");
    field.set_text("xyz");
    assert_eq!(tap(&mut field, VK_Z, ctrl()), Reply::Taken);
    assert_eq!(state(&field), "xyz|");
}

#[test]
fn the_arrows_move_the_caret_one_cluster() {
    // A letter, a family, an e with an accent and an ideograph.
    let text = "a👨\u{200d}👩\u{200d}👧e\u{301}中";
    let mut field = make(&format!("|{text}"));
    let mut stops = Vec::new();
    for _ in 0..5 {
        assert_eq!(tap(&mut field, VK_RIGHT, none()), Reply::Taken);
        stops.push(field.caret());
    }
    assert_eq!(stops, [1, 19, 22, 25, 25]);
    stops.clear();
    for _ in 0..5 {
        tap(&mut field, VK_LEFT, none());
        stops.push(field.caret());
    }
    assert_eq!(stops, [22, 19, 1, 0, 0]);
}

#[test]
fn ctrl_and_the_arrows_move_by_words_and_home_and_end_to_the_ends() {
    assert_eq!(after("|foo bar.rs", &[(VK_RIGHT, ctrl())]), "foo |bar.rs");
    assert_eq!(
        after("|foo bar.rs", &[(VK_RIGHT, ctrl()); 2]),
        "foo bar|.rs"
    );
    assert_eq!(after("foo bar.rs|", &[(VK_LEFT, ctrl())]), "foo bar.|rs");
    assert_eq!(after("foo bar.rs|", &[(VK_LEFT, ctrl()); 3]), "foo |bar.rs");
    for controls in [none(), ctrl()] {
        assert_eq!(after("ab|c", &[(VK_HOME, controls)]), "|abc");
        assert_eq!(after("ab|c", &[(VK_END, controls)]), "abc|");
    }
}

#[test]
fn shift_selects_from_where_the_selection_started() {
    let right = (VK_RIGHT, shift());
    let left = (VK_LEFT, shift());
    assert_eq!(after("ab|cd", &[right, right]), "ab[cd|]");
    assert_eq!(after("ab|cd", &[left]), "a[|b]cd");
    assert_eq!(after("ab|cd", &[right, left, left]), "a[|b]cd");
    // Back to where it started, there is no selection.
    assert_eq!(after("ab|cd", &[right, left]), "ab|cd");
    assert_eq!(after("ab|cd", &[(VK_HOME, shift())]), "[|ab]cd");
    assert_eq!(after("ab|cd", &[(VK_END, shift())]), "ab[cd|]");
    assert_eq!(after("ab|cd", &[(VK_HOME, ctrl_shift())]), "[|ab]cd");
    assert_eq!(after("ab|cd", &[(VK_UP, shift())]), "[|ab]cd");
    assert_eq!(after("ab|cd", &[(VK_DOWN, shift())]), "ab[cd|]");
    assert_eq!(after("|foo bar", &[(VK_RIGHT, ctrl_shift())]), "[foo |]bar");
    assert_eq!(
        after("foo bar|", &[(VK_LEFT, ctrl_shift()); 2]),
        "[|foo bar]"
    );
}

#[test]
fn without_shift_the_arrows_end_a_selection_at_its_edge() {
    for start in ["a[bc|]d", "a[|bc]d"] {
        assert_eq!(after(start, &[(VK_LEFT, none())]), "a|bcd", "{start}");
        assert_eq!(after(start, &[(VK_RIGHT, none())]), "abc|d", "{start}");
        assert_eq!(after(start, &[(VK_HOME, none())]), "|abcd", "{start}");
        assert_eq!(after(start, &[(VK_END, none())]), "abcd|", "{start}");
    }
    // With Ctrl, from that edge to the start of a word.
    assert_eq!(after("[fo|]o bar", &[(VK_RIGHT, ctrl())]), "foo |bar");
    assert_eq!(after("foo b[|ar]", &[(VK_LEFT, ctrl())]), "foo |bar");
}

#[test]
fn the_keys_of_the_user_of_the_field_change_nothing() {
    let mut field = make("ab|c");
    for (key, controls) in [
        (VK_UP, none()),
        (VK_DOWN, none()),
        (VK_PRIOR, none()),
        (VK_F2, none()),
        (VK_SHIFT, shift()),
        (VK_LEFT, alt()),
        (VK_RIGHT, alt()),
        (VK_HOME, alt()),
        (VK_UP, alt_shift()),
        (VK_INSERT, none()),
        (VK_INSERT, ctrl_shift()),
        (VK_DELETE, alt()),
        (VK_BACK, ctrl() | ControlKeys::LEFT_ALT),
        (VK_Z, alt()),
    ] {
        assert_eq!(tap(&mut field, key, controls), Reply::Ignored, "{key:#x}");
    }
    assert_eq!(state(&field), "ab|c");
    assert_eq!(field.take_changes(), Changes::default());
}

#[test]
fn backspace_deletes_the_last_code_point_and_delete_the_next_cluster() {
    assert_eq!(after("e\u{301}|x", &[(VK_BACK, none())]), "e|x");
    assert_eq!(after("ab|c", &[(VK_BACK, none())]), "a|c");
    assert_eq!(after("|ab", &[(VK_BACK, none())]), "|ab");
    // An emoji is a single code point, in two halves in UTF-16.
    assert_eq!(after("a😀|", &[(VK_BACK, none())]), "a|");
    // A family loses its last member, and then the joiner before it.
    let family = "👨\u{200d}👩";
    assert_eq!(
        after(&format!("{family}|"), &[(VK_BACK, none())]),
        "👨\u{200d}|"
    );
    assert_eq!(after("x|e\u{301}y", &[(VK_DELETE, none())]), "x|y");
    assert_eq!(after(&format!("|{family}x"), &[(VK_DELETE, none())]), "|x");
    assert_eq!(after("ab|", &[(VK_DELETE, none())]), "ab|");
}

#[test]
fn after_deleting_the_caret_goes_to_the_start_of_the_cluster_that_it_is_in() {
    // Without 🇸, the 🇪 of Spain and the 🇫 of France make a flag.
    assert_eq!(after("🇪🇸|🇫🇷", &[(VK_BACK, none())]), "|🇪🇫🇷");
    // Without the x, the two regional indicators make a flag.
    assert_eq!(after("🇪|x🇸", &[(VK_DELETE, none())]), "|🇪🇸");
}

#[test]
fn backspace_and_delete_delete_the_selection_also_with_ctrl() {
    for start in ["a[bc|]d", "a[|bc]d"] {
        for key in [VK_BACK, VK_DELETE] {
            for controls in [none(), ctrl()] {
                assert_eq!(after(start, &[(key, controls)]), "a|d", "{start} {key:#x}");
            }
        }
    }
}

#[test]
fn ctrl_backspace_and_ctrl_delete_delete_up_to_the_start_of_a_word() {
    assert_eq!(after("foo bar|", &[(VK_BACK, ctrl())]), "foo |");
    assert_eq!(after("foo bar|", &[(VK_BACK, ctrl()); 2]), "|");
    assert_eq!(after("|foo bar", &[(VK_DELETE, ctrl())]), "|bar");
    assert_eq!(after("main.r|s", &[(VK_BACK, ctrl())]), "main.|s");
    assert_eq!(
        after("C:\\Users\\x|", &[(VK_BACK, ctrl()); 2]),
        "C:\\Users|"
    );
    assert_eq!(after("|", &[(VK_BACK, ctrl()), (VK_DELETE, ctrl())]), "|");
    // With Shift too.
    assert_eq!(after("foo bar|", &[(VK_BACK, ctrl_shift())]), "foo |");
    assert_eq!(after("|foo bar", &[(VK_DELETE, ctrl_shift())]), "|bar");
}

#[test]
fn ctrl_c_and_ctrl_insert_copy_the_selection() {
    for (key, controls) in [(VK_C, ctrl()), (VK_C, ctrl_shift()), (VK_INSERT, ctrl())] {
        let mut field = make("a[bc|]d");
        assert_eq!(tap(&mut field, key, controls), Reply::Copy("bc".to_owned()));
        assert_eq!(state(&field), "a[bc|]d");
        assert_eq!(field.take_changes(), Changes::default());
        // Without a selection, nothing: and the key reaches nobody else.
        let mut field = make("ab|c");
        assert_eq!(tap(&mut field, key, controls), Reply::Taken);
    }
}

#[test]
fn ctrl_x_and_shift_delete_cut_the_selection() {
    for (key, controls) in [(VK_X, ctrl()), (VK_X, ctrl_shift()), (VK_DELETE, shift())] {
        let mut field = make("a[|bc]d");
        assert_eq!(tap(&mut field, key, controls), Reply::Copy("bc".to_owned()));
        assert_eq!(state(&field), "a|d");
        // Undoing brings it back, selected.
        tap(&mut field, VK_Z, ctrl());
        assert_eq!(state(&field), "a[|bc]d");
        let mut field = make("ab|c");
        assert_eq!(tap(&mut field, key, controls), Reply::Taken);
        assert_eq!(state(&field), "ab|c");
    }
}

#[test]
fn ctrl_v_and_shift_insert_ask_for_the_clipboard() {
    for (key, controls) in [(VK_V, ctrl()), (VK_V, ctrl_shift()), (VK_INSERT, shift())] {
        let mut field = make("ab|c");
        assert_eq!(tap(&mut field, key, controls), Reply::Paste);
        assert_eq!(state(&field), "ab|c");
    }
}

#[test]
fn the_commands_work_with_the_characters_that_windows_gives_their_keys() {
    let mut field = make("foo bar|");
    // Ctrl+Backspace writes DEL, and Backspace BS.
    field.key(&event(VK_BACK, 0x7f, ctrl()));
    assert_eq!(state(&field), "foo |");
    field.key(&event(VK_BACK, 0x08, none()));
    assert_eq!(state(&field), "foo|");
    // Ctrl and a letter write a C0 control.
    assert_eq!(field.key(&event(VK_A, 0x01, ctrl())), Reply::Taken);
    assert_eq!(
        field.key(&event(VK_C, 0x03, ctrl())),
        Reply::Copy("foo".to_owned())
    );
    assert_eq!(
        field.key(&event(VK_X, 0x18, ctrl())),
        Reply::Copy("foo".to_owned())
    );
    assert_eq!(state(&field), "|");
    assert_eq!(field.key(&event(VK_V, 0x16, ctrl())), Reply::Paste);
    assert_eq!(field.key(&event(VK_Z, 0x1a, ctrl())), Reply::Taken);
    assert_eq!(state(&field), "[foo|]");
    assert_eq!(field.key(&event(VK_Y, 0x19, ctrl())), Reply::Taken);
    assert_eq!(state(&field), "|");
}

#[test]
fn undoing_goes_back_a_step_and_redoing_forward() {
    let mut field = make("|");
    type_text(&mut field, "foo");
    tap(&mut field, VK_LEFT, none());
    type_text(&mut field, "x");
    assert_eq!(state(&field), "fox|o");
    // Ctrl+Z, and Alt+Backspace, as in Windows.
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "fo|o");
    tap(&mut field, VK_BACK, alt());
    assert_eq!(state(&field), "|");
    assert_eq!(tap(&mut field, VK_Z, ctrl()), Reply::Taken);
    assert_eq!(state(&field), "|");
    // Ctrl+Y, Ctrl+Shift+Z and Alt+Shift+Backspace.
    tap(&mut field, VK_Y, ctrl());
    assert_eq!(state(&field), "foo|");
    tap(&mut field, VK_Z, ctrl_shift());
    assert_eq!(state(&field), "fox|o");
    assert_eq!(tap(&mut field, VK_BACK, alt_shift()), Reply::Taken);
    assert_eq!(state(&field), "fox|o");
    tap(&mut field, VK_Z, ctrl());
    tap(&mut field, VK_BACK, alt_shift());
    assert_eq!(state(&field), "fox|o");
}

#[test]
fn undoing_typing_over_a_selection_brings_the_selection_back() {
    let mut field = make("a[bc|]d");
    type_text(&mut field, "xy");
    assert_eq!(state(&field), "axy|d");
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "a[bc|]d");
    tap(&mut field, VK_Y, ctrl());
    assert_eq!(state(&field), "axy|d");
}

#[test]
fn backspaces_in_a_row_and_deletes_in_a_row_are_a_step_each() {
    let mut field = make("abc|def");
    for key in [VK_BACK, VK_BACK, VK_DELETE, VK_DELETE] {
        tap(&mut field, key, none());
    }
    assert_eq!(state(&field), "a|f");
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "a|def");
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "abc|def");
    // A word deleted is a step of its own.
    let mut field = make("foo bar baz|");
    tap(&mut field, VK_BACK, ctrl());
    tap(&mut field, VK_BACK, ctrl());
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "foo bar |");
}

#[test]
fn a_paste_is_a_step_of_its_own() {
    let mut field = make("ab|");
    type_text(&mut field, "c");
    field.paste("de");
    type_text(&mut field, "f");
    assert_eq!(state(&field), "abcdef|");
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "abcde|");
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "abc|");
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "ab|");
}

#[test]
fn a_new_edit_forgets_the_steps_undone() {
    let mut field = make("|");
    type_text(&mut field, "a");
    tap(&mut field, VK_Z, ctrl());
    type_text(&mut field, "b");
    tap(&mut field, VK_Y, ctrl());
    assert_eq!(state(&field), "b|");
}

#[test]
fn a_paste_becomes_a_single_line() {
    for (clipboard, pasted) in [
        ("plain", "plain"),
        (
            "  a\r\nb\tc\u{7}d\u{85}e\u{2028}f\u{2029}g\u{b}h\u{c}i\r\n",
            "a b cd e f g h i",
        ),
        ("un\ndous\rtres\r\ncatro", "un dous tres catro"),
        // A line feed and then a carriage return are two line breaks; a
        // carriage return and then a line feed, one.
        ("a\n\rb", "a  b"),
        ("a   b", "a   b"),
        ("\u{a0}x\u{3000}", "x"),
        ("\u{1b}[31mred", "[31mred"),
    ] {
        let mut field = make("<|>");
        field.paste(clipboard);
        assert_eq!(state(&field), format!("<{pasted}|>"), "{clipboard:?}");
    }
}

#[test]
fn a_paste_of_only_whitespace_is_a_space_and_of_nothing_nothing() {
    let mut field = make("a[b|]c");
    field.paste(" \r\n\t ");
    assert_eq!(state(&field), "a |c");
    for nothing in ["", "\u{7}\u{1b}\u{0}"] {
        let mut field = make("a[b|]c");
        field.paste(nothing);
        assert_eq!(state(&field), "a[b|]c", "{nothing:?}");
        assert_eq!(field.take_changes(), Changes::default());
    }
}

#[test]
fn a_paste_puts_what_fits_in_whole_clusters() {
    let mut field = make_with("ab|", 5);
    field.paste("cd👍🏽e");
    assert_eq!(state(&field), "abcd|");
    // In place of a selection, more fits.
    let mut field = make_with("[ab|]", 6);
    field.paste("cd👍🏽e");
    assert_eq!(state(&field), "cd👍🏽|");
    // Nothing fits: nothing changes, not even a space.
    let mut field = make_with("abcd[e|]f", 6);
    field.paste("😀");
    assert_eq!(state(&field), "abcd[e|]f");
    let mut field = make_with("abcdef|", 6);
    field.paste("   ");
    assert_eq!(state(&field), "abcdef|");
    assert_eq!(field.take_changes(), Changes::default());
}

#[test]
fn a_click_puts_the_caret_on_the_nearest_side_of_the_cell() {
    let mut field = make("|a中b");
    for (column, side, caret) in [
        (0, Side::Left, 0),
        (0, Side::Right, 1),
        // The first cell of 中 goes before it, and the second one after it.
        (1, Side::Left, 1),
        (1, Side::Right, 1),
        (2, Side::Left, 4),
        (2, Side::Right, 4),
        (3, Side::Left, 4),
        (3, Side::Right, 5),
        // After the text.
        (4, Side::Left, 5),
        (9, Side::Right, 5),
    ] {
        field.press(column, side, 1, false);
        assert_eq!(field.caret(), caret, "{column} {side:?}");
        assert_eq!(field.selection(), caret..caret, "{column} {side:?}");
    }
}

#[test]
fn a_click_with_shift_moves_the_end_of_the_selection() {
    let mut field = make("ab|cdef");
    field.press(5, Side::Left, 1, true);
    assert_eq!(state(&field), "ab[cde|]f");
    field.press(0, Side::Left, 1, true);
    assert_eq!(state(&field), "[|ab]cdef");
}

#[test]
fn dragging_selects_from_where_the_button_went_down() {
    let mut field = make("|abcdef");
    field.press(1, Side::Left, 1, false);
    field.drag(4, Side::Left);
    assert_eq!(state(&field), "a[bcd|]ef");
    field.drag(0, Side::Left);
    assert_eq!(state(&field), "[|a]bcdef");
    // Without a button down, nothing.
    let mut field = make("ab|c");
    field.drag(0, Side::Left);
    assert_eq!(state(&field), "ab|c");
}

#[test]
fn a_double_click_selects_a_word_and_dragging_whole_words() {
    let mut field = make("|foo  bar.rs baz");
    field.press(1, Side::Left, 2, false);
    assert_eq!(state(&field), "[foo|]  bar.rs baz");
    // On the spaces, the spaces; on a sign, its run.
    field.press(4, Side::Left, 2, false);
    assert_eq!(state(&field), "foo[  |]bar.rs baz");
    field.press(8, Side::Right, 2, false);
    assert_eq!(state(&field), "foo  bar[.|]rs baz");
    // After the end, the last word.
    field.press(30, Side::Left, 2, false);
    assert_eq!(state(&field), "foo  bar.rs [baz|]");
    // Dragging after a double click on "bar": to "rs", and back to "foo";
    // inside the first word, it stays whole.
    field.press(6, Side::Left, 2, false);
    field.drag(10, Side::Left);
    assert_eq!(state(&field), "foo  [bar.rs|] baz");
    field.drag(1, Side::Left);
    assert_eq!(state(&field), "[|foo  bar].rs baz");
    field.drag(6, Side::Right);
    assert_eq!(state(&field), "foo  [bar|].rs baz");
}

#[test]
fn a_triple_click_selects_everything_and_dragging_keeps_it() {
    let mut field = make("foo |bar");
    field.press(1, Side::Left, 3, false);
    assert_eq!(state(&field), "[foo bar|]");
    field.drag(0, Side::Left);
    assert_eq!(state(&field), "[foo bar|]");
}

#[test]
fn a_double_click_without_text_selects_nothing() {
    for start in ["|", "\u{200b}|"] {
        let mut field = make(start);
        field.press(0, Side::Left, 2, false);
        field.drag(3, Side::Left);
        assert_eq!(state(&field), start);
    }
}

#[test]
fn a_change_of_the_text_ends_what_the_button_held_down_selects() {
    // A double click on "bar", and then, with the button still down, a
    // change of the text: the words are not where they were.
    let mut field = make("|foo bar");
    field.press(5, Side::Left, 2, false);
    assert_eq!(state(&field), "foo [bar|]");
    type_text(&mut field, "x");
    field.drag(0, Side::Left);
    assert_eq!(state(&field), "foo x|");
    // And undoing, or redoing, is a change of the text too.
    field.press(1, Side::Left, 2, false);
    tap(&mut field, VK_Z, ctrl());
    field.drag(6, Side::Left);
    assert_eq!(state(&field), "foo [bar|]");
    field.press(1, Side::Left, 2, false);
    tap(&mut field, VK_Y, ctrl());
    field.drag(0, Side::Left);
    assert_eq!(state(&field), "foo x|");
}

#[test]
fn a_click_somewhere_else_ends_the_step_of_the_typing() {
    let mut field = make("|");
    type_text(&mut field, "ab");
    // Where the caret is: nothing changes, and the step goes on.
    field.press(2, Side::Left, 1, false);
    type_text(&mut field, "c");
    field.press(0, Side::Left, 1, false);
    field.press(3, Side::Left, 1, false);
    type_text(&mut field, "d");
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "abc|");
    tap(&mut field, VK_Z, ctrl());
    assert_eq!(state(&field), "|");
}

#[test]
fn a_text_that_does_not_fit_shows_the_part_with_the_caret() {
    let mut field = make("|");
    field.set_columns(5);
    let mut seen = Vec::new();
    for c in "abcdefgh".chars() {
        type_text(&mut field, &c.to_string());
        seen.push(shown(&field));
    }
    assert_eq!(
        seen,
        [
            "a|", "ab|", "abc|", "abcd|", "bcde|", "cdef|", "defg|", "efgh|"
        ]
    );
    tap(&mut field, VK_HOME, none());
    assert_eq!(shown(&field), "|abcde");
    tap(&mut field, VK_END, none());
    assert_eq!(shown(&field), "efgh|");
    for _ in 0..4 {
        tap(&mut field, VK_LEFT, none());
    }
    assert_eq!(shown(&field), "|efgh");
    tap(&mut field, VK_LEFT, none());
    assert_eq!(shown(&field), "|defgh");
}

#[test]
fn a_shorter_text_brings_back_the_part_that_the_field_shows() {
    let mut field = make("abcdefgh|");
    field.set_columns(5);
    assert_eq!(shown(&field), "efgh|");
    tap(&mut field, VK_BACK, none());
    assert_eq!(shown(&field), "defg|");
    tap(&mut field, VK_LEFT, shift());
    tap(&mut field, VK_LEFT, shift());
    tap(&mut field, VK_DELETE, none());
    assert_eq!(shown(&field), "bcde|");
    tap(&mut field, VK_A, ctrl());
    tap(&mut field, VK_DELETE, none());
    assert_eq!(shown(&field), "|");
}

#[test]
fn the_field_never_shows_half_a_wide_character() {
    // On the left: the field shows one cell less.
    let mut field = make("a中中|");
    field.set_columns(4);
    assert_eq!(shown(&field), "中|");
    // On the right: that cell is blank.
    let mut field = make("|ab中");
    field.set_columns(3);
    assert_eq!(shown(&field), "|ab ");
}

#[test]
fn the_selection_shows_in_the_cells_that_the_field_shows() {
    let mut field = make("abcdef|");
    field.set_columns(3);
    tap(&mut field, VK_HOME, shift());
    assert_eq!(shown(&field), "[|abc]");
    tap(&mut field, VK_A, ctrl());
    assert_eq!(shown(&field), "[ef|]");
}

#[test]
fn clusters_without_width_take_no_cell() {
    let mut field = make("\u{200b}|x");
    field.set_columns(4);
    assert_eq!(shown(&field), "|x");
}

#[test]
fn a_field_without_cells_shows_nothing() {
    let mut field = make("abc|");
    field.set_columns(2);
    field.set_columns(0);
    let mut cells = vec![CellChar::Blank];
    assert_eq!(field.show(&mut cells), Shown::default());
    assert!(cells.is_empty());
}

#[test]
fn dragging_out_of_the_field_shows_more_on_that_side() {
    let mut field = make("|abcdefgh");
    field.set_columns(5);
    field.press(0, Side::Left, 1, false);
    field.drag(7, Side::Left);
    assert_eq!(shown(&field), "[defg|]h");
    field.drag(9, Side::Left);
    assert_eq!(shown(&field), "[efgh|]");
    let mut field = make("abcdefgh|");
    field.set_columns(5);
    field.press(2, Side::Left, 1, false);
    field.drag(-2, Side::Left);
    assert_eq!(shown(&field), "[|cdef]g");
    field.drag(-5, Side::Left);
    assert_eq!(shown(&field), "[|abcde]");
}

#[test]
fn the_changes_say_what_changed() {
    let mut field = make("ab|");
    type_text(&mut field, "c");
    assert_eq!(
        field.take_changes(),
        Changes {
            shown: true,
            text: true
        }
    );
    assert_eq!(field.take_changes(), Changes::default());
    tap(&mut field, VK_LEFT, none());
    assert_eq!(
        field.take_changes(),
        Changes {
            shown: true,
            text: false
        }
    );
    // A key that changes nothing.
    tap(&mut field, VK_HOME, none());
    field.take_changes();
    tap(&mut field, VK_LEFT, none());
    assert_eq!(field.take_changes(), Changes::default());
    // Cells that show the same part of the text change nothing either.
    field.set_columns(2);
    assert_eq!(field.take_changes(), Changes::default());
    // Another part of it is a change.
    tap(&mut field, VK_END, none());
    field.take_changes();
    field.set_columns(10);
    assert_eq!(
        field.take_changes(),
        Changes {
            shown: true,
            text: false
        }
    );
}

/// A generator of pseudorandom numbers (xorshift64), so that the test
/// sees the same sequences every time.
struct Random(u64);

impl Random {
    fn below(&mut self, n: usize) -> usize {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x % n as u64) as usize
    }
}

/// What the random edits type: letters, digits and signs, an accent alone,
/// ideographs, emoji, regional indicators and joiners.
const PIECES: [&str; 18] = [
    "a", "Z", "7", "_", " ", ".", "\\", "é", "e\u{301}", "\u{301}", "中", "😀", "👍🏽", "🇪", "🇸",
    "\u{200d}", "\u{200b}", "ñ",
];

const KEYS: [u16; 16] = [
    VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_UP, VK_DOWN, VK_BACK, VK_DELETE, VK_INSERT, VK_A, VK_C,
    VK_V, VK_X, VK_Y, VK_Z, VK_RETURN,
];

/// Checks what a field always keeps: the caret and the anchor where the
/// caret stops, the text within the limit and without control characters,
/// the caret in sight, the cells of the field, and the changes since
/// `before`.
fn check(field: &mut Field, before: &(String, usize, usize, usize), context: &str) {
    let text = field.text().to_owned();
    for at in [field.anchor, field.caret] {
        assert!(
            text.is_char_boundary(at) && segment::is_stop(&text, at),
            "{context}: {at} in {text:?}"
        );
    }
    assert!(units(&text) <= field.limit(), "{context}: {text:?}");
    assert!(!text.chars().any(char::is_control), "{context}: {text:?}");
    let mut cells = Vec::new();
    let shown = field.show(&mut cells);
    assert!(cells.len() <= field.columns, "{context}: {cells:?}");
    assert!(shown.caret <= cells.len(), "{context}: {shown:?} {cells:?}");
    if field.columns > 0 {
        assert!(shown.caret < field.columns, "{context}: {shown:?}");
        assert!(
            !matches!(cells.first(), Some(CellChar::WideRight(_))),
            "{context}: {cells:?}"
        );
    }
    let selection = &shown.selection;
    assert!(
        selection.start <= selection.end && selection.end <= cells.len(),
        "{context}: {shown:?} {cells:?}"
    );
    let changes = field.take_changes();
    if text != before.0 {
        assert!(changes.text, "{context}: {changes:?}");
    }
    if (text, field.anchor, field.caret, field.scroll) != *before {
        assert!(changes.shown, "{context}: {changes:?}");
    }
}

#[test]
fn random_edits_keep_the_field_whole_and_undo_to_the_start() {
    let controls = [none(), shift(), ctrl(), ctrl_shift(), alt(), alt_shift()];
    let sides = [Side::Left, Side::Right];
    for seed in 1..=300_u64 {
        let mut random = Random(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut field = Field::new(4 + random.below(12));
        let mut clipboard = String::new();
        // At most two edits each, which are fewer than the steps that can be
        // undone.
        for step in 0..45 {
            let before = (field.text.clone(), field.anchor, field.caret, field.scroll);
            let action = random.below(10);
            match action {
                0..=3 => type_text(&mut field, PIECES[random.below(PIECES.len())]),
                4..=6 => {
                    let key = KEYS[random.below(KEYS.len())];
                    match tap(&mut field, key, controls[random.below(controls.len())]) {
                        Reply::Copy(text) => clipboard = text,
                        Reply::Paste => field.paste(&clipboard),
                        Reply::Ignored | Reply::Taken => {}
                    }
                }
                7 => {
                    let clicks = 1 + random.below(3) as u8;
                    let side = sides[random.below(2)];
                    field.press(random.below(14), side, clicks, random.below(2) == 1);
                }
                8 => field.drag(random.below(20) as isize - 4, sides[random.below(2)]),
                _ => field.set_columns(random.below(12)),
            }
            check(
                &mut field,
                &before,
                &format!("seed {seed}, step {step}, action {action}"),
            );
        }
        // Redoing what was undone, and then undoing and redoing everything,
        // goes from the last text to the empty one and back.
        while field.redo() {}
        let last = field.text.clone();
        while field.undo() {}
        assert_eq!(field.text(), "", "seed {seed}");
        while field.redo() {}
        assert_eq!(field.text, last, "seed {seed}");
    }
}
