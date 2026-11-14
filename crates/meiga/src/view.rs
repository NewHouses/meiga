//! The terminal of the tab that the window shows: its copy of the screen,
//! brought up to date from the terminal of the session before each frame,
//! and the drawing of the cells, their lines and the cursor. The view is
//! the same for all the tabs of the window, with its font, its zoom and its
//! scroll bar: when the window shows another tab, it shows its terminal
//! (ADR 0019).

use std::mem;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use meiga_core::color::{Palette, Rgb};
use meiga_core::geometry::{GridSize, Point, Rect, Size};
use meiga_core::input::{KeyEncoder, KeyEvent, focus_sequence};
use meiga_core::model::TabId;
use meiga_core::term::{Cell, CellFlags, Columns, Cursor, CursorShape, MIN_SIZE, Screen, Scroll};
use meiga_core::text::CellChar;
use meiga_render::{Canvas, Decorations, Font, FontStyle, FontSystem};
use meiga_win::conpty::Input;
use meiga_win::event::Event;
use meiga_win::window::{self, CaretSettings, Waker, Window};

use crate::app::TITLE;
use crate::clipboard;
use crate::frames::{self, Scene};
use crate::mouse::{Context, Mouse, Response};
use crate::session::{Shared, lock};
use crate::tabs::TabAction;

/// The font: the first of these families that is installed, at 12 points,
/// as in Windows Terminal. The zoom changes its size a point at a time,
/// from 6 to 72 points.
pub const FONT_FAMILIES: [&str; 2] = ["Cascadia Mono", "Consolas"];
pub const FONT_POINTS: u16 = 12;
const MIN_POINTS: u16 = 6;
const MAX_POINTS: u16 = 72;

/// How far dim text (SGR 2) goes from its colour towards the one of its
/// background: a third of the way, out of 255.
const DIM: u8 = 85;

/// The dots per inch at a scale of 100 %.
const BASE_DPI: u32 = 96;

/// The virtual-key codes of the keys that meiga takes for itself.
const VK_C: u16 = 0x43;
const VK_V: u16 = 0x56;
const VK_ESCAPE: u16 = 0x1b;
/// T, W and Tab, of the shortcuts of the tabs.
const VK_T: u16 = 0x54;
const VK_W: u16 = 0x57;
const VK_TAB: u16 = 0x09;
/// Page Up, Page Down, End, Home, and the up and down arrows.
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_UP: u16 = 0x26;
const VK_DOWN: u16 = 0x28;
/// The 0, the + and the - keys of the main keyboard and of the numeric
/// keypad. On the main keyboard, the + and the - are the keys with + and -
/// of the Spanish layout, and with = and - of the US one.
const VK_0: u16 = 0x30;
const VK_NUMPAD0: u16 = 0x60;
const VK_ADD: u16 = 0x6b;
const VK_SUBTRACT: u16 = 0x6d;
const VK_OEM_PLUS: u16 = 0xbb;
const VK_OEM_MINUS: u16 = 0xbd;

/// The terminal of the tab that the window shows, as the window shows it.
pub struct TerminalView {
    /// The tab that the view shows. Its reader thread wakes the window up
    /// with its identifier, and so does the view, for its title.
    tab: TabId,
    shared: Arc<Mutex<Shared>>,
    /// Sends to the program the keys, and the answers that the window finds
    /// when it ends a synchronized update, and tells it the size of the
    /// grid. `None` if the program did not start.
    input: Option<Input>,
    waker: Waker,
    /// The fonts of the system, for a new font when the zoom or the scale
    /// changes.
    fonts: FontSystem,
    font: Font,
    /// The size of the font, in points, and the dots per inch of the
    /// window.
    points: u16,
    dpi: u32,
    cell: Size,
    /// The size of the client area of the window.
    client: Size,
    /// The grid of the terminal and of the program.
    grid: GridSize,
    /// The grid that fills the window, while they do not have it: it
    /// changes at most once per frame interval, since they reflow their
    /// text each time. It last changed at `resized_at`.
    wanted: Option<GridSize>,
    resized_at: Option<Instant>,
    /// The whole window has to be drawn again: the font changed.
    redraw: bool,
    screen: Screen,
    caret: Caret,
    mouse: Mouse,
    /// The keys that meiga took when they went down: when they go up, the
    /// program does not get them either.
    taken: Vec<u16>,
    /// The characters of a run of cells, kept to avoid an allocation for
    /// each one.
    run: Vec<CellChar>,
    /// The answers for the program, kept for the same reason.
    answers: Vec<u8>,
    /// Turns the keys into what the program reads.
    encoder: KeyEncoder,
    /// What the program reads for a key, kept for the same reason.
    keys: Vec<u8>,
}

/// The terminal of a tab, as the view needs it to show it.
pub struct TabTerminal {
    pub id: TabId,
    pub shared: Arc<Mutex<Shared>>,
    /// What sends input to its program, or `None` if the program did not
    /// start.
    pub input: Option<Input>,
}

