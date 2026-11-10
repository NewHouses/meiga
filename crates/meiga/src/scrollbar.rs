//! The scroll bar: a strip on the right of the window, with a thumb that
//! shows which part of the history the view shows, as in Windows Terminal.
//!
//! The strip is always there, so that the grid keeps its columns when the
//! history appears. The thumb is thin, and wide while the pointer is over
//! the strip or the left button is down on it. Dragging the thumb moves the
//! view. A press on the strip outside the thumb moves the view a page
//! towards the pointer, and again while the button stays down, until the
//! thumb reaches the pointer, as in the scroll bars of Windows. Without
//! history, as on the alternate screen, the strip is empty.

use std::mem;
use std::time::{Duration, Instant};

use meiga_core::color::Rgb;
use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::term::{History, Scroll, Terminal};
use meiga_render::Canvas;

/// The dots per inch at a scale of 100 %, where the sizes below are in
/// pixels.
const BASE_DPI: u32 = 96;
/// The width of the strip.
const STRIP: u32 = 12;
/// The width of the thumb: thin, and wide.
const THIN: u32 = 3;
const WIDE: u32 = 8;
/// The space above and below the thumb.
const MARGIN: u32 = 2;
/// The shortest thumb: it can still be grabbed with a long history.
const MIN_LENGTH: u32 = 24;

/// The colour of the thumb: thin, wide, and dragged.
const THIN_COLOR: Rgb = Rgb::new(0x4e, 0x53, 0x5c);
const WIDE_COLOR: Rgb = Rgb::new(0x6e, 0x74, 0x7f);
const DRAGGED_COLOR: Rgb = Rgb::new(0x8e, 0x94, 0x9f);

/// While the button stays down outside the thumb, the view moves another
/// page after `FIRST_REPEAT`, and then after each `REPEAT`.
const FIRST_REPEAT: Duration = Duration::from_millis(250);
const REPEAT: Duration = Duration::from_millis(50);

/// How the thumb looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Look {
    Thin,
    Wide,
    Dragged,
}

/// Where the thumb is, in pixels from the top of the window.
#[derive(Clone, Copy, Debug)]
struct Thumb {
    top: i32,
    bottom: i32,
    /// How far its top can move.
    travel: u64,
}

/// The view moves by pages while the left button stays down on the
/// strip, outside the thumb.
#[derive(Clone, Copy, Debug)]
struct Paging {
    /// Towards older rows.
    up: bool,
    /// Where the pointer is: the view stops when the thumb reaches it.
    y: i32,
    /// When the view moves again, or `None` once the thumb reached the
    /// pointer.
    next: Option<Instant>,
}

pub struct ScrollBar {
    /// The sizes, in pixels at the scale of the window.
    strip: u32,
    thin: u32,
    wide: u32,
    margin: u32,
    min_length: u32,
    /// The size of the client area of the window.
    client: Size,
    /// The pointer is over the strip.
    hover: bool,
    /// While the thumb is dragged: how far below its top the pointer took
    /// it.
    grab: Option<i32>,
    paging: Option<Paging>,
    /// The thumb that the window shows, and how it looks: none without
    /// history.
    shown: Option<(Rect, Look)>,
}

impl ScrollBar {
    /// The bar of a window of `client` pixels at `dpi`.
    pub fn new(dpi: u32, client: Size) -> Self {
        let mut bar = Self {
            strip: 0,
            thin: 0,
            wide: 0,
            margin: 0,
            min_length: 0,
            client,
            hover: false,
            grab: None,
            paging: None,
            shown: None,
        };
        bar.set_dpi(dpi);
        bar
    }

    /// The width of the strip at `dpi`, in pixels.
    pub fn width_at(dpi: u32) -> u32 {
        scale(STRIP, dpi)
    }

    pub const fn width(&self) -> u32 {
        self.strip
    }

    /// The window moved to a monitor with `dpi`.
    pub fn set_dpi(&mut self, dpi: u32) {
        self.strip = scale(STRIP, dpi);
        self.thin = scale(THIN, dpi);
        self.wide = scale(WIDE, dpi);
        self.margin = scale(MARGIN, dpi);
        self.min_length = scale(MIN_LENGTH, dpi);
    }

