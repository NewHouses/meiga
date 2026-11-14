use super::*;
use crate::geometry::GridPoint;

fn copy(terminal: &mut Terminal) -> Screen {
    let mut screen = Screen::new();
    terminal.copy_damage(&mut screen);
    screen
}

fn responses(terminal: &mut Terminal) -> Vec<u8> {
    let mut responses = Vec::new();
    terminal.take_responses(&mut responses);
    responses
}

const fn narrow(char: char) -> Cell {
    Cell {
        char: CellChar::Narrow(char),
        fg: Color::Default,
        bg: Color::Default,
        flags: CellFlags::empty(),
    }
}

#[test]
fn the_output_reaches_the_screen() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    assert_eq!(terminal.feed(b"ola\r\nmundo"), Changes::default());
    let screen = copy(&mut terminal);
    assert_eq!(screen.size(), GridSize::new(10, 3));
    assert!(screen.all_changed());
    assert_eq!(
        screen.row(0)[..4],
        [narrow('o'), narrow('l'), narrow('a'), Cell::default()]
    );
    assert_eq!(screen.row_text(1), "mundo");
    assert_eq!(screen.row_text(2), "");
    // A bar that blinks, unless the program chooses another cursor.
    let cursor = Cursor {
        row: 1,
        column: 5,
        shape: CursorShape::Beam,
        blinking: true,
    };
    assert_eq!(screen.cursor(), cursor);
    assert_eq!(screen.palette(), &Palette::MEIGA);
}

#[test]
fn only_what_changed_is_copied_again() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    terminal.feed(b"ola\r\nmundo");
    let mut screen = Screen::new();
    terminal.copy_damage(&mut screen);
    screen.clear_changes();

    // Without output, nothing.
    terminal.copy_damage(&mut screen);
    assert!(!screen.all_changed());
    assert_eq!(screen.changes().count(), 0);

    terminal.feed(b"\x1b[3;1Hxy");
    terminal.copy_damage(&mut screen);
    assert!(!screen.all_changed());
    // Where the cursor was, and the new text with the cursor after it.
    assert_eq!(
        screen.changes().collect::<Vec<_>>(),
        [(1, Columns::new(5, 7)), (2, Columns::new(0, 4))]
    );
    assert_eq!(screen.row_text(1), "mundo");
    assert_eq!(screen.row_text(2), "xy");
    assert_eq!((screen.cursor().row, screen.cursor().column), (2, 2));
}

#[test]
fn a_forgotten_screen_takes_all_of_another_terminal() {
    let mut first = Terminal::new(GridSize::new(10, 3));
    let mut second = Terminal::new(GridSize::new(10, 3));
    first.feed(b"uno\r\ndous");
    second.feed(b"tres");
    let mut screen = Screen::new();
    first.copy_damage(&mut screen);
    // The second terminal was already copied elsewhere, as a tab that was
    // shown before: nothing changed in it since then.
    second.copy_damage(&mut Screen::new());
    screen.clear_changes();

    // Without forgetting it, the screen keeps the rows of the first one.
    second.copy_damage(&mut screen);
    assert_eq!(screen.row_text(1), "dous");
    screen.clear_changes();

    screen.forget();
    second.copy_damage(&mut screen);
    assert!(screen.all_changed());
    assert_eq!(screen.size(), GridSize::new(10, 3));
    assert_eq!(screen.row_text(0), "tres");
    assert_eq!(screen.row_text(1), "");
    assert_eq!((screen.cursor().row, screen.cursor().column), (0, 4));
}

#[test]
fn the_program_gets_its_answers() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    let answers = Changes {
        responses: true,
        ..Changes::default()
    };
    // Device attributes (DA1), status (DSR 5), cursor position (DSR 6) and
    // size in cells.
    assert_eq!(terminal.feed(b"\x1b[c"), answers);
    assert_eq!(terminal.feed(b"\x1b[5n"), answers);
    assert_eq!(terminal.feed(b"\x1b[2;3H\x1b[6n\x1b[18t"), answers);
    assert_eq!(
        responses(&mut terminal),
        b"\x1b[?6c\x1b[0n\x1b[2;3R\x1b[8;3;10t"
    );
    assert_eq!(responses(&mut terminal), b"");
    // The secondary attributes (DA2), with the version of
    // alacritty_terminal, 0.26.0.
    assert_eq!(terminal.feed(b"\x1b[>c"), answers);
    assert_eq!(responses(&mut terminal), b"\x1b[>0;2600;1c");
    // The size in pixels, only once the terminal knows the size of a cell:
    // 3 rows of 20 pixels and 10 columns of 9.
    assert_eq!(terminal.feed(b"\x1b[14t"), Changes::default());
    assert_eq!(responses(&mut terminal), b"");
    terminal.set_cell_size(Size::new(9, 20));
    assert_eq!(terminal.feed(b"\x1b[14t"), answers);
    assert_eq!(responses(&mut terminal), b"\x1b[4;60;90t");
}

#[test]
fn questions_about_colours_are_answered_with_the_palette() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    // A colour of the palette (OSC 4), the text (10), the background (11)
    // and the cursor (12), each with the end that the question used: BEL or
    // ST. The cursor has the colour of the text.
    terminal.feed(b"\x1b]4;1;?\x07\x1b]10;?\x1b\\\x1b]11;?\x07\x1b]12;?\x07");
    assert_eq!(
        String::from_utf8(responses(&mut terminal)).unwrap(),
        "\x1b]4;1;rgb:c5c5/0f0f/1f1f\x07\
         \x1b]10;rgb:d4d4/d7d7/dddd\x1b\\\
         \x1b]11;rgb:1616/1818/1d1d\x07\
         \x1b]12;rgb:d4d4/d7d7/dddd\x07"
    );
    // With the colours that the program changed, and in the order of the
    // questions: the attributes (DA1) after the colour.
    terminal.feed(b"\x1b]11;#ffffff\x07\x1b[c\x1b]11;?\x07\x1b[c");
    assert_eq!(
        String::from_utf8(responses(&mut terminal)).unwrap(),
        "\x1b[?6c\x1b]11;rgb:ffff/ffff/ffff\x07\x1b[?6c"
    );
    // And with the theme.
    let mut theme = Palette::MEIGA;
    theme.colors[1] = Rgb::new(1, 2, 3);
    terminal.set_theme(theme);
    terminal.feed(b"\x1b]4;1;?\x07");
    assert_eq!(responses(&mut terminal), b"\x1b]4;1;rgb:0101/0202/0303\x07");
}

