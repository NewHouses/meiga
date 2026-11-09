//! The mouse in the terminal: the clicks that select text, and the ones
//! that go to the program, if it asked for the mouse.
//!
//! As in Windows Terminal, Shift keeps the mouse for meiga: with it, the
//! clicks select text also in a program that asked for the mouse. A button
//! whose press went to the program sends it its moves and its release too,
//! with Shift or without it. If the window loses the mouse first, the
//! program gets the release then.

use std::mem;
use std::sync::Mutex;
use std::time::Instant;

use meiga_core::geometry::{GridPoint, Point, Size};
use meiga_core::input::KeyEncoder;
use meiga_core::mouse::{
    self, Button, Clicks, Modifiers, MouseAction, Wheel, WheelTurns, arrow_keys, cell_at,
};
use meiga_core::term::{Modes, SelectionKind, Side};
use meiga_win::conpty::Input;
use meiga_win::event::Event;
use meiga_win::window::{self, MouseSettings, Pointer, Window};

use crate::clipboard;
use crate::session::{Shared, lock};

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
    /// The selection changed: a frame has to show it.
    pub redraw: bool,
    /// A message box was shown, and meanwhile the events of the window did
    /// not reach it.
    pub dialog: bool,
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
    /// The arrows of the wheel on the alternate screen.
    encoder: KeyEncoder,
    /// What goes to the program, kept to avoid an allocation each time.
    bytes: Vec<u8>,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            settings: window::mouse_settings(),
            clicks: Clicks::new(),
            selecting: None,
            reported: [false; 3],
            reported_at: None,
            wheel: WheelTurns::new(),
            encoder: KeyEncoder::new(),
            bytes: Vec::new(),
        }
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
                Response {
                    redraw: true,
                    dialog: false,
                }
            }
            // As in Windows Terminal with `copyOnSelect`: the selection
            // went to the clipboard when the left button went up, or goes
            // now if it is still growing. It goes away, and the clipboard
            // is pasted: what the user copied last, maybe in another
            // program.
            Button::Right => {
                drop(shared);
                let redraw = if self.selecting.take().is_some() {
                    self.release(context);
                    clipboard::copy(context.window, context.shared, true)
                } else {
                    lock(context.shared).terminal.clear_selection()
                };
                let dialog = context
                    .input
                    .is_some_and(|input| clipboard::paste(context.window, context.shared, input));
                Response { redraw, dialog }
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
        let modes = shared.terminal.modes();
        let tracked = mouse::is_tracked(&modes);
        // The shape is the one of the next move: Windows asks for it first.
        context.window.set_pointer(if tracked && !modifiers.shift {
            Pointer::Arrow
        } else {
            Pointer::Text
        });
        let (cell, side) = cell_at(at, context.cell, shared.terminal.size());
        if let Some(end) = self.selecting {
            if end == (cell, side) {
                return Response::default();
            }
            self.selecting = Some((cell, side));
            shared.terminal.select_to(cell, side);
            return Response {
                redraw: true,
                dialog: false,
            };
        }
        drop(shared);
        // The moves of a button whose press went to the program, or, if it
        // asked for all of them (`?1003`), the moves without buttons.
        let held = button.filter(|&button| self.reported[index(button)]);
        let action = match (held, button) {
            (Some(held), _) => MouseAction::Move(Some(held)),
            (None, None) if tracked && !modifiers.shift => MouseAction::Move(None),
            _ => return Response::default(),
        };
        if self.reported_at != Some(cell) {
            self.reported_at = Some(cell);
            self.report(context, action, cell, modifiers, &modes);
        }
        Response::default()
    }

    /// The wheel turned: the notches go to the program, if it asked for
    /// the mouse, or, on the alternate screen, as arrows (`?1007`), as in
    /// `less`. Otherwise they will scroll the history (subphase 1.9).
    fn turned(
        &mut self,
        context: &Context<'_>,
        at: Point,
        delta: i16,
        horizontal: bool,
        modifiers: Modifiers,
    ) -> Response {
        let notches = self.wheel.add(delta, horizontal);
        let Some(input) = context.input.filter(|_| notches != 0 && !modifiers.shift) else {
            return Response::default();
        };
        let wheel = match (horizontal, notches > 0) {
            (false, true) => Wheel::Up,
            (false, false) => Wheel::Down,
            (true, true) => Wheel::Right,
            (true, false) => Wheel::Left,
        };
        let (modes, size) = {
            let shared = lock(context.shared);
            (shared.terminal.modes(), shared.terminal.size())
        };
        self.bytes.clear();
        if mouse::is_tracked(&modes) {
            let (cell, _) = cell_at(at, context.cell, size);
            for _ in 0..notches.unsigned_abs() {
                mouse::report(
                    MouseAction::Wheel(wheel),
                    cell,
                    modifiers,
                    &modes,
                    &mut self.bytes,
                );
            }
        } else if modes.alt_screen && modes.alternate_scroll {
            let per_notch = if horizontal {
                self.settings.wheel_chars
            } else {
                // A notch that scrolls a page: the rows of the screen.
                self.settings.wheel_lines.unwrap_or(u32::from(size.rows))
            };
            for _ in 0..notches.unsigned_abs().saturating_mul(per_notch) {
                for key in &arrow_keys(wheel) {
                    self.encoder.encode(key, &modes, &mut self.bytes);
                }
            }
        }
        if !self.bytes.is_empty() {
            input.write(&self.bytes);
        }
        Response::default()
    }

    /// The window can no longer follow the buttons: another window took the
    /// mouse, or a message box had it while it was open. The program gets
    /// the release of each button whose press it got, on the last cell that
    /// it got, and the selection ends where it was.
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
        if self.selecting.take().is_some() {
            clipboard::copy(context.window, context.shared, false);
        }
        self.release(context);
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
        if self.selecting.is_none() && !self.reported.contains(&true) {
            context.window.release_mouse();
        }
    }
}

const fn index(button: Button) -> usize {
    match button {
        Button::Left => 0,
        Button::Middle => 1,
        Button::Right => 2,
    }
}
