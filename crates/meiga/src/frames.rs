//! The frame flow: what changed, when to draw it, and how it reaches the
//! window.

use std::mem;
use std::time::{Duration, Instant};

use meiga_core::color::Rgb;
use meiga_core::damage::Damage;
use meiga_core::geometry::{Rect, Size};
use meiga_core::pacing::{FramePacer, Pace, auto_interval};
use meiga_render::{Canvas, Fit, Surface};
use meiga_win::event::HandlerError;
use meiga_win::window::{self, Window};
use meiga_win::{HDC, HWND};

use crate::trace::{Cause, Kind, Record, Trace};

/// The colour of the marks: magenta, which nothing else uses.
const MARK_COLOR: Rgb = Rgb::new(0xff, 0x00, 0xff);
const MARK_THICKNESS: u32 = 1;

/// What the window shows.
pub trait Scene {
    /// Draws everything that is inside the clip rectangle of `canvas`. It
    /// takes `&mut self` because drawing can fill caches, such as the one of
    /// the glyphs.
    fn draw(&mut self, canvas: &mut Canvas<'_>);
}

/// The framebuffer, and the decisions about when to draw it and what to
/// present.
///
/// The window shows the framebuffer, except in the areas of `damage`: they
/// changed, and the next frame composes them and presents them. The frames
/// follow the pacer: at most one per interval, and the first change after a
/// pause at once.
pub struct Frames {
    surface: Surface,
    damage: Damage,
    pacer: FramePacer,
    minimized: bool,
    /// Whether the user is moving or resizing the window.
    live_resize: bool,
    /// While the marks are on, the areas that the last frame presented,
    /// which have a border in the window.
    marks: Option<Vec<Rect>>,
    trace: Option<Trace>,
}

impl Frames {
    /// The framebuffer of a window of `size`, with `scene` composed in it.
    pub fn new(
        size: Size,
        scene: &mut dyn Scene,
        marks: bool,
        trace: Option<Trace>,
    ) -> Result<Self, HandlerError> {
        let mut frames = Self {
            surface: Surface::new(size)?,
            damage: Damage::new(),
            pacer: FramePacer::new(),
            minimized: false,
            live_resize: false,
            marks: marks.then(Vec::new),
            trace,
        };
        frames.compose_all(scene)?;
        Ok(frames)
    }

    pub fn size(&self) -> Size {
        self.surface.size()
    }

    /// `area` of the scene changed: the next frame draws it.
    pub fn damage(&mut self, area: Rect) {
        let area = area.intersection(Rect::from_size(self.surface.size()));
        if !area.is_empty() {
            self.damage.add(area);
        }
    }

    /// Draws what changed, now or with the frame timer, as the pacer says.
    pub fn request(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        if self.damage.is_empty() || self.minimized {
            return Ok(());
        }
        match self.pacer.request(Instant::now(), interval()) {
            Pace::Now => self.draw(window, scene, Cause::Now),
            Pace::After(delay) => Ok(window.start_frame_timer(delay)?),
            Pace::Scheduled => Ok(()),
        }
    }

    /// The frame timer is due: draws what changed.
    pub fn timer(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        self.draw(window, scene, Cause::Timer)
    }

    /// Starts the frame timer for the first moment that the pacer allows,
    /// even if nothing changed. For the benchmarks, which draw a frame each
    /// time.
    pub fn schedule(&mut self, window: &Window) -> Result<(), HandlerError> {
        if self.minimized {
            return Ok(());
        }
        let delay = match self.pacer.request(Instant::now(), interval()) {
            Pace::Now => Duration::ZERO,
            Pace::After(delay) => delay,
            Pace::Scheduled => return Ok(()),
        };
        Ok(window.start_frame_timer(delay)?)
    }

    /// The window changed size, or it was minimized or restored.
    pub fn resized(
        &mut self,
        window: &Window,
        size: Size,
        minimized: bool,
        scene: &mut dyn Scene,
    ) -> Result<(), HandlerError> {
        if minimized {
            // Nothing is drawn while the window is minimized: the changes
            // wait in `damage`.
            window.stop_frame_timer();
            self.pacer.cancel();
            self.minimized = true;
            return Ok(());
        }
        let restored = mem::take(&mut self.minimized);
        if size == self.surface.size() && !restored {
            return Ok(());
        }
        // While the user resizes the window, the memory grows with room to
        // spare, so that it is not allocated again at every step.
        let fit = if self.live_resize {
            Fit::Room
        } else {
            Fit::Exact
        };
        self.surface.resize(size, fit)?;
        self.compose_all(scene)?;
        if restored {
            // The whole window is painted again from the framebuffer, which
            // is up to date: there is nothing else to present. Windows
            // already asks for that paint after a restore; asking again costs
            // nothing, and meiga does not depend on it.
            self.damage.clear();
            if let Some(marks) = &mut self.marks {
                marks.clear();
            }
            window.invalidate(None);
            Ok(())
        } else {
            // Windows asks to paint (WM_PAINT) the parts of the window that
            // are new. The changes in the rest still have to be presented.
            self.request(window, scene)
        }
    }

