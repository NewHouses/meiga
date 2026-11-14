//! The mouse in the terminal: the clicks that select text, and the ones
//! that go to the program, if it asked for the mouse; the scroll bar; and
//! the wheel, which moves the view of the history, or zooms with Ctrl.
//!
//! As in Windows Terminal, Shift keeps the mouse for meiga: with it, the
//! clicks select text also in a program that asked for the mouse. A button
//! whose press went to the program sends it its moves and its release too,
//! with Shift or without it. If the window loses the mouse first, the
//! program gets the release then. The scroll bar is always for meiga.

use std::mem;
use std::sync::Mutex;
use std::time::Instant;

use meiga_core::geometry::{GridPoint, GridSize, Point, Size};
use meiga_core::input::KeyEncoder;
use meiga_core::mouse::{
    self, Button, Clicks, Modifiers, MouseAction, Wheel, WheelRows, WheelTurns, arrow_keys, cell_at,
};
use meiga_core::term::{Modes, Scroll, SelectionKind, Side, Terminal};
use meiga_win::conpty::Input;
use meiga_win::event::Event;
use meiga_win::window::{self, MouseSettings, Pointer, Window};

use crate::clipboard;
use crate::frames;
use crate::scrollbar::ScrollBar;
use crate::session::{Shared, lock};
use crate::tabs::TabAction;

/// The dots per inch at a scale of 100 %.
const BASE_DPI: u32 = 96;

/// What the mouse needs to handle an event.
pub struct Context<'a> {
    pub window: &'a Window,
    pub shared: &'a Mutex<Shared>,
    /// The input of the program, or `None` if it did not start.
    pub input: Option<&'a Input>,
    /// The size of a cell, in pixels.
    pub cell: Size,
}

/// What the window has to do after an event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Response {
    /// The selection or the view changed: a frame has to show it.
    pub redraw: bool,
    /// A message box was shown, and meanwhile the events of the window did
    /// not reach it.
    pub dialog: bool,
    /// The wheel turned with Ctrl these notches, which zoom: in when they
    /// are positive.
    pub zoom: i32,
    /// A shortcut of the tabs.
    pub tab: Option<TabAction>,
}

impl Response {
    /// Only a frame.
    pub const REDRAW: Self = Self {
        redraw: true,
        dialog: false,
        zoom: 0,
        tab: None,
    };
}

/// The state of the mouse.
pub struct Mouse {
    settings: MouseSettings,
    clicks: Clicks,
    /// The left button is down, selecting text: the cell and the side where
    /// the selection ends.
    selecting: Option<(GridPoint, Side)>,
    /// The buttons whose press went to the program: left, middle and
    /// right.
    reported: [bool; 3],
    /// The cell of the last move that went to the program: it only gets
    /// the moves to another cell.
    reported_at: Option<GridPoint>,
    wheel: WheelTurns,
    /// The turns of the wheel that move the view of the history.
    wheel_rows: WheelRows,
    /// The arrows of the wheel on the alternate screen.
    encoder: KeyEncoder,
    /// What goes to the program, kept to avoid an allocation each time.
    bytes: Vec<u8>,
    bar: ScrollBar,
    /// While a selection grows with the pointer above or below the grid,
    /// the view moves on its own.
    autoscroll: Option<AutoScroll>,
    /// The dots per inch of the window.
    dpi: u32,
}

/// The view moving on its own while a selection grows outside the grid.
#[derive(Clone, Copy, Debug)]
struct AutoScroll {
    /// Rows per second, towards older rows when it is positive.
    speed: f32,
    /// Where the pointer is.
    at: Point,
    /// When the view last moved, and the part of a row that it did not
    /// move then.
    last: Instant,
    carry: f32,
    /// Whether it can still move: not once it reached the end.
    running: bool,
}

impl Mouse {
    /// The mouse of a window of `client` pixels at `dpi`.
    pub fn new(dpi: u32, client: Size) -> Self {
        Self {
            settings: window::mouse_settings(),
            clicks: Clicks::new(),
            selecting: None,
            reported: [false; 3],
            reported_at: None,
            wheel: WheelTurns::new(),
            wheel_rows: WheelRows::new(),
            encoder: KeyEncoder::new(),
            bytes: Vec::new(),
            bar: ScrollBar::new(dpi, client),
            autoscroll: None,
            dpi,
        }
    }

