//! The tab bar, at the top of the window, as in Windows Terminal: a tab for
//! each session, with its title, or the name of its profile, and its close
//! button; and after the last one, the button that opens a tab with the
//! default profile, joined to the one that opens the menu of the profiles
//! (ADR 0020 and 0022). It is always there, also with one tab.
//!
//! The pointer highlights the tab and the button under it. The left button
//! shows a tab as soon as it goes down on it, as in Chrome, and a double
//! click on a tab, as Windows counts them, edits its name, in a field in the
//! place of its title (ADR 0031). The buttons act when the left button goes
//! up on the same button where it went down: moving away first takes the
//! click back. The middle button closes the tab where it goes down and up,
//! and the right one opens its menu, where it goes up, without showing it
//! (ADR 0021). The other clicks do nothing yet.
//!
//! When even the narrowest tabs do not fit, the bar scrolls them (ADR 0025):
//! the wheel, forward and backward or to the sides, a tab for each of its
//! steps; and the buttons on the sides of the tabs, a tab for each press,
//! and again while the left button stays down on them. The tab that is
//! shown is always in view. When the close button or the middle button
//! closes a tab, the other tabs keep their width and their place until the
//! pointer leaves the bar, so that the close button of the next one comes
//! under it, as in Windows Terminal and in Chrome.
//!
//! Before its title, a tab may have one indicator: that its program ended
//! without a clean end, or could not start; or, while it is hidden, that its
//! program rang the bell, or wrote something. Each of them is a symbol that
//! does not move: the bar has no animation.
//!
//! While the window has the title bar of meiga, the bar is its title bar
//! (ADR 0027): on its right go the buttons of the window, minimize, maximize
//! or restore, and close, as in Windows 11. They act as the other buttons,
//! when the left button goes up on the same one where it went down, and the
//! pointer highlights them: the close button, in red. Without the focus,
//! their glyphs are dimmer. The rest of the bar is the caption of the
//! window: dragging it moves the window, and a double click maximizes or
//! restores it, as Windows does with its own; the right button opens the
//! menu of the window, where it goes up; and the band above the tabs, while
//! the window is not maximized, resizes it. Those clicks never reach the
//! bar. While a menu of meiga is open, the rest of the bar is client area,
//! and a click on it only closes the menu.
//!
//! The bar only draws what changed: a title, the highlight of a tab or of a
//! button, and all of it when its layout changes or it scrolls.

use std::mem;
use std::time::{Duration, Instant};

use meiga_core::color::{Palette, Rgb};
use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::model::{self, TabId};
use meiga_core::mouse::{Button, ClickSettings, Clicks};
use meiga_core::text::{self, CellChar};
use meiga_core::ui::tabbar::{CaptionButton, Layout, Part, Side};
use meiga_render::{Canvas, Error, Font, FontStyle, FontSystem};

use crate::rename::View;
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
/// The arrow of a button that scrolls the tabs, while they cannot scroll
/// towards its side: in the colour of a pressed button, darker than the
/// text.
const DISABLED_TEXT: Rgb = Rgb::new(0x3a, 0x3f, 0x49);
/// The indicators: the end of the program, in the bright red of the
/// palette of the terminal, and the bell, in amber. The activity has the
/// colour of the title of its tab.
const ENDED: Rgb = Rgb::new(0xe7, 0x48, 0x56);
const BELL: Rgb = Rgb::new(0xe5, 0xc0, 0x7b);
/// The close button of the window, as the one of Windows 11 and of Windows
/// Terminal: red under the pointer, with a white glyph; and pressed, the
/// red at 90 % over the bar, with the white at 70 %.
const CLOSE_HOVERED: Rgb = Rgb::new(0xc4, 0x2b, 0x1c);
const CLOSE_GLYPH: Rgb = Rgb::new(0xff, 0xff, 0xff);
const CLOSE_PRESSED: Rgb = BAR.mix(CLOSE_HOVERED, 230);
const CLOSE_PRESSED_GLYPH: Rgb = CLOSE_PRESSED.mix(CLOSE_GLYPH, 179);
/// The glyphs of the buttons of the window without the focus: halfway
/// between the text and the bar.
const INACTIVE_GLYPH: Rgb = TEXT.mix(BAR, 128);

/// While the left button stays down on a button that scrolls the tabs, and
/// the pointer on it, they scroll another tab after `FIRST_REPEAT`, as the
/// scroll bar of the terminal, and then after each `REPEAT`, as the
/// buttons of Windows Terminal.
const FIRST_REPEAT: Duration = Duration::from_millis(250);
const REPEAT: Duration = Duration::from_millis(100);

/// A step of the wheel, `WHEEL_DELTA` of Windows: it scrolls a tab.
const WHEEL_STEP: i64 = 120;

/// What the tab of a content shows: the name that the user gave it or,
/// without one, the title of its program or, without one, the name of its
/// profile; and an indicator, if it has one.
pub trait Label {
    fn label(&self) -> &str;

    fn indicator(&self) -> Option<Indicator> {
        None
    }
}

/// What a tab tells before its title (ADR 0025). Only one shows: the first
/// one that the tab has, in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Indicator {
    /// Its program ended without a clean end, or could not start: until it
    /// starts again, also while the tab is shown.
    Ended,
    /// While it was hidden, its program rang the bell.
    Bell,
    /// While it was hidden, its program wrote something.
    Activity,
}

impl Indicator {
    /// How many cells it takes before the title, which a blank one
    /// separates from it.
    const fn cells(self) -> usize {
        match self {
            Self::Bell => 2,
            Self::Ended | Self::Activity => 1,
        }
    }
}

/// How the window frames the bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Whether the bar is the title bar of the window, with its buttons:
    /// while the window has the title bar of meiga.
    pub buttons: bool,
    /// How many pixels above the tabs resize the window from its top edge.
    pub band: u32,
    /// Whether the window is maximized: its button restores it then.
    pub maximized: bool,
}

/// A part of the bar that the pointer highlights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    /// A tab, but not its close button.
    Tab(TabId),
    Close(TabId),
    New,
    Profiles,
    Scroll(Side),
    /// A button of the window.
    Caption(CaptionButton),
}

