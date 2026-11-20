//! The layout of the tab bar: where each tab, its title and its close
//! button go, the buttons that open a tab and the menu of the profiles, the
//! ones that scroll the tabs, and the buttons of the window, and what is
//! under a point (ADR 0020, 0022, 0025 and 0027).
//!
//! The sizes are the ones of Windows Terminal (decided with the user), in
//! pixels at a scale of 100 %: a row of tabs 32 high, and tabs from 100 to
//! 240 wide, as `TabViewItemMinWidth` and `TabViewItemMaxWidth` of the
//! TabView of WinUI that it uses, all of the same width. When they do not
//! fit, they all shrink alike, down to the narrowest. The button that opens
//! a tab goes right after the last one and, joined to it, the one that opens
//! the menu of the profiles, as the split button of Windows Terminal.
//!
//! When even the narrowest tabs do not fit, the bar scrolls (ADR 0025): the
//! tabs go in a strip between two buttons that scroll it, and the buttons
//! that open a tab stay on the right of the strip. The strip shows the part
//! of the row of tabs that its scroll says, and cuts the tabs on its sides.
//!
//! The bar is the title bar of the window (ADR 0027). On its right go the
//! buttons of the window, minimize, maximize or restore, and close, 46 wide,
//! as in Windows 11 and in Windows Terminal, and as high as the whole bar.
//! Between them and the buttons that open a tab there is always room to drag
//! the window, at least 45 pixels, as the drag bar of Windows Terminal, also
//! while the tabs scroll. While the window is not maximized, a band goes
//! above the row of tabs, where the pointer resizes the window, as high as
//! the border that Windows gives a window to resize it, which comes in
//! pixels at the scale of the window; maximized, the bar has no band.
//!
//! For the mouse, a tab takes the whole height of the row of tabs. It is
//! drawn from a gap below the top of the row down to its bottom, so that
//! the active tab, which has the colour of the terminal, looks joined to it.

use std::mem;

use super::{coord, scale};
use crate::geometry::{Point, Rect};

/// The height of the row of tabs: the whole bar while the window is
/// maximized, and the bar without the band above the tabs otherwise.
const ROW: u32 = 32;
/// The widest tab.
const MAX_TAB: u32 = 240;
/// The narrowest tab: when they do not fit even so, the bar scrolls. Its
/// close button always fits.
const MIN_TAB: u32 = 100;
/// The space above the tabs, on the left of the first one, between the last
/// one and the button that opens a tab, and on the right of the button of
/// the profiles, which is joined to that one.
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
/// The button that opens the menu of the profiles: as high as the one that
/// opens a tab, and narrower, with a highlight as high as the one of the
/// plus. Its chevron, in the middle, is twice as wide as it is high, so
/// that its two lines meet at any scale.
const PROFILES: u32 = 24;
const PROFILES_HIGHLIGHT: u32 = 20;
const CHEVRON: u32 = 4;
/// The buttons that scroll the tabs: as the one of the profiles, as high as
/// the one that opens a tab, with a highlight of the same size, and a
/// chevron in the middle, twice as high as it is wide, that points to their
/// side.
const SCROLL: u32 = 24;
const SCROLL_HIGHLIGHT: u32 = 20;
/// The buttons of the window, and the square of their glyphs in their
/// middle, as the 10 by 10 of the glyphs of Segoe Fluent Icons that Windows
/// 11 and Windows Terminal use. The glyph of restore is two squares: the one
/// behind, `GLYPH_OFFSET` pixels above and on the right of the one in front.
const CAPTION: u32 = 46;
const GLYPH: u32 = 10;
const GLYPH_OFFSET: u32 = 2;
/// The narrowest room to drag the window, between the buttons that open a
/// tab and the buttons of the window.
const DRAG: u32 = 45;
/// How long the separators between inactive tabs are, in the middle of
/// their height. They are one pixel wide at any scale.
const SEPARATOR: u32 = 16;
/// How thick the lines of the cross, of the plus and of the glyphs of the
/// buttons of the window are.
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
    /// The button that opens the menu of the profiles.
    Profiles,
    /// The button that scrolls the tabs towards this side.
    Scroll(Side),
    /// A button of the window.
    Caption(CaptionButton),
    /// The bar outside the tabs and the buttons: the band above the tabs,
    /// where the window is resized, and the rest, where it is dragged.
    Empty,
}

/// A button of the window, on the right of the bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptionButton {
    Minimize,
    /// Maximizes the window or, if it is maximized, restores it.
    Maximize,
    Close,
}

impl CaptionButton {
    /// The three, from left to right.
    pub const ALL: [Self; 3] = [Self::Minimize, Self::Maximize, Self::Close];

    /// How many buttons of the window are on its right.
    const fn on_the_right(self) -> u32 {
        match self {
            Self::Minimize => 2,
            Self::Maximize => 1,
            Self::Close => 0,
        }
    }
}

/// A side of the strip of tabs, towards which a button scrolls it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Towards the first tab.
    Left,
    /// Towards the last tab.
    Right,
}

