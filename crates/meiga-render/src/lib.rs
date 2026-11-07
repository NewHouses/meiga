//! CPU renderer of meiga.
//!
//! Composes in a framebuffer only what changed, and presents in the window
//! only the rectangles that changed. From subphase 1.3, it will also
//! rasterize glyphs once with DirectWrite into a cache.
//!
//! The `unsafe` code of the crate, which calls GDI, is in `gdi.rs`, and every
//! block explains why it is sound in a `// SAFETY:` comment. The other
//! modules forbid it.

mod canvas;
mod error;
mod gdi;

pub use canvas::{Canvas, pixel};
pub use error::Error;
pub use gdi::{Fit, Surface};