impl Target {
    /// The tab that the target is in.
    const fn tab(self) -> Option<TabId> {
        match self {
            Target::Tab(id) | Target::Close(id) => Some(id),
            Target::New | Target::Profiles | Target::Scroll(_) | Target::Caption(_) => None,
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
    /// The left button, on the button that opens the menu of the profiles.
    Profiles,
    /// The left button, on a button that scrolls the tabs: they scrolled,
    /// and scroll again while it stays down.
    Scroll(Side),
    /// The left button, on a button of the window.
    Caption(CaptionButton),
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

/// How a tab is drawn.
struct Style<'a> {
    look: Look,
    /// Whether the separator on its left is drawn: between two plain tabs.
    separator: bool,
    /// The field where its name is edited, which goes in the place of its
    /// title.
    field: Option<&'a View<'a>>,
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
    /// While the left button is down on a button that scrolls the tabs,
    /// when they scroll again, once the first frame after the press has set
    /// it.
    repeat: Option<Instant>,
    /// The turn of the wheel that did not scroll a whole pixel yet, in
    /// pixels times [`WHEEL_STEP`].
    wheel: i64,
    /// The index of the tab that the close button or the middle button
    /// closes now.
    closing: Option<usize>,
    /// Tabs closed with the mouse while the pointer is on the bar: the
    /// others keep their width and their place until it leaves it.
    frozen: bool,
    /// The tab that is shown has to be in view: the next frame scrolls the
    /// tabs for it, once the bar is not frozen.
    reveal: bool,
    /// How the window frames the bar.
    frame: Frame,
    /// Whether the window has the focus: the glyphs of its buttons are
    /// dimmer without it.
    focused: bool,
    /// When Windows takes two clicks as a double click.
    click_settings: ClickSettings,
    /// The clicks of the left button on the tabs, and the tab of the last
    /// one: a double click on a tab edits its name.
    clicks: Clicks,
    clicked: Option<TabId>,
    /// The areas that changed since the last frame.
    damage: Vec<Rect>,
    /// The cells of a title, kept to avoid an allocation for each one.
    cells: Vec<CellChar>,
}

impl TabBar {
    /// The bar of `tabs` tabs in a window `width` pixels wide at `dpi`,
    /// which frames it as `frame` says, with a font of `fonts`. The window
    /// does not have the focus yet.
    pub fn new(
        fonts: &FontSystem,
        dpi: u32,
        width: u32,
        tabs: usize,
        frame: Frame,
    ) -> Result<Self, Error> {
        let font = Font::new(fonts, &FONT_FAMILIES, FONT_POINTS, dpi)?;
        Ok(Self {
            fonts: fonts.clone(),
            font,
            dpi,
            layout: Layout::new(dpi, width, tabs, frame.band),
            pointer: None,
            hover: None,
            pressed: None,
            repeat: None,
            wheel: 0,
            closing: None,
            frozen: false,
            reveal: false,
            frame,
            focused: false,
            click_settings: ClickSettings::default(),
            clicks: Clicks::new(),
            clicked: None,
            damage: Vec::new(),
            cells: Vec::new(),
        })
    }

    /// The height of the bar, in pixels, where the terminal starts.
    pub const fn height(&self) -> u32 {
        self.layout.height()
    }

    /// Where each part of the bar is, for the title bar of the window.
    pub const fn layout(&self) -> Layout {
        self.layout
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

    /// The tab at `index`, as far as the bar draws it: its part in the
    /// strip, which is empty if the strip does not show it.
    pub fn tab(&self, index: usize) -> Rect {
        self.layout.tab(index).intersection(self.layout.strip())
    }

    /// The buttons that open a tab and the menu of the profiles, which
    /// opens below them.
    pub fn buttons(&self) -> Rect {
        self.layout.new_tab().union(self.layout.profiles())
    }

    /// The font of the titles, which the menus use too.
    pub const fn font_mut(&mut self) -> &mut Font {
        &mut self.font
    }

    /// The size of the cells of the titles.
    pub const fn cell(&self) -> Size {
        self.font.cell()
    }

    /// Where the title of the tab at `index` goes, after `indicator`, if the
    /// tab has one, as far as the strip shows it: there goes the field where
    /// its name is edited.
    pub fn text_area(&self, index: usize, indicator: Option<Indicator>) -> Rect {
        self.title(index, indicator)
            .0
            .intersection(self.layout.strip())
    }

    /// The window is `width` pixels wide now: the tabs get their width
    /// again, also if they kept it, and the one that is shown has to be in
    /// view.
    pub fn resize<T>(&mut self, width: u32, tabs: &model::Window<T>) {
        if width != self.layout.width() {
            self.frozen = false;
            self.reveal = true;
            self.layout.resize(width, tabs.tab_count());
            self.relayout(tabs);
        }
    }

    /// The window moved to a monitor with `dpi`: a font and a layout for
    /// it, whose strip shows the tab that is shown. If DirectWrite cannot
    /// make the font, the old one stays.
    pub fn set_dpi<T>(&mut self, dpi: u32, tabs: &model::Window<T>) {
        if dpi == self.dpi {
            return;
        }
        self.dpi = dpi;
        if let Ok(font) = Font::new(&self.fonts, &FONT_FAMILIES, FONT_POINTS, dpi) {
            self.font = font;
        }
        self.frozen = false;
        self.reveal = true;
        self.wheel = 0;
        self.layout = Layout::new(
            dpi,
            self.layout.width(),
            tabs.tab_count(),
            self.layout.band(),
        );
        self.relayout(tabs);
    }

    /// The window frames the bar as `frame` says now. Another band moves the
    /// parts of the bar down or up, and the tabs keep their width and their
    /// scroll; with or without the buttons of the window, the parts stay,
    /// since the bar always keeps the room of those buttons. Either way, all
    /// of the bar is drawn again. With the same band, once the window is
    /// maximized or restored, only the button that does it changes, if the
    /// bar has the buttons; but a maximized window has no band, so with the
    /// title bar of meiga the band changes too.
    pub fn set_frame<T>(&mut self, frame: Frame, tabs: &model::Window<T>) {
        let old = mem::replace(&mut self.frame, frame);
        if old.band != frame.band || old.buttons != frame.buttons {
            self.layout.set_band(frame.band);
            self.relayout(tabs);
        } else if old.maximized != frame.maximized && frame.buttons {
            self.damage(self.layout.caption(CaptionButton::Maximize));
        }
    }

    /// The window gained or lost the focus: the glyphs of its buttons
    /// change.
    pub fn set_focused(&mut self, focused: bool) {
        if mem::replace(&mut self.focused, focused) != focused && self.frame.buttons {
            self.damage(self.layout.captions());
        }
    }

    /// Windows takes two clicks as a double click as `settings` says.
    pub const fn set_click_settings(&mut self, settings: ClickSettings) {
        self.click_settings = settings;
    }

    /// A button went down where the bar does not get it: as in Windows, a
    /// click on a tab before it and another one after it are not a double
    /// click.
    pub fn forget_clicks(&mut self) {
        self.clicks.reset();
    }

    /// Tabs opened or closed, but not one alone: [`TabBar::tab_closed`].
    /// The tabs get their width again, also if they kept it, and the one
    /// that is shown has to be in view.
    pub fn tabs_changed<T>(&mut self, tabs: &model::Window<T>) {
        self.closing = None;
        self.frozen = false;
        self.reveal = true;
        self.layout.resize(self.layout.width(), tabs.tab_count());
        self.relayout(tabs);
    }

    /// The tab that was at `index` closed. If the close button or the
    /// middle button closed it, or tabs closed so before, with the pointer
    /// still on the bar, the other tabs keep their width and their place
    /// until it leaves it: the one on its right goes where it was, with its
    /// close button under the pointer. The last tab lets the others get
    /// their width at once, as in Windows Terminal, and so do the keyboard
    /// and the menus, if no tab closed so before.
    pub fn tab_closed<T>(&mut self, tabs: &model::Window<T>, index: usize) {
        let with_mouse = self.closing.take() == Some(index);
        if self.pointer.is_some() && index < tabs.tab_count() && (with_mouse || self.frozen) {
            self.frozen = true;
            self.reveal = true;
            self.layout.keep_widths(tabs.tab_count());
            self.relayout(tabs);
        } else {
            self.tabs_changed(tabs);
        }
    }

    /// Another tab is shown, or the one that is shown is chosen again: it
    /// has to be in view.
    pub const fn shown(&mut self) {
        self.reveal = true;
    }

    /// Before a frame: scrolls the tabs so that the one that is shown is in
    /// view, if it has to be, and the bar is not frozen.
    pub fn settle<T>(&mut self, tabs: &model::Window<T>) {
        if self.reveal && !self.frozen {
            self.reveal = false;
            if self.layout.reveal(tabs.active_index()) {
                self.relayout(tabs);
            }
        }
    }

    /// The label of the tab at `index` changed, or its indicator.
    pub fn label_changed(&mut self, index: usize) {
        self.damage(self.layout.title(index).intersection(self.layout.strip()));
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

    /// The pointer left the bar, or the window. Tabs that kept their width
    /// get it again.
    pub fn left<T>(&mut self, tabs: &model::Window<T>) {
        self.pointer = None;
        self.closing = None;
        self.set_hover(tabs, None);
        self.thaw(tabs);
    }

    /// Scrolls the tabs at once, if they do not fit, so that the one at
    /// `index` is in view: a menu opens below it.
    pub fn scroll_to_tab<T>(&mut self, tabs: &model::Window<T>, index: usize) {
        if self.layout.reveal(index) {
            self.relayout(tabs);
        }
    }

    /// `button` went down where the pointer is, at `now`. Returns what it
    /// asks of the tabs: the left button shows the tab under it at once, or
    /// edits its name if it is the second click on it in a row, a double
    /// click; and scrolls the tabs on a button that scrolls them, if they
    /// can scroll. On a button, or the middle or the right one on a tab, it
    /// waits until it goes up, and the bar gets all the events of the mouse
    /// meanwhile.
    pub fn down<T>(
        &mut self,
        tabs: &model::Window<T>,
        button: Button,
        now: Instant,
    ) -> Option<TabAction> {
        // Another button is already down on the bar.
        if self.pressed.is_some() {
            return None;
        }
        self.closing = None;
        let target = self.hover?;
        if let (Button::Left, Target::Tab(id)) = (button, target) {
            return Some(self.click_tab(id, now));
        }
        // Another press between two clicks on a tab: as in Windows, they are
        // not a double click.
        self.clicks.reset();
        let press = match (button, target) {
            (Button::Left, Target::Close(id)) => Press::Close(id),
            (Button::Left, Target::New) => Press::New,
            (Button::Left, Target::Profiles) => Press::Profiles,
            (Button::Left, Target::Caption(caption)) => Press::Caption(caption),
            (Button::Left, Target::Scroll(side)) => {
                if !self.scroll(tabs, side, |layout| layout.step(side)) {
                    return None;
                }
                // The first frame says when they scroll again.
                self.repeat = None;
                Press::Scroll(side)
            }
            (Button::Middle, Target::Tab(id) | Target::Close(id)) => Press::Middle(id),
            (Button::Right, Target::Tab(id) | Target::Close(id)) => Press::Right(id),
            _ => return None,
        };
        self.pressed = Some(press);
        self.damage_press(tabs, press);
        None
    }

    /// The left button went down on the tab `id` at `now`: it shows it, or
    /// edits its name if it is the second click on it in a row. As in
    /// Windows, the click after a double click is a single one again.
    fn click_tab(&mut self, id: TabId, now: Instant) -> TabAction {
        let Some(at) = self.pointer else {
            return TabAction::Activate(id);
        };
        let count = self
            .clicks
            .press(Button::Left, at, now, &self.click_settings);
        let again = self.clicked.replace(id) == Some(id);
        if count != 2 {
            return TabAction::Activate(id);
        }
        self.clicks.reset();
        if again {
            TabAction::Rename(id)
        } else {
            TabAction::Activate(id)
        }
    }

    /// `button` went up where the pointer is. Returns what its press asks
    /// of the tabs, if it went down on the same place: to close a tab, to
    /// open one, to open the menu of the profiles, or the one of a tab
    /// where the pointer is, or what a button of the window does. The left
    /// button that scrolls the tabs stops.
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
            (Press::Profiles, Button::Left) => (self.hover == Some(Target::Profiles))
                .then_some(TabAction::ProfileMenu { keyboard: false }),
            (Press::Caption(caption), Button::Left) => {
                (self.hover == Some(Target::Caption(caption))).then_some(match caption {
                    CaptionButton::Minimize => TabAction::Minimize,
                    CaptionButton::Maximize => TabAction::ToggleMaximize,
                    CaptionButton::Close => TabAction::CloseWindow,
                })
            }
            (Press::Scroll(_), Button::Left) => None,
            (Press::Middle(id), Button::Middle) => on_tab(id).then_some(TabAction::CloseTab(id)),
            (Press::Right(id), Button::Right) => self
                .pointer
                .filter(|_| on_tab(id))
                .map(|at| TabAction::Menu(id, at)),
            // Another button: the press goes on.
            _ => return None,
        };
        if let Some(TabAction::CloseTab(id)) = action {
            self.closing = tabs.index_of(id);
        }
        self.pressed = None;
        self.repeat = None;
        self.damage_press(tabs, press);
        action
    }

    /// The window no longer gets all the events of the mouse: the press is
    /// taken back.
    pub fn capture_lost<T>(&mut self, tabs: &model::Window<T>) {
        self.repeat = None;
        if let Some(press) = self.pressed.take() {
            self.damage_press(tabs, press);
        }
    }

    /// While the left button stays down on a button that scrolls the tabs,
    /// scrolls them another tab, if it is time at `now`, and the pointer is
    /// on that button. Returns when to do it again: never once they cannot
    /// scroll further, and until the pointer comes back onto the button,
    /// whose highlight a frame draws then.
    pub fn tick<T>(&mut self, tabs: &model::Window<T>, now: Instant) -> Option<Instant> {
        let Some(Press::Scroll(side)) = self.pressed else {
            return None;
        };
        if self.hover != Some(Target::Scroll(side)) || !self.layout.can_scroll(side) {
            return None;
        }
        let due = *self.repeat.get_or_insert(now.checked_add(FIRST_REPEAT)?);
        if now < due {
            return Some(due);
        }
        self.scroll(tabs, side, |layout| layout.step(side));
        self.repeat = now.checked_add(REPEAT);
        self.repeat.filter(|_| self.layout.can_scroll(side))
    }

    /// The wheel turned `delta` 120ths of a step, forward or, if it is
    /// `horizontal`, to the right, if it is positive, with the pointer on
    /// the bar. While the tabs do not fit, they scroll a tab for each step:
    /// towards the first one forward or to the left, and towards the last
    /// one backward or to the right. While a button is down on the bar, it
    /// does nothing: the middle button is the wheel, and while it is down on
    /// a tab, the tabs stay where they are.
    pub fn wheel<T>(&mut self, tabs: &model::Window<T>, delta: i16, horizontal: bool) {
        if self.pressed.is_some() {
            return;
        }
        if !self.layout.overflows() {
            self.wheel = 0;
            return;
        }
        let towards_last = if horizontal {
            i64::from(delta)
        } else {
            -i64::from(delta)
        };
        let turn = self.wheel + towards_last * i64::from(self.layout.tab_width());
        self.wheel = turn % WHEEL_STEP;
        let pixels = turn / WHEEL_STEP;
        if pixels != 0 {
            let side = if pixels < 0 { Side::Left } else { Side::Right };
            self.scroll(tabs, side, |layout| layout.scroll_by(pixels));
        }
    }

    /// Scrolls the tabs towards `side` as `scroll` says, which returns
    /// whether they scrolled, as this does. Nothing changes if the strip, as
    /// it is drawn, cannot scroll towards `side`. Otherwise the user scrolls
    /// it: the tabs that kept their width get it first, and the tab that is
    /// shown no longer has to be in view.
    fn scroll<T>(
        &mut self,
        tabs: &model::Window<T>,
        side: Side,
        scroll: impl FnOnce(&mut Layout) -> bool,
    ) -> bool {
        if !self.layout.can_scroll(side) {
            return false;
        }
        self.thaw(tabs);
        self.reveal = false;
        let scrolled = scroll(&mut self.layout);
        if scrolled {
            self.relayout(tabs);
        }
        scrolled
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
    /// tabs of `tabs` that the strip shows, cut to it, the active one with
    /// the colours of the terminal, `palette`, and the buttons. In the tab
    /// at `editing.0`, if a name is edited, its field, `editing.1`, goes in
    /// the place of the title.
    pub fn draw<T: Label>(
        &mut self,
        canvas: &mut Canvas<'_>,
        tabs: &model::Window<T>,
        palette: &Palette,
        editing: Option<(usize, View<'_>)>,
    ) {
        let clip = canvas.clip();
        if !clip.intersects(self.layout.bar()) {
            return;
        }
        canvas.fill(self.layout.bar(), BAR);
        let active = tabs.active_index();
        let mut strip = canvas.clipped(self.layout.strip());
        let clip = strip.clip();
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
                let style = Style {
                    look,
                    separator: after_plain && look == Look::Plain,
                    field: editing
                        .as_ref()
                        .filter(|(edited, _)| *edited == index)
                        .map(|(_, field)| field),
                };
                self.draw_tab(&mut strip, index, tab, &style, palette);
            }
            after_plain = look == Look::Plain;
        }
        let clip = canvas.clip();
        for side in [Side::Left, Side::Right] {
            if self
                .layout
                .scroll_button(side)
                .is_some_and(|button| clip.intersects(button))
            {
                self.draw_scroll(canvas, side);
            }
        }
        if clip.intersects(self.layout.new_tab()) {
            self.draw_new(canvas);
        }
        if clip.intersects(self.layout.profiles()) {
            self.draw_profiles(canvas);
        }
        if self.frame.buttons {
            for button in CaptionButton::ALL {
                if clip.intersects(self.layout.caption(button)) {
                    self.draw_caption(canvas, button);
                }
            }
        }
    }

    /// Draws the tab at `index` as `style` says, with its indicator, if it
    /// has one, before its title, or before the field where its name is
    /// edited.
    fn draw_tab<T: Label>(
        &mut self,
        canvas: &mut Canvas<'_>,
        index: usize,
        tab: &model::Tab<T>,
        style: &Style<'_>,
        palette: &Palette,
    ) {
        let layout = self.layout;
        let (background, color) = match style.look {
            Look::Active => (palette.background, palette.foreground),
            Look::Hovered => (HOVERED_TAB, HOVERED_TEXT),
            Look::Plain => (BAR, TEXT),
        };
        let rect = layout.tab(index);
        if style.look != Look::Plain {
            canvas.fill(rect, background);
        }
        if style.separator
            && let Some(line) = layout.separator(index)
        {
            canvas.fill(line, SEPARATOR);
        }

        let cell = self.font.cell();
        // A cell is a few dozen pixels.
        let (width, height) = (cell.width.max(1) as i32, cell.height as i32);
        let top = rect.top + (rect.height() as i32 - height) / 2;
        let (title, indicator) = self.title(index, tab.content().indicator());
        if let Some((indicator, slot)) = indicator {
            self.draw_indicator(canvas, indicator, slot, top, color, background);
        }
        let at = Point::new(title.left, top);
        if let Some(field) = style.field {
            // In the part of the title that the strip shows, where the clicks
            // of the field go: see `text_area`.
            let area = title.intersection(layout.strip());
            let selection = palette.selection;
            let canvas = &mut canvas.clipped(area);
            let at = Point::new(area.left, top);
            self.draw_field(canvas, at, field, color, background, selection);
        } else {
            text::fit(
                tab.content().label(),
                columns(title, width),
                &mut self.cells,
            );
            self.font.draw(
                &mut canvas.clipped(title),
                at,
                &self.cells,
                FontStyle::REGULAR,
                color,
                background,
            );
        }

        let id = tab.id();
        if let Some(highlight) = layout.close_highlight(index)
            && let Some(fill) = self.button_color(Target::Close(id), Press::Close(id))
        {
            canvas.fill(highlight, fill);
        }
        if let Some(cross) = layout.cross(index) {
            draw_diagonals(canvas, cross, layout.stroke(), color);
        }
    }

    /// Where the title of the tab at `index` goes, after `indicator`, if the
    /// tab has one, and the slot of the indicator: see [`split_title`].
    fn title(
        &self,
        index: usize,
        indicator: Option<Indicator>,
    ) -> (Rect, Option<(Indicator, Rect)>) {
        // A cell is a few dozen pixels.
        let width = self.font.cell().width.max(1) as i32;
        split_title(self.layout.title(index), width, indicator)
    }

    /// Draws `field`, where the name of a tab is edited, from `at`, the
    /// top-left corner of its first cell, as the title of the tab, with
    /// `color` on `background`, but its selected cells on `selection`, the
    /// background of the selected cells of the terminal; and its caret,
    /// while it is shown, a bar of `color` on the left side of its cell.
    fn draw_field(
        &mut self,
        canvas: &mut Canvas<'_>,
        at: Point,
        field: &View<'_>,
        color: Rgb,
        background: Rgb,
        selection: Rgb,
    ) {
        let cell = self.font.cell();
        let x = |column: usize| at.x + column as i32 * cell.width as i32;
        let cells = field.cells.len();
        let selected = field.shown.selection.start.min(cells)..field.shown.selection.end.min(cells);
        let runs = [
            (0..selected.start, background),
            (selected.clone(), selection),
            (selected.end..cells, background),
        ];
        for (run, fill) in runs {
            if !run.is_empty() {
                let from = Point::new(x(run.start), at.y);
                let style = FontStyle::REGULAR;
                self.font
                    .draw(canvas, from, &field.cells[run], style, color, fill);
            }
        }
        if let Some(thickness) = field.caret {
            let left = x(field.shown.caret);
            let bar = Rect::new(
                left,
                at.y,
                left + thickness as i32,
                at.y + cell.height as i32,
            );
            canvas.fill(bar, color);
        }
    }

    /// Draws `indicator` in `slot`, its cells before the title of a tab
    /// whose title has `color` on `background`, with its text at `top`.
    fn draw_indicator(
        &mut self,
        canvas: &mut Canvas<'_>,
        indicator: Indicator,
        slot: Rect,
        top: i32,
        color: Rgb,
        background: Rgb,
    ) {
        let (symbol, ink) = match indicator {
            Indicator::Ended => {
                // A heavy cross, drawn as the one of the close button: in
                // the fonts of Windows, the glyph of ✖ is wider than a cell.
                let side = slot.width().min(slot.height());
                let (left, top) = (
                    slot.left + (slot.width() - side) as i32 / 2,
                    slot.top + (slot.height() - side) as i32 / 2,
                );
                let square = Rect::new(left, top, left + side as i32, top + side as i32);
                // About 2 pixels in 7.
                let stroke = ((2 * side + 3) / 7).max(1);
                draw_diagonals(canvas, square, stroke, ENDED);
                return;
            }
            Indicator::Bell => ("🔔", BELL),
            Indicator::Activity => ("●", color),
        };
        text::fit(symbol, indicator.cells(), &mut self.cells);
        self.font.draw(
            &mut canvas.clipped(slot),
            Point::new(slot.left, top),
            &self.cells,
            FontStyle::REGULAR,
            ink,
            background,
        );
    }

    /// Draws the button that scrolls the tabs towards `side`, which is
    /// darker, without highlight, while they cannot scroll so.
    fn draw_scroll(&self, canvas: &mut Canvas<'_>, side: Side) {
        let (Some(highlight), Some(chevron)) = (
            self.layout.scroll_highlight(side),
            self.layout.scroll_chevron(side),
        ) else {
            return;
        };
        let target = Target::Scroll(side);
        let color = if !self.layout.can_scroll(side) {
            DISABLED_TEXT
        } else {
            if let Some(fill) = self.button_color(target, Press::Scroll(side)) {
                canvas.fill(highlight, fill);
            }
            if self.hover == Some(target) {
                HOVERED_TEXT
            } else {
                TEXT
            }
        };
        draw_side_chevron(canvas, chevron, self.layout.stroke(), color, side);
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

    /// Draws the button that opens the menu of the profiles.
    fn draw_profiles(&self, canvas: &mut Canvas<'_>) {
        if let Some(fill) = self.button_color(Target::Profiles, Press::Profiles) {
            canvas.fill(self.layout.profiles_highlight(), fill);
        }
        let color = if self.hover == Some(Target::Profiles) {
            HOVERED_TEXT
        } else {
            TEXT
        };
        draw_diagonals(canvas, self.layout.chevron(), self.layout.stroke(), color);
    }

    /// Draws the button `button` of the window: its highlight, as big as
    /// the button, and its glyph, restore while the window is maximized.
    fn draw_caption(&self, canvas: &mut Canvas<'_>, button: CaptionButton) {
        let target = Target::Caption(button);
        let pressed = self.highlight(target, Press::Caption(button));
        let fill = pressed.map(|pressed| match (button, pressed) {
            (CaptionButton::Close, true) => CLOSE_PRESSED,
            (CaptionButton::Close, false) => CLOSE_HOVERED,
            (_, true) => BUTTON_PRESSED,
            (_, false) => BUTTON_HOVERED,
        });
        if let Some(fill) = fill {
            canvas.fill(self.layout.caption(button), fill);
        }
        let color = match (button, pressed) {
            (CaptionButton::Close, Some(true)) => CLOSE_PRESSED_GLYPH,
            (CaptionButton::Close, Some(false)) => CLOSE_GLYPH,
            _ if self.hover == Some(target) => HOVERED_TEXT,
            _ if self.focused => TEXT,
            _ => INACTIVE_GLYPH,
        };
        let (glyph, stroke) = (self.layout.glyph(button), self.layout.stroke());
        match button {
            CaptionButton::Minimize => draw_minimize(canvas, glyph, stroke, color),
            CaptionButton::Maximize if self.frame.maximized => {
                draw_restore(canvas, glyph, self.layout.glyph_offset(), stroke, color);
            }
            CaptionButton::Maximize => draw_outline(canvas, glyph, stroke, color),
            CaptionButton::Close => draw_diagonals(canvas, glyph, stroke, color),
        }
    }

    /// The colour of the highlight of the button `target`, which `press`
    /// presses, if it has one: while the pointer is on it, unless another
    /// button is down.
    fn button_color(&self, target: Target, press: Press) -> Option<Rgb> {
        self.highlight(target, press).map(|pressed| {
            if pressed {
                BUTTON_PRESSED
            } else {
                BUTTON_HOVERED
            }
        })
    }

    /// Whether the button `target`, which `press` presses, is highlighted,
    /// and if so, whether it is pressed: while the pointer is on it, unless
    /// another button is down.
    fn highlight(&self, target: Target, press: Press) -> Option<bool> {
        if self.hover != Some(target) {
            return None;
        }
        match self.pressed {
            None => Some(false),
            Some(pressed) => (pressed == press).then_some(true),
        }
    }

    /// The part of the bar at `at` that the pointer highlights, if any.
    fn target_at<T>(&self, tabs: &model::Window<T>, at: Point) -> Option<Target> {
        let id = |index: usize| tabs.tabs().get(index).map(model::Tab::id);
        match self.layout.part_at(at)? {
            Part::Tab(index) => id(index).map(Target::Tab),
            Part::Close(index) => id(index).map(Target::Close),
            Part::New => Some(Target::New),
            Part::Profiles => Some(Target::Profiles),
            // A button that cannot scroll the tabs is not highlighted, and a
            // press on it does nothing.
            Part::Scroll(side) => self.layout.can_scroll(side).then_some(Target::Scroll(side)),
            // Without the title bar of meiga, the window has its own buttons,
            // and the bar keeps their room empty.
            Part::Caption(button) => self.frame.buttons.then_some(Target::Caption(button)),
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
                Target::Profiles => self.damage(self.layout.profiles()),
                Target::Scroll(side) => self.damage_scroll(side),
                Target::Caption(button) => self.damage(self.layout.caption(button)),
                Target::Tab(_) => {}
            }
        }
    }

    /// `press` began or ended: the button that it presses changes.
    fn damage_press<T>(&mut self, tabs: &model::Window<T>, press: Press) {
        match press {
            Press::Close(id) => self.damage_close(tabs, id),
            Press::New => self.damage(self.layout.new_tab()),
            Press::Profiles => self.damage(self.layout.profiles()),
            Press::Scroll(side) => self.damage_scroll(side),
            Press::Caption(button) => self.damage(self.layout.caption(button)),
            Press::Middle(_) | Press::Right(_) => {}
        }
    }

    fn damage_close<T>(&mut self, tabs: &model::Window<T>, id: TabId) {
        if let Some(close) = tabs.index_of(id).and_then(|index| self.layout.close(index)) {
            self.damage(close.intersection(self.layout.strip()));
        }
    }

    /// The button that scrolls the tabs towards `side` changes, if they can
    /// scroll so: one that cannot looks the same whatever the pointer and
    /// the buttons do, and changes only with the layout, which draws all the
    /// bar again.
    fn damage_scroll(&mut self, side: Side) {
        if !self.layout.can_scroll(side) {
            return;
        }
        if let Some(button) = self.layout.scroll_button(side) {
            self.damage(button);
        }
    }

    /// Tabs that kept their width get it again, where the pointer may be on
    /// another part of the bar.
    fn thaw<T>(&mut self, tabs: &model::Window<T>) {
        if mem::take(&mut self.frozen) {
            self.layout.resize(self.layout.width(), tabs.tab_count());
            self.relayout(tabs);
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

/// How many whole cells `width` pixels wide fit in `area`.
fn columns(area: Rect, width: i32) -> usize {
    (area.width() as i32 / width) as usize
}

/// Where a title goes in `title`, the area of the title of a tab, with cells
/// `width` pixels wide: after `indicator`, if the tab has one, and the area
/// has room for it and a blank cell more. Returns too, then, the slot of the
/// indicator, its cells before the blank one.
fn split_title(
    mut title: Rect,
    width: i32,
    indicator: Option<Indicator>,
) -> (Rect, Option<(Indicator, Rect)>) {
    let fits = |indicator: &Indicator| indicator.cells() < columns(title, width);
    let Some(indicator) = indicator.filter(fits) else {
        return (title, None);
    };
    let right = title.left + indicator.cells() as i32 * width;
    let slot = Rect::new(title.left, title.top, right, title.bottom);
    // And a blank cell between it and the title.
    title.left = right + width;
    (title, Some((indicator, slot)))
}

/// Draws the lines from the two top corners of `rect` down at 45 degrees to
/// its bottom, `stroke` pixels thick, as steps of a pixel: at 45 degrees
/// they are as sharp as straight ones. In a square, they make a cross; in a
/// rectangle twice as wide as it is high, a chevron that points down.
fn draw_diagonals(canvas: &mut Canvas<'_>, rect: Rect, stroke: u32, color: Rgb) {
    // A cross or a chevron is a few pixels.
    let (height, stroke) = (rect.height() as i32, stroke as i32);
    for step in 0..=height - stroke {
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

/// Draws a chevron that points towards `side` in `rect`, which is twice as
/// high as it is wide: the lines from the two corners of its other side to
/// the middle of this one, as [`draw_diagonals`] draws them down.
fn draw_side_chevron(canvas: &mut Canvas<'_>, rect: Rect, stroke: u32, color: Rgb, side: Side) {
    // A chevron is a few pixels.
    let (width, stroke) = (rect.width() as i32, stroke as i32);
    for step in 0..=width - stroke {
        let left = match side {
            Side::Left => rect.right - step - stroke,
            Side::Right => rect.left + step,
        };
        let right = left + stroke;
        canvas.fill(
            Rect::new(left, rect.top + step, right, rect.top + step + stroke),
            color,
        );
        canvas.fill(
            Rect::new(left, rect.bottom - step - stroke, right, rect.bottom - step),
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

/// Draws the glyph of minimize in the square `rect`: a line across its
/// middle, `stroke` pixels thick.
fn draw_minimize(canvas: &mut Canvas<'_>, rect: Rect, stroke: u32, color: Rgb) {
    // A glyph is a few pixels.
    let stroke = stroke as i32;
    let middle = rect.top + (rect.height() as i32 - stroke) / 2;
    canvas.fill(
        Rect::new(rect.left, middle, rect.right, middle + stroke),
        color,
    );
}

/// Draws the sides of `rect`, inside it, `stroke` pixels thick: the glyph
/// of maximize.
fn draw_outline(canvas: &mut Canvas<'_>, rect: Rect, stroke: u32, color: Rgb) {
    // A glyph is a few pixels.
    let stroke = stroke as i32;
    let (top, bottom) = (rect.top + stroke, rect.bottom - stroke);
    canvas.fill(Rect::new(rect.left, rect.top, rect.right, top), color);
    canvas.fill(Rect::new(rect.left, bottom, rect.right, rect.bottom), color);
    canvas.fill(Rect::new(rect.left, top, rect.left + stroke, bottom), color);
    canvas.fill(
        Rect::new(rect.right - stroke, top, rect.right, bottom),
        color,
    );
}

/// Draws the glyph of restore in the square `rect`, with lines `stroke`
/// pixels thick: a square in front, in its bottom left corner, and another
/// one behind it, `offset` pixels up and to the right, without what the
/// one in front hides.
fn draw_restore(canvas: &mut Canvas<'_>, rect: Rect, offset: u32, stroke: u32, color: Rgb) {
    // A glyph is a few pixels.
    let (shift, thick) = (offset as i32, stroke as i32);
    let front = Rect::new(rect.left, rect.top + shift, rect.right - shift, rect.bottom);
    let back = Rect::new(rect.left + shift, rect.top, rect.right, rect.bottom - shift);
    draw_outline(canvas, front, stroke, color);
    // The top and the right side of the one behind, and what shows of its
    // left side above the one in front and of its bottom on its right.
    canvas.fill(
        Rect::new(back.left, back.top, back.right, back.top + thick),
        color,
    );
    canvas.fill(
        Rect::new(back.right - thick, back.top, back.right, back.bottom),
        color,
    );
    canvas.fill(
        Rect::new(back.left, back.top, back.left + thick, front.top),
        color,
    );
    canvas.fill(
        Rect::new(front.right, back.bottom - thick, back.right, back.bottom),
        color,
    );
}

#[cfg(test)]
mod tests {
    use meiga_core::ui::field::Shown;
    use meiga_render::pixel;

    use super::*;

    impl Label for String {
        fn label(&self) -> &str {
            self
        }
    }

    /// A maximized window with the title bar of meiga: the bar has the
    /// buttons of the window, and no band above the tabs.
    const MAXIMIZED: Frame = Frame {
        buttons: true,
        band: 0,
        maximized: true,
    };

    /// The tabs of `programs`, the first one active, in a window of 983
    /// pixels at 100 %, maximized, with the title bar of meiga, and their
    /// bar: the buttons of the window on its right, from 845, and 736 pixels
    /// for the tabs, as the 800 of the bar of 2.8, which had no buttons of
    /// the window nor room to drag it. The window does not have the focus.
    fn bar(programs: &[&str]) -> (TabBar, model::Window<String>) {
        let mut tabs = model::Window::new(programs[0].to_owned());
        for program in &programs[1..] {
            tabs.open((*program).to_owned());
        }
        let fonts = FontSystem::new().unwrap();
        let bar = TabBar::new(&fonts, 96, 983, tabs.tab_count(), MAXIMIZED).unwrap();
        (bar, tabs)
    }

    fn three() -> (TabBar, model::Window<String>) {
        bar(&["pwsh", "cmd", "wsl"])
    }

    /// `count` tabs, the first one active, in a window of 983 pixels at
    /// 100 %. From 8 on, they do not fit even at their narrowest, 100
    /// pixels: the strip, from 28 to 716, is between the buttons that scroll
    /// it, from 4 to 28 and from 716 to 740.
    fn many(count: usize) -> (TabBar, model::Window<String>) {
        let names: Vec<String> = (1..=count).map(|tab| tab.to_string()).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        bar(&names)
    }

    fn damage(bar: &mut TabBar) -> Vec<Rect> {
        let mut areas = Vec::new();
        bar.take_damage(&mut |area| areas.push(area));
        areas
    }

    /// `button` goes down where the pointer is, in a press of its own: it
    /// is never the second click of a double click.
    fn down<T>(bar: &mut TabBar, tabs: &model::Window<T>, button: Button) -> Option<TabAction> {
        bar.clicks.reset();
        bar.down(tabs, button, Instant::now())
    }

    fn id<T>(tabs: &model::Window<T>, index: usize) -> TabId {
        tabs.tabs()[index].id()
    }

    /// The left button goes down and up on the close button under the
    /// pointer, which is the one of the tab at `index`, and the tab closes.
    fn close_with_mouse(bar: &mut TabBar, tabs: &mut model::Window<String>, index: usize) {
        let tab = id(tabs, index);
        assert_eq!(down(bar, tabs, Button::Left), None);
        assert_eq!(bar.up(tabs, Button::Left), Some(TabAction::CloseTab(tab)));
        assert!(tabs.close(tab).is_ok());
        bar.tab_closed(tabs, index);
    }

    /// The whole bar, as its damage says when it is drawn again.
    const WHOLE: Rect = Rect::new(0, 0, 983, 32);

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
        // The button that opens a tab, the one that opens the menu of the
        // profiles, joined to it, and the empty bar.
        assert!(bar.moved(&tabs, Point::new(740, 10)));
        assert_eq!(damage(&mut bar), [Rect::new(728, 4, 756, 32)]);
        bar.moved(&tabs, Point::new(770, 10));
        assert_eq!(
            damage(&mut bar),
            [Rect::new(728, 4, 756, 32), Rect::new(756, 4, 780, 32)]
        );
        bar.moved(&tabs, Point::new(780, 10));
        assert_eq!(damage(&mut bar), [Rect::new(756, 4, 780, 32)]);
        assert!(!bar.moved(&tabs, Point::new(790, 10)));
        assert!(!bar.has_damage());
        // The menu of the profiles opens below both.
        assert_eq!(bar.buttons(), Rect::new(728, 4, 780, 32));
    }

    #[test]
    fn the_left_button_shows_a_tab_when_it_goes_down() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(300, 20));
        let action = down(&mut bar, &tabs, Button::Left);
        assert_eq!(action, Some(TabAction::Activate(id(&tabs, 1))));
        assert!(!bar.is_pressed());
        assert_eq!(bar.up(&tabs, Button::Left), None);
        // The empty bar does nothing: Windows drags the window from it.
        bar.moved(&tabs, Point::new(790, 20));
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert_eq!(down(&mut bar, &tabs, Button::Right), None);
        assert!(!bar.is_pressed());
    }

    #[test]
    fn a_double_click_on_a_tab_edits_its_name() {
        // The second tab goes from 244 to 484, after the first one.
        let (mut bar, tabs) = three();
        let (first, second) = (id(&tabs, 0), id(&tabs, 1));
        let start = Instant::now();
        let ms = Duration::from_millis(1);
        let click = |bar: &mut TabBar, x: i32, after: u32| {
            bar.moved(&tabs, Point::new(x, 20));
            bar.down(&tabs, Button::Left, start + after * ms)
        };
        assert_eq!(click(&mut bar, 300, 0), Some(TabAction::Activate(second)));
        assert_eq!(click(&mut bar, 300, 400), Some(TabAction::Rename(second)));
        // As in Windows, a third click is a first one again, and a fourth
        // one a second one.
        assert_eq!(click(&mut bar, 300, 500), Some(TabAction::Activate(second)));
        assert_eq!(click(&mut bar, 300, 600), Some(TabAction::Rename(second)));
        // So is a click after the time of a double click, 500 ms by default.
        assert_eq!(
            click(&mut bar, 300, 1200),
            Some(TabAction::Activate(second))
        );
        assert_eq!(
            click(&mut bar, 300, 1800),
            Some(TabAction::Activate(second))
        );
        assert_eq!(click(&mut bar, 300, 1900), Some(TabAction::Rename(second)));
        // Near the last click, but on another tab: for Windows, a double
        // click all the same, after which a click is a first one.
        assert_eq!(click(&mut bar, 243, 3000), Some(TabAction::Activate(first)));
        assert_eq!(
            click(&mut bar, 245, 3100),
            Some(TabAction::Activate(second))
        );
        assert_eq!(
            click(&mut bar, 245, 3200),
            Some(TabAction::Activate(second))
        );
        assert_eq!(click(&mut bar, 245, 3300), Some(TabAction::Rename(second)));
        // Another press between them: as in Windows, no double click.
        assert_eq!(
            click(&mut bar, 300, 5000),
            Some(TabAction::Activate(second))
        );
        bar.down(&tabs, Button::Right, start + 5100 * ms);
        bar.up(&tabs, Button::Right);
        assert_eq!(
            click(&mut bar, 300, 5200),
            Some(TabAction::Activate(second))
        );
        // Far from the first click of the pair: neither.
        assert_eq!(
            click(&mut bar, 303, 5300),
            Some(TabAction::Activate(second))
        );
        // Nor with a press that the bar does not get between them, as on
        // the terminal.
        assert_eq!(
            click(&mut bar, 300, 6000),
            Some(TabAction::Activate(second))
        );
        bar.forget_clicks();
        assert_eq!(
            click(&mut bar, 300, 6100),
            Some(TabAction::Activate(second))
        );
        // With the double-click time of Windows at 100 ms, two clicks 200 ms
        // apart are two clicks.
        bar.set_click_settings(ClickSettings {
            time: 100 * ms,
            ..ClickSettings::default()
        });
        let activate = Some(TabAction::Activate(second));
        assert_eq!(click(&mut bar, 300, 7000), activate);
        assert_eq!(click(&mut bar, 300, 7200), activate);
        assert_eq!(click(&mut bar, 300, 7250), Some(TabAction::Rename(second)));
    }

    #[test]
    fn buttons_act_when_the_left_button_goes_up_on_them() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(460, 20));
        damage(&mut bar);
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert!(bar.is_pressed());
        assert_eq!(damage(&mut bar), [Rect::new(456, 6, 480, 30)]);
        // Other buttons do nothing meanwhile.
        assert_eq!(down(&mut bar, &tabs, Button::Middle), None);
        assert_eq!(bar.up(&tabs, Button::Middle), None);
        assert!(bar.is_pressed());
        let action = bar.up(&tabs, Button::Left);
        assert_eq!(action, Some(TabAction::CloseTab(id(&tabs, 1))));
        assert!(!bar.is_pressed());
        assert_eq!(damage(&mut bar), [Rect::new(456, 6, 480, 30)]);

        // Moving away takes the click back.
        bar.moved(&tabs, Point::new(740, 10));
        down(&mut bar, &tabs, Button::Left);
        bar.moved(&tabs, Point::new(740, 100));
        assert_eq!(bar.up(&tabs, Button::Left), None);
        assert!(!bar.is_pressed());
        // And coming back gives it again.
        bar.moved(&tabs, Point::new(740, 10));
        down(&mut bar, &tabs, Button::Left);
        bar.moved(&tabs, Point::new(300, 10));
        bar.moved(&tabs, Point::new(745, 25));
        assert_eq!(bar.up(&tabs, Button::Left), Some(TabAction::New));

        // The button that opens the menu of the profiles, which is not the
        // one that opens a tab, although they are joined.
        bar.moved(&tabs, Point::new(770, 10));
        damage(&mut bar);
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert_eq!(damage(&mut bar), [Rect::new(756, 4, 780, 32)]);
        bar.moved(&tabs, Point::new(750, 10));
        assert_eq!(bar.up(&tabs, Button::Left), None);
        bar.moved(&tabs, Point::new(760, 10));
        down(&mut bar, &tabs, Button::Left);
        let action = bar.up(&tabs, Button::Left);
        assert_eq!(action, Some(TabAction::ProfileMenu { keyboard: false }));
        bar.moved(&tabs, Point::new(750, 10));
        down(&mut bar, &tabs, Button::Left);
        bar.moved(&tabs, Point::new(760, 10));
        assert_eq!(bar.up(&tabs, Button::Left), None);
    }

    #[test]
    fn the_middle_button_closes_the_tab_where_it_goes_down_and_up() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(100, 20));
        assert_eq!(down(&mut bar, &tabs, Button::Middle), None);
        assert!(bar.is_pressed());
        // Its close button is part of the tab.
        bar.moved(&tabs, Point::new(220, 20));
        let action = bar.up(&tabs, Button::Middle);
        assert_eq!(action, Some(TabAction::CloseTab(id(&tabs, 0))));
        // Not on another tab.
        bar.moved(&tabs, Point::new(300, 20));
        down(&mut bar, &tabs, Button::Middle);
        bar.moved(&tabs, Point::new(100, 20));
        assert_eq!(bar.up(&tabs, Button::Middle), None);
        // Not on the button that opens a tab.
        bar.moved(&tabs, Point::new(740, 20));
        assert_eq!(down(&mut bar, &tabs, Button::Middle), None);
        assert!(!bar.is_pressed());
    }

    #[test]
    fn the_right_button_opens_the_menu_of_the_tab_where_it_goes_down_and_up() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(300, 20));
        damage(&mut bar);
        // It does not show the tab, and draws nothing.
        assert_eq!(down(&mut bar, &tabs, Button::Right), None);
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
        down(&mut bar, &tabs, Button::Right);
        bar.moved(&tabs, Point::new(300, 20));
        assert_eq!(bar.up(&tabs, Button::Right), None);
        bar.moved(&tabs, Point::new(100, 20));
        down(&mut bar, &tabs, Button::Right);
        bar.moved(&tabs, Point::new(100, 40));
        assert_eq!(bar.up(&tabs, Button::Right), None);
        // Not on the buttons that open a tab and the menu of the profiles.
        for x in [740, 770] {
            bar.moved(&tabs, Point::new(x, 20));
            assert_eq!(down(&mut bar, &tabs, Button::Right), None);
            assert!(!bar.is_pressed());
        }
    }

    #[test]
    fn losing_the_mouse_takes_the_click_back() {
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(220, 20));
        down(&mut bar, &tabs, Button::Left);
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
        assert_eq!(damage(&mut bar), [WHOLE]);
        // The pointer is on the tab that took the place of the closed one.
        assert_eq!(bar.hover, Some(Target::Tab(id(&tabs, 1))));
        bar.label_changed(1);
        bar.label_changed(1);
        assert_eq!(damage(&mut bar), [Rect::new(252, 4, 456, 32)]);
        // The same width is no change; another one is.
        bar.resize(983, &tabs);
        assert!(!bar.has_damage());
        bar.resize(783, &tabs);
        assert_eq!(damage(&mut bar), [Rect::new(0, 0, 783, 32)]);
        bar.set_dpi(144, &tabs);
        assert_eq!(bar.height(), 48);
        assert_eq!(damage(&mut bar), [Rect::new(0, 0, 783, 48)]);
    }

