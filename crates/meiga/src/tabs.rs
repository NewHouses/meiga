//! The tabs of the window: a session for each one, which goes on while the
//! tab is hidden; the view, which shows the active one (ADR 0019); and the
//! tab bar, above it (ADR 0020).
//!
//! The reader thread of each session gives the output of its program to
//! the terminal of its tab, also while the tab is hidden, then with a lower
//! priority. It wakes the window up with the identifier of its tab, and only
//! once for each kind of change, until the window takes it (ADR 0005). The
//! window does not copy nor draw the screen of a hidden tab, nor takes its
//! text for the clipboard, so the output of a hidden tab wakes it up at most
//! once for them. It takes its title, which the bar shows, each time that it
//! changes, and its synchronized updates cannot wait: the window ends them
//! in time, as while it is minimized (ADR 0013). When a tab is shown again,
//! the view copies its whole screen, once.
//!
//! The window never waits for a reader with a lower priority than its own:
//! before it locks the terminal of a tab, it gives the reader of the tab the
//! normal priority, which a hidden tab only keeps while the window holds the
//! lock.
//!
//! The mouse is for the bar or for the terminal: for the one where a button
//! went down, until it goes up, and otherwise for the one under the
//! pointer. The terminal gets the events with its own coordinates, from
//! below the bar.

use std::convert::Infallible;
use std::mem;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use meiga_core::geometry::{GridSize, Point, Rect, Size};
use meiga_core::model::{self, Close, Switch, TabId};
use meiga_core::shell;
use meiga_core::term::Terminal;
use meiga_render::{Canvas, Font, FontSystem};
use meiga_win::conpty::{Priority, Pty};
use meiga_win::event::{Event, HandlerError};
use meiga_win::window::{Pointer, Waker, Window};

use crate::app::TITLE;
use crate::frames::{self, Scene};
use crate::mouse::Response;
use crate::scrollbar::ScrollBar;
use crate::session::{self, Shared, lock};
use crate::tabbar::{Label, TabBar};
use crate::view::{self, FONT_FAMILIES, FONT_POINTS, TabTerminal, TerminalView};

/// What a shortcut, or a click on the tab bar, asks for the tabs.
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
    /// Show this tab.
    Activate(TabId),
    /// Close this tab, and its session. The last one closes the window.
    CloseTab(TabId),
}

/// A tab: the terminal of its session, the session, and what the bar shows
/// of it.
struct Tab {
    shared: Arc<Mutex<Shared>>,
    /// It closes when the tab closes. `None` if its program could not
    /// start.
    session: Option<Pty>,
    /// While the tab is hidden, when the synchronized update that its
    /// program has open ends, if the program does not end it first.
    sync_due: Option<Instant>,
    /// While the tab is hidden, its program gave another title, which the
    /// next frame takes.
    title_due: bool,
    /// The name of its program, which the bar shows while the program gives
    /// no title.
    name: String,
    /// The title that its program gave, as the window last took it.
    title: Option<String>,
}

impl Tab {
    /// The tab `id`: a terminal of `grid`, with cells of `cell` pixels, and
    /// its session, which runs `command`, or the default shell, and wakes
    /// `waker` up. If the program cannot start, the terminal shows why.
    fn start(command: Option<&str>, grid: GridSize, cell: Size, waker: &Waker, id: TabId) -> Self {
        let mut terminal = Terminal::new(grid);
        terminal.set_cell_size(cell);
        let shared = Arc::new(Mutex::new(Shared::new(terminal)));
        let (program, started) = session::start(command, grid, &shared, waker, id);
        let session = match started {
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
            title_due: false,
            name: shell::program_name(program).to_owned(),
            title: None,
        }
    }