    /// The client area of the window has `client` pixels now.
    pub fn set_client(&mut self, client: Size) {
        self.client = client;
    }

    /// Whether `at` is on the strip.
    pub fn contains(&self, at: Point) -> bool {
        let strip = self.strip_rect();
        (strip.left..strip.right).contains(&at.x) && (strip.top..strip.bottom).contains(&at.y)
    }

    /// Whether the left button went down on the strip and is still down.
    pub const fn is_pressed(&self) -> bool {
        self.grab.is_some() || self.paging.is_some()
    }

    /// The pointer is over the strip, or not. Returns whether that
    /// changed.
    pub fn hover(&mut self, over: bool) -> bool {
        mem::replace(&mut self.hover, over) != over
    }

    /// Brings the thumb up to date with `history`, with `rows` rows on the
    /// screen. Returns the area of the window that changed, if the thumb
    /// moved, changed its length, or changed how it looks.
    pub fn update(&mut self, history: History, rows: u16) -> Option<Rect> {
        let look = self.look();
        let shown = self
            .thumb(history, rows)
            .map(|thumb| (self.rect(thumb, look), look));
        if shown == self.shown {
            return None;
        }
        let old = mem::replace(&mut self.shown, shown).map(|(rect, _)| rect);
        match (old, shown.map(|(rect, _)| rect)) {
            (Some(old), Some(new)) => Some(old.union(new)),
            (old, new) => old.or(new),
        }
    }

    /// Draws the strip, with the thumb of the last [`ScrollBar::update`],
    /// inside the clip rectangle of `canvas`.
    pub fn draw(&self, canvas: &mut Canvas<'_>, background: Rgb) {
        let strip = self.strip_rect();
        if !strip.intersects(canvas.clip()) {
            return;
        }
        canvas.fill(strip, background);
        if let Some((rect, look)) = self.shown {
            let color = match look {
                Look::Thin => THIN_COLOR,
                Look::Wide => WIDE_COLOR,
                Look::Dragged => DRAGGED_COLOR,
            };
            canvas.fill(rect, color);
        }
    }

    /// The left button went down at `at`, on the strip. On the thumb, it
    /// grabs it; elsewhere, it moves the view of `terminal` a page towards
    /// `at`, at `now`. Without history, it does nothing.
    pub fn press(&mut self, at: Point, terminal: &mut Terminal, now: Instant) {
        self.hover = true;
        let Some(thumb) = self.thumb(terminal.history(), terminal.size().rows) else {
            return;
        };
        if (thumb.top..thumb.bottom).contains(&at.y) {
            self.grab = Some(at.y - thumb.top);
            return;
        }
        let up = at.y < thumb.top;
        terminal.scroll(page(up));
        self.paging = Some(Paging {
            up,
            y: at.y,
            next: now.checked_add(FIRST_REPEAT),
        });
    }

    /// The pointer moved to `at`, at `now`, while the left button is still
    /// down after a press on the strip: the thumb follows it, or the pages
    /// go on towards it.
    pub fn drag(&mut self, at: Point, terminal: &mut Terminal, now: Instant) {
        if let Some(grab) = self.grab {
            let offset = self.offset_at(
                at.y.saturating_sub(grab),
                terminal.history(),
                terminal.size().rows,
            );
            terminal.scroll_to(offset);
        } else if let Some(paging) = &mut self.paging {
            paging.y = at.y;
            if paging.next.is_none() {
                paging.next = now.checked_add(REPEAT);
            }
        }
    }

    /// The left button went up, or the window lost the mouse.
    pub fn release(&mut self) {
        self.grab = None;
        self.paging = None;
    }

    /// While the button stays down outside the thumb, moves the view of
    /// `terminal` another page towards the pointer, if it is time at `now`
    /// and the thumb did not reach the pointer yet. Returns when to do it
    /// again.
    pub fn tick(&mut self, terminal: &mut Terminal, now: Instant) -> Option<Instant> {
        let Paging { up, y, next } = self.paging?;
        let next = next?;
        if now < next {
            return Some(next);
        }
        let reached = self
            .thumb(terminal.history(), terminal.size().rows)
            .is_none_or(|thumb| if up { thumb.top <= y } else { thumb.bottom > y });
        let next = if !reached && terminal.scroll(page(up)) {
            now.checked_add(REPEAT)
        } else {
            None
        };
        if let Some(paging) = &mut self.paging {
            paging.next = next;
        }
        next
    }

