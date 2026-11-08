//! Runs a program in a pseudoconsole, shows each block of what it writes with
//! the control characters in sight, and sends it each line that you type,
//! followed by Enter.
//!
//! ```text
//! cargo run -p meiga-win --example conpty -- [--system] [command line]
//! ```
//!
//! It uses the ConPTY that tools\fetch-conpty.ps1 extracts to the target
//! folder or, with `--system`, the one of Windows. Without a command line, it
//! runs `cmd`. The arguments are joined with spaces. It ends when the session
//! ends, with the exit code of the program. Ctrl+Z and Enter close the
//! session.

use std::fmt::Write as _;
use std::io::{self, BufRead};
use std::process::ExitCode;
use std::thread;
use std::time::Instant;

use meiga_core::geometry::GridSize;
use meiga_win::conpty::{Conpty, Output, Source};

/// The size of the pseudoconsole: the default of a console window.
const SIZE: GridSize = GridSize::new(120, 30);

/// Shows what the session sends, and ends the example when it ends.
struct Printer {
    start: Instant,
}

impl Output for Printer {
    fn received(&mut self, bytes: &[u8]) {
        let ms = self.start.elapsed().as_secs_f64() * 1000.0;
        println!("{ms:>9.1} ms {:>6} B  {}", bytes.len(), visible(bytes));
    }

    fn ended(&mut self, exit_code: Option<u32>) {
        let ms = self.start.elapsed().as_secs_f64() * 1000.0;
        match exit_code {
            Some(code) => {
                println!("{ms:>9.1} ms  The session ended: exit code {code} (0x{code:08X}).")
            }
            None => println!("{ms:>9.1} ms  The session ended; the program is still running."),
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

fn main() -> ExitCode {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();
    let system = arguments
        .first()
        .is_some_and(|argument| argument == "--system");
    if system {
        arguments.remove(0);
    }
    let command_line = if arguments.is_empty() {
        "cmd".to_owned()
    } else {
        arguments.join(" ")
    };
    let conpty = if system {
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
    let printer = Printer {
        start: Instant::now(),
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
