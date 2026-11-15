//! A menu over the window: a list of items, which separators can group, the
//! item that the pointer or the keyboard highlights, and where each part goes
//! (ADR 0021). The first one was the menu of a tab, and the second one, the
//! menu of the profiles (ADR 0022); the explorer (subphase 5.2) comes next.
//!
//! The sizes are the ones of the menus of WinUI (`MenuFlyout`), which the
//! menu of a tab is in Windows Terminal, as they are with the mouse and the
//! keyboard, in pixels at a scale of 100 %: a border of 1, and 2 more above
//! the first entry and below the last one; items 28 high, with 4 on their
//! sides and 2 above and below them that their highlight does not reach,
//! and their text 11 from the sides of the highlight; and separators, a line
//! of 1 across the menu, with 1 above and below it. The menu is as wide as
//! its longest item, and never wider than the window.
//!
//! A menu opens with its top-left corner where the pointer is, as the menus
//! of Windows, or below a part of the window, from its left side, such as
//! the tab whose menu the keyboard opens. If it does not fit on that side, it
//! goes on the other one, and it does not leave the window: one higher than
//! the window is cut at the bottom, and the pointer highlights and chooses
//! nothing in what is cut.
//!
//! Only the items that can be chosen are highlighted: the keyboard skips the
//! separators and the disabled items, and goes round from the last item to
//! the first one, and the pointer highlights nothing over them.

use std::borrow::Cow;

use super::{coord, scale};
use crate::geometry::{Point, Rect, Size};
use crate::text;

/// The border, and the space between it and the first and the last entries.
const BORDER: u32 = 1;
const PADDING: u32 = 2;
/// The highlight of an item, the space on its sides and above and below it,
/// and the one between it and the text.
const ITEM: u32 = 28;
const ITEM_SIDE: u32 = 4;
const ITEM_GAP: u32 = 2;
const TEXT_PADDING: u32 = 11;
/// The line of a separator, and the space above and below it.
const LINE: u32 = 1;
const LINE_GAP: u32 = 1;

/// An entry of a menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry<A> {
    /// An item, which asks for `action` when it is chosen, if it is
    /// `enabled`, with its text in bold if it is the default choice.
    Item {
        label: Cow<'static, str>,
        action: A,
        enabled: bool,
        bold: bool,
    },
    /// A line between two groups of items.
    Separator,
}

impl<A> Entry<A> {
    /// An item that can be chosen if `enabled`.
    pub fn item(label: impl Into<Cow<'static, str>>, action: A, enabled: bool) -> Self {
        Self::Item {
            label: label.into(),
            action,
            enabled,
            bold: false,
        }
    }

    /// The same entry, with its text in bold if it is an item: the default
    /// choice, as the profile that the button + opens.
    #[must_use]
    pub fn bold(self) -> Self {
        match self {
            Self::Item {
                label,
                action,
                enabled,
                ..
            } => Self::Item {
                label,
                action,
                enabled,
                bold: true,
            },
            Self::Separator => Self::Separator,
        }
    }

    /// Its text, if it is an item.
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Item { label, .. } => Some(label),
            Self::Separator => None,
        }
    }

    /// Whether it is an item that can be chosen.
    pub const fn is_enabled(&self) -> bool {
        matches!(self, Self::Item { enabled: true, .. })
    }

    /// Whether it is an item with its text in bold.
    pub const fn is_bold(&self) -> bool {
        matches!(self, Self::Item { bold: true, .. })
    }
}

/// Where the keyboard moves the highlight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// The next item that can be chosen or, after the last one, the first.
    Next,
    /// The previous one or, before the first one, the last.
    Previous,
    First,
    Last,
}

/// A menu: its entries, and the item that is highlighted, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu<A> {
    entries: Vec<Entry<A>>,
    /// Always an item that can be chosen.
    highlighted: Option<usize>,
}

impl<A: Copy> Menu<A> {
    /// A menu of `entries`, with nothing highlighted, as when the mouse
    /// opens it.
    pub const fn new(entries: Vec<Entry<A>>) -> Self {
        Self {
            entries,
            highlighted: None,
        }
    }