    fn look(&self) -> Look {
        if self.grab.is_some() {
            Look::Dragged
        } else if self.hover || self.paging.is_some() {
            Look::Wide
        } else {
            Look::Thin
        }
    }

    /// The strip, in the window.
    fn strip_rect(&self) -> Rect {
        let right = i32::try_from(self.client.width).unwrap_or(i32::MAX);
        let bottom = i32::try_from(self.client.height).unwrap_or(i32::MAX);
        // The strip is a few dozen pixels.
        Rect::new(right.saturating_sub(self.strip as i32), 0, right, bottom)
    }

    /// The thumb of `history` with `rows` rows on the screen: as long as
    /// the strip as the screen is of all the rows, and at the bottom when
    /// the view shows the screen. None without history.
    fn thumb(&self, history: History, rows: u16) -> Option<Thumb> {
        if history.rows == 0 {
            return None;
        }
        let track = u64::from(self.client.height.saturating_sub(2 * self.margin));
        let above = history.rows as u64;
        let length = (track * u64::from(rows) / (above + u64::from(rows)))
            .max(u64::from(self.min_length))
            .min(track);
        let travel = track - length;
        let offset = history.offset.min(history.rows) as u64;
        // The client area is far smaller than an i32.
        let top = u64::from(self.margin) + travel * (above - offset) / above;
        Some(Thumb {
            top: top as i32,
            bottom: (top + length) as i32,
            travel,
        })
    }

    /// The thumb in the window, as `look` draws it: in the middle of the
    /// strip.
    fn rect(&self, thumb: Thumb, look: Look) -> Rect {
        let width = match look {
            Look::Thin => self.thin,
            Look::Wide | Look::Dragged => self.wide,
        };
        let left = self.strip_rect().left + (self.strip.saturating_sub(width) / 2) as i32;
        Rect::new(left, thumb.top, left + width as i32, thumb.bottom)
    }

    /// The offset of the view of `history`, with `rows` rows on the screen,
    /// that puts the top of the thumb at `top`, to the nearest row.
    fn offset_at(&self, top: i32, history: History, rows: u16) -> usize {
        let Some(thumb) = self.thumb(history, rows) else {
            return 0;
        };
        if thumb.travel == 0 {
            return history.offset;
        }
        let from_top = u64::try_from(i64::from(top) - i64::from(self.margin))
            .unwrap_or(0)
            .min(thumb.travel);
        // At most the rows of the history, which is a usize.
        let below = (from_top * history.rows as u64 + thumb.travel / 2) / thumb.travel;
        history.rows - below as usize
    }
}

/// `pixels` at a scale of 100 %, at `dpi`: rounded, and at least one.
fn scale(pixels: u32, dpi: u32) -> u32 {
    (pixels.saturating_mul(dpi).saturating_add(BASE_DPI / 2) / BASE_DPI).max(1)
}

const fn page(up: bool) -> Scroll {
    if up { Scroll::PageUp } else { Scroll::PageDown }
}

#[cfg(test)]
mod tests {
    use meiga_core::geometry::GridSize;

    use super::*;

    /// A terminal of 10 × 5 cells with 15 rows of history, and a bar of a
    /// window of 100 × 100 pixels at 100 %: the strip goes from 88 to 100,
    /// and the thumb, 24 pixels long, from 2 to 98.
    fn setup() -> (Terminal, ScrollBar) {
        let mut terminal = Terminal::new(GridSize::new(10, 5));
        let lines: Vec<String> = (1..=20).map(|n| n.to_string()).collect();
        terminal.feed(lines.join("\r\n").as_bytes());
        assert_eq!(terminal.history().rows, 15);
        (terminal, ScrollBar::new(96, Size::new(100, 100)))
    }