#[test]
fn a_change_of_the_palette_changes_every_cell() {
    let mut terminal = Terminal::new(GridSize::new(4, 2));
    terminal.feed(b"\x1b[31mab");
    let mut screen = Screen::new();
    terminal.copy_damage(&mut screen);
    screen.clear_changes();

    // The cells keep the index of their colour, which now is another one.
    terminal.feed(b"\x1b]4;1;rgb:ff/00/00\x07");
    terminal.copy_damage(&mut screen);
    assert!(screen.all_changed());
    assert_eq!(screen.palette().colors[1], Rgb::new(0xff, 0, 0));
    assert_eq!(screen.row(0)[0].fg, Color::Indexed(1));
    screen.clear_changes();

    // Back to the colour of the theme.
    terminal.feed(b"\x1b]104;1\x07");
    terminal.copy_damage(&mut screen);
    assert!(screen.all_changed());
    assert_eq!(screen.palette(), &Palette::MEIGA);
    screen.clear_changes();

    // The colour of the cursor only changes the cells of the cursor.
    terminal.feed(b"\x1b]12;#00ff00\x07");
    terminal.copy_damage(&mut screen);
    assert!(!screen.all_changed());
    assert_eq!(
        screen.changes().collect::<Vec<_>>(),
        [(0, Columns::new(2, 4))]
    );
    assert_eq!(screen.palette().cursor, Rgb::new(0, 0xff, 0));
    screen.clear_changes();

    terminal.copy_damage(&mut screen);
    assert_eq!(screen.changes().count(), 0);
}

#[test]
fn the_title_and_the_bell() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    let title = Changes {
        title: true,
        ..Changes::default()
    };
    assert_eq!(terminal.title(), None);
    assert_eq!(terminal.feed(b"\x1b]0;meiga\x07"), title);
    assert_eq!(terminal.title(), Some("meiga"));
    assert_eq!(terminal.feed(b"\x1b]2;  pwsh  \x1b\\"), title);
    assert_eq!(terminal.title(), Some("pwsh"));
    // Saved and restored (XTWINOPS 22 and 23).
    assert_eq!(terminal.feed(b"\x1b[22t\x1b]0;vim\x07"), title);
    assert_eq!(terminal.title(), Some("vim"));
    assert_eq!(terminal.feed(b"\x1b[23t"), title);
    assert_eq!(terminal.title(), Some("pwsh"));
    assert_eq!(terminal.feed(b"\x1b[23t"), Changes::default());
    // An empty title is no title.
    assert_eq!(terminal.feed(b"\x1b]0;\x07"), title);
    assert_eq!(terminal.title(), None);

    let bell = Changes {
        bell: true,
        ..Changes::default()
    };
    assert_eq!(terminal.feed(b"\x07"), bell);
    assert_eq!(terminal.feed(b"ola"), Changes::default());
}

#[test]
fn osc_52_only_copies() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    let copied = Changes {
        clipboard: true,
        ..Changes::default()
    };
    // "b2xh" is "ola" in base64.
    assert_eq!(terminal.feed(b"\x1b]52;c;b2xh\x07"), copied);
    assert_eq!(terminal.take_clipboard().as_deref(), Some("ola"));
    assert_eq!(terminal.take_clipboard(), None);
    // Reading the clipboard, the primary selection, and text that is not
    // base64.
    for request in [
        &b"\x1b]52;c;?\x07"[..],
        b"\x1b]52;p;b2xh\x07",
        b"\x1b]52;c;!!\x07",
    ] {
        assert_eq!(terminal.feed(request), Changes::default());
    }
    assert_eq!(responses(&mut terminal), b"");
    assert_eq!(terminal.take_clipboard(), None);
}

#[test]
fn the_modes_follow_the_program() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    let defaults = Modes {
        show_cursor: true,
        alternate_scroll: true,
        ..Modes::default()
    };
    assert_eq!(terminal.modes(), defaults);

    terminal.feed(b"\x1b[?9001h\x1b[?1004h\x1b[?2004h\x1b[?1h\x1b=\x1b[?1002h\x1b[?1006h");
    terminal.feed(b"\x1b[?1049h\x1b[?25l\x1b[20h\x1b[?1007l");
    let modes = Modes {
        win32_input: true,
        focus_events: true,
        bracketed_paste: true,
        app_cursor: true,
        app_keypad: true,
        mouse_clicks: false,
        mouse_drag: true,
        mouse_motion: false,
        sgr_mouse: true,
        utf8_mouse: false,
        alternate_scroll: false,
        alt_screen: true,
        show_cursor: false,
        line_feed_new_line: true,
    };
    assert_eq!(terminal.modes(), modes);

    // The mouse modes, and their encodings, exclude each other.
    terminal.feed(b"\x1b[?1000h\x1b[?1005h");
    let mouse = |terminal: &Terminal| {
        let modes = terminal.modes();
        [
            modes.mouse_clicks,
            modes.mouse_drag,
            modes.mouse_motion,
            modes.sgr_mouse,
            modes.utf8_mouse,
        ]
    };
    assert_eq!(mouse(&terminal), [true, false, false, false, true]);
    terminal.feed(b"\x1b[?1003h");
    assert_eq!(mouse(&terminal), [false, false, true, false, true]);

    terminal.feed(b"\x1b[?9001l\x1b[?1004l");
    assert!(!terminal.modes().win32_input);
    assert!(!terminal.modes().focus_events);

    // A reset (RIS) leaves them as at the start.
    terminal.feed(b"\x1b[?9001h\x1bc");
    assert_eq!(terminal.modes(), defaults);
}

