//! A text field: a line of text that the user edits, such as the name of a
//! tab (subphase 3.3), the name of a group (6.3) or the filter of the
//! explorer (5.3) (ADR 0030). This is the field without a window: its text,
//! its caret and its selection, what the keys and the mouse do with them,
//! and the part of the text that it shows in its cells. The window draws
//! it, and reads and writes the clipboard for it.
//!
//! It edits as the text fields of Chrome on Windows:
//! - the caret stops between the grapheme clusters of Unicode (UAX #29), and
//!   moves by words with Ctrl ([`segment`]); Backspace deletes the last code
//!   point before the caret, so that `é` written as `e` and an accent loses
//!   only the accent, and Delete the cluster after it;
//! - with a selection, typing replaces it, and Backspace and Delete delete
//!   it; the arrows without Shift go to its edge;
//! - undoing goes back one step at a time ([`history`]);
//! - a paste becomes a single line: the whitespace at its ends goes away;
//!   each line break and each tab, a space; and the other control
//!   characters, away. A paste of only whitespace is a space.
//!
//! The field holds at most a number of UTF-16 code units, as Windows counts
//! them, that its user chooses: a character that does not fit is not
//! typed, and a paste puts what fits, without splitting a cluster. Its text
//! never has control characters.
//!
//! It shows its text in a row of cells of the grid, as the rest of the
//! text that meiga draws ([`crate::text::cells`]): if the text does not
//! fit, it shows the part where the caret is, and moves it just enough to
//! keep the caret in sight.

mod history;
mod keys;
mod segment;

use std::mem;
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use self::history::{Edit, History, Kind};
use self::segment::{is_stop, next_cluster, previous_cluster, word_at};
use crate::term::Side;
use crate::text::{self, CellChar};

/// What a key did in the field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    /// Nothing: the key is not of the field, and its user decides what it
    /// does, such as Enter, Esc, Tab and the arrows up and down.
    Ignored,
    /// The field took the key.
    Taken,
    /// The field took the key, and its user puts `text` in the clipboard:
    /// Ctrl+C and Ctrl+X, which already removed it from the field.
    Copy(String),
    /// The field took the key, and its user gives the text of the clipboard
    /// to [`Field::paste`]: Ctrl+V and Shift+Insert.
    Paste,
}

/// What changed in the field since its changes were last taken.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    /// What it shows: its text, its caret, its selection or the part of
    /// the text in its cells.
    pub shown: bool,
    /// Its text.
    pub text: bool,
}

/// What the field shows in its cells, besides the text: where the caret
/// is, and which cells are selected.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shown {
    /// The cell on whose left side the caret goes: always one of the cells
    /// of the field, also at the end of a text that does not fit.
    pub caret: usize,
    /// The selected cells, empty without a selection.
    pub selection: Range<usize>,
}

/// What a button held down over the field selects when it moves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Drag {
    /// Nothing: no button went down.
    #[default]
    None,
    /// Up to the cluster under the pointer, after a click.
    Clusters,
    /// Whole words, after a double click on `.0`.
    Words(Range<usize>),
    /// Everything, after a triple click.
    All,
}

/// A text field.
#[derive(Clone, Debug)]
pub struct Field {
    text: String,
    /// Where the selection started, and the caret, where it ends: byte
    /// offsets of the text where the caret can stop. Without a selection,
    /// they are the same.
    anchor: usize,
    caret: usize,
    /// The most UTF-16 code units of the text.
    limit: usize,
    history: History,
    /// The first half of a character outside the Basic Multilingual Plane,
    /// such as most emoji, until the second half comes in the next key.
    high_surrogate: Option<u16>,
    drag: Drag,
    /// How many cells the field shows, and the first cell of the text that
    /// it shows.
    columns: usize,
    scroll: usize,
    changes: Changes,
}

