//! Gives a recording to a terminal of meiga as fast as it can, and shows how
//! fast the terminal read it: what a tab does with the output of its
//! program, without the pseudoconsole and without drawing.
//!
//! ```text
//! cargo run --release -p meiga-core --example replay -- <recording> [--rounds <n>] [--screen]
//! ```
//!
//! The recordings are made with the conpty example of meiga-win and its
//! option `--record`. Each round gives every block, in order and as it came,
//! to a new terminal of the size of the recording, and measures the time, in
//! MB of output per second (1 MB = 1 000 000 bytes). There are 5 rounds
//! unless `--rounds` says otherwise. At the end, it shows the median.
//!
//! A synchronized update that the program does not end is shown when its
//! time passes, counted with the moments of the recording, as meiga will do
//! with a timer.
//!
//! With `--screen`, it also shows what is on the screen at the end, and the
//! answers that meiga would send to the program, numbering rows and columns
//! from 1, as the control sequences do.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use meiga_core::term::recording::Recording;
use meiga_core::term::{Screen, Terminal};

const USAGE: &str = "Usage: replay <recording> [--rounds <n>] [--screen]";

struct Options {
    path: PathBuf,
    rounds: usize,
    screen: bool,
}

fn options() -> Result<Options, String> {
    let mut path = None;
    let mut rounds = 5;
    let mut screen = false;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--rounds" => {
                rounds = arguments
                    .next()
                    .and_then(|rounds| rounds.parse().ok())
                    .filter(|&rounds| rounds > 0)
                    .ok_or("--rounds needs a number greater than 0")?;
            }
            "--screen" => screen = true,
            _ if argument.starts_with("--") => return Err(format!("Unknown option: {argument}")),
            _ if path.is_none() => path = Some(PathBuf::from(argument)),
            _ => return Err(format!("Only one recording: {argument}")),
        }
    }
    Ok(Options {
        path: path.ok_or("Which recording?")?,
        rounds,
        screen,
    })
}

/// Gives every block of `recording` to `terminal`, and its answers to
/// `answers`.
fn replay(recording: &Recording, terminal: &mut Terminal, answers: &mut Vec<u8>) {
    // When the synchronized update that is waiting ends, in the time of the
    // recording.
    let mut sync_end = None;
    for block in recording.blocks() {
        if sync_end.is_some_and(|end| block.time >= end) {
            if terminal.end_sync().responses {
                terminal.take_responses(answers);
            }
            sync_end = None;
        }
        if terminal.feed(block.bytes).responses {
            terminal.take_responses(answers);
        }
        sync_end = match terminal.sync_deadline() {
            Some(deadline) => sync_end
                .or_else(|| Some(block.time + deadline.saturating_duration_since(Instant::now()))),
            None => None,
        };
    }
}

fn main() -> ExitCode {
    let options = match options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let recording = match Recording::read(&options.path) {
        Ok(recording) => recording,
        Err(error) => {
            eprintln!("Could not read {}: {error}", options.path.display());
            return ExitCode::from(1);
        }
    };
    let size = recording.size();
    let megabytes = recording.output_len() as f64 / 1e6;
    println!(
        "{}: {} × {}, {} blocks, {} bytes of output in {:.1} s.",
        options.path.display(),
        size.columns,
        size.rows,
        recording.blocks().len(),
        recording.output_len(),
        recording.duration().as_secs_f64(),
    );
    let mut speeds = Vec::with_capacity(options.rounds);
    let mut last = None;
    for round in 1..=options.rounds {
        let mut terminal = Terminal::new(size);
        let mut answers = Vec::new();
        let start = Instant::now();
        replay(&recording, &mut terminal, &mut answers);
        let time = start.elapsed().max(Duration::from_nanos(1));
        let speed = megabytes / time.as_secs_f64();
        println!(
            "Round {round}: {:.4} s, {speed:.1} MB/s.",
            time.as_secs_f64()
        );
        speeds.push(speed);
        last = Some((terminal, answers));
    }
    speeds.sort_by(f64::total_cmp);
    println!(
        "Median: {:.1} MB/s (from {:.1} to {:.1}).",
        speeds[speeds.len() / 2],
        speeds[0],
        speeds[speeds.len() - 1]
    );
    if options.screen
        && let Some((mut terminal, mut answers)) = last
    {
        show_screen(&mut terminal, &mut answers);
    }
    ExitCode::SUCCESS
}

/// Shows the rows with text, the cursor, the title and the modes of
/// `terminal`, and `answers`.
fn show_screen(terminal: &mut Terminal, answers: &mut Vec<u8>) {
    // A synchronized update that did not end is shown, as when its time
    // passes.
    if terminal.sync_deadline().is_some() {
        terminal.end_sync();
        terminal.take_responses(answers);
    }
    let mut screen = Screen::new();
    terminal.copy_damage(&mut screen);
    let size = screen.size();
    println!(
        "Screen of {} × {}, the rows with text:",
        size.columns, size.rows
    );
    for row in 0..size.rows {
        let text = screen.row_text(row);
        if !text.is_empty() {
            println!("{:>4} │{text}", row + 1);
        }
    }
    let cursor = screen.cursor();
    let blinking = if cursor.blinking { ", blinking" } else { "" };
    println!(
        "Cursor: row {}, column {}, {:?}{blinking}.",
        cursor.row + 1,
        cursor.column + 1,
        cursor.shape
    );
    println!("Title: {}", terminal.title().unwrap_or("(none)"));
    let modes = terminal.modes();
    let on: Vec<&str> = [
        ("win32_input", modes.win32_input),
        ("focus_events", modes.focus_events),
        ("bracketed_paste", modes.bracketed_paste),
        ("app_cursor", modes.app_cursor),
        ("app_keypad", modes.app_keypad),
        ("mouse_clicks", modes.mouse_clicks),
        ("mouse_drag", modes.mouse_drag),
        ("mouse_motion", modes.mouse_motion),
        ("sgr_mouse", modes.sgr_mouse),
        ("utf8_mouse", modes.utf8_mouse),
        ("alternate_scroll", modes.alternate_scroll),
        ("alt_screen", modes.alt_screen),
        ("show_cursor", modes.show_cursor),
        ("line_feed_new_line", modes.line_feed_new_line),
    ]
    .into_iter()
    .filter_map(|(name, on)| on.then_some(name))
    .collect();
    println!("Modes: {}", on.join(", "));
    if answers.is_empty() {
        println!("Answers: (none)");
    } else {
        println!("Answers: {}", visible(answers));
    }
}

/// The bytes as text, with each control character as its symbol in Unicode
/// (ESC as ␛, CR as ␍, LF as ␊…), and the bytes that are not UTF-8 as \xNN.
fn visible(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len());
    for chunk in bytes.utf8_chunks() {
        for c in chunk.valid().chars() {
            text.push(match c {
                '\0'..='\x1f' => char::from_u32(0x2400 + u32::from(c)).unwrap_or(c),
                '\x7f' => '␡',
                c => c,
            });
        }
        for byte in chunk.invalid() {
            let _ = write!(text, "\\x{byte:02X}");
        }
    }
    text
}