    /// Its terminal, for the view to show it.
    fn terminal(&self) -> TabTerminal {
        TabTerminal {
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
    /// the answers to the program, which `answers` holds meanwhile, and
    /// whether its program gave another title, which the next frame takes.
    /// What it copied to the clipboard waits until it is shown, when it is
    /// dropped. Its reader keeps the lower priority of a hidden tab, except
    /// while the window holds the lock.
    fn learn(&mut self, now: Instant, answers: &mut Vec<u8>) {
        // If the reader held the lock with a lower priority, the window
        // would wait for a thread that a busy processor leaves for later: a
        // priority inversion. The mutex of the standard library waits with
        // WaitOnAddress, which does not know who holds it, so Windows cannot
        // raise it.
        self.set_priority(Priority::Normal);
        let due = {
            let mut shared = lock(&self.shared);
            let due = shared.end_late_sync(now, answers);
            self.title_due |= shared.title_changed;
            due
        };
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

    /// While the tab is hidden: takes its title, if its program gave another
    /// one, with the priorities of [`Tab::learn`]. Returns whether it is
    /// another one.
    fn retitle(&mut self) -> bool {
        if !mem::take(&mut self.title_due) {
            return false;
        }
        self.set_priority(Priority::Normal);
        let retitled = lock(&self.shared).take_title(&mut self.title);
        self.set_priority(Priority::BelowNormal);
        retitled
    }
}

impl Label for Tab {
    fn label(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.name)
    }
}

/// The tabs of the window, the view, which shows the active one, and the
/// tab bar.
pub struct Tabs {
    tabs: model::Window<Tab>,
    view: TerminalView,
    bar: TabBar,
    /// The size of the client area of the window: the bar, and below it
    /// the terminal.
    client: Size,
    /// While the window is minimized, when it last took the title of the
    /// active tab: it takes it at most once per frame interval.
    retitled_at: Option<Instant>,
    /// The answers for the programs of the hidden tabs, kept to avoid an
    /// allocation each time.
    answers: Vec<u8>,
}

impl Tabs {
    /// A tab with the grid that fills `window` below the bar, whose session
    /// runs `command`, or the default shell, the view that shows it, and
    /// the bar.
    pub fn new(window: &Window, command: Option<&str>) -> Result<Self, HandlerError> {
        let fonts = FontSystem::new()?;
        let dpi = window.dpi();
        let font = Font::new(&fonts, &FONT_FAMILIES, f32::from(FONT_POINTS), dpi)?;
        let cell = font.cell();
        let client = window.client_size()?;
        let bar = TabBar::new(&fonts, dpi, client.width, 1)?;
        let area = below_bar(client, bar.height());
        let grid = view::grid_size(area, cell, ScrollBar::width_at(dpi));
        let waker = window.waker();
        let Ok(tabs) = model::Window::try_new_with(|id| {
            Ok::<_, Infallible>(Tab::start(command, grid, cell, &waker, id))
        });
        let first = tabs.active();
        let terminal = first.content().terminal();
        let view = TerminalView::new(terminal, fonts, font, dpi, area);
        // The clicks select text until the program asks for the mouse.
        window.set_pointer(Pointer::Text);
        Ok(Self {
            tabs,
            view,
            bar,
            client,
            retitled_at: None,
            answers: Vec::new(),
        })
    }

    pub const fn view(&self) -> &TerminalView {
        &self.view
    }

    pub const fn view_mut(&mut self) -> &mut TerminalView {
        &mut self.view
    }

    /// The client area of the window has `client` pixels now: the bar has
    /// its width, and the terminal the rest.
    pub fn resized(&mut self, client: Size) {
        self.client = client;
        self.bar.resize(client.width, &self.tabs);
        self.view.resized(below_bar(client, self.bar.height()));
    }

    /// The window moved to a monitor with `dpi`: the bar and the view are
    /// made for it. The new size of the window, if it changes, comes next.
    pub fn dpi_changed(&mut self, dpi: u32) {
        self.bar.set_dpi(dpi, &self.tabs);
        self.view.dpi_changed(dpi);
        // The bar can have another height: until the new size comes, the
        // terminal has the rest of the same window.
        self.view.resized(below_bar(self.client, self.bar.height()));
    }

    /// The reader thread of the tab `id` woke the window up, which takes at
    /// `now` what it can. Returns whether a frame has to bring the scene up
    /// to date: the screen of the tab that is shown changed, a program gave
    /// another title, which the frame takes, or a hidden tab has a
    /// synchronized update open, which ends at a time that the next update
    /// returns.
    ///
    /// The titles wait for the frame so that the window takes them at most
    /// once per frame interval: until it does, the reader does not wake it
    /// up again for them, however fast its program changes them.
    pub fn wake(&mut self, window: &Window, id: TabId, now: Instant) -> bool {
        if id == self.tabs.active().id() {
            let woken = self.view.wake(window);
            return woken.screen || woken.title;
        }
        // A hidden tab, or none: a tab that closed after its reader woke
        // the window up.
        let Some(tab) = self.tabs.get_mut(id) else {
            return false;
        };
        tab.learn(now, &mut self.answers);
        tab.title_due || tab.sync_due.is_some()
    }

    /// While a message box was open, the window lost the wake-ups of the
    /// tabs: it takes now what they were for, as [`Tabs::wake`] does, for
    /// all of them.
    pub fn catch_up_wakes(&mut self, window: &Window, now: Instant) -> bool {
        let woken = self.view.wake(window);
        let mut changed = woken.screen || woken.title;
        let active = self.tabs.active().id();
        for (id, tab) in self.tabs.contents_mut() {
            if id != active {
                tab.learn(now, &mut self.answers);
                changed |= tab.title_due || tab.sync_due.is_some();
            }
        }
        changed
    }

    /// Takes the title of the active tab, if its program gave another one,
    /// for the window and the bar.
    fn retitle_active(&mut self, window: &Window) {
        let index = self.tabs.active_index();
        let id = self.tabs.active().id();
        if let Some(tab) = self.tabs.get_mut(id)
            && self.view.take_title(&mut tab.title)
        {
            window.set_title(tab.title.as_deref().unwrap_or(TITLE));
            self.bar.label_changed(index);
        }
    }

    /// Takes the titles that the programs of the hidden tabs gave, for the
    /// bar.
    fn retitle_hidden(&mut self) {
        let active = self.tabs.active_index();
        let Self { tabs, bar, .. } = self;
        for (index, (_, tab)) in tabs.contents_mut().enumerate() {
            if index != active && tab.retitle() {
                bar.label_changed(index);
            }
        }
    }

    /// A message box of the view closed. While it was open, the events of
    /// the window did not reach the tabs: no button of the mouse is
    /// followed any more.
    pub fn dialog_closed(&mut self, window: &Window) {
        self.bar.capture_lost(&self.tabs);
        self.view.dialog_closed(window);
    }

    /// Does what a shortcut or a click asked for the tabs, at `now`.
    /// Returns whether a frame has to show it: the window shows another
    /// tab, or the bar changed.
    pub fn act(&mut self, window: &Window, action: TabAction, now: Instant) -> bool {
        let switch = match action {
            TabAction::New => self.open(window),
            TabAction::Next => self.tabs.activate_next(),
            TabAction::Previous => self.tabs.activate_previous(),
            TabAction::Activate(id) => self.tabs.activate(id).ok().flatten(),
            TabAction::Close => return self.close(window, self.tabs.active().id(), now),
            TabAction::CloseTab(id) => return self.close(window, id, now),
        };
        match switch {
            Some(switch) => {
                self.show(window, switch, now);
                true
            }
            None => self.bar.has_damage(),
        }
    }

    /// Opens a tab at the end, with the default shell and the grid that
    /// fills the window, and activates it. Returns the change of the active
    /// tab.
    fn open(&mut self, window: &Window) -> Option<Switch> {
        let grid = self.view.window_grid();
        let cell = self.view.cell();
        let waker = window.waker();
        let Ok(id) = self
            .tabs
            .try_open_with(|id| Ok::<_, Infallible>(Tab::start(None, grid, cell, &waker, id)));
        self.bar.tabs_changed(&self.tabs);
        // It cannot fail: the tab was just opened.
        self.tabs.activate(id).ok().flatten()
    }

    /// Closes the tab `id` and its session, at `now`. The last tab closes
    /// the window instead. Returns whether a frame has to show it: the bar
    /// without the tab, and another tab if it was the active one.
    fn close(&mut self, window: &Window, id: TabId, now: Instant) -> bool {
        match self.tabs.close(id) {
            Ok(Close::Removed { content, activated }) => {
                if let Some(to) = activated {
                    self.show(window, Switch { from: id, to }, now);
                }
                // Its session closes without waiting. Its reader thread may
                // still wake the window up with the identifier of the tab,
                // which no tab has any more.
                drop(content);
                self.bar.tabs_changed(&self.tabs);
                true
            }
            Ok(Close::LastTab) => {
                window.close();
                false
            }
            // A tab that already closed.
            Err(_) => false,
        }
    }

    /// Shows the tab `switch.to`, at `now`, instead of `switch.from`, which
    /// may have closed. Its title goes to the window.
    fn show(&mut self, window: &Window, switch: Switch, now: Instant) {
        // It cannot fail: the model has just activated the tab.
        let Some(to) = self.tabs.get_mut(switch.to) else {
            return;
        };
        // Its reader gets the normal priority back before the view locks its
        // terminal: if the reader held the lock, the window would wait for a
        // thread that the busy processor would leave for later.
        to.set_priority(Priority::Normal);
        // The view ends now the synchronized update that its program may
        // have open, and from now on the ones that follow, and takes its
        // title, which it copies now.
        to.sync_due = None;
        to.title_due = false;
        let terminal = to.terminal();
        self.view.show(window, terminal, now, &mut to.title);
        window.set_title(to.title.as_deref().unwrap_or(TITLE));
        if let Some(from) = self.tabs.get_mut(switch.from) {
            // The view no longer ends the synchronized update that its
            // program may have open, and its reader gets the lower priority.
            // The frame draws the whole bar.
            from.learn(now, &mut self.answers);
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
                    tab.learn(now, answers);
                }
                tab.sync_due
            })
            .min()
    }