impl TerminalView {
    /// The view of the terminal of a tab, drawn with `font`, of `fonts`, in
    /// a window of `client` pixels at `dpi`.
    pub fn new(
        terminal: TabTerminal,
        waker: Waker,
        fonts: FontSystem,
        font: Font,
        dpi: u32,
        client: Size,
    ) -> Self {
        let TabTerminal { id, shared, input } = terminal;
        let grid = lock(&shared).terminal.size();
        Self {
            tab: id,
            shared,
            input,
            waker,
            fonts,
            cell: font.cell(),
            font,
            points: FONT_POINTS,
            dpi,
            client,
            grid,
            wanted: None,
            resized_at: None,
            redraw: false,
            screen: Screen::new(),
            caret: Caret::new(dpi, Instant::now()),
            mouse: Mouse::new(dpi, client),
            taken: Vec::new(),
            run: Vec::new(),
            answers: Vec::new(),
            encoder: KeyEncoder::new(),
            keys: Vec::new(),
        }
    }

    pub const fn is_focused(&self) -> bool {
        self.caret.focused
    }

    /// The dots per inch for which the view draws.
    pub const fn dpi(&self) -> u32 {
        self.dpi
    }

    /// The size of the cells, in pixels.
    pub const fn cell(&self) -> Size {
        self.cell
    }

    /// The grid that fills the window: the one of a new tab.
    pub fn window_grid(&self) -> GridSize {
        grid_size(self.client, self.cell, self.mouse.bar().width())
    }

    /// The reader thread of the tab that the view shows woke the window up:
    /// its title, its screen, the clipboard, or more than one, changed, or
    /// its program left a synchronized update open. The window takes the
    /// title, and the text for the clipboard if it can. Returns whether the
    /// screen changed, which a frame has to show: the next update also
    /// learns when the synchronized update ends, even with the window
    /// minimized.
    pub fn wake(&self, window: &Window) -> bool {
        let (title, clipboard, screen) = {
            let mut shared = lock(&self.shared);
            let title = mem::take(&mut shared.title_changed)
                .then(|| shared.terminal.title().map(str::to_owned));
            let clipboard = mem::take(&mut shared.clipboard_changed)
                .then(|| shared.terminal.take_clipboard())
                .flatten();
            (title, clipboard, shared.screen_changed)
        };
        if let Some(title) = title {
            window.set_title(title.as_deref().unwrap_or(TITLE));
        }
        // A program cannot change the clipboard from a window that the user
        // is not using: the window needs the focus, and to be the foreground
        // window. If another program keeps the clipboard open, the text is
        // lost, as in Windows Terminal.
        if let Some(text) = clipboard
            && self.is_focused()
            && window.is_foreground()
        {
            let _ = meiga_win::clipboard::write_text(window, &text);
        }
        screen
    }

    /// Shows the terminal of another tab, from `now`. What the mouse was
    /// doing with the old one ends, and both programs learn that the focus
    /// moved, if the window has it and they asked. The next update copies
    /// all the screen of the new one, once, and gives it the grid that
    /// fills the window, if it changed while the tab was hidden. The window
    /// takes its title; what its program copied to the clipboard while it
    /// was hidden is dropped (ADR 0019).
    pub fn show(&mut self, window: &Window, terminal: TabTerminal, now: Instant) {
        let context = Context {
            window,
            shared: &self.shared,
            input: self.input.as_ref(),
            cell: self.cell,
        };
        self.mouse.switched(&context);
        let focused = self.caret.focused;
        if focused {
            self.tell_focus(false);
        }
        let TabTerminal { id, shared, input } = terminal;
        self.tab = id;
        self.shared = shared;
        self.input = input;
        if focused {
            self.tell_focus(true);
        }
        let (title, grid) = {
            let mut shared = lock(&self.shared);
            // The zoom, or the scale of the window, may have changed the
            // cells while the tab was hidden.
            shared.terminal.set_cell_size(self.cell);
            shared.title_changed = false;
            shared.clipboard_changed = false;
            let _ = shared.terminal.take_clipboard();
            (
                shared.terminal.title().map(str::to_owned),
                shared.terminal.size(),
            )
        };
        window.set_title(title.as_deref().unwrap_or(TITLE));
        // The next copy takes the whole screen, in the same memory, and the
        // frame draws the whole window, its margins too.
        self.screen.forget();
        self.redraw = true;
        // A new screen is activity: the cursor blinks again from the start.
        self.caret.since = now;
        // The terminal of a hidden tab keeps its grid. The one that fills
        // the window now goes to it at the next update, without waiting for
        // the frame interval since the last one.
        self.grid = grid;
        self.resized_at = None;
        self.resized(self.client);
    }