#[test]
fn colours_and_styles_reach_the_cells() {
    let mut terminal = Terminal::new(GridSize::new(10, 1));
    terminal.feed(b"\x1b[1;3;4;31;42mA");
    terminal.feed(b"\x1b[0;38;5;200;48;2;1;2;3;7;9mB");
    terminal.feed(b"\x1b[0;2;8;4:3;91;104mC");
    terminal.feed(b"\x1b[0;4:2mD\x1b[0;4:4mE\x1b[0;4:5mF");
    terminal.feed(b"\x1b[0;35;45m\x1b[39;49mG");
    let screen = copy(&mut terminal);
    let cells: Vec<(Color, Color, CellFlags)> = screen.row(0)[..7]
        .iter()
        .map(|cell| (cell.fg, cell.bg, cell.flags))
        .collect();
    let default = Color::Default;
    assert_eq!(
        cells,
        [
            (
                Color::Indexed(1),
                Color::Indexed(2),
                CellFlags::BOLD | CellFlags::ITALIC | CellFlags::UNDERLINE
            ),
            (
                Color::Indexed(200),
                Color::Rgb(Rgb::new(1, 2, 3)),
                CellFlags::INVERSE | CellFlags::STRIKEOUT
            ),
            (
                Color::Indexed(9),
                Color::Indexed(12),
                CellFlags::DIM | CellFlags::HIDDEN | CellFlags::UNDERCURL
            ),
            (default, default, CellFlags::DOUBLE_UNDERLINE),
            (default, default, CellFlags::DOTTED_UNDERLINE),
            (default, default, CellFlags::DASHED_UNDERLINE),
            (default, default, CellFlags::empty()),
        ]
    );
    assert_eq!(screen.row_text(0), "ABCDEFG");
}

#[test]
fn a_wide_character_takes_two_cells() {
    let mut terminal = Terminal::new(GridSize::new(6, 2));
    terminal.feed("a中b".as_bytes());
    let screen = copy(&mut terminal);
    let chars: Vec<CellChar> = screen.row(0).iter().map(|cell| cell.char).collect();
    assert_eq!(
        chars,
        [
            CellChar::Narrow('a'),
            CellChar::WideLeft('中'),
            CellChar::WideRight('中'),
            CellChar::Narrow('b'),
            CellChar::Blank,
            CellChar::Blank,
        ]
    );
    assert_eq!(screen.row_text(0), "a中b");

    // When it does not fit at the end of a row, it goes to the next one.
    let mut terminal = Terminal::new(GridSize::new(4, 2));
    terminal.feed("abc中".as_bytes());
    let screen = copy(&mut terminal);
    assert_eq!(screen.row(0)[3].char, CellChar::Blank);
    assert_eq!(screen.row_text(0), "abc");
    assert_eq!(screen.row(1)[0].char, CellChar::WideLeft('中'));
    assert_eq!(screen.row_text(1), "中");
}

#[test]
fn a_tab_leaves_blank_cells() {
    let mut terminal = Terminal::new(GridSize::new(20, 1));
    terminal.feed(b"a\tb");
    let screen = copy(&mut terminal);
    assert_eq!(screen.row(0)[1].char, CellChar::Blank);
    assert_eq!(screen.row_text(0), "a       b");
}