impl Field {
    /// An empty field that holds at most `limit` UTF-16 code units. It
    /// shows nothing until it knows how many cells it has.
    pub fn new(limit: usize) -> Self {
        Self {
            text: String::new(),
            anchor: 0,
            caret: 0,
            limit,
            history: History::default(),
            high_surrogate: None,
            drag: Drag::None,
            columns: 0,
            scroll: 0,
            changes: Changes::default(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// The most UTF-16 code units of its text.
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Where the caret is: a byte offset of the text.
    pub const fn caret(&self) -> usize {
        self.caret
    }

    /// The selected part of the text, as byte offsets: empty without a
    /// selection.
    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.caret)..self.anchor.max(self.caret)
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }

    /// Puts `text` in the field, in a single line and cut to the limit,
    /// with the caret at its end, instead of what the field had: undoing
    /// does not bring that back.
    pub fn set_text(&mut self, text: &str) {
        let line = one_line(text);
        let line = fitting(&line, self.limit);
        self.track(|field| {
            if field.text != line {
                field.text = line.to_owned();
                field.changes.text = true;
            }
            let end = field.text.len();
            (field.anchor, field.caret) = (end, end);
            field.history.clear();
            field.high_surrogate = None;
            field.drag = Drag::None;
        });
    }

    /// Selects all the text, with the caret at its end.
    pub fn select_all(&mut self) {
        let end = self.text.len();
        self.select(0, end);
    }

    /// Pastes `clipboard`, the text of the clipboard, in a single line,
    /// instead of the selection: as much of it as fits. Nothing changes if
    /// it has no text, or only control characters.
    pub fn paste(&mut self, clipboard: &str) {
        let line = one_line(clipboard);
        if line.is_empty() {
            return;
        }
        let trimmed = line.trim();
        let text = if trimmed.is_empty() { " " } else { trimmed };
        let text = fitting(text, self.room());
        if text.is_empty() {
            return;
        }
        self.track(|field| field.replace(field.selection(), text, Kind::Other));
    }

    /// The changes since the last time that they were taken.
    pub fn take_changes(&mut self) -> Changes {
        mem::take(&mut self.changes)
    }

    /// The field shows `columns` cells.
    pub fn set_columns(&mut self, columns: usize) {
        self.track(|field| field.columns = columns);
    }

    /// What the field shows: the cells of the text that it shows go to
    /// `cells`, instead of what it had, at most as many as its cells. A
    /// wide character that the right side of the field cuts is not shown.
    pub fn show(&self, cells: &mut Vec<CellChar>) -> Shown {
        cells.clear();
        cells.extend(text::cells(&self.text).skip(self.scroll).take(self.columns));
        if cells.len() == self.columns
            && let Some(last @ CellChar::WideLeft(_)) = cells.last_mut()
        {
            *last = CellChar::Blank;
        }
        let shown = self.scroll..self.scroll + self.columns;
        let cell =
            |at: usize| text::width(&self.text[..at]).clamp(shown.start, shown.end) - shown.start;
        let selection = self.selection();
        Shown {
            caret: cell(self.caret),
            selection: cell(selection.start)..cell(selection.end),
        }
    }

    /// A button went down on the cell `column` of the field, counting from
    /// the first one, at `side` of it: the `clicks`-th click in a row. A
    /// click puts the caret where it is, or, with Shift, moves the end of
    /// the selection there; a double click selects the word, the run of
    /// signs or the run of spaces under it; and a triple click, all the
    /// text.
    pub fn press(&mut self, column: usize, side: Side, clicks: u8, shift: bool) {
        let column = isize::try_from(column).unwrap_or(isize::MAX);
        match clicks {
            2 => {
                let word = self.word_under(column);
                self.select(word.start, word.end);
                self.drag = Drag::Words(word);
            }
            3 => {
                self.select_all();
                self.drag = Drag::All;
            }
            _ => {
                let at = self.stop_at(column, side);
                if shift {
                    self.select(self.anchor, at);
                } else {
                    self.select(at, at);
                }
                self.drag = Drag::Clusters;
            }
        }
    }

    /// The pointer moved to the cell `column`, at `side` of it, with the
    /// button down since [`Field::press`]: the selection goes from where it
    /// started to there, with whole words after a double click. The cell can
    /// be outside the field, before its first one or after its last one:
    /// then the field shows more of the text on that side.
    pub fn drag(&mut self, column: isize, side: Side) {
        match self.drag.clone() {
            Drag::None | Drag::All => {}
            Drag::Clusters => {
                let at = self.stop_at(column, side);
                self.select(self.anchor, at);
            }
            Drag::Words(word) => {
                let under = self.word_under(column);
                if under.start >= word.start {
                    self.select(word.start, under.end.max(word.end));
                } else {
                    self.select(word.end, under.start);
                }
            }
        }
    }

    /// Moves the anchor to `anchor` and the caret to `caret`. Returns
    /// whether they changed: then the next edit starts another step.
    fn select(&mut self, anchor: usize, caret: usize) -> bool {
        if (anchor, caret) == (self.anchor, self.caret) {
            return false;
        }
        self.track(|field| {
            (field.anchor, field.caret) = (anchor, caret);
            field.history.close();
        });
        true
    }

    /// The place where the caret stops that is nearest to `side` of the
    /// cell `column` of the field, which can be outside it. On a cluster of
    /// several cells, the first half of its cells goes before it, and the
    /// second half after it.
    fn stop_at(&self, column: isize, side: Side) -> usize {
        let Some(cell) = self.cell_of(column) else {
            return 0;
        };
        // In halves of a cell.
        let half = 2 * cell + usize::from(side == Side::Right);
        let mut start = 0;
        for (at, cluster) in self.text.grapheme_indices(true) {
            let width = text::width(cluster);
            if half < 2 * start + width {
                return at;
            }
            start += width;
        }
        self.text.len()
    }

    /// The run of clusters of the same class as the one in the cell
    /// `column` of the field, which can be outside it: the last cluster,
    /// after the end of the text.
    fn word_under(&self, column: isize) -> Range<usize> {
        let cell = self.cell_of(column).unwrap_or(0);
        let mut start = 0;
        let mut last = None;
        for (at, cluster) in self.text.grapheme_indices(true) {
            let width = text::width(cluster);
            if width > 0 {
                last = Some(at);
                if cell < start + width {
                    break;
                }
            }
            start += width;
        }
        last.map_or(self.text.len()..self.text.len(), |at| {
            word_at(&self.text, at)
        })
    }

    /// The cell of the text in the cell `column` of the field, or `None`
    /// before the start of the text.
    fn cell_of(&self, column: isize) -> Option<usize> {
        self.scroll.checked_add_signed(column)
    }

    /// Replaces `range` of the text with `with`, which fits, and puts the
    /// caret at the end of what it inserted. It keeps the edit, as `kind`,
    /// so that it can be undone. `range` or `with` is not empty.
    fn replace(&mut self, range: Range<usize>, with: &str, kind: Kind) {
        let removed = self.text[range.clone()].to_owned();
        let before = (self.anchor, self.caret);
        self.text.replace_range(range.clone(), with);
        self.edited();
        // What it inserted can join the text around it in another cluster:
        // the caret goes to its end, or, after deleting, to its start.
        let end = range.start + with.len();
        let caret = if is_stop(&self.text, end) {
            end
        } else if with.is_empty() {
            previous_cluster(&self.text, end)
        } else {
            next_cluster(&self.text, end)
        };
        (self.anchor, self.caret) = (caret, caret);
        self.history.record(Edit {
            at: range.start,
            removed,
            inserted: with.to_owned(),
            before,
            after: caret,
            kind,
        });
    }

    /// The text changed: a button held down since before selects nothing
    /// more, because the places that it took are not the same any more.
    fn edited(&mut self) {
        self.changes.text = true;
        self.drag = Drag::None;
    }

    /// Undoes the last step. Returns whether there was one.
    fn undo(&mut self) -> bool {
        let Some(edit) = self.history.undo() else {
            return false;
        };
        let Edit {
            at,
            ref removed,
            ref inserted,
            before,
            ..
        } = *edit;
        self.text.replace_range(at..at + inserted.len(), removed);
        (self.anchor, self.caret) = before;
        self.edited();
        true
    }

    /// Does again the last step undone. Returns whether there was one.
    fn redo(&mut self) -> bool {
        let Some(edit) = self.history.redo() else {
            return false;
        };
        let Edit {
            at,
            ref removed,
            ref inserted,
            after,
            ..
        } = *edit;
        self.text.replace_range(at..at + removed.len(), inserted);
        (self.anchor, self.caret) = (after, after);
        self.edited();
        true
    }

    /// The UTF-16 code units that fit in place of the selection.
    fn room(&self) -> usize {
        let selected = units(self.selected_text());
        self.limit.saturating_sub(units(&self.text) - selected)
    }

    /// Runs `change`, and then brings the part of the text that the field
    /// shows up to date, and notes what changed.
    fn track<R>(&mut self, change: impl FnOnce(&mut Self) -> R) -> R {
        let before = (self.anchor, self.caret, self.scroll);
        let result = change(self);
        self.follow_caret();
        if self.changes.text || (self.anchor, self.caret, self.scroll) != before {
            self.changes.shown = true;
        }
        result
    }

    /// Moves the part of the text that the field shows just enough to show
    /// the caret, which needs a cell after the text when it is at its end;
    /// never further than that, when the text gets shorter; and never with
    /// half a wide character on the left.
    fn follow_caret(&mut self) {
        if self.columns == 0 {
            self.scroll = 0;
            return;
        }
        let caret = text::width(&self.text[..self.caret]);
        let total = text::width(&self.text);
        let mut scroll = self
            .scroll
            .clamp((caret + 1).saturating_sub(self.columns), caret)
            .min((total + 1).saturating_sub(self.columns));
        if matches!(
            text::cells(&self.text).nth(scroll),
            Some(CellChar::WideRight(_))
        ) {
            scroll += 1;
        }
        self.scroll = scroll;
    }
}

/// `text` in a single line: each line break (CR LF, CR, LF, VT, FF, NEL,
/// LS or PS) and each tab becomes a space, and the other control characters
/// go away.
fn one_line(text: &str) -> String {
    let mut line = String::with_capacity(text.len());
    let mut after_cr = false;
    for c in text.chars() {
        match c {
            '\n' if after_cr => {}
            '\r' | '\n' | '\t' | '\u{b}' | '\u{c}' | '\u{85}' | '\u{2028}' | '\u{2029}' => {
                line.push(' ');
            }
            c if c.is_control() => {}
            c => line.push(c),
        }
        after_cr = c == '\r';
    }
    line
}

/// The longest start of `text`, in whole clusters, that has at most `room`
/// UTF-16 code units.
fn fitting(text: &str, room: usize) -> &str {
    let mut units_so_far = 0;
    let mut end = 0;
    for (at, cluster) in text.grapheme_indices(true) {
        units_so_far += units(cluster);
        if units_so_far > room {
            break;
        }
        end = at + cluster.len();
    }
    &text[..end]
}

/// How many UTF-16 code units `text` has.
fn units(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

#[cfg(test)]
mod tests;