    /// A key went down or up. Ctrl+C copies the selection, if there is
    /// one; Ctrl+V pastes; Esc removes the selection, if there is one; on
    /// the primary screen, the keys of the history move its view; Ctrl and
    /// +, - or 0 zoom; and Ctrl+Shift+T, Ctrl+Shift+W, Ctrl+Tab and
    /// Ctrl+Shift+Tab are for the tabs. The rest go to the program, as the
    /// modes that it turned on say, and a key that goes down removes the
    /// selection and brings the view back to the screen, where the program
    /// writes, unless it is a modifier key.
    pub fn key(&mut self, window: &Window, event: &KeyEvent) -> Response {
        // A key that meiga took does not reach the program while it repeats,
        // nor when it goes up. The ones that move the view, zoom or go to
        // the next tab do it again while they repeat.
        if let Some(taken) = self.taken.iter().position(|&key| key == event.key) {
            if !event.down {
                self.taken.swap_remove(taken);
            } else if repeats(event.key) {
                return self.shortcut(window, event).unwrap_or_default();
            }
            return Response::default();
        }
        if event.down
            && let Some(response) = self.shortcut(window, event)
        {
            // With a message box, the key goes up in it.
            if !response.dialog {
                self.taken.push(event.key);
            }
            return response;
        }
        let mut response = Response::default();
        if event.down && !is_modifier(event.key) {
            let mut shared = lock(&self.shared);
            let cleared = shared.terminal.clear_selection();
            let back = shared.terminal.scroll(Scroll::Bottom);
            response.redraw = cleared || back;
        }
        let Some(input) = &self.input else {
            return response;
        };
        let modes = lock(&self.shared).terminal.modes();
        self.keys.clear();
        self.encoder.encode(event, &modes, &mut self.keys);
        input.write(&self.keys);
        response
    }

    /// What a key that goes down does if meiga takes it.
    fn shortcut(&mut self, window: &Window, event: &KeyEvent) -> Option<Response> {
        let controls = event.controls;
        let (ctrl, alt, shift) = (controls.ctrl(), controls.alt(), controls.shift());
        let tab = |action| {
            Some(Response {
                tab: Some(action),
                ..Response::default()
            })
        };
        match event.key {
            // The tabs, with fixed shortcuts until subphase 2.6, as in
            // Windows Terminal. Ctrl+T and Ctrl+W go to the program.
            VK_T if ctrl && shift && !alt => tab(TabAction::New),
            VK_W if ctrl && shift && !alt => tab(TabAction::Close),
            VK_TAB if ctrl && !alt && shift => tab(TabAction::Previous),
            VK_TAB if ctrl && !alt => tab(TabAction::Next),
            // Ctrl, and Shift or not, but not Alt, nor AltGr.
            VK_C if ctrl && !alt => {
                clipboard::copy(window, &self.shared, true).then_some(Response::REDRAW)
            }
            VK_V if ctrl && !alt => {
                let cleared = lock(&self.shared).terminal.clear_selection();
                let pasted = self.input.as_ref().map_or_else(Response::default, |input| {
                    clipboard::paste(window, &self.shared, input)
                });
                Some(Response {
                    redraw: cleared || pasted.redraw,
                    ..pasted
                })
            }
            VK_ESCAPE if !ctrl && !alt && !shift => lock(&self.shared)
                .terminal
                .clear_selection()
                .then_some(Response::REDRAW),
            // The history, as in Windows Terminal: Shift and Page Up or Page
            // Down, with Ctrl or without it, by pages; and Ctrl+Shift with
            // the arrows, by rows, or with Home and End, to its ends.
            VK_PRIOR if shift && !alt => self.scroll_history(Scroll::PageUp),
            VK_NEXT if shift && !alt => self.scroll_history(Scroll::PageDown),
            VK_UP if ctrl && shift && !alt => self.scroll_history(Scroll::Rows(1)),
            VK_DOWN if ctrl && shift && !alt => self.scroll_history(Scroll::Rows(-1)),
            VK_HOME if ctrl && shift && !alt => self.scroll_history(Scroll::Top),
            VK_END if ctrl && shift && !alt => self.scroll_history(Scroll::Bottom),
            // The zoom: Ctrl alone.
            VK_OEM_PLUS | VK_ADD if ctrl && !alt && !shift => Some(self.zoom(1)),
            VK_OEM_MINUS | VK_SUBTRACT if ctrl && !alt && !shift => Some(self.zoom(-1)),
            VK_0 | VK_NUMPAD0 if ctrl && !alt && !shift => Some(self.set_points(FONT_POINTS)),
            _ => None,
        }
    }

    /// Moves the view of the history, on the primary screen. On the
    /// alternate one, which has none, the key goes to the program.
    fn scroll_history(&self, scroll: Scroll) -> Option<Response> {
        let mut shared = lock(&self.shared);
        if shared.terminal.modes().alt_screen {
            return None;
        }
        Some(Response {
            redraw: shared.terminal.scroll(scroll),
            ..Response::default()
        })
    }

    /// Makes the font `steps` points bigger, or smaller if they are
    /// negative, from 6 to 72 points.
    fn zoom(&mut self, steps: i32) -> Response {
        let points = i32::from(self.points)
            .saturating_add(steps)
            .clamp(i32::from(MIN_POINTS), i32::from(MAX_POINTS));
        // Between 6 and 72.
        self.set_points(points as u16)
    }

    /// Draws with the font at `points`.
    fn set_points(&mut self, points: u16) -> Response {
        if points == self.points {
            return Response::default();
        }
        self.points = points;
        self.new_font();
        Response::REDRAW
    }

