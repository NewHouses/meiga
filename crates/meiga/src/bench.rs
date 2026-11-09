//! The benchmarks (`--bench`): frames without a pause, to measure what a
//! partial frame, a full frame and a full frame of text cost to meiga and to
//! the rest of the system.

use meiga_core::color::{BACKGROUND, FOREGROUND, Rgb};
use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::text::CellChar;
use meiga_render::{Canvas, Font, FontStyle};

use crate::options::BenchMode;

/// The side of the square of the partial frames, and how far it moves at
/// each frame.
const SQUARE: u32 = 64;
const STEP: u64 = 16;
const SQUARE_COLOR: Rgb = Rgb::new(0x3b, 0x82, 0xf6);

/// The colours of the full frames, one at each frame. They are close, so
/// that the window does not flash.
const FULL_COLORS: [Rgb; 2] = [BACKGROUND, Rgb::new(0x1c, 0x1f, 0x26)];

/// The characters of the text frames: the printable ones of ASCII, from `!`
/// to `~`, which never leave a cell blank.
const FIRST_CHAR: u8 = b'!';
const CHARS: u64 = 94;

pub struct Bench {
    mode: BenchMode,
    frame: u64,
    square: Rect,
    /// The font of the text frames, and the cells of a row, which are kept
    /// to fill the next one without allocating memory.
    text: Option<(Font, Vec<CellChar>)>,
}

impl Bench {
    /// The benchmark `mode`. The text one needs `font`, which the others do
    /// not use: without it, its frames are empty.
    pub fn new(mode: BenchMode, font: Option<Font>) -> Self {
        Self {
            mode,
            frame: 0,
            square: Rect::EMPTY,
            text: font.map(|font| (font, Vec::new())),
        }
    }

    /// Goes to the next frame in a window of `size`, and returns the areas
    /// that change.
    pub fn step(&mut self, size: Size) -> [Rect; 2] {
        self.frame += 1;
        match self.mode {
            BenchMode::Partial => {
                let old = self.square;
                self.square = square_at(self.frame, size);
                [old, self.square]
            }
            BenchMode::Full | BenchMode::Text => [Rect::from_size(size), Rect::EMPTY],
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<'_>) {
        match self.mode {
            BenchMode::Partial => {
                canvas.fill(canvas.clip(), BACKGROUND);
                canvas.fill(self.square, SQUARE_COLOR);
            }
            BenchMode::Full => {
                let color = FULL_COLORS[usize::from(self.frame % 2 == 1)];
                canvas.fill(canvas.clip(), color);
            }
            BenchMode::Text => {
                let Some((font, row)) = &mut self.text else {
                    canvas.fill(canvas.clip(), BACKGROUND);
                    return;
                };
                // The rows cover the whole window, the last ones cut by its
                // edges, so nothing else needs a background.
                let cell = font.cell();
                let size = canvas.size();
                let columns = size.width.div_ceil(cell.width);
                for y in 0..size.height.div_ceil(cell.height) {
                    row.clear();
                    row.extend((0..columns).map(|x| text_at(self.frame, y, x)));
                    let top = i32::try_from(y * cell.height).unwrap_or(i32::MAX);
                    font.draw(
                        canvas,
                        Point::new(0, top),
                        row,
                        FontStyle::REGULAR,
                        FOREGROUND,
                        BACKGROUND,
                    );
                }
            }
        }
    }
}

/// Where the square is at `frame`: it crosses the middle of the window from
/// left to right, `STEP` pixels at each frame, and starts again.
fn square_at(frame: u64, size: Size) -> Rect {
    // The positions where all of it fits, from 0 to width - SQUARE.
    let positions = u64::from(size.width.saturating_sub(SQUARE)) + 1;
    let x = i32::try_from(frame.wrapping_mul(STEP) % positions).unwrap_or(0);
    let y = i32::try_from(size.height / 2).unwrap_or(0) - SQUARE as i32 / 2;
    Rect::new(x, y, x + SQUARE as i32, y + SQUARE as i32)
}

/// The character of the cell at `row` and `column` in `frame`: the text
/// moves one cell to the left at each frame, so every cell changes.
fn text_at(frame: u64, row: u32, column: u32) -> CellChar {
    let index = (u64::from(row) * 7 + u64::from(column) + frame) % CHARS;
    // Less than CHARS, so it fits in a byte, and the sum stays in ASCII.
    CellChar::Narrow(char::from(FIRST_CHAR + index as u8))
}

#[cfg(test)]
mod tests {
    use meiga_render::{FontSystem, pixel};

    use super::*;
    use crate::view::{FONT_FAMILIES, FONT_POINTS};

    const SIZE: Size = Size::new(200, 100);

    #[test]
    fn the_square_moves_and_changes_where_it_was_and_where_it_is() {
        let mut bench = Bench::new(BenchMode::Partial, None);
        assert_eq!(bench.step(SIZE), [Rect::EMPTY, Rect::new(16, 18, 80, 82)]);
        assert_eq!(
            bench.step(SIZE),
            [Rect::new(16, 18, 80, 82), Rect::new(32, 18, 96, 82)]
        );
    }

    #[test]
    fn the_square_starts_again_when_it_reaches_the_right_side() {
        // 200 - 64 = 136, so it fits at x = 0, 16, ..., 128, and 144 is
        // position 7 of the next round.
        assert_eq!(square_at(8, SIZE).left, 128);
        assert_eq!(square_at(9, SIZE).left, 7);
        assert_eq!(square_at(3, Size::new(10, 10)).left, 0);
    }

    #[test]
    fn full_frames_change_the_whole_window_and_its_colour() {
        let mut bench = Bench::new(BenchMode::Full, None);
        let mut pixels = vec![0; SIZE.area() as usize];
        let mut colors = Vec::new();
        for _ in 0..2 {
            assert_eq!(bench.step(SIZE), [Rect::from_size(SIZE), Rect::EMPTY]);
            let mut canvas = Canvas::new(&mut pixels, SIZE, SIZE.width as usize).unwrap();
            bench.draw(&mut canvas);
            colors.push(canvas.get(199, 99).unwrap());
        }
        assert_eq!(colors, [pixel(FULL_COLORS[1]), pixel(FULL_COLORS[0])]);
    }

    #[test]
    fn text_frames_fill_the_whole_window_and_change_every_cell() {
        let font = Font::new(&FontSystem::new().unwrap(), &FONT_FAMILIES, FONT_POINTS, 96).unwrap();
        let mut bench = Bench::new(BenchMode::Text, Some(font));
        assert_eq!(bench.step(SIZE), [Rect::from_size(SIZE), Rect::EMPTY]);
        let mut pixels = vec![0; SIZE.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, SIZE, SIZE.width as usize).unwrap();
        bench.draw(&mut canvas);
        assert!(pixels.iter().all(|&p| p != 0));
        assert!(pixels.contains(&pixel(FOREGROUND)));
        for (row, column) in [(0, 0), (3, 17)] {
            assert_ne!(text_at(1, row, column), text_at(2, row, column));
            assert_ne!(text_at(1, row, column), CellChar::Blank);
        }
        assert_eq!(text_at(0, 0, 0), CellChar::Narrow('!'));
        assert_eq!(text_at(93, 0, 0), CellChar::Narrow('~'));
    }
}
