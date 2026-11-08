//! Fonts with DirectWrite: the font of the cells, the size of a cell, and
//! the coverage of each glyph.
//!
//! DirectWrite is a COM API, which the `windows` crate wraps: its methods
//! check the HRESULT and manage the reference counts, but calling them is
//! still `unsafe`, and every block explains why it is sound.

use std::mem::ManuallyDrop;
use std::ptr;
use std::rc::Rc;

use meiga_core::geometry::{Rect, Size};
use windows::Win32::Foundation::{E_FAIL, RECT};
use windows::Win32::Globalization::GetUserDefaultLocaleName;
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_METRICS, DWRITE_FONT_STRETCH_NORMAL,
    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_GLYPH_METRICS, DWRITE_GLYPH_RUN,
    DWRITE_GRID_FIT_MODE, DWRITE_GRID_FIT_MODE_ENABLED, DWRITE_MEASURING_MODE_NATURAL,
    DWRITE_READING_DIRECTION, DWRITE_READING_DIRECTION_LEFT_TO_RIGHT, DWRITE_RENDERING_MODE,
    DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC, DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
    DWRITE_TEXTURE_ALIASED_1x1, DWriteCreateFactory, IDWriteFactory2, IDWriteFontCollection,
    IDWriteFontFace, IDWriteFontFallback, IDWriteNumberSubstitution, IDWriteTextAnalysisSource,
    IDWriteTextAnalysisSource_Impl,
};
use windows::core::{BOOL, HSTRING};
use windows_core::{OutRef, implement};

use crate::Error;

/// The longest locale name, with its null: `LOCALE_NAME_MAX_LENGTH`.
const LOCALE_NAME_MAX_LENGTH: usize = 85;

/// DirectWrite and the fonts of the system, shared by every [`Typeface`].
///
/// Creating it reads the list of fonts of the system, which takes about
/// 1.5 ms, so it is done once.
#[derive(Clone)]
pub struct FontSystem {
    factory: IDWriteFactory2,
    collection: IDWriteFontCollection,
    /// The font fallback of the system: which installed font has a
    /// character that the font of the cells lacks.
    fallback: IDWriteFontFallback,
    /// The locale of the user, with its null. The fallback uses it to choose
    /// among fonts, for example for the CJK ideographs, which Chinese and
    /// Japanese draw differently.
    locale: Rc<[u16]>,
}

impl FontSystem {
    pub fn new() -> Result<Self, Error> {
        // SAFETY: DWriteCreateFactory has no preconditions. The shared
        // factory is the one of the process, with its caches.
        let factory: IDWriteFactory2 = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }
            .map_err(|error| Error::com("DWriteCreateFactory", error))?;
        let mut collection = None;
        // SAFETY: `collection` receives the collection. FALSE: do not look
        // for fonts installed while the program runs.
        unsafe { factory.GetSystemFontCollection(&mut collection, false) }
            .map_err(|error| Error::com("IDWriteFactory::GetSystemFontCollection", error))?;
        let collection = collection.ok_or_else(|| {
            Error::com(
                "IDWriteFactory::GetSystemFontCollection",
                windows_core::Error::from_hresult(E_FAIL),
            )
        })?;
        // SAFETY: GetSystemFontFallback has no parameters.
        let fallback = unsafe { factory.GetSystemFontFallback() }
            .map_err(|error| Error::com("IDWriteFactory2::GetSystemFontFallback", error))?;
        Ok(Self {
            factory,
            collection,
            fallback,
            locale: user_locale(),
        })
    }
}

/// The locale of the user, such as `gl-ES`, with its null, or `en-US` if
/// Windows does not give it.
fn user_locale() -> Rc<[u16]> {
    let mut name = [0; LOCALE_NAME_MAX_LENGTH];
    // SAFETY: the buffer has room for the longest name and its null.
    let len = unsafe { GetUserDefaultLocaleName(&mut name) };
    match usize::try_from(len) {
        // The length includes the null.
        Ok(len @ 2..=LOCALE_NAME_MAX_LENGTH) => name[..len].into(),
        _ => "en-US\0".encode_utf16().collect(),
    }
}

