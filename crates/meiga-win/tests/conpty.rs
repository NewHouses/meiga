//! Runs real programs in pseudoconsoles: in the ConPTY of meiga, which
//! tools\fetch-conpty.ps1 extracts to <target>\conpty, and in the one of
//! Windows.

// Clippy allows them in test functions (clippy.toml), but not in the helpers
// of an integration test, where a panic is also how a test fails.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::mem;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{self, Child, Stdio};
use std::ptr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use meiga_core::geometry::GridSize;
use meiga_win::conpty::{self, Command, Conpty, Input, Output, Priority, Pty, Source};
use meiga_win::{Error, profiles};
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::System::Console::{CTRL_CLOSE_EVENT, SetConsoleCtrlHandler};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
    QueryInformationJobObject, SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentThread, GetThreadPriority, OpenProcess, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW, THREAD_PRIORITY_BELOW_NORMAL,
    THREAD_PRIORITY_NORMAL,
};
use windows_sys::core::BOOL;

/// How long a test waits for a session at most. A session that gets stuck
/// while it closes ends 10 s after the last of its programs ended.
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

/// `ERROR_FILE_NOT_FOUND` and `ERROR_DIRECTORY`: the errors of
/// CreateProcessW when the program, or the folder where it starts, are not
/// there.
const FILE_NOT_FOUND: i32 = 2;
const NOT_A_FOLDER: i32 = 267;

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
    Failed(Error),
}

/// How a session ended.
#[derive(Debug)]
enum End {
    /// The program ended, with this exit code, if there is one.
    Ended(Option<u32>),
    /// The program could not start.
    Failed(Error),
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

    fn failed(&mut self, error: Error) {
        let _ = self.0.send(Event::Failed(error));
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

    fn failed(&mut self, error: Error) {
        self.channel.failed(error);
    }
}

/// What a session has shown.
struct Transcript {
    events: Receiver<Event>,
    output: Vec<u8>,
}

impl Transcript {
    /// Waits for the next event and keeps its output. Returns how the
    /// session ended, when it ends.
    fn next(&mut self, deadline: Instant) -> Option<End> {
        let timeout = deadline.saturating_duration_since(Instant::now());
        match self.events.recv_timeout(timeout) {
            Ok(Event::Received(bytes)) => {
                self.output.extend(bytes);
                None
            }
            Ok(Event::Ended(exit_code)) => Some(End::Ended(exit_code)),
            Ok(Event::Failed(error)) => Some(End::Failed(error)),
            Err(error) => panic!("{error} after {TIMEOUT:?}; it showed {:?}", self.text()),
        }
    }

    /// Waits until the output contains `text`.
    fn wait_for(&mut self, text: &str) {
        let deadline = Instant::now() + TIMEOUT;
        while !self.text().contains(text) {
            if let Some(end) = self.next(deadline) {
                panic!(
                    "it ended ({end:?}) before showing {text:?}: {:?}",
                    self.text()
                );
            }
        }
    }

    /// Waits until the session ends.
    fn wait_for_any_end(&mut self) -> End {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(end) = self.next(deadline) {
                return end;
            }
        }
    }

    /// Waits until the session ends, and returns the exit code of the
    /// program.
    fn wait_for_end(&mut self) -> Option<u32> {
        match self.wait_for_any_end() {
            End::Ended(exit_code) => exit_code,
            End::Failed(error) => panic!("the program could not start: {error}"),
        }
    }

    /// Waits until the session tells that its program could not start, and
    /// returns why.
    fn wait_for_failure(&mut self) -> Error {
        match self.wait_for_any_end() {
            End::Failed(error) => error,
            End::Ended(exit_code) => panic!(
                "the program started and ended ({exit_code:?}): {:?}",
                self.text()
            ),
        }
    }

    fn text(&self) -> String {
        String::from_utf8_lossy(&self.output).into_owned()
    }
}