/// Where the parts of the bar go, for a number of tabs, a width, a scale,
/// and the band above the tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The sizes above, in pixels at the scale of the window.
    row: u32,
    max_tab: u32,
    min_tab: u32,
    gap: u32,
    padding: u32,
    close: u32,
    close_highlight: u32,
    cross: u32,
    new: u32,
    new_highlight: u32,
    plus: u32,
    profiles: u32,
    profiles_highlight: u32,
    /// The short side of the chevrons: the one of the profiles is twice as
    /// wide as it is high, and the ones that scroll, twice as high as they
    /// are wide.
    chevron: u32,
    scroll_button: u32,
    scroll_highlight: u32,
    caption: u32,
    glyph: u32,
    glyph_offset: u32,
    drag: u32,
    separator: u32,
    stroke: u32,
    /// The height of the band above the row of tabs, where the pointer
    /// resizes the window: none while it is maximized.
    band: u32,
    /// The width of the bar: the one of the window.
    width: u32,
    tabs: usize,
    /// The width of each tab.
    tab_width: u32,
    /// The narrowest tabs do not fit: they go in the strip between the
    /// buttons that scroll it.
    overflow: bool,
    /// While they do not fit, how many pixels of the row of tabs are hidden
    /// on the left of the strip.
    scroll: u32,
}

impl Layout {
    /// The bar of `tabs` tabs in a window `width` pixels wide, at `dpi`,
    /// with a band `band` pixels high above the row of tabs.
    pub fn new(dpi: u32, width: u32, tabs: usize, band: u32) -> Self {
        let size = |pixels| scale(pixels, dpi);
        let stroke = size(STROKE);
        let plus = size(PLUS);
        let mut layout = Self {
            row: size(ROW),
            max_tab: size(MAX_TAB),
            min_tab: size(MIN_TAB),
            gap: size(GAP),
            padding: size(PADDING),
            close: size(CLOSE),
            close_highlight: size(CLOSE_HIGHLIGHT),
            cross: size(CROSS),
            new: size(NEW),
            new_highlight: size(NEW_HIGHLIGHT),
            plus: plus + plus.abs_diff(stroke) % 2,
            profiles: size(PROFILES),
            profiles_highlight: size(PROFILES_HIGHLIGHT),
            chevron: size(CHEVRON),
            scroll_button: size(SCROLL),
            scroll_highlight: size(SCROLL_HIGHLIGHT),
            caption: size(CAPTION),
            glyph: size(GLYPH),
            glyph_offset: size(GLYPH_OFFSET),
            drag: size(DRAG),
            separator: size(SEPARATOR),
            stroke,
            band,
            width,
            tabs,
            tab_width: 0,
            overflow: false,
            scroll: 0,
        };
        layout.fit();
        layout
    }

    /// The height of the row of tabs at `dpi`, without building a layout.
    pub fn row_at(dpi: u32) -> u32 {
        scale(ROW, dpi)
    }

    /// The width of the narrowest bar at `dpi`: its buttons, the room to
    /// drag the window, and a strip that shows a whole tab of the narrowest
    /// between the buttons that scroll it.
    pub fn min_width(dpi: u32) -> u32 {
        let layout = Self::new(dpi, 0, 0, 0);
        layout
            .fixed()
            .saturating_add(layout.scroll_button.saturating_mul(2))
            .saturating_add(layout.min_tab)
    }

    /// The band above the row of tabs is `band` pixels high now: none while
    /// the window is maximized.
    pub const fn set_band(&mut self, band: u32) {
        self.band = band;
    }

    /// The bar is now `width` pixels wide, with `tabs` tabs: the tabs get
    /// their width again, and the strip keeps its scroll, as far as it can.
    pub fn resize(&mut self, width: u32, tabs: usize) {
        self.width = width;
        self.tabs = tabs;
        self.fit();
    }

    /// The bar has `tabs` tabs now, fewer, because some closed while the
    /// pointer was on it: the others keep their width, and the strip its
    /// scroll, so that the tab on the right of the one that closed goes
    /// where that one was, and its close button under the pointer (ADR
    /// 0025). What they leave on the right stays empty until the bar is
    /// resized.
    pub fn keep_widths(&mut self, tabs: usize) {
        self.tabs = tabs;
    }

    /// The height of the whole bar: the band and the row of tabs.
    pub const fn height(&self) -> u32 {
        self.band.saturating_add(self.row)
    }