    /// The window moved to a monitor with `dpi`: a font, a scroll bar and
    /// a cursor for it. Its new size, if it changes, comes next, and gives
    /// the grid; if it does not change, the grid is the one of the new
    /// cells in the same window.
    pub fn dpi_changed(&mut self, dpi: u32) {
        if dpi == self.dpi {
            return;
        }
        self.dpi = dpi;
        self.caret.set_dpi(dpi);
        self.mouse.set_dpi(dpi);
        self.new_font();
    }

    /// Makes the font of the size and the scale that the view has now. The
    /// cells change size, and with them the grid that fills the window. If
    /// DirectWrite cannot make it, the old one stays.
    fn new_font(&mut self) {
        if let Ok(font) = Font::new(
            &self.fonts,
            &FONT_FAMILIES,
            f32::from(self.points),
            self.dpi,
        ) {
            self.font = font;
            self.cell = self.font.cell();
            lock(&self.shared).terminal.set_cell_size(self.cell);
        }
        self.redraw = true;
        self.resized(self.client);
    }

    /// The client area of the window has `client` pixels now: the grid
    /// that fills it is for the terminal and the program, at most once per
    /// frame interval.
    pub fn resized(&mut self, client: Size) {
        self.client = client;
        self.mouse.bar_mut().set_client(client);
        let grid = self.window_grid();
        self.wanted = (grid != self.grid).then_some(grid);
    }

    /// Gives the grid that fills the window to the terminal and to the
    /// program, if it changed, unless it changed less than a frame interval
    /// before `now`. Returns when it has to, then.
    fn resize_grid(&mut self, now: Instant) -> Option<Instant> {
        let grid = self.wanted?;
        if let Some(due) = self
            .resized_at
            .and_then(|at| at.checked_add(frames::interval()))
            && now < due
        {
            return Some(due);
        }
        self.wanted = None;
        self.resized_at = Some(now);
        self.grid = grid;
        let mut shared = lock(&self.shared);
        shared.terminal.resize(grid);
        // The program learns it with the lock held, so that no output of
        // it reaches the terminal in between. ConPTY does not wait for it.
        // If it fails, the program keeps the old size, and its text may
        // look wrong until the next one.
        if let Some(input) = &self.input {
            let _ = input.resize(grid);
        }
        None
    }

    /// A mouse event of the window. The wheel with Ctrl zooms.
    pub fn mouse(&mut self, window: &Window, event: &Event) -> Response {
        let context = Context {
            window,
            shared: &self.shared,
            input: self.input.as_ref(),
            cell: self.cell,
        };
        let mut response = self.mouse.event(&context, event);
        if response.zoom != 0 {
            response.redraw |= self.zoom(response.zoom).redraw;
            response.zoom = 0;
        }
        response
    }

    /// A message box of the view closed. While it was open, the events of
    /// the window did not reach the view: the keys that meiga took went up
    /// in the box, and the mouse buttons maybe too, so none is followed
    /// any more.
    pub fn dialog_closed(&mut self, window: &Window) {
        self.taken.clear();
        let context = Context {
            window,
            shared: &self.shared,
            input: self.input.as_ref(),
            cell: self.cell,
        };
        self.mouse.forget_buttons(&context);
    }

    /// The window gained or lost the focus, at `now`. With it, the cursor
    /// starts blinking again; without it, it is an outline that does not
    /// blink. The next update gives its cells as changed. The program
    /// learns it too, if it asked.
    pub fn focus(&mut self, focused: bool, now: Instant) {
        if focused == self.caret.focused {
            return;
        }
        // The keys that meiga took go up in another window.
        self.taken.clear();
        if focused {
            // The settings can change while meiga runs: for example, when
            // the user connects to the session through Remote Desktop.
            self.caret.read_settings();
            self.mouse.read_settings();
            self.caret.since = now;
        }
        self.caret.focused = focused;
        self.screen.mark_cursor_changed();
        self.tell_focus(focused);
    }

    /// Tells the program whether the window has the focus, if it asked.
    fn tell_focus(&self, focused: bool) {
        if let Some(input) = &self.input {
            let modes = lock(&self.shared).terminal.modes();
            input.write(focus_sequence(focused, &modes));
        }
    }

    /// Sends the program the answers that ending a synchronized update gave,
    /// and wakes the window up for the title, if it changed.
    fn send(&mut self, title: bool) {
        if !self.answers.is_empty() {
            if let Some(input) = &self.input {
                input.write(&self.answers);
            }
            self.answers.clear();
        }
        if title {
            // The window takes the title when it wakes up.
            self.waker.wake(self.tab.get());
        }
    }

    /// The pixels of `columns` of `row`.
    fn area(&self, row: u16, columns: Columns) -> Rect {
        let (width, height) = cell_size(self.cell);
        Rect::new(
            i32::from(columns.start) * width,
            i32::from(row) * height,
            i32::from(columns.end) * width,
            (i32::from(row) + 1) * height,
        )
    }
}