    fn offset(terminal: &Terminal) -> usize {
        terminal.history().offset
    }

    #[test]
    fn the_thumb_follows_the_view_and_widens_under_the_pointer() {
        let (mut terminal, mut bar) = setup();
        // Thin, at the bottom, in the middle of the strip.
        assert_eq!(
            bar.update(terminal.history(), 5),
            Some(Rect::new(92, 74, 95, 98))
        );
        assert_eq!(bar.update(terminal.history(), 5), None);
        // At the top: both places are drawn again.
        assert!(terminal.scroll(Scroll::Top));
        assert_eq!(
            bar.update(terminal.history(), 5),
            Some(Rect::new(92, 2, 95, 98))
        );
        assert!(bar.hover(true));
        assert!(!bar.hover(true));
        assert_eq!(
            bar.update(terminal.history(), 5),
            Some(Rect::new(90, 2, 98, 26))
        );
        let mut pixels = vec![0; 100 * 100];
        let mut canvas = Canvas::new(&mut pixels, Size::new(100, 100), 100).unwrap();
        bar.draw(&mut canvas, Rgb::new(1, 2, 3));
        let at = |x: u32, y: u32| canvas.get(x, y);
        assert_eq!(at(94, 10), Some(meiga_render::pixel(WIDE_COLOR)));
        assert_eq!(at(94, 30), Some(meiga_render::pixel(Rgb::new(1, 2, 3))));
        assert_eq!(at(87, 10), Some(0));
    }

    #[test]
    fn dragging_the_thumb_moves_the_view_to_the_nearest_row() {
        let (mut terminal, mut bar) = setup();
        let now = Instant::now();
        // Grabbed 6 pixels below its top.
        bar.press(Point::new(93, 80), &mut terminal, now);
        assert!(bar.is_pressed());
        assert_eq!(offset(&terminal), 0);
        bar.drag(Point::new(93, 8), &mut terminal, now);
        assert_eq!(offset(&terminal), 15);
        // Its top at 38, half of the way: 7.5 rows above the screen.
        bar.drag(Point::new(93, 44), &mut terminal, now);
        assert_eq!(offset(&terminal), 7);
        bar.drag(Point::new(93, 500), &mut terminal, now);
        assert_eq!(offset(&terminal), 0);
        bar.release();
        assert!(!bar.is_pressed());
    }

    #[test]
    fn a_press_outside_the_thumb_moves_by_pages_until_the_thumb_reaches_the_pointer() {
        let (mut terminal, mut bar) = setup();
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        // Above the thumb, which goes from 74 to 98.
        bar.press(Point::new(93, 40), &mut terminal, start);
        assert_eq!(offset(&terminal), 5);
        assert_eq!(bar.tick(&mut terminal, at(100)), Some(at(250)));
        assert_eq!(offset(&terminal), 5);
        assert_eq!(bar.tick(&mut terminal, at(250)), Some(at(300)));
        assert_eq!(offset(&terminal), 10);
        // The thumb, from 26 to 50, is under the pointer now.
        assert_eq!(bar.tick(&mut terminal, at(300)), None);
        assert_eq!(offset(&terminal), 10);
        // Lower, the pages do not go back, as in Windows; higher, they go
        // on.
        bar.drag(Point::new(93, 90), &mut terminal, at(400));
        assert_eq!(bar.tick(&mut terminal, at(450)), None);
        bar.drag(Point::new(93, 10), &mut terminal, at(500));
        assert_eq!(bar.tick(&mut terminal, at(550)), Some(at(600)));
        assert_eq!(offset(&terminal), 15);
        assert_eq!(bar.tick(&mut terminal, at(600)), None);
    }

    #[test]
    fn without_history_the_strip_is_empty() {
        let mut terminal = Terminal::new(GridSize::new(10, 5));
        let mut bar = ScrollBar::new(96, Size::new(100, 100));
        assert_eq!(bar.update(terminal.history(), 5), None);
        bar.press(Point::new(93, 50), &mut terminal, Instant::now());
        assert!(!bar.is_pressed());
        // At 150 %, the strip is 18 pixels wide.
        assert_eq!(ScrollBar::width_at(144), 18);
    }
}
