//! The terminal with real output: recordings of the ConPTY that meiga ships,
//! made with the conpty example of meiga-win and its option `--record`.
//!
//! - `cmd-echo.mrec`: `cmd /c echo ola`.
//! - `spinner.mrec`: `tools\perf\scenarios\spinner.ps1`, for 3 s.
//! - `ai-stream.mrec`: `tools\perf\scenarios\ai-stream.ps1`, for 12 s.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use meiga_core::geometry::GridSize;
use meiga_core::term::recording::Recording;
use meiga_core::term::{
    Cell, CellFlags, Color, Columns, Cursor, CursorShape, Modes, Screen, Terminal,
};
use meiga_core::text::CellChar;

const CMD_ECHO: &[u8] = include_bytes!("data/cmd-echo.mrec");
const SPINNER: &[u8] = include_bytes!("data/spinner.mrec");
const AI_STREAM: &[u8] = include_bytes!("data/ai-stream.mrec");

/// The size of the pseudoconsole of the conpty example.
const SIZE: GridSize = GridSize::new(120, 30);

fn read(data: &[u8]) -> Recording {
    let recording = Recording::from_bytes(data.to_vec()).unwrap();
    assert_eq!(recording.size(), SIZE);
    recording
}

/// What a terminal shows, and what it answered.
#[derive(Debug, PartialEq)]
struct Shown {
    text: Vec<String>,
    cells: Vec<Vec<Cell>>,
    cursor: Cursor,
    title: Option<String>,
    modes: Modes,
    answers: Vec<u8>,
}

/// Gives `pieces` to a new terminal, one after another.
fn feed<'a>(pieces: impl IntoIterator<Item = &'a [u8]>) -> Shown {
    let mut terminal = Terminal::new(SIZE);
    let mut answers = Vec::new();
    for piece in pieces {
        terminal.feed(piece);
        terminal.take_responses(&mut answers);
    }
    let mut screen = Screen::new();
    terminal.copy_damage(&mut screen);
    let rows = 0..SIZE.rows;
    Shown {
        text: rows.clone().map(|row| screen.row_text(row)).collect(),
        cells: rows.map(|row| screen.row(row).to_vec()).collect(),
        cursor: screen.cursor(),
        title: terminal.title().map(str::to_owned),
        modes: terminal.modes(),
        answers,
    }
}

fn blocks(recording: &Recording) -> impl Iterator<Item = &[u8]> {
    recording.blocks().map(|block| block.bytes)
}

/// The modes after the program ends: ConPTY turns off the ones that it turns
/// on when it starts, win32-input-mode and the focus events.
const MODES_AT_THE_END: Modes = Modes {
    win32_input: false,
    focus_events: false,
    bracketed_paste: false,
    app_cursor: false,
    app_keypad: false,
    mouse_clicks: false,
    mouse_drag: false,
    mouse_motion: false,
    sgr_mouse: false,
    utf8_mouse: false,
    alternate_scroll: true,
    alt_screen: false,
    show_cursor: true,
    line_feed_new_line: false,
};

#[test]
fn cmd_writes_ola() {
    let recording = read(CMD_ECHO);
    let mut terminal = Terminal::new(SIZE);
    let mut win32_input = Vec::new();
    for block in blocks(&recording) {
        terminal.feed(block);
        win32_input.push(terminal.modes().win32_input);
    }
    // While the program runs, ConPTY wants whole key events.
    assert!(win32_input.contains(&true));

    let shown = feed(blocks(&recording));
    assert_eq!(shown.text[0], "ola");
    assert!(shown.text[1..].iter().all(String::is_empty));
    assert_eq!((shown.cursor.row, shown.cursor.column), (1, 0));
    assert_eq!(shown.title, None);
    assert_eq!(shown.modes, MODES_AT_THE_END);
    // ConPTY asks for the device attributes when it starts.
    assert_eq!(shown.answers, b"\x1b[?6c");
}