#[test]
fn the_cursor_follows_the_program() {
    let mut terminal = Terminal::new(GridSize::new(6, 2));
    let cursor = |terminal: &mut Terminal, bytes: &[u8]| {
        terminal.feed(bytes);
        let cursor = copy(terminal).cursor();
        (cursor.column, cursor.shape, cursor.blinking)
    };
    // A bar that blinks, until the program chooses another one.
    assert_eq!(cursor(&mut terminal, b""), (0, CursorShape::Beam, true));
    // DECSCUSR.
    assert_eq!(
        cursor(&mut terminal, b"\x1b[6 q"),
        (0, CursorShape::Beam, false)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[1 q"),
        (0, CursorShape::Block, true)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[4 q"),
        (0, CursorShape::Underline, false)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[?12h"),
        (0, CursorShape::Underline, true)
    );
    // Back to the one of meiga.
    assert_eq!(
        cursor(&mut terminal, b"\x1b[0 q"),
        (0, CursorShape::Beam, true)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[?12l"),
        (0, CursorShape::Beam, false)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[?25l"),
        (0, CursorShape::Hidden, false)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[?25h"),
        (0, CursorShape::Beam, false)
    );
    // On the right half of a wide character, it is drawn over all of it.
    let on_wide = cursor(&mut terminal, "a中\x1b[3G".as_bytes());
    assert_eq!(on_wide, (1, CursorShape::Beam, false));
}

#[test]
fn the_cursor_on_a_lone_right_half_stays_in_its_cell() {
    // Deleting the left half of a wide character leaves its right half
    // alone, here in the first column, with the cursor on it.
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    terminal.feed("中\r\x1b[P".as_bytes());
    let screen = copy(&mut terminal);
    assert_eq!(screen.row(0)[0].char, CellChar::Blank);
    let cursor = screen.cursor();
    assert_eq!(
        (cursor.row, cursor.column, cursor.shape),
        (0, 0, CursorShape::Beam)
    );
    // And after a narrow character, which is not its left half.
    terminal.feed("\x1b[2Ja中\x1b[2G\x1b[P".as_bytes());
    let screen = copy(&mut terminal);
    assert_eq!(screen.row(0)[1].char, CellChar::Blank);
    assert_eq!(screen.cursor().column, 1);
}

#[test]
fn the_cells_of_the_cursor_are_changes_when_it_is_drawn() {
    let mut terminal = Terminal::new(GridSize::new(6, 2));
    let mut screen = Screen::new();
    let mut changes = |terminal: &mut Terminal, bytes: &[u8]| {
        terminal.feed(bytes);
        terminal.copy_damage(&mut screen);
        let changes: Vec<_> = screen.changes().collect();
        screen.clear_changes();
        changes
    };
    changes(&mut terminal, "a中".as_bytes());
    // From the end of the row to the right half of the wide character,
    // which is drawn from its left half.
    assert_eq!(
        changes(&mut terminal, b"\x1b[3G"),
        [(0, Columns::new(1, 5))]
    );
    assert_eq!(changes(&mut terminal, b"\x1b[3G"), []);
    // Hidden, only where it was; while it is hidden, nothing.
    assert_eq!(
        changes(&mut terminal, b"\x1b[?25l"),
        [(0, Columns::new(1, 3))]
    );
    assert_eq!(changes(&mut terminal, b"\x1b[6G"), []);
    // Shown again, where it is now: in the last column, only one cell.
    assert_eq!(
        changes(&mut terminal, b"\x1b[?25h"),
        [(0, Columns::new(5, 6))]
    );
}

#[test]
fn a_synchronized_update_waits_for_its_end() {
    let mut terminal = Terminal::new(GridSize::new(10, 1));
    let before = Instant::now();
    assert_eq!(terminal.feed(b"\x1b[?2026hola\x1b[c"), Changes::default());
    let deadline = terminal.sync_deadline().unwrap();
    assert!(deadline > before);
    assert!(deadline <= Instant::now() + std::time::Duration::from_millis(150));
    assert_eq!(copy(&mut terminal).row_text(0), "");

    // When the deadline passes.
    let answers = Changes {
        responses: true,
        ..Changes::default()
    };
    assert_eq!(terminal.end_sync(), answers);
    assert_eq!(terminal.sync_deadline(), None);
    assert_eq!(copy(&mut terminal).row_text(0), "ola");
    assert_eq!(responses(&mut terminal), b"\x1b[?6c");

    // When the program ends it.
    terminal.feed(b"\x1b[?2026hxy");
    assert_eq!(copy(&mut terminal).row_text(0), "ola");
    terminal.feed(b"z\x1b[?2026l");
    assert_eq!(terminal.sync_deadline(), None);
    assert_eq!(copy(&mut terminal).row_text(0), "olaxyz");
}

#[test]
fn after_a_resize_everything_is_copied_again() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    terminal.feed(b"ola");
    let mut screen = Screen::new();
    terminal.copy_damage(&mut screen);
    screen.clear_changes();
    for size in [GridSize::new(20, 5), GridSize::new(5, 2)] {
        terminal.resize(size);
        assert_eq!(terminal.size(), size);
        terminal.copy_damage(&mut screen);
        assert_eq!(screen.size(), size);
        assert!(screen.all_changed());
        assert_eq!(screen.row_text(0), "ola");
        screen.clear_changes();
    }
}

#[test]
fn the_grid_is_never_smaller_than_two_columns_and_one_row() {
    let mut terminal = Terminal::new(GridSize::new(0, 0));
    assert_eq!(terminal.size(), MIN_SIZE);
    terminal.feed("中😀ab\r\n中\x1b[5;5H中\x1b[2@\x1b[L".as_bytes());
    terminal.resize(GridSize::new(1, 0));
    assert_eq!(terminal.size(), MIN_SIZE);
    terminal.feed("中中".as_bytes());
    let screen = copy(&mut terminal);
    assert_eq!(screen.size(), MIN_SIZE);
    assert_eq!(screen.row_text(0), "中");
}

/// The text of each row of the screen.
fn texts(terminal: &mut Terminal) -> Vec<String> {
    let screen = copy(terminal);
    (0..screen.size().rows)
        .map(|row| screen.row_text(row))
        .collect()
}

/// The row and the column of the cursor.
fn cursor_at(terminal: &Terminal) -> (u16, u16) {
    let cursor = terminal.cursor();
    (cursor.row, cursor.column)
}

#[test]
fn a_taller_screen_adds_blank_rows_and_leaves_the_history_alone() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    terminal.feed(b"a\r\nb\r\nc\r\nd\r\ne");
    terminal.resize(GridSize::new(10, 5));
    assert_eq!(texts(&mut terminal), ["c", "d", "e", "", ""]);
    assert_eq!(cursor_at(&terminal), (2, 1));
    assert_eq!(terminal.history().rows, 2);
}

#[test]
fn a_shorter_screen_keeps_the_cursor_and_the_rows_above_it() {
    // Only blank rows go when the text fits.
    let mut terminal = Terminal::new(GridSize::new(10, 5));
    terminal.feed(b"a\r\nb");
    terminal.resize(GridSize::new(10, 3));
    assert_eq!(texts(&mut terminal), ["a", "b", ""]);
    assert_eq!(cursor_at(&terminal), (1, 1));
    assert_eq!(terminal.history().rows, 0);

    // When it does not, the first rows go into the history, and they stay
    // there when the screen grows again.
    let mut terminal = Terminal::new(GridSize::new(10, 5));
    terminal.feed(b"a\r\nb\r\nc");
    terminal.resize(GridSize::new(10, 2));
    assert_eq!(texts(&mut terminal), ["b", "c"]);
    assert_eq!(cursor_at(&terminal), (1, 1));
    terminal.resize(GridSize::new(10, 5));
    assert_eq!(texts(&mut terminal), ["b", "c", "", "", ""]);
    assert_eq!(cursor_at(&terminal), (1, 1));
    assert_eq!(terminal.history().rows, 1);
}

#[test]
fn narrower_lines_wrap_downwards_and_join_again_when_wider() {
    let mut terminal = Terminal::new(GridSize::new(8, 3));
    terminal.feed(b"abcdef\r\nxy");
    terminal.resize(GridSize::new(4, 3));
    assert_eq!(texts(&mut terminal), ["abcd", "ef", "xy"]);
    assert_eq!(cursor_at(&terminal), (2, 2));
    assert_eq!(terminal.history().rows, 0);
    terminal.resize(GridSize::new(8, 3));
    assert_eq!(texts(&mut terminal), ["abcdef", "xy", ""]);
    assert_eq!(cursor_at(&terminal), (1, 2));
    terminal.feed(b"z");
    assert_eq!(texts(&mut terminal), ["abcdef", "xyz", ""]);
}

#[test]
fn the_cursor_keeps_its_row_when_many_rows_below_it_wrap() {
    // As in ConPTY: the screen starts at the row of the cursor, and the rows
    // that do not fit below it go away.
    let mut terminal = Terminal::new(GridSize::new(4, 4));
    terminal.feed(b"x\r\na\r\nbbbb\r\ncccc\x1b[2;1H");
    terminal.resize(GridSize::new(2, 4));
    assert_eq!(texts(&mut terminal), ["a", "bb", "bb", "cc"]);
    assert_eq!(cursor_at(&terminal), (0, 0));
    terminal.feed(b"A");
    assert_eq!(texts(&mut terminal)[0], "A");
}

#[test]
fn the_cursor_keeps_its_place_after_the_last_character_of_a_full_row() {
    let mut terminal = Terminal::new(GridSize::new(4, 2));
    terminal.feed(b"abcd");
    terminal.resize(GridSize::new(6, 2));
    terminal.feed(b"e");
    assert_eq!(texts(&mut terminal), ["abcde", ""]);

    let mut terminal = Terminal::new(GridSize::new(4, 2));
    terminal.feed(b"abcd");
    terminal.resize(GridSize::new(2, 2));
    terminal.feed(b"e");
    assert_eq!(texts(&mut terminal), ["cd", "e"]);
}

#[test]
fn on_the_alternate_screen_the_primary_one_gets_the_last_size_when_the_program_leaves() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    terminal.feed(b"a\r\nb\r\nc\r\nd\r\ne\x1b[?1049h\x1b[HALT");
    terminal.resize(GridSize::new(10, 5));
    assert_eq!(texts(&mut terminal), ["ALT", "", "", "", ""]);
    terminal.resize(GridSize::new(4, 2));
    assert_eq!(texts(&mut terminal), ["ALT", ""]);
    // Back to the size that the primary screen had: it does not change.
    terminal.resize(GridSize::new(10, 3));
    terminal.feed(b"\x1b[?1049l");
    assert_eq!(texts(&mut terminal), ["c", "d", "e"]);
    assert_eq!(cursor_at(&terminal), (2, 1));
    assert_eq!(terminal.history().rows, 2);

    // Taller: blank rows at the bottom, as on the primary screen, and the
    // history stays where it is.
    terminal.feed(b"\x1b[?1049h");
    terminal.resize(GridSize::new(10, 5));
    terminal.feed(b"\x1b[?1049l");
    assert_eq!(texts(&mut terminal), ["c", "d", "e", "", ""]);
    assert_eq!(cursor_at(&terminal), (2, 1));
    assert_eq!(terminal.history().rows, 2);
}

#[test]
fn the_output_after_leaving_the_alternate_screen_reaches_the_primary_one() {
    // In the same read, inside a synchronized update, and in one that ends at
    // its deadline.
    for (output, end) in [
        (&b"\x1b[?1049lX"[..], false),
        (b"\x1b[?2026h\x1b[?1049lX\x1b[?2026l", false),
        (b"\x1b[?2026h\x1b[?1049lX", true),
    ] {
        let mut terminal = Terminal::new(GridSize::new(10, 3));
        terminal.feed(b"a\r\nb\x1b[?1049h");
        terminal.resize(GridSize::new(10, 4));
        terminal.feed(output);
        if end {
            terminal.end_sync();
        }
        assert_eq!(texts(&mut terminal), ["a", "bX", "", ""], "{output:?}");
    }
}

#[test]
fn a_reset_on_the_alternate_screen_leaves_an_empty_primary_screen_with_history() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    terminal.feed(b"a\r\nb\r\nc\r\nd\x1b[?1049h");
    terminal.resize(GridSize::new(10, 4));
    terminal.feed(b"\x1bc");
    assert!(!terminal.modes().alt_screen);
    assert_eq!(texts(&mut terminal), ["", "", "", ""]);
    assert_eq!(terminal.history().rows, 0);
    terminal.feed(b"1\r\n2\r\n3\r\n4\r\n5\r\n6");
    assert_eq!(texts(&mut terminal), ["3", "4", "5", "6"]);
    assert_eq!(terminal.history().rows, 2);
}

/// A terminal of 10 × 3 with the lines 1 to 10: 7 of them in the history.
fn ten_lines() -> Terminal {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    let lines: Vec<String> = (1..=10).map(|line| line.to_string()).collect();
    terminal.feed(lines.join("\r\n").as_bytes());
    terminal
}

#[test]
fn the_view_moves_through_the_history() {
    let mut terminal = ten_lines();
    assert_eq!(terminal.history(), History { rows: 7, offset: 0 });
    assert!(terminal.scroll(Scroll::Rows(2)));
    assert_eq!(texts(&mut terminal), ["6", "7", "8"]);
    // The cursor is in a row that the view does not show.
    assert_eq!(terminal.cursor().shape, CursorShape::Hidden);
    assert!(terminal.scroll(Scroll::PageUp));
    assert_eq!(texts(&mut terminal), ["3", "4", "5"]);
    assert!(terminal.scroll(Scroll::Top));
    assert_eq!(texts(&mut terminal), ["1", "2", "3"]);
    assert!(!terminal.scroll(Scroll::Rows(1)));
    assert!(terminal.scroll(Scroll::PageDown));
    assert_eq!(texts(&mut terminal), ["4", "5", "6"]);
    assert!(terminal.scroll_to(1));
    assert_eq!(texts(&mut terminal), ["7", "8", "9"]);
    assert!(terminal.scroll_to(100));
    assert_eq!(terminal.history(), History { rows: 7, offset: 7 });
    assert!(terminal.scroll(Scroll::Bottom));
    assert_eq!(texts(&mut terminal), ["8", "9", "10"]);
    assert_eq!(terminal.cursor().shape, CursorShape::Beam);
    assert!(!terminal.scroll(Scroll::Rows(-1)));
}

#[test]
fn the_output_does_not_move_the_view_of_the_history() {
    let mut terminal = ten_lines();
    terminal.scroll(Scroll::Rows(2));
    terminal.feed(b"\r\n11\r\n12");
    assert_eq!(texts(&mut terminal), ["6", "7", "8"]);
    assert_eq!(terminal.history(), History { rows: 9, offset: 4 });
}

#[test]
fn the_rows_that_leave_a_region_below_the_top_do_not_move_the_view() {
    // They do not go into the history. The region scrolls with line feeds at
    // its bottom, with `CSI S`, and with characters and a tab that wrap
    // there; `CSI M` moves the rows below the cursor, which is not on the
    // first row; and a region that is not valid changes nothing.
    for output in [
        &b"\x1b[2;3r\x1b[3;1H\n\n\n\n\n\n\n\n\n\n"[..],
        b"\x1b[2;3r\x1b[S\x1b[S\x1b[S",
        b"\x1b[2;3r\x1b[3;1Habcdefghijklmnopqrstuvwxy",
        b"\x1b[2;3r\x1b[3;10Hx\t",
        b"\x1b[2;3r\x1b[1;1r\x1b[3;1H\n\n",
        b"\x1b[2;1H\x1b[M\x1b[M",
    ] {
        let mut terminal = ten_lines();
        terminal.scroll(Scroll::Rows(2));
        terminal.feed(output);
        assert_eq!(
            terminal.history(),
            History { rows: 7, offset: 2 },
            "{output:?}"
        );
        assert_eq!(texts(&mut terminal), ["6", "7", "8"], "{output:?}");
    }

    // From the first row of the history, it does not go beyond it.
    let mut terminal = ten_lines();
    terminal.scroll(Scroll::Top);
    terminal.feed(b"\x1b[2;3r\x1b[3;1H\n\n\n\n\n\n\n\n\n\n");
    assert_eq!(terminal.history(), History { rows: 7, offset: 7 });
    assert_eq!(texts(&mut terminal), ["1", "2", "3"]);

    // A region that starts at the top sends its rows to the history, and the
    // view moves with them.
    let mut terminal = ten_lines();
    terminal.scroll(Scroll::Rows(2));
    terminal.feed(b"\x1b[1;2r\x1b[2;1H\n\n");
    assert_eq!(terminal.history(), History { rows: 9, offset: 4 });
    assert_eq!(texts(&mut terminal), ["6", "7", "8"]);
}

/// With the view two rows above the screen of `terminal`, two line feeds at
/// the bottom of the screen: the rows go into the history, and the view
/// keeps showing the same ones.
fn assert_the_rows_go_into_the_history(terminal: &mut Terminal) {
    terminal.scroll(Scroll::Bottom);
    terminal.scroll(Scroll::Rows(2));
    let shown = texts(terminal);
    let rows = terminal.history().rows;
    let bottom = terminal.size().rows;
    terminal.feed(format!("\x1b[{bottom};1H\n\n").as_bytes());
    assert_eq!(texts(terminal), shown);
    assert_eq!(
        terminal.history(),
        History {
            rows: rows + 2,
            offset: 4
        }
    );
}

#[test]
fn a_new_size_a_reset_or_deccolm_make_the_region_the_whole_screen_again() {
    let mut terminal = ten_lines();
    terminal.feed(b"\x1b[2;3r");
    terminal.resize(GridSize::new(10, 4));
    assert_the_rows_go_into_the_history(&mut terminal);

    // A region of the alternate screen, which the primary one gets back
    // with another size.
    let mut terminal = ten_lines();
    terminal.feed(b"\x1b[?1049h");
    terminal.resize(GridSize::new(10, 4));
    terminal.feed(b"\x1b[2;3r\x1b[?1049l");
    assert_the_rows_go_into_the_history(&mut terminal);

    let mut terminal = ten_lines();
    terminal.feed(b"\x1b[2;3r\x1bc1\r\n2\r\n3\r\n4\r\n5\r\n6");
    assert_the_rows_go_into_the_history(&mut terminal);

    // DECCOLM, set or reset: alacritty_terminal keeps the columns, but it
    // clears the screen and the region.
    for column_mode in [&b"\x1b[?3h"[..], b"\x1b[?3l"] {
        let mut terminal = ten_lines();
        terminal.feed(b"\x1b[2;3r");
        terminal.feed(column_mode);
        assert_the_rows_go_into_the_history(&mut terminal);
    }
}

#[test]
fn a_resize_keeps_the_view_inside_the_history() {
    let mut terminal = ten_lines();
    terminal.scroll(Scroll::Top);
    for size in [
        GridSize::new(10, 8),
        GridSize::new(2, 2),
        GridSize::new(10, 3),
    ] {
        terminal.resize(size);
        let history = terminal.history();
        assert!(history.offset <= history.rows, "{size:?}: {history:?}");
        copy(&mut terminal);
    }
    // At the bottom, it stays at the bottom.
    terminal.scroll(Scroll::Bottom);
    terminal.resize(GridSize::new(10, 2));
    assert_eq!(terminal.history().offset, 0);
}

/// The selected cells of `row`: `#` for each one, and `.` for the others.
fn marks(screen: &Screen, row: u16) -> String {
    screen
        .row(row)
        .iter()
        .map(|cell| {
            if cell.flags.contains(CellFlags::SELECTED) {
                '#'
            } else {
                '.'
            }
        })
        .collect()
}

#[test]
fn a_selection_of_cells_marks_them_and_gives_their_text() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    terminal.feed(b"ola mundo\r\nadeus");
    let mut screen = Screen::new();
    terminal.copy_damage(&mut screen);
    screen.clear_changes();

    // A click that does not move selects nothing.
    terminal.select(SelectionKind::Cells, GridPoint::new(1, 0), Side::Left);
    assert!(!terminal.has_selection());
    assert_eq!(terminal.selection_text(), None);
    terminal.copy_damage(&mut screen);
    assert_eq!(screen.changes().count(), 0);

    // From the left of the second cell to the right of the sixth one.
    terminal.select_to(GridPoint::new(5, 0), Side::Right);
    assert!(terminal.has_selection());
    assert_eq!(terminal.selection_text().as_deref(), Some("la mu"));
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 0), ".#####....");
    assert_eq!(
        screen.changes().collect::<Vec<_>>(),
        [(0, Columns::new(1, 6))]
    );
    screen.clear_changes();

    // To the left of the third cell of the next row: the rest of the first
    // row, with an LF at its end, and two cells of the second one.
    terminal.select_to(GridPoint::new(2, 1), Side::Left);
    assert_eq!(terminal.selection_text().as_deref(), Some("la mundo\nad"));
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 0), ".#########");
    assert_eq!(marks(&screen, 1), "##........");
    assert_eq!(
        screen.changes().collect::<Vec<_>>(),
        [(0, Columns::new(6, 10)), (1, Columns::new(0, 2))]
    );
    screen.clear_changes();

    // Backwards, from where it started.
    terminal.select_to(GridPoint::new(0, 0), Side::Left);
    assert_eq!(terminal.selection_text().as_deref(), Some("o"));
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 0), "#.........");
    assert_eq!(marks(&screen, 1), "..........");
    screen.clear_changes();

    assert!(terminal.clear_selection());
    assert!(!terminal.clear_selection());
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 0), "..........");
    assert_eq!(
        screen.changes().collect::<Vec<_>>(),
        [(0, Columns::new(0, 1))]
    );
    assert_eq!(screen.row_text(0), "ola mundo");
}