/// A font for the cells of a grid, at a size and a DPI: the size of a cell,
/// where its baseline is, and the coverage of its glyphs.
pub struct Typeface {
    system: FontSystem,
    /// The family that was found, among the ones that were asked for.
    family: String,
    family_name: HSTRING,
    face: IDWriteFontFace,
    /// The size of the em square, in pixels.
    em: f32,
    cell: Size,
    /// The distance from the top of a cell to the baseline, in pixels.
    baseline: f32,
    /// The texture of the last glyph, kept to rasterize the next one without
    /// allocating memory.
    texture: Vec<u8>,
}

impl Typeface {
    /// The first of `families` that is installed, at `points` and `dpi`.
    pub fn new(
        system: &FontSystem,
        families: &[&str],
        points: f32,
        dpi: u32,
    ) -> Result<Self, Error> {
        let Some((family, face)) = families.iter().find_map(|&name| find(system, name)) else {
            return Err(Error::no_font(families));
        };
        let em = points * dpi as f32 / 72.0;
        let metrics = font_metrics(&face);
        let units = em / f32::from(metrics.designUnitsPerEm.max(1));
        let ascent = f32::from(metrics.ascent) * units;
        let descent = f32::from(metrics.descent) * units;
        let half_gap = f32::from(metrics.lineGap) * units / 2.0;
        // The baseline is rounded first, so that what goes below it, the
        // descent, also fits in the cell.
        let baseline = (ascent + half_gap).round();
        let height = (baseline + descent + half_gap).round();
        // In a monospaced font, every character advances the same.
        let advance = glyph_index(&face, '0')
            .and_then(|glyph| advance(&face, glyph, em))
            .unwrap_or(em / 2.0);
        // Both are small positive numbers, so the conversions are exact.
        let cell = Size::new(advance.round().max(1.0) as u32, height.max(1.0) as u32);
        Ok(Self {
            system: system.clone(),
            family_name: HSTRING::from(family.as_str()),
            family,
            face,
            em,
            cell,
            baseline,
            texture: Vec::new(),
        })
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub const fn cell(&self) -> Size {
        self.cell
    }

    /// Rasterizes `c` in a box of `columns` cells, 1 or 2, and writes its
    /// coverage in `coverage`: rows of the width of the box, as many as the
    /// height of a cell. What goes outside the box is cut.
    ///
    /// If the font of the cells does not have the character, it uses the
    /// font that the fallback of the system chooses, and if no font has it,
    /// the glyph of the missing characters of the font of the cells. The
    /// glyph is centred in the box when it advances a different width.
    pub fn rasterize(&mut self, c: char, columns: u32, coverage: &mut [u8]) -> Result<(), Error> {
        let width = self.cell.width.saturating_mul(columns);
        let size = Size::new(width, self.cell.height);
        if coverage.len() < size.area() as usize {
            return Ok(());
        }
        let (face, glyph, em) = self.face_for(c);
        let left = advance(&face, glyph, em)
            .map(|advance| ((width as f32 - advance) / 2.0).round())
            .unwrap_or(0.0);
        let (mode, grid_fit) = rendering();
        let no_advance = 0.0_f32;
        let run = DWRITE_GLYPH_RUN {
            fontFace: ManuallyDrop::new(Some(face)),
            fontEmSize: em,
            glyphCount: 1,
            glyphIndices: &glyph,
            glyphAdvances: &no_advance,
            glyphOffsets: ptr::null(),
            isSideways: BOOL(0),
            bidiLevel: 0,
        };
        // SAFETY: the glyph run points to `glyph` and `no_advance`, which
        // outlive the call, and holds a reference to the face. No transform:
        // the em size is in pixels.
        let analysis = unsafe {
            self.system.factory.CreateGlyphRunAnalysis(
                &run,
                None,
                mode,
                DWRITE_MEASURING_MODE_NATURAL,
                grid_fit,
                DWRITE_TEXT_ANTIALIAS_MODE_GRAYSCALE,
                left,
                self.baseline,
            )
        };
        // The run held a reference to the face, which ManuallyDrop does not
        // release by itself.
        drop(ManuallyDrop::into_inner(run.fontFace));
        let analysis = analysis
            .map_err(|error| Error::com("IDWriteFactory2::CreateGlyphRunAnalysis", error))?;

        // With grayscale antialiasing, the "aliased" texture holds one byte
        // of coverage per pixel, from 0 to 255.
        // SAFETY: GetAlphaTextureBounds only takes the texture type.
        let bounds = unsafe { analysis.GetAlphaTextureBounds(DWRITE_TEXTURE_ALIASED_1x1) }
            .map_err(|error| Error::com("IDWriteGlyphRunAnalysis::GetAlphaTextureBounds", error))?;
        let bounds = Rect::new(bounds.left, bounds.top, bounds.right, bounds.bottom);
        let visible = bounds.intersection(Rect::from_size(size));
        if visible.is_empty() {
            return Ok(());
        }
        self.texture.clear();
        self.texture.resize(bounds.area() as usize, 0);
        let texture_bounds = RECT {
            left: bounds.left,
            top: bounds.top,
            right: bounds.right,
            bottom: bounds.bottom,
        };
        // SAFETY: the texture has one byte for each pixel of the bounds.
        unsafe {
            analysis.CreateAlphaTexture(
                DWRITE_TEXTURE_ALIASED_1x1,
                &texture_bounds,
                &mut self.texture,
            )
        }
        .map_err(|error| Error::com("IDWriteGlyphRunAnalysis::CreateAlphaTexture", error))?;

        // The visible part is inside both the texture and the box, so the
        // differences are not negative.
        let texture_width = bounds.width() as usize;
        let (from, to) = (
            (visible.left - bounds.left) as usize,
            (visible.right - bounds.left) as usize,
        );
        for y in visible.top..visible.bottom {
            let source = (y - bounds.top) as usize * texture_width;
            let target = y as usize * width as usize + visible.left as usize;
            coverage[target..target + (to - from)]
                .copy_from_slice(&self.texture[source + from..source + to]);
        }
        Ok(())
    }

    /// The face, the glyph and the em size with which to draw `c`.
    fn face_for(&self, c: char) -> (IDWriteFontFace, u16, f32) {
        if let Some(glyph) = glyph_index(&self.face, c) {
            return (self.face.clone(), glyph, self.em);
        }
        self.fallback(c)
            .unwrap_or_else(|| (self.face.clone(), 0, self.em))
    }

    /// The font that the fallback of the system chooses for `c`, if any has
    /// it.
    fn fallback(&self, c: char) -> Option<(IDWriteFontFace, u16, f32)> {
        let mut units = [0; 2];
        let text = c.encode_utf16(&mut units);
        let len = text.len() as u32;
        let source: IDWriteTextAnalysisSource = OneChar {
            text: units,
            len,
            locale: self.system.locale.to_vec(),
        }
        .into();
        let (mut mapped_len, mut font, mut scale) = (0, None, 1.0);
        // SAFETY: the source, the collection and the name are valid during
        // the call, and the three out parameters receive the result.
        unsafe {
            self.system.fallback.MapCharacters(
                &source,
                0,
                len,
                &self.system.collection,
                &self.family_name,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                &mut mapped_len,
                &mut font,
                &mut scale,
            )
        }
        .ok()?;
        // SAFETY: CreateFontFace has no parameters.
        let face = unsafe { font?.CreateFontFace() }.ok()?;
        let glyph = glyph_index(&face, c)?;
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        Some((face, glyph, self.em * scale))
    }
}

/// How DirectWrite rasterizes the glyphs: antialiasing in both directions,
/// and the outlines fitted to the pixel grid (hinting), as the font says.
fn rendering() -> (DWRITE_RENDERING_MODE, DWRITE_GRID_FIT_MODE) {
    (
        DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC,
        DWRITE_GRID_FIT_MODE_ENABLED,
    )
}

/// The family `name` and its face of normal weight, width and style, if the
/// family is installed.
fn find(system: &FontSystem, name: &str) -> Option<(String, IDWriteFontFace)> {
    let (mut index, mut exists) = (0, BOOL(0));
    // SAFETY: the name is valid during the call, and the two out parameters
    // receive the result.
    unsafe {
        system
            .collection
            .FindFamilyName(&HSTRING::from(name), &mut index, &mut exists)
    }
    .ok()?;
    if !exists.as_bool() {
        return None;
    }
    // SAFETY: `index` is the one that FindFamilyName returned.
    let family = unsafe { system.collection.GetFontFamily(index) }.ok()?;
    // SAFETY: GetFirstMatchingFont only takes the three properties.
    let font = unsafe {
        family.GetFirstMatchingFont(
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
        )
    }
    .ok()?;
    // SAFETY: CreateFontFace has no parameters.
    let face = unsafe { font.CreateFontFace() }.ok()?;
    Some((name.to_owned(), face))
}

fn font_metrics(face: &IDWriteFontFace) -> DWRITE_FONT_METRICS {
    let mut metrics = DWRITE_FONT_METRICS::default();
    // SAFETY: `metrics` receives the metrics.
    unsafe { face.GetMetrics(&mut metrics) };
    metrics
}

/// The glyph of `c` in `face`, or `None` if the face does not have it.
fn glyph_index(face: &IDWriteFontFace, c: char) -> Option<u16> {
    let code_point = u32::from(c);
    let mut glyph = 0;
    // SAFETY: one code point in, one glyph index out.
    unsafe { face.GetGlyphIndices(&code_point, 1, &mut glyph) }.ok()?;
    (glyph != 0).then_some(glyph)
}

/// How far `glyph` advances at `em` pixels per em.
fn advance(face: &IDWriteFontFace, glyph: u16, em: f32) -> Option<f32> {
    let mut metrics = DWRITE_GLYPH_METRICS::default();
    // SAFETY: one glyph index in, its metrics out. FALSE: horizontal text.
    unsafe { face.GetDesignGlyphMetrics(&glyph, 1, &mut metrics, false) }.ok()?;
    let units = f32::from(font_metrics(face).designUnitsPerEm.max(1));
    Some(metrics.advanceWidth as f32 * em / units)
}

/// The text of one character, which the font fallback analyses: DirectWrite
/// asks for it through the COM interface `IDWriteTextAnalysisSource`.
#[implement(IDWriteTextAnalysisSource)]
struct OneChar {
    /// The character in UTF-16: one unit, or two (a surrogate pair).
    text: [u16; 2],
    len: u32,
    /// The locale, with its null.
    locale: Vec<u16>,
}

impl IDWriteTextAnalysisSource_Impl for OneChar_Impl {
    fn GetTextAtPosition(
        &self,
        position: u32,
        text: *mut *mut u16,
        len: *mut u32,
    ) -> windows_core::Result<()> {
        let rest = self.len.saturating_sub(position);
        // SAFETY: DirectWrite passes two valid out parameters. The text lives
        // as long as this object, which DirectWrite holds while it reads it,
        // and it only reads it: the mutable pointer is how the binding
        // spells `WCHAR const**`.
        unsafe {
            *text = if rest == 0 {
                ptr::null_mut()
            } else {
                self.text.as_ptr().add(position as usize).cast_mut()
            };
            *len = rest;
        }
        Ok(())
    }

