//! The framebuffer in a DIB section, and its presentation in a window with
//! GDI.

use std::ptr::{self, NonNull};
use std::slice;

use meiga_core::color::Rgb;
use meiga_core::geometry::{Rect, Size};
use windows_sys::Win32::Foundation::{COLORREF, HWND, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleDC, CreateDIBSection,
    CreateSolidBrush, DIB_RGB_COLORS, DeleteDC, DeleteObject, FillRect, GdiFlush, GetDC, HBITMAP,
    HDC, HGDIOBJ, ReleaseDC, SRCCOPY, SelectObject,
};

use crate::Error;
use crate::canvas::Canvas;

/// How the framebuffer adapts to a new size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    /// Exactly the size, so that it uses no more memory than it needs.
    Exact,
    /// Keep the memory if the size fits in it and, if not, grow with room to
    /// spare: a quarter more in each direction. For when the user resizes the
    /// window with the mouse, so that the memory is not allocated again at
    /// every step.
    Room,
}

/// The framebuffer: pixels in memory, which are drawn with a [`Canvas`] and
/// copied to a window with GDI.
///
/// The pixels are a DIB section (a device-independent bitmap whose memory the
/// process can write directly), selected in a memory device context, from
/// which `BitBlt` copies them to the window.
pub struct Surface {
    dib: Dib,
    size: Size,
}

impl Surface {
    pub fn new(size: Size) -> Result<Self, Error> {
        Ok(Self {
            dib: Dib::new(size)?,
            size,
        })
    }

    /// The size in use, which is the size of the window.
    pub const fn size(&self) -> Size {
        self.size
    }

    /// The size of the memory, at least as big as [`Surface::size`].
    pub const fn capacity(&self) -> Size {
        self.dib.capacity
    }

    /// Changes the size in use.
    ///
    /// Returns `true` if the memory was allocated again: then its pixels are
    /// lost, and everything must be drawn again.
    pub fn resize(&mut self, size: Size, fit: Fit) -> Result<bool, Error> {
        let capacity = self.dib.capacity;
        let fits = size.width <= capacity.width && size.height <= capacity.height;
        let target = match fit {
            Fit::Exact if capacity == Dib::capacity_for(size) => None,
            Fit::Exact => Some(size),
            Fit::Room if fits => None,
            Fit::Room => Some(Size::new(
                with_room(size.width).max(capacity.width),
                with_room(size.height).max(capacity.height),
            )),
        };
        if let Some(target) = target {
            self.dib = Dib::new(target)?;
        }
        self.size = size;
        Ok(target.is_some())
    }

