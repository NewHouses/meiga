//! A menu over the window, as the user sees it and uses it (ADR 0021): its
//! frame and its entries, drawn over the tab bar and the terminal; the
//! highlight, which follows the pointer and the arrows; and the item that a
//! click, Enter or Space chooses. What goes where, and which items can be
//! highlighted, is `meiga_core::ui::menu`.
//!
//! While it is open, it gets all the mouse and the keyboard. A button that
//! goes down outside it closes it, and does nothing else; Esc closes it too.
//! An item is chosen when the left or the right button goes up on it, if it
//! went down in the menu, or with Enter or Space while it is highlighted;
//! the menu closes then.
//!
//! The menu only draws what changed: all of it when it opens and when it
//! closes, and the items whose highlight changes.

use std::mem;

use meiga_core::color::Rgb;
use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::input::KeyEvent;
use meiga_core::mouse::Button;
use meiga_core::text::{self, CellChar};
use meiga_core::ui::menu::{Anchor, Entry, Layout, Menu, Step};
use meiga_render::{Canvas, Font, FontStyle};

/// The colours, until they can be configured (subphase 8.3), next to the
/// ones of the tab bar.
const BACKGROUND: Rgb = Rgb::new(0x1c, 0x1f, 0x25);
const BORDER: Rgb = Rgb::new(0x3a, 0x3f, 0x49);
const HIGHLIGHT: Rgb = Rgb::new(0x2b, 0x2f, 0x37);
const LINE: Rgb = Rgb::new(0x2b, 0x2f, 0x37);
const TEXT: Rgb = Rgb::new(0xd4, 0xd7, 0xdd);
const DISABLED: Rgb = Rgb::new(0x5c, 0x62, 0x6d);

/// The virtual-key codes of the keys of a menu.
const VK_RETURN: u16 = 0x0d;
const VK_ESCAPE: u16 = 0x1b;
const VK_SPACE: u16 = 0x20;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_UP: u16 = 0x26;
const VK_DOWN: u16 = 0x28;

/// An open menu.
struct Shown<A> {
    menu: Menu<A>,
    layout: Layout,
    /// The button that went down in the menu and did not go up yet: the
    /// window captures the mouse meanwhile.
    pressed: Option<Button>,
    /// Whether the pointer moved in the window since the menu opened, or
    /// since it left the window.
    pointer: bool,
}

/// A menu of the window, open or closed, whose items ask for `A`.
pub struct MenuView<A> {
    shown: Option<Shown<A>>,
    /// The areas that changed since the last frame.
    damage: Vec<Rect>,
    /// The cells of a label, kept to avoid an allocation for each one.
    cells: Vec<CellChar>,
}

impl<A: Copy> MenuView<A> {
    pub const fn new() -> Self {
        Self {
            shown: None,
            damage: Vec::new(),
            cells: Vec::new(),
        }
    }

    pub const fn is_open(&self) -> bool {
        self.shown.is_some()
    }

    /// Whether a button went down in the menu and did not go up yet.
    pub fn is_pressed(&self) -> bool {
        self.shown
            .as_ref()
            .is_some_and(|shown| shown.pressed.is_some())
    }

    /// Opens a menu of `entries`, instead of the one that may be open, at
    /// `dpi`, with text in cells of `cell` pixels, where `anchor` says,
    /// inside `bounds`. The keyboard, which opens it with `keyboard`,
    /// highlights its first item.
    pub fn open(
        &mut self,
        entries: Vec<Entry<A>>,
        dpi: u32,
        cell: Size,
        anchor: Anchor,
        bounds: Rect,
        keyboard: bool,
    ) {
        self.close();
        let layout = Layout::new(&entries, dpi, cell, anchor, bounds);
        let mut menu = Menu::new(entries);
        if keyboard {
            menu.step(Step::First);
        }
        self.damage(layout.menu());
        self.shown = Some(Shown {
            menu,
            layout,
            pressed: None,
            pointer: false,
        });
    }

    /// Closes the menu, if it is open: the next frame draws what it covered.
    /// Returns whether it was open.
    pub fn close(&mut self) -> bool {
        match self.shown.take() {
            Some(shown) => {
                self.damage(shown.layout.menu());
                true
            }
            None => false,
        }
    }

