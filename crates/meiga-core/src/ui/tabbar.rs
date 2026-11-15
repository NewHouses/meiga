//! The layout of the tab bar: where each tab, its title and its close
//! button go, and the button that opens a tab, and what is under a point
//! (ADR 0020).
//!
//! The sizes are the ones of Windows Terminal (decided with the user), in
//! pixels at a scale of 100 %: a bar 32 high, as its row of tabs, and tabs up
//! to 240 wide, as `TabViewItemMaxWidth` of the TabView of WinUI that it
//! uses, all of the same width. When they do not fit, they all shrink alike,
//! without a minimum: the minimum, and moving along the bar, come in
//! subphase 2.8. The button that opens a tab goes right after the last one.
//!
//! For the mouse, a tab takes the whole height of the bar. It is drawn from
//! a gap below the top of the bar down to its bottom, so that the active
//! tab, which has the colour of the terminal, looks joined to it.

use crate::geometry::{Point, Rect};

/// The dots per inch at a scale of 100 %, where the sizes below are in
/// pixels.
const BASE_DPI: u32 = 96;
/// The height of the bar.
const HEIGHT: u32 = 32;
/// The widest tab.
const MAX_TAB: u32 = 240;
/// The space above the tabs, on the left of the first one, between the last
/// one and the button that opens a tab, and on the right of that button.
const GAP: u32 = 4;
/// The space between the left side of a tab and its title.
const PADDING: u32 = 8;
/// The close button: a square on the right of the tab, with a gap on its
/// right. The pointer highlights a smaller square in its middle, and the
/// cross goes in the middle of that one.
const CLOSE: u32 = 24;
const CLOSE_HIGHLIGHT: u32 = 20;
const CROSS: u32 = 8;
/// The button that opens a tab: a square as high as the tabs, with its
/// highlight and its plus in the middle. The plus has arms of the same
/// length: its side grows a pixel if it does not have the parity of the
/// stroke.
const NEW: u32 = 28;
const NEW_HIGHLIGHT: u32 = 24;
const PLUS: u32 = 10;
/// How long the separators between inactive tabs are, in the middle of
/// their height. They are one pixel wide at any scale.
const SEPARATOR: u32 = 16;
/// How thick the lines of the cross and of the plus are.
const STROKE: u32 = 1;

/// A part of the bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The tab at this index, but not its close button.
    Tab(usize),
    /// The close button of the tab at this index.
    Close(usize),
    /// The button that opens a tab.
    New,
    /// The bar outside the tabs and the button.
    Empty,
}

/// Where the parts of the bar go, for a number of tabs, a width and a
/// scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The sizes above, in pixels at the scale of the window.
    height: u32,
    max_tab: u32,
    gap: u32,
    padding: u32,
    close: u32,
    close_highlight: u32,
    cross: u32,
    new: u32,
    new_highlight: u32,
    plus: u32,
    separator: u32,
    stroke: u32,
    /// The width of the bar: the one of the window.
    width: u32,
    tabs: usize,
    /// The width of each tab.
    tab_width: u32,
}

impl Layout {
    /// The bar of `tabs` tabs in a window `width` pixels wide, at `dpi`.
    pub fn new(dpi: u32, width: u32, tabs: usize) -> Self {
        let size = |pixels| scale(pixels, dpi);
        let stroke = size(STROKE);
        let plus = size(PLUS);
        let mut layout = Self {
            height: size(HEIGHT),
            max_tab: size(MAX_TAB),
            gap: size(GAP),
            padding: size(PADDING),
            close: size(CLOSE),
            close_highlight: size(CLOSE_HIGHLIGHT),
            cross: size(CROSS),
            new: size(NEW),
            new_highlight: size(NEW_HIGHLIGHT),
            plus: plus + plus.abs_diff(stroke) % 2,
            separator: size(SEPARATOR),
            stroke,
            width,
            tabs,
            tab_width: 0,
        };
        layout.tab_width = layout.fit_tabs();
        layout
    }

    /// The height of the bar at `dpi`, without building a layout.
    pub fn height_at(dpi: u32) -> u32 {
        scale(HEIGHT, dpi)
    }