    /// The pixels in use, to draw on them.
    pub fn canvas(&mut self) -> Canvas<'_> {
        let stride = self.dib.capacity.width as usize;
        let len = stride * self.dib.capacity.height as usize;
        // SAFETY: `bits` points to the `len` pixels of the DIB section, which
        // are aligned and valid while `self.dib` exists. `&mut self` keeps
        // any other Rust code from using them while the canvas lives, and GDI
        // does not use them either: the only GDI calls that read them (the
        // copies in `copy`) are finished with GdiFlush before it returns.
        let pixels = unsafe { slice::from_raw_parts_mut(self.dib.bits.as_ptr(), len) };
        // The size in use always fits in the memory.
        Canvas::new(pixels, self.size, stride).unwrap_or_else(Canvas::empty)
    }

    /// Copies `rects` of the framebuffer to the same place in `window`.
    pub fn present(&self, window: HWND, rects: &[Rect]) -> Result<(), Error> {
        if rects.is_empty() {
            return Ok(());
        }
        let target = WindowDc::get(window)?;
        self.copy(target.0, rects)
    }

    /// Copies `rect` of the framebuffer to `dc`, for example the one that
    /// `BeginPaint` returns for `WM_PAINT`.
    pub fn paint(&self, dc: HDC, rect: Rect) -> Result<(), Error> {
        self.copy(dc, &[rect])
    }

    /// Draws in `window`, not in the framebuffer, the border of each of
    /// `rects`, `thickness` pixels wide. Presenting those rectangles again
    /// erases it.
    pub fn mark(
        &self,
        window: HWND,
        rects: &[Rect],
        color: Rgb,
        thickness: u32,
    ) -> Result<(), Error> {
        if rects.is_empty() {
            return Ok(());
        }
        let brush = Brush::new(color)?;
        let target = WindowDc::get(window)?;
        let bounds = Rect::from_size(self.size);
        let mut result = Ok(());
        for side in rects
            .iter()
            .flat_map(|rect| rect.intersection(bounds).border(thickness))
            .filter(|side| !side.is_empty())
        {
            let side = RECT {
                left: side.left,
                top: side.top,
                right: side.right,
                bottom: side.bottom,
            };
            // SAFETY: the DC and the brush are valid while `target` and
            // `brush` live, and `side` is a valid RECT.
            if unsafe { FillRect(target.0, &side, brush.0) } == 0 && result.is_ok() {
                result = Err(Error::last("FillRect"));
            }
        }
        result
    }

    /// Copies `rects`, cut to the size in use, to `target`.
    fn copy(&self, target: HDC, rects: &[Rect]) -> Result<(), Error> {
        let bounds = Rect::from_size(self.size);
        let mut result = Ok(());
        for rect in rects {
            let area = rect.intersection(bounds);
            if area.is_empty() {
                continue;
            }
            // The area is inside the size in use, so its sides fit in an i32.
            let (width, height) = (area.width() as i32, area.height() as i32);
            // SAFETY: both DCs are valid, and the area is inside the DIB, so
            // GDI only reads its pixels.
            let copied = unsafe {
                BitBlt(
                    target,
                    area.left,
                    area.top,
                    width,
                    height,
                    self.dib.dc,
                    area.left,
                    area.top,
                    SRCCOPY,
                )
            };
            if copied == 0 && result.is_ok() {
                result = Err(Error::last("BitBlt"));
            }
        }
        // GDI can queue the copies and do them later. They read the pixels,
        // which the canvas can change as soon as this returns, so they must
        // be finished now.
        // SAFETY: GdiFlush has no preconditions.
        unsafe { GdiFlush() };
        result
    }
}

/// A DIB section selected in a memory device context, which owns both.
struct Dib {
    dc: HDC,
    bitmap: HBITMAP,
    /// The bitmap that the device context had before, which must be selected
    /// again before deleting `bitmap`.
    previous: HGDIOBJ,
    bits: NonNull<u32>,
    capacity: Size,
}

impl Dib {
    /// The memory for `size`: at least one pixel, because GDI does not create
    /// empty bitmaps.
    fn capacity_for(size: Size) -> Size {
        Size::new(size.width.max(1), size.height.max(1))
    }

    fn new(size: Size) -> Result<Self, Error> {
        let capacity = Self::capacity_for(size);
        let (Ok(width), Ok(height)) = (
            i32::try_from(capacity.width),
            i32::try_from(capacity.height),
        ) else {
            return Err(Error::too_big("CreateDIBSection"));
        };
        if (capacity.area())
            .checked_mul(4)
            .and_then(|bytes| isize::try_from(bytes).ok())
            .is_none()
        {
            return Err(Error::too_big("CreateDIBSection"));
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                // Negative: the first row in memory is the top one, as in
                // the window.
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..Default::default()
            },
            ..Default::default()
        };

        // SAFETY: with a null handle, CreateCompatibleDC creates a memory
        // device context compatible with the screen.
        let dc = unsafe { CreateCompatibleDC(ptr::null_mut()) };
        if dc.is_null() {
            return Err(Error::last("CreateCompatibleDC"));
        }
        let mut bits = ptr::null_mut();
        // SAFETY: `info` describes a 32-bit DIB without a colour table, and
        // `bits` receives the address of its pixels. Without a file mapping
        // (null section), GDI allocates the memory.
        let bitmap =
            unsafe { CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, ptr::null_mut(), 0) };
        let bits = NonNull::new(bits.cast::<u32>());
        let (false, Some(bits)) = (bitmap.is_null(), bits) else {
            let error = Error::last("CreateDIBSection");
            // SAFETY: the device context was created above, and nothing else
            // uses it.
            unsafe { DeleteDC(dc) };
            return Err(error);
        };
        // SAFETY: both handles are valid. A new memory device context has a
        // 1 × 1 bitmap selected, which is returned.
        let previous = unsafe { SelectObject(dc, bitmap) };
        if previous.is_null() {
            let error = Error::last("SelectObject");
            // SAFETY: both were created above, and nothing else uses them.
            unsafe {
                DeleteObject(bitmap);
                DeleteDC(dc);
            }
            return Err(error);
        }
        Ok(Self {
            dc,
            bitmap,
            previous,
            bits,
            capacity,
        })
    }
}

