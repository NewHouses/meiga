//! The session of a tab: the program that runs in a pseudoconsole, the
//! reader thread that gives its output to the terminal of the tab, and what
//! that thread shares with the window. The writer thread of the session
//! creates the pseudoconsole and the program, so the window never waits for
//! Windows to create them (ADR 0028): if the program cannot start, the
//! reader tells it, as the end of a program.

use std::fmt;
use std::io;
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use meiga_core::geometry::GridSize;
use meiga_core::model::TabId;
use meiga_core::profile::Launch;
use meiga_core::term::{Changes, Terminal};
use meiga_win::Error;
use meiga_win::conpty::{self, Command, Input, Output, Pty};
use meiga_win::window::Waker;

/// `ERROR_DIRECTORY`: the error of Windows when the folder where a program
/// starts is not there, or is not a folder.
const ERROR_DIRECTORY: i32 = 267;

/// What the reader thread of the session and the window share, behind a
/// mutex: the terminal, and what the window has been woken up for.
///
/// The reader wakes the window up only if it was not already woken up for
/// the same thing (ADR 0005): for the screen, until the window copies it;
/// for the title and for the clipboard, until the window takes them; for
/// the bell, until the tab is hidden again (ADR 0025); and for a
/// synchronized update, until the window knows when it ends. While the
/// window is minimized, or the tab is hidden, the window does not copy the
/// screen, so the output does not wake it up again, except to end a
/// synchronized update in time, and for the title, which the tab bar shows
/// for every tab (ADR 0020). When a tab is hidden, the window forgets the
/// screen and the bell, so the first output and the first bell after it
/// wake it up, once: its tab tells them (ADR 0025). What a hidden tab
/// copied to the clipboard waits until the tab is shown, and is dropped
/// then (ADR 0019), also in a synchronized update that its program has not
/// ended yet (ADR 0020). The end of the program, or the failure of a program
/// that could not start, always wakes the window up, once (ADR 0024 and ADR
/// 0028).
pub struct Shared {
    pub terminal: Terminal,
    /// The screen may have changed since the window last copied it or, for
    /// a hidden tab, since it was hidden.
    pub screen_changed: bool,
    /// The title changed since the window last took it.
    pub title_changed: bool,
    /// The program copied text to the clipboard (OSC 52) since the window
    /// last took it.
    pub clipboard_changed: bool,
    /// The program rang the bell since its tab was last hidden, or since it
    /// started.
    pub bell_rang: bool,
    /// The window knows when the synchronized update that the program has
    /// open ends, and ends it then, even while it is minimized.
    sync_noted: bool,
    /// The program ended, or could not start, and the window has not taken
    /// it yet.
    exit: Option<Exit>,
}

/// How the program of a session ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    /// With this exit code.
    Code(u32),
    /// It was still running some seconds after ConPTY closed its output,
    /// which only happens if ConPTY itself fails.
    Unknown,
    /// It could not start: the writer of the session could not create the
    /// pseudoconsole or the program (ADR 0028).
    NotStarted,
}

impl Exit {
    /// Whether it is a clean end, which closes the tab: only the exit code
    /// 0, as in Windows Terminal (ADR 0024). An end without an exit code, or
    /// a program that could not start, is not one.
    pub const fn is_clean(self) -> bool {
        matches!(self, Self::Code(0))
    }
}

impl Shared {
    pub const fn new(terminal: Terminal) -> Self {
        Self {
            terminal,
            screen_changed: false,
            title_changed: false,
            clipboard_changed: false,
            bell_rang: false,
            sync_noted: false,
            exit: None,
        }
    }

    /// Notes that the terminal read output, with `changes`, and says whether
    /// the window has to be woken up for it.
    pub fn note(&mut self, changes: Changes) -> bool {
        let screen = !mem::replace(&mut self.screen_changed, true);
        let others = self.note_others(changes);
        // Until the update ends, the terminal keeps what the program writes,
        // its questions too: the window has to end it in time, even if it
        // does not copy the screen.
        let sync =
            self.terminal.sync_deadline().is_some() && !mem::replace(&mut self.sync_noted, true);
        screen || others || sync
    }

    /// Notes the changes of the title, of the clipboard and the bell, and
    /// says whether the window has to be woken up for them.
    fn note_others(&mut self, changes: Changes) -> bool {
        let title = changes.title && !mem::replace(&mut self.title_changed, true);
        let clipboard = changes.clipboard && !mem::replace(&mut self.clipboard_changed, true);
        let bell = changes.bell && !mem::replace(&mut self.bell_rang, true);
        title || clipboard || bell
    }

