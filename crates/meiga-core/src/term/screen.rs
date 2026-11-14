//! What the terminal shows, in meiga's own types: the cells of the grid,
//! their colours and styles, the cursor, and the cells that changed. The
//! renderer draws from it without knowing `alacritty_terminal`.

use std::ops::BitOr;

use crate::color::{Palette, Rgb};
use crate::geometry::GridSize;
use crate::text::CellChar;

/// The colour of the text or of the background of a cell, as the program
/// gave it. The palette of the screen ([`Screen::palette`]) turns it into an
/// [`Rgb`] when it is drawn: [`Color::rgb`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Color {
    /// The default colour: the one of the text, for the text, and the one of
    /// the background, for the background.
    #[default]
    Default,
    /// One of the 256 colours of the palette: 0–7 are the normal ones, 8–15
    /// their bright versions, 16–231 a cube of 6 × 6 × 6 and 232–255 greys.
    Indexed(u8),
    /// A colour given by its components (*truecolor*).
    Rgb(Rgb),
}

impl Color {
    /// The colour to draw with `palette`: `default` if it is
    /// [`Color::Default`].
    pub fn rgb(self, palette: &Palette, default: Rgb) -> Rgb {
        match self {
            Color::Default => default,
            Color::Indexed(index) => palette.colors[usize::from(index)],
            Color::Rgb(rgb) => rgb,
        }
    }
}

/// The styles of a cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CellFlags(u16);

impl CellFlags {
    pub const BOLD: Self = Self(1);
    pub const DIM: Self = Self(1 << 1);
    pub const ITALIC: Self = Self(1 << 2);
    pub const UNDERLINE: Self = Self(1 << 3);
    pub const DOUBLE_UNDERLINE: Self = Self(1 << 4);
    pub const UNDERCURL: Self = Self(1 << 5);
    pub const DOTTED_UNDERLINE: Self = Self(1 << 6);
    pub const DASHED_UNDERLINE: Self = Self(1 << 7);
    pub const STRIKEOUT: Self = Self(1 << 8);
    /// The colours of the text and of the background are swapped.
    pub const INVERSE: Self = Self(1 << 9);
    /// The text is not drawn, only the background.
    pub const HIDDEN: Self = Self(1 << 10);
    /// The cell is in the selection: its background has the colour of the
    /// selection.
    pub const SELECTED: Self = Self(1 << 11);

    pub const fn empty() -> Self {
        Self(0)
    }

    /// Whether it has all the styles of `other`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for CellFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// A cell of the grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Cell {
    pub char: CellChar,
    pub fg: Color,
    pub bg: Color,
    pub flags: CellFlags,
}

/// How the cursor is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CursorShape {
    #[default]
    Block,
    Underline,
    /// A vertical bar.
    Beam,
    /// The outline of a block.
    HollowBlock,
    /// It is not drawn.
    Hidden,
}

/// The cursor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Cursor {
    pub row: u16,
    pub column: u16,
    pub shape: CursorShape,
    /// Whether it blinks: by default it does, unless the program asks for a
    /// steady one. meiga only makes it blink if Windows allows it.
    pub blinking: bool,
}

/// Columns of a row: from `start`, included, to `end`, excluded. When `end`
/// is not after `start`, it has none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Columns {
    pub start: u16,
    pub end: u16,
}

impl Columns {
    pub const fn new(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    pub const fn is_empty(self) -> bool {
        self.end <= self.start
    }

    pub const fn contains(self, column: u16) -> bool {
        self.start <= column && column < self.end
    }

    /// The columns of both.
    fn union(self, other: Columns) -> Columns {
        if self.is_empty() {
            other
        } else if other.is_empty() {
            self
        } else {
            Columns::new(self.start.min(other.start), self.end.max(other.end))
        }
    }
}

/// A copy of what the terminal shows, and of the cells that changed since
/// the last [`clear_changes`](Screen::clear_changes).
///
/// It is meant to be kept between frames: the terminal only copies to it
/// the cells that changed, and its memory is reused.
#[derive(Clone, Debug, Default)]
pub struct Screen {
    size: GridSize,
    /// The cells, row after row.
    cells: Vec<Cell>,
    /// The columns that changed in each row.
    changed: Vec<Columns>,
    /// Every cell changed, for example because the size or the palette
    /// changed.
    all_changed: bool,
    cursor: Cursor,
    palette: Palette,
}

impl Screen {
    pub const fn new() -> Self {
        Self {
            size: GridSize::new(0, 0),
            cells: Vec::new(),
            changed: Vec::new(),
            all_changed: false,
            cursor: Cursor {
                row: 0,
                column: 0,
                shape: CursorShape::Block,
                blinking: false,
            },
            palette: Palette::MEIGA,
        }
    }

    pub const fn size(&self) -> GridSize {
        self.size
    }

    /// The cells of `row`, or none if it is outside.
    pub fn row(&self, row: u16) -> &[Cell] {
        let columns = usize::from(self.size.columns);
        let start = usize::from(row) * columns;
        self.cells.get(start..start + columns).unwrap_or_default()
    }

    pub const fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// The colours with which the cells are drawn: the ones of the theme,
    /// with the changes of the program.
    pub const fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Whether every cell changed.
    pub const fn all_changed(&self) -> bool {
        self.all_changed
    }

