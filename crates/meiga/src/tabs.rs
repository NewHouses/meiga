//! The tabs of the window: a session for each one, which goes on while the
//! tab is hidden, and the view, which shows the active one (ADR 0019).
//!
//! The reader thread of each session gives the output of its program to
//! the terminal of its tab, also while the tab is hidden, then with a lower
//! priority. It wakes the window up with the identifier of its tab, and only
//! once for each kind of change, until the window takes it (ADR 0005). The
//! window does not copy nor draw the screen of a hidden tab, nor takes its
//! title or its text for the clipboard, so the output of a hidden tab wakes
//! it up at most once for each of them. Only its synchronized updates cannot
//! wait: the window ends them in time, as while it is minimized (ADR 0013).
//! When a tab is shown again, the view copies its whole screen, once.
//!
//! The window never waits for a reader with a lower priority than its own:
//! before it locks the terminal of a tab, it gives the reader of the tab the
//! normal priority, which a hidden tab only keeps while the window holds the
//! lock.

use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use meiga_core::geometry::{GridSize, Rect, Size};
use meiga_core::model::{self, Close, Switch, TabId};
use meiga_core::term::Terminal;
use meiga_render::{Canvas, Font, FontSystem};
use meiga_win::conpty::{Priority, Pty};
use meiga_win::event::HandlerError;
use meiga_win::window::{Pointer, Waker, Window};

use crate::frames::Scene;
use crate::scrollbar::ScrollBar;
use crate::session::{self, Shared, lock};
use crate::view::{self, FONT_FAMILIES, FONT_POINTS, TabTerminal, TerminalView};

/// What a shortcut asks for the tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabAction {
    /// Open a tab with the default shell at the end, and show it.
    New,
    /// Show the next tab or, from the last one, the first.
    Next,
    /// Show the previous tab or, from the first one, the last.
    Previous,
    /// Close the tab that is shown, and its session. The last one closes
    /// the window.
    Close,
}

/// A tab: the terminal of its session, and the session.
struct Tab {
    shared: Arc<Mutex<Shared>>,
    /// It closes when the tab closes. `None` if its program could not
    /// start.
    session: Option<Pty>,
    /// While the tab is hidden, when the synchronized update that its
    /// program has open ends, if the program does not end it first.
    sync_due: Option<Instant>,
}

impl Tab {
    /// The tab `id`: a terminal of `grid`, with cells of `cell` pixels, and
    /// its session, which runs `command`, or the default shell, and wakes
    /// `waker` up. If the program cannot start, the terminal shows why.
    fn start(command: Option<&str>, grid: GridSize, cell: Size, waker: &Waker, id: TabId) -> Self {
        let mut terminal = Terminal::new(grid);
        terminal.set_cell_size(cell);
        let shared = Arc::new(Mutex::new(Shared::new(terminal)));
        let session = match session::start(command, grid, &shared, waker, id) {
            Ok(pty) => Some(pty),
            Err(message) => {
                lock(&shared).terminal.feed(message.as_bytes());
                None
            }
        };
        Self {
            shared,
            session,
            sync_due: None,
        }
    }

    /// Its terminal, for the view to show it, as the tab `id`.
    fn terminal(&self, id: TabId) -> TabTerminal {
        TabTerminal {
            id,
            shared: Arc::clone(&self.shared),
            input: self.session.as_ref().map(Pty::input),
        }
    }

    /// Sets the priority of its reader thread.
    fn set_priority(&self, priority: Priority) {
        // If it fails, the reader keeps the priority that it had, which only
        // changes how soon it runs while the processor is busy.
        if let Some(session) = &self.session {
            let _ = session.set_reader_priority(priority);
        }
    }

    /// While the tab is hidden: learns at `now` when the synchronized update
    /// that its program has open ends, and ends it if its time is up, with
    /// the answers to the program, which `answers` holds meanwhile. Its
    /// title waits until it is shown, when the text that it copied to the
    /// clipboard is dropped. Its reader keeps the lower priority of a hidden
    /// tab, except while the window holds the lock.
    fn learn_sync(&mut self, now: Instant, answers: &mut Vec<u8>) {
        // If the reader held the lock with a lower priority, the window
        // would wait for a thread that a busy processor leaves for later: a
        // priority inversion. The mutex of the standard library waits with
        // WaitOnAddress, which does not know who holds it, so Windows cannot
        // raise it.
        self.set_priority(Priority::Normal);
        let (due, _) = lock(&self.shared).end_late_sync(now, answers);
        self.set_priority(Priority::BelowNormal);
        self.sync_due = due;
        // Outside the lock, as the reader thread does.
        if !answers.is_empty() {
            if let Some(session) = &self.session {
                session.write(answers);
            }
            answers.clear();
        }
    }
}

/// The tabs of the window, and the view, which shows the active one.
pub struct Tabs {
    tabs: model::Window<Tab>,
    view: TerminalView,
    /// The answers for the programs of the hidden tabs, kept to avoid an
    /// allocation each time.
    answers: Vec<u8>,
}

impl Tabs {
    /// A tab with the grid that fills `window`, whose session runs
    /// `command`, or the default shell, and the view that shows it.
    pub fn new(window: &Window, command: Option<&str>) -> Result<Self, HandlerError> {
        let fonts = FontSystem::new()?;
        let dpi = window.dpi();
        let font = Font::new(&fonts, &FONT_FAMILIES, f32::from(FONT_POINTS), dpi)?;
        let cell = font.cell();
        let client = window.client_size()?;
        let grid = view::grid_size(client, cell, ScrollBar::width_at(dpi));
        let waker = window.waker();
        let Ok(tabs) = model::Window::try_new_with(|id| {
            Ok::<_, Infallible>(Tab::start(command, grid, cell, &waker, id))
        });
        let first = tabs.active();
        let terminal = first.content().terminal(first.id());
        let view = TerminalView::new(terminal, waker, fonts, font, dpi, client);
        // The clicks select text until the program asks for the mouse.
        window.set_pointer(Pointer::Text);
        Ok(Self {
            tabs,
            view,
            answers: Vec::new(),
        })
    }