    /// The bar is now `width` pixels wide, with `tabs` tabs.
    pub fn resize(&mut self, width: u32, tabs: usize) {
        self.width = width;
        self.tabs = tabs;
        self.tab_width = self.fit_tabs();
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn tabs(&self) -> usize {
        self.tabs
    }

    pub const fn tab_width(&self) -> u32 {
        self.tab_width
    }

    /// How thick the lines of the cross and of the plus are.
    pub const fn stroke(&self) -> u32 {
        self.stroke
    }

    /// The whole bar.
    pub fn bar(&self) -> Rect {
        Rect::new(0, 0, coord(self.width.into()), coord(self.height.into()))
    }

    /// The tab at `index`, as it is drawn: from the gap below the top of
    /// the bar to its bottom.
    pub fn tab(&self, index: usize) -> Rect {
        let left = u64::from(self.gap)
            .saturating_add((index as u64).saturating_mul(u64::from(self.tab_width)));
        Rect::new(
            coord(left),
            coord(self.gap.into()),
            coord(left.saturating_add(u64::from(self.tab_width))),
            coord(self.height.into()),
        )
    }

    /// What changes when the tab at `index` changes how it looks: its whole
    /// column of the bar, and the separator on its right, which belongs to
    /// the next tab.
    pub fn tab_area(&self, index: usize) -> Rect {
        let tab = self.tab(index);
        Rect::new(tab.left, 0, tab.right.saturating_add(1), tab.bottom).intersection(self.bar())
    }

    /// The close button of the tab at `index`, or `None` if the tab is too
    /// narrow for it.
    pub fn close(&self, index: usize) -> Option<Rect> {
        let tab = self.tab(index);
        let gap = coord(self.gap.into());
        let right = tab.right - gap;
        let left = right - coord(self.close.into());
        if left < tab.left + gap {
            return None;
        }
        let top = tab.top + coord((self.tab_height().saturating_sub(self.close) / 2).into());
        Some(Rect::new(left, top, right, top + coord(self.close.into())))
    }

    /// The square of the close button that the pointer highlights.
    pub fn close_highlight(&self, index: usize) -> Option<Rect> {
        self.close(index)
            .map(|close| centered(close, self.close_highlight))
    }

    /// The square of the cross of the close button.
    pub fn cross(&self, index: usize) -> Option<Rect> {
        self.close(index).map(|close| centered(close, self.cross))
    }

    /// The room for the title of the tab at `index`: from the padding on its
    /// left to its close button, or to the padding on its right.
    pub fn title(&self, index: usize) -> Rect {
        let tab = self.tab(index);
        let left = tab.left + coord(self.padding.into());
        let right = self
            .close(index)
            .map_or(tab.right - coord(self.padding.into()), |close| close.left);
        Rect::new(left, tab.top, right.max(left), tab.bottom)
    }

    /// The separator on the left of the tab at `index`, between it and the
    /// previous one, or `None` for the first tab.
    pub fn separator(&self, index: usize) -> Option<Rect> {
        if index == 0 || index >= self.tabs {
            return None;
        }
        let tab = self.tab(index);
        let top = tab.top + coord((self.tab_height().saturating_sub(self.separator) / 2).into());
        Some(Rect::new(
            tab.left,
            top,
            tab.left + 1,
            top + coord(self.separator.into()),
        ))
    }

    /// The button that opens a tab, after the last one.
    pub fn new_tab(&self) -> Rect {
        let left = self.tab(self.tabs).left + coord(self.gap.into());
        let top = coord(self.gap.into());
        Rect::new(
            left,
            top,
            left + coord(self.new.into()),
            top + coord(self.new.into()),
        )
    }

    /// The square of that button that the pointer highlights.
    pub fn new_highlight(&self) -> Rect {
        centered(self.new_tab(), self.new_highlight)
    }

    /// The square of its plus.
    pub fn plus(&self) -> Rect {
        centered(self.new_tab(), self.plus)
    }

    /// The part of the bar at `at`, or `None` outside the bar. Tabs and the
    /// button that opens one take the whole height of the bar.
    pub fn part_at(&self, at: Point) -> Option<Part> {
        if !self.bar().contains_point(at) {
            return None;
        }
        let new = self.new_tab();
        if (new.left..new.right).contains(&at.x) {
            return Some(Part::New);
        }
        let gap = coord(self.gap.into());
        if self.tab_width == 0 || at.x < gap {
            return Some(Part::Empty);
        }
        // `at.x - gap` is not negative.
        let index = ((at.x - gap) as u64 / u64::from(self.tab_width)) as usize;
        if index >= self.tabs {
            return Some(Part::Empty);
        }
        let on_close = self
            .close(index)
            .is_some_and(|close| close.contains_point(at));
        Some(if on_close {
            Part::Close(index)
        } else {
            Part::Tab(index)
        })
    }

    /// The height of the tabs as they are drawn.
    const fn tab_height(&self) -> u32 {
        self.height.saturating_sub(self.gap)
    }

    /// The width of each tab: the widest one if they fit, and otherwise all
    /// alike, as wide as they fit.
    fn fit_tabs(&self) -> u32 {
        let room = self
            .width
            .saturating_sub(3 * self.gap)
            .saturating_sub(self.new);
        let tabs = u32::try_from(self.tabs.max(1)).unwrap_or(u32::MAX);
        (room / tabs).min(self.max_tab)
    }
}

/// `pixels` at a scale of 100 %, at `dpi`: rounded, and at least one.
fn scale(pixels: u32, dpi: u32) -> u32 {
    (pixels.saturating_mul(dpi).saturating_add(BASE_DPI / 2) / BASE_DPI).max(1)
}

/// A coordinate, which a bar never reaches the end of.
fn coord(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// The square of `side` pixels in the middle of `rect`.
fn centered(rect: Rect, side: u32) -> Rect {
    let side = coord(side.into());
    let left = rect.left + (coord(rect.width().into()) - side) / 2;
    let top = rect.top + (coord(rect.height().into()) - side) / 2;
    Rect::new(left, top, left + side, top + side)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three tabs in a window of 800 pixels, at 100 %.
    fn three() -> Layout {
        Layout::new(96, 800, 3)
    }

    #[test]
    fn tabs_are_as_wide_as_in_windows_terminal_and_the_button_follows_them() {
        let layout = three();
        assert_eq!(layout.height(), 32);
        assert_eq!(Layout::height_at(96), 32);
        assert_eq!(layout.bar(), Rect::new(0, 0, 800, 32));
        assert_eq!(layout.tab_width(), 240);
        assert_eq!(layout.tab(0), Rect::new(4, 4, 244, 32));
        assert_eq!(layout.tab(2), Rect::new(484, 4, 724, 32));
        assert_eq!(layout.new_tab(), Rect::new(728, 4, 756, 32));
        assert_eq!(layout.new_highlight(), Rect::new(730, 6, 754, 30));
        // 11 pixels, so that both arms of a line of 1 have 5.
        assert_eq!(layout.plus(), Rect::new(736, 12, 747, 23));
    }

    #[test]
    fn a_tab_has_its_title_and_then_its_close_button() {
        let layout = three();
        assert_eq!(layout.close(0), Some(Rect::new(216, 6, 240, 30)));
        assert_eq!(layout.close_highlight(0), Some(Rect::new(218, 8, 238, 28)));
        assert_eq!(layout.cross(0), Some(Rect::new(224, 14, 232, 22)));
        assert_eq!(layout.title(0), Rect::new(12, 4, 216, 32));
        // The separators go between tabs, in the first column of the second.
        assert_eq!(layout.separator(0), None);
        assert_eq!(layout.separator(1), Some(Rect::new(244, 10, 245, 26)));
        assert_eq!(layout.separator(3), None);
        assert_eq!(layout.tab_area(0), Rect::new(4, 0, 245, 32));
    }

    #[test]
    fn tabs_that_do_not_fit_shrink_alike_and_narrow_ones_have_no_close_button() {
        let mut layout = three();
        // 800 - 3 gaps of 4 - the button of 28 = 760.
        layout.resize(800, 10);
        assert_eq!(layout.tab_width(), 76);
        assert_eq!(layout.new_tab(), Rect::new(768, 4, 796, 32));
        layout.resize(800, 30);
        assert_eq!(layout.tab_width(), 25);
        assert_eq!(layout.close(0), None);
        assert_eq!(layout.title(0), Rect::new(12, 4, 21, 32));
        // A window narrower than the button: the tabs have no room.
        layout.resize(20, 2);
        assert_eq!(layout.tab_width(), 0);
        assert_eq!(layout.title(1), Rect::new(12, 4, 12, 32));
    }

    #[test]
    fn the_sizes_follow_the_scale() {
        let layout = Layout::new(144, 1920, 2);
        assert_eq!(layout.height(), 48);
        assert_eq!(layout.tab_width(), 360);
        assert_eq!(layout.tab(1), Rect::new(366, 6, 726, 48));
        assert_eq!(layout.close(1), Some(Rect::new(684, 9, 720, 45)));
        assert_eq!(layout.new_tab(), Rect::new(732, 6, 774, 48));
        assert_eq!(layout.stroke(), 2);
        // 15 pixels, and one more for arms of 7 on both sides of a line of
        // 2.
        assert_eq!(layout.plus().width(), 16);
        assert_eq!(Layout::new(120, 800, 1).plus().width(), 13);
        assert_eq!(Layout::height_at(120), 40);
    }

    #[test]
    fn the_part_under_the_pointer() {
        let layout = three();
        let part = |x, y| layout.part_at(Point::new(x, y));
        assert_eq!(part(4, 20), Some(Part::Tab(0)));
        // Above the drawn tab, in the gap: still the tab.
        assert_eq!(part(100, 1), Some(Part::Tab(0)));
        assert_eq!(part(220, 10), Some(Part::Close(0)));
        // Beside the close button, above and below it.
        assert_eq!(part(220, 5), Some(Part::Tab(0)));
        assert_eq!(part(220, 30), Some(Part::Tab(0)));
        assert_eq!(part(243, 20), Some(Part::Tab(0)));
        assert_eq!(part(244, 20), Some(Part::Tab(1)));
        assert_eq!(part(723, 20), Some(Part::Tab(2)));
        assert_eq!(part(725, 20), Some(Part::Empty));
        assert_eq!(part(728, 0), Some(Part::New));
        assert_eq!(part(755, 31), Some(Part::New));
        assert_eq!(part(756, 20), Some(Part::Empty));
        assert_eq!(part(2, 20), Some(Part::Empty));
        assert_eq!(part(100, 32), None);
        assert_eq!(part(-1, 10), None);
        assert_eq!(part(800, 10), None);
    }
}
