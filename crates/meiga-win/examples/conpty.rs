//! Runs a program in a pseudoconsole, shows each block of what it writes with
//! the control characters in sight, and sends it each line that you type,
//! followed by Enter.
//!
//! ```text
//! cargo run -p meiga-win --example conpty -- [--system] [--record <file>] [--quiet] [command line]
//! ```
//!
//! It uses the ConPTY that tools\fetch-conpty.ps1 extracts to the target
//! folder or, with `--system`, the one of Windows. Without a command line, it
//! runs `cmd`. The arguments are joined with spaces. It ends when the session
//! ends, with the exit code of the program. Ctrl+Z and Enter close the
//! session.
//!
//! With `--record`, it also writes what the session sends to a recording
//! (`meiga_core::term::recording`), which the replay example of meiga-core
//! gives to a terminal of meiga. With `--quiet`, it does not show the
//! blocks, only the end of the session. The options go before the command
//! line.

use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, BufRead, BufWriter};
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Instant;

use meiga_core::geometry::GridSize;
use meiga_core::term::recording::Recorder;
use meiga_win::conpty::{Conpty, Input, Output, Source};

/// The size of the pseudoconsole: the default of a console window.
const SIZE: GridSize = GridSize::new(120, 30);

const USAGE: &str = "Usage: conpty [--system] [--record <file>] [--quiet] [command line]";

/// Shows what the session sends, records it, and ends the example when the
/// session ends.
struct Printer {
    start: Instant,
    quiet: bool,
    recording: Option<(PathBuf, Recorder<BufWriter<File>>)>,
}

impl Output for Printer {
    fn received(&mut self, bytes: &[u8], _input: &Input) {
        if let Some((path, recorder)) = &mut self.recording
            && let Err(error) = recorder.record(bytes)
        {
            eprintln!("Could not write to {}: {error}", path.display());
            self.recording = None;
        }
        if !self.quiet {
            let ms = self.start.elapsed().as_secs_f64() * 1000.0;
            println!("{ms:>9.1} ms {:>6} B  {}", bytes.len(), visible(bytes));
        }
    }

    fn ended(&mut self, exit_code: Option<u32>) {
        let ms = self.start.elapsed().as_secs_f64() * 1000.0;
        match exit_code {
            Some(code) => {
                println!("{ms:>9.1} ms  The session ended: exit code {code} (0x{code:08X}).")
            }
            None => println!("{ms:>9.1} ms  The session ended; the program is still running."),
        }
        // process::exit does not drop the recorder: it has to be finished.
        if let Some((path, recorder)) = self.recording.take() {
            let (blocks, bytes) = (recorder.blocks(), recorder.bytes());
            match recorder.finish() {
                Ok(_) => println!(
                    "Recorded {blocks} blocks, {bytes} bytes: {}",
                    path.display()
                ),
                Err(error) => eprintln!("Could not write to {}: {error}", path.display()),
            }
        }
        // The exit code of a process is the u32 that Windows keeps.
        std::process::exit(exit_code.map_or(1, |code| code as i32));
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

/// The ConPTY that tools\fetch-conpty.ps1 extracts to <target>\conpty: a
/// folder above this example, which runs from <target>\debug\examples.
fn bundled() -> Result<Conpty, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let folder = exe
        .ancestors()
        .map(|folder| folder.join("conpty"))
        .find(|folder| folder.join("conpty.dll").is_file())
        .ok_or("The ConPTY of meiga is not in the target folder: run tools\\fetch-conpty.ps1, or use --system.")?;
    Conpty::bundled(&folder).map_err(|error| error.to_string())
}

#[derive(Default)]
struct Options {
    system: bool,
    record: Option<PathBuf>,
    quiet: bool,
    /// The program and its arguments.
    command: Vec<String>,
}

fn options() -> Result<Options, String> {
    let mut options = Options::default();
    let mut arguments = std::env::args().skip(1).peekable();
    while let Some(option) = arguments.next_if(|argument| argument.starts_with("--")) {
        match option.as_str() {
            "--system" => options.system = true,
            "--record" => {
                let path = arguments.next().ok_or("--record needs a file")?;
                options.record = Some(PathBuf::from(path));
            }
            "--quiet" => options.quiet = true,
            _ => return Err(format!("Unknown option: {option}")),
        }
    }
    options.command = arguments.collect();
    Ok(options)
}

fn main() -> ExitCode {
    let options = match options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let command_line = if options.command.is_empty() {
        "cmd".to_owned()
    } else {
        options.command.join(" ")
    };
    let conpty = if options.system {
        Conpty::system()
    } else {
        match bundled() {
            Ok(conpty) => conpty,
            Err(message) => {
                eprintln!("{message}");
                return ExitCode::from(2);
            }
        }
    };
    match conpty.source() {
        Source::Bundled(folder) => println!("ConPTY: {}", folder.display()),
        Source::System => println!("ConPTY: Windows (kernel32.dll)"),
    }
    let recording = match options.record {
        Some(path) => match Recorder::create(&path, SIZE) {
            Ok(recorder) => Some((path, recorder)),
            Err(error) => {
                eprintln!("Could not create {}: {error}", path.display());
                return ExitCode::from(2);
            }
        },
        None => None,
    };
    let printer = Printer {
        start: Instant::now(),
        quiet: options.quiet,
        recording,
    };
    let pty = match conpty.spawn(&command_line, SIZE, printer) {
        Ok(pty) => pty,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    println!(
        "Running `{command_line}` (process {}). Type a line and Enter to send it; Ctrl+Z and Enter close the session.",
        pty.process_id()
    );
    for line in io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let mut bytes = line.into_bytes();
        bytes.push(b'\r');
        pty.write(&bytes);
    }
    drop(pty);
    // The reader thread ends the example when the session ends.
    loop {
        thread::park();
    }
}
