//! Drawing on pixels in memory.

#![forbid(unsafe_code)]

use meiga_core::color::Rgb;
use meiga_core::geometry::{Point, Rect, Size};

/// A colour as a pixel of the framebuffer, `0x00RRGGBB`: in memory, the bytes
/// blue, green, red and one that is not used.
pub const fn pixel(color: Rgb) -> u32 {
    (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
}

/// How the coverage of a glyph becomes colour, between the background and the
/// colour of the text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blending {
    /// The power to which the two colours are raised before mixing them, in
    /// proportion to the coverage; the mix is raised to its inverse. With 1,
    /// the values of the pixels mix as they are. With more, they mix more as
    /// light does, and the edges get lighter: light text on a dark background
    /// looks heavier, and dark text on a light background thinner.
    pub gamma: f32,
    /// How much the partial coverage of the edges of black text grows before
    /// mixing, from 0 (not at all): the coverage `a` becomes
    /// `a (k + 1) / (a k + 1)`. Lighter text gets proportionally less, and
    /// white text none. It makes up for the gamma, which thins dark text.
    pub contrast: f32,
}

/// The 256 colours between a background and the colour of the text: the
/// colour of a pixel of text with each coverage, from 0 (none: the
/// background) to 255 (all: the colour of the text).
///
/// It is computed once for each pair of colours, so that drawing a pixel of
/// text is reading a table.
#[derive(Clone)]
pub struct Shades {
    pixels: [u32; 256],
}

impl Shades {
    pub fn new(text: Rgb, background: Rgb, blending: Blending) -> Self {
        let gamma = if blending.gamma.is_finite() && blending.gamma > 0.0 {
            blending.gamma
        } else {
            1.0
        };
        let contrast = if blending.contrast.is_finite() {
            // Light text gets lighter edges from the gamma: the contrast is
            // for dark text, all of it for black and none for white.
            let lightness =
                (0.30 * f32::from(text.r) + 0.59 * f32::from(text.g) + 0.11 * f32::from(text.b))
                    / 255.0;
            blending.contrast.max(0.0) * (1.0 - lightness)
        } else {
            0.0
        };
        let light = |value: u8| (f32::from(value) / 255.0).powf(gamma);
        let channels = [
            (light(text.r), light(background.r)),
            (light(text.g), light(background.g)),
            (light(text.b), light(background.b)),
        ];
        let mut pixels = [0; 256];
        for (coverage, shade) in pixels.iter_mut().enumerate() {
            let a = coverage as f32 / 255.0;
            let a = a * (contrast + 1.0) / (a * contrast + 1.0);
            let [r, g, b] = channels.map(|(text, background)| {
                // Inside 0..=1, so the conversion neither saturates nor wraps.
                ((background + (text - background) * a).powf(1.0 / gamma) * 255.0).round() as u8
            });
            *shade = pixel(Rgb::new(r, g, b));
        }
        // Exactly the two colours at the ends, without rounding errors.
        pixels[0] = pixel(background);
        pixels[255] = pixel(text);
        Self { pixels }
    }

    /// The pixel of text with `coverage`.
    pub const fn get(&self, coverage: u8) -> u32 {
        self.pixels[coverage as usize]
    }
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