impl Scene for TerminalView {
    fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
        let resize = self.resize_grid(now);
        let (sync, title, tick, history) = {
            let mut shared = lock(&self.shared);
            // The view moves on its own while the left button holds the
            // scroll bar, or a selection outside the grid.
            let tick = self.mouse.tick(&mut shared.terminal, now, self.cell);
            // If the program did not end its synchronized update in time, it
            // is shown as it is.
            let (sync, title) = shared.end_late_sync(now, &mut self.answers);
            shared.terminal.copy_damage(&mut self.screen);
            shared.screen_changed = false;
            (sync, title, tick, shared.terminal.history())
        };
        self.send(title);

        // What the program shows is activity: the cursor blinks again from
        // the start, shown.
        if self.screen.all_changed() || self.screen.changes().next().is_some() {
            self.caret.since = now;
        }
        let (shown, blink) = self.caret.phase(self.screen.cursor(), now);
        if mem::replace(&mut self.caret.shown, shown) != shown {
            self.screen.mark_cursor_changed();
        }

        let rows = self.screen.size().rows;
        if let Some(area) = self.mouse.bar_mut().update(history, rows) {
            damage(area);
        }
        if mem::take(&mut self.redraw) || self.screen.all_changed() {
            // Also the margins, which have the colour of the background.
            damage(Rect::new(0, 0, i32::MAX, i32::MAX));
        } else {
            for (row, columns) in self.screen.changes() {
                damage(self.area(row, columns));
            }
        }
        self.screen.clear_changes();
        [sync, blink, resize, tick].into_iter().flatten().min()
    }

    fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
        // Only the synchronized update: the screen is copied when the
        // window is restored, and the cursor does not blink in a window
        // that is not shown.
        let (sync, title) = lock(&self.shared).end_late_sync(now, &mut self.answers);
        self.send(title);
        sync
    }

    fn draw(&mut self, canvas: &mut Canvas<'_>) {
        let Self {
            font,
            cell,
            screen,
            caret,
            run,
            mouse,
            ..
        } = self;
        let palette = screen.palette();
        let size = screen.size();
        let (width, height) = cell_size(*cell);
        let grid = Rect::new(
            0,
            0,
            i32::from(size.columns) * width,
            i32::from(size.rows) * height,
        );
        // The margins: the window can be larger than the grid.
        let clip = canvas.clip();
        canvas.fill(
            Rect::new(grid.right, clip.top, clip.right, clip.bottom),
            palette.background,
        );
        canvas.fill(
            Rect::new(clip.left, grid.bottom, grid.right, clip.bottom),
            palette.background,
        );

        let visible = clip.intersection(grid);
        if !visible.is_empty() {
            // `visible` is inside the grid, so the divisions are not
            // negative.
            let columns =
                (visible.left / width) as usize..((visible.right - 1) / width + 1) as usize;
            for row in visible.top / height..=(visible.bottom - 1) / height {
                let cells = screen.row(row as u16);
                draw_row(
                    canvas,
                    font,
                    run,
                    cells,
                    columns.clone(),
                    row * height,
                    palette,
                );
            }
            draw_cursor(canvas, font, screen, caret, *cell);
        }
        mouse.bar().draw(canvas, palette.background);
    }
}

/// The grid of cells of `cell` pixels that fills a client area of `client`
/// pixels, without the strip of the scroll bar, of `bar` pixels: at least
/// the smallest one of the terminal.
pub fn grid_size(client: Size, cell: Size, bar: u32) -> GridSize {
    let cells = |pixels: u32, cell: u32| u16::try_from(pixels / cell.max(1)).unwrap_or(u16::MAX);
    GridSize::new(
        cells(client.width.saturating_sub(bar), cell.width).max(MIN_SIZE.columns),
        cells(client.height, cell.height).max(MIN_SIZE.rows),
    )
}

/// Whether the action of `key`, when meiga takes it, goes on while it
/// repeats: the keys that move the view of the history, the ones that
/// zoom, and Tab, which goes from tab to tab.
const fn repeats(key: u16) -> bool {
    matches!(
        key,
        VK_PRIOR
            | VK_NEXT
            | VK_END
            | VK_HOME
            | VK_UP
            | VK_DOWN
            | VK_OEM_PLUS
            | VK_OEM_MINUS
            | VK_ADD
            | VK_SUBTRACT
            | VK_TAB
    )
}

/// The width and the height of `cell`, as coordinates. A cell is a few
/// dozen pixels.
fn cell_size(cell: Size) -> (i32, i32) {
    (cell.width as i32, cell.height as i32)
}

/// Whether `key` is only a modifier or a lock key, or Print Screen: they do
/// not remove the selection.
const fn is_modifier(key: u16) -> bool {
    matches!(
        key,
        // Shift, Ctrl, Alt, Caps Lock, Print Screen, the Windows keys, Num
        // Lock, Scroll Lock, and the left and right Shift, Ctrl and Alt.
        0x10..=0x12 | 0x14 | 0x2c | 0x5b | 0x5c | 0x90 | 0x91 | 0xa0..=0xa5
    )
}

/// How a cell is drawn.
struct Style {
    font: FontStyle,
    text: Rgb,
    background: Rgb,
    /// The text is not drawn (SGR 8), only the background.
    hidden: bool,
}