    pub fn entries(&self) -> &[Entry<A>] {
        &self.entries
    }

    /// The entry that is highlighted.
    pub const fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    /// Highlights the entry at `index`, if it is an item that can be chosen,
    /// and otherwise nothing: the pointer is over it, or over no item.
    /// Returns whether the highlight changed.
    pub fn hover(&mut self, index: Option<usize>) -> bool {
        let index = index.filter(|&index| self.can_choose(index));
        self.highlight(index)
    }

    /// Moves the highlight as the keyboard asks. Without a highlight, the
    /// next item is the first one, and the previous one, the last. Returns
    /// whether the highlight changed: not if no item can be chosen.
    pub fn step(&mut self, step: Step) -> bool {
        let count = self.entries.len();
        let choosable = |index: &usize| self.can_choose(*index);
        let target = match (step, self.highlighted) {
            (Step::First, _) | (Step::Next, None) => (0..count).find(choosable),
            (Step::Last, _) | (Step::Previous, None) => (0..count).rev().find(choosable),
            (Step::Next, Some(current)) => (current + 1..count).chain(0..=current).find(choosable),
            (Step::Previous, Some(current)) => (0..current)
                .rev()
                .chain((current..count).rev())
                .find(choosable),
        };
        target.is_some() && self.highlight(target)
    }

    /// The action of the item at `index`, if it can be chosen.
    pub fn action(&self, index: usize) -> Option<A> {
        match self.entries.get(index) {
            Some(Entry::Item {
                action,
                enabled: true,
                ..
            }) => Some(*action),
            _ => None,
        }
    }

    /// The action of the item that is highlighted, if any.
    pub fn chosen(&self) -> Option<A> {
        self.highlighted.and_then(|index| self.action(index))
    }

    fn can_choose(&self, index: usize) -> bool {
        self.entries.get(index).is_some_and(Entry::is_enabled)
    }

    fn highlight(&mut self, index: Option<usize>) -> bool {
        let changed = self.highlighted != index;
        self.highlighted = index;
        changed
    }
}

/// Where a menu opens, in the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    /// With its top-left corner at this point: where the pointer was.
    At(Point),
    /// Below this part of the window, from its left side.
    Below(Rect),
}

/// Where each part of a menu goes, in the window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The sizes above, in pixels at the scale of the window.
    border: u32,
    item_side: u32,
    item_gap: u32,
    text_padding: u32,
    line: u32,
    line_gap: u32,
    /// The whole menu.
    menu: Rect,
    /// The client area of the window, which cuts the menu if it is higher.
    bounds: Rect,
    /// The room of each entry: across the menu, inside its border.
    entries: Vec<Rect>,
}

impl Layout {
    /// The menu of `entries` at `dpi`, whose text has cells of `cell`
    /// pixels, where `anchor` says, inside `bounds`: the client area of the
    /// window.
    pub fn new<A>(
        entries: &[Entry<A>],
        dpi: u32,
        cell: Size,
        anchor: Anchor,
        bounds: Rect,
    ) -> Self {
        let size = |pixels| scale(pixels, dpi);
        let (border, padding, item_side, item_gap) =
            (size(BORDER), size(PADDING), size(ITEM_SIDE), size(ITEM_GAP));
        let (text_padding, line, line_gap) = (size(TEXT_PADDING), size(LINE), size(LINE_GAP));
        let item = size(ITEM) + 2 * item_gap;
        let separator = line + 2 * line_gap;

        let columns = entries
            .iter()
            .filter_map(Entry::label)
            .map(text::width)
            .max()
            .unwrap_or(0);
        let text = u64::from(cell.width).saturating_mul(columns as u64);
        let chrome = 2 * u64::from(border + item_side + text_padding);
        let width = u32::try_from(text.saturating_add(chrome))
            .unwrap_or(u32::MAX)
            .min(bounds.width());
        let height = entries
            .iter()
            .fold(2 * (border + padding), |height, entry| {
                height.saturating_add(match entry {
                    Entry::Item { .. } => item,
                    Entry::Separator => separator,
                })
            });
        let menu = Rect::at(
            place(anchor, Size::new(width, height), bounds),
            Size::new(width, height),
        );

        let (left, right) = (
            menu.left + coord(border.into()),
            menu.right - coord(border.into()),
        );
        let mut top = menu.top + coord((border + padding).into());
        let entries = entries
            .iter()
            .map(|entry| {
                let height = coord(
                    match entry {
                        Entry::Item { .. } => item,
                        Entry::Separator => separator,
                    }
                    .into(),
                );
                let room = Rect::new(left, top, right, top.saturating_add(height));
                top = room.bottom;
                room
            })
            .collect();
        Self {
            border,
            item_side,
            item_gap,
            text_padding,
            line,
            line_gap,
            menu,
            bounds,
            entries,
        }
    }