    /// Whether the menu covers all of `area`.
    pub fn covers(&self, area: Rect) -> bool {
        self.shown
            .as_ref()
            .is_some_and(|shown| shown.layout.menu().contains(area))
    }

    /// The pointer moved to `at`, in the window: it highlights the item
    /// under it, if it can be chosen. Returns whether it is its first move
    /// since the menu opened, or since it left the window, whose leave the
    /// window has to follow then.
    pub fn moved(&mut self, at: Point) -> bool {
        let Some(shown) = &mut self.shown else {
            return false;
        };
        let entered = !mem::replace(&mut shown.pointer, true);
        let index = shown.layout.entry_at(at);
        self.hover(index);
        entered
    }

    /// The pointer left the window.
    pub fn left(&mut self) {
        if let Some(shown) = &mut self.shown {
            shown.pointer = false;
        }
        self.hover(None);
    }

    /// `button` went down at `at`: outside the menu, it closes it, and in
    /// it, the left or the right button presses it.
    pub fn down(&mut self, button: Button, at: Point) {
        let Some(shown) = &mut self.shown else {
            return;
        };
        if !shown.layout.menu().contains_point(at) {
            self.close();
        } else if shown.pressed.is_none() && button != Button::Middle {
            shown.pressed = Some(button);
        }
    }

    /// `button` went up at `at`. Returns the action of the item under it,
    /// if the button went down in the menu and the item can be chosen: the
    /// menu closes then.
    pub fn up(&mut self, button: Button, at: Point) -> Option<A> {
        let shown = self.shown.as_mut()?;
        if shown.pressed != Some(button) {
            return None;
        }
        shown.pressed = None;
        let chosen = shown
            .layout
            .entry_at(at)
            .and_then(|index| shown.menu.action(index));
        self.choose(chosen)
    }

    /// The window no longer gets all the events of the mouse: the press is
    /// taken back.
    pub fn capture_lost(&mut self) {
        if let Some(shown) = &mut self.shown {
            shown.pressed = None;
        }
    }

    /// A key went down: the arrows, Home and End move the highlight, Esc
    /// closes the menu, and Enter and Space return the action of the item
    /// that is highlighted, and close it. The other keys do nothing.
    pub fn key(&mut self, event: &KeyEvent) -> Option<A> {
        let shown = self.shown.as_mut()?;
        if !event.down {
            return None;
        }
        let step = match event.key {
            VK_UP => Step::Previous,
            VK_DOWN => Step::Next,
            VK_HOME => Step::First,
            VK_END => Step::Last,
            VK_RETURN | VK_SPACE => {
                let chosen = shown.menu.chosen();
                return self.choose(chosen);
            }
            VK_ESCAPE => {
                self.close();
                return None;
            }
            _ => return None,
        };
        let old = shown.menu.highlighted();
        if shown.menu.step(step) {
            let new = shown.menu.highlighted();
            self.highlight_changed(old, new);
        }
        None
    }

    /// Whether some area changed since the last frame.
    pub fn has_damage(&self) -> bool {
        !self.damage.is_empty()
    }

    /// Gives `damage` the areas that changed since the last frame.
    pub fn take_damage(&mut self, damage: &mut dyn FnMut(Rect)) {
        for area in self.damage.drain(..) {
            damage(area);
        }
    }

