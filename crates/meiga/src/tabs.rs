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
//!
//! A new tab runs the default profile or, from the menu of the profiles, the
//! one that the user chooses (ADR 0022). The first one runs the program of
//! the command line of meiga, if it gives one.
//!
//! The menu of a tab, which the right button opens on its tab, and the Menu
//! key or Shift+F10 on the tab that is shown, goes over the bar and the
//! terminal (ADR 0021), as the menu of the profiles, which the button next
//! to + and Ctrl+Shift+Space open, below those buttons. While it is open, it
//! gets all the mouse, and the keys that go down; the ones that went down
//! before go up for the program, which gets each key that it got go up too.
//! What changes under it, all inside it, is not drawn until it closes. It
//! closes when an item is chosen, and also when the window loses the focus,
//! changes size or scale, starts moving, or is minimized.

use std::convert::Infallible;
use std::mem;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use meiga_core::geometry::{GridSize, Point, Rect, Size};
use meiga_core::input::KeyEvent;
use meiga_core::model::{self, Close, Closed, Switch, TabId};
use meiga_core::profile::Launch;
use meiga_core::term::Terminal;
use meiga_core::ui::menu::{Anchor, Entry};
use meiga_render::{Canvas, Font, FontSystem};
use meiga_win::conpty::{Priority, Pty};
use meiga_win::event::{Event, HandlerError};
use meiga_win::window::{Pointer, Waker, Window};

use crate::app::TITLE;
use crate::frames::{self, Scene};
use crate::menu::MenuView;
use crate::mouse::Response;
use crate::scrollbar::ScrollBar;
use crate::session::{self, Shared, lock};
use crate::shells::Shells;
use crate::tabbar::{Label, TabBar};
use crate::view::{self, FONT_FAMILIES, FONT_POINTS, TabTerminal, TerminalView};

/// What a shortcut, a click on the tab bar, or an item of a menu asks for
/// the tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabAction {
    /// Open a tab with the default profile at the end, and show it.
    New,
    /// Open the menu of the profiles below the buttons that open a tab,
    /// with its first item highlighted if the keyboard asked for it.
    ProfileMenu { keyboard: bool },
    /// Open a tab with the profile at this place of the menu of the
    /// profiles at the end, and show it.
    OpenProfile(usize),
    /// Show the next tab or, from the last one, the first.
    Next,
    /// Show the previous tab or, from the first one, the last.
    Previous,
    /// Show the tab at this place of the bar, from 0, if there is one: the
    /// keyboard asked for it.
    Nth(usize),
    /// Show the last tab: the keyboard asked for it.
    Last,
    /// Find the shells again, and open a tab with the profile at this place
    /// of the menu of the profiles, from 0, at the end, and show it, if
    /// there is one: the keyboard asked for it, without the menu.
    Profile(usize),
    /// Close the tab that is shown, and its session. The last one closes
    /// the window.
    Close,
    /// Show this tab.
    Activate(TabId),
    /// Close this tab, and its session. The last one closes the window.
    CloseTab(TabId),
    /// Open the menu of this tab, with its top-left corner at this point of
    /// the window: the right button went up on the tab.
    Menu(TabId, Point),
    /// Open the menu of the tab that is shown, below its tab: the keyboard
    /// asked for it.
    ActiveMenu,
    /// Open a tab with the program of this one, right after it, and show
    /// it.
    Duplicate(TabId),
    /// The same with the tab that is shown: the keyboard asked for it.
    DuplicateActive,
    /// Close all the tabs but this one, and their sessions, and show this
    /// one.
    CloseOthers(TabId),
    /// The same with the tab that is shown: the keyboard asked for it.
    CloseAllButActive,
    /// Close the tabs on the right of this one, and their sessions. If the
    /// one that is shown is one of them, this one is shown.
    CloseToTheRight(TabId),
    /// Close the tabs on the right of the one that is shown, and their
    /// sessions: the keyboard asked for it.
    CloseRightOfActive,
}