    /// Ends the synchronized update that the program has open if its time is
    /// up at `now`, as [`Shared::end_sync`] does. Returns when the update
    /// that is still open ends, which the window knows from now on.
    pub fn end_late_sync(&mut self, now: Instant, answers: &mut Vec<u8>) -> Option<Instant> {
        if self
            .terminal
            .sync_deadline()
            .is_some_and(|deadline| deadline <= now)
        {
            self.end_sync(answers);
        }
        let deadline = self.terminal.sync_deadline();
        self.sync_noted = deadline.is_some();
        deadline
    }

    /// Ends the synchronized update that the program has open, if it has
    /// one: what it wrote meanwhile is shown as it is, and its answers go to
    /// `answers`. A title, or text for the clipboard, that it gave meanwhile
    /// waits for the window to take it, and a bell that it rang meanwhile
    /// is noted for its tab (ADR 0025), without waking the window up: the
    /// window is the one that ends the update, or the end of the program,
    /// which wakes it up anyway. None is left open, so the next one that the
    /// program leaves open wakes the window up again.
    pub fn end_sync(&mut self, answers: &mut Vec<u8>) {
        if self.terminal.sync_deadline().is_some() {
            let changes = self.terminal.end_sync();
            if changes.responses {
                self.terminal.take_responses(answers);
            }
            self.note_others(changes);
        }
        self.sync_noted = false;
    }

    /// Takes the title of the terminal into `title`, if it changed since the
    /// window last took it. Returns whether it is another one.
    pub fn take_title(&mut self, title: &mut Option<String>) -> bool {
        mem::take(&mut self.title_changed) && store_title(title, self.terminal.title())
    }

    /// Takes the text that the program copied to the clipboard (OSC 52), if
    /// it copied some since the window last took it.
    pub fn take_clipboard(&mut self) -> Option<String> {
        mem::take(&mut self.clipboard_changed)
            .then(|| self.terminal.take_clipboard())
            .flatten()
    }

    /// Notes that the program ended with `exit_code`, or without one, which
    /// the window takes with [`Shared::take_exit`]. If it is not a clean
    /// end, the terminal shows how it ended, and what the user can do then
    /// (ADR 0024); a clean end closes the tab, so it shows nothing.
    fn end(&mut self, exit_code: Option<u32>) {
        let exit = exit_code.map_or(Exit::Unknown, Exit::Code);
        if !exit.is_clean() {
            self.tell(&exit_line(exit_code));
        }
        self.exit = Some(exit);
    }

    /// Notes that the program could not start, which the window takes with
    /// [`Shared::take_exit`], as an end that is not clean: the terminal
    /// shows `failure`, why, and what the user can do then (ADR 0024 and
    /// ADR 0028).
    fn fail(&mut self, failure: &str) {
        self.tell(failure);
        self.exit = Some(Exit::NotStarted);
    }

    /// Shows `failure`, why the program of the tab could not start, and
    /// what the user can do then, when the session tells it at once,
    /// without a reader: the tab knows it already.
    pub fn not_started(&mut self, failure: &str) {
        self.tell(failure);
    }

    /// Shows `line`, how the program ended or why it could not start, as
    /// [`ending`] says, in a row of its own, also when the view shows the
    /// history, below what the program left in a synchronized update, which
    /// the terminal shows first. The program has ended: its answers are
    /// dropped.
    fn tell(&mut self, line: &str) {
        self.end_sync(&mut Vec::new());
        let new_line = !self.terminal.at_empty_row_start();
        let changes = self.terminal.feed(ending(line, new_line).as_bytes());
        self.note(changes);
    }

    /// How the program ended, if it ended since the window last took it.
    pub fn take_exit(&mut self) -> Option<Exit> {
        self.exit.take()
    }

    /// Leaves the terminal as a new program expects it, for the session
    /// that starts again in its tab, as [`Terminal::restart`] says. The
    /// title, the text for the clipboard and the end that the last program
    /// gave, and that the window did not take, are dropped. The window
    /// forgets the screen and the bell too, as when the tab is hidden: it
    /// copies the whole screen when it shows the tab, at once if the tab is
    /// shown, and the first output and the first bell of the new program
    /// wake it up, once, also while the tab is hidden, so that its tab
    /// tells them (ADR 0025).
    pub fn restart(&mut self) {
        self.terminal.restart();
        self.screen_changed = false;
        self.title_changed = false;
        self.clipboard_changed = false;
        self.bell_rang = false;
        self.sync_noted = false;
        self.exit = None;
    }
}

