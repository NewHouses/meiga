//! The tests of the editing of the name of a tab: how it ends, its keys,
//! its mouse, its place and its caret.

use std::ops::Range;

use meiga_core::input::ControlKeys;
use meiga_core::model;

use super::*;

const VK_BACK: u16 = 0x08;
const VK_TAB: u16 = 0x09;
const VK_SHIFT: u16 = 0x10;
const VK_MENU: u16 = 0x12;
const VK_HOME: u16 = 0x24;
const VK_UP: u16 = 0x26;
const VK_DOWN: u16 = 0x28;
const VK_A: u16 = 0x41;
const VK_C: u16 = 0x43;
const VK_V: u16 = 0x56;
const VK_X: u16 = 0x58;
const VK_F2: u16 = 0x71;

/// A caret one pixel wide at 100 %, which blinks every 500 ms for 5 s, as
/// the one of Windows by default.
const CARET: CaretSettings = CaretSettings {
    blink: Some(Duration::from_millis(500)),
    timeout: Duration::from_secs(5),
    width: 1,
};

/// The title of a tab at 100 %: 10 cells of 8 × 16 pixels, from 100 to 180.
const PLACE: Place = Place {
    area: Rect::new(100, 0, 180, 32),
    cell: Size::new(8, 16),
};

const MS: Duration = Duration::from_millis(1);

/// The editing of the name of a tab that shows `text`, at [`PLACE`], from
/// `start`, in a window that has the focus if `focused`.
fn editor_with(text: &str, start: Instant, focused: bool) -> Editor {
    let tab = model::Window::new(()).active().id();
    let settings = Settings {
        caret: CARET,
        clicks: ClickSettings::default(),
        dpi: 96,
    };
    Editor::new(tab, text, PLACE, settings, focused, start)
}

/// The same, in a window that has the focus.
fn editor(text: &str, start: Instant) -> Editor {
    editor_with(text, start, true)
}

/// A key that goes down.
fn down(key: u16, character: u16, controls: ControlKeys) -> KeyEvent {
    KeyEvent {
        key,
        scan: 0,
        character,
        down: true,
        controls,
        repeat: 1,
    }
}

fn enter() -> KeyEvent {
    down(VK_RETURN, 0x0d, ControlKeys::empty())
}

/// A key that writes no character, with Ctrl.
fn ctrl(key: u16) -> KeyEvent {
    down(key, 0, ControlKeys::LEFT_CTRL)
}

/// A key that writes no character.
fn tap(key: u16) -> KeyEvent {
    down(key, 0, ControlKeys::empty())
}

/// Types `text`, of letters and spaces, a key for each character.
fn type_text(editor: &mut Editor, text: &str, now: Instant) {
    for c in text.chars() {
        let key = u16::from(c.to_ascii_uppercase() as u8);
        let event = down(key, c as u16, ControlKeys::empty());
        assert_eq!(editor.key(&event, now), Edit::Taken, "{c:?}");
    }
}

/// What the field shows: its text, the cell of its caret, and its selected
/// cells.
fn shown(editor: &mut Editor) -> (String, usize, Range<usize>) {
    let view = editor.view();
    let text = view
        .cells
        .iter()
        .filter_map(|cell| match *cell {
            CellChar::Blank => Some(' '),
            CellChar::Narrow(c) | CellChar::WideLeft(c) => Some(c),
            CellChar::WideRight(_) => None,
        })
        .collect();
    (text, view.shown.caret, view.shown.selection)
}

/// The left button goes down at `x`, in the middle of the height of the
/// field, at `now`. Returns whether it is the field's.
fn click(editor: &mut Editor, x: i32, now: Instant) -> bool {
    editor.down(Button::Left, Point::new(x, 10), Modifiers::default(), now)
}

/// The width of the caret, if it is shown.
fn caret(editor: &mut Editor) -> Option<u32> {
    editor.view().caret
}

#[test]
fn it_starts_with_the_text_of_the_tab_all_selected() {
    let start = Instant::now();
    let mut editor = editor("pwsh", start);
    assert_eq!(shown(&mut editor), ("pwsh".to_owned(), 4, 0..4));
    assert_eq!(caret(&mut editor), Some(1));
    // The first frame draws it, and comes back when the caret blinks.
    assert_eq!(editor.update(start), (true, Some(start + 500 * MS)));
    assert_eq!(editor.update(start), (false, Some(start + 500 * MS)));
}

