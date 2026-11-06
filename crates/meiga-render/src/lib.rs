//! CPU renderer of meiga.
//!
//! Rasterizes glyphs once with DirectWrite into a cache, composes only the
//! cells that changed into a framebuffer and presents only the changed
//! rectangles.
