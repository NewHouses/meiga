//! The editing of the name of a tab, in its tab on the bar (ADR 0031): the
//! text field of `meiga_core::ui::field`, which a double click on a tab,
//! "Rename tab" of its menu and Ctrl+Shift+R open, with the text that the
//! tab shows, all of it selected.
//!
//! While it is open, it gets all the keys, as a menu: Enter ends it with
//! the name, and Esc without it; the other keys are the field's, and the
//! shortcuts of meiga do nothing. The spaces at the ends of a name go away,
//! and a name of only spaces is none: the tab shows the title of its
//! program again. Enter without changing the text changes nothing, so a tab
//! that showed the title of its program goes on showing its titles.
//!
//! The mouse in the field is the field's: a click puts the caret, with
//! Shift moves the end of the selection, and dragging selects, also out of
//! the field, which then shows more of the text towards that side, again
//! after each [`DRAG_REPEAT`] while the pointer stays out of it; a double
//! click selects a word, and a triple click all of it. The other buttons
//! do nothing in it.
//!
//! The caret blinks as the cursor of the terminal, while the window has the
//! focus. Without the focus it is not shown, and the editing goes on when
//! the window gets it again.

use std::mem;
use std::time::{Duration, Instant};

use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::input::KeyEvent;
use meiga_core::model::TabId;
use meiga_core::mouse::{Button, ClickSettings, Clicks, Modifiers};
use meiga_core::term::Side;
use meiga_core::text::CellChar;
use meiga_core::ui::field::{Field, Reply, Shown};
use meiga_win::window::{self, CaretSettings};

use crate::blink;

/// The most UTF-16 code units of a name: as many as a title can have (ADR
/// 0029, decision 6).
pub const NAME_LIMIT: usize = 1000;

/// While the left button that went down in the field stays down, with the
/// pointer out of the field on a side, the field shows more of the text
/// towards it after each `DRAG_REPEAT`, as the buttons that scroll the
/// tabs.
pub const DRAG_REPEAT: Duration = Duration::from_millis(100);

const VK_RETURN: u16 = 0x0d;
const VK_ESCAPE: u16 = 0x1b;

/// How the editing of a name ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Done {
    /// Esc, or Enter with the text that the tab showed: nothing changes.
    Unchanged,
    /// The tab has this name from now on or, with `None`, shows again the
    /// title of its program, or the name of its profile.
    Renamed(Option<String>),
}

/// What a key did in the editing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Nothing: Tab, the arrows up and down, and the keys without a
    /// character.
    Ignored,
    /// The field took it.
    Taken,
    /// The field took it, and `text` goes to the clipboard.
    Copy(String),
    /// The field took it, and the text of the clipboard goes to
    /// [`Editor::paste`].
    Paste,
    /// The editing ends.
    Done(Done),
}

/// Where the field is in the window: the place of the title of its tab,
/// after its indicator; and the size of its cells, those of the bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub area: Rect,
    pub cell: Size,
}

/// What the field takes of Windows and of the window: the settings of the
/// caret and of the clicks, and the scale.
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub caret: CaretSettings,
    pub clicks: ClickSettings,
    pub dpi: u32,
}

impl Settings {
    /// The settings of Windows now, at `dpi`.
    pub fn read(dpi: u32) -> Self {
        Self {
            caret: window::caret_settings(),
            clicks: window::mouse_settings().clicks,
            dpi,
        }
    }
}

/// What the field shows in its tab.
pub struct View<'a> {
    /// The cells of the text that it shows.
    pub cells: &'a [CellChar],
    /// Where the caret and the selection are, in those cells.
    pub shown: Shown,
    /// The width of the caret, in pixels, while it is shown.
    pub caret: Option<u32>,
}

