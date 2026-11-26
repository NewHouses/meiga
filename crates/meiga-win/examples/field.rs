//! Tries the text field of meiga by hand, in the console: the field of
//! `meiga_core::ui::field`, which the names of the tabs will use, drawn as a
//! line of text, with the keys that the console gives, which are the ones
//! that meiga gets, and with the clipboard of Windows.
//!
//! ```text
//! cargo run -p meiga-win --example field -- [--columns N] [--limit N] [text]
//! ```
//!
//! The field shows N cells, 20 by default, or fewer if the console is
//! narrower, and holds at most N UTF-16 code units, 1000 by default. With
//! `text`, it starts with it, all selected. Enter shows its text, and starts
//! again with an empty field; Esc ends. Under the field, a line says how many
//! code units it has, and what the last key did.
//!
//! The terminal where it runs takes some keys of the field for itself:
//! Windows Terminal pastes with Ctrl+Shift+V and Shift+Insert, and often
//! with Ctrl+V, and copies with Ctrl+C, Ctrl+Shift+C and Ctrl+Insert while
//! it has a selection of its own; meiga pastes with Ctrl+V and
//! Ctrl+Shift+V, and copies with Ctrl+C while it has a selection. They type
//! what they paste, as keys. So F5 also asks the field to paste the
//! clipboard. The field shows the characters without width, such as a
//! combining accent, as meiga draws them until subphase 8.1: without a cell.

use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::iter;
use std::ops::RangeInclusive;
use std::process::ExitCode;
use std::ptr;

use meiga_core::input::{ControlKeys, KeyEvent};
use meiga_core::text::{self, CellChar};
use meiga_core::ui::field::{Field, Reply, Shown};
use meiga_win::clipboard;
use meiga_win::window::{Window, WindowOptions};
use windows_sys::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::Console::{
    CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT,
    ENABLE_PROCESSED_INPUT, ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    GetConsoleMode, GetConsoleScreenBufferInfo, GetStdHandle, INPUT_RECORD, KEY_EVENT,
    KEY_EVENT_RECORD, ReadConsoleInputW, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetConsoleMode,
};
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, MsgWaitForMultipleObjects, PM_REMOVE, PeekMessageW, QS_ALLINPUT,
};

const USAGE: &str =
    "Usage: cargo run -p meiga-win --example field -- [--columns N] [--limit N] [text]";

/// The exit code when the command line is wrong, as in meiga.
const USAGE_ERROR: u8 = 2;

/// The keys that the example takes for itself.
const VK_RETURN: u16 = 0x0d;
const VK_ESCAPE: u16 = 0x1b;
const VK_F5: u16 = 0x74;

/// The cells that the copied text takes in the line under the field.
const PREVIEW: usize = 24;

fn main() -> ExitCode {
    let options = match Options::parse(env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}\n\n{USAGE}");
            return ExitCode::from(USAGE_ERROR);
        }
    };
    match run(&options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// What the command line asks for.
struct Options {
    columns: usize,
    limit: usize,
    text: Option<String>,
}

impl Options {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut options = Self {
            columns: 20,
            limit: 1000,
            text: None,
        };
        let mut words = Vec::new();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--columns" => options.columns = number(args.next(), "--columns", 1..=500)?,
                "--limit" => options.limit = number(args.next(), "--limit", 1..=100_000)?,
                _ => words.push(arg),
            }
        }
        if !words.is_empty() {
            options.text = Some(words.join(" "));
        }
        Ok(options)
    }
}

/// The number of an option, which has to be in `range`.
fn number(
    value: Option<String>,
    option: &str,
    range: RangeInclusive<usize>,
) -> Result<usize, String> {
    value
        .and_then(|value| value.parse().ok())
        .filter(|number| range.contains(number))
        .ok_or_else(|| {
            format!(
                "{option} needs a number from {} to {}.",
                range.start(),
                range.end()
            )
        })
}

