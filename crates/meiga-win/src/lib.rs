//! Windows platform layer of meiga.
//!
//! Wraps ConPTY, shell profile discovery, the file system, the clipboard, the
//! Win32 window and DPI handling. Every `unsafe` block in this crate must
//! explain why it is sound in a `// SAFETY:` comment.

pub mod conpty;
mod error;
pub mod event;
pub mod window;

pub use error::Error;
// The handles of a window and of a device context, which the binary passes
// from this crate to meiga-render. They are raw pointers: copying them is
// safe, and only the functions of Windows use them.
pub use windows_sys::Win32::Foundation::HWND;
pub use windows_sys::Win32::Graphics::Gdi::HDC;

/// Converts `text` to the null-terminated UTF-16 that the `W` functions of
/// Windows expect.
pub(crate) fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
