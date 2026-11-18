//! Runs real programs in pseudoconsoles: in the ConPTY of meiga, which
//! tools\fetch-conpty.ps1 extracts to <target>\conpty, and in the one of
//! Windows.

// Clippy allows them in test functions (clippy.toml), but not in the helpers
// of an integration test, where a panic is also how a test fails.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::mem;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::os::windows::process::CommandExt;
use std::process::{self, Child, Stdio};
use std::ptr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use meiga_core::geometry::GridSize;
use meiga_win::conpty::{Command, Conpty, Input, Output, Priority, Pty, Source};
use meiga_win::profiles;
use windows_sys::Win32::Foundation::TRUE;
use windows_sys::Win32::System::Console::{CTRL_CLOSE_EVENT, SetConsoleCtrlHandler};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
    QueryInformationJobObject, SetInformationJobObject,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentThread, GetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL, THREAD_PRIORITY_NORMAL,
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
    let (sender, _events) = mpsc::channel();
    let Err(error) = bundled().spawn(&command, GridSize::new(80, 25), Channel(sender)) else {
        panic!("it started in a folder that is not there");
    };
    // ERROR_DIRECTORY: the directory name is invalid.
    assert_eq!(
        error.os_error().and_then(io::Error::raw_os_error),
        Some(267),
        "{error}"
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
    // Another process closes a session that gets stuck and another one,
    // waits for them, and exits. Its job ends nothing: the job only counts
    // its programs, and those of its sessions, which are in it too.
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

/// A process that closes a session that gets stuck, and then one that ends
/// at once, waits for them, tells how long it waited, and exits.
fn exiter() {
    // The test says go once this process is in its job.
    let mut go = String::new();
    io::stdin().read_line(&mut go).unwrap();
    let conpty = bundled();
    let (stuck, mut transcript) = spawn_helper(&conpty, "closer");
    transcript.wait_for("closer-ready");
    let (quick, mut transcript) = spawn(&conpty, "cmd /k");
    transcript.wait_for(">");
    let closing = Instant::now();
    // The writer of the quick one comes after the one of the stuck one,
    // which has to stay while it runs.
    drop(stuck);
    drop(quick);
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