    pub const fn bar(&self) -> &ScrollBar {
        &self.bar
    }

    pub const fn bar_mut(&mut self) -> &mut ScrollBar {
        &mut self.bar
    }

    /// The window moved to a monitor with `dpi`.
    pub fn set_dpi(&mut self, dpi: u32) {
        self.dpi = dpi;
        self.bar.set_dpi(dpi);
    }

    /// Reads again the settings of the mouse, which can change while meiga
    /// runs.
    pub fn read_settings(&mut self) {
        self.settings = window::mouse_settings();
    }

    /// Handles a mouse event of the window.
    pub fn event(&mut self, context: &Context<'_>, event: &Event) -> Response {
        match *event {
            Event::MouseDown {
                button,
                at,
                modifiers,
            } => self.down(context, button, at, modifiers),
            Event::MouseUp {
                button,
                at,
                modifiers,
            } => self.up(context, button, at, modifiers),
            Event::MouseMoved {
                at,
                button,
                modifiers,
            } => self.moved(context, at, button, modifiers),
            Event::Wheel {
                at,
                delta,
                horizontal,
                modifiers,
            } => self.turned(context, at, delta, horizontal, modifiers),
            Event::CaptureLost => {
                self.forget_buttons(context);
                Response::default()
            }
            Event::MouseLeft => {
                if self.bar.hover(false) {
                    Response::REDRAW
                } else {
                    Response::default()
                }
            }
            _ => Response::default(),
        }
    }

    fn down(
        &mut self,
        context: &Context<'_>,
        button: Button,
        at: Point,
        modifiers: Modifiers,
    ) -> Response {
        // The scroll bar is for meiga, unless another button is already
        // selecting or reporting.
        if self.bar.contains(at) && self.selecting.is_none() && !self.reported.contains(&true) {
            if button == Button::Left && !self.bar.is_pressed() {
                let mut shared = lock(context.shared);
                self.bar.press(at, &mut shared.terminal, Instant::now());
                drop(shared);
                if self.bar.is_pressed() {
                    context.window.capture_mouse();
                }
            }
            return Response::REDRAW;
        }
        let mut shared = lock(context.shared);
        let modes = shared.terminal.modes();
        let (cell, side) = cell_at(at, context.cell, shared.terminal.size());
        if mouse::is_tracked(&modes) && !modifiers.shift {
            drop(shared);
            self.report(context, MouseAction::Press(button), cell, modifiers, &modes);
            self.reported[index(button)] = true;
            self.reported_at = Some(cell);
            context.window.capture_mouse();
            return Response::default();
        }
        match button {
            Button::Left => {
                let kind =
                    match self
                        .clicks
                        .press(button, at, Instant::now(), &self.settings.clicks)
                    {
                        1 => SelectionKind::Cells,
                        2 => SelectionKind::Words,
                        _ => SelectionKind::Lines,
                    };
                shared.terminal.select(kind, cell, side);
                self.selecting = Some((cell, side));
                context.window.capture_mouse();
                Response::REDRAW
            }
            // As in Windows Terminal with `copyOnSelect`: the selection
            // went to the clipboard when the left button went up, or goes
            // now if it is still growing. It goes away, and the clipboard
            // is pasted: what the user copied last, maybe in another
            // program.
            Button::Right => {
                drop(shared);
                let redraw = if self.selecting.take().is_some() {
                    self.autoscroll = None;
                    self.release(context);
                    clipboard::copy(context.window, context.shared, true)
                } else {
                    lock(context.shared).terminal.clear_selection()
                };
                let pasted = context.input.map_or_else(Response::default, |input| {
                    clipboard::paste(context.window, context.shared, input)
                });
                Response {
                    redraw: redraw || pasted.redraw,
                    ..pasted
                }
            }
            Button::Middle => Response::default(),
        }
    }