fn run(options: &Options) -> Result<(), Box<dyn Error>> {
    // The clipboard needs a window of this thread: this one, never shown.
    let window = Window::create(&WindowOptions {
        title: "meiga: the text field",
        title_bar: false,
    })?;
    let console = Console::open()?;
    let columns = options
        .columns
        .min(console.width().saturating_sub(3).max(1));
    let mut field = Field::new(options.limit);
    field.set_columns(columns);
    if let Some(text) = &options.text {
        field.set_text(text);
        field.select_all();
    }
    let mut out = io::stdout().lock();
    writeln!(
        out,
        "A field of {columns} cells and at most {} UTF-16 code units. Enter shows its text, Esc \
         ends, and F5 pastes.",
        options.limit
    )?;
    let mut status = "Type.".to_owned();
    let mut high = None;
    loop {
        draw(&mut out, &field, columns, &status, console.width())?;
        let event = console.next_key()?;
        let reply = if event.down && event.key == VK_F5 {
            Reply::Paste
        } else {
            field.key(&event)
        };
        let name = key_name(&event, &mut high);
        status = match reply {
            Reply::Copy(text) => match clipboard::write_text(&window, &text) {
                Ok(()) => format!("{name}: «{}» went to the clipboard.", preview(&text)),
                Err(error) => format!("{name}: the clipboard is busy: {error}"),
            },
            Reply::Paste => match clipboard::read_text(&window) {
                Ok(Some(text)) => {
                    field.paste(&text);
                    format!("{name}: «{}» came from the clipboard.", preview(&text))
                }
                Ok(None) => format!("{name}: the clipboard has no text."),
                Err(error) => format!("{name}: the clipboard is busy: {error}"),
            },
            Reply::Taken => format!("{name}: the field took it."),
            // A key that goes up and does nothing: the line stays.
            Reply::Ignored if !event.down => status,
            Reply::Ignored => match event.key {
                VK_RETURN => {
                    write!(out, "\r\x1b[2KIts text: «{}»\r\n", field.text())?;
                    field.set_text("");
                    format!("{name}: the field starts again.")
                }
                VK_ESCAPE => {
                    write!(out, "\r\n\r\n")?;
                    return Ok(());
                }
                _ => format!("{name}: not a key of the field."),
            },
        };
    }
}

/// Draws the field, between brackets and with the selection in reverse
/// video, and the line under it, over the ones drawn before; and puts the
/// cursor of the console where the caret is, which the next drawing
/// starts from.
fn draw(
    out: &mut impl Write,
    field: &Field,
    columns: usize,
    status: &str,
    width: usize,
) -> io::Result<()> {
    let mut cells = Vec::new();
    let Shown { caret, selection } = field.show(&mut cells);
    let mut line = String::from("[");
    for (index, &cell) in cells.iter().enumerate() {
        if index == selection.start && !selection.is_empty() {
            line.push_str("\x1b[7m");
        }
        push_cell(&mut line, cell);
        if index + 1 == selection.end && !selection.is_empty() {
            line.push_str("\x1b[27m");
        }
    }
    line.extend(iter::repeat_n(' ', columns.saturating_sub(cells.len())));
    line.push(']');
    let units: usize = field.text().chars().map(char::len_utf16).sum();
    let under = fitted(
        &format!("{units} of {} UTF-16 code units · {status}", field.limit()),
        width.saturating_sub(1),
    );
    write!(
        out,
        "\r\x1b[2K{line}\r\n\x1b[2K{under}\x1b[1A\x1b[{}G",
        caret + 2
    )?;
    out.flush()
}

/// Adds what `cell` shows to `line`: the right half of a wide character is
/// already there.
fn push_cell(line: &mut String, cell: CellChar) {
    match cell {
        CellChar::Blank => line.push(' '),
        CellChar::Narrow(c) | CellChar::WideLeft(c) => line.push(c),
        CellChar::WideRight(_) => {}
    }
}

/// `text`, cut to `columns` cells with an ellipsis.
fn fitted(text: &str, columns: usize) -> String {
    let mut cells = Vec::new();
    text::fit(text, columns, &mut cells);
    let mut out = String::new();
    for cell in cells {
        push_cell(&mut out, cell);
    }
    out
}