/// A tab: the terminal of its session, the session, and what the bar shows
/// of it.
struct Tab {
    shared: Arc<Mutex<Shared>>,
    /// It closes when the tab closes. `None` if its program could not
    /// start.
    session: Option<Pty>,
    /// What its session runs, and the name that the bar shows while its
    /// program gives no title: a duplicate of the tab runs it too.
    launch: Launch,
    /// While the tab is hidden, when the synchronized update that its
    /// program has open ends, if the program does not end it first.
    sync_due: Option<Instant>,
    /// While the tab is hidden, its program gave another title, which the
    /// next frame takes.
    title_due: bool,
    /// The title that its program gave, as the window last took it.
    title: Option<String>,
}

impl Tab {
    /// The tab `id`: a terminal of `grid`, with cells of `cell` pixels, and
    /// its session, which runs what `launch` says, with the variables of
    /// `environment`, an environment block, and wakes `waker` up. If the
    /// program cannot start, the terminal shows why.
    fn start(
        launch: Launch,
        environment: &[u16],
        grid: GridSize,
        cell: Size,
        waker: &Waker,
        id: TabId,
    ) -> Self {
        let mut terminal = Terminal::new(grid);
        terminal.set_cell_size(cell);
        let shared = Arc::new(Mutex::new(Shared::new(terminal)));
        let session = match session::start(&launch, environment, grid, &shared, waker, id) {
            Ok(pty) => Some(pty),
            Err(message) => {
                lock(&shared).terminal.feed(message.as_bytes());
                None
            }
        };
        Self {
            shared,
            session,
            launch,
            sync_due: None,
            title_due: false,
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
        self.title.as_deref().unwrap_or(&self.launch.name)
    }
}

/// The tabs of the window, the view, which shows the active one, the tab
/// bar, the menu that may be open, and the shells that new tabs run.
pub struct Tabs {
    tabs: model::Window<Tab>,
    view: TerminalView,
    bar: TabBar,
    menu: MenuView<TabAction>,
    shells: Shells,
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
    /// runs what `first` says, the view that shows it, and the bar. New tabs
    /// run the profiles of `shells`.
    pub fn new(window: &Window, first: Launch, shells: Shells) -> Result<Self, HandlerError> {
        let fonts = FontSystem::new()?;
        let dpi = window.dpi();
        let font = Font::new(&fonts, &FONT_FAMILIES, f32::from(FONT_POINTS), dpi)?;
        let cell = font.cell();
        let client = window.client_size()?;
        let bar = TabBar::new(&fonts, dpi, client.width, 1)?;
        let area = below_bar(client, bar.height());
        let grid = view::grid_size(area, cell, ScrollBar::width_at(dpi));
        let waker = window.waker();
        let environment = shells.environment();
        let Ok(tabs) = model::Window::try_new_with(|id| {
            Ok::<_, Infallible>(Tab::start(first, environment, grid, cell, &waker, id))
        });
        let first = tabs.active();
        let terminal = first.content().terminal();
        let shortcuts = shells.shortcuts();
        let view = TerminalView::new(terminal, fonts, font, dpi, area, shortcuts);
        // The clicks select text until the program asks for the mouse.
        window.set_pointer(Pointer::Text);
        Ok(Self {
            tabs,
            view,
            bar,
            menu: MenuView::new(),
            shells,
            client,
            retitled_at: None,
            answers: Vec::new(),
        })
    }

    pub const fn view(&self) -> &TerminalView {
        &self.view
    }

    /// The client area of the window has `client` pixels now: the bar has
    /// its width, and the terminal the rest. If it changed, the menu
    /// closes.
    pub fn resized(&mut self, window: &Window, client: Size) {
        if client != self.client {
            self.close_menu(window);
        }
        self.client = client;
        self.bar.resize(client.width, &self.tabs);
        self.view.resized(below_bar(client, self.bar.height()));
    }

    /// The window moved to a monitor with `dpi`: the bar and the view are
    /// made for it, and the menu closes. The new size of the window, if it
    /// changes, comes next.
    pub fn dpi_changed(&mut self, window: &Window, dpi: u32) {
        self.close_menu(window);
        self.bar.set_dpi(dpi, &self.tabs);
        self.view.dpi_changed(dpi);
        // The bar can have another height: until the new size comes, the
        // terminal has the rest of the same window.
        self.view.resized(below_bar(self.client, self.bar.height()));
    }

