//! The session of a tab: the program that runs in a pseudoconsole, the
//! reader thread that gives its output to the terminal of the tab, and what
//! that thread shares with the window.

use std::mem;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use meiga_core::geometry::GridSize;
use meiga_core::model::TabId;
use meiga_core::profile::Launch;
use meiga_core::term::{Changes, Terminal};
use meiga_win::Error;
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
/// synchronized update that its program has not ended yet (ADR 0020).
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
}

impl Shared {
    pub const fn new(terminal: Terminal) -> Self {
        Self {
            terminal,
            screen_changed: false,
            title_changed: false,
            clipboard_changed: false,
            sync_noted: false,
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

/// What the terminal shows when `launch` cannot start: why, without the
/// cursor, since nothing can be typed.
fn start_error(launch: &Launch, error: &Error) -> String {
    let command = &launch.command;
    match &launch.folder {
        Some(folder) => format!("[could not start `{command}` in `{folder}`: {error}]\x1b[?25l"),
        None => format!("[could not start `{command}`: {error}]\x1b[?25l"),
    }
}

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
        let wake = {
            let mut shared = lock(&self.shared);
            let new_line = shared.terminal.cursor().column > 0;
            let changes = shared.terminal.feed(end(exit_code, new_line).as_bytes());
            shared.note(changes)
        };
        if wake {
            self.waker.wake(self.tab.get());
        }
    }
}

/// What the terminal shows when the program ends, as Windows Terminal: a
/// line with its exit code, in a row of its own if `new_line`, with the
/// default colours. The cursor is hidden: nothing can be typed any more.
/// The ConPTY does not turn off the mouse nor the bracketed paste of a
/// program that ends: they are turned off, so that the mouse selects text
/// again.
fn end(exit_code: Option<u32>, new_line: bool) -> String {
    const RESET: &str = "\x1b[?1000;1002;1003;1005;1006;2004l\x1b[?25l";
    let new_line = if new_line { "\r\n" } else { "" };
    match exit_code {
        Some(code) => {
            format!("\x1b[0m{new_line}[process exited with code {code} (0x{code:08x})]{RESET}")
        }
        None => format!("\x1b[0m{new_line}[process exited]{RESET}"),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use meiga_core::term::{CursorShape, Screen};

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
    fn the_end_of_the_program_is_on_a_row_of_its_own_without_the_cursor() {
        let mut terminal = Terminal::new(GridSize::new(50, 3));
        terminal.feed(b"\x1b[31mola\x1b[?1002;1006;2004h");
        terminal.feed(end(Some(0xC000_013A), true).as_bytes());
        let mut screen = Screen::new();
        terminal.copy_damage(&mut screen);
        assert_eq!(screen.row_text(0), "ola");
        assert_eq!(
            screen.row_text(1),
            "[process exited with code 3221225786 (0xc000013a)]"
        );
        assert_eq!(screen.row(1)[0].fg, meiga_core::term::Color::Default);
        assert_eq!(screen.cursor().shape, CursorShape::Hidden);
        // The mouse selects text again.
        let modes = terminal.modes();
        assert!(!modes.mouse_drag && !modes.sgr_mouse && !modes.bracketed_paste);

        assert_eq!(
            end(Some(0), false),
            "\x1b[0m[process exited with code 0 (0x00000000)]\
             \x1b[?1000;1002;1003;1005;1006;2004l\x1b[?25l"
        );
        assert_eq!(
            end(None, false),
            "\x1b[0m[process exited]\x1b[?1000;1002;1003;1005;1006;2004l\x1b[?25l"
        );
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