    fn GetTextBeforePosition(
        &self,
        position: u32,
        text: *mut *mut u16,
        len: *mut u32,
    ) -> windows_core::Result<()> {
        let before = if position <= self.len { position } else { 0 };
        // SAFETY: as in GetTextAtPosition.
        unsafe {
            *text = if before == 0 {
                ptr::null_mut()
            } else {
                self.text.as_ptr().cast_mut()
            };
            *len = before;
        }
        Ok(())
    }

    fn GetParagraphReadingDirection(&self) -> DWRITE_READING_DIRECTION {
        DWRITE_READING_DIRECTION_LEFT_TO_RIGHT
    }

    fn GetLocaleName(
        &self,
        position: u32,
        len: *mut u32,
        locale: *mut *mut u16,
    ) -> windows_core::Result<()> {
        // SAFETY: as in GetTextAtPosition: the whole text has the same
        // locale.
        unsafe {
            *len = self.len.saturating_sub(position);
            *locale = self.locale.as_ptr().cast_mut();
        }
        Ok(())
    }

    fn GetNumberSubstitution(
        &self,
        position: u32,
        len: *mut u32,
        substitution: OutRef<'_, IDWriteNumberSubstitution>,
    ) -> windows_core::Result<()> {
        // SAFETY: DirectWrite passes a valid out parameter.
        unsafe { *len = self.len.saturating_sub(position) };
        // No substitution: digits are drawn as they are.
        substitution.write(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAMILIES: [&str; 2] = ["Cascadia Mono", "Consolas"];

    fn typeface(dpi: u32) -> Typeface {
        Typeface::new(&FontSystem::new().unwrap(), &FAMILIES, 12.0, dpi).unwrap()
    }

    /// Rasterizes `c` in `columns` cells, and returns its coverage.
    fn coverage(typeface: &mut Typeface, c: char, columns: u32) -> Vec<u8> {
        let cell = typeface.cell();
        let mut coverage = vec![0; (cell.area() * u64::from(columns)) as usize];
        typeface.rasterize(c, columns, &mut coverage).unwrap();
        coverage
    }

    /// The columns of the box that have ink.
    fn inked_columns(coverage: &[u8], width: usize) -> Vec<usize> {
        (0..width)
            .filter(|&x| coverage.chunks_exact(width).any(|row| row[x] != 0))
            .collect()
    }

    #[test]
    fn the_cell_grows_with_the_dpi() {
        let sizes: Vec<Size> = [96, 120, 144].map(|dpi| typeface(dpi).cell()).to_vec();
        for pair in sizes.windows(2) {
            assert!(pair[1].width >= pair[0].width && pair[1].height > pair[0].height);
        }
        // Monospaced fonts are taller than wide.
        assert!(
            sizes
                .iter()
                .all(|size| size.height > size.width && size.width > 0)
        );
    }

    #[test]
    fn a_missing_font_is_an_error() {
        let system = FontSystem::new().unwrap();
        let error = Typeface::new(&system, &["No Such Font 1.3"], 12.0, 96)
            .err()
            .unwrap();
        assert_eq!(
            error.to_string(),
            "None of these fonts is installed: No Such Font 1.3."
        );
        assert!(FAMILIES.contains(&typeface(96).family()));
    }

    #[test]
    fn letters_have_grayscale_ink_inside_their_cell() {
        let mut typeface = typeface(96);
        let width = typeface.cell().width as usize;
        let m = coverage(&mut typeface, 'M', 1);
        assert!(m.contains(&255));
        assert!(m.iter().any(|&c| c != 0 && c != 255), "no antialiasing");
        // 'M' spans most of its cell; '|' only the middle.
        assert!(inked_columns(&m, width).len() >= width / 2);
        let bar = coverage(&mut typeface, '|', 1);
        assert!(inked_columns(&bar, width).len() <= 3);
        // A space has no ink.
        assert!(coverage(&mut typeface, ' ', 1).iter().all(|&c| c == 0));
    }

    #[test]
    fn descenders_go_below_the_baseline_and_stay_in_the_cell() {
        let mut typeface = typeface(144);
        let width = typeface.cell().width as usize;
        let baseline = typeface.baseline as usize;
        let rows_with_ink = |coverage: &[u8]| -> Vec<usize> {
            coverage
                .chunks_exact(width)
                .enumerate()
                .filter(|(_, row)| row.iter().any(|&c| c != 0))
                .map(|(y, _)| y)
                .collect()
        };
        let x = rows_with_ink(&coverage(&mut typeface, 'x', 1));
        let g = rows_with_ink(&coverage(&mut typeface, 'g', 1));
        assert!(x.iter().all(|&y| y <= baseline));
        assert!(g.iter().any(|&y| y > baseline + 1));
        assert!(g.last().unwrap() < &(typeface.cell().height as usize));
    }

    #[test]
    fn wide_characters_fill_both_cells_with_the_fallback_font() {
        let mut typeface = typeface(120);
        let width = typeface.cell().width as usize;
        // The font of the cells has no ideographs: the fallback finds one
        // that has them, and it is centred in the two cells.
        assert_eq!(glyph_index(&typeface.face, '中'), None);
        let ideograph = coverage(&mut typeface, '中', 2);
        let columns = inked_columns(&ideograph, 2 * width);
        assert!(columns.first().unwrap() < &width && columns.last().unwrap() >= &width);
    }

    #[test]
    fn a_character_that_no_font_has_draws_the_missing_glyph() {
        let mut typeface = typeface(96);
        // A noncharacter, which no font has.
        let missing = coverage(&mut typeface, '\u{fdd0}', 1);
        assert!(missing.iter().any(|&c| c != 0));
    }
}
