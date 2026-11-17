//! The session of a tab: the program that runs in a pseudoconsole, the
//! reader thread that gives its output to the terminal of the tab, and what
//! that thread shares with the window.

use std::fmt;
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use meiga_core::geometry::GridSize;
use meiga_core::model::TabId;
use meiga_core::profile::Launch;
use meiga_core::term::{Changes, Terminal};
use meiga_win::conpty::{Command, Conpty, Input, Output, Pty};
use meiga_win::window::Waker;

/// What the reader thread of the session and the window share, behind a
/// mutex: the terminal, and what the window has been woken up for.
///
/// The reader wakes the window up only if it was not already woken up for
/// the same thing (ADR 0005): for the screen, until the window copies it;
/// for the title and for the clipboard, until the window takes them; and
/// for a synchronized update, until the window knows when it ends. While
/// the window is minimized, or the tab is hidden, the window does not copy
/// the screen, so the output does not wake it up again, except to end a
/// synchronized update in time, and for the title, which the tab bar shows
/// for every tab (ADR 0020). What a hidden tab copied to the clipboard
/// waits until the tab is shown, and is dropped then (ADR 0019), also in a
/// synchronized update that its program has not ended yet (ADR 0020). The
/// end of the program always wakes the window up, once (ADR 0024).
pub struct Shared {
    pub terminal: Terminal,
    /// The screen may have changed since the window last copied it.
    pub screen_changed: bool,
    /// The title changed since the window last took it.
    pub title_changed: bool,
    /// The program copied text to the clipboard (OSC 52) since the window
    /// last took it.
    pub clipboard_changed: bool,
    /// The window knows when the synchronized update that the program has
    /// open ends, and ends it then, even while it is minimized.
    sync_noted: bool,
    /// The program ended, and the window has not taken it yet.
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
}

impl Exit {
    /// Whether it is a clean end, which closes the tab: only the exit code
    /// 0, as in Windows Terminal (ADR 0024). An end without an exit code is
    /// not one.
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

