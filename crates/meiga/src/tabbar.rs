//! The tab bar, at the top of the window, as in Windows Terminal: a tab for
//! each session, with its title, or the name of its program, and its close
//! button, and the button that opens a tab, after the last one (ADR 0020).
//! It is always there, also with one tab.
//!
//! The pointer highlights the tab and the button under it. The left button
//! shows a tab as soon as it goes down on it, as in Chrome. The close
//! button and the button that opens a tab act when the left button goes up
//! on the same button where it went down: moving away first takes the
//! click back. The middle button closes the tab where it goes down and up,
//! and the right one opens its menu, where it goes up, without showing it
//! (ADR 0021). The wheel and the other clicks do nothing yet.
//!
//! The bar only draws what changed: a title, the highlight of a tab or of a
//! button, and all of it when its layout changes.

use std::mem;

use meiga_core::color::{Palette, Rgb};
use meiga_core::geometry::{Point, Rect};
use meiga_core::model::{self, TabId};
use meiga_core::mouse::Button;
use meiga_core::text::{self, CellChar};
use meiga_core::ui::tabbar::{Layout, Part};
use meiga_render::{Canvas, Error, Font, FontStyle, FontSystem};

use crate::tabs::TabAction;
use crate::view::FONT_FAMILIES;

/// The size of the titles, in points: the font of the terminal, smaller,
/// and without its zoom.
const FONT_POINTS: f32 = 9.0;

/// The colours, until they can be configured (subphase 8.3). The active tab
/// has the ones of the terminal, so that it looks joined to it.
const BAR: Rgb = Rgb::new(0x0e, 0x10, 0x14);
const HOVERED_TAB: Rgb = Rgb::new(0x1c, 0x1f, 0x25);
const TEXT: Rgb = Rgb::new(0x9a, 0xa0, 0xaa);
const HOVERED_TEXT: Rgb = Rgb::new(0xc9, 0xcd, 0xd4);
const SEPARATOR: Rgb = Rgb::new(0x2b, 0x2f, 0x37);
const BUTTON_HOVERED: Rgb = Rgb::new(0x2b, 0x2f, 0x37);
const BUTTON_PRESSED: Rgb = Rgb::new(0x3a, 0x3f, 0x49);

/// What the tab of a content shows: the title of its program or, without
/// one, the name of the program.
pub trait Label {
    fn label(&self) -> &str;
}

/// A part of the bar that the pointer highlights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    /// A tab, but not its close button.
    Tab(TabId),
    Close(TabId),
    New,
}

impl Target {
    /// The tab that the target is in.
    const fn tab(self) -> Option<TabId> {
        match self {
            Target::Tab(id) | Target::Close(id) => Some(id),
            Target::New => None,
        }
    }
}

/// A button of the mouse that went down on the bar, and acts if it goes up
/// on the same place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Press {
    /// The left button, on the close button of a tab.
    Close(TabId),
    /// The left button, on the button that opens a tab.
    New,
    /// The middle button, on a tab.
    Middle(TabId),
    /// The right button, on a tab.
    Right(TabId),
}

/// How a tab looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Look {
    Active,
    Hovered,
    Plain,
}

pub struct TabBar {
    /// The fonts of the system, for a new font when the scale changes.
    fonts: FontSystem,
    font: Font,
    dpi: u32,
    layout: Layout,
    /// Where the pointer is, while it is over the bar.
    pointer: Option<Point>,
    hover: Option<Target>,
    pressed: Option<Press>,
    /// The areas that changed since the last frame.
    damage: Vec<Rect>,
    /// The cells of a title, kept to avoid an allocation for each one.
    cells: Vec<CellChar>,
}

impl TabBar {
    /// The bar of `tabs` tabs in a window `width` pixels wide at `dpi`,
    /// with a font of `fonts`.
    pub fn new(fonts: &FontSystem, dpi: u32, width: u32, tabs: usize) -> Result<Self, Error> {
        let font = Font::new(fonts, &FONT_FAMILIES, FONT_POINTS, dpi)?;
        Ok(Self {
            fonts: fonts.clone(),
            font,
            dpi,
            layout: Layout::new(dpi, width, tabs),
            pointer: None,
            hover: None,
            pressed: None,
            damage: Vec::new(),
            cells: Vec::new(),
        })
    }