    pub const fn view(&self) -> &TerminalView {
        &self.view
    }

    pub const fn view_mut(&mut self) -> &mut TerminalView {
        &mut self.view
    }

    /// The reader thread of the tab `id` woke the window up, which takes at
    /// `now` what it can. Returns whether a frame has to bring the scene up
    /// to date: the screen of the tab that is shown changed, or a hidden
    /// tab has a synchronized update open, which ends at a time that the
    /// next update returns.
    pub fn wake(&mut self, window: &Window, id: TabId, now: Instant) -> bool {
        if id == self.tabs.active().id() {
            return self.view.wake(window);
        }
        // A hidden tab, or none: a tab that closed after its reader woke
        // the window up.
        let Some(tab) = self.tabs.get_mut(id) else {
            return false;
        };
        tab.learn_sync(now, &mut self.answers);
        tab.sync_due.is_some()
    }

    /// While a message box was open, the window lost the wake-ups of the
    /// tabs: it takes now what they were for, as [`Tabs::wake`] does, for
    /// all of them.
    pub fn catch_up_wakes(&mut self, window: &Window, now: Instant) -> bool {
        let active = self.tabs.active().id();
        let mut changed = self.view.wake(window);
        for (id, tab) in self.tabs.contents_mut() {
            if id != active {
                tab.learn_sync(now, &mut self.answers);
                changed |= tab.sync_due.is_some();
            }
        }
        changed
    }

    /// Does what a shortcut asked for the tabs, at `now`. Returns whether
    /// the window shows another tab, which a frame has to draw.
    pub fn act(&mut self, window: &Window, action: TabAction, now: Instant) -> bool {
        let switch = match action {
            TabAction::New => self.open(window),
            TabAction::Next => self.tabs.activate_next(),
            TabAction::Previous => self.tabs.activate_previous(),
            TabAction::Close => return self.close_active(window, now),
        };
        let Some(switch) = switch else {
            return false;
        };
        self.show(window, switch, now);
        true
    }

    /// Opens a tab at the end, with the default shell and the grid that
    /// fills the window, and activates it, as the button + will (subphase
    /// 2.3). Returns the change of the active tab.
    fn open(&mut self, window: &Window) -> Option<Switch> {
        let grid = self.view.window_grid();
        let cell = self.view.cell();
        let waker = window.waker();
        let Ok(id) = self
            .tabs
            .try_open_with(|id| Ok::<_, Infallible>(Tab::start(None, grid, cell, &waker, id)));
        // It cannot fail: the tab was just opened.
        self.tabs.activate(id).ok().flatten()
    }

    /// Closes the active tab and its session, at `now`. The last tab closes
    /// the window instead. Returns whether the window shows another tab.
    fn close_active(&mut self, window: &Window, now: Instant) -> bool {
        let from = self.tabs.active().id();
        match self.tabs.close(from) {
            Ok(Close::Removed { content, activated }) => {
                if let Some(to) = activated {
                    self.show(window, Switch { from, to }, now);
                }
                // Its session closes without waiting. Its reader thread may
                // still wake the window up with the identifier of the tab,
                // which no tab has any more.
                drop(content);
                activated.is_some()
            }
            Ok(Close::LastTab) => {
                window.close();
                false
            }
            // The active tab is always a tab of the window.
            Err(_) => false,
        }
    }

    /// Shows the tab `switch.to`, at `now`, instead of `switch.from`, which
    /// may have closed.
    fn show(&mut self, window: &Window, switch: Switch, now: Instant) {
        // It cannot fail: the model has just activated the tab.
        let Some(to) = self.tabs.get_mut(switch.to) else {
            return;
        };
        // Its reader gets the normal priority back before the view locks its
        // terminal: if the reader held the lock, the window would wait for a
        // thread that the busy processor would leave for later.
        to.set_priority(Priority::Normal);
        // From now on, the view ends its synchronized updates.
        to.sync_due = None;
        let terminal = to.terminal(switch.to);
        self.view.show(window, terminal, now);
        if let Some(from) = self.tabs.get_mut(switch.from) {
            // The view no longer ends the synchronized update that its
            // program may have open, and its reader gets the lower priority.
            from.learn_sync(now, &mut self.answers);
        }
    }

    /// Ends the synchronized updates of the hidden tabs whose time is up at
    /// `now`. Returns when the next one ends.
    fn end_hidden_syncs(&mut self, now: Instant) -> Option<Instant> {
        let active = self.tabs.active().id();
        let answers = &mut self.answers;
        self.tabs
            .contents_mut()
            .filter(|&(id, _)| id != active)
            .filter_map(|(_, tab)| {
                if tab.sync_due.is_some_and(|due| due <= now) {
                    tab.learn_sync(now, answers);
                }
                tab.sync_due
            })
            .min()
    }
}

impl Scene for Tabs {
    fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
        let shown = self.view.update(now, damage);
        let hidden = self.end_hidden_syncs(now);
        earliest(shown, hidden)
    }

    fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
        let shown = self.view.update_hidden(now);
        let hidden = self.end_hidden_syncs(now);
        earliest(shown, hidden)
    }

    fn draw(&mut self, canvas: &mut Canvas<'_>) {
        self.view.draw(canvas);
    }
}

/// The earlier of two moments, if there is one.
fn earliest(first: Option<Instant>, second: Option<Instant>) -> Option<Instant> {
    first.into_iter().chain(second).min()
}
