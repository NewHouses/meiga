//! The handler of the window: it takes the events to the content and to the
//! frames.

use meiga_core::geometry::Rect;
use meiga_render::Canvas;
use meiga_win::HDC;
use meiga_win::event::{Event, Handler, HandlerError, Key};
use meiga_win::window::Window;

use crate::bench::Bench;
use crate::demo::{Demo, Mouse};
use crate::frames::{Frames, Scene};
use crate::options::BenchMode;
use crate::trace::Trace;

/// What the window shows.
enum Content {
    Demo(Demo),
    Bench(Bench),
}

impl Scene for Content {
    fn draw(&self, canvas: &mut Canvas<'_>) {
        match self {
            Content::Demo(demo) => demo.draw(canvas),
            Content::Bench(bench) => bench.draw(canvas),
        }
    }
}

pub struct App {
    frames: Frames,
    content: Content,
}

impl App {
    /// The content of `window`, with the first frame composed. A benchmark
    /// starts its frames as soon as the window runs.
    pub fn new(
        window: &Window,
        bench: Option<BenchMode>,
        trace: Option<Trace>,
    ) -> Result<Self, HandlerError> {
        let content = match bench {
            Some(mode) => Content::Bench(Bench::new(mode)),
            None => Content::Demo(Demo::new()),
        };
        // The marks would add work to the frames that a benchmark measures.
        let marks = bench.is_none();
        let mut frames = Frames::new(window.client_size()?, &content, marks, trace)?;
        if bench.is_some() {
            frames.schedule(window)?;
        }
        Ok(Self { frames, content })
    }

    const fn is_bench(&self) -> bool {
        matches!(self.content, Content::Bench(_))
    }
}

impl Handler for App {
    fn event(&mut self, window: &Window, event: Event) -> Result<(), HandlerError> {
        match event {
            Event::Resized { size, minimized } => {
                self.frames
                    .resized(window, size, minimized, &self.content)?;
                if self.is_bench() {
                    self.frames.schedule(window)?;
                }
            }
            Event::SizeMoveStarted => self.frames.size_move_started(),
            Event::SizeMoveEnded => self.frames.size_move_ended(&self.content)?,
            Event::FrameTimer => {
                if let Content::Bench(bench) = &mut self.content {
                    for area in bench.step(self.frames.size()) {
                        self.frames.damage(area);
                    }
                }
                self.frames.timer(window, &self.content)?;
                if self.is_bench() {
                    self.frames.schedule(window)?;
                }
            }
            Event::KeyDown {
                key: Key::P,
                repeat: false,
            } => self.frames.toggle_marks(window),
            event => {
                if let Content::Demo(demo) = &mut self.content {
                    let frames = &mut self.frames;
                    let size = frames.size();
                    match demo.event(event, size, |area| frames.damage(area)) {
                        Some(Mouse::Capture) => window.capture_mouse(),
                        Some(Mouse::Release) => window.release_mouse(),
                        None => {}
                    }
                    self.frames.request(window, &self.content)?;
                }
            }
        }
        Ok(())
    }

    fn paint(&mut self, _window: &Window, dc: HDC, area: Rect) -> Result<(), HandlerError> {
        self.frames.paint(dc, area)
    }
}