/// Puts `new` in `title`, in the same memory if it can. Returns whether it
/// is another one.
pub fn store_title(title: &mut Option<String>, new: Option<&str>) -> bool {
    if title.as_deref() == new {
        return false;
    }
    match new {
        Some(new) => {
            let title = title.get_or_insert_with(String::new);
            title.clear();
            title.push_str(new);
        }
        None => *title = None,
    }
    true
}

/// Locks what the reader thread and the window share. Neither of them
/// leaves it half changed if it panics: the window can still show it.
pub fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the reader thread of a session wakes up: the window, with the tab of
/// the session.
pub trait Wake: Clone + Send + 'static {
    fn wake(&self, tab: TabId);
}

impl Wake for Waker {
    fn wake(&self, tab: TabId) {
        Waker::wake(self, tab.get());
    }
}

/// Runs what `launch` says in a session of `size`, with the variables of
/// `environment`, an environment block, and gives its output to the
/// terminal of `shared`. Its reader thread wakes `waker` up with the
/// identifier of the tab of the session, `tab`.
///
/// It returns at once: the writer thread of the session creates the
/// pseudoconsole and the program, in the ConPTY next to meiga or else in
/// the one of Windows, which the first writer loads (ADR 0028). If the
/// program cannot start, the reader tells it, as the end of a program, and
/// the terminal shows why: which program, in which folder, and why. Only
/// when the session itself cannot start, for example because Windows cannot
/// give it its threads, it returns that, which the terminal shows with
/// [`Shared::not_started`].
pub fn start<W: Wake>(
    launch: &Launch,
    environment: &[u16],
    size: GridSize,
    shared: &Arc<Mutex<Shared>>,
    waker: &W,
    tab: TabId,
) -> Result<Pty, String> {
    let reader = Reader {
        shared: Arc::clone(shared),
        waker: waker.clone(),
        tab,
        answers: Vec::new(),
        launch: launch.clone(),
    };
    let command = Command {
        line: &launch.command,
        folder: launch.folder.as_deref(),
        environment: Some(environment),
    };
    conpty::spawn(&command, size, reader).map_err(|error| failure(launch, &error))
}

/// Why `launch` could not start, because of `error`, as the terminal shows
/// it.
fn failure(launch: &Launch, error: &Error) -> String {
    start_error(
        launch,
        error,
        error.os_error().and_then(io::Error::raw_os_error),
    )
}

/// Why `launch` could not start: which program, in which folder, and
/// `error`, whose code of Windows is `code`. When the folder is not there,
/// a second line says so, as Windows Terminal does.
fn start_error(launch: &Launch, error: &dyn fmt::Display, code: Option<i32>) -> String {
    let command = &launch.command;
    match &launch.folder {
        Some(folder) if code == Some(ERROR_DIRECTORY) => format!(
            "[could not start `{command}` in `{folder}`: {error}]\r\n\
             Could not access starting directory \"{folder}\""
        ),
        Some(folder) => format!("[could not start `{command}` in `{folder}`: {error}]"),
        None => format!("[could not start `{command}`: {error}]"),
    }
}

/// What the user can do when the program of a tab ended without a clean
/// end, or could not start, as Windows Terminal says it.
const HINT: &str = "You can now close this tab with Ctrl+D, or press Enter to restart.";

/// Gives the output of the program to the terminal, on the reader thread of
/// the session, and answers the program.
struct Reader<W> {
    shared: Arc<Mutex<Shared>>,
    waker: W,
    /// The tab of the session, which the window learns with each wake-up.
    tab: TabId,
    /// The answers for the program, kept to avoid an allocation each time.
    answers: Vec<u8>,
    /// What the session runs, which the terminal tells if it cannot start.
    launch: Launch,
}

impl<W: Wake> Output for Reader<W> {
    fn received(&mut self, bytes: &[u8], input: &Input) {
        let wake = {
            let mut shared = lock(&self.shared);
            let changes = shared.terminal.feed(bytes);
            if changes.responses {
                shared.terminal.take_responses(&mut self.answers);
            }
            shared.note(changes)
        };
        // Outside the lock: the window never waits for the input pipe.
        if !self.answers.is_empty() {
            input.write(&self.answers);
            self.answers.clear();
        }
        if wake {
            self.waker.wake(self.tab);
        }
    }