    pub fn size_move_started(&mut self) {
        self.live_resize = true;
    }

    /// The user finished moving or resizing the window: the framebuffer
    /// gives back the memory that it does not need.
    pub fn size_move_ended(&mut self, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        self.live_resize = false;
        if self.surface.resize(self.surface.size(), Fit::Exact)? {
            // New memory, without the pixels: compose it again. The window
            // already shows the same thing.
            self.compose_all(scene)?;
        }
        Ok(())
    }

    /// Windows asks to paint `area` of the window (WM_PAINT): it is copied
    /// from the framebuffer.
    pub fn paint(&mut self, dc: HDC, area: Rect) -> Result<(), HandlerError> {
        let start = Instant::now();
        // If it fails, that area is wrong until it is painted again, which
        // is no reason to stop meiga.
        let _ = self.surface.paint(dc, area);
        let present = start.elapsed();
        let area = area.intersection(Rect::from_size(self.surface.size()));
        log(
            &mut self.trace,
            &Record {
                kind: Kind::Paint,
                cause: None,
                at: start,
                interval: interval(),
                size: self.surface.size(),
                rects: &[area],
                compose: Duration::ZERO,
                present,
            },
        )
    }

    /// Shows or hides the marks: a border around each area that a frame
    /// presents, drawn only in the window. The next frame erases them.
    pub fn toggle_marks(&mut self, window: &Window) {
        match self.marks.take() {
            Some(marks) => {
                // If it fails, they stay until that area is painted again.
                let _ = erase(&self.surface, window.handle(), &marks);
            }
            None => self.marks = Some(Vec::new()),
        }
    }

    /// Composes the areas that changed and presents them.
    fn draw(
        &mut self,
        window: &Window,
        scene: &mut dyn Scene,
        cause: Cause,
    ) -> Result<(), HandlerError> {
        // The frame can come before its timer, when the timer is late.
        window.stop_frame_timer();
        if self.damage.is_empty() || self.minimized {
            self.pacer.cancel();
            return Ok(());
        }

        let start = Instant::now();
        let mut canvas = self.surface.canvas();
        for &area in self.damage.rects() {
            scene.draw(&mut canvas.clipped(area));
        }
        let composed = Instant::now();

        let handle = window.handle();
        let erased = match &self.marks {
            Some(marks) => erase(&self.surface, handle, marks),
            None => Ok(()),
        };
        let presented = self.surface.present(handle, self.damage.rects());
        let present = composed.elapsed();
        match erased.and(presented) {
            Ok(()) => {
                if let Some(marks) = &mut self.marks {
                    marks.clear();
                    marks.extend_from_slice(self.damage.rects());
                    // If it fails, the frame is right, only without marks.
                    let _ = self.surface.mark(handle, marks, MARK_COLOR, MARK_THICKNESS);
                }
            }
            // Windows did not take the pixels, for example because the
            // session is locked or disconnected: paint the whole window when
            // it can.
            Err(_) => window.invalidate(None),
        }

        log(
            &mut self.trace,
            &Record {
                kind: Kind::Frame,
                cause: Some(cause),
                at: start,
                interval: interval(),
                size: self.surface.size(),
                rects: self.damage.rects(),
                compose: composed - start,
                present,
            },
        )?;
        self.damage.clear();
        self.pacer.drawn(start);
        Ok(())
    }

    /// Composes the whole framebuffer, without presenting it.
    fn compose_all(&mut self, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        let start = Instant::now();
        scene.draw(&mut self.surface.canvas());
        let compose = start.elapsed();
        let size = self.surface.size();
        log(
            &mut self.trace,
            &Record {
                kind: Kind::Compose,
                cause: None,
                at: start,
                interval: interval(),
                size,
                rects: &[Rect::from_size(size)],
                compose,
                present: Duration::ZERO,
            },
        )
    }
}

/// The interval between frames now: `max_fps = "auto"`.
fn interval() -> Duration {
    auto_interval(window::is_remote_session())
}

/// Presents again, from the framebuffer, the borders of `marks`, which
/// erases them from the window.
fn erase(surface: &Surface, window: HWND, marks: &[Rect]) -> Result<(), meiga_render::Error> {
    for mark in marks {
        surface.present(window, &mark.border(MARK_THICKNESS))?;
    }
    Ok(())
}

/// Writes `record` in the frame log, if there is one.
fn log(trace: &mut Option<Trace>, record: &Record<'_>) -> Result<(), HandlerError> {
    if let Some(trace) = trace {
        trace
            .write(record)
            .map_err(|error| format!("Cannot write the frame log: {error}"))?;
    }
    Ok(())
}
