//! Text in cells: the glyphs of a font, rasterized the first time that they
//! are drawn, and drawn on the canvas with the colours of each cell.

#![forbid(unsafe_code)]

use meiga_core::color::Rgb;
use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::text::CellChar;

use crate::Error;
use crate::canvas::{Blending, Canvas, Shades};
use crate::dwrite::{FontSystem, Typeface};
use crate::glyphs::{CAPACITY, GlyphCache, SlotId};

/// How text mixes with its background, until it can be configured
/// (subphase 8.3). With these values, a line of Cascadia Mono at 150 % has
/// the same ink as in Windows Terminal, within 0.5 %, both light text on a
/// dark background and dark text on a light one.
pub const BLENDING: Blending = Blending {
    gamma: 1.5,
    contrast: 0.5,
};

/// The pairs of colours whose shades a font keeps. A screen of a terminal
/// uses a few: when there are more, the oldest pair is computed again if it
/// comes back.
const SHADES: usize = 64;

/// A font for the cells of a grid, at a size and a DPI, with its glyph cache.
pub struct Font {
    typeface: Typeface,
    glyphs: GlyphCache,
    /// The two cells of the last wide character, which are rasterized
    /// together.
    wide: Vec<u8>,
    /// The shades of the last pairs of colours, the most recent last.
    shades: Vec<(Rgb, Rgb, Box<Shades>)>,
    blending: Blending,
}

impl Font {
    /// The first of `families` that is installed, at `points` and `dpi`.
    pub fn new(
        system: &FontSystem,
        families: &[&str],
        points: f32,
        dpi: u32,
    ) -> Result<Self, Error> {
        let typeface = Typeface::new(system, families, points, dpi)?;
        let glyphs = GlyphCache::new(typeface.cell(), CAPACITY);
        Ok(Self {
            typeface,
            glyphs,
            wide: Vec::new(),
            shades: Vec::new(),
            blending: BLENDING,
        })
    }

    /// The family that was found.
    pub fn family(&self) -> &str {
        self.typeface.family()
    }

    /// The size of a cell, in pixels.
    pub const fn cell(&self) -> Size {
        self.typeface.cell()
    }

    /// How many glyphs are in its cache.
    pub fn cached_glyphs(&self) -> usize {
        self.glyphs.len()
    }

    /// Draws `cells` in a row, from `at`, the top-left corner of the first
    /// one, with the colours `text` and `background`.
    ///
    /// Only the cells inside the clip rectangle of `canvas` are drawn: first
    /// the background of all of them, and then the ink of their glyphs. A
    /// glyph is rasterized the first time that it is drawn. If DirectWrite
    /// cannot rasterize one, it stays blank, and drawing goes on.
    pub fn draw(
        &mut self,
        canvas: &mut Canvas<'_>,
        at: Point,
        cells: &[CellChar],
        text: Rgb,
        background: Rgb,
    ) {
        let cell = self.cell();
        let (Ok(count), Ok(width), Ok(height)) = (
            i32::try_from(cells.len()),
            i32::try_from(cell.width),
            i32::try_from(cell.height),
        ) else {
            return;
        };
        let row = Rect::new(
            at.x,
            at.y,
            at.x.saturating_add(width.saturating_mul(count)),
            at.y.saturating_add(height),
        );
        let visible = row.intersection(canvas.clip());
        if visible.is_empty() || width == 0 {
            return;
        }
        canvas.fill(visible, background);

        // The cells that the visible part touches. `visible` is inside the
        // row, so the divisions are not negative.
        let first = ((visible.left - at.x) / width) as usize;
        let last = ((visible.right - 1 - at.x) / width) as usize;
        let Self {
            typeface,
            glyphs,
            wide,
            shades,
            blending,
        } = self;
        let shades = shades_for(shades, text, background, *blending);
        for (index, &c) in cells.iter().enumerate().take(last + 1).skip(first) {
            let Some(slot) = slot(glyphs, typeface, wide, c) else {
                continue;
            };
            let glyph = glyphs.glyph(slot);
            // `index` is less than `count`, which fits in an i32.
            let x = at.x.saturating_add(width.saturating_mul(index as i32));
            canvas.draw_coverage(
                Point::new(x, at.y),
                glyph.coverage,
                cell.width as usize,
                glyph.ink,
                shades,
            );
        }
    }
}

