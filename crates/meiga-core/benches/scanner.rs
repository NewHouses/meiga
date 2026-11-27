//! What the scanner costs against the time that the terminal takes to read
//! the same output, which includes it (ADR 0032): it has to stay under 1 %.
//!
//! ```text
//! cargo bench -p meiga-core --bench scanner -- [--rounds <n>] [<recording>...]
//! ```
//!
//! It measures five outputs that it makes, of 4 MiB each, in blocks of
//! 64 KiB, as meiga reads them from ConPTY: text with a colour for each
//! character, text full of `]`, text without sequences, a prompt with the
//! marks of OSC 133 and OSC 9;9 every 20 rows, and rows with links (OSC 8);
//! then the recordings of the tests of meiga-core, and the ones given. Each round reads each output with a new
//! scanner and then with a new terminal, of the size of the output. After a
//! round that is not counted, it shows the median of the rounds, 11 unless
//! `--rounds` says otherwise, and the time of the scanner against the time of
//! the terminal: the median of the rounds, and the lowest and the highest.
//! A relative path of a recording is taken from the folder of the workspace,
//! where cargo is run: cargo bench runs the benchmark in the folder of
//! meiga-core.

use std::fmt::Write as _;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use meiga_core::geometry::GridSize;
use meiga_core::term::recording::Recording;
use meiga_core::term::{Scanner, Terminal};

const USAGE: &str = "Usage: scanner [--rounds <n>] [<recording>...]";

/// The size of each output that the benchmark makes.
const MADE: usize = 4 * 1024 * 1024;

/// The most that meiga reads from ConPTY at once.
const BLOCK: usize = 64 * 1024;

/// The size of the terminal for the outputs that the benchmark makes.
const SIZE: GridSize = GridSize::new(120, 30);

const RECORDINGS: &[(&str, &[u8])] = &[
    (
        "cmd-echo.mrec",
        include_bytes!("../tests/data/cmd-echo.mrec"),
    ),
    ("spinner.mrec", include_bytes!("../tests/data/spinner.mrec")),
    (
        "ai-stream.mrec",
        include_bytes!("../tests/data/ai-stream.mrec"),
    ),
    (
        "shell-marks.mrec",
        include_bytes!("../tests/data/shell-marks.mrec"),
    ),
];

/// An output, in the blocks in which the terminal reads it.
struct Output {
    name: String,
    size: GridSize,
    blocks: Vec<Vec<u8>>,
}

impl Output {
    /// Lines from `line`, numbered from 0, up to [`MADE`] bytes, in blocks
    /// of [`BLOCK`] bytes.
    fn made(name: &str, mut line: impl FnMut(usize) -> String) -> Self {
        let mut bytes = Vec::with_capacity(MADE + 4096);
        let mut number = 0;
        while bytes.len() < MADE {
            bytes.extend_from_slice(line(number).as_bytes());
            number += 1;
        }
        bytes.truncate(MADE);
        Self {
            name: name.to_owned(),
            size: SIZE,
            blocks: bytes.chunks(BLOCK).map(<[u8]>::to_vec).collect(),
        }
    }

    fn recorded(name: &str, recording: &Recording) -> Self {
        Self {
            name: name.to_owned(),
            size: recording.size(),
            blocks: recording
                .blocks()
                .map(|block| block.bytes.to_vec())
                .collect(),
        }
    }

    fn len(&self) -> usize {
        self.blocks.iter().map(Vec::len).sum()
    }
}

/// A log with a colour for each character, as the output with the most
/// sequences: about 5 ESC every 100 bytes.
fn colours(number: usize) -> String {
    let mut line = format!(
        "\x1b[2m2026-01-15 10:00:{:02}.{:03}\x1b[0m \x1b[2mDEBUG\x1b[0m \x1b[38;5;187msession\x1b[0m: ",
        number % 60,
        number % 1000
    );
    for column in 0..40 {
        let red = (number * 7 + column * 5) % 256;
        let green = (column * 11) % 256;
        let blue = (number + column * 3) % 256;
        let _ = write!(line, "\x1b[38;2;{red};{green};{blue}m\u{2588}");
    }
    line.push_str("\x1b[0m\r\n");
    line
}

/// Lists inside lists: a `]` every 4 bytes, and no ESC.
fn brackets(_: usize) -> String {
    "[[1,2],[3,4],[5,6],[7,8]] [9] [[10,11],[12]] [[13],[14,15],[16,17]]\r\n".to_owned()
}

/// Words, without sequences.
fn text(number: usize) -> String {
    const WORDS: [&str; 8] = [
        "meiga", "terminal", "ola", "cartafol", "prompt", "saida", "orde", "tab",
    ];
    let mut line = String::new();
    for word in 0..14 {
        line.push_str(WORDS[(number + word * 3) % WORDS.len()]);
        line.push(' ');
    }
    line.push_str("\r\n");
    line
}

