//! CPU renderer of meiga.
//!
//! Composes in a framebuffer only what changed, and presents in the window
//! only the rectangles that changed. Text is drawn from a cache of glyphs,
//! which DirectWrite rasterizes once.
//!
//! The `unsafe` code of the crate is in `gdi.rs`, which calls GDI, and in
//! `dwrite.rs`, which calls DirectWrite. Every block explains why it is
//! sound in a `// SAFETY:` comment. The other modules forbid it.

mod canvas;
mod dwrite;
mod error;
mod gdi;
mod glyphs;
mod text;

pub use canvas::{Blending, Canvas, Shades, pixel};
pub use dwrite::FontSystem;
pub use error::Error;
pub use gdi::{Fit, Surface};
pub use text::{BLENDING, Font};