    fn up(
        &mut self,
        context: &Context<'_>,
        button: Button,
        at: Point,
        modifiers: Modifiers,
    ) -> Response {
        if button == Button::Left && self.bar.is_pressed() {
            self.bar.release();
            self.release(context);
            // Holding the mouse may have ended the tracking of its leave.
            if self.bar.contains(at) {
                context.window.track_mouse_leave();
            }
            self.bar.hover(self.bar.contains(at));
            return Response::REDRAW;
        }
        if self.reported[index(button)] {
            self.reported[index(button)] = false;
            let (modes, cell) = {
                let shared = lock(context.shared);
                let (cell, _) = cell_at(at, context.cell, shared.terminal.size());
                (shared.terminal.modes(), cell)
            };
            self.report(
                context,
                MouseAction::Release(button),
                cell,
                modifiers,
                &modes,
            );
            self.release(context);
        } else if button == Button::Left && self.selecting.take().is_some() {
            self.autoscroll = None;
            self.release(context);
            // What is selected goes to the clipboard, and stays selected.
            clipboard::copy(context.window, context.shared, false);
        }
        Response::default()
    }

    fn moved(
        &mut self,
        context: &Context<'_>,
        at: Point,
        button: Option<Button>,
        modifiers: Modifiers,
    ) -> Response {
        let mut shared = lock(context.shared);
        if self.bar.is_pressed() {
            context.window.set_pointer(Pointer::Arrow);
            self.bar.drag(at, &mut shared.terminal, Instant::now());
            return Response::REDRAW;
        }
        let modes = shared.terminal.modes();
        let tracked = mouse::is_tracked(&modes);
        let free = self.selecting.is_none() && !self.reported.contains(&true);
        let on_bar = free && self.bar.contains(at);
        // The shape is the one of the next move: Windows asks for it first.
        context
            .window
            .set_pointer(if on_bar || (tracked && !modifiers.shift) {
                Pointer::Arrow
            } else {
                Pointer::Text
            });
        let size = shared.terminal.size();
        let (cell, side) = cell_at(at, context.cell, size);
        if let Some(end) = self.selecting {
            let started = self.follow(at, size, context.cell, Instant::now());
            if end == (cell, side) {
                return Response {
                    redraw: started,
                    ..Response::default()
                };
            }
            self.selecting = Some((cell, side));
            shared.terminal.select_to(cell, side);
            return Response::REDRAW;
        }
        drop(shared);
        let mut response = Response::default();
        if free && self.bar.hover(on_bar) {
            if on_bar {
                context.window.track_mouse_leave();
            }
            response.redraw = true;
        }
        if on_bar {
            return response;
        }
        // The moves of a button whose press went to the program, or, if it
        // asked for all of them (`?1003`), the moves without buttons.
        let held = button.filter(|&button| self.reported[index(button)]);
        let action = match (held, button) {
            (Some(held), _) => MouseAction::Move(Some(held)),
            (None, None) if tracked && !modifiers.shift => MouseAction::Move(None),
            _ => return response,
        };
        if self.reported_at != Some(cell) {
            self.reported_at = Some(cell);
            self.report(context, action, cell, modifiers, &modes);
        }
        response
    }