/// The editing of the name of a tab.
pub struct Editor {
    tab: TabId,
    field: Field,
    /// The text that the tab showed when the editing started.
    initial: String,
    place: Place,
    clicks: Clicks,
    click_settings: ClickSettings,
    /// While the left button that went down in the field is down: where the
    /// pointer is.
    pointer: Option<Point>,
    /// While that pointer is out of the field on a side: when the field
    /// shows more of the text towards it.
    repeat: Option<Instant>,
    caret: CaretSettings,
    /// The width of the caret, in pixels, at the scale of the window.
    thickness: u32,
    focused: bool,
    /// When the caret started to blink: at the last key or click.
    since: Instant,
    /// Whether the last frame showed the caret.
    caret_shown: bool,
    /// Whether the field has to be drawn again.
    changed: bool,
    cells: Vec<CellChar>,
    /// The keys that were down when the editing started, until they go up:
    /// they do nothing while they repeat, as the Enter that chose "Rename
    /// tab" in the menu, which would end it at once.
    stale: Vec<u16>,
}

impl Editor {
    /// The editing of the name of the tab `tab`, which shows `text`, at
    /// `place`, with `settings`, while the window has the focus or not. It
    /// starts at `now`, with all the text selected.
    pub fn new(
        tab: TabId,
        text: &str,
        place: Place,
        settings: Settings,
        focused: bool,
        now: Instant,
    ) -> Self {
        let mut field = Field::new(NAME_LIMIT);
        field.set_text(text);
        field.select_all();
        let mut editor = Self {
            tab,
            initial: field.text().to_owned(),
            field,
            place,
            clicks: Clicks::new(),
            click_settings: settings.clicks,
            pointer: None,
            repeat: None,
            caret: settings.caret,
            thickness: blink::thickness(settings.caret, settings.dpi),
            focused,
            since: now,
            caret_shown: focused,
            changed: true,
            cells: Vec::new(),
            stale: Vec::new(),
        };
        editor.field.set_columns(editor.columns());
        editor
    }

    /// The tab whose name it edits.
    pub const fn tab(&self) -> TabId {
        self.tab
    }

    /// The field is now at `place`: the bar moved, or the window changed
    /// its scale.
    pub fn place(&mut self, place: Place) {
        if place == self.place {
            return;
        }
        self.place = place;
        self.field.set_columns(self.columns());
        self.changed = true;
    }

    /// The window is at `dpi` now: the caret has its width there.
    pub fn set_dpi(&mut self, dpi: u32) {
        self.thickness = blink::thickness(self.caret, dpi);
        self.changed = true;
    }

    /// The keys of `keys` are down now: they do nothing until they go up.
    pub fn ignore(&mut self, keys: impl IntoIterator<Item = u16>) {
        self.stale.extend(keys);
    }

    /// Whether `key` was down when the editing started, and did not go up
    /// yet: it does nothing while it repeats.
    pub fn was_down(&self, key: u16) -> bool {
        self.stale.contains(&key)
    }

    /// `key` went up.
    pub fn key_up(&mut self, key: u16) {
        self.stale.retain(|&stale| stale != key);
    }

    /// The window gained or lost the focus, at `now`. With it, the caret
    /// blinks again from the start; without it, it is not shown. The keys
    /// that were down go up in another window.
    pub fn focus(&mut self, focused: bool, now: Instant) {
        self.stale.clear();
        if focused != self.focused {
            self.focused = focused;
            self.since = now;
            self.changed = true;
        }
    }

    /// A key, at `now`: the field's, but Enter, which ends the editing with
    /// the name, and Esc, which ends it without it.
    pub fn key(&mut self, event: &KeyEvent, now: Instant) -> Edit {
        let edit = match self.field.key(event) {
            Reply::Ignored if event.down && event.key == VK_RETURN => Edit::Done(self.commit()),
            Reply::Ignored if event.down && event.key == VK_ESCAPE => Edit::Done(Done::Unchanged),
            Reply::Ignored => return Edit::Ignored,
            Reply::Taken => Edit::Taken,
            Reply::Copy(text) => Edit::Copy(text),
            Reply::Paste => Edit::Paste,
        };
        self.since = now;
        edit
    }

    /// Pastes `text`, the text of the clipboard, at `now`.
    pub fn paste(&mut self, text: &str, now: Instant) {
        self.field.paste(text);
        self.since = now;
    }

    /// How the editing ends with Enter, or a click outside the field: the
    /// text of the field, without the spaces at its ends, is the name, and
    /// none if nothing is left; but if it is the text that the tab showed,
    /// nothing changes.
    pub fn commit(&self) -> Done {
        let text = self.field.text();
        if text == self.initial {
            return Done::Unchanged;
        }
        let name = text.trim();
        Done::Renamed((!name.is_empty()).then(|| name.to_owned()))
    }