    /// Draws the coverage of a glyph whose cell has its top-left corner at
    /// `at`. `coverage` has rows of `width` bytes, and only its rectangle
    /// `ink` is drawn: each pixel with coverage takes its colour from
    /// `shades`, and the others stay as they are, so they must already be
    /// the background of `shades`. If `ink` is not inside `coverage`,
    /// nothing is drawn.
    pub fn draw_coverage(
        &mut self,
        at: Point,
        coverage: &[u8],
        width: usize,
        ink: Rect,
        shades: &Shades,
    ) {
        let rows = coverage.len().checked_div(width).unwrap_or(0);
        let (Ok(columns), Ok(rows)) = (i32::try_from(width), i32::try_from(rows)) else {
            return;
        };
        if ink.is_empty() || !Rect::new(0, 0, columns, rows).contains(ink) {
            return;
        }
        let area = ink.offset(at).intersection(self.clip);
        if area.is_empty() {
            return;
        }
        // The area is inside the clip, so inside the canvas, and inside the
        // ink moved to `at`, so `x - at.x` and `y - at.y` are inside the
        // coverage: none of the differences is negative.
        let (from, to) = ((area.left - at.x) as usize, (area.right - at.x) as usize);
        for y in area.top..area.bottom {
            let source = (y - at.y) as usize * width;
            let target = y as usize * self.stride;
            let row = &coverage[source + from..source + to];
            let pixels =
                &mut self.pixels[target + area.left as usize..target + area.right as usize];
            for (pixel, &c) in pixels.iter_mut().zip(row) {
                if c != 0 {
                    *pixel = shades.get(c);
                }
            }
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

    const LINEAR: Blending = Blending {
        gamma: 1.0,
        contrast: 0.0,
    };
    const WHITE: Rgb = Rgb::new(0xff, 0xff, 0xff);
    const BLACK: Rgb = Rgb::new(0, 0, 0);

    #[test]
    fn shades_go_from_the_background_to_the_text() {
        let shades = Shades::new(WHITE, BLUE, LINEAR);
        assert_eq!(shades.get(0), pixel(BLUE));
        assert_eq!(shades.get(255), pixel(WHITE));
        assert_eq!(shades.get(51), pixel(Rgb::new(51, 51, 0xff)));
    }

    #[test]
    fn gamma_lightens_the_edges_and_contrast_darkens_those_of_dark_text() {
        let shade =
            |text, background, blending| Shades::new(text, background, blending).get(128) & 0xff;
        let gamma = Blending {
            gamma: 2.0,
            ..LINEAR
        };
        let contrast = Blending {
            contrast: 1.0,
            ..LINEAR
        };
        // Light text: the gamma makes it heavier, and the contrast does not
        // touch white.
        assert_eq!(shade(WHITE, BLACK, LINEAR), 128);
        assert!(shade(WHITE, BLACK, gamma) > 128);
        assert_eq!(shade(WHITE, BLACK, contrast), 128);
        // Dark text: the gamma makes it thinner, and the contrast heavier.
        assert_eq!(shade(BLACK, WHITE, LINEAR), 127);
        assert!(shade(BLACK, WHITE, gamma) > 127);
        assert!(shade(BLACK, WHITE, contrast) < 127);
        // Values that make no sense mix as linear ones.
        let nonsense = Blending {
            gamma: f32::NAN,
            contrast: -3.0,
        };
        assert_eq!(shade(BLACK, WHITE, nonsense), 127);
    }

    #[test]
    fn coverage_paints_only_the_pixels_with_ink_inside_the_clip() {
        let mut pixels = [0; 8 * 4];
        let mut canvas = Canvas::new(&mut pixels, Size::new(8, 4), 8).unwrap();
        let shades = Shades::new(WHITE, BLACK, LINEAR);
        // A cell of 3 × 2 with coverage in three pixels.
        let coverage = [255, 0, 51, 0, 0, 255];
        canvas.draw_coverage(
            Point::new(2, 1),
            &coverage,
            3,
            Rect::new(0, 0, 3, 2),
            &shades,
        );
        assert_eq!(canvas.get(2, 1), Some(pixel(WHITE)));
        assert_eq!(canvas.get(3, 1), Some(0));
        assert_eq!(canvas.get(4, 1), Some(pixel(Rgb::new(51, 51, 51))));
        assert_eq!(canvas.get(4, 2), Some(pixel(WHITE)));

        // Cut by the clip, and by the edge of the canvas.
        let mut clipped = canvas.clipped(Rect::new(0, 0, 8, 2));
        clipped.draw_coverage(
            Point::new(6, 1),
            &coverage,
            3,
            Rect::new(0, 0, 3, 2),
            &shades,
        );
        assert_eq!(canvas.get(6, 1), Some(pixel(WHITE)));
        assert_eq!(canvas.get(6, 2), Some(0));
        assert_eq!(pixels.iter().filter(|&&p| p != 0).count(), 4);
    }

    #[test]
    fn coverage_outside_its_rows_draws_nothing() {
        let mut pixels = [0; 4 * 4];
        let mut canvas = Canvas::new(&mut pixels, Size::new(4, 4), 4).unwrap();
        let shades = Shades::new(WHITE, BLACK, LINEAR);
        let coverage = [255; 4];
        for ink in [Rect::new(0, 0, 2, 3), Rect::new(0, 0, 3, 2)] {
            canvas.draw_coverage(Point::new(0, 0), &coverage, 2, ink, &shades);
        }
        canvas.draw_coverage(
            Point::new(0, 0),
            &coverage,
            0,
            Rect::new(0, 0, 1, 1),
            &shades,
        );
        assert!(pixels.iter().all(|&p| p == 0));
    }
}