/// Waits until the writer of the session of `pty` has started its program,
/// without sleeping, so that the test can close the session right after.
fn wait_for_program(pty: &Pty) {
    let deadline = Instant::now() + TIMEOUT;
    while pty.process_id().is_none() {
        assert!(Instant::now() < deadline, "the program did not start");
        thread::yield_now();
    }
}

fn spawn(conpty: &Conpty, command_line: &str) -> (Pty, Transcript) {
    spawn_command(conpty, &Command::new(command_line))
}

fn spawn_command(conpty: &Conpty, command: &Command<'_>) -> (Pty, Transcript) {
    let (sender, events) = mpsc::channel();
    let pty = conpty
        .spawn(command, GridSize::new(80, 25), Channel(sender))
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
fn a_program_starts_in_its_folder_with_its_environment() {
    // A short folder, which a line of the terminal holds.
    let folder = profiles::system_folder().unwrap();
    let mut environment = profiles::environment();
    environment.set("MEIGA_CHECK", "ola meiga");
    environment.remove("COMPUTERNAME");
    let block = environment.block();
    let command = Command {
        line: "cmd /d /c \"cd & echo [%MEIGA_CHECK%] [%COMPUTERNAME%]\"",
        folder: Some(&folder),
        environment: Some(&block),
    };
    let (_pty, mut transcript) = spawn_command(&bundled(), &command);
    assert_eq!(transcript.wait_for_end(), Some(0));
    let text = transcript.text();
    // cmd writes the folder as the disk has it: C:\Windows\System32 for
    // C:\WINDOWS\system32.
    let line = format!("{folder}\r\n").to_lowercase();
    assert!(text.to_lowercase().contains(&line), "{text:?}");
    // cmd leaves a variable that is not there as it is.
    assert!(text.contains("[ola meiga] [%COMPUTERNAME%]"), "{text:?}");
}

#[test]
fn a_program_does_not_start_in_a_folder_that_is_not_there() {
    let command = Command {
        folder: Some(r"C:\meiga\no such folder"),
        ..Command::new("cmd /c exit")
    };
    // The session starts at once, and its writer tells, through the reader,
    // why the program did not start.
    let (pty, mut transcript) = spawn_command(&bundled(), &command);
    let error = transcript.wait_for_failure();
    assert_eq!(
        error.os_error().and_then(io::Error::raw_os_error),
        Some(NOT_A_FOLDER),
        "{error}"
    );
    assert!(
        error.to_string().starts_with("CreateProcessW failed: "),
        "{error}"
    );
    assert_eq!(pty.process_id(), None);
    // What is written to it, and its size, do nothing.
    pty.write(b"ola\r");
    pty.resize(GridSize::new(100, 30)).unwrap();
}

#[test]
fn a_program_that_is_not_there_does_not_start() {
    let (pty, mut transcript) = spawn(&bundled(), "meiga-there-is-no-such-program-ola");
    let error = transcript.wait_for_failure();
    assert_eq!(
        error.os_error().and_then(io::Error::raw_os_error),
        Some(FILE_NOT_FOUND),
        "{error}"
    );
    assert_eq!(pty.process_id(), None);
}

#[test]
fn what_is_written_before_the_program_starts_reaches_it() {
    // The writer creates the program after spawn returns: the line waits for
    // it, as what the user types in a tab that opens. cmd reads it and shows
    // it twice: "olaola" only comes from cmd.
    let (pty, mut transcript) = spawn(&bundled(), "cmd /d /v:on /c \"set /p M=& echo !M!!M!\"");
    pty.write(b"ola\r");
    assert_eq!(transcript.wait_for_end(), Some(0));
    assert!(
        transcript.text().contains("olaola"),
        "{:?}",
        transcript.text()
    );
}

#[test]
fn the_size_given_before_the_program_starts_is_the_one_that_it_gets() {
    // As when the window changes size while a tab opens.
    let (pty, mut transcript) = spawn(
        &bundled(),
        "powershell -NoProfile -Command \"'size=' + [Console]::WindowWidth + 'x' + [Console]::WindowHeight\"",
    );
    pty.resize(GridSize::new(123, 37)).unwrap();
    assert_eq!(transcript.wait_for_end(), Some(0));
    assert!(
        transcript.text().contains("size=123x37"),
        "{:?}",
        transcript.text()
    );
}

#[test]
fn the_process_of_the_program_is_known_once_it_starts() {
    let (pty, mut transcript) = spawn(&bundled(), "cmd /k");
    transcript.wait_for(">");
    let id = pty.process_id().expect("the program has started");
    assert_eq!(image_name(id).as_deref(), Some("cmd.exe"));
    drop(pty);
    assert_eq!(transcript.wait_for_end(), Some(CONSOLE_CLOSED));
}

#[test]
fn the_conpty_next_to_the_executable_runs_programs() {
    // This test program has no conpty.dll next to it: the writer of the first
    // session loads the ConPTY of Windows.
    let (sender, events) = mpsc::channel();
    let _pty = conpty::spawn(
        &Command::new("cmd /c echo ola"),
        GridSize::new(80, 25),
        Channel(sender),
    )
    .unwrap();
    let mut transcript = Transcript {
        events,
        output: Vec::new(),
    };
    assert_eq!(transcript.wait_for_end(), Some(0));
    assert!(
        transcript.text().contains("ola\r\n"),
        "{:?}",
        transcript.text()
    );
}

#[test]
fn a_bundled_conpty_needs_its_two_files_and_its_functions() {
    // In the temporary folder of the target, which cargo gives integration
    // tests: a library that this process loads stays there until it exits.
    let folder =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("conpty-files-{}", process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let error_of = |folder: &Path| match Conpty::bundled(folder) {
        Ok(_) => panic!("{} has a ConPTY", folder.display()),
        Err(error) => error,
    };
    // Without conpty.dll, and then without OpenConsole.exe.
    for missing in ["conpty.dll", "OpenConsole.exe"] {
        let error = error_of(&folder);
        assert_eq!(
            error.os_error().map(io::Error::kind),
            Some(io::ErrorKind::NotFound),
            "{error}"
        );
        assert!(
            error
                .to_string()
                .contains(&format!("{missing} does not exist")),
            "{error}"
        );
        fs::write(folder.join(missing), b"not a program").unwrap();
    }
    // Both are there, but conpty.dll is not a library: ERROR_BAD_EXE_FORMAT.
    let error = error_of(&folder);
    assert!(
        error.to_string().starts_with("LoadLibraryExW failed: "),
        "{error}"
    );
    assert_eq!(
        error.os_error().and_then(io::Error::raw_os_error),
        Some(193),
        "{error}"
    );
    // A library of Windows, without the functions of ConPTY:
    // ERROR_PROC_NOT_FOUND.
    let library = PathBuf::from(profiles::system_folder().unwrap()).join("version.dll");
    fs::copy(library, folder.join("conpty.dll")).unwrap();
    let error = error_of(&folder);
    assert!(
        error.to_string().starts_with("GetProcAddress failed: "),
        "{error}"
    );
    assert_eq!(
        error.os_error().and_then(io::Error::raw_os_error),
        Some(127),
        "{error}"
    );
}

/// The file name of the program of the process `id`, if it is still there.
fn image_name(id: u32) -> Option<String> {
    // SAFETY: it asks only for the right to read the name, and does not let
    // new processes inherit the handle. It returns null if it fails.
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, id) };
    if handle.is_null() {
        return None;
    }
    // SAFETY: OpenProcess has just opened it, and nothing else owns it.
    let process = unsafe { OwnedHandle::from_raw_handle(handle) };
    let mut name = vec![0_u16; 32_768];
    let mut length = name.len() as u32;
    // SAFETY: the buffer has `length` characters, and the handle has the
    // right to read the name.
    let read = unsafe {
        QueryFullProcessImageNameW(
            process.as_raw_handle(),
            PROCESS_NAME_WIN32,
            name.as_mut_ptr(),
            &mut length,
        )
    };
    if read == 0 {
        return None;
    }
    let path = String::from_utf16_lossy(name.get(..length as usize)?);
    path.rsplit('\\').next().map(str::to_lowercase)
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
            &Command::new("cmd /d /v:on /c \"set /p M=& echo !M!!M!\""),
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
fn the_program_gets_the_new_size() {
    let (pty, mut transcript) = spawn(&bundled(), "cmd /k");
    transcript.wait_for(">");
    pty.resize(GridSize::new(123, 37)).unwrap();
    // Windows PowerShell shows the size of the console after a mark that the
    // line that runs it does not have. The output of mode, in the language
    // of Windows, has other numbers, such as the code page 437.
    pty.write(
        b"powershell -NoProfile -Command \"'size=' + [Console]::WindowWidth + 'x' + [Console]::WindowHeight\"\r",
    );
    transcript.wait_for("size=123x37");
    pty.write(b"exit 3\r");
    assert_eq!(transcript.wait_for_end(), Some(3));
    // Once the session ends, it does nothing.
    pty.resize(GridSize::new(80, 25)).unwrap();
}

/// Sends with each block of output the priority of the reader thread, which
/// it reads on that thread, as it gives the block.
struct Priorities(Sender<(i32, Vec<u8>)>);

impl Output for Priorities {
    fn received(&mut self, bytes: &[u8], _input: &Input) {
        // SAFETY: GetCurrentThread returns a pseudo handle of this thread,
        // which GetThreadPriority can read.
        let priority = unsafe { GetThreadPriority(GetCurrentThread()) };
        let _ = self.0.send((priority, bytes.to_vec()));
    }

    fn ended(&mut self, _exit_code: Option<u32>) {}

    fn failed(&mut self, error: Error) {
        panic!("the program could not start: {error}");
    }
}

#[test]
fn the_priority_given_before_the_program_starts_is_the_one_of_its_output() {
    // The reader starts with the session, before the writer starts the
    // program: a tab that is hidden at once, as one that restarts while it
    // is hidden, gets the lower priority from its first output.
    let (sender, blocks) = mpsc::channel();
    let pty = bundled()
        .spawn(
            &Command::new("cmd /d /c echo priority-0"),
            GridSize::new(80, 25),
            Priorities(sender),
        )
        .unwrap();
    pty.set_reader_priority(Priority::BelowNormal).unwrap();
    let mut output = Vec::new();
    let mut priorities = Vec::new();
    while !String::from_utf8_lossy(&output).contains("priority-0") {
        let (priority, block) = blocks.recv_timeout(TIMEOUT).unwrap();
        priorities.push(priority);
        output.extend(block);
    }
    assert!(
        priorities
            .iter()
            .all(|&priority| priority == THREAD_PRIORITY_BELOW_NORMAL),
        "{priorities:?}"
    );
}

#[test]
fn the_priority_of_the_reader_changes_while_the_session_runs() {
    let (sender, blocks) = mpsc::channel();
    let pty = bundled()
        .spawn(
            &Command::new("cmd /k"),
            GridSize::new(80, 25),
            Priorities(sender),
        )
        .unwrap();
    // The priority of the block that brings `text`, which the program writes
    // after the line that is typed now. The words are not in the folder
    // that the prompt shows, which comes before.
    let typed = |line: &[u8], text: &str| {
        pty.write(line);
        let mut output = Vec::new();
        loop {
            let (priority, block) = blocks.recv_timeout(TIMEOUT).unwrap();
            output.extend(block);
            if String::from_utf8_lossy(&output).contains(text) {
                return priority;
            }
        }
    };
    assert_eq!(
        typed(b"echo priority-1\r", "priority-1"),
        THREAD_PRIORITY_NORMAL
    );
    pty.set_reader_priority(Priority::BelowNormal).unwrap();
    assert_eq!(
        typed(b"echo priority-2\r", "priority-2"),
        THREAD_PRIORITY_BELOW_NORMAL
    );
    pty.set_reader_priority(Priority::Normal).unwrap();
    assert_eq!(
        typed(b"echo priority-3\r", "priority-3"),
        THREAD_PRIORITY_NORMAL
    );
    drop(pty);
    // Once the session has ended, it does nothing: the reader thread, whose
    // last step tells that the session ended, is ending or has ended, and
    // SetThreadPriority does nothing on a thread that has ended.
    let (pty, mut transcript) = spawn(&bundled(), "cmd /c exit 4");
    assert_eq!(transcript.wait_for_end(), Some(4));
    pty.set_reader_priority(Priority::BelowNormal).unwrap();
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
    // can fail by itself. The session closes as soon as the writer has
    // created the program.
    let started = Instant::now();
    let (pty, mut transcript) = spawn(&bundled(), "cmd /k");
    wait_for_program(&pty);
    drop(pty);
    let exit_code = transcript.wait_for_end();
    assert!(
        matches!(exit_code, Some(CONSOLE_CLOSED | KILLED | NOT_CONNECTED)),
        "{exit_code:?} after {:?}",
        started.elapsed()
    );
}

#[test]
fn sessions_dropped_as_soon_as_they_start_end_too() {
    // Closed before the writer creates the pseudoconsole, while it creates it,
    // or while the program starts: the session ends in each case, without a
    // program, or with the program that started and ended. As many tabs that
    // open and close at once, the sessions start at the same time.
    let conpty = bundled();
    let sessions: Vec<_> = (0..20_u64)
        .map(|index| {
            let conpty = conpty.clone();
            thread::spawn(move || {
                let (pty, mut transcript) = spawn(&conpty, "cmd /k");
                thread::sleep(Duration::from_millis(index % 4));
                drop(pty);
                transcript.wait_for_any_end()
            })
        })
        .collect();
    for session in sessions {
        let end = session.join().unwrap();
        assert!(
            matches!(
                end,
                End::Ended(None | Some(CONSOLE_CLOSED | KILLED | NOT_CONNECTED))
            ),
            "{end:?}"
        );
    }
}

#[test]
fn sessions_dropped_while_the_shell_of_a_launcher_starts_end_too() {
    // The same can happen to a program that the program of the session has
    // just started, here the cmd /k of cmd /c: the session has to end it
    // too. The moment is narrow, so many sessions close, each at a different
    // moment after its program starts. One that gets stuck ends after some
    // seconds.
    let conpty = bundled();
    let sessions: Vec<_> = (0..20)
        .map(|index| {
            let conpty = conpty.clone();
            thread::spawn(move || {
                let started = Instant::now();
                let (pty, mut transcript) = spawn(&conpty, "cmd /c cmd /k");
                wait_for_program(&pty);
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

#[test]
fn a_session_whose_programs_keep_ending_is_not_ended_by_the_writer() {
    // Windows asks the three programs to end one at a time, and each takes
    // 4 s: the session takes 12 s to end, longer than the 10 s that the
    // writer waits for a program to end, but one ends every 4 s.
    let (pty, mut transcript) = spawn_helper(&bundled(), "slow");
    for index in 0..3 {
        transcript.wait_for(&format!("slow-ready-{index}"));
    }
    let closing = Instant::now();
    drop(pty);
    let exit_code = transcript.wait_for_end();
    let elapsed = closing.elapsed();
    // The writer did not end it: its console did, as it asked.
    assert_eq!(exit_code, Some(CONSOLE_CLOSED), "after {elapsed:?}");
    assert!(elapsed >= Duration::from_secs(11), "{elapsed:?}");
}

#[test]
fn the_sessions_that_were_closed_end_before_the_process_exits() {
    // Another process closes a session that gets stuck, another one, and two
    // that are starting, waits for them, and exits. The job of the test
    // counts its programs, and those of its sessions, which are in it too:
    // it ends the ones left only when it is dropped, after the test has
    // counted them.
    let job = Job::new();
    let mut exiter = process::Command::new(env::current_exe().unwrap())
        .args(HELPER_ARGUMENTS)
        .env(HELPER, "exiter")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    job.add(&exiter);
    writeln!(exiter.stdin.take().unwrap(), "go").unwrap();
    let lines = lines_of(BufReader::new(exiter.stdout.take().unwrap()));
    let deadline = Instant::now() + TIMEOUT;
    let mut output = Vec::new();
    let waited = loop {
        let line = lines
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap_or_else(|error| panic!("{error}; the helper wrote {output:?}"));
        // The test runner writes the name of the test before it, on the same
        // line.
        if let Some(ms) = line
            .rsplit_once("waited ")
            .and_then(|(_, line)| line.strip_suffix(" ms"))
        {
            break Duration::from_millis(ms.parse().unwrap());
        }
        output.push(line);
    };
    let status = loop {
        if let Some(status) = exiter.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "the helper did not exit");
        thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success(), "{status}");
    // Its consoles end a moment after their programs.
    let deadline = Instant::now() + Duration::from_secs(5);
    while job.running() > 0 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(job.running(), 0, "programs left after it waited {waited:?}");
    // The stuck session ended 10 s after its last program, once it started
    // the one that got stuck. The writer used to see that only when it
    // looked again, after 20 s.
    assert!(
        (Duration::from_secs(10)..Duration::from_secs(17)).contains(&waited),
        "{waited:?}"
    );
}

/// The variable that makes this test program run one of the programs of
/// [`helper`], instead of the tests.
const HELPER: &str = "MEIGA_TEST_HELPER";

/// The arguments of the test runner that run only [`helper`], with what it
/// writes in sight.
const HELPER_ARGUMENTS: [&str; 5] = [
    "--exact",
    "helper",
    "--ignored",
    "--nocapture",
    "--test-threads=1",
];

/// The variable that tells a slow helper which one it is: the first one
/// starts the other two.
const SLOW: &str = "MEIGA_TEST_SLOW";

/// How long a slow helper takes to handle `CTRL_CLOSE_EVENT`: less than the
/// 5 s that Windows gives each program.
const SLOW_CLOSE: Duration = Duration::from_secs(4);

/// The programs that some tests run, from this same test program, with
/// [`HELPER`] set to their role. The test runner runs it only with
/// `--ignored`, and then, without [`HELPER`], it does nothing.
#[test]
#[ignore = "a program that other tests run"]
fn helper() {
    match env::var(HELPER).as_deref() {
        Ok("closer") => closer(),
        Ok("slow") => slow(),
        Ok("exiter") => exiter(),
        _ => {}
    }
}

/// Runs `role` of [`helper`] in a session.
fn spawn_helper(conpty: &Conpty, role: &str) -> (Pty, Transcript) {
    let mut environment = profiles::environment();
    environment.set(HELPER, role);
    let block = environment.block();
    let exe = env::current_exe().unwrap();
    let line = format!("\"{}\" {}", exe.display(), HELPER_ARGUMENTS.join(" "));
    let command = Command {
        line: &line,
        folder: None,
        environment: Some(&block),
    };
    spawn_command(conpty, &command)
}

/// A program that, when its console closes, starts another one in it, as
/// the shell of a tab can start one when the tab closes, such as the git of
/// its prompt. The session gets stuck: the new program connects to the
/// console while ConPTY closes it (ADR 0011 and ADR 0026).
fn closer() {
    // SAFETY: the handler is a function, which lives as long as the
    // process.
    unsafe { SetConsoleCtrlHandler(Some(start_on_close), TRUE) };
    println!("closer-ready");
    loop {
        thread::park();
    }
}

unsafe extern "system" fn start_on_close(event: u32) -> BOOL {
    if event == CTRL_CLOSE_EVENT {
        let _ = process::Command::new("cmd")
            .raw_arg("/d /c ping -n 30 127.0.0.1 >nul")
            .spawn();
        // So that it starts before this program ends.
        thread::sleep(Duration::from_millis(500));
    }
    TRUE
}

/// A program that takes [`SLOW_CLOSE`] to handle `CTRL_CLOSE_EVENT`. The
/// first one starts two more in its console, and waits for them: they end
/// first, as Windows asks the newest programs first.
fn slow() {
    // SAFETY: as in `closer`.
    unsafe { SetConsoleCtrlHandler(Some(wait_on_close), TRUE) };
    let index = env::var(SLOW).unwrap_or_else(|_| "0".to_owned());
    let mut others = Vec::new();
    if index == "0" {
        for index in 1..=2 {
            let other = process::Command::new(env::current_exe().unwrap())
                .args(HELPER_ARGUMENTS)
                .env(SLOW, index.to_string())
                .spawn()
                .unwrap();
            others.push(other);
        }
    }
    println!("slow-ready-{index}");
    for mut other in others {
        let _ = other.wait();
    }
    loop {
        thread::park();
    }
}

unsafe extern "system" fn wait_on_close(event: u32) -> BOOL {
    if event == CTRL_CLOSE_EVENT {
        thread::sleep(SLOW_CLOSE);
    }
    TRUE
}

/// A process that closes a session that gets stuck, one that ends at once,
/// and two that are starting, waits for them, tells how long it waited, and
/// exits.
fn exiter() {
    // The test says go once this process is in its job.
    let mut go = String::new();
    io::stdin().read_line(&mut go).unwrap();
    let conpty = bundled();
    let (stuck, mut transcript) = spawn_helper(&conpty, "closer");
    transcript.wait_for("closer-ready");
    let (quick, mut transcript) = spawn(&conpty, "cmd /k");
    transcript.wait_for(">");
    // The program of one has just started, and connects to its console; the
    // writer of the other one may not even have created its pseudoconsole.
    // The process waits for both, as for the others (ADR 0026 and ADR 0028).
    let (starting, _) = spawn(&conpty, "cmd /k");
    wait_for_program(&starting);
    let (unborn, _) = spawn(&conpty, "cmd /k");
    let closing = Instant::now();
    // The writer of the quick one comes after the one of the stuck one,
    // which has to stay while it runs.
    drop(stuck);
    drop(quick);
    drop(starting);
    drop(unborn);
    meiga_win::conpty::wait_for_closed_sessions();
    println!("waited {} ms", closing.elapsed().as_millis());
}

/// The lines that `reader` reads, on another thread.
fn lines_of(reader: impl BufRead + Send + 'static) -> Receiver<String> {
    let (sender, lines) = mpsc::channel();
    thread::spawn(move || {
        for line in reader.lines() {
            let Ok(line) = line else { break };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    lines
}

/// A job for the programs of a test, which ends those still running when it
/// is dropped, also if the test fails.
struct Job(OwnedHandle);

impl Job {
    fn new() -> Self {
        // SAFETY: no security attributes and no name. It returns null if it
        // fails.
        let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        assert!(!handle.is_null(), "{}", io::Error::last_os_error());
        // SAFETY: CreateJobObjectW has just created it, and nothing else owns
        // it.
        let job = Self(unsafe { OwnedHandle::from_raw_handle(handle) });
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: the job is valid, and the information is the structure of
        // its class, with its size.
        let set = unsafe {
            SetInformationJobObject(
                job.0.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                ptr::from_ref(&limits).cast(),
                mem::size_of_val(&limits) as u32,
            )
        };
        assert_ne!(set, 0, "{}", io::Error::last_os_error());
        job
    }

    fn add(&self, child: &Child) {
        // SAFETY: both handles are valid while `self` and `child` exist.
        let added =
            unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), child.as_raw_handle()) };
        assert_ne!(added, 0, "{}", io::Error::last_os_error());
    }

    /// How many programs of the job are running.
    fn running(&self) -> u32 {
        let mut information = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        // SAFETY: the job is valid, and the information is the structure of
        // its class, with its size.
        let queried = unsafe {
            QueryInformationJobObject(
                self.0.as_raw_handle(),
                JobObjectBasicAccountingInformation,
                ptr::from_mut(&mut information).cast(),
                mem::size_of_val(&information) as u32,
                ptr::null_mut(),
            )
        };
        assert_ne!(queried, 0, "{}", io::Error::last_os_error());
        information.ActiveProcesses
    }
}