    /// A mouse event of the window, for the bar or for the terminal.
    pub fn mouse(&mut self, window: &Window, event: &Event) -> Response {
        let at = match *event {
            Event::MouseDown { at, .. }
            | Event::MouseUp { at, .. }
            | Event::MouseMoved { at, .. }
            | Event::Wheel { at, .. } => at,
            // For both: the pointer left the window, or another window
            // took the mouse.
            Event::MouseLeft | Event::CaptureLost => {
                if *event == Event::CaptureLost {
                    self.bar.capture_lost(&self.tabs);
                } else {
                    self.bar.left(&self.tabs);
                }
                let mut response = self.view.mouse(window, event);
                response.redraw |= self.bar.has_damage();
                return response;
            }
            _ => return Response::default(),
        };
        let top = coordinate(self.bar.height());
        if self.bar.is_pressed() || (at.y < top && !self.view.holds_mouse()) {
            return self.bar_mouse(window, event, at);
        }
        self.bar.left(&self.tabs);
        let mut response = self.view.mouse(window, &raised(event, top));
        response.redraw |= self.bar.has_damage();
        response
    }

    /// A mouse event of the bar, with the pointer at `at`.
    fn bar_mouse(&mut self, window: &Window, event: &Event, at: Point) -> Response {
        let mut response = Response::default();
        window.set_pointer(Pointer::Arrow);
        if self.bar.moved(&self.tabs, at) {
            window.track_mouse_leave();
            // The terminal no longer has the pointer: its scroll bar loses
            // the highlight.
            response.redraw = self.view.mouse(window, &Event::MouseLeft).redraw;
        }
        match *event {
            Event::MouseDown { button, .. } => {
                response.tab = self.bar.down(&self.tabs, button);
                if self.bar.is_pressed() {
                    window.capture_mouse();
                }
            }
            Event::MouseUp { button, .. } => {
                let pressed = self.bar.is_pressed();
                response.tab = self.bar.up(&self.tabs, button);
                if pressed && !self.bar.is_pressed() {
                    window.release_mouse();
                    // Holding the mouse may have ended the tracking of its
                    // leave.
                    if self.bar.contains(at) {
                        window.track_mouse_leave();
                    }
                }
            }
            // The wheel on the bar does nothing yet (subphase 2.8).
            _ => {}
        }
        response.redraw |= self.bar.has_damage();
        response
    }
}