/// The start of `text`, for the line under the field.
fn preview(text: &str) -> String {
    fitted(text, PREVIEW)
}

/// The name of the key of `event`, with its modifier keys, or the character
/// that it writes. A character outside the Basic Multilingual Plane comes
/// in two keys: `high` keeps its first half until the second one.
fn key_name(event: &KeyEvent, high: &mut Option<u16>) -> String {
    let controls = event.controls;
    let writes = !(controls.ctrl() || controls.alt()) || controls.alt_gr();
    match event.character {
        unit @ 0xd800..=0xdbff if writes => {
            *high = Some(unit);
            return "The first half of a character".to_owned();
        }
        unit @ 0xdc00..=0xdfff if writes => {
            let pair = high
                .take()
                .map(|high| char::decode_utf16([high, unit]).next());
            return match pair {
                Some(Some(Ok(c))) => format!("«{c}»"),
                _ => "The second half of a character".to_owned(),
            };
        }
        unit if writes => {
            if let Some(c) =
                char::from_u32(u32::from(unit)).filter(|c| *c != ' ' && !c.is_control())
            {
                return format!("«{c}»");
            }
        }
        _ => {}
    }
    let mut name = String::new();
    let modifier = matches!(event.key, 0x10..=0x12);
    if !modifier {
        if controls.alt_gr() {
            name.push_str("AltGr+");
        } else {
            if controls.ctrl() {
                name.push_str("Ctrl+");
            }
            if controls.alt() {
                name.push_str("Alt+");
            }
        }
        if controls.shift() {
            name.push_str("Shift+");
        }
    }
    let key = match event.key {
        0x08 => "Backspace".to_owned(),
        0x09 => "Tab".to_owned(),
        0x0d => "Enter".to_owned(),
        0x10 => "Shift".to_owned(),
        0x11 => "Ctrl".to_owned(),
        0x12 => "Alt".to_owned(),
        0x1b => "Esc".to_owned(),
        0x20 => "Space".to_owned(),
        0x21 => "Page Up".to_owned(),
        0x22 => "Page Down".to_owned(),
        0x23 => "End".to_owned(),
        0x24 => "Home".to_owned(),
        0x25 => "←".to_owned(),
        0x26 => "↑".to_owned(),
        0x27 => "→".to_owned(),
        0x28 => "↓".to_owned(),
        0x2d => "Insert".to_owned(),
        0x2e => "Delete".to_owned(),
        key @ 0x70..=0x87 => format!("F{}", key - 0x6f),
        key @ (0x30..=0x39 | 0x41..=0x5a) => char::from(key as u8).to_string(),
        key => format!("the key {key:#04x}"),
    };
    name + &key
}

/// The console of the example: its input gives whole key events, and its
/// output understands VT sequences, until it is dropped, which gives them
/// back the modes that they had.
struct Console {
    input: HANDLE,
    output: HANDLE,
    modes: Option<(CONSOLE_MODE, CONSOLE_MODE)>,
}