    fn ended(&mut self, exit_code: Option<u32>) {
        lock(&self.shared).end(exit_code);
        // Always, also when the window was already woken up for the screen
        // and did not copy it, as in a hidden tab: it closes the tab, or
        // forgets its session. It is the last thing that this thread does.
        self.waker.wake(self.tab);
    }

    fn failed(&mut self, error: Error) {
        let failure = failure(&self.launch, &error);
        lock(&self.shared).fail(&failure);
        // As at the end of a program: the window forgets the session, and
        // the tab tells it.
        self.waker.wake(self.tab);
    }
}

/// The line of the end of a program, as Windows Terminal says it, with its
/// exit code, if it has one.
fn exit_line(exit_code: Option<u32>) -> String {
    match exit_code {
        Some(code) => format!("[process exited with code {code} (0x{code:08x})]"),
        None => "[process exited]".to_owned(),
    }
}

/// What the terminal shows when the program ends without a clean end, or
/// cannot start, as Windows Terminal: `line`, how it ended or why it did not
/// start, in a row of its own if `new_line`, with the default colours and
/// characters, and below it what the user can do then. What the program, or
/// ConPTY, left does not change how it reads. The screen is erased from the
/// row of the line to its end (`ESC [ J`), unlike Windows Terminal: what
/// the program left below the cursor, as a full-screen program that ends at
/// once, does not mix with them, and the insert mode only moves blank
/// cells. The line-drawing characters are turned off, and the wrapping on.
/// A scrolling region goes back to the whole screen, between `ESC 7` and
/// `ESC 8`, which keep the cursor where the program left it: below a
/// region, the last row does not scroll, and the hint would cover the line.
/// The cursor is hidden: nothing can be typed any more. The ConPTY does not
/// turn off the mouse nor the bracketed paste of a program that ends: they
/// are turned off, so that the mouse selects text again.
fn ending(line: &str, new_line: bool) -> String {
    const READABLE: &str = "\x1b[0m\x1b(B\x0f\x1b[?7h\x1b7\x1b[r\x1b8";
    const RESET: &str = "\x1b[?1000;1002;1003;1005;1006;2004l\x1b[?25l";
    let place = if new_line { "\r\n\x1b[J" } else { "\x1b[J" };
    format!("{READABLE}{place}{line}\r\n{HINT}{RESET}")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use meiga_core::term::{CursorShape, Screen, Scroll};

    use super::*;

    #[test]
    fn the_window_is_woken_up_once_until_it_takes_what_changed() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(10, 2)));
        let title = Changes {
            title: true,
            ..Changes::default()
        };
        assert!(shared.note(Changes::default()));
        assert!(!shared.note(Changes::default()));
        // The title is something else.
        assert!(shared.note(title));
        assert!(!shared.note(title));
        // Once the window copies the screen, the output wakes it up again.
        shared.screen_changed = false;
        assert!(shared.note(title));
        shared.title_changed = false;
        assert!(shared.note(title));
    }

    #[test]
    fn a_synchronized_update_wakes_the_window_once_and_ends_at_its_deadline() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(10, 2)));
        // The window was woken up for the screen and did not copy it, as
        // while it is minimized.
        shared.screen_changed = true;
        let changes = shared.terminal.feed(b"\x1b[?2026h\x1b]0;sync\x07\x1b[c");
        assert!(!changes.title && !changes.responses);
        assert!(shared.note(changes));
        assert!(!shared.note(Changes::default()));

        let deadline = shared.terminal.sync_deadline().unwrap();
        let mut answers = Vec::new();
        let early = deadline - Duration::from_millis(1);
        assert_eq!(shared.end_late_sync(early, &mut answers), Some(deadline));
        assert!(answers.is_empty());
        // At its deadline, the title and the answer to the question come
        // out, and there is nothing left to wait for. The title waits for
        // the window, which ended the update.
        assert_eq!(shared.end_late_sync(deadline, &mut answers), None);
        assert_eq!(answers, b"\x1b[?6c");
        assert_eq!(shared.terminal.title(), Some("sync"));
        assert!(shared.title_changed);

        // The next update wakes the window up again.
        let changes = shared.terminal.feed(b"\x1b[?2026h");
        assert!(shared.note(changes));
    }

    #[test]
    fn a_synchronized_update_can_end_before_its_deadline() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(10, 2)));
        shared.screen_changed = true;
        let changes = shared.terminal.feed(b"\x1b[?2026h\x1b]52;c;b2xh\x07\x1b[c");
        assert!(!changes.clipboard && !changes.responses);
        assert!(shared.note(changes));
        // As when its tab is shown: the text for the clipboard and the
        // answer to the question come out now, and the window, which ended
        // the update, takes the text.
        let mut answers = Vec::new();
        shared.end_sync(&mut answers);
        assert_eq!(answers, b"\x1b[?6c");
        assert_eq!(shared.terminal.sync_deadline(), None);
        assert_eq!(shared.take_clipboard().as_deref(), Some("ola"));
        // The next update wakes the window up again.
        let changes = shared.terminal.feed(b"\x1b[?2026h");
        assert!(shared.note(changes));
    }

    #[test]
    fn the_end_of_the_program_is_on_a_row_of_its_own_with_what_to_do_and_without_the_cursor() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(70, 4)));
        shared.screen_changed = false;
        shared.terminal.feed(b"\x1b[31mola\x1b[?1002;1006;2004h");
        shared.end(Some(0xC000_013A));
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(0), "ola");
        assert_eq!(
            screen.row_text(1),
            "[process exited with code 3221225786 (0xc000013a)]"
        );
        assert_eq!(screen.row_text(2), HINT);
        assert_eq!(screen.row(1)[0].fg, meiga_core::term::Color::Default);
        assert_eq!(screen.cursor().shape, CursorShape::Hidden);
        // The mouse selects text again.
        let modes = shared.terminal.modes();
        assert!(!modes.mouse_drag && !modes.sgr_mouse && !modes.bracketed_paste);
        // The window takes the end once.
        assert!(shared.screen_changed);
        assert_eq!(shared.take_exit(), Some(Exit::Code(0xC000_013A)));
        assert_eq!(shared.take_exit(), None);

        assert_eq!(
            ending(&exit_line(Some(3)), false),
            "\x1b[0m\x1b(B\x0f\x1b[?7h\x1b7\x1b[r\x1b8\x1b[J\
             [process exited with code 3 (0x00000003)]\r\n\
             You can now close this tab with Ctrl+D, or press Enter to restart.\
             \x1b[?1000;1002;1003;1005;1006;2004l\x1b[?25l"
        );
        assert_eq!(
            ending(&exit_line(None), true),
            "\x1b[0m\x1b(B\x0f\x1b[?7h\x1b7\x1b[r\x1b8\r\n\x1b[J[process exited]\r\n\
             You can now close this tab with Ctrl+D, or press Enter to restart.\
             \x1b[?1000;1002;1003;1005;1006;2004l\x1b[?25l"
        );
    }

    #[test]
    fn the_end_goes_below_what_the_program_left_in_a_synchronized_update() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(70, 4)));
        // Until the update ends, the cursor seems to be at the start of an
        // empty row, and the question waits for its answer.
        shared.terminal.feed(b"ola\r\n\x1b[?2026hadeus\x1b[c");
        shared.end(Some(1));
        assert_eq!(shared.terminal.sync_deadline(), None);
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(1), "adeus");
        assert_eq!(
            screen.row_text(2),
            "[process exited with code 1 (0x00000001)]"
        );
        assert_eq!(screen.row_text(3), HINT);
        // Nobody answers a program that has ended.
        let mut answers = Vec::new();
        shared.terminal.take_responses(&mut answers);
        assert!(answers.is_empty());
    }

    #[test]
    fn the_end_reads_well_whatever_the_program_left() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(30, 6)));
        // Rows with text below the cursor, longer than the line and the end
        // of the hint, the line-drawing characters in the two sets, the
        // second one in use, the insert mode, no wrapping, and a scrolling
        // region of the first two rows, with the cursor at the end of the
        // second one: a new line there would scroll the region.
        shared.terminal.feed(
            b"ola\r\nabc\r\nabcdefghijklmnopqrstuvwxyz\r\n\r\n\r\n0123456789\
              \x1b[1;2r\x1b[2;4H\x1b(0\x1b)0\x0e\x1b[4h\x1b[?7l",
        );
        shared.end(None);
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        let rows: Vec<String> = (0..6).map(|row| screen.row_text(row)).collect();
        assert_eq!(
            rows,
            [
                "ola",
                "abc",
                "[process exited]",
                &HINT[..30],
                &HINT[30..60],
                &HINT[60..]
            ]
        );
    }

    #[test]
    fn only_the_exit_code_0_is_a_clean_end_which_shows_nothing() {
        assert!(Exit::Code(0).is_clean());
        for exit in [
            Exit::Code(1),
            Exit::Code(0xC000_013A),
            Exit::Unknown,
            Exit::NotStarted,
        ] {
            assert!(!exit.is_clean(), "{exit:?}");
        }
        let mut shared = Shared::new(Terminal::new(GridSize::new(70, 3)));
        shared.terminal.feed(b"ola");
        shared.end(Some(0));
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(0), "ola");
        assert_eq!(screen.row_text(1), "");
        assert_eq!(shared.take_exit(), Some(Exit::Code(0)));
    }

    #[test]
    fn the_end_is_on_a_row_of_its_own_also_with_the_view_in_the_history() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(70, 3)));
        shared.terminal.feed(b"1\r\n2\r\n3\r\n4\r\nprompt>");
        shared.terminal.scroll(Scroll::Top);
        shared.end(None);
        shared.terminal.scroll(Scroll::Bottom);
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(0), "prompt>");
        assert_eq!(screen.row_text(1), "[process exited]");
        assert_eq!(screen.row_text(2), HINT);
    }

    #[test]
    fn a_program_that_cannot_start_tells_why_and_its_folder_if_it_is_not_there() {
        let mut launch = Launch::command_line("non-existent");
        let error = "CreateProcessW failed";
        assert_eq!(
            start_error(&launch, &error, Some(2)),
            "[could not start `non-existent`: CreateProcessW failed]"
        );
        // Without a folder of its own, the folder of meiga is there.
        assert_eq!(
            start_error(&launch, &error, Some(ERROR_DIRECTORY)),
            "[could not start `non-existent`: CreateProcessW failed]"
        );
        launch.folder = Some(r"C:\non-existent".to_owned());
        assert_eq!(
            start_error(&launch, &error, None),
            r"[could not start `non-existent` in `C:\non-existent`: CreateProcessW failed]"
        );
        assert_eq!(
            start_error(&launch, &error, Some(2)),
            r"[could not start `non-existent` in `C:\non-existent`: CreateProcessW failed]"
        );
        // ERROR_DIRECTORY: the folder is not there, as Windows Terminal says.
        assert_eq!(
            start_error(&launch, &error, Some(ERROR_DIRECTORY)),
            "[could not start `non-existent` in `C:\\non-existent`: CreateProcessW failed]\r\n\
             Could not access starting directory \"C:\\non-existent\""
        );
    }

    #[test]
    fn a_failure_is_on_a_row_of_its_own_with_what_to_do_and_the_window_takes_it() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(80, 5)));
        // What ConPTY may have written before the program failed: a mode,
        // and a question that waits in a synchronized update.
        shared
            .terminal
            .feed(b"\x1b[?2004h\x1b[31mola\x1b[?2026h\x1b[c");
        shared.screen_changed = false;
        shared.fail("[could not start `x` in `C:\\y`: why]\r\nCould not access starting directory \"C:\\y\"");
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(0), "ola");
        assert_eq!(screen.row_text(1), "[could not start `x` in `C:\\y`: why]");
        assert_eq!(
            screen.row_text(2),
            "Could not access starting directory \"C:\\y\""
        );
        assert_eq!(screen.row_text(3), HINT);
        assert_eq!(screen.row(1)[0].fg, meiga_core::term::Color::Default);
        assert_eq!(screen.cursor().shape, CursorShape::Hidden);
        assert!(!shared.terminal.modes().bracketed_paste);
        // Nobody answers a program that did not start.
        let mut answers = Vec::new();
        shared.terminal.take_responses(&mut answers);
        assert!(answers.is_empty());
        // The window takes it once, as an end that is not clean.
        assert!(shared.screen_changed);
        assert_eq!(shared.take_exit(), Some(Exit::NotStarted));
        assert_eq!(shared.take_exit(), None);
    }

    #[test]
    fn a_session_that_cannot_start_shows_it_without_an_end_for_the_window() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(70, 3)));
        shared.not_started("[could not start `x`: why]");
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(0), "[could not start `x`: why]");
        assert_eq!(screen.row_text(1), HINT);
        assert_eq!(screen.cursor().shape, CursorShape::Hidden);
        // The tab knows it: it has no session.
        assert_eq!(shared.take_exit(), None);
    }

    #[test]
    fn a_restart_drops_what_the_last_program_left() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(70, 3)));
        shared.screen_changed = true;
        let changes = shared
            .terminal
            .feed(b"\x1b]0;vello\x07\x1b]52;c;b2xh\x07\x07\x1b[?2026h");
        assert!(shared.note(changes));
        shared.end(Some(1));
        // The end already forgot the update; the restart forgets it too,
        // whatever led to it.
        shared.sync_noted = true;
        shared.restart();
        assert!(!shared.title_changed && !shared.clipboard_changed && !shared.sync_noted);
        assert_eq!(shared.take_exit(), None);
        assert_eq!(shared.take_clipboard(), None);
        assert_eq!(shared.terminal.title(), None);
        // The window copies the whole screen when it shows the tab: the
        // first output of the new program, and its first bell, wake the
        // window up again, as the next synchronized update.
        assert!(!shared.screen_changed && !shared.bell_rang);
        let changes = shared.terminal.feed(b"\x07");
        assert!(changes.bell);
        assert!(shared.note(changes));
        assert!(shared.screen_changed && shared.bell_rang);
        let changes = shared.terminal.feed(b"\x1b[?2026h");
        assert!(shared.note(changes));
    }

    #[test]
    fn the_bell_wakes_the_window_once_until_the_tab_is_hidden_again() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(10, 2)));
        // The window was woken up for the screen and did not copy it, as
        // while the tab is hidden.
        shared.screen_changed = true;
        let bell = shared.terminal.feed(b"\x07");
        assert!(bell.bell);
        assert!(shared.note(bell));
        assert!(!shared.note(bell));
        // Copying the screen does not change it: only hiding the tab again.
        shared.screen_changed = false;
        shared.note(Changes::default());
        assert!(!shared.note(bell));
        shared.bell_rang = false;
        assert!(shared.note(bell));
        // A bell in a synchronized update that the window ends rings then,
        // without waking it up: the window learns it there.
        shared.bell_rang = false;
        let changes = shared.terminal.feed(b"\x1b[?2026h\x07");
        assert!(!changes.bell);
        assert!(shared.note(changes));
        shared.end_sync(&mut Vec::new());
        assert!(shared.bell_rang);
    }

    #[test]
    fn the_clipboard_of_the_program_wakes_the_window_once() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(10, 2)));
        shared.screen_changed = true;
        let changes = shared.terminal.feed(b"\x1b]52;c;b2xh\x07");
        assert!(changes.clipboard);
        assert!(shared.note(changes));
        assert!(!shared.note(changes));
        assert!(shared.clipboard_changed);
        assert_eq!(shared.take_clipboard().as_deref(), Some("ola"));
        assert!(!shared.clipboard_changed);
        assert_eq!(shared.take_clipboard(), None);
    }

    #[test]
    fn the_title_is_taken_once_and_only_another_one_counts() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(10, 2)));
        let mut title = None;
        assert!(!shared.take_title(&mut title));
        let changes = shared.terminal.feed(b"\x1b]0;ola\x07");
        assert!(shared.note(changes));
        assert!(shared.take_title(&mut title));
        assert_eq!(title.as_deref(), Some("ola"));
        assert!(!shared.title_changed);
        assert!(!shared.take_title(&mut title));
        // The same title again is not a change: nothing to take.
        let changes = shared.terminal.feed(b"\x1b]0;ola\x07");
        assert!(!changes.title);
        shared.note(changes);
        assert!(!shared.title_changed);
        assert!(!shared.take_title(&mut title));
        // Another one and back before the window takes it: it changed, but
        // it is not another one.
        let changes = shared.terminal.feed(b"\x1b]0;adeus\x07\x1b]0;ola\x07");
        shared.note(changes);
        assert!(shared.title_changed);
        assert!(!shared.take_title(&mut title));
        assert!(!shared.title_changed);
        assert_eq!(title.as_deref(), Some("ola"));
        // An empty title is no title.
        let changes = shared.terminal.feed(b"\x1b]0;\x07");
        shared.note(changes);
        assert!(shared.take_title(&mut title));
        assert_eq!(title, None);
    }

    /// Notes the tabs that the readers woke the window up for.
    #[derive(Clone, Default)]
    struct Woken(Arc<Mutex<Vec<TabId>>>);

    impl Wake for Woken {
        fn wake(&self, tab: TabId) {
            self.0
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(tab);
        }
    }

    impl Woken {
        /// Whether a reader woke the window up, and only for the tab 7.
        fn only_for_7(&self) -> bool {
            let woken = self.0.lock().unwrap_or_else(PoisonError::into_inner);
            !woken.is_empty() && woken.iter().all(|&tab| tab == TabId::from_raw(7))
        }
    }

    /// Runs `launch` in the session of the tab 7, with a terminal of 200 x 6,
    /// which wakes `woken` up, and waits until the window would learn how it
    /// ended. Returns that, the rows of the terminal, and the session.
    fn run(launch: &Launch, woken: &Woken) -> (Exit, Vec<String>, Pty) {
        let size = GridSize::new(200, 6);
        let shared = Arc::new(Mutex::new(Shared::new(Terminal::new(size))));
        let environment = meiga_win::profiles::environment().block();
        let pty = start(
            launch,
            &environment,
            size,
            &shared,
            woken,
            TabId::from_raw(7),
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        let exit = loop {
            if let Some(exit) = lock(&shared).take_exit() {
                break exit;
            }
            assert!(Instant::now() < deadline, "the session did not end");
            std::thread::sleep(Duration::from_millis(10));
        };
        let mut screen = Screen::new();
        lock(&shared).terminal.copy_damage(&mut screen);
        let rows = (0..6).map(|row| screen.row_text(row)).collect();
        (exit, rows, pty)
    }

    #[test]
    fn the_reader_gives_the_output_to_the_terminal_and_tells_the_end() {
        let woken = Woken::default();
        let launch = Launch::command_line("cmd /d /c echo ola& exit 3");
        let (exit, rows, pty) = run(&launch, &woken);
        assert_eq!(exit, Exit::Code(3));
        let ola = rows.iter().position(|row| row == "ola");
        let end = rows
            .iter()
            .position(|row| row == "[process exited with code 3 (0x00000003)]");
        assert!(ola.is_some() && ola < end, "{rows:?}");
        assert!(pty.process_id().is_some());
        assert!(woken.only_for_7());
    }

    /// An output that drops what the session sends.
    struct Dropped;

    impl Output for Dropped {
        fn received(&mut self, _bytes: &[u8], _input: &Input) {}
        fn ended(&mut self, _exit_code: Option<u32>) {}
        fn failed(&mut self, _error: Error) {}
    }

    #[test]
    fn the_reader_answers_the_program_and_wakes_the_window_once_for_the_screen() {
        // What another session sends, given to the reader by hand, with the
        // input of a session whose program waits for a line.
        let size = GridSize::new(80, 5);
        let pty = conpty::spawn(&Command::new("cmd /d /c set /p M="), size, Dropped).unwrap();
        let input = pty.input();
        let shared = Arc::new(Mutex::new(Shared::new(Terminal::new(size))));
        let woken = Woken::default();
        let mut reader = Reader {
            shared: Arc::clone(&shared),
            waker: woken.clone(),
            tab: TabId::from_raw(7),
            answers: Vec::new(),
            launch: Launch::command_line("cmd /d /c set /p M="),
        };
        // A question, the attributes of the terminal, which the reader
        // answers at once, and then more text, which does not wake the
        // window up again until it copies the screen.
        reader.received(b"ola\x1b[c", &input);
        assert!(reader.answers.is_empty());
        reader.received(b" meiga", &input);
        assert!(woken.only_for_7());
        assert_eq!(
            woken.0.lock().unwrap_or_else(PoisonError::into_inner).len(),
            1
        );
        let mut screen = Screen::new();
        lock(&shared).terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(0), "ola meiga");
        // Nothing waits for an answer any more.
        let mut answers = Vec::new();
        lock(&shared).terminal.take_responses(&mut answers);
        assert!(answers.is_empty());
    }

    #[test]
    fn a_program_that_cannot_start_is_told_by_the_reader_with_its_folder() {
        let woken = Woken::default();
        let mut launch = Launch::command_line("cmd /d /c exit");
        launch.folder = Some(r"C:\meiga\no such folder".to_owned());
        let (exit, rows, pty) = run(&launch, &woken);
        assert_eq!(exit, Exit::NotStarted);
        let first = rows
            .iter()
            .position(|row| row.starts_with("[could not start "))
            .unwrap_or_else(|| panic!("{rows:?}"));
        assert!(
            rows[first].starts_with(
                r"[could not start `cmd /d /c exit` in `C:\meiga\no such folder`: CreateProcessW failed: "
            ),
            "{rows:?}"
        );
        assert_eq!(
            rows[first + 1],
            r#"Could not access starting directory "C:\meiga\no such folder""#
        );
        assert_eq!(rows[first + 2], HINT);
        assert_eq!(pty.process_id(), None);
        assert!(woken.only_for_7());
    }
}