    /// The height of the bar, in pixels, where the terminal starts.
    pub const fn height(&self) -> u32 {
        self.layout.height()
    }

    /// Whether a button of the mouse went down on the bar and did not go up
    /// yet: meanwhile, the bar gets all the events of the mouse.
    pub const fn is_pressed(&self) -> bool {
        self.pressed.is_some()
    }

    /// Whether `at` is on the bar.
    pub fn contains(&self, at: Point) -> bool {
        self.layout.bar().contains_point(at)
    }

    /// The tab at `index`, as the bar draws it.
    pub fn tab(&self, index: usize) -> Rect {
        self.layout.tab(index)
    }

    /// The font of the titles, which the menus use too.
    pub const fn font_mut(&mut self) -> &mut Font {
        &mut self.font
    }

    /// The window is `width` pixels wide now.
    pub fn resize<T>(&mut self, width: u32, tabs: &model::Window<T>) {
        if width != self.layout.width() {
            self.layout.resize(width, tabs.tab_count());
            self.relayout(tabs);
        }
    }

    /// The window moved to a monitor with `dpi`: a font and a layout for
    /// it. If DirectWrite cannot make the font, the old one stays.
    pub fn set_dpi<T>(&mut self, dpi: u32, tabs: &model::Window<T>) {
        if dpi == self.dpi {
            return;
        }
        self.dpi = dpi;
        if let Ok(font) = Font::new(&self.fonts, &FONT_FAMILIES, FONT_POINTS, dpi) {
            self.font = font;
        }
        self.layout = Layout::new(dpi, self.layout.width(), tabs.tab_count());
        self.relayout(tabs);
    }

    /// A tab opened or closed.
    pub fn tabs_changed<T>(&mut self, tabs: &model::Window<T>) {
        self.layout.resize(self.layout.width(), tabs.tab_count());
        self.relayout(tabs);
    }

    /// The label of the tab at `index` changed.
    pub fn label_changed(&mut self, index: usize) {
        self.damage(self.layout.title(index));
    }

    /// The parts of the bar moved: the pointer may be on another one, and
    /// all of it is drawn again.
    fn relayout<T>(&mut self, tabs: &model::Window<T>) {
        self.hover = self.pointer.and_then(|at| self.target_at(tabs, at));
        self.damage.clear();
        self.damage(self.layout.bar());
    }

    /// The pointer moved to `at`: on the bar or, while a button is down on
    /// it, anywhere. Returns whether it came onto the bar, whose leave the
    /// window has to follow then.
    pub fn moved<T>(&mut self, tabs: &model::Window<T>, at: Point) -> bool {
        let inside = self.contains(at);
        let entered = inside && self.pointer.is_none();
        self.pointer = inside.then_some(at);
        let hover = self.target_at(tabs, at);
        self.set_hover(tabs, hover);
        entered
    }

    /// The pointer left the bar, or the window.
    pub fn left<T>(&mut self, tabs: &model::Window<T>) {
        self.pointer = None;
        self.set_hover(tabs, None);
    }

    /// `button` went down where the pointer is. Returns what it asks of the
    /// tabs: the left button shows the tab under it at once. On a button,
    /// or the middle or the right one on a tab, it waits until it goes up,
    /// and the bar gets all the events of the mouse meanwhile.
    pub fn down<T>(&mut self, tabs: &model::Window<T>, button: Button) -> Option<TabAction> {
        // Another button is already down on the bar.
        if self.pressed.is_some() {
            return None;
        }
        let press = match (button, self.hover?) {
            (Button::Left, Target::Tab(id)) => return Some(TabAction::Activate(id)),
            (Button::Left, Target::Close(id)) => Press::Close(id),
            (Button::Left, Target::New) => Press::New,
            (Button::Middle, Target::Tab(id) | Target::Close(id)) => Press::Middle(id),
            (Button::Right, Target::Tab(id) | Target::Close(id)) => Press::Right(id),
            _ => return None,
        };
        self.pressed = Some(press);
        self.damage_press(tabs, press);
        None
    }