#[test]
fn a_double_click_selects_a_word_a_path_or_up_to_the_other_bracket() {
    let mut terminal = Terminal::new(GridSize::new(40, 2));
    terminal.feed("cat C:\\x\\f.rs:12:5 (ok, \"si\") a  b\r\n中文 ok".as_bytes());
    let mut word = |column, row| {
        terminal.select(
            SelectionKind::Words,
            GridPoint::new(column, row),
            Side::Left,
        );
        terminal.selection_text()
    };
    assert_eq!(word(1, 0).as_deref(), Some("cat"));
    // Neither the colons nor the backslashes end a word.
    assert_eq!(word(9, 0).as_deref(), Some("C:\\x\\f.rs:12:5"));
    assert_eq!(word(20, 0).as_deref(), Some("ok"));
    assert_eq!(word(25, 0).as_deref(), Some("si"));
    // On a bracket, up to the other one, in both directions.
    assert_eq!(word(19, 0).as_deref(), Some("(ok, \"si\")"));
    assert_eq!(word(28, 0).as_deref(), Some("(ok, \"si\")"));
    // On a space or a separator, only it.
    assert_eq!(word(3, 0).as_deref(), Some(" "));
    assert_eq!(word(31, 0).as_deref(), Some(" "));
    assert_eq!(word(22, 0).as_deref(), Some(","));
    // Past the end of the text, a cell without text.
    assert_eq!(word(36, 0), None);
    // On either half of a wide character.
    assert_eq!(word(1, 1).as_deref(), Some("中文"));
    assert_eq!(word(2, 1).as_deref(), Some("中文"));
}