    #[test]
    fn the_wheel_scrolls_a_tab_for_each_step_while_the_tabs_do_not_fit() {
        // While they fit, it does nothing.
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(300, 20));
        damage(&mut bar);
        bar.wheel(&tabs, -120, false);
        assert!(!bar.has_damage());

        // 112 pixels of the row of tabs do not fit in the strip.
        let (mut bar, tabs) = many(8);
        assert_eq!(bar.layout.max_scroll(), 112);
        bar.moved(&tabs, Point::new(250, 20));
        damage(&mut bar);
        // Backward, a tab towards the last one, which draws all the bar,
        // and the pointer is on the next tab.
        bar.wheel(&tabs, -120, false);
        assert_eq!(bar.layout.scroll(), 100);
        assert_eq!(damage(&mut bar), [WHOLE]);
        assert!(bar.tab(0).is_empty());
        assert_eq!(bar.tab(1), Rect::new(28, 4, 128, 32));
        assert_eq!(bar.hover, Some(Target::Tab(id(&tabs, 3))));
        // As far as they can, and then nothing.
        bar.wheel(&tabs, -120, false);
        assert_eq!(bar.layout.scroll(), 112);
        damage(&mut bar);
        bar.wheel(&tabs, -120, false);
        assert!(!bar.has_damage());
        // Forward, towards the first one.
        bar.wheel(&tabs, 120, false);
        assert_eq!(bar.layout.scroll(), 12);
        bar.wheel(&tabs, 120, false);
        assert_eq!(bar.layout.scroll(), 0);
        // A smooth wheel turns less than a step at a time: what does not
        // make a whole pixel waits for the next turn.
        bar.wheel(&tabs, -50, false);
        assert_eq!(bar.layout.scroll(), 41);
        bar.wheel(&tabs, -70, false);
        assert_eq!(bar.layout.scroll(), 100);
        // To the sides: to the left, towards the first one.
        bar.wheel(&tabs, -120, true);
        assert_eq!(bar.layout.scroll(), 0);
        bar.wheel(&tabs, 120, true);
        assert_eq!(bar.layout.scroll(), 100);
    }

    #[test]
    fn the_buttons_on_the_sides_scroll_a_tab_and_again_while_they_stay_down() {
        // 312 pixels do not fit.
        let (mut bar, tabs) = many(10);
        let now = Instant::now();
        let after = |milliseconds| now + Duration::from_millis(milliseconds);
        // Towards the first tab, they cannot scroll: the pointer on the
        // button highlights nothing, and the press does nothing.
        bar.moved(&tabs, Point::new(10, 20));
        assert_eq!(bar.hover, None);
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert!(!bar.is_pressed());
        // Towards the last one, until the first tab that the strip hides,
        // even in part, is all in it.
        bar.moved(&tabs, Point::new(720, 20));
        damage(&mut bar);
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert!(bar.is_pressed());
        assert_eq!(bar.layout.scroll(), 12);
        assert_eq!(damage(&mut bar), [WHOLE]);
        // The first frame says when they scroll again.
        assert_eq!(bar.tick(&tabs, now), Some(after(250)));
        assert_eq!(bar.tick(&tabs, after(249)), Some(after(250)));
        assert_eq!(bar.layout.scroll(), 12);
        // Not while the pointer is out of the button.
        bar.moved(&tabs, Point::new(700, 20));
        assert_eq!(bar.tick(&tabs, after(300)), None);
        assert_eq!(bar.layout.scroll(), 12);
        // Back on it, at once, and then every 100 ms, until they cannot
        // scroll further.
        bar.moved(&tabs, Point::new(720, 20));
        assert_eq!(bar.tick(&tabs, after(400)), Some(after(500)));
        assert_eq!(bar.layout.scroll(), 112);
        assert_eq!(bar.tick(&tabs, after(450)), Some(after(500)));
        assert_eq!(bar.tick(&tabs, after(500)), Some(after(600)));
        assert_eq!(bar.layout.scroll(), 212);
        assert_eq!(bar.tick(&tabs, after(600)), None);
        assert_eq!(bar.layout.scroll(), 312);
        // The button goes up: nothing else, and nothing is drawn, since the
        // button that cannot scroll looks the same.
        damage(&mut bar);
        assert_eq!(bar.up(&tabs, Button::Left), None);
        assert!(!bar.is_pressed());
        assert!(!bar.has_damage());
        assert_eq!(bar.tick(&tabs, after(700)), None);

        // Towards the first one, until the tab on the left of the first one
        // that is all in the strip is all in it too.
        bar.moved(&tabs, Point::new(10, 20));
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert_eq!(bar.layout.scroll(), 300);
        assert!(bar.tick(&tabs, after(1000)).is_some());
        // Losing the mouse ends the press.
        bar.capture_lost(&tabs);
        assert!(!bar.is_pressed());
        assert_eq!(bar.tick(&tabs, after(2000)), None);
        assert_eq!(bar.layout.scroll(), 300);
    }

    #[test]
    fn what_cannot_scroll_the_strip_changes_nothing() {
        // Ten tabs, scrolled to the end, and the sixth closed with the mouse,
        // with its close button from (288, 6) to (312, 30): the others keep
        // their width, and the strip its scroll, past the end of the row.
        let (mut bar, mut tabs) = many(10);
        bar.moved(&tabs, Point::new(300, 20));
        bar.wheel(&tabs, -4 * 120, false);
        assert_eq!(bar.layout.scroll(), 312);
        close_with_mouse(&mut bar, &mut tabs, 5);
        assert!(bar.frozen);
        // The pointer on the button that scrolls the strip to the right,
        // which cannot, draws nothing, and the left button there does
        // nothing.
        bar.moved(&tabs, Point::new(742, 20));
        damage(&mut bar);
        bar.moved(&tabs, Point::new(720, 20));
        assert_eq!(bar.hover, None);
        assert!(!bar.has_damage());
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert!(!bar.is_pressed());
        // Nor the wheel towards the last tab.
        bar.wheel(&tabs, -120, false);
        assert!(bar.frozen);
        assert!(!bar.has_damage());
        // Nor, towards the first one, while the middle button is down on a
        // tab: the middle button is the wheel.
        bar.moved(&tabs, Point::new(250, 20));
        assert_eq!(down(&mut bar, &tabs, Button::Middle), None);
        bar.wheel(&tabs, 120, false);
        assert_eq!(bar.layout.scroll(), 312);
        assert!(bar.frozen);
        bar.moved(&tabs, Point::new(742, 20));
        assert_eq!(bar.up(&tabs, Button::Middle), None);
        // No one scrolled the strip: once the pointer leaves the bar, the
        // tabs get their width, and the strip shows the tab that is shown,
        // the first one.
        bar.left(&tabs);
        bar.settle(&tabs);
        assert_eq!(bar.layout.scroll(), 0);

        // A button that the strip, as it is drawn, can scroll lets the tabs
        // get their width first, also if then they fit, and it scrolls
        // nothing: eight tabs, and the close button of the third, from
        // (300, 6) to (324, 30), closes it.
        let (mut bar, mut tabs) = many(8);
        bar.moved(&tabs, Point::new(310, 20));
        close_with_mouse(&mut bar, &mut tabs, 2);
        assert!(bar.layout.can_scroll(Side::Right));
        bar.moved(&tabs, Point::new(720, 20));
        assert_eq!(bar.hover, Some(Target::Scroll(Side::Right)));
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert!(!bar.frozen);
        assert!(!bar.layout.overflows());
        assert_eq!(bar.layout.tab_width(), 105);
    }

    #[test]
    fn tabs_closed_with_the_mouse_keep_their_width_until_the_pointer_leaves_the_bar() {
        // Six tabs of 122 pixels: the close button of the third goes from
        // (342, 6) to (366, 30).
        let (mut bar, mut tabs) = many(6);
        assert_eq!(bar.layout.tab_width(), 122);
        bar.moved(&tabs, Point::new(350, 20));
        close_with_mouse(&mut bar, &mut tabs, 2);
        // The next tab takes its place, with its close button under the
        // pointer.
        assert_eq!(bar.layout.tab_width(), 122);
        assert_eq!(bar.hover, Some(Target::Close(id(&tabs, 2))));
        assert_eq!(damage(&mut bar), [WHOLE]);
        // So with the middle button, and then with the keyboard too.
        let third = id(&tabs, 2);
        down(&mut bar, &tabs, Button::Middle);
        assert_eq!(
            bar.up(&tabs, Button::Middle),
            Some(TabAction::CloseTab(third))
        );
        assert!(tabs.close(third).is_ok());
        bar.tab_closed(&tabs, 2);
        let first = id(&tabs, 0);
        assert!(tabs.close(first).is_ok());
        bar.tab_closed(&tabs, 0);
        assert_eq!(bar.layout.tab_width(), 122);
        // A button that opens a tab follows the last one.
        assert_eq!(bar.buttons(), Rect::new(374, 4, 426, 32));
        // Once the pointer leaves the bar, they get their width.
        damage(&mut bar);
        bar.left(&tabs);
        assert_eq!(bar.layout.tab_width(), 240);
        assert_eq!(damage(&mut bar), [WHOLE]);
    }

    #[test]
    fn the_last_tab_and_the_keyboard_let_the_others_get_their_width_at_once() {
        // The close button of the last of six tabs goes from (708, 6) to
        // (732, 30).
        let (mut bar, mut tabs) = many(6);
        bar.moved(&tabs, Point::new(720, 20));
        close_with_mouse(&mut bar, &mut tabs, 5);
        assert_eq!(bar.layout.tab_width(), 147);
        // With the pointer on the bar, the keyboard does not keep them.
        bar.moved(&tabs, Point::new(300, 20));
        let first = id(&tabs, 0);
        assert!(tabs.close(first).is_ok());
        bar.tab_closed(&tabs, 0);
        assert_eq!(bar.layout.tab_width(), 184);
        // Nor the mouse, once the pointer is out of the bar: a menu has it.
        bar.moved(&tabs, Point::new(200, 20));
        let tab = id(&tabs, 1);
        down(&mut bar, &tabs, Button::Middle);
        assert_eq!(
            bar.up(&tabs, Button::Middle),
            Some(TabAction::CloseTab(tab))
        );
        bar.left(&tabs);
        assert!(tabs.close(tab).is_ok());
        bar.tab_closed(&tabs, 1);
        assert_eq!(bar.layout.tab_width(), 240);
    }

    #[test]
    fn the_tab_that_is_shown_comes_into_view() {
        let (mut bar, mut tabs) = many(10);
        assert!(tabs.activate(id(&tabs, 9)).is_ok());
        bar.shown();
        damage(&mut bar);
        bar.settle(&tabs);
        assert_eq!(bar.layout.scroll(), 312);
        assert_eq!(bar.tab(9), Rect::new(616, 4, 716, 32));
        assert_eq!(damage(&mut bar), [WHOLE]);
        // Once: the wheel takes it away.
        bar.wheel(&tabs, 4 * 120, false);
        assert_eq!(bar.layout.scroll(), 0);
        bar.settle(&tabs);
        assert_eq!(bar.layout.scroll(), 0);
        // Also before the frame: the user scrolls the tabs, and the one
        // that was just shown no longer has to be in view.
        bar.shown();
        bar.wheel(&tabs, -120, false);
        bar.settle(&tabs);
        assert_eq!(bar.layout.scroll(), 100);
        // Titles out of the strip draw nothing, also under the buttons on
        // its right.
        damage(&mut bar);
        bar.label_changed(8);
        bar.label_changed(9);
        assert!(!bar.has_damage());
        // The menu of the tab opens below it.
        bar.scroll_to_tab(&tabs, 9);
        assert_eq!(bar.layout.scroll(), 312);

        // While the tabs keep their width, they do not scroll: the close
        // button of the third, from (300, 6) to (324, 30), closes it.
        bar.scroll_to_tab(&tabs, 0);
        bar.moved(&tabs, Point::new(310, 20));
        close_with_mouse(&mut bar, &mut tabs, 2);
        assert!(tabs.activate(id(&tabs, 8)).is_ok());
        bar.shown();
        bar.settle(&tabs);
        assert_eq!(bar.layout.scroll(), 0);
        // Once the pointer leaves the bar.
        bar.left(&tabs);
        bar.settle(&tabs);
        assert_eq!(bar.layout.scroll(), 212);
    }

    #[test]
    fn the_strip_cuts_the_tabs_between_the_buttons_that_scroll_it() {
        let (mut bar, tabs) = many(10);
        let palette = Palette::MEIGA;
        // The strip hides half of the first tab, the active one, on its
        // left: both buttons can scroll it. Their chevrons point to their
        // sides, from (17, 14) to the middle of (14, 14) to (18, 22), and
        // from (726, 14) to the middle of (726, 14) to (730, 22).
        bar.wheel(&tabs, -60, false);
        assert_eq!(bar.layout.scroll(), 50);
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 27, 31), pixel(BAR));
        assert_eq!(at(&pixels, 28, 31), pixel(palette.background));
        assert_eq!(at(&pixels, 77, 31), pixel(palette.background));
        for (x, y) in [(17, 14), (14, 17), (14, 18), (17, 21), (726, 14), (729, 18)] {
            assert_eq!(at(&pixels, x, y), pixel(TEXT), "({x}, {y})");
        }
        assert_eq!(at(&pixels, 15, 14), pixel(BAR));
        // At the first tab, the button on the left is darker, and the
        // pointer does not highlight it.
        bar.wheel(&tabs, 120, false);
        bar.moved(&tabs, Point::new(10, 20));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 17, 14), pixel(DISABLED_TEXT));
        assert_eq!(at(&pixels, 6, 6), pixel(BAR));
        assert_eq!(at(&pixels, 726, 14), pixel(TEXT));
        // The one on the right is highlighted, from (718, 6) to (738, 30).
        bar.moved(&tabs, Point::new(720, 20));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 718, 6), pixel(BUTTON_HOVERED));
        assert_eq!(at(&pixels, 726, 14), pixel(HOVERED_TEXT));
        down(&mut bar, &tabs, Button::Left);
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 737, 29), pixel(BUTTON_PRESSED));
    }

    /// The content of a tab with an indicator.
    struct Marked(&'static str, Option<Indicator>);

    impl Label for Marked {
        fn label(&self) -> &str {
            self.0
        }

        fn indicator(&self) -> Option<Indicator> {
            self.1
        }
    }

    #[test]
    fn an_indicator_goes_before_the_title_with_a_blank_cell() {
        // Tabs of 184 pixels: the title of the second one goes from 196 to
        // 344, of the third from 380 to 528, and of the fourth from 564 to
        // 712. The tabs are plain.
        let mut tabs = model::Window::new(Marked("pwsh", None));
        tabs.open(Marked("cmd", Some(Indicator::Ended)));
        tabs.open(Marked("wsl", Some(Indicator::Activity)));
        tabs.open(Marked("ssh", Some(Indicator::Bell)));
        let fonts = FontSystem::new().unwrap();
        let mut bar = TabBar::new(&fonts, 96, 983, tabs.tab_count(), MAXIMIZED).unwrap();
        // A cell is a few pixels.
        let cell = bar.font_mut().cell().width as i32;
        let pixels = draw(&mut bar, &tabs);

        // The end: a red cross, as steps, in a square as wide as a cell, in
        // the middle of the height of the tab.
        let side = cell.min(28);
        let top = 4 + (28 - side) / 2;
        for (x, y) in [
            (196, top),
            (196 + side - 1, top),
            (196, top + side - 1),
            (196 + side - 1, top + side - 1),
        ] {
            assert_eq!(at(&pixels, x as usize, y as usize), pixel(ENDED));
        }
        assert_eq!(
            at(&pixels, (196 + side / 2) as usize, top as usize),
            pixel(BAR)
        );
        let title = Rect::new(196 + 2 * cell, 4, 344, 32);
        assert!(text_in(&pixels, title, TEXT, BAR));
        // The activity, in the colour of the title.
        let dot = Rect::new(380, 4, 380 + cell, 32);
        assert!(text_in(&pixels, dot, TEXT, BAR));
        // The bell, two cells wide, in amber.
        let bell = Rect::new(564, 4, 564 + 2 * cell, 32);
        assert!(text_in(&pixels, bell, BELL, BAR));
        let title = Rect::new(564 + 3 * cell, 4, 712, 32);
        assert!(text_in(&pixels, title, TEXT, BAR));
    }

    #[test]
    fn the_cells_of_an_area_are_whole_cells() {
        assert_eq!(columns(Rect::new(10, 0, 45, 32), 10), 3);
        assert_eq!(columns(Rect::new(10, 0, 50, 32), 10), 4);
        assert_eq!(columns(Rect::new(10, 0, 19, 32), 10), 0);
    }

    #[test]
    fn an_indicator_needs_room_for_itself_and_a_blank_cell() {
        // Three cells of 10 pixels: the dot, a blank one and one of title.
        let three = Rect::new(10, 4, 40, 32);
        let dot = Some(Indicator::Activity);
        let slot = Rect::new(10, 4, 20, 32);
        let expected = (Rect::new(30, 4, 40, 32), Some((Indicator::Activity, slot)));
        assert_eq!(split_title(three, 10, dot), expected);
        // The bell, two cells, and the blank one: no room left for the title.
        let bell = Some(Indicator::Bell);
        let slot = Rect::new(10, 4, 30, 32);
        let expected = (Rect::new(40, 4, 40, 32), Some((Indicator::Bell, slot)));
        assert_eq!(split_title(three, 10, bell), expected);
        // Without room for the blank cell, without an indicator.
        let two = Rect::new(10, 4, 30, 32);
        assert_eq!(split_title(two, 10, bell), (two, None));
        assert_eq!(split_title(three, 10, None), (three, None));
    }

    #[test]
    fn the_field_of_a_name_goes_where_the_title_goes() {
        // The same tabs: the field goes after the indicator, and a blank
        // cell, as far as the strip shows it.
        let mut tabs = model::Window::new(Marked("pwsh", None));
        tabs.open(Marked("cmd", Some(Indicator::Ended)));
        tabs.open(Marked("wsl", Some(Indicator::Activity)));
        tabs.open(Marked("ssh", Some(Indicator::Bell)));
        let fonts = FontSystem::new().unwrap();
        let bar = TabBar::new(&fonts, 96, 983, tabs.tab_count(), MAXIMIZED).unwrap();
        let cell = bar.cell().width as i32;
        assert_eq!(bar.text_area(0, None), Rect::new(12, 4, 160, 32));
        let ended = Some(Indicator::Ended);
        assert_eq!(
            bar.text_area(1, ended),
            Rect::new(196 + 2 * cell, 4, 344, 32)
        );
        let bell = Some(Indicator::Bell);
        assert_eq!(
            bar.text_area(3, bell),
            Rect::new(564 + 3 * cell, 4, 712, 32)
        );
        // Out of the strip, nothing. The first tab goes from 28 to 128, and
        // its close button from 100.
        let (bar, _) = many(10);
        assert_eq!(bar.text_area(0, None), Rect::new(36, 4, 100, 32));
        assert!(bar.text_area(7, None).is_empty());
    }

    /// Draws the whole bar of `tabs` on a canvas of its size, and returns
    /// its pixels.
    fn draw<T: Label>(bar: &mut TabBar, tabs: &model::Window<T>) -> Vec<u32> {
        let size = Size::new(983, bar.height());
        let mut pixels = vec![0; size.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, size, 983).unwrap();
        bar.draw(&mut canvas, tabs, &Palette::MEIGA, None);
        pixels
    }

    fn at(pixels: &[u32], x: usize, y: usize) -> u32 {
        pixels[y * 983 + x]
    }

    #[test]
    fn the_active_tab_has_the_colours_of_the_terminal() {
        // Tabs of 184 pixels, and the pointer on the fourth.
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
        assert_eq!(at(&pixels, 372, 18), pixel(SEPARATOR));
        assert_eq!(at(&pixels, 188, 18), pixel(BAR));
        assert_eq!(at(&pixels, 556, 18), pixel(HOVERED_TAB));
        // Each title, in the colours of its tab.
        for (title, text, background) in [
            (
                Rect::new(12, 4, 160, 32),
                palette.foreground,
                palette.background,
            ),
            (Rect::new(196, 4, 344, 32), TEXT, BAR),
            (Rect::new(564, 4, 712, 32), HOVERED_TEXT, HOVERED_TAB),
        ] {
            assert!(text_in(&pixels, title, text, background), "{title:?}");
        }
        // The crosses, as steps, in the colour of the text of their tabs.
        for (x, y) in [(168, 14), (175, 14), (171, 17), (172, 17), (175, 21)] {
            assert_eq!(at(&pixels, x, y), pixel(palette.foreground));
        }
        assert_eq!(at(&pixels, 169, 14), pixel(palette.background));
        assert_eq!(at(&pixels, 352, 14), pixel(TEXT));
        assert_eq!(at(&pixels, 720, 14), pixel(HOVERED_TEXT));
        // The plus, with arms of 5 pixels around its middle, (757, 17).
        for (x, y) in [(752, 17), (762, 17), (757, 12), (757, 22), (757, 17)] {
            assert_eq!(at(&pixels, x, y), pixel(TEXT));
        }
        for (x, y) in [(756, 16), (763, 17), (757, 23)] {
            assert_eq!(at(&pixels, x, y), pixel(BAR));
        }
        // The chevron, from (780, 16) to (788, 20): two lines as steps,
        // which meet in the middle of its bottom.
        for (x, y) in [(780, 16), (787, 16), (783, 19), (784, 19)] {
            assert_eq!(at(&pixels, x, y), pixel(TEXT));
        }
        for (x, y) in [(781, 16), (783, 18), (783, 20)] {
            assert_eq!(at(&pixels, x, y), pixel(BAR));
        }
    }

    #[test]
    fn the_field_of_a_name_shows_its_selection_and_its_caret() {
        // Tabs of 184 pixels: the title of the active one goes from 12 to
        // 160, and its cells are in the middle of the height of the tab.
        let (mut bar, tabs) = bar(&["pwsh", "cmd", "wsl", "ssh"]);
        let cell = bar.cell();
        let (width, height) = (cell.width as i32, cell.height as i32);
        let top = 4 + (28 - height) / 2;
        let cells: Vec<CellChar> = text::cells("abc").collect();
        let field = View {
            cells: &cells,
            shown: Shown {
                caret: 3,
                selection: 0..2,
            },
            caret: Some(2),
        };
        let size = Size::new(983, bar.height());
        let mut pixels = vec![0; size.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, size, 983).unwrap();
        bar.draw(&mut canvas, &tabs, &Palette::MEIGA, Some((0, field)));
        let palette = Palette::MEIGA;
        // The selected cells on the background of the selection of the
        // terminal, and the other one on the background of the tab, which is
        // above and below them too.
        let selected = Rect::new(12, top, 12 + 2 * width, top + height);
        assert!(text_in(
            &pixels,
            selected,
            palette.foreground,
            palette.selection
        ));
        let other = Rect::new(12 + 2 * width, top, 12 + 3 * width, top + height);
        assert!(text_in(
            &pixels,
            other,
            palette.foreground,
            palette.background
        ));
        assert_eq!(
            at(&pixels, 12, (top - 1) as usize),
            pixel(palette.background)
        );
        let below = (top + height) as usize;
        assert_eq!(at(&pixels, 12, below), pixel(palette.background));
        // The caret, two pixels wide, on the left side of the fourth cell, as
        // high as a cell.
        let caret = 12 + 3 * width;
        for y in [top, top + height - 1] {
            for x in [caret, caret + 1] {
                assert_eq!(
                    at(&pixels, x as usize, y as usize),
                    pixel(palette.foreground)
                );
            }
        }
        assert_eq!(
            at(&pixels, caret as usize, below),
            pixel(palette.background)
        );
        // And nothing of the title after it.
        for y in 4..32 {
            for x in caret + 2..160 {
                assert_eq!(
                    at(&pixels, x as usize, y as usize),
                    pixel(palette.background)
                );
            }
        }
    }

    #[test]
    fn the_field_of_a_tab_that_the_strip_cuts_starts_where_the_strip_does() {
        // Three tenths of a notch of the wheel scroll the tabs 30 pixels:
        // the title of the first one goes from 6 to 70, and the strip shows
        // it from 28, where the clicks of the field find its first cell.
        let (mut bar, tabs) = many(10);
        bar.wheel(&tabs, -36, false);
        assert_eq!(bar.text_area(0, None), Rect::new(28, 4, 70, 32));
        let cell = bar.cell();
        let (width, height) = (cell.width as i32, cell.height as i32);
        let top = 4 + (28 - height) / 2;
        let cells: Vec<CellChar> = text::cells("abc").collect();
        let field = View {
            cells: &cells,
            shown: Shown {
                caret: 0,
                selection: 0..3,
            },
            caret: None,
        };
        let size = Size::new(983, bar.height());
        let mut pixels = vec![0; size.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, size, 983).unwrap();
        bar.draw(&mut canvas, &tabs, &Palette::MEIGA, Some((0, field)));
        let palette = Palette::MEIGA;
        let selected = Rect::new(28, top, 28 + 3 * width, top + height);
        assert!(text_in(
            &pixels,
            selected,
            palette.foreground,
            palette.selection
        ));
    }

    #[test]
    fn buttons_are_highlighted_under_the_pointer_and_while_pressed() {
        // The highlight of the close button of the second tab goes from
        // (458, 8) to (478, 28), the one of the button that opens a tab
        // from (730, 6) to (754, 30), and the one of the button that opens
        // the menu of the profiles from (758, 6) to (778, 30).
        let (mut bar, tabs) = three();
        bar.moved(&tabs, Point::new(460, 20));
        assert_eq!(at(&draw(&mut bar, &tabs), 458, 8), pixel(BUTTON_HOVERED));
        down(&mut bar, &tabs, Button::Left);
        assert_eq!(at(&draw(&mut bar, &tabs), 477, 27), pixel(BUTTON_PRESSED));
        // While it is pressed, no other button is highlighted.
        bar.moved(&tabs, Point::new(740, 10));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 458, 8), pixel(BAR));
        assert_eq!(at(&pixels, 730, 6), pixel(BAR));
        assert_eq!(bar.up(&tabs, Button::Left), None);
        assert_eq!(at(&draw(&mut bar, &tabs), 730, 6), pixel(BUTTON_HOVERED));
        // The pointer on the chevron gives it the colour of the hovered
        // text, (764, 16) being its first pixel.
        bar.moved(&tabs, Point::new(770, 10));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 730, 6), pixel(BAR));
        assert_eq!(at(&pixels, 757, 6), pixel(BAR));
        assert_eq!(at(&pixels, 758, 6), pixel(BUTTON_HOVERED));
        assert_eq!(at(&pixels, 764, 16), pixel(HOVERED_TEXT));
        down(&mut bar, &tabs, Button::Left);
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 777, 29), pixel(BUTTON_PRESSED));
        assert_eq!(at(&pixels, 764, 16), pixel(HOVERED_TEXT));
    }

    #[test]
    fn the_buttons_of_the_window_act_when_the_left_button_goes_up_on_them() {
        // Minimize from 845 to 891, maximize to 937 and close to 983, as
        // high as the bar: at the top row of the screen, and close at its
        // corner, while the window is maximized.
        let (mut bar, tabs) = three();
        for (at, action) in [
            (Point::new(860, 20), TabAction::Minimize),
            (Point::new(900, 0), TabAction::ToggleMaximize),
            (Point::new(982, 0), TabAction::CloseWindow),
        ] {
            bar.moved(&tabs, at);
            assert_eq!(down(&mut bar, &tabs, Button::Left), None);
            assert!(bar.is_pressed());
            assert_eq!(bar.up(&tabs, Button::Left), Some(action));
            assert!(!bar.is_pressed());
        }
        // Moving away takes the click back, also onto another button.
        bar.moved(&tabs, Point::new(960, 10));
        down(&mut bar, &tabs, Button::Left);
        bar.moved(&tabs, Point::new(900, 10));
        assert_eq!(bar.up(&tabs, Button::Left), None);
        // And coming back gives it again.
        bar.moved(&tabs, Point::new(960, 10));
        down(&mut bar, &tabs, Button::Left);
        bar.moved(&tabs, Point::new(960, 100));
        bar.moved(&tabs, Point::new(970, 30));
        assert_eq!(bar.up(&tabs, Button::Left), Some(TabAction::CloseWindow));
        // The other buttons of the mouse do nothing on them.
        for button in [Button::Middle, Button::Right] {
            assert_eq!(down(&mut bar, &tabs, button), None);
            assert!(!bar.is_pressed());
        }
    }

    #[test]
    fn the_pointer_and_the_press_draw_again_only_the_button_of_the_window() {
        let (mut bar, tabs) = three();
        let (minimize, maximize) = (Rect::new(845, 0, 891, 32), Rect::new(891, 0, 937, 32));
        assert!(bar.moved(&tabs, Point::new(860, 20)));
        assert_eq!(damage(&mut bar), [minimize]);
        assert!(!bar.moved(&tabs, Point::new(870, 2)));
        assert_eq!(damage(&mut bar), []);
        bar.moved(&tabs, Point::new(900, 20));
        assert_eq!(damage(&mut bar), [minimize, maximize]);
        down(&mut bar, &tabs, Button::Left);
        assert_eq!(damage(&mut bar), [maximize]);
        assert_eq!(bar.up(&tabs, Button::Left), Some(TabAction::ToggleMaximize));
        assert_eq!(damage(&mut bar), [maximize]);
        bar.left(&tabs);
        assert_eq!(damage(&mut bar), [maximize]);
        // Once the window is maximized or restored, only its button; and in
        // practice, the band above the tabs changes with it.
        bar.set_frame(
            Frame {
                maximized: false,
                ..MAXIMIZED
            },
            &tabs,
        );
        assert_eq!(damage(&mut bar), [maximize]);
    }

    #[test]
    fn a_band_above_the_tabs_moves_them_down_but_not_the_buttons_of_the_window() {
        let (mut bar, tabs) = three();
        damage(&mut bar);
        let restored = Frame {
            band: 8,
            maximized: false,
            ..MAXIMIZED
        };
        bar.set_frame(restored, &tabs);
        assert_eq!(bar.height(), 40);
        assert_eq!(damage(&mut bar), [Rect::new(0, 0, 983, 40)]);
        bar.set_frame(restored, &tabs);
        assert!(!bar.has_damage());
        // The band, which Windows uses to resize the window, is not a part
        // of the tabs.
        bar.moved(&tabs, Point::new(300, 5));
        assert!(!bar.has_damage());
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert!(!bar.is_pressed());
        bar.moved(&tabs, Point::new(300, 10));
        assert_eq!(damage(&mut bar), [Rect::new(244, 8, 485, 40)]);
        // The buttons of the window are as high as the whole bar.
        bar.moved(&tabs, Point::new(960, 2));
        damage(&mut bar);
        down(&mut bar, &tabs, Button::Left);
        assert_eq!(damage(&mut bar), [Rect::new(937, 0, 983, 40)]);
        assert_eq!(bar.up(&tabs, Button::Left), Some(TabAction::CloseWindow));
        // Without the band again, the tabs keep their width.
        let width = bar.tab(1).width();
        bar.set_frame(MAXIMIZED, &tabs);
        assert_eq!(bar.height(), 32);
        assert_eq!(damage(&mut bar), [WHOLE]);
        assert_eq!(bar.tab(1).width(), width);
    }

    #[test]
    fn without_the_title_bar_of_meiga_the_room_of_the_buttons_stays_empty() {
        let (mut bar, tabs) = three();
        damage(&mut bar);
        let windows = Frame {
            buttons: false,
            ..MAXIMIZED
        };
        bar.set_frame(windows, &tabs);
        assert_eq!(damage(&mut bar), [WHOLE]);
        bar.set_focused(true);
        bar.moved(&tabs, Point::new(960, 10));
        assert!(!bar.has_damage());
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert!(!bar.is_pressed());
        let pixels = draw(&mut bar, &tabs);
        for (x, y) in [(865, 15), (911, 11), (955, 11), (960, 10)] {
            assert_eq!(at(&pixels, x, y), pixel(BAR), "({x}, {y})");
        }
        // With it again, they are there, under the pointer.
        bar.set_frame(MAXIMIZED, &tabs);
        assert_eq!(damage(&mut bar), [WHOLE]);
        assert_eq!(down(&mut bar, &tabs, Button::Left), None);
        assert!(bar.is_pressed());
        assert_eq!(bar.up(&tabs, Button::Left), Some(TabAction::CloseWindow));
    }

    #[test]
    fn the_buttons_of_the_window_have_the_glyphs_of_windows() {
        let (mut bar, tabs) = three();
        bar.set_focused(true);
        let pixels = draw(&mut bar, &tabs);
        // Minimize: a line across the middle of its square, from (863, 11)
        // to (873, 21).
        for x in 863..873 {
            assert_eq!(at(&pixels, x, 15), pixel(TEXT));
        }
        for (x, y) in [(862, 15), (873, 15), (867, 14), (867, 16)] {
            assert_eq!(at(&pixels, x, y), pixel(BAR), "({x}, {y})");
        }
        // Restore, while the window is maximized: the square in front, from
        // (909, 13) to (917, 21), and of the one behind, from (911, 11) to
        // (919, 19), what shows.
        for (x, y) in [
            (909, 13),
            (916, 13),
            (909, 17),
            (909, 20),
            (913, 20),
            (916, 20),
            (911, 11),
            (918, 11),
            (918, 18),
            (911, 12),
            (917, 18),
        ] {
            assert_eq!(at(&pixels, x, y), pixel(TEXT), "({x}, {y})");
        }
        for (x, y) in [(910, 11), (914, 12), (912, 15), (917, 15), (918, 19)] {
            assert_eq!(at(&pixels, x, y), pixel(BAR), "({x}, {y})");
        }
        // Close: a cross, as steps, from (955, 11) to (965, 21).
        for (x, y) in [
            (955, 11),
            (964, 11),
            (959, 15),
            (960, 15),
            (955, 20),
            (964, 20),
        ] {
            assert_eq!(at(&pixels, x, y), pixel(TEXT), "({x}, {y})");
        }
        assert_eq!(at(&pixels, 960, 11), pixel(BAR));

        // Maximize, once the window is restored, below the band: the sides
        // of its square, from (909, 15) to (919, 25).
        let restored = Frame {
            band: 8,
            maximized: false,
            ..MAXIMIZED
        };
        bar.set_frame(restored, &tabs);
        let pixels = draw(&mut bar, &tabs);
        for (x, y) in [
            (909, 15),
            (913, 15),
            (918, 15),
            (909, 20),
            (918, 20),
            (909, 24),
            (913, 24),
            (918, 24),
        ] {
            assert_eq!(at(&pixels, x, y), pixel(TEXT), "({x}, {y})");
        }
        for (x, y) in [(910, 16), (913, 20), (917, 23), (919, 20), (913, 25)] {
            assert_eq!(at(&pixels, x, y), pixel(BAR), "({x}, {y})");
        }
    }

    #[test]
    fn the_close_button_of_the_window_is_red_and_the_others_grey() {
        let (mut bar, tabs) = three();
        bar.set_focused(true);
        bar.moved(&tabs, Point::new(960, 20));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 936, 0), pixel(BAR));
        assert_eq!(at(&pixels, 937, 0), pixel(CLOSE_HOVERED));
        assert_eq!(at(&pixels, 982, 31), pixel(CLOSE_HOVERED));
        assert_eq!(at(&pixels, 955, 11), pixel(CLOSE_GLYPH));
        down(&mut bar, &tabs, Button::Left);
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 937, 0), pixel(CLOSE_PRESSED));
        assert_eq!(at(&pixels, 955, 11), pixel(CLOSE_PRESSED_GLYPH));
        assert_eq!(bar.up(&tabs, Button::Left), Some(TabAction::CloseWindow));

        bar.moved(&tabs, Point::new(860, 20));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 845, 0), pixel(BUTTON_HOVERED));
        assert_eq!(at(&pixels, 865, 15), pixel(HOVERED_TEXT));
        assert_eq!(at(&pixels, 937, 0), pixel(BAR));
        assert_eq!(at(&pixels, 955, 11), pixel(TEXT));
        down(&mut bar, &tabs, Button::Left);
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 890, 31), pixel(BUTTON_PRESSED));
        assert_eq!(at(&pixels, 865, 15), pixel(HOVERED_TEXT));
        // While it is down, the pointer on another button highlights
        // nothing, and gives its glyph the colour of the hovered text.
        bar.moved(&tabs, Point::new(960, 20));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 845, 0), pixel(BAR));
        assert_eq!(at(&pixels, 865, 15), pixel(TEXT));
        assert_eq!(at(&pixels, 937, 0), pixel(BAR));
        assert_eq!(at(&pixels, 955, 11), pixel(HOVERED_TEXT));
    }

    #[test]
    fn without_the_focus_the_glyphs_of_the_buttons_of_the_window_are_dimmer() {
        let (mut bar, tabs) = three();
        let captions = Rect::new(845, 0, 983, 32);
        let pixels = draw(&mut bar, &tabs);
        for (x, y) in [(865, 15), (911, 11), (955, 11)] {
            assert_eq!(at(&pixels, x, y), pixel(INACTIVE_GLYPH), "({x}, {y})");
        }
        // The pointer still highlights them.
        bar.moved(&tabs, Point::new(860, 20));
        let pixels = draw(&mut bar, &tabs);
        assert_eq!(at(&pixels, 845, 0), pixel(BUTTON_HOVERED));
        assert_eq!(at(&pixels, 865, 15), pixel(HOVERED_TEXT));
        bar.left(&tabs);
        damage(&mut bar);
        // The focus draws the three again, once.
        bar.set_focused(true);
        assert_eq!(damage(&mut bar), [captions]);
        bar.set_focused(true);
        assert!(!bar.has_damage());
        assert_eq!(at(&draw(&mut bar, &tabs), 865, 15), pixel(TEXT));
        bar.set_focused(false);
        assert_eq!(damage(&mut bar), [captions]);
        assert_eq!(at(&draw(&mut bar, &tabs), 865, 15), pixel(INACTIVE_GLYPH));
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