    /// The whole menu.
    pub const fn menu(&self) -> Rect {
        self.menu
    }

    /// How thick its border is.
    pub const fn border(&self) -> u32 {
        self.border
    }

    /// The room of the entry at `index`: across the menu, inside its border.
    /// The pointer is on an item anywhere in its room.
    pub fn entry(&self, index: usize) -> Rect {
        self.entries.get(index).copied().unwrap_or(Rect::EMPTY)
    }

    /// The highlight of the item at `index`.
    pub fn highlight(&self, index: usize) -> Rect {
        inset(self.entry(index), self.item_side, self.item_gap)
    }

    /// The room for the text of the item at `index`.
    pub fn text(&self, index: usize) -> Rect {
        inset(self.highlight(index), self.text_padding, 0)
    }

    /// The line of the separator at `index`.
    pub fn line(&self, index: usize) -> Rect {
        let room = self.entry(index);
        let top = room.top + coord(self.line_gap.into());
        Rect::new(room.left, top, room.right, top + coord(self.line.into()))
    }

    /// The entry at `at`, if any: not on the border, nor above the first
    /// entry or below the last one, nor out of the window, where a menu
    /// higher than it is cut.
    pub fn entry_at(&self, at: Point) -> Option<usize> {
        if !self.bounds.contains_point(at) {
            return None;
        }
        self.entries.iter().position(|room| room.contains_point(at))
    }
}

/// The top-left corner of a menu of `size` where `anchor` says, inside
/// `bounds`: on the other side if it does not fit on that one, and, if it
/// does not fit on either, as far as it can without leaving them.
fn place(anchor: Anchor, size: Size, bounds: Rect) -> Point {
    let (width, height) = (coord(size.width.into()), coord(size.height.into()));
    let (x, y, other_x, other_y) = match anchor {
        Anchor::At(at) => (
            at.x,
            at.y,
            at.x.saturating_sub(width),
            at.y.saturating_sub(height),
        ),
        Anchor::Below(part) => (
            part.left,
            part.bottom,
            part.right.saturating_sub(width),
            part.top.saturating_sub(height),
        ),
    };
    let x = if x.saturating_add(width) > bounds.right {
        other_x
    } else {
        x
    };
    let y = if y.saturating_add(height) > bounds.bottom {
        other_y
    } else {
        y
    };
    Point::new(
        x.min(bounds.right.saturating_sub(width)).max(bounds.left),
        y.min(bounds.bottom.saturating_sub(height)).max(bounds.top),
    )
}

