//! Drawing on pixels in memory.

#![forbid(unsafe_code)]

use meiga_core::color::Rgb;
use meiga_core::geometry::{Rect, Size};

/// A colour as a pixel of the framebuffer, `0x00RRGGBB`: in memory, the bytes
/// blue, green, red and one that is not used.
pub const fn pixel(color: Rgb) -> u32 {
    (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
}

/// Pixels in memory, row after row, to draw on.
///
/// Every drawing is cut to the clip rectangle, which is never outside the
/// canvas, so no drawing can write outside it.
pub struct Canvas<'a> {
    pixels: &'a mut [u32],
    /// Pixels from the start of a row to the start of the next one. It can be
    /// more than the width, when the memory is bigger than what is in use.
    stride: usize,
    size: Size,
    clip: Rect,
}

impl<'a> Canvas<'a> {
    /// A canvas of `size`, whose rows start every `stride` pixels in
    /// `pixels`. Returns `None` if the rows do not fit in `pixels`.
    pub fn new(pixels: &'a mut [u32], size: Size, stride: usize) -> Option<Self> {
        let width = usize::try_from(size.width).ok()?;
        let height = usize::try_from(size.height).ok()?;
        if width > stride || i32::try_from(size.width.max(size.height)).is_err() {
            return None;
        }
        // The last row only needs `width` pixels.
        let needed = match height {
            0 => 0,
            _ => stride.checked_mul(height - 1)?.checked_add(width)?,
        };
        if pixels.len() < needed {
            return None;
        }
        Some(Self {
            pixels,
            stride,
            size,
            clip: Rect::from_size(size),
        })
    }

    /// A canvas without pixels, on which nothing is drawn.
    pub fn empty() -> Canvas<'static> {
        Canvas {
            pixels: Default::default(),
            stride: 0,
            size: Size::default(),
            clip: Rect::EMPTY,
        }
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    /// The rectangle outside which nothing is drawn.
    pub const fn clip(&self) -> Rect {
        self.clip
    }

    /// The same pixels, with the drawings cut to `rect` too.
    pub fn clipped(&mut self, rect: Rect) -> Canvas<'_> {
        Canvas {
            pixels: &mut *self.pixels,
            stride: self.stride,
            size: self.size,
            clip: self.clip.intersection(rect),
        }
    }

    /// Fills `rect` with `color`.
    pub fn fill(&mut self, rect: Rect, color: Rgb) {
        let area = rect.intersection(self.clip);
        if area.is_empty() {
            return;
        }
        let value = pixel(color);
        // The clip is inside the canvas, so the coordinates are not negative.
        let (left, right) = (area.left as usize, area.right as usize);
        for y in area.top as usize..area.bottom as usize {
            let row = y * self.stride;
            self.pixels[row + left..row + right].fill(value);
        }
    }

    /// Draws the border of `rect`, `thickness` pixels wide, inside it.
    pub fn outline(&mut self, rect: Rect, thickness: u32, color: Rgb) {
        for side in rect.border(thickness) {
            self.fill(side, color);
        }
    }

    /// The pixel at (`x`, `y`), or `None` outside the canvas.
    pub fn get(&self, x: u32, y: u32) -> Option<u32> {
        if x >= self.size.width || y >= self.size.height {
            return None;
        }
        let index = usize::try_from(y)
            .ok()?
            .checked_mul(self.stride)?
            .checked_add(usize::try_from(x).ok()?)?;
        self.pixels.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgb = Rgb::new(0xff, 0, 0);
    const BLUE: Rgb = Rgb::new(0, 0, 0xff);

    #[test]
    fn pixels_are_red_green_blue_from_the_high_byte() {
        assert_eq!(pixel(Rgb::new(0x12, 0x34, 0x56)), 0x0012_3456);
    }

    #[test]
    fn rows_must_fit_in_the_memory() {
        let mut pixels = [0; 30];
        assert!(Canvas::new(&mut pixels, Size::new(10, 3), 10).is_some());
        assert!(Canvas::new(&mut pixels, Size::new(8, 3), 11).is_some());
        assert!(Canvas::new(&mut pixels, Size::new(10, 4), 10).is_none());
        assert!(Canvas::new(&mut pixels, Size::new(11, 2), 10).is_none());
    }

    #[test]
    fn fill_is_cut_to_the_canvas_and_respects_the_stride() {
        let mut pixels = [0; 6 * 4];
        let mut canvas = Canvas::new(&mut pixels, Size::new(4, 4), 6).unwrap();
        canvas.fill(Rect::new(-5, 2, 100, 100), RED);
        assert_eq!(canvas.get(0, 1), Some(0));
        assert_eq!(canvas.get(0, 2), Some(pixel(RED)));
        assert_eq!(canvas.get(3, 3), Some(pixel(RED)));
        assert_eq!(canvas.get(4, 3), None);
        // The two pixels after each row belong to nobody: they stay as they
        // were.
        assert_eq!(pixels[2 * 6 + 4..2 * 6 + 6], [0, 0]);
        assert_eq!(pixels[3 * 6..3 * 6 + 4], [pixel(RED); 4]);
    }

    #[test]
    fn clipped_canvases_only_draw_inside_the_clip() {
        let mut pixels = [0; 10 * 10];
        let mut canvas = Canvas::new(&mut pixels, Size::new(10, 10), 10).unwrap();
        let mut clipped = canvas.clipped(Rect::new(2, 2, 5, 5));
        assert_eq!(clipped.clip(), Rect::new(2, 2, 5, 5));
        clipped.fill(Rect::new(0, 0, 10, 10), BLUE);
        let mut twice = clipped.clipped(Rect::new(4, 4, 20, 20));
        assert_eq!(twice.clip(), Rect::new(4, 4, 5, 5));
        twice.fill(Rect::new(0, 0, 10, 10), RED);
        let painted = pixels.iter().filter(|&&p| p != 0).count();
        assert_eq!(painted, 9);
        assert_eq!(pixels[4 * 10 + 4], pixel(RED));
        assert_eq!(pixels[2 * 10 + 2], pixel(BLUE));
    }

    #[test]
    fn outline_draws_only_the_border() {
        let mut pixels = [0; 6 * 6];
        let mut canvas = Canvas::new(&mut pixels, Size::new(6, 6), 6).unwrap();
        canvas.outline(Rect::new(1, 1, 5, 5), 1, RED);
        assert_eq!(canvas.get(1, 1), Some(pixel(RED)));
        assert_eq!(canvas.get(4, 2), Some(pixel(RED)));
        assert_eq!(canvas.get(2, 2), Some(0));
        assert_eq!(canvas.get(0, 0), Some(0));
        assert_eq!(pixels.iter().filter(|&&p| p != 0).count(), 12);
    }

    #[test]
    fn an_empty_canvas_draws_nothing() {
        let mut canvas = Canvas::empty();
        canvas.fill(Rect::new(0, 0, 10, 10), RED);
        assert_eq!(canvas.get(0, 0), None);
    }
}