impl Style {
    fn of(cell: &Cell, palette: &Palette) -> Self {
        let flags = cell.flags;
        let mut text = cell.fg.rgb(palette, palette.foreground);
        let mut background = cell.bg.rgb(palette, palette.background);
        if flags.contains(CellFlags::INVERSE) {
            mem::swap(&mut text, &mut background);
        }
        if flags.contains(CellFlags::DIM) {
            text = text.mix(background, DIM);
        }
        // The selected text keeps its colour.
        if flags.contains(CellFlags::SELECTED) {
            background = palette.selection;
        }
        Self {
            font: FontStyle {
                bold: flags.contains(CellFlags::BOLD),
                italic: flags.contains(CellFlags::ITALIC),
            },
            text,
            background,
            hidden: flags.contains(CellFlags::HIDDEN),
        }
    }
}

/// Draws `columns` of the row of `cells` whose top is at `top`: in runs of
/// cells with the same colours and styles, each with one call to the font.
fn draw_row(
    canvas: &mut Canvas<'_>,
    font: &mut Font,
    run: &mut Vec<CellChar>,
    cells: &[Cell],
    columns: Range<usize>,
    top: i32,
    palette: &Palette,
) {
    let (width, height) = cell_size(font.cell());
    let end = columns.end.min(cells.len());
    let mut start = columns.start;
    while start < end {
        let first = &cells[start];
        let same =
            |cell: &Cell| cell.fg == first.fg && cell.bg == first.bg && cell.flags == first.flags;
        let stop = cells[start + 1..end]
            .iter()
            .position(|cell| !same(cell))
            .map_or(end, |count| start + 1 + count);
        let style = Style::of(first, palette);
        run.clear();
        if style.hidden {
            run.resize(stop - start, CellChar::Blank);
        } else {
            run.extend(cells[start..stop].iter().map(|cell| cell.char));
        }
        // The columns of a row fit in a u16, and so their pixels in an i32.
        let left = start as i32 * width;
        font.draw(
            canvas,
            Point::new(left, top),
            run,
            style.font,
            style.text,
            style.background,
        );
        if !style.hidden {
            let area = Rect::new(left, top, stop as i32 * width, top + height);
            draw_lines(
                canvas,
                first.flags,
                area,
                font.decorations(),
                style.text,
                font.cell(),
            );
        }
        start = stop;
    }
}

/// Draws the lines of the cells of `area` with `flags`: the underlines and
/// the strikeout, in `color`.
fn draw_lines(
    canvas: &mut Canvas<'_>,
    flags: CellFlags,
    area: Rect,
    decorations: Decorations,
    color: Rgb,
    cell: Size,
) {
    let Decorations {
        underline,
        strikeout,
    } = decorations;
    // A cell is a few dozen pixels.
    let thickness = underline.thickness as i32;
    let line = |canvas: &mut Canvas<'_>, top: u32, height: u32| {
        let top = area.top + top as i32;
        canvas.fill(
            Rect::new(area.left, top, area.right, top + height as i32),
            color,
        );
    };
    if flags.contains(CellFlags::UNDERLINE) {
        line(canvas, underline.top, underline.thickness);
    }
    if flags.contains(CellFlags::DOUBLE_UNDERLINE) {
        // Two lines with a gap as thick as them, higher if they do not fit.
        let top = underline
            .top
            .min(cell.height.saturating_sub(3 * underline.thickness));
        line(canvas, top, underline.thickness);
        line(canvas, top + 2 * underline.thickness, underline.thickness);
    }
    let top = area.top + underline.top as i32;
    if flags.contains(CellFlags::DOTTED_UNDERLINE) {
        // Dots as long as the line is thick, with gaps as long, at the same
        // places of the row however much of it a frame draws: from the left
        // of the window, not of the area.
        let period = 2 * thickness;
        let first = area.left - area.left.rem_euclid(period);
        for left in (first..area.right).step_by(period as usize) {
            let dot = Rect::new(
                left.max(area.left),
                top,
                (left + thickness).min(area.right),
                top + thickness,
            );
            canvas.fill(dot, color);
        }
    }
    if flags.contains(CellFlags::DASHED_UNDERLINE) {
        // A dash in the middle of each cell, two thirds of its width.
        let width = cell.width as i32;
        let dash = (width * 2 / 3).max(1);
        for left in (area.left..area.right).step_by(width.max(1) as usize) {
            let left = left + (width - dash) / 2;
            canvas.fill(Rect::new(left, top, left + dash, top + thickness), color);
        }
    }
    if flags.contains(CellFlags::UNDERCURL) {
        // A zigzag with a period of a cell, as high as the space below the
        // underline allows.
        let period = (cell.width as i32).max(4);
        let height = (cell.height as i32 / 8).max(thickness);
        let top = top.min(area.top + cell.height as i32 - height - thickness);
        let half = period / 2;
        for x in area.left..area.right {
            let phase = x.rem_euclid(period);
            let rise = if phase < half { phase } else { period - phase };
            let y = top + rise * height / half;
            canvas.fill(Rect::new(x, y, x + 1, y + thickness), color);
        }
    }
    if flags.contains(CellFlags::STRIKEOUT) {
        line(canvas, strikeout.top, strikeout.thickness);
    }
}