    /// The wheel turned. The notches go to the program, if it asked for
    /// the mouse, unless Shift keeps them for meiga or the pointer is on the
    /// scroll bar. Otherwise, with Ctrl they zoom; on the alternate screen
    /// they go as arrows (`?1007`), as in `less`; and on the primary one
    /// they move the view of the history, by the lines of a notch that
    /// Windows says, or by pages.
    fn turned(
        &mut self,
        context: &Context<'_>,
        at: Point,
        delta: i16,
        horizontal: bool,
        modifiers: Modifiers,
    ) -> Response {
        let (modes, size) = {
            let shared = lock(context.shared);
            (shared.terminal.modes(), shared.terminal.size())
        };
        let for_meiga = modifiers.shift || self.bar.contains(at);
        self.bytes.clear();
        if mouse::is_tracked(&modes) && !for_meiga {
            let notches = self.wheel.add(delta, horizontal);
            let (cell, _) = cell_at(at, context.cell, size);
            for _ in 0..notches.unsigned_abs() {
                mouse::report(
                    MouseAction::Wheel(direction(horizontal, notches)),
                    cell,
                    modifiers,
                    &modes,
                    &mut self.bytes,
                );
            }
        } else if modifiers.ctrl {
            let notches = if horizontal {
                0
            } else {
                self.wheel.add(delta, false)
            };
            return Response {
                zoom: notches,
                ..Response::default()
            };
        } else if modes.alt_screen && modes.alternate_scroll && !for_meiga {
            let notches = self.wheel.add(delta, horizontal);
            let per_notch = if horizontal {
                self.settings.wheel_chars
            } else {
                // A notch that scrolls a page: the rows of the screen.
                self.settings.wheel_lines.unwrap_or(u32::from(size.rows))
            };
            for _ in 0..notches.unsigned_abs().saturating_mul(per_notch) {
                for key in &arrow_keys(direction(horizontal, notches)) {
                    self.encoder.encode(key, &modes, &mut self.bytes);
                }
            }
        } else if !horizontal {
            return self.scroll_history(context, delta);
        }
        if let Some(input) = context.input
            && !self.bytes.is_empty()
        {
            input.write(&self.bytes);
        }
        Response::default()
    }

    /// Moves the view of the history for a turn of the wheel of `delta`.
    fn scroll_history(&mut self, context: &Context<'_>, delta: i16) -> Response {
        let mut shared = lock(context.shared);
        let moved = match self.settings.wheel_lines {
            Some(lines) => {
                let rows = self.wheel_rows.add(delta, lines);
                rows != 0 && shared.terminal.scroll(Scroll::Rows(rows))
            }
            None => {
                let notches = self.wheel.add(delta, false);
                let page = if notches > 0 {
                    Scroll::PageUp
                } else {
                    Scroll::PageDown
                };
                (0..notches.unsigned_abs())
                    .fold(false, |moved, _| shared.terminal.scroll(page) || moved)
            }
        };
        // A selection that is growing ends under the pointer, in the rows
        // that the view shows now.
        if moved && let Some((cell, side)) = self.selecting {
            shared.terminal.select_to(cell, side);
        }
        Response {
            redraw: moved,
            ..Response::default()
        }
    }

    /// Moves the view on its own, if it has to at `now`: by pages while the
    /// left button stays down on the scroll bar, outside its thumb, or
    /// while a selection grows with the pointer above or below the grid.
    /// `cell` is the size of a cell. Returns when it has to move again.
    pub fn tick(&mut self, terminal: &mut Terminal, now: Instant, cell: Size) -> Option<Instant> {
        let paging = self.bar.tick(terminal, now);
        let selecting = self.autoscroll(terminal, now, cell);
        paging.into_iter().chain(selecting).min()
    }

    /// While a selection grows, with the pointer at `at`, on a grid of
    /// `size` cells of `cell` pixels: above or below the grid, the view
    /// moves on its own, faster the further the pointer is, as in Windows
    /// Terminal: 2 rows per second, and the square of the distance in
    /// pixels at 100 %, divided by 25. Returns whether it starts moving
    /// now, at `now`.
    fn follow(&mut self, at: Point, size: GridSize, cell: Size, now: Instant) -> bool {
        // A grid is far smaller than an i32.
        let bottom = i32::from(size.rows) * cell.height as i32;
        let distance = if at.y < 0 {
            at.y
        } else if at.y >= bottom {
            at.y - bottom + 1
        } else {
            self.autoscroll = None;
            return false;
        };
        let pixels = distance.unsigned_abs() as f32 * BASE_DPI as f32 / self.dpi.max(1) as f32;
        let speed = pixels * pixels / 25.0 + 2.0;
        let speed = if distance < 0 { speed } else { -speed };
        match &mut self.autoscroll {
            Some(scroll) => {
                scroll.speed = speed;
                scroll.at = at;
                let restarts = !scroll.running;
                if restarts {
                    scroll.running = true;
                    scroll.last = now;
                }
                restarts
            }
            None => {
                self.autoscroll = Some(AutoScroll {
                    speed,
                    at,
                    last: now,
                    carry: 0.0,
                    running: true,
                });
                true
            }
        }
    }

