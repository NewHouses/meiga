//! The frame log (`--trace-frames`): a CSV line for each frame that meiga
//! draws, to check and to measure what it presents.

use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, Write as _};
use std::path::Path;
use std::time::{Duration, Instant};

use meiga_core::geometry::{Rect, Size};

const HEADER: &str =
    "frame,time_ms,kind,cause,interval_ms,width,height,rects,pixels,compose_us,present_us,areas\n";

/// What drew the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// meiga presented what changed.
    Frame,
    /// Windows asked to paint an area (`WM_PAINT`), which was copied from the
    /// framebuffer.
    Paint,
    /// The whole framebuffer was composed, without presenting it: at the
    /// start, and when the window changes size.
    Compose,
}

/// Why a frame was drawn when it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// At once, because the last frame was more than an interval ago.
    Now,
    /// When the frame timer was due.
    Timer,
}

/// One line of the log.
pub struct Record<'a> {
    pub kind: Kind,
    pub cause: Option<Cause>,
    /// When the frame started.
    pub at: Instant,
    /// The interval between frames at that moment.
    pub interval: Duration,
    /// The size of the framebuffer.
    pub size: Size,
    /// The areas presented (a frame), painted (a paint) or composed (a
    /// compose).
    pub rects: &'a [Rect],
    pub compose: Duration,
    pub present: Duration,
}

/// The open log.
pub struct Trace {
    file: File,
    start: Instant,
    lines: u64,
    /// The text of a line, kept to avoid an allocation for each one.
    line: String,
}

impl Trace {
    /// Creates the file, or empties it if it exists, and writes the header.
    pub fn create(path: &Path) -> io::Result<Self> {
        let mut file = File::create(path)?;
        file.write_all(HEADER.as_bytes())?;
        Ok(Self {
            file,
            start: Instant::now(),
            lines: 0,
            line: String::new(),
        })
    }

    /// Writes a line. It goes to the file at once, so that it is complete
    /// even if meiga ends without closing it.
    pub fn write(&mut self, record: &Record<'_>) -> io::Result<()> {
        self.lines += 1;
        self.line.clear();
        format_line(&mut self.line, self.lines, self.start, record);
        self.file.write_all(self.line.as_bytes())
    }
}

/// The CSV line of `record`, the `number`th one of a log that started at
/// `start`. Decimals use a point, whatever the language of Windows.
fn format_line(line: &mut String, number: u64, start: Instant, record: &Record<'_>) {
    let kind = match record.kind {
        Kind::Frame => "frame",
        Kind::Paint => "paint",
        Kind::Compose => "compose",
    };
    let cause = match record.cause {
        Some(Cause::Now) => "now",
        Some(Cause::Timer) => "timer",
        None => "",
    };
    let pixels: u64 = record.rects.iter().map(|rect| rect.area()).sum();
    // Writing to a String does not fail.
    let _ = write!(line, "{number},");
    write_millis(line, record.at.saturating_duration_since(start));
    let _ = write!(line, ",{kind},{cause},");
    write_millis(line, record.interval);
    let _ = write!(
        line,
        ",{},{},{},{pixels},{},{},",
        record.size.width,
        record.size.height,
        record.rects.len(),
        record.compose.as_micros(),
        record.present.as_micros(),
    );
    for (index, rect) in record.rects.iter().enumerate() {
        let separator = if index == 0 { "" } else { ";" };
        let _ = write!(
            line,
            "{separator}{} {} {} {}",
            rect.left,
            rect.top,
            rect.width(),
            rect.height()
        );
    }
    line.push('\n');
}

/// Writes `duration` in milliseconds with three decimals, rounded to the
/// microsecond. Integers, because formatting floating-point numbers would
/// add some 20 KB to the program.
fn write_millis(line: &mut String, duration: Duration) {
    let micros = duration.as_nanos().saturating_add(500) / 1000;
    let _ = write!(line, "{}.{:03}", micros / 1000, micros % 1000);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_is_one_csv_line() {
        let start = Instant::now();
        let mut line = String::new();
        format_line(
            &mut line,
            7,
            start,
            &Record {
                kind: Kind::Frame,
                cause: Some(Cause::Timer),
                at: start + Duration::from_micros(1_500_250),
                interval: Duration::from_nanos(33_333_333),
                size: Size::new(800, 600),
                rects: &[Rect::new(10, 20, 74, 84), Rect::new(0, 0, 2, 3)],
                compose: Duration::from_micros(41),
                present: Duration::from_micros(120),
            },
        );
        assert_eq!(
            line,
            "7,1500.250,frame,timer,33.333,800,600,2,4102,41,120,10 20 64 64;0 0 2 3\n"
        );
        assert_eq!(line.split(',').count(), HEADER.split(',').count());
    }

    #[test]
    fn a_paint_has_no_cause() {
        let start = Instant::now();
        let mut line = String::new();
        format_line(
            &mut line,
            1,
            start,
            &Record {
                kind: Kind::Paint,
                cause: None,
                at: start,
                interval: Duration::from_nanos(16_666_666),
                size: Size::new(4, 2),
                rects: &[Rect::new(0, 0, 4, 2)],
                compose: Duration::ZERO,
                present: Duration::from_micros(9),
            },
        );
        assert_eq!(line, "1,0.000,paint,,16.667,4,2,1,8,0,9,0 0 4 2\n");
    }
}
