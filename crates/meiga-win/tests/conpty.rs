//! Runs real programs in pseudoconsoles: in the ConPTY of meiga, which
//! tools\fetch-conpty.ps1 extracts to <target>\conpty, and in the one of
//! Windows.

// Clippy allows them in test functions (clippy.toml), but not in the helpers
// of an integration test, where a panic is also how a test fails.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use meiga_core::geometry::GridSize;
use meiga_win::conpty::{Conpty, Input, Output, Pty, Source};

/// How long a test waits for a session at most. A session that gets stuck
/// while it closes ends after 10 or 20 s.
const TIMEOUT: Duration = Duration::from_secs(60);

/// `STATUS_CONTROL_C_EXIT`: the exit code of a console program whose console
/// closes.
const CONSOLE_CLOSED: u32 = 0xC000_013A;

/// The exit code of a program that the session ends because it did not end
/// by itself after the pseudoconsole closed.
const KILLED: u32 = 1;

/// `STATUS_DLL_INIT_FAILED`: the exit code of a console program whose
/// console ends before the program has connected to it.
const NOT_CONNECTED: u32 = 0xC000_0142;

/// The ConPTY that tools\fetch-conpty.ps1 extracts to <target>\conpty: a
/// folder above this test program, which runs from <target>\debug\deps.
fn bundled() -> Conpty {
    let exe = std::env::current_exe().unwrap();
    let folder = exe
        .ancestors()
        .map(|folder| folder.join("conpty"))
        .find(|folder| folder.join("conpty.dll").is_file())
        .expect("the ConPTY of meiga is not in the target folder: run tools\\fetch-conpty.ps1");
    Conpty::bundled(&folder).unwrap()
}

enum Event {
    Received(Vec<u8>),
    Ended(Option<u32>),
}

/// Sends what the session shows to the test.
struct Channel(Sender<Event>);

impl Output for Channel {
    fn received(&mut self, bytes: &[u8], _input: &Input) {
        let _ = self.0.send(Event::Received(bytes.to_vec()));
    }

    fn ended(&mut self, exit_code: Option<u32>) {
        let _ = self.0.send(Event::Ended(exit_code));
    }
}

/// Like [`Channel`], but it also types a line on the reader thread when
/// the program first shows something, as a terminal answers a question.
struct Answering {
    channel: Channel,
    line: Option<&'static [u8]>,
}

impl Output for Answering {
    fn received(&mut self, bytes: &[u8], input: &Input) {
        if let Some(line) = self.line.take() {
            input.write(line);
        }
        self.channel.received(bytes, input);
    }

    fn ended(&mut self, exit_code: Option<u32>) {
        self.channel.ended(exit_code);
    }
}

/// What a session has shown.
struct Transcript {
    events: Receiver<Event>,
    output: Vec<u8>,
}

impl Transcript {
    /// Waits for the next event and keeps its output. Returns the exit code
    /// when the session ends.
    fn next(&mut self, deadline: Instant) -> Option<Option<u32>> {
        let timeout = deadline.saturating_duration_since(Instant::now());
        match self.events.recv_timeout(timeout) {
            Ok(Event::Received(bytes)) => {
                self.output.extend(bytes);
                None
            }
            Ok(Event::Ended(exit_code)) => Some(exit_code),
            Err(error) => panic!("{error} after {TIMEOUT:?}; it showed {:?}", self.text()),
        }
    }

    /// Waits until the output contains `text`.
    fn wait_for(&mut self, text: &str) {
        let deadline = Instant::now() + TIMEOUT;
        while !self.text().contains(text) {
            if let Some(exit_code) = self.next(deadline) {
                panic!(
                    "it ended ({exit_code:?}) before showing {text:?}: {:?}",
                    self.text()
                );
            }
        }
    }

    /// Waits until the session ends, and returns the exit code of the
    /// program.
    fn wait_for_end(&mut self) -> Option<u32> {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(exit_code) = self.next(deadline) {
                return exit_code;
            }
        }
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.output).into_owned()
    }
}

fn spawn(conpty: &Conpty, command_line: &str) -> (Pty, Transcript) {
    let (sender, events) = mpsc::channel();
    let pty = conpty
        .spawn(command_line, GridSize::new(80, 25), Channel(sender))
        .unwrap();
    let transcript = Transcript {
        events,
        output: Vec::new(),
    };
    (pty, transcript)
}

