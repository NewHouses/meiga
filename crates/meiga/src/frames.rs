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
    /// Brings the scene up to date before a frame, at `now`: gives `damage`
    /// the areas that changed. Returns when it has to be brought up to date
    /// again, even if nothing else changes, if it has to: for example, when
    /// the cursor blinks.
    fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
        let _ = (now, damage);
        None
    }

    /// While the window is minimized nothing is drawn, and the scene is not
    /// brought up to date: does at `now` only what cannot wait until the
    /// window is restored, such as answering the program. Returns when it has
    /// to do it again, if it has to.
    fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
        let _ = now;
        None
    }

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
/// pause at once. Each frame first brings the scene up to date
/// ([`Scene::update`]), which adds the areas that changed in it.
pub struct Frames {
    surface: Surface,
    damage: Damage,
    pacer: FramePacer,
    /// The scene changed, and it gives the areas when it is brought up to
    /// date: there has to be a frame, even if `damage` is empty.
    changed: bool,
    /// When the scene has to be brought up to date again, as it asked.
    due: Option<Instant>,
    minimized: bool,
    /// Whether the user is moving or resizing the window.
    live_resize: bool,
    /// With `--marks`, the areas that the last frame presented, which have
    /// a border in the window.
    marks: Option<Vec<Rect>>,
    trace: Option<Trace>,
}

impl Frames {
    /// The framebuffer of `window`, with `scene` composed in it, and with
    /// the marks if `marks`.
    pub fn new(
        window: &Window,
        scene: &mut dyn Scene,
        trace: Option<Trace>,
        marks: bool,
    ) -> Result<Self, HandlerError> {
        let mut frames = Self {
            surface: Surface::new(window.client_size()?)?,
            damage: Damage::new(),
            pacer: FramePacer::new(),
            changed: false,
            due: None,
            minimized: false,
            live_resize: false,
            marks: marks.then(Vec::new),
            trace,
        };
        frames.compose_all(scene)?;
        // The first paint of the window copies all of it.
        frames.damage.clear();
        frames.wait_for_scene(window)?;
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

    /// The scene changed: a frame brings it up to date and draws what
    /// changed in it, as [`Frames::request`].
    pub fn changed(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        self.changed = true;
        self.request(window, scene)
    }

    /// Draws what changed, now or with the frame timer, as the pacer says.
    pub fn request(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        if self.minimized {
            return self.hidden(window, scene);
        }
        if self.damage.is_empty() && !self.changed {
            return self.wait_for_scene(window);
        }
        match self.pacer.request(Instant::now(), interval()) {
            Pace::Now => self.draw(window, scene, Cause::Now),
            Pace::After(delay) => Ok(window.start_frame_timer(delay)?),
            Pace::Scheduled => Ok(()),
        }
    }

    /// The frame timer is due: draws what changed or, if no frame was
    /// waiting, the scene asked for this moment, which is a change like any
    /// other. While the window is minimized, the scene does what cannot wait.
    pub fn timer(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        if self.pacer.is_scheduled() {
            return self.draw(window, scene, Cause::Timer);
        }
        self.due = None;
        self.changed(window, scene)
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
            // wait in `damage`, and in the scene, which is not brought up to
            // date until the window is restored, except for what cannot wait.
            window.stop_frame_timer();
            self.pacer.cancel();
            self.minimized = true;
            return self.hidden(window, scene);
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
            // The frame timer may still be set for what the scene had to do
            // while the window was minimized: now it is for the frames.
            window.stop_frame_timer();
            self.wait_for_scene(window)
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
    pub fn size_move_ended(
        &mut self,
        window: &Window,
        scene: &mut dyn Scene,
    ) -> Result<(), HandlerError> {
        self.live_resize = false;
        if self.surface.resize(self.surface.size(), Fit::Exact)? {
            // New memory, without the pixels: compose it again. The window
            // already shows the same thing, except what changed in the scene
            // meanwhile, which the next frame presents.
            self.compose_all(scene)?;
            return self.request(window, scene);
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

    /// Composes the areas that changed and presents them.
    fn draw(
        &mut self,
        window: &Window,
        scene: &mut dyn Scene,
        cause: Cause,
    ) -> Result<(), HandlerError> {
        // The frame can come before its timer, when the timer is late.
        window.stop_frame_timer();
        if self.minimized {
            self.pacer.cancel();
            return Ok(());
        }
        let start = Instant::now();
        self.update(scene, start);
        if self.damage.is_empty() {
            self.pacer.cancel();
            return self.wait_for_scene(window);
        }

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
        self.wait_for_scene(window)
    }

    /// Brings `scene` up to date at `now`, and adds to `damage` the areas
    /// that changed in it.
    fn update(&mut self, scene: &mut dyn Scene, now: Instant) {
        self.changed = false;
        let bounds = Rect::from_size(self.surface.size());
        let damage = &mut self.damage;
        self.due = scene.update(now, &mut |area| {
            let area = area.intersection(bounds);
            if !area.is_empty() {
                damage.add(area);
            }
        });
    }

    /// While the window is minimized, does what cannot wait in the scene,
    /// and keeps the frame timer only for the next time that it has to,
    /// without drawing anything.
    fn hidden(&mut self, window: &Window, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        let now = Instant::now();
        match scene.update_hidden(now) {
            Some(due) => Ok(window.start_frame_timer(due.saturating_duration_since(now))?),
            None => {
                window.stop_frame_timer();
                Ok(())
            }
        }
    }

    /// If no frame is waiting, starts the frame timer for the moment that
    /// the scene asked for, if it asked for one.
    fn wait_for_scene(&mut self, window: &Window) -> Result<(), HandlerError> {
        if self.minimized || self.pacer.is_scheduled() {
            return Ok(());
        }
        match self.due {
            Some(due) => {
                Ok(window.start_frame_timer(due.saturating_duration_since(Instant::now()))?)
            }
            None => Ok(()),
        }
    }

    /// Composes the whole framebuffer, with the scene up to date, without
    /// presenting it.
    fn compose_all(&mut self, scene: &mut dyn Scene) -> Result<(), HandlerError> {
        let start = Instant::now();
        self.update(scene, start);
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