    /// Whether `at` is in the field.
    pub fn contains(&self, at: Point) -> bool {
        self.place.area.contains_point(at)
    }

    /// `button` went down at `at`, in the field, with `modifiers`, at `now`.
    /// Returns whether the window has to get all the mouse until it goes
    /// up: the left button selects while it is down.
    pub fn down(&mut self, button: Button, at: Point, modifiers: Modifiers, now: Instant) -> bool {
        if button != Button::Left || !self.contains(at) {
            return false;
        }
        let clicks = self.clicks.press(button, at, now, &self.click_settings);
        let (column, side) = self.column_at(at);
        self.field
            .press(column.unsigned_abs(), side, clicks, modifiers.shift);
        self.pointer = Some(at);
        self.repeat = None;
        self.since = now;
        true
    }

    /// The pointer moved to `at`, at `now`. While the left button that went
    /// down in the field is down, it selects up to there.
    pub fn moved(&mut self, at: Point, now: Instant) {
        if self.pointer.is_none() {
            return;
        }
        self.pointer = Some(at);
        self.drag(now);
    }

    /// `button` went up: the left one ends the selection with the mouse.
    pub fn up(&mut self, button: Button) {
        if button == Button::Left {
            self.pointer = None;
            self.repeat = None;
        }
    }

    /// The window no longer gets all the mouse.
    pub fn capture_lost(&mut self) {
        self.pointer = None;
        self.repeat = None;
    }

    /// Whether the left button that went down in the field is still down.
    pub const fn is_pressed(&self) -> bool {
        self.pointer.is_some()
    }

    /// Selects up to where the pointer is, at `now`, and, while it is out
    /// of the field on a side, says when to do it again.
    fn drag(&mut self, now: Instant) {
        let Some(at) = self.pointer else {
            return;
        };
        let (column, side) = self.column_at(at);
        self.field.drag(column, side);
        let out = column < 0 || column.unsigned_abs() >= self.columns();
        self.repeat = if out {
            self.repeat.or_else(|| now.checked_add(DRAG_REPEAT))
        } else {
            None
        };
        self.since = now;
    }

    /// Brings the field up to date at `now`: while the pointer is out of it
    /// with the left button down, it shows more of the text when it is
    /// time; and the caret blinks. Returns whether it has to be drawn
    /// again, and when to come back.
    pub fn update(&mut self, now: Instant) -> (bool, Option<Instant>) {
        if self.repeat.is_some_and(|due| due <= now) {
            self.repeat = None;
            self.drag(now);
        }
        let edited = self.field.take_changes().shown;
        let changed = mem::take(&mut self.changed) || edited;
        let (shown, next) = blink::phase(&self.caret, self.focused, self.since, now);
        let shown = shown && self.focused;
        let blinked = shown != self.caret_shown;
        self.caret_shown = shown;
        let next = match (next, self.repeat) {
            (Some(next), Some(repeat)) => Some(next.min(repeat)),
            (next, repeat) => next.or(repeat),
        };
        (changed || blinked, next)
    }

    /// What the field shows now.
    pub fn view(&mut self) -> View<'_> {
        let shown = self.field.show(&mut self.cells);
        View {
            cells: &self.cells,
            shown,
            caret: self.caret_shown.then_some(self.thickness),
        }
    }

    /// How many cells the field has.
    fn columns(&self) -> usize {
        let width = self.place.cell.width.max(1);
        (self.place.area.width() / width) as usize
    }

    /// The cell of the field under `at`, from its first one, which can be
    /// out of it, and the side of the cell: its right half from the middle.
    fn column_at(&self, at: Point) -> (isize, Side) {
        let width = i64::from(self.place.cell.width.max(1));
        let x = i64::from(at.x) - i64::from(self.place.area.left);
        let column = isize::try_from(x.div_euclid(width)).unwrap_or(isize::MAX);
        let side = if x.rem_euclid(width) > width / 2 {
            Side::Right
        } else {
            Side::Left
        };
        (column, side)
    }
}

#[cfg(test)]
mod tests;
