//! The parts of the interface that meiga draws itself, as layouts without
//! Windows: where each part goes, at the scale of the window, and what is
//! under a point.

pub mod menu;
pub mod tabbar;

/// The dots per inch at a scale of 100 %, where the sizes of the parts are
/// in pixels.
const BASE_DPI: u32 = 96;

/// `pixels` at a scale of 100 %, at `dpi`: rounded, and at least one.
fn scale(pixels: u32, dpi: u32) -> u32 {
    (pixels.saturating_mul(dpi).saturating_add(BASE_DPI / 2) / BASE_DPI).max(1)
}

/// A coordinate, which a part of the window never reaches the end of.
fn coord(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