#[test]
fn a_triple_click_selects_the_rows_of_a_line() {
    let mut terminal = Terminal::new(GridSize::new(10, 4));
    // The first line does not fit in a row.
    terminal.feed(b"0123456789abc\r\nnext");
    terminal.select(SelectionKind::Lines, GridPoint::new(1, 1), Side::Left);
    // Without an LF at the end: pasted into a shell, it does not run.
    assert_eq!(terminal.selection_text().as_deref(), Some("0123456789abc"));
    let screen = copy(&mut terminal);
    assert_eq!(marks(&screen, 0), "##########");
    assert_eq!(marks(&screen, 1), "##########");
    assert_eq!(marks(&screen, 2), "..........");

    // Dragging to the next line adds it.
    terminal.select_to(GridPoint::new(0, 2), Side::Left);
    assert_eq!(
        terminal.selection_text().as_deref(),
        Some("0123456789abc\nnext")
    );
}

#[test]
fn a_wide_character_is_selected_whole() {
    let mut terminal = Terminal::new(GridSize::new(10, 2));
    terminal.feed("a中b".as_bytes());
    // From its right half.
    terminal.select(SelectionKind::Cells, GridPoint::new(2, 0), Side::Left);
    terminal.select_to(GridPoint::new(3, 0), Side::Right);
    assert_eq!(terminal.selection_text().as_deref(), Some("中b"));
    assert_eq!(marks(&copy(&mut terminal), 0), ".###......");
    // To its left half.
    terminal.select(SelectionKind::Cells, GridPoint::new(0, 0), Side::Left);
    terminal.select_to(GridPoint::new(1, 0), Side::Right);
    assert_eq!(terminal.selection_text().as_deref(), Some("a中"));
    assert_eq!(marks(&copy(&mut terminal), 0), "###.......");
}