    /// Moves the view of `terminal` while a selection grows outside the
    /// grid, the rows that its speed gives since the last time, and the
    /// selection ends under the pointer. Returns when to do it again: at
    /// the next frame, until the view reaches the end of the history.
    fn autoscroll(&mut self, terminal: &mut Terminal, now: Instant, cell: Size) -> Option<Instant> {
        let scroll = self.autoscroll.as_mut().filter(|scroll| scroll.running)?;
        let elapsed = now.saturating_duration_since(scroll.last).as_secs_f32();
        scroll.last = now;
        let rows = scroll.speed * elapsed + scroll.carry;
        scroll.carry = rows.fract();
        // At most some thousands of rows per frame.
        let rows = rows.trunc() as i32;
        if rows != 0 {
            if !terminal.scroll(Scroll::Rows(rows)) {
                scroll.running = false;
                return None;
            }
            let end = cell_at(scroll.at, cell, terminal.size());
            terminal.select_to(end.0, end.1);
            self.selecting = Some(end);
        }
        now.checked_add(frames::interval())
    }

    /// The window can no longer follow the buttons: another window took the
    /// mouse, or a message box had it while it was open. The program gets
    /// the release of each button whose press it got, on the last cell that
    /// it got, the selection ends where it was, and the scroll bar lets its
    /// thumb go.
    pub fn forget_buttons(&mut self, context: &Context<'_>) {
        let pressed = mem::take(&mut self.reported);
        if let Some(cell) = self.reported_at
            && pressed.contains(&true)
        {
            let modes = lock(context.shared).terminal.modes();
            for button in [Button::Left, Button::Middle, Button::Right] {
                if pressed[index(button)] {
                    self.report(
                        context,
                        MouseAction::Release(button),
                        cell,
                        Modifiers::default(),
                        &modes,
                    );
                }
            }
        }
        self.autoscroll = None;
        if self.selecting.take().is_some() {
            clipboard::copy(context.window, context.shared, false);
        }
        self.bar.release();
        self.release(context);
    }

    /// The window shows another terminal now, instead of the one of
    /// `context`: what the mouse was doing with that one ends, as when
    /// another window takes the mouse, and what it counted for it does not
    /// go on in the other one, such as the clicks or the last cell that went
    /// to its program.
    pub fn switched(&mut self, context: &Context<'_>) {
        self.forget_buttons(context);
        self.clicks = Clicks::new();
        self.reported_at = None;
        self.wheel = WheelTurns::new();
        self.wheel_rows = WheelRows::new();
    }

    /// Sends the report of `action` on `cell` to the program, if it asked
    /// for it.
    fn report(
        &mut self,
        context: &Context<'_>,
        action: MouseAction,
        cell: GridPoint,
        modifiers: Modifiers,
        modes: &Modes,
    ) {
        let Some(input) = context.input else {
            return;
        };
        self.bytes.clear();
        // Each report goes in one write: the ConPTY reads it whole.
        if mouse::report(action, cell, modifiers, modes, &mut self.bytes) {
            input.write(&self.bytes);
        }
    }

    /// Lets the mouse go, if no button needs it any more.
    fn release(&self, context: &Context<'_>) {
        if self.selecting.is_none() && !self.reported.contains(&true) && !self.bar.is_pressed() {
            context.window.release_mouse();
        }
    }
}

/// Where `notches` of the wheel go: up or down, or, if `horizontal`, to
/// the right or to the left.
const fn direction(horizontal: bool, notches: i32) -> Wheel {
    match (horizontal, notches > 0) {
        (false, true) => Wheel::Up,
        (false, false) => Wheel::Down,
        (true, true) => Wheel::Right,
        (true, false) => Wheel::Left,
    }
}

const fn index(button: Button) -> usize {
    match button {
        Button::Left => 0,
        Button::Middle => 1,
        Button::Right => 2,
    }
}