impl Console {
    fn open() -> io::Result<Self> {
        // SAFETY: GetStdHandle has no preconditions.
        let (input, output) = unsafe {
            (
                GetStdHandle(STD_INPUT_HANDLE),
                GetStdHandle(STD_OUTPUT_HANDLE),
            )
        };
        let mut console = Self {
            input,
            output,
            modes: None,
        };
        let (mut input_mode, mut output_mode) = (0, 0);
        // SAFETY: GetConsoleMode fails for a handle that is not of a
        // console, and the modes are valid to write.
        let console_handles = unsafe {
            GetConsoleMode(input, &mut input_mode) != 0
                && GetConsoleMode(output, &mut output_mode) != 0
        };
        if !console_handles {
            return Err(io::Error::other(
                "The example needs a console: run it in one, without redirecting its input or its \
                 output.",
            ));
        }
        console.modes = Some((input_mode, output_mode));
        let keys = input_mode
            & !(ENABLE_PROCESSED_INPUT
                | ENABLE_LINE_INPUT
                | ENABLE_ECHO_INPUT
                | ENABLE_VIRTUAL_TERMINAL_INPUT);
        // SAFETY: both handles are of the console, as GetConsoleMode said.
        let set = unsafe {
            SetConsoleMode(input, keys) != 0
                && SetConsoleMode(output, output_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
        };
        if !set {
            return Err(io::Error::last_os_error());
        }
        Ok(console)
    }

    /// The number of columns of the window of the console, or 80 if it does
    /// not say.
    fn width(&self) -> usize {
        // SAFETY: the zeroed structure is valid: it only has integers.
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        // SAFETY: the handle is the output of the console, and `info` is
        // valid to write.
        if unsafe { GetConsoleScreenBufferInfo(self.output, &mut info) } == 0 {
            return 80;
        }
        let window = info.srWindow;
        usize::try_from(i32::from(window.Right) - i32::from(window.Left) + 1).unwrap_or(80)
    }

    /// The next key that goes down or up, as meiga gets it. Meanwhile, the
    /// window of the thread gets its messages: after a copy, it owns the
    /// clipboard, and another program that takes it waits for the window
    /// to get a message.
    fn next_key(&self) -> io::Result<KeyEvent> {
        loop {
            // SAFETY: the handle is the input of the console, which stays
            // while the example runs.
            let woken =
                unsafe { MsgWaitForMultipleObjects(1, &self.input, 0, INFINITE, QS_ALLINPUT) };
            if woken == WAIT_OBJECT_0 {
                let mut record = INPUT_RECORD::default();
                let mut read = 0;
                // SAFETY: the record has room for one event, and `read` is
                // valid to write.
                if unsafe { ReadConsoleInputW(self.input, &mut record, 1, &mut read) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                if read == 1 && u32::from(record.EventType) == KEY_EVENT {
                    // SAFETY: the record is of a key, so its union holds a
                    // KEY_EVENT_RECORD.
                    return Ok(key_event(unsafe { &record.Event.KeyEvent }));
                }
            } else if woken == WAIT_OBJECT_0 + 1 {
                let mut message = MSG::default();
                // SAFETY: `message` is valid for PeekMessageW to fill in.
                while unsafe { PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
                    // SAFETY: `message` holds the message that PeekMessageW
                    // took.
                    unsafe { DispatchMessageW(&message) };
                }
            } else {
                return Err(io::Error::last_os_error());
            }
        }
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        if let Some((input, output)) = self.modes {
            // SAFETY: both handles are of the console, as GetConsoleMode
            // said when it gave these modes.
            unsafe {
                SetConsoleMode(self.input, input);
                SetConsoleMode(self.output, output);
            }
        }
    }
}

/// A key event of the console, as meiga gets it: the console gives the
/// same members, and the same bits of the modifier keys.
fn key_event(record: &KEY_EVENT_RECORD) -> KeyEvent {
    let controls = [
        ControlKeys::RIGHT_ALT,
        ControlKeys::LEFT_ALT,
        ControlKeys::RIGHT_CTRL,
        ControlKeys::LEFT_CTRL,
        ControlKeys::SHIFT,
        ControlKeys::NUM_LOCK,
        ControlKeys::SCROLL_LOCK,
        ControlKeys::CAPS_LOCK,
        ControlKeys::ENHANCED,
    ]
    .into_iter()
    .filter(|keys| record.dwControlKeyState & u32::from(keys.bits()) != 0)
    .fold(ControlKeys::empty(), |all, keys| all | keys);
    KeyEvent {
        key: record.wVirtualKeyCode,
        scan: record.wVirtualScanCode,
        // SAFETY: ReadConsoleInputW gives the character in UTF-16; any
        // value is a code unit.
        character: unsafe { record.uChar.UnicodeChar },
        down: record.bKeyDown != 0,
        controls,
        repeat: record.wRepeatCount,
    }
}