#[test]
fn the_selection_moves_with_the_text_and_goes_away_with_it() {
    let mut terminal = Terminal::new(GridSize::new(10, 3));
    terminal.feed(b"ab\r\ncd\r\nef");
    let mut screen = Screen::new();
    terminal.select(SelectionKind::Cells, GridPoint::new(0, 1), Side::Left);
    terminal.select_to(GridPoint::new(0, 1), Side::Right);
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 1), "#.........");
    screen.clear_changes();

    // The screen scrolls up: the selection goes up with its text.
    terminal.feed(b"\r\ngh");
    assert_eq!(terminal.selection_text().as_deref(), Some("c"));
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 0), "#.........");
    assert_eq!(marks(&screen, 1), "..........");
    screen.clear_changes();

    // From the first row to the second one, and the program clears the
    // first one: the selection goes away, also from the second row, which
    // did not change.
    terminal.select(SelectionKind::Cells, GridPoint::new(0, 0), Side::Left);
    terminal.select_to(GridPoint::new(1, 1), Side::Right);
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 0), "##########");
    assert_eq!(marks(&screen, 1), "##........");
    screen.clear_changes();
    terminal.feed(b"\x1b[1;1H\x1b[K");
    assert!(!terminal.has_selection());
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 0), "..........");
    assert_eq!(marks(&screen, 1), "..........");
    assert_eq!(screen.row_text(1), "ef");

    // Writing over it does not remove it, but clearing the screen does.
    terminal.select(SelectionKind::Lines, GridPoint::new(0, 2), Side::Left);
    terminal.feed(b"\x1b[3;1Hxy");
    assert_eq!(terminal.selection_text().as_deref(), Some("xy"));
    terminal.feed(b"\x1b[2J");
    assert!(!terminal.has_selection());
    terminal.copy_damage(&mut screen);
    assert_eq!(marks(&screen, 2), "..........");
}