    /// The rows that changed, with their columns that changed.
    pub fn changes(&self) -> impl Iterator<Item = (u16, Columns)> + '_ {
        let all = Columns::new(0, self.size.columns);
        self.changed
            .iter()
            .zip(0..self.size.rows)
            .filter_map(move |(&columns, row)| {
                let columns = if self.all_changed { all } else { columns };
                (!columns.is_empty()).then_some((row, columns))
            })
    }

    /// Forgets the changes, once they are drawn.
    pub fn clear_changes(&mut self) {
        self.all_changed = false;
        self.changed.fill(Columns::default());
    }

    /// The text of `row`, without the blanks at its end: each character
    /// once, also the wide ones, and the blank cells as spaces.
    pub fn row_text(&self, row: u16) -> String {
        let mut text: String = self
            .row(row)
            .iter()
            .filter_map(|cell| match cell.char {
                CellChar::Blank => Some(' '),
                CellChar::Narrow(c) | CellChar::WideLeft(c) => Some(c),
                CellChar::WideRight(_) => None,
            })
            .collect();
        text.truncate(text.trim_end_matches(' ').len());
        text
    }

    /// Forgets what it shows: the next copy of a terminal copies all of it,
    /// as for a screen of another size, in the same memory. For when it
    /// starts to show another terminal.
    pub fn forget(&mut self) {
        self.size = GridSize::new(0, 0);
    }

    /// Takes `size`, with every cell blank and changed.
    pub(super) fn resize(&mut self, size: GridSize) {
        self.size = size;
        self.cells.clear();
        self.cells.resize(
            usize::from(size.columns) * usize::from(size.rows),
            Cell::default(),
        );
        self.changed.clear();
        self.changed
            .resize(usize::from(size.rows), Columns::default());
        self.all_changed = true;
    }

    /// The cells of `row` from `columns.start`, which have to fit in it.
    pub(super) fn cells_mut(&mut self, row: u16, columns: Columns) -> &mut [Cell] {
        let start = usize::from(row) * usize::from(self.size.columns);
        self.cells
            .get_mut(start + usize::from(columns.start)..start + usize::from(columns.end))
            .unwrap_or_default()
    }

    pub(super) fn mark_changed(&mut self, row: u16, columns: Columns) {
        if let Some(changed) = self.changed.get_mut(usize::from(row)) {
            *changed = changed.union(columns);
        }
    }

    pub(super) fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    /// Adds to the changes the cells of the cursor, if it is drawn: two, in
    /// case it is over a wide character. For when it changes how it looks
    /// without moving, for example when it blinks.
    pub fn mark_cursor_changed(&mut self) {
        let Cursor {
            row, column, shape, ..
        } = self.cursor;
        if shape != CursorShape::Hidden {
            let end = column.saturating_add(2).min(self.size.columns);
            self.mark_changed(row, Columns::new(column, end));
        }
    }

    /// Takes `palette`. If its colours change, every cell changes; if only
    /// the one of the cursor does, only the cells of the cursor.
    pub(super) fn set_palette(&mut self, palette: &Palette) {
        if self.palette.colors != palette.colors
            || self.palette.foreground != palette.foreground
            || self.palette.background != palette.background
            || self.palette.selection != palette.selection
        {
            self.all_changed = true;
        } else if self.palette.cursor != palette.cursor {
            self.mark_cursor_changed();
        } else {
            return;
        }
        self.palette.clone_from(palette);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_screen_is_empty() {
        let screen = Screen::new();
        assert_eq!(screen.size(), GridSize::new(0, 0));
        assert!(screen.row(0).is_empty());
        assert_eq!(screen.changes().count(), 0);
    }

    #[test]
    fn after_resizing_every_cell_is_blank_and_changed() {
        let mut screen = Screen::new();
        screen.resize(GridSize::new(4, 2));
        assert_eq!(screen.row(1), [Cell::default(); 4]);
        assert!(screen.row(2).is_empty());
        assert!(screen.all_changed());
        let changes: Vec<_> = screen.changes().collect();
        assert_eq!(changes, [(0, Columns::new(0, 4)), (1, Columns::new(0, 4))]);
        screen.clear_changes();
        assert_eq!(screen.changes().count(), 0);
    }

    #[test]
    fn changes_add_up_until_they_are_cleared() {
        let mut screen = Screen::new();
        screen.resize(GridSize::new(10, 3));
        screen.clear_changes();
        screen.mark_changed(1, Columns::new(2, 4));
        screen.mark_changed(1, Columns::new(6, 7));
        screen.mark_changed(9, Columns::new(0, 1));
        let changes: Vec<_> = screen.changes().collect();
        assert_eq!(changes, [(1, Columns::new(2, 7))]);
    }

    #[test]
    fn the_text_of_a_row_shows_each_character_once() {
        let mut screen = Screen::new();
        screen.resize(GridSize::new(6, 1));
        let chars = [
            CellChar::Narrow('a'),
            CellChar::Blank,
            CellChar::WideLeft('中'),
            CellChar::WideRight('中'),
            CellChar::Blank,
            CellChar::Blank,
        ];
        for (cell, char) in screen
            .cells_mut(0, Columns::new(0, 6))
            .iter_mut()
            .zip(chars)
        {
            cell.char = char;
        }
        assert_eq!(screen.row_text(0), "a 中");
    }

    #[test]
    fn flags_combine() {
        let flags = CellFlags::BOLD | CellFlags::ITALIC;
        assert!(flags.contains(CellFlags::BOLD));
        assert!(flags.contains(CellFlags::BOLD | CellFlags::ITALIC));
        assert!(!flags.contains(CellFlags::UNDERLINE));
        assert!(CellFlags::empty().is_empty());
    }
}