/// A shell with its marks and its folder at each prompt, every 20 rows, far
/// more often than anybody types commands.
fn marks(number: usize) -> String {
    let mut lines = format!(
        "\x1b]133;D;{}\x07\x1b]133;A\x07\x1b]9;9;\"C:\\Windows\\System32\"\x1b\\PS C:\\Windows\\System32> \x1b]133;B\x07dir\r\n\x1b]133;C\x07",
        number % 3
    );
    for file in 0..20 {
        let _ = write!(lines, "f{file}.dll   {}\r\n", number * 20 + file);
    }
    lines
}

/// Files with links, two OSC 8 in each row, as `ls --hyperlink` writes them.
fn links(number: usize) -> String {
    format!(
        "\x1b]8;;file:///home/ana/f{number}.rs\x1b\\f{number}.rs\x1b]8;;\x1b\\   {}\r\n",
        number * 7
    )
}

/// The median of `values`, which are not empty.
fn median<T: Copy + PartialOrd>(values: &mut [T]) -> T {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values[values.len() / 2]
}

/// What a round measures: the time of the scanner, the time of the
/// terminal, and how many things the scanner found.
fn round(output: &Output) -> (Duration, Duration, usize) {
    let mut scanner = Scanner::new();
    let mut found = 0;
    let start = Instant::now();
    for block in &output.blocks {
        scanner.scan(black_box(block), |report| {
            black_box(report);
            found += 1;
        });
    }
    let scan = start.elapsed();
    let mut terminal = Terminal::new(output.size);
    let start = Instant::now();
    for block in &output.blocks {
        black_box(terminal.feed(black_box(block)));
    }
    // A synchronized update that is still open is read too.
    if terminal.sync_deadline().is_some() {
        black_box(terminal.end_sync());
    }
    let feed = start.elapsed();
    (scan, feed, found)
}

/// `path`, from the folder of the workspace if it is relative.
fn from_workspace(path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        return path.to_owned();
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(path)
}

fn milliseconds(time: Duration) -> f64 {
    time.as_secs_f64() * 1e3
}

fn main() -> ExitCode {
    let mut rounds = 11;
    let mut outputs = vec![
        Output::made("colours", colours),
        Output::made("brackets", brackets),
        Output::made("text", text),
        Output::made("marks", marks),
        Output::made("links", links),
    ];
    for (name, data) in RECORDINGS {
        match Recording::from_bytes(data.to_vec()) {
            Ok(recording) => outputs.push(Output::recorded(name, &recording)),
            Err(error) => {
                eprintln!("Could not read {name}: {error}");
                return ExitCode::from(1);
            }
        }
    }
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--rounds" => {
                let Some(n) = arguments
                    .next()
                    .and_then(|n| n.parse().ok())
                    .filter(|&n| n > 0)
                else {
                    eprintln!("--rounds needs a number greater than 0\n{USAGE}");
                    return ExitCode::from(2);
                };
                rounds = n;
            }
            // What cargo bench gives to every benchmark.
            "--bench" => {}
            _ if argument.starts_with("--") => {
                eprintln!("Unknown option: {argument}\n{USAGE}");
                return ExitCode::from(2);
            }
            path => match Recording::read(&from_workspace(path)) {
                Ok(recording) => {
                    let name = Path::new(path).file_name().map_or_else(
                        || path.to_owned(),
                        |name| name.to_string_lossy().into_owned(),
                    );
                    outputs.push(Output::recorded(&name, &recording));
                }
                Err(error) => {
                    eprintln!("Could not read {path}: {error}");
                    return ExitCode::from(1);
                }
            },
        }
    }

    println!(
        "{:<18} {:>11} {:>7} {:>6} {:>12} {:>12}  Scanner/terminal, in % (lowest–highest)",
        "Output", "Bytes", "Blocks", "Found", "Scanner ms", "Terminal ms"
    );
    for output in &outputs {
        // A round that is not counted, to have everything in memory.
        let _ = round(output);
        let mut scans = Vec::with_capacity(rounds);
        let mut feeds = Vec::with_capacity(rounds);
        let mut shares = Vec::with_capacity(rounds);
        let mut found = 0;
        for _ in 0..rounds {
            let (scan, feed, count) = round(output);
            found = count;
            scans.push(scan);
            feeds.push(feed);
            shares.push(100.0 * scan.as_secs_f64() / feed.as_secs_f64().max(1e-9));
        }
        let share = median(&mut shares);
        println!(
            "{:<18} {:>11} {:>7} {:>6} {:>12.3} {:>12.3}  {share:.2} ({:.2}–{:.2})",
            output.name,
            output.len(),
            output.blocks.len(),
            found,
            milliseconds(median(&mut scans)),
            milliseconds(median(&mut feeds)),
            shares[0],
            shares[shares.len() - 1],
        );
    }
    ExitCode::SUCCESS
}
