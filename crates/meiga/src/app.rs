//! The handler of the window: it takes the events to the content and to the
//! frames.

use std::mem;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use meiga_core::geometry::{GridSize, Rect};
use meiga_core::term::Terminal;
use meiga_render::{Canvas, Font, FontSystem};
use meiga_win::HDC;
use meiga_win::conpty::Pty;
use meiga_win::event::{Event, Handler, HandlerError, Key};
use meiga_win::window::Window;

use crate::bench::Bench;
use crate::frames::{Frames, Scene};
use crate::options::{BenchMode, Options};
use crate::session::{self, Shared, lock};
use crate::trace::Trace;
use crate::view::{FONT_FAMILIES, FONT_POINTS, TerminalView};

/// The title of the window while the program does not give one.
pub const TITLE: &str = "meiga";

/// What the window shows. Both hold a whole font, so they go in boxes.
enum Content {
    Terminal(Box<TerminalView>),
    Bench(Box<Bench>),
}

impl Scene for Content {
    fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
        match self {
            Content::Terminal(view) => view.update(now, damage),
            Content::Bench(_) => None,
        }
    }

    fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
        match self {
            Content::Terminal(view) => view.update_hidden(now),
            Content::Bench(_) => None,
        }
    }

    fn draw(&mut self, canvas: &mut Canvas<'_>) {
        match self {
            Content::Terminal(view) => view.draw(canvas),
            Content::Bench(bench) => bench.draw(canvas),
        }
    }
}

pub struct App {
    frames: Frames,
    content: Content,
    /// The session of the terminal, which closes when it is dropped. `None`
    /// for the benchmarks, and if the program could not start.
    _session: Option<Pty>,
}

impl App {
    /// The content of `window`, with the first frame composed: the terminal
    /// with the program of `options` running, or a benchmark, which starts
    /// its frames as soon as the window runs.
    pub fn new(
        window: &Window,
        options: &Options,
        trace: Option<Trace>,
    ) -> Result<Self, HandlerError> {
        // Only the content with text reads the fonts of the system.
        let (mut content, session) = match options.bench {
            Some(BenchMode::Text) => {
                let system = FontSystem::new()?;
                let font = Font::new(&system, &FONT_FAMILIES, FONT_POINTS, window.dpi())?;
                let bench = Bench::new(BenchMode::Text, Some(font));
                (Content::Bench(Box::new(bench)), None)
            }
            Some(mode) => (Content::Bench(Box::new(Bench::new(mode, None))), None),
            None => terminal(window, options.command.as_deref())?,
        };
        let mut frames = Frames::new(window, &mut content, trace)?;
        if options.bench.is_some() {
            frames.schedule(window)?;
        }
        Ok(Self {
            frames,
            content,
            _session: session,
        })
    }

    const fn is_bench(&self) -> bool {
        matches!(self.content, Content::Bench(_))
    }

    /// The reader thread of the session woke the window up: the title, the
    /// screen, or both, changed.
    fn wake(&mut self, window: &Window) -> Result<(), HandlerError> {
        let Content::Terminal(view) = &self.content else {
            return Ok(());
        };
        let (title, screen) = {
            let mut shared = lock(view.shared());
            let title = mem::take(&mut shared.title_changed)
                .then(|| shared.terminal.title().map(str::to_owned));
            (title, shared.screen_changed)
        };
        if let Some(title) = title {
            window.set_title(title.as_deref().unwrap_or(TITLE));
        }
        if screen {
            self.frames.changed(window, &mut self.content)?;
        }
        Ok(())
    }
}

/// The terminal, with the size that fills `window`, and its session, which
/// runs `command`, or the default shell. If the program cannot start, the
/// terminal shows why.
fn terminal(
    window: &Window,
    command: Option<&str>,
) -> Result<(Content, Option<Pty>), HandlerError> {
    let font = Font::new(
        &FontSystem::new()?,
        &FONT_FAMILIES,
        FONT_POINTS,
        window.dpi(),
    )?;
    let cell = font.cell();
    let client = window.client_size()?;
    // The grid keeps this size until it can change with the window
    // (subphase 1.9).
    let cells = |pixels: u32, cell: u32| u16::try_from(pixels / cell.max(1)).unwrap_or(u16::MAX);
    let mut terminal = Terminal::new(GridSize::new(
        cells(client.width, cell.width),
        cells(client.height, cell.height),
    ));
    terminal.set_cell_size(cell);
    let size = terminal.size();
    let shared = Arc::new(Mutex::new(Shared::new(terminal)));
    let waker = window.waker();
    let session = match session::start(command, size, &shared, &waker) {
        Ok(pty) => Some(pty),
        Err(message) => {
            lock(&shared).terminal.feed(message.as_bytes());
            None
        }
    };
    let input = session.as_ref().map(Pty::input);
    let view = TerminalView::new(shared, input, waker, font, window.dpi());
    Ok((Content::Terminal(Box::new(view)), session))
}

impl Handler for App {
    fn event(&mut self, window: &Window, event: Event) -> Result<(), HandlerError> {
        match event {
            Event::Resized { size, minimized } => {
                self.frames
                    .resized(window, size, minimized, &mut self.content)?;
                if self.is_bench() {
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
            Event::Wake => self.wake(window)?,
            Event::Focus { focused } => {
                if let Content::Terminal(view) = &mut self.content {
                    view.focus(focused, Instant::now());
                    self.frames.changed(window, &mut self.content)?;
                }
            }
            // Until the keyboard reaches the program (subphase 1.7).
            Event::KeyDown {
                key: Key::P,
                repeat: false,
            } => self.frames.toggle_marks(window),
            _ => {}
        }
        Ok(())
    }

    fn paint(&mut self, _window: &Window, dc: HDC, area: Rect) -> Result<(), HandlerError> {
        self.frames.paint(dc, area)
    }
}