impl Drop for Dib {
    fn drop(&mut self) {
        // SAFETY: `new` created the device context and the bitmap, and
        // nothing else uses them. GDI requires that a bitmap is not selected
        // when it is deleted, so the previous one is selected first.
        unsafe {
            SelectObject(self.dc, self.previous);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
        }
    }
}

/// The device context of a window, released when it is dropped.
struct WindowDc(HDC, HWND);

impl WindowDc {
    fn get(window: HWND) -> Result<Self, Error> {
        // SAFETY: GetDC accepts any handle and returns null for one that is
        // not a window.
        let dc = unsafe { GetDC(window) };
        if dc.is_null() {
            return Err(Error::last("GetDC"));
        }
        Ok(Self(dc, window))
    }
}

impl Drop for WindowDc {
    fn drop(&mut self) {
        // SAFETY: GetDC returned this device context for this window.
        unsafe { ReleaseDC(self.1, self.0) };
    }
}

/// A solid GDI brush, deleted when it is dropped.
struct Brush(HGDIOBJ);

impl Brush {
    fn new(color: Rgb) -> Result<Self, Error> {
        // SAFETY: CreateSolidBrush has no preconditions.
        let brush = unsafe { CreateSolidBrush(colorref(color)) };
        if brush.is_null() {
            return Err(Error::last("CreateSolidBrush"));
        }
        Ok(Self(brush))
    }
}

impl Drop for Brush {
    fn drop(&mut self) {
        // SAFETY: the brush was created in `new`, and it is no longer in use.
        unsafe { DeleteObject(self.0) };
    }
}

/// A quarter more than `value`.
fn with_room(value: u32) -> u32 {
    value.saturating_add(value / 4)
}

/// Converts a colour to a GDI `COLORREF`, which stores it as `0x00BBGGRR`.
const fn colorref(color: Rgb) -> COLORREF {
    color.r as u32 | (color.g as u32) << 8 | (color.b as u32) << 16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::pixel;

    const GREEN: Rgb = Rgb::new(0x20, 0xc0, 0x40);

    #[test]
    fn resize_with_room_keeps_the_memory_while_it_fits() {
        let mut surface = Surface::new(Size::new(400, 300)).unwrap();
        assert_eq!(surface.capacity(), Size::new(400, 300));
        assert!(surface.resize(Size::new(420, 300), Fit::Room).unwrap());
        assert_eq!(surface.capacity(), Size::new(525, 375));
        assert!(!surface.resize(Size::new(500, 200), Fit::Room).unwrap());
        assert_eq!(surface.size(), Size::new(500, 200));
        assert!(surface.resize(Size::new(500, 200), Fit::Exact).unwrap());
        assert_eq!(surface.capacity(), Size::new(500, 200));
        assert!(!surface.resize(Size::new(500, 200), Fit::Exact).unwrap());
    }

    #[test]
    fn an_empty_size_still_has_memory() {
        let mut surface = Surface::new(Size::new(0, 0)).unwrap();
        assert_eq!(surface.capacity(), Size::new(1, 1));
        assert_eq!(surface.canvas().size(), Size::new(0, 0));
        assert!(!surface.resize(Size::new(0, 0), Fit::Exact).unwrap());
    }

    #[test]
    fn the_canvas_draws_in_the_memory_that_gdi_copies() {
        let mut source = Surface::new(Size::new(64, 32)).unwrap();
        source.resize(Size::new(40, 20), Fit::Room).unwrap();
        source.canvas().fill(Rect::new(10, 5, 20, 15), GREEN);
        let mut target = Surface::new(Size::new(64, 32)).unwrap();
        // Only the part inside the size in use (40 × 20) is copied.
        source
            .paint(target.dib.dc, Rect::new(0, 0, 64, 32))
            .unwrap();
        let canvas = target.canvas();
        assert_eq!(canvas.get(10, 5), Some(pixel(GREEN)));
        assert_eq!(canvas.get(19, 14), Some(pixel(GREEN)));
        assert_eq!(canvas.get(20, 14), Some(0));
        assert_eq!(canvas.get(9, 5), Some(0));
    }

    #[test]
    fn colorref_stores_blue_green_red() {
        assert_eq!(colorref(Rgb::new(0x12, 0x34, 0x56)), 0x0056_3412);
    }
}