/// Draws the cursor, if it is shown, over the cells that are already drawn.
fn draw_cursor(
    canvas: &mut Canvas<'_>,
    font: &mut Font,
    screen: &Screen,
    caret: &Caret,
    cell: Size,
) {
    let cursor = screen.cursor();
    if cursor.shape == CursorShape::Hidden || !caret.shown {
        return;
    }
    let cells = screen.row(cursor.row);
    let column = usize::from(cursor.column);
    let Some(under) = cells.get(column) else {
        return;
    };
    // Over a wide character, it takes its two cells.
    let wide = matches!(under.char, CellChar::WideLeft(_));
    let (width, height) = cell_size(cell);
    let (x, y) = (
        i32::from(cursor.column) * width,
        i32::from(cursor.row) * height,
    );
    let area = Rect::new(x, y, x + if wide { 2 * width } else { width }, y + height);
    if !area.intersects(canvas.clip()) {
        return;
    }
    let palette = screen.palette();
    let color = palette.cursor;
    let thickness = caret.thickness as i32;
    let shape = if caret.focused {
        cursor.shape
    } else {
        CursorShape::HollowBlock
    };
    match shape {
        CursorShape::Beam => canvas.fill(Rect::new(x, y, x + thickness, y + height), color),
        CursorShape::Underline => canvas.fill(
            Rect::new(x, area.bottom - thickness, area.right, area.bottom),
            color,
        ),
        CursorShape::HollowBlock => canvas.outline(area, caret.thickness, color),
        CursorShape::Block => {
            // The cells under it, with the colour of their background on the
            // one of the cursor.
            let style = Style::of(under, palette);
            let count = if wide { 2 } else { 1 };
            let mut chars = [CellChar::Blank; 2];
            if !style.hidden {
                for (char, cell) in chars.iter_mut().zip(&cells[column..]) {
                    *char = cell.char;
                }
            }
            font.draw(
                canvas,
                Point::new(x, y),
                &chars[..count],
                style.font,
                style.background,
                color,
            );
        }
        CursorShape::Hidden => {}
    }
}

/// The width of the caret of `settings` at `dpi`, rounded, and at least one
/// pixel.
fn thickness(settings: CaretSettings, dpi: u32) -> u32 {
    let scaled = (settings.width.saturating_mul(dpi) + BASE_DPI / 2) / BASE_DPI;
    scaled.max(1)
}

/// How the cursor looks and blinks.
struct Caret {
    settings: CaretSettings,
    dpi: u32,
    /// The thickness of the bar, of the underline and of the outline, in
    /// pixels: the width of the caret of Windows, at the scale of the window.
    thickness: u32,
    focused: bool,
    /// When it started to blink: at the last activity. It stops after the
    /// timeout of the caret, shown.
    since: Instant,
    /// Whether it is drawn now: not in the hidden half of a blink.
    shown: bool,
}

impl Caret {
    fn new(dpi: u32, now: Instant) -> Self {
        let settings = window::caret_settings();
        Self {
            settings,
            dpi,
            thickness: thickness(settings, dpi),
            focused: false,
            since: now,
            shown: true,
        }
    }

    fn read_settings(&mut self) {
        self.settings = window::caret_settings();
        self.thickness = thickness(self.settings, self.dpi);
    }

    fn set_dpi(&mut self, dpi: u32) {
        self.dpi = dpi;
        self.thickness = thickness(self.settings, dpi);
    }