impl Scene for Tabs {
    fn update(
        &mut self,
        window: &Window,
        now: Instant,
        damage: &mut dyn FnMut(Rect),
    ) -> Option<Instant> {
        let top = coordinate(self.bar.height());
        // The view gives its areas from its own top-left corner, and the
        // whole window when it draws all of it, with the bar: when it shows
        // another tab, or the colours of the terminal change.
        let shown = self.view.update(window, now, &mut |area| {
            damage(if area == frames::ALL {
                area
            } else {
                area.offset(Point::new(0, top))
            });
        });
        // After the view, which may have ended a synchronized update with
        // a title in it.
        self.retitle_active(window);
        let hidden = self.end_hidden_syncs(now);
        self.retitle_hidden();
        self.bar.take_damage(damage);
        earliest(shown, hidden)
    }

    fn update_hidden(&mut self, window: &Window, now: Instant) -> Option<Instant> {
        let shown = self.view.update_hidden(window, now);
        let hidden = self.end_hidden_syncs(now);
        // The title of the active tab goes to the window, which the taskbar
        // shows, at most once per frame interval. The bar waits until the
        // window is restored.
        let title = if !self.view.title_pending() {
            None
        } else if let Some(next) = self
            .retitled_at
            .and_then(|at| at.checked_add(frames::interval()))
            .filter(|&next| now < next)
        {
            Some(next)
        } else {
            self.retitled_at = Some(now);
            self.retitle_active(window);
            None
        };
        earliest(earliest(shown, hidden), title)
    }