#[test]
fn the_bundled_conpty_runs_a_program_and_reads_what_it_shows() {
    let conpty = bundled();
    assert!(matches!(conpty.source(), Source::Bundled(_)));
    let (_pty, mut transcript) = spawn(&conpty, "cmd /c echo ola");
    assert_eq!(transcript.wait_for_end(), Some(0));
    assert!(
        transcript.text().contains("ola\r\n"),
        "{:?}",
        transcript.text()
    );
}

#[test]
fn the_conpty_of_windows_runs_a_program_and_reads_what_it_shows() {
    let conpty = Conpty::system();
    assert_eq!(conpty.source(), &Source::System);
    let (_pty, mut transcript) = spawn(&conpty, "cmd /c echo ola");
    assert_eq!(transcript.wait_for_end(), Some(0));
    assert!(
        transcript.text().contains("ola\r\n"),
        "{:?}",
        transcript.text()
    );
}

#[test]
fn the_session_tells_the_exit_code_of_the_program() {
    let (_pty, mut transcript) = spawn(&bundled(), "cmd /c exit 7");
    assert_eq!(transcript.wait_for_end(), Some(7));
}

#[test]
fn what_is_written_reaches_the_program() {
    let (pty, mut transcript) = spawn(&bundled(), "cmd /k");
    transcript.wait_for(">");
    // "olaola" is not in what is written: only cmd can show it.
    pty.write(b"set M=ola\r");
    pty.write(b"echo %M%%M%\r");
    pty.write(b"exit 5\r");
    assert_eq!(transcript.wait_for_end(), Some(5));
    assert!(
        transcript.text().contains("olaola"),
        "{:?}",
        transcript.text()
    );
}

#[test]
fn the_reader_thread_can_answer_the_program() {
    let (sender, events) = mpsc::channel();
    let output = Answering {
        channel: Channel(sender),
        line: Some(b"ola\r"),
    };
    // cmd reads a line and shows it twice: "olaola" only comes from cmd.
    let _pty = bundled()
        .spawn(
            "cmd /d /v:on /c \"set /p M=& echo !M!!M!\"",
            GridSize::new(80, 25),
            output,
        )
        .unwrap();
    let mut transcript = Transcript {
        events,
        output: Vec::new(),
    };
    assert_eq!(transcript.wait_for_end(), Some(0));
    assert!(
        transcript.text().contains("olaola"),
        "{:?}",
        transcript.text()
    );
}

#[test]
fn dropping_a_session_closes_it_without_waiting() {
    let (pty, mut transcript) = spawn(&bundled(), "cmd /k");
    transcript.wait_for(">");
    let dropping = Instant::now();
    drop(pty);
    let dropped = dropping.elapsed();
    assert!(dropped < Duration::from_millis(100), "{dropped:?}");
    // cmd ends because its console closes, not because it is killed.
    assert_eq!(transcript.wait_for_end(), Some(CONSOLE_CLOSED));
}

#[test]
fn a_session_dropped_while_its_program_starts_ends_too() {
    // When the pseudoconsole closes while the program is still connecting
    // to it, the program can wait for ever, and the session ends it, or it
    // can fail by itself.
    let started = Instant::now();
    let (pty, mut transcript) = spawn(&bundled(), "cmd /k");
    drop(pty);
    let exit_code = transcript.wait_for_end();
    assert!(
        matches!(exit_code, Some(CONSOLE_CLOSED | KILLED | NOT_CONNECTED)),
        "{exit_code:?} after {:?}",
        started.elapsed()
    );
}

#[test]
fn sessions_dropped_while_the_shell_of_a_launcher_starts_end_too() {
    // The same can happen to a program that the program of the session has
    // just started, here the cmd /k of cmd /c: the session has to end it
    // too. The moment is narrow, so many sessions close, each at a different
    // moment after it starts. One that gets stuck ends after some seconds.
    let conpty = bundled();
    let sessions: Vec<_> = (0..20)
        .map(|index| {
            let conpty = conpty.clone();
            thread::spawn(move || {
                let started = Instant::now();
                let (pty, mut transcript) = spawn(&conpty, "cmd /c cmd /k");
                thread::sleep(Duration::from_millis(10 + 2 * index));
                drop(pty);
                (transcript.wait_for_end(), started.elapsed())
            })
        })
        .collect();
    for session in sessions {
        let (exit_code, elapsed) = session.join().unwrap();
        assert!(
            matches!(exit_code, Some(CONSOLE_CLOSED | KILLED | NOT_CONNECTED)),
            "{exit_code:?} after {elapsed:?}"
        );
    }
}