/// The shades of `text` on `background`, computed the first time that the
/// pair is used.
fn shades_for(
    cache: &mut Vec<(Rgb, Rgb, Box<Shades>)>,
    text: Rgb,
    background: Rgb,
    blending: Blending,
) -> &Shades {
    let index = match cache
        .iter()
        .rposition(|&(t, b, _)| t == text && b == background)
    {
        Some(index) => index,
        None => {
            if cache.len() == SHADES {
                cache.remove(0);
            }
            cache.push((
                text,
                background,
                Box::new(Shades::new(text, background, blending)),
            ));
            cache.len() - 1
        }
    };
    &cache[index].2
}

/// The slot of the glyph of `c`, rasterizing it if it is not in the cache,
/// or `None` for a blank cell.
fn slot(
    glyphs: &mut GlyphCache,
    typeface: &mut Typeface,
    wide: &mut Vec<u8>,
    c: CellChar,
) -> Option<SlotId> {
    if c == CellChar::Blank {
        return None;
    }
    if let Some(slot) = glyphs.find(c) {
        return Some(slot);
    }
    match c {
        CellChar::Blank => None,
        CellChar::Narrow(character) => Some(glyphs.insert(c, |coverage| {
            // A glyph that cannot be rasterized stays blank, until the cache
            // drops it.
            let _ = typeface.rasterize(character, 1, coverage);
        })),
        CellChar::WideLeft(character) | CellChar::WideRight(character) => {
            // The two halves are one glyph: it is rasterized once, in two
            // cells, and both halves go to the cache, because the other one
            // is usually drawn right after.
            let cell = glyphs.cell();
            let width = cell.width as usize;
            wide.clear();
            wide.resize(2 * cell.area() as usize, 0);
            let _ = typeface.rasterize(character, 2, wide);
            let wide = &*wide;
            let half = |right: bool| {
                move |coverage: &mut [u8]| {
                    let rows = wide.chunks_exact(2 * width);
                    for (row, out) in rows.zip(coverage.chunks_exact_mut(width)) {
                        out.copy_from_slice(&row[usize::from(right) * width..][..width]);
                    }
                }
            };
            // The other half first: the slot of the last insertion is the
            // one that stays valid.
            let right = matches!(c, CellChar::WideRight(_));
            let other = if right {
                CellChar::WideLeft(character)
            } else {
                CellChar::WideRight(character)
            };
            glyphs.insert(other, half(!right));
            Some(glyphs.insert(c, half(right)))
        }
    }
}

#[cfg(test)]
mod tests {
    use meiga_core::text::cells;

    use super::*;
    use crate::canvas::pixel;

    const TEXT: Rgb = Rgb::new(0xff, 0xff, 0xff);
    const BACKGROUND: Rgb = Rgb::new(0x10, 0x20, 0x30);

    fn font() -> Font {
        let system = FontSystem::new().unwrap();
        Font::new(&system, &["Cascadia Mono", "Consolas"], 12.0, 96).unwrap()
    }