    /// The height of the band above the row of tabs.
    pub const fn band(&self) -> u32 {
        self.band
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

    /// Whether the narrowest tabs do not fit, and the bar scrolls them.
    pub const fn overflows(&self) -> bool {
        self.overflow
    }

    /// How many pixels of the row of tabs the strip hides on its left.
    pub const fn scroll(&self) -> u32 {
        self.scroll
    }

    /// How far the strip can scroll: the part of the row of tabs that does
    /// not fit in it.
    pub fn max_scroll(&self) -> u32 {
        if !self.overflow {
            return 0;
        }
        let row = (self.tabs as u64).saturating_mul(self.tab_width.into());
        u32::try_from(row.saturating_sub(self.viewport().into())).unwrap_or(u32::MAX)
    }

    /// Whether the strip can scroll towards `side`.
    pub fn can_scroll(&self, side: Side) -> bool {
        match side {
            Side::Left => self.scroll > 0,
            Side::Right => self.scroll < self.max_scroll(),
        }
    }

    /// How thick the lines of the cross, of the plus and of the glyphs of
    /// the buttons of the window are.
    pub const fn stroke(&self) -> u32 {
        self.stroke
    }

    /// The whole bar, from the top of the band.
    pub fn bar(&self) -> Rect {
        Rect::new(0, 0, coord(self.width.into()), coord(self.height().into()))
    }

    /// The part of the bar where the tabs are: the whole bar or, while they
    /// do not fit, the strip between the buttons that scroll it, as high as
    /// the bar. What is drawn of the tabs is cut to it.
    pub fn strip(&self) -> Rect {
        if !self.overflow {
            return self.bar();
        }
        let left = u64::from(self.gap) + u64::from(self.scroll_button);
        Rect::new(
            coord(left),
            0,
            coord(left + u64::from(self.viewport())),
            coord(self.height().into()),
        )
    }

    /// The tab at `index`, as it is drawn: from the gap below the top of
    /// the row of tabs to the bottom of the bar. While the tabs scroll, it
    /// may be out of the strip, in part or all of it.
    pub fn tab(&self, index: usize) -> Rect {
        let left = self.start().saturating_add(signed(
            (index as u64).saturating_mul(u64::from(self.tab_width)),
        ));
        Rect::new(
            clamped(left),
            coord(self.top().into()),
            clamped(left.saturating_add(self.tab_width.into())),
            coord(self.height().into()),
        )
    }

    /// What changes when the tab at `index` changes how it looks: its whole
    /// column of the row of tabs, and the separator on its right, which
    /// belongs to the next tab, as far as they are in the strip.
    pub fn tab_area(&self, index: usize) -> Rect {
        let tab = self.tab(index);
        let band = coord(self.band.into());
        Rect::new(tab.left, band, tab.right.saturating_add(1), tab.bottom)
            .intersection(self.strip())
    }

    /// The close button of the tab at `index`, or `None` if the tab is too
    /// narrow for it, which the narrowest one is not.
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

    /// The button that opens a tab: after the last one or, while the tabs do
    /// not fit, after the button that scrolls them to the right.
    pub fn new_tab(&self) -> Rect {
        let gap = coord(self.gap.into());
        let left = if self.overflow {
            self.strip().right + coord(self.scroll_button.into()) + gap
        } else {
            self.tab(self.tabs).left + gap
        };
        let top = coord(self.top().into());
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

    /// The button that opens the menu of the profiles, joined to the one
    /// that opens a tab.
    pub fn profiles(&self) -> Rect {
        let new = self.new_tab();
        Rect::new(
            new.right,
            new.top,
            new.right + coord(self.profiles.into()),
            new.bottom,
        )
    }

    /// The part of that button that the pointer highlights.
    pub fn profiles_highlight(&self) -> Rect {
        centered_size(self.profiles(), self.profiles_highlight, self.new_highlight)
    }

    /// The rectangle of its chevron, which points down.
    pub fn chevron(&self) -> Rect {
        centered_size(self.profiles(), 2 * self.chevron, self.chevron)
    }

    /// The button that scrolls the tabs towards `side`, on that side of the
    /// strip, as high as the one that opens a tab, or `None` while they fit.
    pub fn scroll_button(&self, side: Side) -> Option<Rect> {
        if !self.overflow {
            return None;
        }
        let strip = self.strip();
        let width = coord(self.scroll_button.into());
        let left = match side {
            Side::Left => strip.left - width,
            Side::Right => strip.right,
        };
        let top = coord(self.top().into());
        Some(Rect::new(
            left,
            top,
            left + width,
            top + coord(self.new.into()),
        ))
    }

    /// The part of that button that the pointer highlights.
    pub fn scroll_highlight(&self, side: Side) -> Option<Rect> {
        self.scroll_button(side)
            .map(|button| centered_size(button, self.scroll_highlight, self.new_highlight))
    }

    /// The rectangle of its chevron, which points to its side.
    pub fn scroll_chevron(&self, side: Side) -> Option<Rect> {
        self.scroll_button(side)
            .map(|button| centered_size(button, self.chevron, 2 * self.chevron))
    }

    /// The button `button` of the window, on the right of the bar, as high
    /// as the whole bar, also over the band.
    pub fn caption(&self, button: CaptionButton) -> Rect {
        let width = i64::from(self.caption);
        let right = i64::from(self.width) - i64::from(button.on_the_right()) * width;
        Rect::new(
            clamped(right - width),
            0,
            clamped(right),
            coord(self.height().into()),
        )
    }

    /// The three buttons of the window together.
    pub fn captions(&self) -> Rect {
        self.caption(CaptionButton::Minimize)
            .union(self.caption(CaptionButton::Close))
    }

    /// The square of the glyph of the button `button` of the window, in its
    /// middle.
    pub fn glyph(&self, button: CaptionButton) -> Rect {
        centered(self.caption(button), self.glyph)
    }

    /// How far the square behind of the glyph of restore is from the one in
    /// front: as many pixels up as to the right.
    pub const fn glyph_offset(&self) -> u32 {
        self.glyph_offset
    }

    /// The part of the bar at `at`, or `None` outside the bar. The buttons
    /// of the window take the whole height of the bar, and the tabs and the
    /// other buttons, the whole height of the row of tabs, below the band.
    /// Only the part of a tab that is in the strip is under the pointer.
    pub fn part_at(&self, at: Point) -> Option<Part> {
        if !self.bar().contains_point(at) {
            return None;
        }
        let across = |rect: Rect| (rect.left..rect.right).contains(&at.x);
        for button in CaptionButton::ALL {
            if across(self.caption(button)) {
                return Some(Part::Caption(button));
            }
        }
        if at.y < coord(self.band.into()) {
            return Some(Part::Empty);
        }
        if across(self.new_tab()) {
            return Some(Part::New);
        }
        if across(self.profiles()) {
            return Some(Part::Profiles);
        }
        for side in [Side::Left, Side::Right] {
            if self.scroll_button(side).is_some_and(across) {
                return Some(Part::Scroll(side));
            }
        }
        let from_start = i64::from(at.x) - self.start();
        if self.tab_width == 0 || !across(self.strip()) || from_start < 0 {
            return Some(Part::Empty);
        }
        // `from_start` is not negative.
        let index =
            usize::try_from(from_start as u64 / u64::from(self.tab_width)).unwrap_or(usize::MAX);
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

    /// Scrolls the strip `pixels` towards the last tab or, if they are
    /// negative, towards the first one, as far as it can. Returns whether
    /// it scrolled.
    pub fn scroll_by(&mut self, pixels: i64) -> bool {
        let target = i64::from(self.scroll).saturating_add(pixels).max(0);
        self.scroll_to(target.unsigned_abs())
    }

    /// Scrolls the strip a tab towards `side`, as the buttons do: until the
    /// first tab that it hides on that side, even in part, is all in it,
    /// as in Firefox. Returns whether it scrolled.
    pub fn step(&mut self, side: Side) -> bool {
        if self.tab_width == 0 {
            return false;
        }
        let width = u64::from(self.tab_width);
        let scroll = u64::from(self.scroll);
        let target = match side {
            // The first tab that is all in the strip, and the one before.
            Side::Left => scroll.div_ceil(width).saturating_sub(1) * width,
            Side::Right => {
                let viewport = u64::from(self.viewport());
                // The first tab that is not all in the strip.
                let hidden = (scroll + viewport) / width;
                ((hidden + 1) * width).saturating_sub(viewport)
            }
        };
        self.scroll_to(target)
    }

    /// Scrolls the strip as little as it can so that the tab at `index` is
    /// all in it, or its left side, if the strip is narrower than a tab:
    /// the one that is shown is always in view. Returns whether it
    /// scrolled.
    pub fn reveal(&mut self, index: usize) -> bool {
        if !self.overflow || index >= self.tabs {
            return false;
        }
        let width = u64::from(self.tab_width);
        let left = (index as u64).saturating_mul(width);
        let right = left.saturating_add(width);
        let scroll = u64::from(self.scroll);
        let viewport = u64::from(self.viewport());
        let target = if left < scroll {
            left
        } else if right > scroll + viewport {
            right.saturating_sub(viewport).min(left)
        } else {
            return false;
        };
        self.scroll_to(target)
    }

    /// Scrolls the strip to `target`, or as far as it can. Returns whether
    /// it scrolled.
    fn scroll_to(&mut self, target: u64) -> bool {
        let scroll = u32::try_from(target.min(self.max_scroll().into())).unwrap_or(0);
        mem::replace(&mut self.scroll, scroll) != scroll
    }

    /// Where the first tab is, or would be: after the gap on the left of
    /// the bar or, while the tabs do not fit, at the left of the strip,
    /// minus its scroll.
    fn start(&self) -> i64 {
        if self.overflow {
            i64::from(self.strip().left) - i64::from(self.scroll)
        } else {
            i64::from(self.gap)
        }
    }

    /// The top of the tabs, as they are drawn, and of the buttons that open
    /// a tab or scroll the strip: the gap below the band.
    const fn top(&self) -> u32 {
        self.band.saturating_add(self.gap)
    }

    /// The width of the bar that is not for the tabs: the gaps on the left
    /// of the first tab, after the last one and after the button of the
    /// profiles, the buttons that open a tab, the narrowest room to drag the
    /// window, and the buttons of the window.
    const fn fixed(&self) -> u32 {
        self.gap
            .saturating_mul(3)
            .saturating_add(self.new)
            .saturating_add(self.profiles)
            .saturating_add(self.drag)
            .saturating_add(self.caption.saturating_mul(3))
    }

    /// The width of the bar without what is not for the tabs: where the tabs
    /// go while they fit.
    const fn room(&self) -> u32 {
        self.width.saturating_sub(self.fixed())
    }

    /// The width of the strip while the tabs do not fit: the room without
    /// the buttons that scroll it.
    const fn viewport(&self) -> u32 {
        self.room().saturating_sub(2 * self.scroll_button)
    }

    /// The height of the tabs as they are drawn: the row without the gap
    /// above them.
    const fn tab_height(&self) -> u32 {
        self.row.saturating_sub(self.gap)
    }

    /// Gives the tabs their width: the widest one if they fit, and otherwise
    /// all alike, as wide as they fit, down to the narrowest one; below it,
    /// they keep it, and the strip scrolls them. The scroll stays, as far as
    /// the strip can scroll.
    fn fit(&mut self) {
        let tabs = u32::try_from(self.tabs.max(1)).unwrap_or(u32::MAX);
        let width = (self.room() / tabs).min(self.max_tab);
        self.overflow = width < self.min_tab;
        self.tab_width = if self.overflow { self.min_tab } else { width };
        self.scroll = self.scroll.min(self.max_scroll());
    }
}

/// A distance of the row of tabs as a signed number of pixels.
fn signed(pixels: u64) -> i64 {
    i64::try_from(pixels).unwrap_or(i64::MAX)
}

/// A position of the row of tabs as a coordinate of the window: a tab
/// hidden far away from the strip stays far away.
fn clamped(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX })
}

/// The square of `side` pixels in the middle of `rect`.
fn centered(rect: Rect, side: u32) -> Rect {
    centered_size(rect, side, side)
}

/// The rectangle of `width` by `height` pixels in the middle of `rect`.
fn centered_size(rect: Rect, width: u32, height: u32) -> Rect {
    let (width, height) = (coord(width.into()), coord(height.into()));
    let left = rect.left + (coord(rect.width().into()) - width) / 2;
    let top = rect.top + (coord(rect.height().into()) - height) / 2;
    Rect::new(left, top, left + width, top + height)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three tabs in a window of 983 pixels, at 100 %, maximized: the 800
    /// pixels of the bar of 2.8 and the 183 of the room to drag the window
    /// and its buttons.
    fn three() -> Layout {
        Layout::new(96, 983, 3, 0)
    }

    #[test]
    fn tabs_are_as_wide_as_in_windows_terminal_and_the_buttons_follow_them() {
        let layout = three();
        assert_eq!(layout.height(), 32);
        assert_eq!(layout.band(), 0);
        assert_eq!(Layout::row_at(96), 32);
        assert_eq!(layout.bar(), Rect::new(0, 0, 983, 32));
        assert_eq!(layout.tab_width(), 240);
        assert_eq!(layout.tab(0), Rect::new(4, 4, 244, 32));
        assert_eq!(layout.tab(2), Rect::new(484, 4, 724, 32));
        assert_eq!(layout.new_tab(), Rect::new(728, 4, 756, 32));
        assert_eq!(layout.new_highlight(), Rect::new(730, 6, 754, 30));
        // 11 pixels, so that both arms of a line of 1 have 5.
        assert_eq!(layout.plus(), Rect::new(736, 12, 747, 23));
        // The button of the profiles, joined to it.
        assert_eq!(layout.profiles(), Rect::new(756, 4, 780, 32));
        assert_eq!(layout.profiles_highlight(), Rect::new(758, 6, 778, 30));
        assert_eq!(layout.chevron(), Rect::new(764, 16, 772, 20));
    }

    #[test]
    fn the_buttons_of_the_window_are_on_its_right_as_high_as_the_bar() {
        let layout = three();
        let caption = |button| layout.caption(button);
        assert_eq!(caption(CaptionButton::Minimize), Rect::new(845, 0, 891, 32));
        assert_eq!(caption(CaptionButton::Maximize), Rect::new(891, 0, 937, 32));
        assert_eq!(caption(CaptionButton::Close), Rect::new(937, 0, 983, 32));
        assert_eq!(layout.captions(), Rect::new(845, 0, 983, 32));
        // The glyphs, 10 by 10 in their middle, with lines of 1.
        assert_eq!(
            layout.glyph(CaptionButton::Close),
            Rect::new(955, 11, 965, 21)
        );
        assert_eq!(
            layout.glyph(CaptionButton::Minimize),
            Rect::new(863, 11, 873, 21)
        );
        assert_eq!(layout.glyph_offset(), 2);
        assert_eq!(layout.stroke(), 1);
        // Between the button of the profiles and the buttons of the window,
        // the gap of 4 after the first ones, and 845 - 784 = 61 pixels to
        // drag the window.
        assert_eq!(
            caption(CaptionButton::Minimize).left - layout.profiles().right,
            65
        );
        // At 150 %: 69 wide and 48 high, with glyphs of 15 and lines of 2.
        let layout = Layout::new(144, 2195, 2, 0);
        assert_eq!(
            layout.caption(CaptionButton::Close),
            Rect::new(2126, 0, 2195, 48)
        );
        assert_eq!(
            layout.caption(CaptionButton::Minimize),
            Rect::new(1988, 0, 2057, 48)
        );
        assert_eq!(
            layout.glyph(CaptionButton::Close),
            Rect::new(2153, 16, 2168, 31)
        );
        assert_eq!(layout.glyph_offset(), 3);
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
    fn a_band_above_the_tabs_moves_them_down_and_the_buttons_of_the_window_cover_it() {
        let mut layout = Layout::new(96, 983, 3, 8);
        assert_eq!(layout.band(), 8);
        assert_eq!(layout.height(), 40);
        assert_eq!(layout.bar(), Rect::new(0, 0, 983, 40));
        assert_eq!(layout.tab_width(), 240);
        assert_eq!(layout.tab(0), Rect::new(4, 12, 244, 40));
        assert_eq!(layout.new_tab(), Rect::new(728, 12, 756, 40));
        assert_eq!(layout.profiles(), Rect::new(756, 12, 780, 40));
        assert_eq!(layout.close(0), Some(Rect::new(216, 14, 240, 38)));
        assert_eq!(layout.separator(1), Some(Rect::new(244, 18, 245, 34)));
        // A tab does not change the band.
        assert_eq!(layout.tab_area(0), Rect::new(4, 8, 245, 40));
        assert_eq!(
            layout.caption(CaptionButton::Close),
            Rect::new(937, 0, 983, 40)
        );
        assert_eq!(
            layout.glyph(CaptionButton::Close),
            Rect::new(955, 15, 965, 25)
        );
        let part = |x, y| layout.part_at(Point::new(x, y));
        assert_eq!(part(100, 7), Some(Part::Empty));
        assert_eq!(part(100, 8), Some(Part::Tab(0)));
        assert_eq!(part(730, 7), Some(Part::Empty));
        assert_eq!(part(730, 8), Some(Part::New));
        assert_eq!(part(960, 0), Some(Part::Caption(CaptionButton::Close)));
        assert_eq!(part(845, 7), Some(Part::Caption(CaptionButton::Minimize)));
        assert_eq!(part(100, 40), None);
        // The buttons that scroll the strip go below the band too, and the
        // strip covers it.
        let crowded = Layout::new(96, 983, 8, 8);
        assert_eq!(
            crowded.scroll_button(Side::Left),
            Some(Rect::new(4, 12, 28, 40))
        );
        assert_eq!(crowded.strip(), Rect::new(28, 0, 716, 40));
        assert_eq!(crowded.tab_area(6), Rect::new(628, 8, 716, 40));
        // Maximized, without the band, it is the bar of a maximized window.
        layout.set_band(0);
        assert_eq!(layout, three());
    }

    #[test]
    fn there_is_always_room_to_drag_the_window_between_the_buttons() {
        for dpi in [96, 120, 144, 168, 192] {
            let narrowest = Layout::min_width(dpi);
            let widths = (narrowest..narrowest + 600)
                .step_by(7)
                .chain([1920, 2560, 3840, 7680]);
            for width in widths {
                for tabs in 1..=40 {
                    let layout = Layout::new(dpi, width, tabs, 0);
                    let room = layout.caption(CaptionButton::Minimize).left
                        - layout.profiles().right
                        - coord(layout.gap.into());
                    assert!(
                        room >= coord(scale(DRAG, dpi).into()),
                        "{room} pixels to drag at {dpi} DPI, {width} wide, with {tabs} tabs"
                    );
                }
            }
        }
        // Seven tabs that fit leave a pixel more, and eight that scroll, the
        // narrowest room, all of it.
        let room = |tabs| {
            let layout = Layout::new(96, 983, tabs, 0);
            layout.caption(CaptionButton::Minimize).left - layout.profiles().right - 4
        };
        assert_eq!(room(7), 46);
        assert_eq!(room(8), 45);
    }

    #[test]
    fn the_narrowest_bar_shows_a_whole_tab_between_the_buttons_that_scroll() {
        // 3 gaps of 4, the buttons of 28 and 24, the room to drag of 45,
        // the buttons of the window, 3 × 46, the buttons that scroll, 2 ×
        // 24, and the narrowest tab, 100.
        assert_eq!(Layout::min_width(96), 395);
        assert_eq!(Layout::min_width(120), 495);
        assert_eq!(Layout::min_width(144), 593);
        let layout = Layout::new(96, 395, 2, 0);
        assert!(layout.overflows());
        assert_eq!(layout.strip().width(), 100);
        assert!(Layout::new(96, 394, 2, 0).strip().width() < 100);
        // One tab fits, wider than the narrowest.
        let layout = Layout::new(96, 395, 1, 0);
        assert!(!layout.overflows());
        assert_eq!(layout.tab_width(), 148);
    }

    #[test]
    fn tabs_that_do_not_fit_shrink_alike_down_to_100_and_keep_their_close_button() {
        let mut layout = three();
        // 983 - 3 gaps of 4 - the buttons of 28 and 24 - the room to drag
        // of 45 - the buttons of the window, 3 × 46 = 736.
        layout.resize(983, 7);
        assert_eq!(layout.tab_width(), 105);
        assert!(!layout.overflows());
        assert_eq!(layout.strip(), layout.bar());
        assert_eq!(layout.scroll_button(Side::Left), None);
        assert_eq!(layout.scroll_button(Side::Right), None);
        assert_eq!(layout.max_scroll(), 0);
        assert!(!layout.can_scroll(Side::Right));
        assert_eq!(layout.new_tab(), Rect::new(743, 4, 771, 32));
        assert_eq!(layout.profiles(), Rect::new(771, 4, 795, 32));
        assert_eq!(layout.close(6), Some(Rect::new(711, 6, 735, 30)));
        assert_eq!(layout.title(6), Rect::new(642, 4, 711, 32));
        // Down to 100 pixels: seven of them fit in 947 pixels, and not in
        // 946.
        layout.resize(947, 7);
        assert_eq!(layout.tab_width(), 100);
        assert!(!layout.overflows());
        layout.resize(946, 7);
        assert!(layout.overflows());
    }

    #[test]
    fn the_narrowest_tabs_that_do_not_fit_go_in_a_strip_between_two_buttons() {
        let mut layout = three();
        layout.resize(983, 8);
        assert!(layout.overflows());
        assert_eq!(layout.tab_width(), 100);
        // 736 - the two buttons of 24 = 688.
        assert_eq!(layout.strip(), Rect::new(28, 0, 716, 32));
        let (left, right) = (Side::Left, Side::Right);
        assert_eq!(layout.scroll_button(left), Some(Rect::new(4, 4, 28, 32)));
        assert_eq!(
            layout.scroll_button(right),
            Some(Rect::new(716, 4, 740, 32))
        );
        assert_eq!(layout.scroll_highlight(left), Some(Rect::new(6, 6, 26, 30)));
        // 4 pixels wide and 8 high, in the middle.
        assert_eq!(layout.scroll_chevron(left), Some(Rect::new(14, 14, 18, 22)));
        assert_eq!(
            layout.scroll_chevron(right),
            Some(Rect::new(726, 14, 730, 22))
        );
        // The buttons that open a tab stay on the right of the strip.
        assert_eq!(layout.new_tab(), Rect::new(744, 4, 772, 32));
        assert_eq!(layout.profiles(), Rect::new(772, 4, 796, 32));
        assert_eq!(layout.tab(0), Rect::new(28, 4, 128, 32));
        assert_eq!(layout.tab(7), Rect::new(728, 4, 828, 32));
        // Their columns, as far as they are in the strip.
        assert_eq!(layout.tab_area(6), Rect::new(628, 0, 716, 32));
        assert!(layout.tab_area(7).is_empty());
        // 800 - 688.
        assert_eq!(layout.max_scroll(), 112);
        assert!(!layout.can_scroll(left));
        assert!(layout.can_scroll(right));
        // A window narrower than the buttons: the strip has no room.
        layout.resize(20, 2);
        assert!(layout.strip().is_empty());
        assert_eq!(layout.max_scroll(), 200);
    }

    #[test]
    fn the_strip_scrolls_by_tabs_by_pixels_and_to_show_a_tab() {
        let mut layout = Layout::new(96, 983, 8, 0);
        // Up to the seventh tab, which it cut.
        assert!(layout.step(Side::Right));
        assert_eq!(layout.scroll(), 12);
        assert_eq!(layout.tab(6), Rect::new(616, 4, 716, 32));
        assert!(layout.step(Side::Right));
        assert_eq!(layout.scroll(), 112);
        assert!(!layout.can_scroll(Side::Right));
        assert!(!layout.step(Side::Right));
        // The second tab, which it cut, and then the first one.
        assert!(layout.step(Side::Left));
        assert_eq!(layout.scroll(), 100);
        assert!(layout.step(Side::Left));
        assert_eq!(layout.scroll(), 0);
        assert!(!layout.step(Side::Left));
        // By pixels, as far as it can.
        assert!(layout.scroll_by(50));
        assert_eq!(layout.tab(0), Rect::new(-22, 4, 78, 32));
        assert!(layout.scroll_by(-80));
        assert_eq!(layout.scroll(), 0);
        assert!(!layout.scroll_by(-1));
        assert!(layout.scroll_by(1000));
        assert_eq!(layout.scroll(), 112);
        // To show a tab all, as little as it can.
        assert!(layout.reveal(0));
        assert_eq!(layout.scroll(), 0);
        assert!(layout.reveal(6));
        assert_eq!(layout.scroll(), 12);
        assert!(!layout.reveal(5));
        assert!(layout.reveal(7));
        assert_eq!(layout.scroll(), 112);
        // A narrower window keeps the scroll, and a wider one, as far as it
        // can: 1033 - 247 - 48 = 738, of the 800 of the tabs.
        layout.resize(883, 8);
        assert_eq!(layout.scroll(), 112);
        layout.resize(1033, 8);
        assert_eq!(layout.scroll(), 62);
        // Once they fit, it does not scroll: (1183 - 247) / 8 = 117.
        layout.resize(1183, 8);
        assert!(!layout.overflows());
        assert_eq!(layout.scroll(), 0);
        assert!(!layout.reveal(7));
        // A strip narrower than a tab shows its left side: 333 - 247 - 48 =
        // 38.
        layout.resize(333, 8);
        assert!(layout.reveal(1));
        assert_eq!(layout.scroll(), 100);
    }

    #[test]
    fn tabs_keep_their_width_and_the_strip_its_scroll_when_some_close_in_place() {
        // While they fit, the + follows the last tab.
        let mut layout = Layout::new(96, 983, 5, 0);
        assert_eq!(layout.tab_width(), 147);
        layout.keep_widths(4);
        assert_eq!(layout.tab_width(), 147);
        assert_eq!(layout.new_tab(), Rect::new(596, 4, 624, 32));
        layout.resize(983, 4);
        assert_eq!(layout.tab_width(), 184);
        // Scrolled to the end, the place of the last tab stays empty.
        let mut layout = Layout::new(96, 983, 10, 0);
        assert!(layout.scroll_by(1000));
        assert_eq!(layout.scroll(), 312);
        layout.keep_widths(9);
        assert!(layout.overflows());
        assert_eq!(layout.scroll(), 312);
        assert!(!layout.can_scroll(Side::Right));
        assert_eq!(layout.part_at(Point::new(715, 31)), Some(Part::Empty));
        assert_eq!(layout.part_at(Point::new(615, 31)), Some(Part::Tab(8)));
        layout.resize(983, 9);
        assert_eq!(layout.scroll(), 212);
    }

    #[test]
    fn the_sizes_follow_the_scale() {
        let layout = Layout::new(144, 2195, 2, 0);
        assert_eq!(layout.height(), 48);
        assert_eq!(layout.tab_width(), 360);
        assert_eq!(layout.tab(1), Rect::new(366, 6, 726, 48));
        assert_eq!(layout.close(1), Some(Rect::new(684, 9, 720, 45)));
        assert_eq!(layout.new_tab(), Rect::new(732, 6, 774, 48));
        assert_eq!(layout.profiles(), Rect::new(774, 6, 810, 48));
        assert_eq!(layout.profiles_highlight(), Rect::new(777, 9, 807, 45));
        assert_eq!(layout.chevron(), Rect::new(786, 24, 798, 30));
        assert_eq!(layout.stroke(), 2);
        // 15 pixels, and one more for arms of 7 on both sides of a line of
        // 2.
        assert_eq!(layout.plus().width(), 16);
        assert_eq!(Layout::new(120, 983, 1, 0).plus().width(), 13);
        // 5 pixels high, and twice as wide, as at every scale.
        assert_eq!(
            Layout::new(120, 983, 1, 0).chevron().size(),
            crate::geometry::Size::new(10, 5)
        );
        assert_eq!(Layout::row_at(120), 40);
        // The narrowest tab, 150 pixels, and the buttons that scroll, 36:
        // 2195 - 3 gaps of 6 - 42 - 36 - the room to drag, 68, - the
        // buttons of the window, 3 × 69 = 1824, for 12 tabs, but not 13.
        assert_eq!(Layout::new(144, 2195, 12, 0).tab_width(), 152);
        let crowded = Layout::new(144, 2195, 13, 0);
        assert!(crowded.overflows());
        assert_eq!(crowded.tab_width(), 150);
        assert_eq!(
            crowded.scroll_button(Side::Left),
            Some(Rect::new(6, 6, 42, 48))
        );
        // 1824 - 2 × 36.
        assert_eq!(crowded.strip(), Rect::new(42, 0, 1794, 48));
    }

    #[test]
    fn the_part_under_the_pointer_while_the_tabs_scroll() {
        let mut layout = Layout::new(96, 983, 8, 0);
        assert!(layout.scroll_by(50));
        let part = |x, y| layout.part_at(Point::new(x, y));
        assert_eq!(part(3, 20), Some(Part::Empty));
        assert_eq!(part(4, 20), Some(Part::Scroll(Side::Left)));
        assert_eq!(part(27, 0), Some(Part::Scroll(Side::Left)));
        // The first tab goes from 28 - 50 = -22, and only its part in the
        // strip is under the pointer.
        assert_eq!(part(28, 31), Some(Part::Tab(0)));
        assert_eq!(part(77, 31), Some(Part::Tab(0)));
        assert_eq!(part(78, 31), Some(Part::Tab(1)));
        // Its close button, from 78 - 4 - 24 = 50.
        assert_eq!(part(50, 20), Some(Part::Close(0)));
        assert_eq!(part(715, 31), Some(Part::Tab(7)));
        assert_eq!(part(716, 20), Some(Part::Scroll(Side::Right)));
        assert_eq!(part(739, 31), Some(Part::Scroll(Side::Right)));
        assert_eq!(part(740, 20), Some(Part::Empty));
        assert_eq!(part(744, 20), Some(Part::New));
        assert_eq!(part(795, 20), Some(Part::Profiles));
        assert_eq!(part(796, 20), Some(Part::Empty));
        // The room to drag the window, and then its buttons.
        assert_eq!(part(844, 31), Some(Part::Empty));
        assert_eq!(part(845, 20), Some(Part::Caption(CaptionButton::Minimize)));
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
        assert_eq!(part(756, 20), Some(Part::Profiles));
        assert_eq!(part(779, 0), Some(Part::Profiles));
        assert_eq!(part(780, 20), Some(Part::Empty));
        assert_eq!(part(844, 0), Some(Part::Empty));
        let minimize = Some(Part::Caption(CaptionButton::Minimize));
        assert_eq!(part(845, 0), minimize);
        assert_eq!(part(890, 31), minimize);
        let maximize = Some(Part::Caption(CaptionButton::Maximize));
        assert_eq!(part(891, 20), maximize);
        assert_eq!(part(936, 20), maximize);
        let close = Some(Part::Caption(CaptionButton::Close));
        assert_eq!(part(937, 20), close);
        assert_eq!(part(982, 31), close);
        assert_eq!(part(2, 20), Some(Part::Empty));
        assert_eq!(part(100, 32), None);
        assert_eq!(part(-1, 10), None);
        assert_eq!(part(983, 10), None);
    }
}