    /// `button` went up where the pointer is. Returns what its press asks
    /// of the tabs, if it went down on the same place: to close a tab, to
    /// open one, or to open the menu of a tab where the pointer is.
    pub fn up<T>(&mut self, tabs: &model::Window<T>, button: Button) -> Option<TabAction> {
        let press = self.pressed?;
        let on_tab = |id| self.hover.is_some_and(|hover| hover.tab() == Some(id));
        let action = match (press, button) {
            (Press::Close(id), Button::Left) => {
                (self.hover == Some(Target::Close(id))).then_some(TabAction::CloseTab(id))
            }
            (Press::New, Button::Left) => {
                (self.hover == Some(Target::New)).then_some(TabAction::New)
            }
            (Press::Middle(id), Button::Middle) => on_tab(id).then_some(TabAction::CloseTab(id)),
            (Press::Right(id), Button::Right) => self
                .pointer
                .filter(|_| on_tab(id))
                .map(|at| TabAction::Menu(id, at)),
            // Another button: the press goes on.
            _ => return None,
        };
        self.pressed = None;
        self.damage_press(tabs, press);
        action
    }

    /// The window no longer gets all the events of the mouse: the press is
    /// taken back.
    pub fn capture_lost<T>(&mut self, tabs: &model::Window<T>) {
        if let Some(press) = self.pressed.take() {
            self.damage_press(tabs, press);
        }
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

    /// Draws the part of the bar that is inside the clip of `canvas`: the
    /// tabs of `tabs`, the active one with the colours of the terminal,
    /// `palette`.
    pub fn draw<T: Label>(
        &mut self,
        canvas: &mut Canvas<'_>,
        tabs: &model::Window<T>,
        palette: &Palette,
    ) {
        let clip = canvas.clip();
        if !clip.intersects(self.layout.bar()) {
            return;
        }
        canvas.fill(self.layout.bar(), BAR);
        let active = tabs.active_index();
        // Whether the tab on the left is plain: between two plain tabs
        // there is a separator.
        let mut after_plain = false;
        for (index, tab) in tabs.tabs().iter().enumerate() {
            let look = if index == active {
                Look::Active
            } else if self.hover.and_then(Target::tab) == Some(tab.id()) {
                Look::Hovered
            } else {
                Look::Plain
            };
            if clip.intersects(self.layout.tab_area(index)) {
                let separator = after_plain && look == Look::Plain;
                self.draw_tab(canvas, index, tab, look, separator, palette);
            }
            after_plain = look == Look::Plain;
        }
        if clip.intersects(self.layout.new_tab()) {
            self.draw_new(canvas);
        }
    }

    /// Draws the tab at `index`, which looks as `look`, with the separator
    /// on its left if `separator`.
    fn draw_tab<T: Label>(
        &mut self,
        canvas: &mut Canvas<'_>,
        index: usize,
        tab: &model::Tab<T>,
        look: Look,
        separator: bool,
        palette: &Palette,
    ) {
        let layout = self.layout;
        let (background, color) = match look {
            Look::Active => (palette.background, palette.foreground),
            Look::Hovered => (HOVERED_TAB, HOVERED_TEXT),
            Look::Plain => (BAR, TEXT),
        };
        let rect = layout.tab(index);
        if look != Look::Plain {
            canvas.fill(rect, background);
        }
        if separator && let Some(line) = layout.separator(index) {
            canvas.fill(line, SEPARATOR);
        }

        let title = layout.title(index);
        let cell = self.font.cell();
        let columns = title.width() / cell.width.max(1);
        text::fit(tab.content().label(), columns as usize, &mut self.cells);
        // A cell is a few dozen pixels.
        let top = rect.top + (rect.height() as i32 - cell.height as i32) / 2;
        self.font.draw(
            &mut canvas.clipped(title),
            Point::new(title.left, top),
            &self.cells,
            FontStyle::REGULAR,
            color,
            background,
        );

        let id = tab.id();
        if let Some(highlight) = layout.close_highlight(index)
            && let Some(fill) = self.button_color(Target::Close(id), Press::Close(id))
        {
            canvas.fill(highlight, fill);
        }
        if let Some(cross) = layout.cross(index) {
            draw_cross(canvas, cross, layout.stroke(), color);
        }
    }

    /// Draws the button that opens a tab.
    fn draw_new(&self, canvas: &mut Canvas<'_>) {
        if let Some(fill) = self.button_color(Target::New, Press::New) {
            canvas.fill(self.layout.new_highlight(), fill);
        }
        let color = if self.hover == Some(Target::New) {
            HOVERED_TEXT
        } else {
            TEXT
        };
        draw_plus(canvas, self.layout.plus(), self.layout.stroke(), color);
    }

    /// The colour of the highlight of the button `target`, which `press`
    /// presses, if it has one: while the pointer is on it, unless another
    /// button is down.
    fn button_color(&self, target: Target, press: Press) -> Option<Rgb> {
        if self.hover != Some(target) {
            return None;
        }
        match self.pressed {
            None => Some(BUTTON_HOVERED),
            Some(pressed) if pressed == press => Some(BUTTON_PRESSED),
            Some(_) => None,
        }
    }

    /// The part of the bar at `at` that the pointer highlights, if any.
    fn target_at<T>(&self, tabs: &model::Window<T>, at: Point) -> Option<Target> {
        let id = |index: usize| tabs.tabs().get(index).map(model::Tab::id);
        match self.layout.part_at(at)? {
            Part::Tab(index) => id(index).map(Target::Tab),
            Part::Close(index) => id(index).map(Target::Close),
            Part::New => Some(Target::New),
            Part::Empty => None,
        }
    }

    /// The pointer highlights `hover` now: what changes is drawn again.
    /// From a tab to its close button, only the button.
    fn set_hover<T>(&mut self, tabs: &model::Window<T>, hover: Option<Target>) {
        let old = mem::replace(&mut self.hover, hover);
        if old == hover {
            return;
        }
        let (old_tab, new_tab) = (old.and_then(Target::tab), hover.and_then(Target::tab));
        if old_tab != new_tab {
            for id in [old_tab, new_tab].into_iter().flatten() {
                if let Some(index) = tabs.index_of(id) {
                    self.damage(self.layout.tab_area(index));
                }
            }
        }
        for target in [old, hover].into_iter().flatten() {
            match target {
                Target::Close(id) => self.damage_close(tabs, id),
                Target::New => self.damage(self.layout.new_tab()),
                Target::Tab(_) => {}
            }
        }
    }

    /// `press` began or ended: the button that it presses changes.
    fn damage_press<T>(&mut self, tabs: &model::Window<T>, press: Press) {
        match press {
            Press::Close(id) => self.damage_close(tabs, id),
            Press::New => self.damage(self.layout.new_tab()),
            Press::Middle(_) | Press::Right(_) => {}
        }
    }

    fn damage_close<T>(&mut self, tabs: &model::Window<T>, id: TabId) {
        if let Some(close) = tabs.index_of(id).and_then(|index| self.layout.close(index)) {
            self.damage(close);
        }
    }

    /// `area` of the bar changed: the next frame draws it, unless it draws
    /// an area around it already.
    fn damage(&mut self, area: Rect) {
        let area = area.intersection(self.layout.bar());
        if !area.is_empty() && !self.damage.iter().any(|damage| damage.contains(area)) {
            self.damage.push(area);
        }
    }
}

/// Draws a cross in the square `rect`, with lines `stroke` pixels thick, as
/// steps of a pixel: at 45 degrees they are as sharp as straight ones.
fn draw_cross(canvas: &mut Canvas<'_>, rect: Rect, stroke: u32, color: Rgb) {
    // A cross is a few pixels.
    let (side, stroke) = (rect.width() as i32, stroke as i32);
    for step in 0..=side - stroke {
        let top = rect.top + step;
        canvas.fill(
            Rect::new(
                rect.left + step,
                top,
                rect.left + step + stroke,
                top + stroke,
            ),
            color,
        );
        canvas.fill(
            Rect::new(
                rect.right - step - stroke,
                top,
                rect.right - step,
                top + stroke,
            ),
            color,
        );
    }
}

/// Draws a plus in the square `rect`, with lines `stroke` pixels thick.
fn draw_plus(canvas: &mut Canvas<'_>, rect: Rect, stroke: u32, color: Rgb) {
    // A plus is a few pixels.
    let (side, stroke) = (rect.width() as i32, stroke as i32);
    let middle = (side - stroke) / 2;
    let (x, y) = (rect.left + middle, rect.top + middle);
    canvas.fill(Rect::new(rect.left, y, rect.right, y + stroke), color);
    canvas.fill(Rect::new(x, rect.top, x + stroke, rect.bottom), color);
}

#[cfg(test)]
mod tests {
    use meiga_core::geometry::Size;
    use meiga_render::pixel;

