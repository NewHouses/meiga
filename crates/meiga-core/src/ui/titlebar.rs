//! The bar of tabs as the title bar of the window (ADR 0027): what each
//! point of it is for Windows, which drags the window, resizes it from its
//! top edge, or presses its buttons, and how small the window can be.
//!
//! Windows asks a window what is at a point with `WM_NCHITTEST`, and does
//! what that part does by itself: the title bar drags the window and, with a
//! double click, maximizes or restores it; the top edge resizes it; and the
//! button to maximize opens the snap layouts of Windows 11 under the
//! pointer. meiga answers for its bar with a [`Hit`], and draws the bar and
//! its buttons itself.

use super::tabbar::{CaptionButton, Layout, Part};
use crate::geometry::{Point, Size};

/// What a point of the window is for Windows: the answer to
/// `WM_NCHITTEST` for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// A part of meiga: the tabs, their buttons, the terminal.
    Client,
    /// The title bar: it drags the window, and maximizes or restores it with
    /// a double click.
    Caption,
    /// The top edge of the window, which resizes it.
    Top,
    /// The top left corner of the window, which resizes it.
    TopLeft,
    Minimize,
    Maximize,
    Close,
}

/// The bar of tabs, as the title bar of the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleBar {
    layout: Layout,
    /// A menu of meiga is open: a click anywhere only closes it, so the bar
    /// does not drag the window.
    menu: bool,
}

impl TitleBar {
    /// The title bar of the bar of tabs `layout`, while a menu of meiga is
    /// open or not.
    pub const fn new(layout: Layout, menu: bool) -> Self {
        Self { layout, menu }
    }

    /// What the point `at` of the window is, `corner` pixels from its left
    /// side being the corner of its top edge, which resizes the window
    /// also to the left.
    ///
    /// The buttons of the window are always theirs, also over the band. The
    /// band above the tabs is the top edge, and the bar outside the tabs and
    /// their buttons, the title bar, unless a menu is open: then it is for
    /// meiga, which closes the menu. The rest of the window is for meiga.
    pub fn hit(&self, at: Point, corner: i32) -> Hit {
        match self.layout.part_at(at) {
            Some(Part::Caption(button)) => match button {
                CaptionButton::Minimize => Hit::Minimize,
                CaptionButton::Maximize => Hit::Maximize,
                CaptionButton::Close => Hit::Close,
            },
            Some(Part::Empty) if at.y < i32::try_from(self.layout.band()).unwrap_or(i32::MAX) => {
                if at.x < corner {
                    Hit::TopLeft
                } else {
                    Hit::Top
                }
            }
            Some(Part::Empty) if !self.menu => Hit::Caption,
            _ => Hit::Client,
        }
    }
}

/// The smallest size of the area of the window that meiga draws, at `dpi`,
/// with a band `band` pixels high above the tabs: the narrowest bar, which
/// still shows a whole tab and its room to drag the window, and only the
/// bar, without the terminal, as in Windows Terminal.
pub fn min_client(dpi: u32, band: u32) -> Size {
    Size::new(
        Layout::min_width(dpi),
        band.saturating_add(Layout::row_at(dpi)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three tabs in a window of 983 pixels, at 100 %, with a band `band`
    /// pixels high.
    fn bar(band: u32, menu: bool) -> TitleBar {
        TitleBar::new(Layout::new(96, 983, 3, band), menu)
    }

    #[test]
    fn maximized_the_bar_drags_the_window_and_its_top_row_belongs_to_what_is_below() {
        let bar = bar(0, false);
        let hit = |x, y| bar.hit(Point::new(x, y), 10);
        assert_eq!(hit(845, 0), Hit::Minimize);
        assert_eq!(hit(891, 31), Hit::Maximize);
        // The last pixel on the right, in the top row: Fitts.
        assert_eq!(hit(982, 0), Hit::Close);
        // The room to drag the window, the gaps and the bar beside the tabs.
        assert_eq!(hit(800, 10), Hit::Caption);
        assert_eq!(hit(844, 0), Hit::Caption);
        assert_eq!(hit(2, 20), Hit::Caption);
        assert_eq!(hit(5, 0), Hit::Client);
        // The tabs and their buttons, also in the top row.
        assert_eq!(hit(100, 0), Hit::Client);
        assert_eq!(hit(220, 10), Hit::Client);
        assert_eq!(hit(730, 0), Hit::Client);
        assert_eq!(hit(760, 20), Hit::Client);
        // The terminal.
        assert_eq!(hit(100, 32), Hit::Client);
        assert_eq!(hit(983, 10), Hit::Client);
    }

    #[test]
    fn restored_the_band_resizes_the_window_but_not_under_its_buttons() {
        let bar = bar(8, false);
        let hit = |x, y| bar.hit(Point::new(x, y), 10);
        assert_eq!(hit(0, 0), Hit::TopLeft);
        assert_eq!(hit(9, 7), Hit::TopLeft);
        assert_eq!(hit(10, 0), Hit::Top);
        assert_eq!(hit(100, 7), Hit::Top);
        assert_eq!(hit(800, 0), Hit::Top);
        assert_eq!(hit(845, 0), Hit::Minimize);
        assert_eq!(hit(982, 0), Hit::Close);
        assert_eq!(hit(100, 8), Hit::Client);
        assert_eq!(hit(800, 8), Hit::Caption);
        assert_eq!(hit(2, 8), Hit::Caption);
        assert_eq!(hit(100, 40), Hit::Client);
    }

    #[test]
    fn with_a_menu_open_the_bar_is_for_meiga_but_the_edge_and_the_buttons_are_not() {
        let bar = bar(8, true);
        let hit = |x, y| bar.hit(Point::new(x, y), 10);
        assert_eq!(hit(800, 10), Hit::Client);
        assert_eq!(hit(2, 20), Hit::Client);
        assert_eq!(hit(100, 7), Hit::Top);
        assert_eq!(hit(0, 0), Hit::TopLeft);
        assert_eq!(hit(960, 0), Hit::Close);
        assert_eq!(hit(100, 20), Hit::Client);
    }

    #[test]
    fn the_window_is_at_least_as_big_as_the_narrowest_bar() {
        assert_eq!(min_client(96, 8), Size::new(395, 40));
        assert_eq!(min_client(120, 0), Size::new(495, 40));
        assert_eq!(min_client(144, 11), Size::new(593, 59));
    }
}