    /// Draws the part of the menu that is inside the clip of `canvas`, with
    /// `font`, if it is open.
    pub fn draw(&mut self, canvas: &mut Canvas<'_>, font: &mut Font) {
        let Self { shown, cells, .. } = self;
        let Some(Shown { menu, layout, .. }) = shown else {
            return;
        };
        let rect = layout.menu();
        if !canvas.clip().intersects(rect) {
            return;
        }
        canvas.outline(rect, layout.border(), BORDER);
        // The border is a few pixels.
        let border = layout.border() as i32;
        canvas.fill(
            Rect::new(
                rect.left + border,
                rect.top + border,
                rect.right - border,
                rect.bottom - border,
            ),
            BACKGROUND,
        );
        let cell = font.cell();
        for (index, entry) in menu.entries().iter().enumerate() {
            let Entry::Item { label, enabled, .. } = entry else {
                canvas.fill(layout.line(index), LINE);
                continue;
            };
            let background = if menu.highlighted() == Some(index) {
                canvas.fill(layout.highlight(index), HIGHLIGHT);
                HIGHLIGHT
            } else {
                BACKGROUND
            };
            let room = layout.text(index);
            if !canvas.clip().intersects(room) {
                continue;
            }
            let columns = room.width() / cell.width.max(1);
            text::fit(label, columns as usize, cells);
            // A cell is a few dozen pixels.
            let top = room.top + (room.height() as i32 - cell.height as i32) / 2;
            font.draw(
                &mut canvas.clipped(room),
                Point::new(room.left, top),
                cells,
                FontStyle::REGULAR,
                if *enabled { TEXT } else { DISABLED },
                background,
            );
        }
    }

    /// Closes the menu if `action` was chosen, and returns it.
    fn choose(&mut self, action: Option<A>) -> Option<A> {
        if action.is_some() {
            self.close();
        }
        action
    }

    /// Highlights the entry at `index`, if it can be chosen, and otherwise
    /// nothing.
    fn hover(&mut self, index: Option<usize>) {
        let Some(shown) = &mut self.shown else {
            return;
        };
        let old = shown.menu.highlighted();
        if shown.menu.hover(index) {
            let new = shown.menu.highlighted();
            self.highlight_changed(old, new);
        }
    }

    /// The highlight moved from `old` to `new`: both items are drawn again.
    fn highlight_changed(&mut self, old: Option<usize>, new: Option<usize>) {
        let Some(shown) = &self.shown else {
            return;
        };
        let areas = [old, new].map(|index| index.map(|index| shown.layout.highlight(index)));
        for area in areas.into_iter().flatten() {
            self.damage(area);
        }
    }

    /// `area` changed: the next frame draws it, unless it draws an area
    /// around it already.
    fn damage(&mut self, area: Rect) {
        if !area.is_empty() && !self.damage.iter().any(|damage| damage.contains(area)) {
            self.damage.push(area);
        }
    }
}

#[cfg(test)]
mod tests {
    use meiga_core::input::ControlKeys;
    use meiga_render::{FontSystem, pixel};

    use super::*;
    use crate::view::FONT_FAMILIES;

    /// The menu of a tab, with "Close other tabs" disabled, as with a
    /// single tab.
    fn entries() -> Vec<Entry<u8>> {
        vec![
            Entry::item("Duplicate tab", 1, true),
            Entry::Separator,
            Entry::item("Close tab", 2, true),
            Entry::item("Close other tabs", 3, false),
            Entry::item("Close tabs to the right", 4, true),
        ]
    }

    /// A menu with cells of 7 x 16 pixels, at 100 %, opened at (100, 50) of
    /// a window of 800 x 600, as in `meiga_core::ui::menu`: its items are
    /// from y = 53 to 85, 88 to 120, 120 to 152 and 152 to 184, and their
    /// highlights 2 pixels shorter above and below, from x = 105 to 288.
    fn open(keyboard: bool) -> MenuView<u8> {
        let mut view = MenuView::new();
        view.open(
            entries(),
            96,
            Size::new(7, 16),
            Anchor::At(Point::new(100, 50)),
            Rect::new(0, 0, 800, 600),
            keyboard,
        );
        view
    }

    const MENU: Rect = Rect::new(100, 50, 293, 187);
    const FIRST: Rect = Rect::new(105, 55, 288, 83);
    const SECOND: Rect = Rect::new(105, 90, 288, 118);

    fn damage(view: &mut MenuView<u8>) -> Vec<Rect> {
        let mut areas = Vec::new();
        view.take_damage(&mut |area| areas.push(area));
        areas
    }

    fn key(key: u16) -> KeyEvent {
        KeyEvent {
            key,
            scan: 0,
            character: 0,
            down: true,
            controls: ControlKeys::empty(),
            repeat: 1,
        }
    }

