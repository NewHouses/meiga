//! The selection: the text that the user marks with the mouse, which the
//! screen shows and the clipboard takes.
//!
//! `alacritty_terminal` keeps it in its grid, so it follows the text: it
//! moves up with it when the screen scrolls, and goes away when the
//! program clears the part of the screen where it is, switches to the
//! alternate screen or resets the terminal. Writing over its cells does not
//! remove it.

use alacritty_terminal::index::{Column, Direction, Line, Point};
use alacritty_terminal::selection::{Selection, SelectionRange, SelectionType};
use alacritty_terminal::term::{self, cell::Flags};

use super::{Columns, Terminal};
use crate::geometry::GridPoint;

/// The characters that end a word, for a double click: the spaces, the
/// quotes, the brackets and a few separators. A path, a URL or a position
/// in a file, such as `C:\x\f.rs:12:5`, is a single word.
pub(super) const WORD_DELIMITERS: &str = " \t\"'`()[]{}<>,;|│";

/// How a selection grows from where it starts: as the clicks that start
/// it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionKind {
    /// A click: from a side of a cell to a side of another one.
    Cells,
    /// A double click: whole words, which end at the spaces, the quotes,
    /// the brackets and a few separators ([`WORD_DELIMITERS`]). On a
    /// bracket, up to the one that closes or opens it; on one of those
    /// characters, only its cell.
    Words,
    /// A triple click: whole lines, with all the rows of a line that did
    /// not fit in one.
    Lines,
}

/// A half of a cell. A selection of cells starts and ends at a side of a
/// cell: a cell is in it once it goes past the middle of the cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Terminal {
    /// Starts a selection of `kind` at `side` of the cell `point` of the
    /// screen, instead of the one that there was. A selection of cells is
    /// empty until it ends at another side; one of words or of lines
    /// already has the word or the line of `point`.
    pub fn select(&mut self, kind: SelectionKind, point: GridPoint, side: Side) {
        let point = self.grid_point(point);
        let selection = match kind {
            SelectionKind::Words => self.word_at(point),
            SelectionKind::Cells => Selection::new(SelectionType::Simple, point, side.into()),
            SelectionKind::Lines => Selection::new(SelectionType::Lines, point, side.into()),
        };
        self.term.selection = Some(selection);
    }

    /// Moves the end of the selection to `side` of the cell `point`: the
    /// selection goes from where it started to there, with whole words or
    /// lines if it is of words or of lines. Without a selection, it does
    /// nothing.
    pub fn select_to(&mut self, point: GridPoint, side: Side) {
        let point = self.grid_point(point);
        if let Some(selection) = &mut self.term.selection {
            selection.update(point, side.into());
        }
    }

    /// Removes the selection. Returns whether there was one with cells.
    pub fn clear_selection(&mut self) -> bool {
        let had = self.has_selection();
        self.term.selection = None;
        had
    }

    /// Whether there is a selection with cells: a click that started one
    /// and did not move does not have any.
    pub fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }

    /// The text of the selection, if it has any: its rows end with LF,
    /// except the last one and the ones of a line that did not fit in one
    /// row, without the blanks at their end. A wide character comes once.
    pub fn selection_text(&self) -> Option<String> {
        let mut text = self.term.selection_to_string()?;
        // alacritty_terminal ends a selection of lines with LF; pasted
        // into a shell, it would run the line.
        if text.ends_with('\n') {
            text.pop();
        }
        Some(text).filter(|text| !text.is_empty())
    }

    /// The cells of the selection, in the grid of `alacritty_terminal`, if
    /// it has any.
    pub(super) fn selection_range(&self) -> Option<SelectionRange> {
        self.term
            .selection
            .as_ref()
            .and_then(|selection| selection.to_range(&self.term))
    }

    /// The selection of a double click on `point`.
    fn word_at(&self, point: Point) -> Selection {
        // As alacritty_terminal does, but only once: it would search for
        // the other bracket at every frame, maybe through all the history.
        if let Some(other) = self.term.bracket_search(point) {
            return cells(point.min(other), point.max(other));
        }
        // alacritty_terminal would also take the words on both sides.
        let cell = &self.term.grid()[point];
        if !cell
            .flags
            .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
            && WORD_DELIMITERS.contains(cell.c)
        {
            return cells(point, point);
        }
        Selection::new(SelectionType::Semantic, point, Direction::Left)
    }

    /// The point of the grid of `alacritty_terminal` of the cell `point` of
    /// the screen, or of the nearest one inside it.
    fn grid_point(&self, point: GridPoint) -> Point {
        let row = point.row.min(self.size.rows - 1);
        let column = point.column.min(self.size.columns - 1);
        term::viewport_to_point(
            self.term.grid().display_offset(),
            Point::new(usize::from(row), Column(usize::from(column))),
        )
    }
}

/// A selection of cells from `start` to `end`, both included.
fn cells(start: Point, end: Point) -> Selection {
    let mut selection = Selection::new(SelectionType::Simple, start, Direction::Left);
    selection.update(end, Direction::Right);
    selection
}

impl From<Side> for Direction {
    fn from(side: Side) -> Self {
        match side {
            Side::Left => Direction::Left,
            Side::Right => Direction::Right,
        }
    }
}

/// The columns that `range` selects in the row of the grid at `line`, of
/// `columns` columns.
pub(super) fn selected_columns(range: &SelectionRange, line: Line, columns: u16) -> Columns {
    if line < range.start.line || line > range.end.line {
        return Columns::default();
    }
    // The columns of the grid are the ones of the screen, so they fit in a
    // u16.
    let column = |column: Column| column.0.min(usize::from(columns)) as u16;
    let start = if line == range.start.line {
        column(range.start.column)
    } else {
        0
    };
    let end = if line == range.end.line {
        column(range.end.column).saturating_add(1)
    } else {
        columns
    };
    Columns::new(start, end.min(columns))
}

/// The rows of the screen, of `rows`, with cells of `range`, when the
/// screen shows the grid from `display_offset` lines above its end.
pub(super) fn shown_rows(
    range: &SelectionRange,
    display_offset: usize,
    rows: u16,
) -> std::ops::Range<u16> {
    let offset = i32::try_from(display_offset).unwrap_or(i32::MAX);
    let row = |line: i32| line.saturating_add(offset).clamp(0, i32::from(rows)) as u16;
    row(range.start.line.0)..row(range.end.line.0.saturating_add(1))
}