    fn draw(&mut self, canvas: &mut Canvas<'_>) {
        self.bar.draw(canvas, &self.tabs, self.view.palette());
        let terminal = Rect {
            top: coordinate(self.bar.height()),
            ..Rect::from_size(canvas.size())
        };
        let mut terminal = canvas.region(terminal);
        if !terminal.clip().is_empty() {
            self.view.draw(&mut terminal);
        }
    }
}

/// The size of the terminal in a client area of `client` pixels, below a
/// bar `bar` pixels high.
fn below_bar(client: Size, bar: u32) -> Size {
    Size::new(client.width, client.height.saturating_sub(bar))
}

/// `event`, with its point `by` pixels higher.
fn raised(event: &Event, by: i32) -> Event {
    let mut event = *event;
    if let Event::MouseDown { at, .. }
    | Event::MouseUp { at, .. }
    | Event::MouseMoved { at, .. }
    | Event::Wheel { at, .. } = &mut event
    {
        at.y = at.y.saturating_sub(by);
    }
    event
}

/// A height of the bar as a coordinate: a few dozen pixels.
fn coordinate(pixels: u32) -> i32 {
    i32::try_from(pixels).unwrap_or(i32::MAX)
}

/// The earlier of two moments, if there is one.
fn earliest(first: Option<Instant>, second: Option<Instant>) -> Option<Instant> {
    first.into_iter().chain(second).min()
}