    /// Whether `cursor` is shown at `now`, and when that changes next, while
    /// it blinks. It only blinks if Windows allows it, if the window has the
    /// focus and if the program did not ask for a steady cursor, and only
    /// until the timeout of the caret after the last activity.
    fn phase(&self, cursor: Cursor, now: Instant) -> (bool, Option<Instant>) {
        let blinks = self.focused && cursor.blinking && cursor.shape != CursorShape::Hidden;
        let Some(period) = self.settings.blink.filter(|_| blinks) else {
            return (true, None);
        };
        let Some(end) = self.since.checked_add(self.settings.timeout) else {
            return (true, None);
        };
        if now >= end || period.is_zero() {
            return (true, None);
        }
        let elapsed = now.saturating_duration_since(self.since);
        // The halves of a blink so far: fewer than ten with the defaults of
        // Windows, and they fit in a u32 with any timeout of Windows, in
        // milliseconds.
        let halves = u32::try_from(elapsed.as_nanos() / period.as_nanos()).unwrap_or(u32::MAX);
        let next = period
            .checked_mul(halves.saturating_add(1))
            .and_then(|offset| self.since.checked_add(offset))
            .map_or(end, |next| next.min(end));
        (halves.is_multiple_of(2), Some(next))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use meiga_core::color::{BACKGROUND, FOREGROUND};
    use meiga_core::term::Color;
    use meiga_render::{Line, pixel};

    use super::*;

    fn caret(blink: Option<Duration>) -> (Caret, Instant) {
        let now = Instant::now();
        let caret = Caret {
            settings: CaretSettings {
                blink,
                timeout: Duration::from_millis(5000),
                width: 1,
            },
            dpi: 96,
            thickness: 1,
            focused: true,
            since: now,
            shown: true,
        };
        (caret, now)
    }

    const BAR: Cursor = Cursor {
        row: 0,
        column: 0,
        shape: CursorShape::Beam,
        blinking: true,
    };

    #[test]
    fn the_cursor_blinks_until_the_timeout_and_stays_shown() {
        let (caret, start) = caret(Some(Duration::from_millis(500)));
        let at = |ms| start + Duration::from_millis(ms);
        assert_eq!(caret.phase(BAR, at(0)), (true, Some(at(500))));
        assert_eq!(caret.phase(BAR, at(499)), (true, Some(at(500))));
        assert_eq!(caret.phase(BAR, at(500)), (false, Some(at(1000))));
        assert_eq!(caret.phase(BAR, at(1200)), (true, Some(at(1500))));
        assert_eq!(caret.phase(BAR, at(1700)), (false, Some(at(2000))));
        // The last change is the end, when it is shown for good.
        assert_eq!(caret.phase(BAR, at(4800)), (false, Some(at(5000))));
        assert_eq!(caret.phase(BAR, at(5000)), (true, None));
        assert_eq!(caret.phase(BAR, at(60_000)), (true, None));
    }

    #[test]
    fn the_cursor_does_not_blink_unless_everything_allows_it() {
        let (caret, now) = caret(Some(Duration::from_millis(500)));
        let steady = Cursor {
            blinking: false,
            ..BAR
        };
        assert_eq!(caret.phase(steady, now), (true, None));
        let hidden = Cursor {
            shape: CursorShape::Hidden,
            ..BAR
        };
        assert_eq!(caret.phase(hidden, now), (true, None));
        // Windows does not allow it, for example through Remote Desktop.
        let (still, now) = self::caret(None);
        assert_eq!(still.phase(BAR, now), (true, None));
        // Without the focus.
        let (mut unfocused, now) = self::caret(Some(Duration::from_millis(500)));
        unfocused.focused = false;
        assert_eq!(unfocused.phase(BAR, now), (true, None));
    }

    #[test]
    fn a_dotted_underline_keeps_its_dots_when_a_frame_draws_part_of_it() {
        // Four cells of 9 × 19 pixels, as at a scale of 100 %.
        let cell = Size::new(9, 19);
        let size = Size::new(36, 19);
        for thickness in [1, 2] {
            let decorations = Decorations {
                underline: Line { top: 15, thickness },
                strikeout: Line { top: 9, thickness },
            };
            let draw = |area: Rect| {
                let mut pixels = vec![0; 36 * 19];
                let mut canvas = Canvas::new(&mut pixels, size, 36).unwrap();
                let flags = CellFlags::DOTTED_UNDERLINE;
                draw_lines(&mut canvas, flags, area, decorations, FOREGROUND, cell);
                pixels[15 * 36..16 * 36].to_vec()
            };
            let whole = draw(Rect::new(0, 0, 36, 19));
            let dots: Vec<bool> = whole.iter().map(|&p| p == pixel(FOREGROUND)).collect();
            let expected: Vec<bool> = (0..36).map(|x| x % (2 * thickness) < thickness).collect();
            assert_eq!(dots, expected, "thickness {thickness}");
            // A frame that draws only the second and the third cell, which
            // start at an odd pixel, draws the same dots there.
            let part = draw(Rect::new(9, 0, 27, 19));
            assert_eq!(part[9..27], whole[9..27], "thickness {thickness}");
            assert!(part[..9].iter().chain(&part[27..]).all(|&p| p == 0));
        }
    }

    #[test]
    fn styles_give_the_colours_of_the_text_and_its_background() {
        let palette = Palette::MEIGA;
        let cell = |fg, bg, flags| Cell {
            char: CellChar::Narrow('a'),
            fg,
            bg,
            flags,
        };
        let plain = Style::of(
            &cell(Color::Default, Color::Default, CellFlags::empty()),
            &palette,
        );
        assert_eq!((plain.text, plain.background), (FOREGROUND, BACKGROUND));
        assert_eq!(plain.font, FontStyle::REGULAR);

        let red = Rgb::new(0xc5, 0x0f, 0x1f);
        let inverse = Style::of(
            &cell(
                Color::Indexed(1),
                Color::Default,
                CellFlags::INVERSE | CellFlags::BOLD,
            ),
            &palette,
        );
        assert_eq!((inverse.text, inverse.background), (BACKGROUND, red));
        assert!(inverse.font.bold && !inverse.font.italic);

        let rgb = Rgb::new(10, 20, 30);
        let dim = Style::of(
            &cell(
                Color::Rgb(rgb),
                Color::Indexed(15),
                CellFlags::DIM | CellFlags::HIDDEN,
            ),
            &palette,
        );
        assert_eq!(dim.text, rgb.mix(palette.colors[15], DIM));
        assert_eq!(dim.background, palette.colors[15]);
        assert!(dim.hidden);
    }
}
