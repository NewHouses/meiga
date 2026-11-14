//! When to draw the next frame: at most one per interval, as decided in ADR
//! 0005 (`docs/adr/0005-fios-e-fotogramas.md`).

use std::time::{Duration, Instant};

/// The interval between frames when `max_fps` is `auto`: 30 frames per
/// second in remote sessions (Remote Desktop) and 60 locally, as decided in
/// ADR 0017 (`docs/adr/0017-os-fotogramas-por-escritorio-remoto.md`) with the
/// measurements of subphase 1.10.
pub const fn auto_interval(remote: bool) -> Duration {
    let fps = if remote { 30 } else { 60 };
    Duration::from_nanos(1_000_000_000 / fps)
}

/// What to do after a change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    /// Draw the frame now.
    Now,
    /// Draw it after this time: start a timer.
    After(Duration),
    /// A frame is already scheduled, and it will show the change too.
    Scheduled,
}

/// Decides when to draw, so that there is at most one frame per interval and
/// the first change after a pause is drawn at once.
#[derive(Clone, Debug, Default)]
pub struct FramePacer {
    last_frame: Option<Instant>,
    due: Option<Instant>,
}

impl FramePacer {
    pub const fn new() -> Self {
        Self {
            last_frame: None,
            due: None,
        }
    }

    /// Something changed at `now`: when should the frame that shows it be
    /// drawn?
    pub fn request(&mut self, now: Instant, interval: Duration) -> Pace {
        if let Some(due) = self.due {
            // The timer is late, for example because messages kept arriving
            // and it never got its turn: do not wait for it.
            return if now >= due {
                Pace::Now
            } else {
                Pace::Scheduled
            };
        }
        let Some(due) = self.last_frame.and_then(|last| last.checked_add(interval)) else {
            return Pace::Now;
        };
        if now >= due {
            return Pace::Now;
        }
        self.due = Some(due);
        Pace::After(due - now)
    }

    /// Whether a frame is scheduled.
    pub const fn is_scheduled(&self) -> bool {
        self.due.is_some()
    }

    /// Forgets the scheduled frame, for example because the window was
    /// minimized.
    pub fn cancel(&mut self) {
        self.due = None;
    }

    /// A frame was drawn at `now`.
    pub fn drawn(&mut self, now: Instant) {
        self.last_frame = Some(now);
        self.due = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTERVAL: Duration = Duration::from_millis(33);

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn the_first_change_is_drawn_at_once() {
        let mut pacer = FramePacer::new();
        assert_eq!(pacer.request(Instant::now(), INTERVAL), Pace::Now);
    }

    #[test]
    fn a_change_inside_the_interval_waits_for_the_rest_of_it() {
        let start = Instant::now();
        let mut pacer = FramePacer::new();
        pacer.drawn(start);
        assert_eq!(pacer.request(start + ms(10), INTERVAL), Pace::After(ms(23)));
        assert!(pacer.is_scheduled());
        assert_eq!(pacer.request(start + ms(20), INTERVAL), Pace::Scheduled);
    }

    #[test]
    fn a_change_after_the_interval_is_drawn_at_once() {
        let start = Instant::now();
        let mut pacer = FramePacer::new();
        pacer.drawn(start);
        assert_eq!(pacer.request(start + ms(33), INTERVAL), Pace::Now);
        assert_eq!(pacer.request(start + ms(500), INTERVAL), Pace::Now);
    }

    #[test]
    fn a_late_timer_does_not_delay_the_frame() {
        let start = Instant::now();
        let mut pacer = FramePacer::new();
        pacer.drawn(start);
        assert_eq!(pacer.request(start + ms(1), INTERVAL), Pace::After(ms(32)));
        assert_eq!(pacer.request(start + ms(34), INTERVAL), Pace::Now);
    }

    #[test]
    fn drawing_and_cancelling_end_the_wait() {
        let start = Instant::now();
        let mut pacer = FramePacer::new();
        pacer.drawn(start);
        pacer.request(start + ms(1), INTERVAL);
        pacer.cancel();
        assert!(!pacer.is_scheduled());
        pacer.request(start + ms(2), INTERVAL);
        pacer.drawn(start + ms(33));
        assert!(!pacer.is_scheduled());
        assert_eq!(pacer.request(start + ms(40), INTERVAL), Pace::After(ms(26)));
    }

    #[test]
    fn auto_is_30_fps_in_remote_sessions_and_60_locally() {
        assert_eq!(auto_interval(true), Duration::from_nanos(33_333_333));
        assert_eq!(auto_interval(false), Duration::from_nanos(16_666_666));
    }
}