    use super::*;

    impl Label for String {
        fn label(&self) -> &str {
            self
        }
    }

    /// The tabs of `programs`, the first one active, in a window of 800
    /// pixels at 100 %, and their bar.
    fn bar(programs: &[&str]) -> (TabBar, model::Window<String>) {
        let mut tabs = model::Window::new(programs[0].to_owned());
        for program in &programs[1..] {
            tabs.open((*program).to_owned());
        }
        let fonts = FontSystem::new().unwrap();
        let bar = TabBar::new(&fonts, 96, 800, tabs.tab_count()).unwrap();
        (bar, tabs)
    }

    fn three() -> (TabBar, model::Window<String>) {
        bar(&["pwsh", "cmd", "wsl"])
    }

    fn damage(bar: &mut TabBar) -> Vec<Rect> {
        let mut areas = Vec::new();
        bar.take_damage(&mut |area| areas.push(area));
        areas
    }

    fn id(tabs: &model::Window<String>, index: usize) -> TabId {
        tabs.tabs()[index].id()
    }

    #[test]
    fn the_pointer_draws_again_only_what_it_highlights() {
        let (mut bar, tabs) = three();
        // Onto the second tab, which the window follows from now on.
        assert!(bar.moved(&tabs, Point::new(300, 20)));
        assert_eq!(damage(&mut bar), [Rect::new(244, 0, 485, 32)]);
        assert!(!bar.moved(&tabs, Point::new(310, 2)));
        assert_eq!(damage(&mut bar), []);
        // Onto its close button: only the button.
        bar.moved(&tabs, Point::new(460, 20));
        assert_eq!(damage(&mut bar), [Rect::new(456, 6, 480, 30)]);
        // Onto the next tab: both tabs, with the button.
        bar.moved(&tabs, Point::new(500, 20));
        assert_eq!(
            damage(&mut bar),
            [Rect::new(244, 0, 485, 32), Rect::new(484, 0, 725, 32)]
        );
        bar.left(&tabs);
        assert_eq!(damage(&mut bar), [Rect::new(484, 0, 725, 32)]);
        // The button that opens a tab, and the empty bar.
        assert!(bar.moved(&tabs, Point::new(740, 10)));
        assert_eq!(damage(&mut bar), [Rect::new(728, 4, 756, 32)]);
        bar.moved(&tabs, Point::new(780, 10));
        assert_eq!(damage(&mut bar), [Rect::new(728, 4, 756, 32)]);
        assert!(!bar.moved(&tabs, Point::new(790, 10)));
        assert!(!bar.has_damage());
    }