/// Pieces of output that change the grid in many ways.
const PIECES: &[&str] = &[
    "ab",
    "xyz",
    "中",
    "😀",
    "e\u{301}",
    "\r",
    "\n",
    "\r\n",
    "\t",
    "\x08",
    "\x1b[H",
    "\x1b[2;3H",
    "\x1b[A",
    "\x1b[B",
    "\x1b[3C",
    "\x1b[D",
    "\x1b[5G",
    "\x1b[d",
    "\x1b[K",
    "\x1b[1K",
    "\x1b[2K",
    "\x1b[J",
    "\x1b[1J",
    "\x1b[2J",
    "\x1b[3J",
    "\x1b[@",
    "\x1b[P",
    "\x1b[2P",
    "\x1b[L",
    "\x1b[M",
    "\x1b[3X",
    "\x1b[S",
    "\x1b[2T",
    "\x1b[2;3r",
    "\x1b[r",
    "\x1bM",
    "\x1bD",
    "\x1bE",
    "\x1b[?7l",
    "\x1b[?7h",
    "\x1b[4h",
    "\x1b[4l",
    "\x1b[?1049h",
    "\x1b[?1049l",
    "\x1b[?47h",
    "\x1b[?47l",
    "\x1b[?25l",
    "\x1b[?25h",
    "\x1b[?6h",
    "\x1b[?6l",
    "\x1b[1;31m",
    "\x1b[0m",
    "\x1b[7m",
    "\x1b[48;5;200m",
    "\x1b[38;2;1;2;3m",
    "\x1b7",
    "\x1b8",
    "\x1b[s",
    "\x1b[u",
    "\x1bc",
    "\x1b#8",
    "\x1bH",
    "\x1b[g",
    "\x1b[3g",
    "\x1b[3b",
    "\x1b[?3h",
    "\x1b[20h",
    "\x1b[20l",
    "\x1b[6 q",
    "\x1b]0;t\x07",
];

#[test]
fn copying_only_the_damage_gives_the_same_screen_as_copying_everything() {
    const SIZES: [GridSize; 7] = [
        GridSize::new(2, 1),
        GridSize::new(2, 2),
        GridSize::new(3, 1),
        GridSize::new(3, 3),
        GridSize::new(5, 4),
        GridSize::new(10, 6),
        GridSize::new(80, 24),
    ];
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut random = move |below: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % below as u64) as usize
    };
    for case in 0..300 {
        let size = SIZES[random(SIZES.len())];
        let mut damaged = Terminal::new(size);
        let mut whole = Terminal::new(size);
        let mut screen = Screen::new();
        let mut output = Vec::new();
        for _ in 0..random(80) {
            output.extend_from_slice(PIECES[random(PIECES.len())].as_bytes());
        }
        let mut rest = &output[..];
        while !rest.is_empty() {
            let (piece, after) = rest.split_at(1 + random(rest.len().min(16)));
            rest = after;
            if random(16) == 0 {
                let size = SIZES[random(SIZES.len())];
                damaged.resize(size);
                whole.resize(size);
            }
            // The selection changes, sometimes with points outside the grid.
            let size = damaged.size();
            let point = GridPoint::new(
                random(usize::from(size.columns) + 2) as u16,
                random(usize::from(size.rows) + 2) as u16,
            );
            let side = if random(2) == 0 {
                Side::Left
            } else {
                Side::Right
            };
            match random(8) {
                0 => {
                    let kind = [
                        SelectionKind::Cells,
                        SelectionKind::Words,
                        SelectionKind::Lines,
                    ][random(3)];
                    damaged.select(kind, point, side);
                    whole.select(kind, point, side);
                }
                1 | 2 => {
                    damaged.select_to(point, side);
                    whole.select_to(point, side);
                }
                3 => {
                    damaged.clear_selection();
                    whole.clear_selection();
                }
                4 => {
                    let scroll = [
                        Scroll::Rows(random(5) as i32 - 2),
                        Scroll::PageUp,
                        Scroll::PageDown,
                        Scroll::Top,
                        Scroll::Bottom,
                    ][random(5)];
                    damaged.scroll(scroll);
                    whole.scroll(scroll);
                }
                _ => {}
            }
            damaged.feed(piece);
            whole.feed(piece);
            damaged.copy_damage(&mut screen);
            let expected = copy(&mut whole);
            let context = || {
                format!(
                    "case {case}, {}×{}, after {} of {:?}",
                    screen.size().columns,
                    screen.size().rows,
                    output.len() - rest.len(),
                    String::from_utf8_lossy(&output),
                )
            };
            for row in 0..expected.size().rows {
                assert_eq!(
                    screen.row(row),
                    expected.row(row),
                    "row {row}, {}",
                    context()
                );
            }
            assert_eq!(screen.cursor(), expected.cursor(), "{}", context());
            screen.clear_changes();
        }
    }
}
