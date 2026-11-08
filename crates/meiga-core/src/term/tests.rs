use super::*;

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
    let cursor = Cursor {
        row: 1,
        column: 5,
        shape: CursorShape::Block,
        blinking: false,
    };
    assert_eq!(screen.cursor(), cursor);
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
    // The size in pixels waits for the renderer of the terminal (1.6).
    assert_eq!(terminal.feed(b"\x1b[14t"), Changes::default());
    assert_eq!(responses(&mut terminal), b"");
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
    assert_eq!(cursor(&mut terminal, b""), (0, CursorShape::Block, false));
    // DECSCUSR.
    assert_eq!(
        cursor(&mut terminal, b"\x1b[6 q"),
        (0, CursorShape::Beam, false)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[5 q"),
        (0, CursorShape::Beam, true)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[4 q"),
        (0, CursorShape::Underline, false)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[0 q"),
        (0, CursorShape::Block, false)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[?12h"),
        (0, CursorShape::Block, true)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[?25l"),
        (0, CursorShape::Hidden, true)
    );
    assert_eq!(
        cursor(&mut terminal, b"\x1b[?25h"),
        (0, CursorShape::Block, true)
    );
    // On the right half of a wide character, it is drawn over all of it.
    let on_wide = cursor(&mut terminal, "a中\x1b[3G".as_bytes());
    assert_eq!(on_wide, (1, CursorShape::Block, true));
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
        (0, 0, CursorShape::Block)
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