#[test]
fn enter_ends_it_with_the_name_and_esc_without_it() {
    let now = Instant::now();
    let mut editor = editor("pwsh", now);
    type_text(&mut editor, "build", now);
    // Going up, it does nothing.
    let up = KeyEvent {
        down: false,
        ..enter()
    };
    assert_eq!(editor.key(&up, now), Edit::Ignored);
    let named = Done::Renamed(Some("build".to_owned()));
    assert_eq!(editor.key(&enter(), now), Edit::Done(named));

    let mut editor = self::editor("pwsh", now);
    type_text(&mut editor, "build", now);
    let esc = down(VK_ESCAPE, 0x1b, ControlKeys::empty());
    assert_eq!(editor.key(&esc, now), Edit::Done(Done::Unchanged));
}

#[test]
fn the_spaces_at_the_ends_go_away_and_a_name_of_only_spaces_is_none() {
    let now = Instant::now();
    let mut editor = editor("pwsh", now);
    type_text(&mut editor, "  my build ", now);
    let named = Done::Renamed(Some("my build".to_owned()));
    assert_eq!(editor.commit(), named);

    let mut editor = self::editor("pwsh", now);
    type_text(&mut editor, "   ", now);
    assert_eq!(editor.commit(), Done::Renamed(None));

    // Backspace removes all the text, which is selected.
    let mut editor = self::editor("pwsh", now);
    let backspace = down(VK_BACK, 0x08, ControlKeys::empty());
    assert_eq!(editor.key(&backspace, now), Edit::Taken);
    assert_eq!(editor.key(&enter(), now), Edit::Done(Done::Renamed(None)));
}

#[test]
fn enter_with_the_text_that_the_tab_showed_changes_nothing() {
    let now = Instant::now();
    let mut editor = editor("pwsh", now);
    assert_eq!(editor.key(&enter(), now), Edit::Done(Done::Unchanged));

    // Also if it is the same again after editing it.
    let mut editor = self::editor("pwsh", now);
    type_text(&mut editor, "pwsh", now);
    assert_eq!(editor.commit(), Done::Unchanged);

    // Without taking its spaces away: the tab goes on showing its titles.
    let editor = self::editor(" pwsh ", now);
    assert_eq!(editor.commit(), Done::Unchanged);
}

#[test]
fn a_name_has_at_most_as_many_utf16_units_as_a_title() {
    let now = Instant::now();
    let mut editor = editor("pwsh", now);
    editor.paste(&"a".repeat(NAME_LIMIT + 1), now);
    let named = Done::Renamed(Some("a".repeat(NAME_LIMIT)));
    assert_eq!(editor.commit(), named);

    // A longer text that the tab showed is cut too, and Enter leaves it as
    // it was.
    let editor = self::editor(&"b".repeat(NAME_LIMIT + 1), now);
    assert_eq!(editor.commit(), Done::Unchanged);
}

#[test]
fn the_keys_of_the_field_are_its_own_and_the_others_do_nothing() {
    let now = Instant::now();
    let mut editor = editor("pwsh", now);
    let tab = down(VK_TAB, 0x09, ControlKeys::empty());
    for other in [tab, tap(VK_UP), tap(VK_DOWN), tap(VK_F2)] {
        assert_eq!(editor.key(&other, now), Edit::Ignored, "{:#x}", other.key);
    }
    assert_eq!(editor.key(&ctrl(VK_C), now), Edit::Copy("pwsh".to_owned()));
    assert_eq!(editor.key(&ctrl(VK_V), now), Edit::Paste);
    editor.paste("tab 2", now);
    assert_eq!(shown(&mut editor), ("tab 2".to_owned(), 5, 5..5));
    assert_eq!(editor.key(&tap(VK_HOME), now), Edit::Taken);
    assert_eq!(shown(&mut editor), ("tab 2".to_owned(), 0, 0..0));
    // Ctrl+X cuts the selection, which Ctrl+A selected.
    assert_eq!(editor.key(&ctrl(VK_A), now), Edit::Taken);
    assert_eq!(editor.key(&ctrl(VK_X), now), Edit::Copy("tab 2".to_owned()));
    assert_eq!(editor.commit(), Done::Renamed(None));
}