    /// Notes the changes of the title and of the clipboard, and says
    /// whether the window has to be woken up for them.
    fn note_others(&mut self, changes: Changes) -> bool {
        let title = changes.title && !mem::replace(&mut self.title_changed, true);
        let clipboard = changes.clipboard && !mem::replace(&mut self.clipboard_changed, true);
        title || clipboard
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
    /// waits for the window to take it, without waking it up: the window is
    /// the one that ends the update. None is left open, so the next one
    /// that the program leaves open wakes the window up again.
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

    /// Notes that the program ended with `exit`, which the window takes with
    /// [`Shared::take_exit`]. If it is not a clean end, the terminal shows
    /// how it ended, and what the user can do then (ADR 0024); a clean end
    /// closes the tab, so it shows nothing.
    fn end(&mut self, exit: Exit) {
        if !exit.is_clean() {
            // Below what the program left in a synchronized update, which
            // the terminal shows first. The program has ended: its answers
            // are dropped.
            self.end_sync(&mut Vec::new());
            // In a row of its own, also when the view shows the history.
            let new_line = !self.terminal.at_empty_row_start();
            let changes = self.terminal.feed(end(exit, new_line).as_bytes());
            self.note(changes);
        }
        self.exit = Some(exit);
    }

    /// How the program ended, if it ended since the window last took it.
    pub fn take_exit(&mut self) -> Option<Exit> {
        self.exit.take()
    }

    /// Leaves the terminal as a new program expects it, for the session
    /// that starts again in its tab, as [`Terminal::restart`] says. The
    /// title, the text for the clipboard and the end that the last program
    /// gave, and that the window did not take, are dropped. The screen is
    /// left as it was: the window copies all of it when it shows the tab.
    pub fn restart(&mut self) {
        self.terminal.restart();
        self.title_changed = false;
        self.clipboard_changed = false;
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

/// Runs what `launch` says in a session of `size`, with the variables of
/// `environment`, an environment block, and gives its output to the
/// terminal of `shared`. Its reader thread wakes `waker` up with the
/// identifier of the tab of the session, `tab`.
///
/// If the program cannot start, the error is what the terminal shows: which
/// program, in which folder, and why.
pub fn start(
    launch: &Launch,
    environment: &[u16],
    size: GridSize,
    shared: &Arc<Mutex<Shared>>,
    waker: &Waker,
    tab: TabId,
) -> Result<Pty, String> {
    let reader = Reader {
        shared: Arc::clone(shared),
        waker: waker.clone(),
        tab,
        answers: Vec::new(),
    };
    let command = Command {
        line: &launch.command,
        folder: launch.folder.as_deref(),
        environment: Some(environment),
    };
    Conpty::load()
        .spawn(&command, size, reader)
        .map_err(|error| start_error(launch, &error))
}

/// What the terminal shows when `launch` cannot start: why, and below what
/// the user can do then, without the cursor, since nothing can be typed.
fn start_error(launch: &Launch, error: &dyn fmt::Display) -> String {
    let command = &launch.command;
    match &launch.folder {
        Some(folder) => {
            format!("[could not start `{command}` in `{folder}`: {error}]\r\n{HINT}\x1b[?25l")
        }
        None => format!("[could not start `{command}`: {error}]\r\n{HINT}\x1b[?25l"),
    }
}

/// What the user can do when the program of a tab ended without a clean
/// end, or could not start, as Windows Terminal says it.
const HINT: &str = "You can now close this tab with Ctrl+D, or press Enter to restart.";

/// Gives the output of the program to the terminal, on the reader thread of
/// the session, and answers the program.
struct Reader {
    shared: Arc<Mutex<Shared>>,
    waker: Waker,
    /// The tab of the session, which the window learns with each wake-up.
    tab: TabId,
    /// The answers for the program, kept to avoid an allocation each time.
    answers: Vec<u8>,
}

impl Output for Reader {
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
            self.waker.wake(self.tab.get());
        }
    }

    fn ended(&mut self, exit_code: Option<u32>) {
        let exit = exit_code.map_or(Exit::Unknown, Exit::Code);
        lock(&self.shared).end(exit);
        // Always, also when the window was already woken up for the screen
        // and did not copy it, as in a hidden tab: it closes the tab, or
        // forgets its session. It is the last thing that this thread does.
        self.waker.wake(self.tab.get());
    }
}

/// What the terminal shows when the program ends without a clean end, as
/// Windows Terminal: a line with its exit code, in a row of its own if
/// `new_line`, with the default colours and characters, and below it what
/// the user can do then. Neither the line-drawing characters, the insert
/// mode nor a screen without wrapping that the program left change how it
/// reads; they do not move the cursor. The cursor is hidden: nothing can be
/// typed any more. The ConPTY does not turn off the mouse nor the bracketed
/// paste of a program that ends: they are turned off, so that the mouse
/// selects text again.
fn end(exit: Exit, new_line: bool) -> String {
    const READABLE: &str = "\x1b[0m\x1b(B\x0f\x1b[4l\x1b[?7h";
    const RESET: &str = "\x1b[?1000;1002;1003;1005;1006;2004l\x1b[?25l";
    let new_line = if new_line { "\r\n" } else { "" };
    match exit {
        Exit::Code(code) => format!(
            "{READABLE}{new_line}[process exited with code {code} (0x{code:08x})]\r\n{HINT}{RESET}"
        ),
        Exit::Unknown => format!("{READABLE}{new_line}[process exited]\r\n{HINT}{RESET}"),
    }
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
        shared.end(Exit::Code(0xC000_013A));
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
            end(Exit::Code(3), false),
            "\x1b[0m\x1b(B\x0f\x1b[4l\x1b[?7h[process exited with code 3 (0x00000003)]\r\n\
             You can now close this tab with Ctrl+D, or press Enter to restart.\
             \x1b[?1000;1002;1003;1005;1006;2004l\x1b[?25l"
        );
        assert_eq!(
            end(Exit::Unknown, true),
            "\x1b[0m\x1b(B\x0f\x1b[4l\x1b[?7h\r\n[process exited]\r\n\
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
        shared.end(Exit::Code(1));
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
    fn the_end_reads_well_whatever_characters_and_modes_the_program_left() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(30, 6)));
        // The line-drawing characters in the two sets, the second one in
        // use, the insert mode and no wrapping, with the cursor above a row
        // with text.
        shared
            .terminal
            .feed(b"ola\r\nxyz\x1b[1;4H\x1b(0\x1b)0\x0e\x1b[4h\x1b[?7l");
        shared.end(Exit::Unknown);
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        let rows: Vec<String> = (0..6).map(|row| screen.row_text(row)).collect();
        assert_eq!(
            rows,
            [
                "ola",
                "[process exited]",
                &HINT[..30],
                &HINT[30..60],
                &HINT[60..],
                ""
            ]
        );
    }

    #[test]
    fn only_the_exit_code_0_is_a_clean_end_which_shows_nothing() {
        assert!(Exit::Code(0).is_clean());
        for exit in [Exit::Code(1), Exit::Code(0xC000_013A), Exit::Unknown] {
            assert!(!exit.is_clean(), "{exit:?}");
        }
        let mut shared = Shared::new(Terminal::new(GridSize::new(70, 3)));
        shared.terminal.feed(b"ola");
        shared.end(Exit::Code(0));
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
        shared.end(Exit::Unknown);
        shared.terminal.scroll(Scroll::Bottom);
        let mut screen = Screen::new();
        shared.terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(0), "prompt>");
        assert_eq!(screen.row_text(1), "[process exited]");
        assert_eq!(screen.row_text(2), HINT);
    }

    #[test]
    fn a_program_that_cannot_start_tells_why_and_what_to_do() {
        let mut launch = Launch::command_line("non-existent");
        let error = "CreateProcessW failed";
        assert_eq!(
            start_error(&launch, &error),
            "[could not start `non-existent`: CreateProcessW failed]\r\n\
             You can now close this tab with Ctrl+D, or press Enter to restart.\x1b[?25l"
        );
        launch.folder = Some(r"C:\non-existent".to_owned());
        assert!(
            start_error(&launch, &error)
                .starts_with(r"[could not start `non-existent` in `C:\non-existent`: ")
        );
    }

    #[test]
    fn a_restart_drops_what_the_last_program_left() {
        let mut shared = Shared::new(Terminal::new(GridSize::new(70, 3)));
        shared.screen_changed = true;
        let changes = shared
            .terminal
            .feed(b"\x1b]0;vello\x07\x1b]52;c;b2xh\x07\x1b[?2026h");
        assert!(shared.note(changes));
        shared.end(Exit::Code(1));
        shared.restart();
        assert!(!shared.title_changed && !shared.clipboard_changed);
        assert_eq!(shared.take_exit(), None);
        assert_eq!(shared.take_clipboard(), None);
        assert_eq!(shared.terminal.title(), None);
        // The screen of the window is left as it was, and the next
        // synchronized update wakes the window up again.
        assert!(shared.screen_changed);
        let changes = shared.terminal.feed(b"\x1b[?2026h");
        assert!(shared.note(changes));
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
        // The same title again is taken, but it is not another one.
        let changes = shared.terminal.feed(b"\x1b]0;ola\x07");
        shared.note(changes);
        assert!(!shared.take_title(&mut title));
        assert!(!shared.title_changed);
        // An empty title is no title.
        let changes = shared.terminal.feed(b"\x1b]0;\x07");
        shared.note(changes);
        assert!(shared.take_title(&mut title));
        assert_eq!(title, None);
    }
}