    #[test]
    fn the_left_button_shows_a_tab_when_it_goes_down() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(300, 20));
        let action = bar.down(&tabs, Button::Left);
        assert_eq!(action, Some(TabAction::Activate(id(&tabs, 1))));
        assert!(!bar.is_pressed());
        assert_eq!(bar.up(&tabs, Button::Left), None);
        // The empty bar does nothing yet.
        bar.moved(&tabs, Point::new(790, 20));
        assert_eq!(bar.down(&tabs, Button::Left), None);
        assert_eq!(bar.down(&tabs, Button::Right), None);
        assert!(!bar.is_pressed());
    }

    #[test]
    fn buttons_act_when_the_left_button_goes_up_on_them() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(460, 20));
        damage(&mut bar);
        assert_eq!(bar.down(&tabs, Button::Left), None);
        assert!(bar.is_pressed());
        assert_eq!(damage(&mut bar), [Rect::new(456, 6, 480, 30)]);
        // Other buttons do nothing meanwhile.
        assert_eq!(bar.down(&tabs, Button::Middle), None);
        assert_eq!(bar.up(&tabs, Button::Middle), None);
        assert!(bar.is_pressed());
        let action = bar.up(&tabs, Button::Left);
        assert_eq!(action, Some(TabAction::CloseTab(id(&tabs, 1))));
        assert!(!bar.is_pressed());
        assert_eq!(damage(&mut bar), [Rect::new(456, 6, 480, 30)]);

        // Moving away takes the click back.
        bar.moved(&tabs, Point::new(740, 10));
        bar.down(&tabs, Button::Left);
        bar.moved(&tabs, Point::new(740, 100));
        assert_eq!(bar.up(&tabs, Button::Left), None);
        assert!(!bar.is_pressed());
        // And coming back gives it again.
        bar.moved(&tabs, Point::new(740, 10));
        bar.down(&tabs, Button::Left);
        bar.moved(&tabs, Point::new(300, 10));
        bar.moved(&tabs, Point::new(745, 25));
        assert_eq!(bar.up(&tabs, Button::Left), Some(TabAction::New));
    }

    #[test]
    fn the_middle_button_closes_the_tab_where_it_goes_down_and_up() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(100, 20));
        assert_eq!(bar.down(&tabs, Button::Middle), None);
        assert!(bar.is_pressed());
        // Its close button is part of the tab.
        bar.moved(&tabs, Point::new(220, 20));
        let action = bar.up(&tabs, Button::Middle);
        assert_eq!(action, Some(TabAction::CloseTab(id(&tabs, 0))));
        // Not on another tab.
        bar.moved(&tabs, Point::new(300, 20));
        bar.down(&tabs, Button::Middle);
        bar.moved(&tabs, Point::new(100, 20));
        assert_eq!(bar.up(&tabs, Button::Middle), None);
        // Not on the button that opens a tab.
        bar.moved(&tabs, Point::new(740, 20));
        assert_eq!(bar.down(&tabs, Button::Middle), None);
        assert!(!bar.is_pressed());
    }

    #[test]
    fn the_right_button_opens_the_menu_of_the_tab_where_it_goes_down_and_up() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(300, 20));
        damage(&mut bar);
        // It does not show the tab, and draws nothing.
        assert_eq!(bar.down(&tabs, Button::Right), None);
        assert!(bar.is_pressed());
        assert!(!bar.has_damage());
        // The menu opens where the button goes up, also on the close button
        // of the tab.
        bar.moved(&tabs, Point::new(460, 20));
        let action = bar.up(&tabs, Button::Right);
        let menu = TabAction::Menu(id(&tabs, 1), Point::new(460, 20));
        assert_eq!(action, Some(menu));
        assert!(!bar.is_pressed());
        // Not on another tab, nor below the bar.
        bar.moved(&tabs, Point::new(100, 20));
        bar.down(&tabs, Button::Right);
        bar.moved(&tabs, Point::new(300, 20));
        assert_eq!(bar.up(&tabs, Button::Right), None);
        bar.moved(&tabs, Point::new(100, 20));
        bar.down(&tabs, Button::Right);
        bar.moved(&tabs, Point::new(100, 40));
        assert_eq!(bar.up(&tabs, Button::Right), None);
        // Not on the button that opens a tab.
        bar.moved(&tabs, Point::new(740, 20));
        assert_eq!(bar.down(&tabs, Button::Right), None);
        assert!(!bar.is_pressed());
    }

    #[test]
    fn losing_the_mouse_takes_the_click_back() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(220, 20));
        bar.down(&tabs, Button::Left);
        damage(&mut bar);
        bar.capture_lost(&tabs);
        assert!(!bar.is_pressed());
        assert_eq!(damage(&mut bar), [Rect::new(216, 6, 240, 30)]);
        assert_eq!(bar.up(&tabs, Button::Left), None);
    }

    #[test]
    fn a_new_layout_draws_the_whole_bar_and_finds_the_pointer_again() {
        let (mut bar, mut tabs) = three();
        bar.moved(&tabs, Point::new(300, 20));
        damage(&mut bar);
        let closed = tabs.close(id(&tabs, 1));
        assert!(matches!(
            closed,
            Ok(model::Close::Removed {
                activated: None,
                ..
            })
        ));
        bar.tabs_changed(&tabs);
        assert_eq!(damage(&mut bar), [Rect::new(0, 0, 800, 32)]);
        // The pointer is on the tab that took the place of the closed one.
        assert_eq!(bar.hover, Some(Target::Tab(id(&tabs, 1))));
        bar.label_changed(1);
        bar.label_changed(1);
        assert_eq!(damage(&mut bar), [Rect::new(252, 4, 456, 32)]);
        // The same width is no change; another one is.
        bar.resize(800, &tabs);
        assert!(!bar.has_damage());
        bar.resize(600, &tabs);
        assert_eq!(damage(&mut bar), [Rect::new(0, 0, 600, 32)]);
        bar.set_dpi(144, &tabs);
        assert_eq!(bar.height(), 48);
        assert_eq!(damage(&mut bar), [Rect::new(0, 0, 600, 48)]);
    }

    /// Draws the whole bar of `tabs` on a canvas of its size, and returns
    /// its pixels.
    fn draw(bar: &mut TabBar, tabs: &model::Window<String>) -> Vec<u32> {
        let size = Size::new(800, bar.height());
        let mut pixels = vec![0; size.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, size, 800).unwrap();
        bar.draw(&mut canvas, tabs, &Palette::MEIGA);
        pixels
    }

    fn at(pixels: &[u32], x: usize, y: usize) -> u32 {
        pixels[y * 800 + x]
    }

    #[test]
    fn the_active_tab_has_the_colours_of_the_terminal() {
        // Tabs of 190 pixels, and the pointer on the fourth.
        let (mut bar, tabs) = bar(&["pwsh", "cmd", "wsl", "ssh"]);
        bar.moved(&tabs, Point::new(600, 20));
        let pixels = draw(&mut bar, &tabs);
        let palette = Palette::MEIGA;
        // Above the tabs, the bar; the active tab, down to its bottom; the
        // tab under the pointer; and a plain one.
        assert_eq!(at(&pixels, 100, 2), pixel(BAR));
        assert_eq!(at(&pixels, 5, 31), pixel(palette.background));
        assert_eq!(at(&pixels, 590, 31), pixel(HOVERED_TAB));
        assert_eq!(at(&pixels, 250, 31), pixel(BAR));
        // A separator only between two plain tabs: the second and the
        // third.
        assert_eq!(at(&pixels, 384, 18), pixel(SEPARATOR));
        assert_eq!(at(&pixels, 194, 18), pixel(BAR));
        assert_eq!(at(&pixels, 574, 18), pixel(HOVERED_TAB));
        // Each title, in the colours of its tab.
        for (title, text, background) in [
            (
                Rect::new(12, 4, 166, 32),
                palette.foreground,
                palette.background,
            ),
            (Rect::new(202, 4, 356, 32), TEXT, BAR),
            (Rect::new(582, 4, 736, 32), HOVERED_TEXT, HOVERED_TAB),
        ] {
            assert!(text_in(&pixels, title, text, background), "{title:?}");
        }
        // The crosses, as steps, in the colour of the text of their tabs.
        for (x, y) in [(174, 14), (181, 14), (177, 17), (178, 17), (181, 21)] {
            assert_eq!(at(&pixels, x, y), pixel(palette.foreground));
        }
        assert_eq!(at(&pixels, 175, 14), pixel(palette.background));
        assert_eq!(at(&pixels, 364, 14), pixel(TEXT));
        assert_eq!(at(&pixels, 744, 14), pixel(HOVERED_TEXT));
        // The plus, with arms of 5 pixels around its middle, (781, 17).
        for (x, y) in [(776, 17), (786, 17), (781, 12), (781, 22), (781, 17)] {
            assert_eq!(at(&pixels, x, y), pixel(TEXT));
        }
        for (x, y) in [(780, 16), (787, 17), (781, 23)] {
            assert_eq!(at(&pixels, x, y), pixel(BAR));
        }
    }

    #[test]
    fn buttons_are_highlighted_under_the_pointer_and_while_pressed() {
        // The highlight of the close button of the second tab goes from
        // (458, 8) to (478, 28), and the one of the button that opens a
        // tab from (730, 6) to (754, 30).
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(460, 20));
        assert_eq!(at(&draw(&mut bar, &tabs), 458, 8), pixel(BUTTON_HOVERED));
        bar.down(&tabs, Button::Left);
        assert_eq!(at(&draw(&mut bar, &tabs), 477, 27), pixel(BUTTON_PRESSED));
        // While it is pressed, no other button is highlighted.
        bar.moved(&tabs, Point::new(740, 10));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 458, 8), pixel(BAR));
        assert_eq!(at(&pixels, 730, 6), pixel(BAR));
        assert_eq!(bar.up(&tabs, Button::Left), None);
        assert_eq!(at(&draw(&mut bar, &tabs), 730, 6), pixel(BUTTON_HOVERED));
    }

    /// Whether `area` has text of `text` on `background`: some of its
    /// pixels have ink, and all of them are between the two colours.
    fn text_in(pixels: &[u32], area: Rect, text: Rgb, background: Rgb) -> bool {
        let channels = |color: Rgb| [color.r, color.g, color.b];
        let (text, background_channels) = (channels(text), channels(background));
        let mut ink = false;
        for y in area.top..area.bottom {
            for x in area.left..area.right {
                let value = at(pixels, x as usize, y as usize);
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
    }
}