#[test]
fn the_spinner_only_changes_what_it_draws_again() {
    let recording = read(SPINNER);
    let mut terminal = Terminal::new(SIZE);
    let mut screen = Screen::new();
    let mut frames = 0;
    for block in blocks(&recording) {
        terminal.feed(block);
        terminal.copy_damage(&mut screen);
        if block.starts_with(b"\r\x1b[38;5;") {
            frames += 1;
            let changes: Vec<(u16, Columns)> = screen.changes().collect();
            // All the line the first time and then, with the cursor hidden,
            // only the glyph, and the seconds when they change.
            let expected = if frames == 1 {
                changes == [(0, Columns::new(0, 16))]
            } else {
                changes == [(0, Columns::new(0, 1))] || changes == [(0, Columns::new(0, 13))]
            };
            assert!(expected, "frame {frames}: {changes:?}");
        }
        screen.clear_changes();
    }
    assert!(frames > 25, "{frames} frames");

    let shown = feed(blocks(&recording));
    let row = &shown.cells[0];
    assert!(matches!(
        row[0].char,
        CellChar::Narrow('\u{2800}'..='\u{28ff}')
    ));
    assert!(matches!(row[0].fg, Color::Indexed(16..=231)));
    assert!(shown.text[0].ends_with(" Thinking (3 s)"));
    assert!(row[2..10].iter().all(|cell| cell.flags == CellFlags::BOLD));
    assert!(row[11..16].iter().all(|cell| cell.flags == CellFlags::DIM));
    assert!(shown.text[1..].iter().all(String::is_empty));
    // Shown again when the scenario ends, in the next row: a bar that
    // blinks, as the program did not choose another cursor.
    let cursor = Cursor {
        row: 1,
        column: 0,
        shape: CursorShape::Beam,
        blinking: true,
    };
    assert_eq!(shown.cursor, cursor);
    assert_eq!(shown.modes, MODES_AT_THE_END);
}

#[test]
fn the_ai_answer_keeps_its_text_and_colours() {
    let shown = feed(blocks(&read(AI_STREAM)));
    assert_eq!(
        shown.text[0],
        "💡 On a big tree, the first run is slow because the disk is cold. The"
    );
    let bulb = &shown.cells[0][..2];
    assert_eq!(bulb[0].char, CellChar::WideLeft('💡'));
    assert_eq!(bulb[1].char, CellChar::WideRight('💡'));

    assert_eq!(
        shown.text[3],
        "❯ Why does my Rust release build take so long?"
    );
    let arrow = shown.cells[3][0];
    assert_eq!(
        (arrow.fg, arrow.flags),
        (Color::Indexed(6), CellFlags::BOLD)
    );

    assert_eq!(shown.text[5], "Where the time goes");
    let title = shown.cells[5][0];
    assert_eq!(
        (title.fg, title.flags),
        (Color::Indexed(141), CellFlags::BOLD)
    );

    assert_eq!(
        shown.text[23],
        "┌──────────────────────┬──────────────┬──────────────┐"
    );
    assert_eq!((shown.cursor.row, shown.cursor.column), (27, 0));
    assert_eq!(shown.title, None);
    assert_eq!(shown.modes, MODES_AT_THE_END);
    assert_eq!(shown.answers, b"\x1b[?6c");
}

#[test]
fn how_the_output_is_split_does_not_change_what_is_shown() {
    let mut state = 0x853c_49e6_748f_ea9b_u64;
    for data in [CMD_ECHO, SPINNER, AI_STREAM] {
        let recording = read(data);
        let output: Vec<u8> = blocks(&recording).flatten().copied().collect();
        let expected = feed(blocks(&recording));
        assert_eq!(feed([&output[..]]), expected);
        // Every sequence, and every character of more than one byte, split.
        assert_eq!(feed(output.chunks(1)), expected);
        for _ in 0..20 {
            let mut pieces = Vec::new();
            let mut rest = &output[..];
            while !rest.is_empty() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let len = 1 + (state % 300) as usize;
                let (piece, after) = rest.split_at(len.min(rest.len()));
                pieces.push(piece);
                rest = after;
            }
            assert_eq!(feed(pieces), expected);
        }
    }
}

#[test]
fn copying_only_what_changed_gives_the_same_screen() {
    for data in [CMD_ECHO, SPINNER, AI_STREAM] {
        let recording = read(data);
        let mut damaged = Terminal::new(SIZE);
        let mut whole = Terminal::new(SIZE);
        let mut screen = Screen::new();
        for (index, block) in blocks(&recording).enumerate() {
            damaged.feed(block);
            whole.feed(block);
            damaged.copy_damage(&mut screen);
            let mut expected = Screen::new();
            whole.copy_damage(&mut expected);
            for row in 0..SIZE.rows {
                assert_eq!(
                    screen.row(row),
                    expected.row(row),
                    "block {index}, row {row}"
                );
            }
            assert_eq!(screen.cursor(), expected.cursor(), "block {index}");
            screen.clear_changes();
        }
    }
}