#[test]
fn alt_going_up_writes_the_character_of_its_code() {
    let now = Instant::now();
    let mut editor = editor("pwsh", now);
    let alt = KeyEvent {
        down: false,
        ..down(VK_MENU, u16::from(b'x'), ControlKeys::empty())
    };
    assert_eq!(editor.key(&alt, now), Edit::Taken);
    assert_eq!(editor.commit(), Done::Renamed(Some("x".to_owned())));
}

#[test]
fn the_keys_that_were_down_do_nothing_until_they_go_up() {
    let mut editor = editor("pwsh", Instant::now());
    editor.ignore([VK_RETURN, VK_SHIFT]);
    assert!(editor.was_down(VK_RETURN) && editor.was_down(VK_SHIFT));
    assert!(!editor.was_down(VK_TAB));
    editor.key_up(VK_RETURN);
    assert!(!editor.was_down(VK_RETURN) && editor.was_down(VK_SHIFT));
    // The window lost the focus: they go up in another window.
    editor.focus(false, Instant::now());
    assert!(!editor.was_down(VK_SHIFT));
}

#[test]
fn a_click_puts_the_caret_and_with_shift_moves_the_end_of_the_selection() {
    let now = Instant::now();
    let mut editor = editor("pwsh dev", now);
    // The left half of the third cell, up to its middle.
    assert!(click(&mut editor, 100 + 2 * 8 + 4, now));
    assert!(editor.is_pressed());
    assert_eq!(shown(&mut editor), ("pwsh dev".to_owned(), 2, 2..2));
    editor.up(Button::Left);
    assert!(!editor.is_pressed());
    // The right half of the sixth one, a second later, with Shift.
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    let at = Point::new(100 + 5 * 8 + 5, 10);
    assert!(editor.down(Button::Left, at, shift, now + 1000 * MS));
    assert_eq!(shown(&mut editor), ("pwsh dev".to_owned(), 6, 2..6));
}

#[test]
fn a_double_click_selects_a_word_and_a_triple_click_all_the_text() {
    let now = Instant::now();
    let mut editor = editor("pwsh dev", now);
    click(&mut editor, 109, now);
    editor.up(Button::Left);
    click(&mut editor, 109, now + 100 * MS);
    assert_eq!(shown(&mut editor), ("pwsh dev".to_owned(), 4, 0..4));
    editor.up(Button::Left);
    click(&mut editor, 109, now + 200 * MS);
    assert_eq!(shown(&mut editor), ("pwsh dev".to_owned(), 8, 0..8));
}

#[test]
fn only_the_left_button_selects_and_only_in_the_field() {
    let now = Instant::now();
    let mut editor = editor("pwsh", now);
    let modifiers = Modifiers::default();
    assert!(!editor.down(Button::Right, Point::new(110, 10), modifiers, now));
    assert!(!editor.down(Button::Left, Point::new(99, 10), modifiers, now));
    assert!(!editor.is_pressed());
    editor.moved(Point::new(110, 10), now);
    assert_eq!(shown(&mut editor), ("pwsh".to_owned(), 4, 0..4));
    assert!(editor.contains(Point::new(100, 0)) && editor.contains(Point::new(179, 31)));
    assert!(!editor.contains(Point::new(180, 10)) && !editor.contains(Point::new(120, 32)));

    // Another button going up does not end the selection, but losing the
    // mouse does.
    assert!(click(&mut editor, 110, now));
    editor.up(Button::Right);
    assert!(editor.is_pressed());
    editor.capture_lost();
    assert!(!editor.is_pressed());
    editor.moved(Point::new(130, 10), now);
    assert_eq!(shown(&mut editor), ("pwsh".to_owned(), 1, 1..1));
}

