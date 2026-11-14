//! The handler of the window: it takes the events to the content and to the
//! frames.

use std::time::Instant;

use meiga_core::geometry::Rect;
use meiga_core::model::TabId;
use meiga_render::{Canvas, Font, FontSystem};
use meiga_win::HDC;
use meiga_win::event::{Event, Handler, HandlerError};
use meiga_win::window::Window;

use crate::bench::Bench;
use crate::frames::{Frames, Scene};
use crate::mouse::Response;
use crate::options::{BenchMode, Options};
use crate::tabs::Tabs;
use crate::trace::Trace;
use crate::view::{FONT_FAMILIES, FONT_POINTS};

/// The title of the window while the program does not give one.
pub const TITLE: &str = "meiga";

/// What the window shows. Both hold a whole font, so they go in boxes.
enum Content {
    Terminal(Box<Tabs>),
    Bench(Box<Bench>),
}

impl Scene for Content {
    fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
        match self {
            Content::Terminal(tabs) => tabs.update(now, damage),
            Content::Bench(_) => None,
        }
    }

    fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
        match self {
            Content::Terminal(tabs) => tabs.update_hidden(now),
            Content::Bench(_) => None,
        }
    }

    fn draw(&mut self, canvas: &mut Canvas<'_>) {
        match self {
            Content::Terminal(tabs) => tabs.draw(canvas),
            Content::Bench(bench) => bench.draw(canvas),
        }
    }
}

pub struct App {
    frames: Frames,
    /// The tabs own their sessions, which close when they are dropped.
    content: Content,
}

impl App {
    /// The content of `window`, with the first frame composed: the terminal
    /// with the program of `options` running in its first tab, or a
    /// benchmark, which starts its frames as soon as the window runs.
    pub fn new(
        window: &Window,
        options: &Options,
        trace: Option<Trace>,
    ) -> Result<Self, HandlerError> {
        // Only the content with text reads the fonts of the system.
        let mut content = match options.bench {
            Some(BenchMode::Text) => {
                let system = FontSystem::new()?;
                let font = Font::new(
                    &system,
                    &FONT_FAMILIES,
                    f32::from(FONT_POINTS),
                    window.dpi(),
                )?;
                Content::Bench(Box::new(Bench::new(BenchMode::Text, Some(font))))
            }
            Some(mode) => Content::Bench(Box::new(Bench::new(mode, None))),
            None => Content::Terminal(Box::new(Tabs::new(window, options.command.as_deref())?)),
        };
        let mut frames = Frames::new(window, &mut content, trace, options.marks)?;
        if options.bench.is_some() {
            frames.schedule(window)?;
        }
        Ok(Self { frames, content })
    }

    const fn is_bench(&self) -> bool {
        matches!(self.content, Content::Bench(_))
    }

    /// The reader thread of the session of a tab, whose identifier is
    /// `tab`, woke the window up: its title, its screen, the clipboard, a
    /// synchronized update, or more than one, changed.
    fn wake(&mut self, window: &Window, tab: u64) -> Result<(), HandlerError> {
        let Content::Terminal(tabs) = &mut self.content else {
            return Ok(());
        };
        if tabs.wake(window, TabId::from_raw(tab), Instant::now()) {
            self.frames.changed(window, &mut self.content)?;
        }
        Ok(())
    }

    /// Does what the view asks after a key or a mouse event.
    fn respond(&mut self, window: &Window, response: Response) -> Result<(), HandlerError> {
        let mut changed = response.redraw;
        if response.dialog {
            if let Content::Terminal(tabs) = &mut self.content {
                tabs.view_mut().dialog_closed(window);
            }
            self.catch_up(window)?;
            // While the message box was open, the window lost its wake-ups,
            // and maybe some paint: what changed meanwhile is shown now.
            window.invalidate(None);
            if let Content::Terminal(tabs) = &mut self.content {
                changed |= tabs.catch_up_wakes(window, Instant::now());
            }
        }
        if let Some(action) = response.tab
            && let Content::Terminal(tabs) = &mut self.content
        {
            changed |= tabs.act(window, action, Instant::now());
        }
        if changed {
            self.frames.changed(window, &mut self.content)?;
        }
        Ok(())
    }

    /// While a message box of the view was open, the events of the window
    /// did not reach the view: the window may have another scale or size
    /// now. They are taken as if they had just changed.
    fn catch_up(&mut self, window: &Window) -> Result<(), HandlerError> {
        let Content::Terminal(tabs) = &self.content else {
            return Ok(());
        };
        let dpi = window.dpi();
        if dpi != tabs.view().dpi() {
            self.event(window, Event::DpiChanged { dpi })?;
        }
        let size = window.client_size()?;
        let minimized = window.is_minimized();
        self.event(window, Event::Resized { size, minimized })
    }
}

impl Handler for App {
    fn event(&mut self, window: &Window, event: Event) -> Result<(), HandlerError> {
        match event {
            Event::Resized { size, minimized } => {
                // A minimized window keeps its grid.
                if let Content::Terminal(tabs) = &mut self.content
                    && !minimized
                {
                    tabs.view_mut().resized(size);
                }
                self.frames
                    .resized(window, size, minimized, &mut self.content)?;
                if self.is_bench() {
                    self.frames.schedule(window)?;
                }
            }
            Event::DpiChanged { dpi } => {
                if let Content::Terminal(tabs) = &mut self.content {
                    tabs.view_mut().dpi_changed(dpi);
                    // The new size of the window, if it changes, comes right
                    // after this event. A frame now would give the terminal
                    // and the program the grid of the new cells in the old
                    // window, which does not last: the frame comes after it.
                    self.frames.schedule(window)?;
                }
            }
            Event::SizeMoveStarted => self.frames.size_move_started(),
            Event::SizeMoveEnded => self.frames.size_move_ended(window, &mut self.content)?,
            Event::FrameTimer => {
                if let Content::Bench(bench) = &mut self.content {
                    for area in bench.step(self.frames.size()) {
                        self.frames.damage(area);
                    }
                }
                self.frames.timer(window, &mut self.content)?;
                if self.is_bench() {
                    self.frames.schedule(window)?;
                }
            }
            Event::Wake(tab) => self.wake(window, tab)?,
            Event::Focus { focused } => {
                if let Content::Terminal(tabs) = &mut self.content {
                    tabs.view_mut().focus(focused, Instant::now());
                    self.frames.changed(window, &mut self.content)?;
                }
            }
            Event::Key(key) => {
                if let Content::Terminal(tabs) = &mut self.content {
                    let response = tabs.view_mut().key(window, &key);
                    self.respond(window, response)?;
                }
            }
            Event::MouseDown { .. }
            | Event::MouseUp { .. }
            | Event::MouseMoved { .. }
            | Event::Wheel { .. }
            | Event::CaptureLost
            | Event::MouseLeft => {
                if let Content::Terminal(tabs) = &mut self.content {
                    let response = tabs.view_mut().mouse(window, &event);
                    self.respond(window, response)?;
                }
            }
        }
        Ok(())
    }

    fn paint(&mut self, _window: &Window, dc: HDC, area: Rect) -> Result<(), HandlerError> {
        self.frames.paint(dc, area)
    }
}