    #[test]
    fn it_draws_all_of_it_when_it_opens_and_closes_and_only_the_highlights_between() {
        let mut view = open(false);
        assert!(view.is_open());
        assert_eq!(damage(&mut view), [MENU]);
        // The first move in the window, onto the first item: the window
        // follows the leave of the pointer from now on.
        assert!(view.moved(Point::new(150, 60)));
        assert_eq!(damage(&mut view), [FIRST]);
        assert!(!view.moved(Point::new(160, 70)));
        assert!(!view.has_damage());
        // Onto the second item: both are drawn again.
        view.moved(Point::new(150, 100));
        assert_eq!(damage(&mut view), [FIRST, SECOND]);
        // The disabled item highlights nothing, and outside the menu
        // neither.
        view.moved(Point::new(150, 130));
        assert_eq!(damage(&mut view), [SECOND]);
        assert!(!view.moved(Point::new(500, 130)));
        assert!(!view.has_damage());
        // The pointer leaves the window, and comes back.
        view.moved(Point::new(150, 60));
        damage(&mut view);
        view.left();
        assert_eq!(damage(&mut view), [FIRST]);
        assert!(view.moved(Point::new(500, 130)));
        assert!(view.close());
        assert!(!view.close());
        assert_eq!(damage(&mut view), [MENU]);
        assert!(!view.is_open());
    }

    #[test]
    fn a_button_that_goes_down_and_up_in_the_menu_chooses_the_item_under_it() {
        let mut view = open(false);
        view.down(Button::Left, Point::new(150, 100));
        assert!(view.is_pressed());
        // It goes up on another item: that one, and the menu closes.
        assert_eq!(view.up(Button::Left, Point::new(150, 160)), Some(4));
        assert!(!view.is_open());
        assert!(!view.is_pressed());

        // The right button too, but not on a disabled item, nor when
        // another button goes up.
        let mut view = open(false);
        view.down(Button::Right, Point::new(150, 130));
        assert_eq!(view.up(Button::Left, Point::new(150, 130)), None);
        assert!(view.is_pressed());
        assert_eq!(view.up(Button::Right, Point::new(150, 130)), None);
        assert!(!view.is_pressed());
        // A button that went down before the menu opened chooses nothing.
        assert_eq!(view.up(Button::Left, Point::new(150, 60)), None);
        // The middle button presses nothing.
        view.down(Button::Middle, Point::new(150, 60));
        assert!(!view.is_pressed());
        // Losing the mouse takes the press back.
        view.down(Button::Left, Point::new(150, 60));
        view.capture_lost();
        assert!(!view.is_pressed());
        assert_eq!(view.up(Button::Left, Point::new(150, 60)), None);
        assert!(view.is_open());
        view.down(Button::Right, Point::new(150, 60));
        assert_eq!(view.up(Button::Right, Point::new(150, 60)), Some(1));
    }

    #[test]
    fn a_button_that_goes_down_outside_closes_it() {
        let mut view = open(false);
        damage(&mut view);
        view.down(Button::Left, Point::new(99, 60));
        assert!(!view.is_open());
        assert!(!view.is_pressed());
        assert_eq!(damage(&mut view), [MENU]);
        // Its border is the menu.
        let mut view = open(false);
        view.down(Button::Right, Point::new(100, 50));
        assert!(view.is_open());
        // Closed, it takes nothing.
        let mut view = MenuView::<u8>::new();
        view.down(Button::Left, Point::new(150, 60));
        assert_eq!(view.up(Button::Left, Point::new(150, 60)), None);
        assert_eq!(view.key(&key(VK_RETURN)), None);
        assert!(!view.has_damage());
    }