#[test]
fn dragging_out_of_the_field_shows_more_of_the_text_again_and_again() {
    let start = Instant::now();
    let mut editor = editor("abcdefghijklmnopqrst", start);
    editor.key(&tap(VK_HOME), start);
    assert_eq!(shown(&mut editor), ("abcdefghij".to_owned(), 0, 0..0));
    click(&mut editor, 100 + 2 * 8, start);
    editor.moved(Point::new(100 + 4 * 8, 10), start);
    assert_eq!(shown(&mut editor), ("abcdefghij".to_owned(), 4, 2..4));
    assert_eq!(editor.update(start).1, Some(start + 500 * MS));

    // Out on the right, in the right half of the cell after the last one:
    // up to after it, which the field shows.
    editor.moved(Point::new(185, 10), start);
    assert_eq!(shown(&mut editor), ("cdefghijkl".to_owned(), 9, 0..9));
    assert_eq!(editor.update(start).1, Some(start + DRAG_REPEAT));
    // Not yet; and then once more, while the pointer stays there.
    assert_eq!(editor.update(start + 99 * MS).1, Some(start + DRAG_REPEAT));
    let later = start + DRAG_REPEAT;
    assert_eq!(editor.update(later), (true, Some(later + DRAG_REPEAT)));
    assert_eq!(shown(&mut editor), ("efghijklmn".to_owned(), 9, 0..9));

    // Back in the field: no more.
    editor.moved(Point::new(100 + 6 * 8 + 2, 10), later);
    assert_eq!(shown(&mut editor), ("efghijklmn".to_owned(), 6, 0..6));
    assert_eq!(editor.update(later).1, Some(later + 500 * MS));

    // Out on the left, in the left half of the cell before the first one;
    // and the button goes up there.
    editor.moved(Point::new(95, 10), later);
    assert_eq!(shown(&mut editor), ("defghijklm".to_owned(), 0, 0..0));
    assert_eq!(editor.update(later).1, Some(later + DRAG_REPEAT));
    editor.up(Button::Left);
    assert_eq!(editor.update(later + DRAG_REPEAT).1, Some(later + 500 * MS));
    assert_eq!(shown(&mut editor), ("defghijklm".to_owned(), 0, 0..0));
}

#[test]
fn the_field_follows_its_place_and_the_scale_of_the_window() {
    let now = Instant::now();
    let mut editor = editor("pwsh dev", now);
    editor.update(now);
    let place = Place {
        area: Rect::new(100, 0, 140, 32),
        ..PLACE
    };
    editor.place(place);
    assert!(editor.update(now).0);
    // 5 cells: the end of the text, and the caret after it.
    assert_eq!(shown(&mut editor), (" dev".to_owned(), 4, 0..4));
    assert!(!editor.contains(Point::new(140, 10)));
    editor.place(place);
    assert!(!editor.update(now).0);

    editor.set_dpi(192);
    assert!(editor.update(now).0);
    assert_eq!(caret(&mut editor), Some(2));
}

#[test]
fn the_caret_blinks_while_the_window_has_the_focus() {
    let start = Instant::now();
    let mut editor = editor("pwsh", start);
    editor.update(start);
    let half = 500 * MS;
    assert_eq!(editor.update(start + half), (true, Some(start + 2 * half)));
    assert_eq!(caret(&mut editor), None);
    // A key shows it again, and it blinks from then.
    let typed = start + 600 * MS;
    type_text(&mut editor, "x", typed);
    assert_eq!(editor.update(typed), (true, Some(typed + half)));
    assert_eq!(caret(&mut editor), Some(1));
    // Hidden near the end, and after the timeout of the caret, shown
    // without blinking.
    assert_eq!(editor.update(typed + 4900 * MS).1, Some(typed + 5000 * MS));
    assert_eq!(caret(&mut editor), None);
    assert_eq!(editor.update(typed + 5000 * MS), (true, None));
    assert_eq!(caret(&mut editor), Some(1));

    // Without the focus it is not shown, and with it, it blinks again.
    let lost = typed + 6000 * MS;
    editor.focus(false, lost);
    assert_eq!(editor.update(lost), (true, None));
    assert_eq!(caret(&mut editor), None);
    let back = lost + 100 * MS;
    editor.focus(true, back);
    assert_eq!(editor.update(back), (true, Some(back + half)));
    assert_eq!(caret(&mut editor), Some(1));
    // The same focus again changes nothing.
    editor.focus(true, back + half);
    assert_eq!(editor.update(back), (false, Some(back + half)));
}

#[test]
fn without_the_focus_from_the_start_the_caret_is_not_shown() {
    let start = Instant::now();
    let mut editor = editor_with("pwsh", start, false);
    assert_eq!(caret(&mut editor), None);
    assert_eq!(editor.update(start), (true, None));
    assert_eq!(caret(&mut editor), None);
}