/// `rect` without `x` pixels on its sides, and `y` above and below.
fn inset(rect: Rect, x: u32, y: u32) -> Rect {
    let (x, y) = (coord(x.into()), coord(y.into()));
    let inner = Rect::new(rect.left + x, rect.top + y, rect.right - x, rect.bottom - y);
    if inner.is_empty() { Rect::EMPTY } else { inner }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The entries of the menu of a tab, with "Close other tabs" disabled:
    /// a disabled item between two that can be chosen.
    fn entries() -> Vec<Entry<u8>> {
        vec![
            Entry::item("Duplicate tab", 1, true),
            Entry::Separator,
            Entry::item("Close tab", 2, true),
            Entry::item("Close other tabs", 3, false),
            Entry::item("Close tabs to the right", 4, true),
        ]
    }

    /// Cells of 7 x 16 pixels, at 100 %, in a window of 800 x 600.
    fn layout(anchor: Anchor) -> Layout {
        Layout::new(
            &entries(),
            96,
            Size::new(7, 16),
            anchor,
            Rect::new(0, 0, 800, 600),
        )
    }

    #[test]
    fn the_menu_is_as_wide_as_its_longest_item_and_as_high_as_its_entries() {
        let layout = layout(Anchor::At(Point::new(100, 50)));
        // 23 cells of 7, 11 and 4 on each side, and the border: 193. Four
        // items of 2 + 28 + 2, a separator of 1 + 1 + 1, and 1 + 2 above
        // and below: 137.
        assert_eq!(layout.menu(), Rect::new(100, 50, 293, 187));
        assert_eq!(layout.border(), 1);
        assert_eq!(layout.entry(0), Rect::new(101, 53, 292, 85));
        assert_eq!(layout.entry(1), Rect::new(101, 85, 292, 88));
        assert_eq!(layout.entry(2), Rect::new(101, 88, 292, 120));
        assert_eq!(layout.entry(4), Rect::new(101, 152, 292, 184));
        assert_eq!(layout.entry(5), Rect::EMPTY);
        assert_eq!(layout.highlight(0), Rect::new(105, 55, 288, 83));
        assert_eq!(layout.text(0), Rect::new(116, 55, 277, 83));
        assert_eq!(layout.line(1), Rect::new(101, 86, 292, 87));
    }

    #[test]
    fn the_pointer_is_on_an_item_anywhere_in_its_room() {
        let layout = layout(Anchor::At(Point::new(100, 50)));
        let at = |x, y| layout.entry_at(Point::new(x, y));
        assert_eq!(at(101, 53), Some(0));
        // Beside and below its highlight.
        assert_eq!(at(102, 84), Some(0));
        assert_eq!(at(200, 86), Some(1));
        assert_eq!(at(291, 183), Some(4));
        // The border, and the space above the first entry.
        assert_eq!(at(100, 60), None);
        assert_eq!(at(200, 52), None);
        assert_eq!(at(200, 185), None);
        assert_eq!(at(99, 60), None);
    }

    #[test]
    fn a_menu_that_does_not_fit_goes_to_the_other_side_and_stays_in_the_window() {
        // On the right and at the bottom: its right and bottom sides at the
        // pointer.
        assert_eq!(
            layout(Anchor::At(Point::new(700, 500))).menu(),
            Rect::new(507, 363, 700, 500)
        );
        // Not on either side: inside the window, from its top-left corner.
        let bounds = Rect::new(0, 0, 250, 150);
        let narrow = Layout::new(
            &entries(),
            96,
            Size::new(7, 16),
            Anchor::At(Point::new(120, 100)),
            bounds,
        );
        assert_eq!(narrow.menu(), Rect::new(0, 0, 193, 137));
        // A window narrower than the menu: as wide as it.
        let bounds = Rect::new(0, 0, 150, 600);
        let cut = Layout::new(
            &entries(),
            96,
            Size::new(7, 16),
            Anchor::At(Point::new(40, 10)),
            bounds,
        );
        assert_eq!(cut.menu(), Rect::new(0, 10, 150, 147));
        assert_eq!(cut.text(0), Rect::new(16, 15, 134, 43));
        // One higher than the window: from its top, cut at the bottom.
        let bounds = Rect::new(0, 0, 800, 100);
        let short = Layout::new(
            &entries(),
            96,
            Size::new(7, 16),
            Anchor::At(Point::new(40, 10)),
            bounds,
        );
        assert_eq!(short.menu(), Rect::new(40, 0, 233, 137));
        // Only what the window shows takes the pointer: Close other tabs,
        // which it cuts, above its bottom side, and not Close tabs to the
        // right, below it.
        assert_eq!(short.entry_at(Point::new(100, 99)), Some(3));
        assert_eq!(short.entry_at(Point::new(100, 100)), None);
        assert_eq!(short.entry_at(Point::new(100, 120)), None);
    }

    #[test]
    fn below_a_part_of_the_window_from_its_left_side() {
        let tab = Rect::new(244, 4, 484, 32);
        assert_eq!(
            layout(Anchor::Below(tab)).menu(),
            Rect::new(244, 32, 437, 169)
        );
        // Near the right side of the window: up to the right side of the
        // part.
        let tab = Rect::new(700, 4, 790, 32);
        assert_eq!(
            layout(Anchor::Below(tab)).menu(),
            Rect::new(597, 32, 790, 169)
        );
    }

    #[test]
    fn the_sizes_follow_the_scale() {
        // Cells of 11 x 25, at 150 %: 23 x 11 + 2 x (17 + 6) + 2 x 2 wide,
        // and 2 x (2 + 3) + 4 x (42 + 2 x 3) + (2 + 2 x 2) high.
        let layout = Layout::new(
            &entries(),
            144,
            Size::new(11, 25),
            Anchor::At(Point::new(0, 0)),
            Rect::new(0, 0, 1920, 1080),
        );
        assert_eq!(layout.menu(), Rect::new(0, 0, 303, 208));
        assert_eq!(layout.border(), 2);
        assert_eq!(layout.entry(0), Rect::new(2, 5, 301, 53));
        assert_eq!(layout.highlight(0), Rect::new(8, 8, 295, 50));
        assert_eq!(layout.line(1), Rect::new(2, 55, 301, 57));
    }

    #[test]
    fn the_keyboard_skips_what_cannot_be_chosen_and_goes_round() {
        let mut menu = Menu::new(entries());
        assert_eq!(menu.highlighted(), None);
        assert!(menu.step(Step::Next));
        assert_eq!(menu.highlighted(), Some(0));
        // Over the separator.
        assert!(menu.step(Step::Next));
        assert_eq!(menu.highlighted(), Some(2));
        // Over the disabled item.
        assert!(menu.step(Step::Next));
        assert_eq!(menu.highlighted(), Some(4));
        assert!(menu.step(Step::Next));
        assert_eq!(menu.highlighted(), Some(0));
        assert!(menu.step(Step::Previous));
        assert_eq!(menu.highlighted(), Some(4));
        assert!(menu.step(Step::Previous));
        assert_eq!(menu.highlighted(), Some(2));
        assert!(menu.step(Step::First));
        assert_eq!(menu.highlighted(), Some(0));
        assert!(!menu.step(Step::First));
        assert!(menu.step(Step::Last));
        assert_eq!(menu.chosen(), Some(4));
        // Without a highlight, the previous item is the last one.
        menu.hover(None);
        assert!(menu.step(Step::Previous));
        assert_eq!(menu.highlighted(), Some(4));
    }

    #[test]
    fn the_pointer_only_highlights_what_can_be_chosen() {
        let mut menu = Menu::new(entries());
        assert!(menu.hover(Some(2)));
        assert_eq!(menu.chosen(), Some(2));
        assert!(!menu.hover(Some(2)));
        // The separator, the disabled item, and outside the entries.
        assert!(menu.hover(Some(1)));
        assert_eq!(menu.highlighted(), None);
        assert!(!menu.hover(Some(3)));
        assert!(!menu.hover(Some(9)));
        assert_eq!(menu.chosen(), None);
        assert_eq!(menu.action(3), None);
        assert_eq!(menu.action(4), Some(4));
    }

    #[test]
    fn a_menu_without_items_to_choose_highlights_nothing() {
        let mut menu = Menu::new(vec![
            Entry::item("Close other tabs", 3, false),
            Entry::Separator,
        ]);
        assert!(!menu.step(Step::Next));
        assert!(!menu.step(Step::Last));
        assert_eq!(menu.highlighted(), None);
        assert_eq!(menu.entries()[0].label(), Some("Close other tabs"));
        assert_eq!(menu.entries()[1].label(), None);
    }
}