    #[test]
    fn the_keyboard_moves_the_highlight_and_chooses() {
        // Opened with the keyboard, its first item is highlighted.
        let mut view = open(true);
        damage(&mut view);
        assert_eq!(view.key(&key(VK_DOWN)), None);
        assert_eq!(damage(&mut view), [FIRST, SECOND]);
        // Down skips the disabled item; Home goes to the first item, and Up
        // from it, round, to the last one.
        view.key(&key(VK_DOWN));
        view.key(&key(VK_HOME));
        view.key(&key(VK_UP));
        assert_eq!(view.key(&key(VK_SPACE)), Some(4));
        assert!(!view.is_open());

        let mut view = open(true);
        view.key(&key(VK_END));
        view.key(&key(VK_UP));
        // The other keys, and the ones that go up, do nothing.
        assert_eq!(view.key(&key(0x41)), None);
        let up = KeyEvent {
            down: false,
            ..key(VK_RETURN)
        };
        assert_eq!(view.key(&up), None);
        assert!(view.is_open());
        assert_eq!(view.key(&key(VK_RETURN)), Some(2));

        // Opened with the mouse, Enter chooses nothing until an item is
        // highlighted, and Esc closes it.
        let mut view = open(false);
        assert_eq!(view.key(&key(VK_RETURN)), None);
        assert!(view.is_open());
        assert_eq!(view.key(&key(VK_ESCAPE)), None);
        assert!(!view.is_open());
    }

    #[test]
    fn it_covers_only_the_areas_inside_it() {
        let view = open(false);
        assert!(view.covers(FIRST));
        assert!(view.covers(MENU));
        assert!(!view.covers(Rect::new(99, 55, 288, 83)));
        assert!(!MenuView::<u8>::new().covers(FIRST));
    }

    #[test]
    fn it_draws_its_border_its_highlight_and_its_text() {
        let fonts = FontSystem::new().unwrap();
        let mut font = Font::new(&fonts, &FONT_FAMILIES, 9.0, 96).unwrap();
        let mut view = MenuView::new();
        view.open(
            entries(),
            96,
            font.cell(),
            Anchor::At(Point::new(10, 10)),
            Rect::new(0, 0, 400, 300),
            true,
        );
        let layout = view.shown.as_ref().unwrap().layout.clone();
        let size = Size::new(400, 300);
        let mut pixels = vec![0; size.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, size, 400).unwrap();
        view.draw(&mut canvas, &mut font);
        let at = |x: i32, y: i32| pixels[y as usize * 400 + x as usize];
        let menu = layout.menu();
        assert_eq!((menu.left, menu.top, menu.height()), (10, 10, 137));
        // The border, the background above the first item, and the first
        // item, highlighted, beside its text.
        assert_eq!(at(10, 10), pixel(BORDER));
        assert_eq!(at(menu.right - 1, menu.bottom - 1), pixel(BORDER));
        assert_eq!(at(11, 12), pixel(BACKGROUND));
        assert_eq!(at(16, 16), pixel(HIGHLIGHT));
        // The line of the separator, from border to border.
        assert_eq!(layout.line(1), Rect::new(11, 46, menu.right - 1, 47));
        assert_eq!(at(11, 46), pixel(LINE));
        assert_eq!(at(11, 45), pixel(BACKGROUND));
        // Nothing outside the menu.
        assert_eq!(at(9, 10), 0);
        assert_eq!(at(menu.right, 10), 0);
        // The text of the items, in their colours: the first one on the
        // highlight, and the disabled one dimmer.
        let text_in = |area: Rect, text: Rgb, background: Rgb| {
            let channels = |color: Rgb| [color.r, color.g, color.b];
            let (text, background_channels) = (channels(text), channels(background));
            let mut ink = false;
            for y in area.top..area.bottom {
                for x in area.left..area.right {
                    let value = at(x, y);
                    let [_, r, g, b] = value.to_be_bytes();
                    let between = [r, g, b]
                        .iter()
                        .zip(text.iter().zip(&background_channels))
                        .all(|(&c, (&t, &b))| (t.min(b)..=t.max(b)).contains(&c));
                    if !between {
                        return false;
                    }
                    ink |= value != pixel(background);
                }
            }
            ink
        };
        assert!(text_in(layout.text(0), TEXT, HIGHLIGHT));
        assert!(text_in(layout.text(2), TEXT, BACKGROUND));
        assert!(!text_in(layout.text(2), DISABLED, BACKGROUND));
        assert!(text_in(layout.text(3), DISABLED, BACKGROUND));
    }
}