    /// The window gained or lost the focus, at `now`. Without it, the menu
    /// closes.
    pub fn focus(&mut self, window: &Window, focused: bool, now: Instant) {
        if !focused {
            self.close_menu(window);
        }
        self.view.focus(focused, now);
    }

    /// Closes the menu, if it is open, and lets the mouse go if a button
    /// was down in it. Returns whether it was open: a frame has to draw
    /// what it covered.
    pub fn close_menu(&mut self, window: &Window) -> bool {
        if self.menu.is_pressed() {
            window.release_mouse();
        }
        self.menu.close()
    }

    /// A key of the window. While the menu is open, the keys that go down
    /// are for it, and the program does not get them, nor when they go up;
    /// but one that was down before does nothing while it repeats: the
    /// program gets it when it goes up, if it went down for the program,
    /// and nobody does, if a shortcut took it, as the one that opened the
    /// menu.
    pub fn key(&mut self, window: &Window, event: &KeyEvent) -> Response {
        if !self.menu.is_open() || !event.down {
            return self.view.key(window, event);
        }
        if self.view.holds_key(event.key) || self.view.took_for_a_shortcut(event.key) {
            return Response::default();
        }
        let pressed = self.menu.is_pressed();
        self.view.take_key(event);
        let tab = self.menu.key(event);
        // Closed with a button down in it: the window lets the mouse go.
        if pressed && !self.menu.is_pressed() {
            window.release_mouse();
        }
        Response {
            redraw: self.menu.has_damage(),
            tab,
            ..Response::default()
        }
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

    /// Does what a shortcut, a click or the menu asked for the tabs, at
    /// `now`. Returns whether a frame has to show it: the window shows
    /// another tab, the bar changed, or the menu opened.
    pub fn act(&mut self, window: &Window, action: TabAction, now: Instant) -> bool {
        let active = self.tabs.active().id();
        let switch = match action {
            TabAction::New => self.open(window, self.shells.default_launch()),
            TabAction::OpenProfile(index) => match self.shells.launch(index) {
                Some(launch) => self.open(window, launch),
                None => None,
            },
            TabAction::Profile(index) => {
                // The profile at that place of the menu, as it would show
                // them now.
                self.shells.refresh();
                match self.shells.launch(index) {
                    Some(launch) => self.open(window, launch),
                    None => None,
                }
            }
            TabAction::ProfileMenu { keyboard } => {
                return self.open_profile_menu(window, keyboard);
            }
            TabAction::Duplicate(id) => self.duplicate(window, id),
            TabAction::DuplicateActive => self.duplicate(window, active),
            TabAction::Next => self.tabs.activate_next(),
            TabAction::Previous => self.tabs.activate_previous(),
            TabAction::Nth(index) => match self.tabs.tabs().get(index) {
                Some(tab) => self.tabs.activate(tab.id()).ok().flatten(),
                None => None,
            },
            TabAction::Last => match self.tabs.tabs().last() {
                Some(tab) => self.tabs.activate(tab.id()).ok().flatten(),
                None => None,
            },
            TabAction::Activate(id) => self.tabs.activate(id).ok().flatten(),
            TabAction::Close => return self.close(window, active, now),
            TabAction::CloseTab(id) => return self.close(window, id, now),
            TabAction::CloseOthers(id) => {
                return self.close_several(window, now, |tabs| tabs.close_others(id));
            }
            TabAction::CloseAllButActive => {
                return self.close_several(window, now, |tabs| tabs.close_others(active));
            }
            TabAction::CloseToTheRight(id) => {
                return self.close_several(window, now, |tabs| tabs.close_to_the_right(id));
            }
            TabAction::CloseRightOfActive => {
                return self.close_several(window, now, |tabs| tabs.close_to_the_right(active));
            }
            TabAction::Menu(id, at) => return self.open_menu(window, id, Anchor::At(at), false),
            TabAction::ActiveMenu => {
                let tab = self.bar.tab(self.tabs.active_index());
                return self.open_menu(window, active, Anchor::Below(tab), true);
            }
        };
        match switch {
            Some(switch) => {
                self.show(window, switch, now);
                true
            }
            None => self.bar.has_damage(),
        }
    }

    /// Opens a tab at the end, which runs what `launch` says, and activates
    /// it. Returns the change of the active tab.
    fn open(&mut self, window: &Window, launch: Launch) -> Option<Switch> {
        let id = self.open_tab(window, launch);
        self.bar.tabs_changed(&self.tabs);
        // It cannot fail: the tab was just opened.
        self.tabs.activate(id).ok().flatten()
    }

    /// Opens a tab with the program of the tab `id`, right after it, and
    /// activates it, as "Duplicate tab" in Chrome and in Windows Terminal.
    /// Returns the change of the active tab.
    fn duplicate(&mut self, window: &Window, id: TabId) -> Option<Switch> {
        let index = self.tabs.index_of(id)?;
        let launch = self.tabs.get(id)?.launch.clone();
        let copy = self.open_tab(window, launch);
        // It cannot fail: the copy is the last tab, after the one at
        // `index`.
        let _ = self.tabs.move_tab(copy, index + 1);
        self.bar.tabs_changed(&self.tabs);
        self.tabs.activate(copy).ok().flatten()
    }

    /// Opens a tab at the end, whose session runs what `launch` says, with
    /// the grid that fills the window. Returns its identifier.
    fn open_tab(&mut self, window: &Window, launch: Launch) -> TabId {
        let grid = self.view.window_grid();
        let cell = self.view.cell();
        let waker = window.waker();
        let environment = self.shells.environment();
        let Ok(id) = self.tabs.try_open_with(|id| {
            Ok::<_, Infallible>(Tab::start(launch, environment, grid, cell, &waker, id))
        });
        id
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

    /// Closes the tabs that `close` closes, and their sessions, at `now`.
    /// Returns whether a frame has to show it: the bar without them, and
    /// another tab if the active one was one of them.
    fn close_several(
        &mut self,
        window: &Window,
        now: Instant,
        close: impl FnOnce(&mut model::Window<Tab>) -> Result<Closed<Tab>, model::Error>,
    ) -> bool {
        let from = self.tabs.active().id();
        // A tab that already closed.
        let Ok(Closed {
            contents,
            activated,
        }) = close(&mut self.tabs)
        else {
            return false;
        };
        if contents.is_empty() {
            return false;
        }
        if let Some(to) = activated {
            self.show(window, Switch { from, to }, now);
        }
        // Their sessions close without waiting, as in `close`.
        drop(contents);
        self.bar.tabs_changed(&self.tabs);
        true
    }

    /// Opens the menu of the tab `id` where `anchor` says, with its first
    /// item highlighted if the keyboard opens it. Returns whether a frame
    /// has to show it.
    fn open_menu(&mut self, window: &Window, id: TabId, anchor: Anchor, keyboard: bool) -> bool {
        let Some(index) = self.tabs.index_of(id) else {
            return false;
        };
        let count = self.tabs.tab_count();
        let entries = vec![
            Entry::item("Duplicate tab", TabAction::Duplicate(id), true),
            Entry::Separator,
            Entry::item("Close tab", TabAction::CloseTab(id), true),
            Entry::item("Close other tabs", TabAction::CloseOthers(id), count > 1),
            Entry::item(
                "Close tabs to the right",
                TabAction::CloseToTheRight(id),
                index + 1 < count,
            ),
        ];
        self.show_menu(window, entries, anchor, keyboard)
    }

    /// Opens the menu of the profiles below the buttons that open a tab,
    /// with the shells that are installed now, and the default profile in
    /// bold. Its first item is highlighted if the keyboard opens it.
    /// Returns whether a frame has to show it.
    fn open_profile_menu(&mut self, window: &Window, keyboard: bool) -> bool {
        self.shells.refresh();
        let profiles = self.shells.profiles();
        let entries = profiles
            .list()
            .iter()
            .enumerate()
            .map(|(index, profile)| {
                let item = Entry::item(profile.name.clone(), TabAction::OpenProfile(index), true);
                if index == profiles.default_index() {
                    item.bold()
                } else {
                    item
                }
            })
            .collect();
        let anchor = Anchor::Below(self.bar.buttons());
        self.show_menu(window, entries, anchor, keyboard)
    }

    /// Opens a menu of `entries` where `anchor` says, instead of the one
    /// that may be open, with its first item highlighted if the keyboard
    /// opens it. From now on, the mouse is for the menu: a button that is
    /// down on the bar or on the terminal is let go of, and they lose the
    /// pointer. Returns whether a frame has to show it.
    fn show_menu(
        &mut self,
        window: &Window,
        entries: Vec<Entry<TabAction>>,
        anchor: Anchor,
        keyboard: bool,
    ) -> bool {
        if self.bar.is_pressed() {
            self.bar.capture_lost(&self.tabs);
            window.release_mouse();
        }
        self.bar.left(&self.tabs);
        self.view.forget_mouse(window);
        window.set_pointer(Pointer::Arrow);
        let cell = self.bar.font_mut().cell();
        let bounds = Rect::from_size(self.client);
        self.menu
            .open(entries, self.view.dpi(), cell, anchor, bounds, keyboard);
        true
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

    /// A mouse event of the window, for the menu while it is open, and
    /// otherwise for the bar or for the terminal.
    pub fn mouse(&mut self, window: &Window, event: &Event) -> Response {
        if self.menu.is_open() {
            return self.menu_mouse(window, event);
        }
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

    /// A mouse event while the menu is open: it gets all of them, and the
    /// window captures the mouse while a button is down in it.
    fn menu_mouse(&mut self, window: &Window, event: &Event) -> Response {
        let pressed = self.menu.is_pressed();
        let mut tab = None;
        match *event {
            Event::MouseMoved { at, .. } => {
                window.set_pointer(Pointer::Arrow);
                if self.menu.moved(at) {
                    window.track_mouse_leave();
                }
            }
            Event::MouseDown { button, at, .. } => self.menu.down(button, at),
            Event::MouseUp { button, at, .. } => tab = self.menu.up(button, at),
            Event::MouseLeft => self.menu.left(),
            Event::CaptureLost => self.menu.capture_lost(),
            // The wheel does nothing.
            _ => {}
        }
        match (pressed, self.menu.is_pressed()) {
            (false, true) => window.capture_mouse(),
            (true, false) if *event != Event::CaptureLost => {
                window.release_mouse();
                // Holding the mouse may have ended the tracking of its
                // leave. If the pointer is out of the window, Windows tells
                // it at once.
                if self.menu.is_open() {
                    window.track_mouse_leave();
                }
            }
            _ => {}
        }
        Response {
            redraw: self.menu.has_damage(),
            tab,
            ..Response::default()
        }
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
        // What changes in the bar and in the terminal under the menu, all
        // inside it, is not drawn: it shows when the menu closes, which
        // draws again all that it covered. The view gives its areas from
        // its own top-left corner, and the whole window when it draws all
        // of it, with the bar: when it shows another tab, or the colours of
        // the terminal change.
        let menu = &self.menu;
        let shown = self.view.update(window, now, &mut |area| {
            let area = if area == frames::ALL {
                area
            } else {
                area.offset(Point::new(0, top))
            };
            if !menu.covers(area) {
                damage(area);
            }
        });
        // After the view, which may have ended a synchronized update with
        // a title in it.
        self.retitle_active(window);
        let hidden = self.end_hidden_syncs(now);
        self.retitle_hidden();
        let menu = &self.menu;
        self.bar.take_damage(&mut |area| {
            if !menu.covers(area) {
                damage(area);
            }
        });
        self.menu.take_damage(damage);
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
        // Under the menu there is only the menu, which covers all of it.
        if !self.menu.covers(canvas.clip()) {
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
        self.menu.draw(canvas, self.bar.font_mut());
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