    /// The pixels of a canvas of `size` after `draw`.
    fn draw(size: Size, draw: impl FnOnce(&mut Canvas<'_>)) -> Vec<u32> {
        let mut pixels = vec![0; size.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, size, size.width as usize).unwrap();
        draw(&mut canvas);
        pixels
    }

    #[test]
    fn a_row_paints_its_background_and_the_ink_of_its_glyphs() {
        let mut font = font();
        let cell = font.cell();
        let size = Size::new(cell.width * 4, cell.height);
        let row: Vec<CellChar> = cells("a b").collect();
        let pixels = draw(size, |canvas| {
            font.draw(canvas, Point::new(0, 0), &row, TEXT, BACKGROUND);
        });
        // Three cells: the fourth one keeps its pixels.
        let at = |x: u32, y: u32| pixels[(y * size.width + x) as usize];
        let cell_pixels = |n: u32| -> Vec<u32> {
            (0..cell.height)
                .flat_map(|y| (n * cell.width..(n + 1) * cell.width).map(move |x| at(x, y)))
                .collect()
        };
        assert!(cell_pixels(3).iter().all(|&p| p == 0));
        // The blank cell is only background; the others have ink.
        assert!(cell_pixels(1).iter().all(|&p| p == pixel(BACKGROUND)));
        for n in [0, 2] {
            assert!(cell_pixels(n).contains(&pixel(TEXT)));
        }
        assert_eq!(font.cached_glyphs(), 2);
    }

    #[test]
    fn only_the_cells_inside_the_clip_are_drawn_and_rasterized() {
        let mut font = font();
        let cell = font.cell();
        let size = Size::new(cell.width * 10, cell.height);
        let row: Vec<CellChar> = cells("abcdefghij").collect();
        // The clip covers the end of the third cell and the fourth one.
        let clip = Rect::new(
            3 * cell.width as i32 - 1,
            0,
            4 * cell.width as i32,
            cell.height as i32,
        );
        let pixels = draw(size, |canvas| {
            font.draw(
                &mut canvas.clipped(clip),
                Point::new(0, 0),
                &row,
                TEXT,
                BACKGROUND,
            );
        });
        assert_eq!(font.cached_glyphs(), 2);
        for (index, &pixel) in pixels.iter().enumerate() {
            let x = (index as u32 % size.width) as i32;
            let inside = x >= clip.left && x < clip.right;
            assert_eq!(pixel != 0, inside, "pixel {x}");
        }
    }

    #[test]
    fn a_wide_character_is_rasterized_once_for_both_cells() {
        let mut font = font();
        let cell = font.cell();
        let size = Size::new(cell.width * 2, cell.height);
        let row: Vec<CellChar> = cells("中").collect();
        // Only the right half is drawn, but both are cached.
        let right = Rect::new(
            cell.width as i32,
            0,
            2 * cell.width as i32,
            cell.height as i32,
        );
        draw(size, |canvas| {
            font.draw(
                &mut canvas.clipped(right),
                Point::new(0, 0),
                &row,
                TEXT,
                BACKGROUND,
            );
        });
        assert_eq!(font.cached_glyphs(), 2);
        let pixels = draw(size, |canvas| {
            font.draw(canvas, Point::new(0, 0), &row, TEXT, BACKGROUND);
        });
        assert_eq!(font.cached_glyphs(), 2);
        let inked = |range: std::ops::Range<u32>| {
            pixels.iter().enumerate().any(|(index, &p)| {
                range.contains(&(index as u32 % size.width)) && p != pixel(BACKGROUND)
            })
        };
        assert!(inked(0..cell.width) && inked(cell.width..2 * cell.width));
    }

    #[test]
    fn the_shades_of_the_oldest_pair_are_dropped_first() {
        let mut cache = Vec::new();
        for value in 0..=SHADES as u8 {
            shades_for(&mut cache, Rgb::new(value, 0, 0), BACKGROUND, BLENDING);
        }
        assert_eq!(cache.len(), SHADES);
        assert_eq!(cache[0].0, Rgb::new(1, 0, 0));
        let again = shades_for(&mut cache, Rgb::new(5, 0, 0), BACKGROUND, BLENDING).get(255);
        assert_eq!(again, pixel(Rgb::new(5, 0, 0)));
        assert_eq!(cache.len(), SHADES);
    }
}
