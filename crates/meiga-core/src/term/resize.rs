//! The new size of the screen, as the console host of ConPTY and Windows
//! Terminal place it, so that the rows of the program and those of the
//! terminal stay the same (ADR 0016).
//!
//! When the size changes, ConPTY reflows its buffer, which has no history:
//! the text stays at the top, and it grows downwards. Then it asks the
//! terminal where its cursor is (`CSI 6n`), and takes that position.
//! alacritty_terminal reflows the screen and the history together, and keeps
//! the cursor at the bottom: when the screen grows, it brings back lines from
//! the history, and when lines wrap, it pushes the top ones into it, even with
//! blank rows below. Then what the program writes lands on other rows.
//!
//! So, after alacritty_terminal reflows, the screen moves to where Windows
//! Terminal puts it (`Terminal::UserResize`): its first row is the one where
//! the old first row went, unless the text and the cursor no longer fit below
//! it; then its last row is the last one with text, or the one of the cursor,
//! whichever is lower; and the cursor always stays on the screen. The lines
//! that went into the history stay there.
//!
//! To know where the old first row and the cursor went, the resize marks
//! their cells before alacritty_terminal reflows. It cannot trust the cursor
//! of alacritty_terminal: when many rows below it wrap, they push it above
//! the screen, and alacritty_terminal leaves it on the first row, which is no
//! longer its row.
//!
//! On the alternate screen, ConPTY does not reflow the primary one: it
//! resizes it once, to the last size, when the program goes back to it.
//! meiga sets the primary screen aside until then, which also saves reflowing
//! its history while a full-screen program runs.

use std::cmp::{Ordering, max, min};
use std::mem;

use alacritty_terminal::grid::{Dimensions, Grid, Scroll};
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::Term;
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::vte::ansi::Color;

use super::{Listener, SCROLLBACK_LINES, TermSize};
use crate::geometry::GridSize;

/// A flag that alacritty_terminal does not use, the last of its 16: it marks
/// the cells that the resize follows.
const MARK: Flags = Flags::from_bits_retain(0x8000);

/// The character of the cell of the cursor while it is marked. It is a
/// noncharacter, which programs do not write, and it makes the cell
/// occupied: alacritty_terminal keeps it when it reflows, and wraps the
/// blanks before it as text, as ConPTY does with the blanks before the
/// cursor.
const CURSOR: char = '\u{FFFF}';

/// Changes the size of the screen of `term` to `size`, which is not its size,
/// on the primary screen. The selection goes away.
pub(super) fn resize(term: &mut Term<Listener>, size: GridSize) {
    let at_bottom = term.grid().display_offset() == 0;
    let rows = i32::from(size.rows);
    let old_rows = term.grid().screen_lines() as u16;
    let marks = Marks::set(term.grid_mut());
    // The width first, with the same rows: alacritty_terminal removes no row
    // when it reflows.
    if usize::from(size.columns) != term.grid().columns() {
        term.resize(TermSize(GridSize::new(size.columns, old_rows)));
    }
    if let Some(cursor) = marks.find(term.grid()).cursor {
        term.grid_mut().cursor.point = cursor;
    }
    if size.rows < old_rows {
        // alacritty_terminal removes the rows at the bottom: before that, the
        // screen starts where it has to, so that they are the ones that
        // ConPTY removes too.
        let top = window_top(term.grid(), rows, marks.find(term.grid()).top);
        move_screen(term.grid_mut(), top);
        term.resize(TermSize(size));
    } else {
        term.resize(TermSize(size));
        let top = window_top(term.grid(), rows, marks.find(term.grid()).top);
        move_screen(term.grid_mut(), top);
        // Back to the size, without the rows at the bottom.
        let grid = term.grid_mut();
        let columns = grid.columns();
        grid.resize::<Color>(false, usize::from(size.rows), columns);
    }
    let found = marks.find(term.grid());
    marks.remove(term.grid_mut(), found);
    term.selection = None;
    let grid = term.grid_mut();
    grid.scroll_display(if at_bottom {
        Scroll::Bottom
    } else {
        // Where it was, as long as the history still reaches it.
        Scroll::Delta(0)
    });
}

/// Sets the primary screen of `term` aside, while the alternate one is
/// shown, and leaves in its place a blank grid without history, which
/// follows the size of the terminal: until the program goes back to the
/// primary screen, resizing only changes the alternate one.
pub(super) fn set_aside_primary(term: &mut Term<Listener>) -> Grid<Cell> {
    let lines = term.screen_lines();
    let columns = term.columns();
    // alacritty_terminal only gives the primary screen through swap_alt,
    // which clears the grid that becomes the alternate screen: the alternate
    // screen goes out before, and comes back after.
    let alternate = mem::replace(term.grid_mut(), Grid::new(lines, columns, 0));
    term.swap_alt();
    let primary = mem::replace(term.grid_mut(), Grid::new(lines, columns, 0));
    term.swap_alt();
    *term.grid_mut() = alternate;
    primary
}

/// Gives back to `term` the primary screen that was set aside, when the
/// program has just left the alternate screen. As ConPTY does then, it gets
/// the size of the terminal. Returns whether its size changed, which also
/// resets the scrolling region.
pub(super) fn bring_back_primary(term: &mut Term<Listener>, primary: Grid<Cell>) -> bool {
    let size = GridSize::new(term.columns() as u16, term.screen_lines() as u16);
    let fits = primary.columns() == term.columns() && primary.screen_lines() == term.screen_lines();
    *term.grid_mut() = primary;
    if !fits {
        resize(term, size);
    }
    !fits
}

/// After a full reset on the alternate screen: the primary screen that was
/// set aside goes away, and the blank grid that took its place gets a
/// history, as the new primary screen.
pub(super) fn new_primary(term: &mut Term<Listener>) {
    let lines = term.screen_lines();
    let columns = term.columns();
    *term.grid_mut() = Grid::new(lines, columns, SCROLLBACK_LINES);
}

/// The cells that the resize follows: the first one of the screen, to know
/// where the screen started, and the one of the cursor.
struct Marks {
    /// What the cell of the cursor had.
    cursor_char: char,
    /// The cursor was waiting to wrap after the last column: the next
    /// character goes to the next row.
    pending_wrap: bool,
    /// The cursor was on the first cell of the screen: the two marks are
    /// one.
    cursor_first: bool,
}

/// Where the marked cells are.
struct Found {
    /// The row of the cell that was the first one of the screen.
    top: Option<i32>,
    /// The cell of the cursor.
    cursor: Option<Point>,
}

impl Marks {
    fn set(grid: &mut Grid<Cell>) -> Self {
        let first = Point::new(Line(0), Column(0));
        let cursor = grid.cursor.point;
        let pending_wrap = grid.cursor.input_needs_wrap;
        // Writing through the index of the row makes the cell occupied.
        grid[first].flags.insert(MARK);
        let cell = &mut grid[cursor];
        cell.flags.insert(MARK);
        Self {
            cursor_char: mem::replace(&mut cell.c, CURSOR),
            pending_wrap,
            cursor_first: cursor == first,
        }
    }

    /// Where the marked cells are now. They are usually near the screen:
    /// the search goes from the bottom up.
    fn find(&self, grid: &Grid<Cell>) -> Found {
        let mut found = Found {
            top: None,
            cursor: None,
        };
        let top = -(grid.history_size() as i32);
        let rows = grid.screen_lines() as i32;
        for line in (top..rows).rev() {
            for (column, cell) in grid[Line(line)].into_iter().enumerate() {
                if !cell.flags.contains(MARK) {
                    continue;
                }
                let is_cursor = cell.c == CURSOR;
                if is_cursor {
                    found.cursor = Some(Point::new(Line(line), Column(column)));
                }
                if !is_cursor || self.cursor_first {
                    found.top = Some(line);
                }
            }
            if found.top.is_some() && found.cursor.is_some() {
                break;
            }
        }
        found
    }

    /// Gives the cell of the cursor what it had, puts the cursor on it, and
    /// removes the marks.
    fn remove(self, grid: &mut Grid<Cell>, found: Found) {
        if let Some(line) = found.top {
            for cell in &mut grid[Line(line)] {
                cell.flags.remove(MARK);
            }
        }
        let Some(point) = found.cursor else {
            return;
        };
        let cell = &mut grid[point];
        cell.c = self.cursor_char;
        cell.flags.remove(MARK);
        if !(0..grid.screen_lines() as i32).contains(&point.line.0) {
            return;
        }
        let last_column = Column(grid.columns() - 1);
        let cursor = &mut grid.cursor;
        // The next character goes after the one of the cell, maybe on the
        // same row now.
        if self.pending_wrap && point.column < last_column {
            cursor.point = Point::new(point.line, point.column + 1);
            cursor.input_needs_wrap = false;
        } else {
            cursor.point = point;
            cursor.input_needs_wrap = self.pending_wrap;
        }
    }
}

/// Where the screen of `rows` has to start, as a row of the screen or of the
/// history: where the first row went, at `first`, unless the text and the
/// cursor do not fit below it; and never below the cursor.
fn window_top(grid: &Grid<Cell>, rows: i32, first: Option<i32>) -> i32 {
    let cursor = grid.cursor.point.line.0;
    let bottom = last_text_row(grid).map_or(cursor, |last| max(last, cursor));
    let mut top = bottom - rows + 1;
    if let Some(first) = first {
        top = max(top, first);
    }
    min(top, cursor)
}

/// The lowest row of the screen with text, as ConPTY measures it: spaces do
/// not count, even with a colour.
fn last_text_row(grid: &Grid<Cell>) -> Option<i32> {
    let rows = grid.screen_lines() as i32;
    (0..rows).rev().find(|&line| {
        grid[Line(line)]
            .into_iter()
            .any(|cell| !matches!(cell.c, ' ' | '\t'))
    })
}

/// Moves the screen so that its first row is `top`, a row of the screen or
/// of the history. Above it, the rows go into the history, and blank rows
/// come in at the bottom; below it, the screen grows with the rows that come
/// back from the history.
fn move_screen(grid: &mut Grid<Cell>, top: i32) {
    let rows = grid.screen_lines();
    match top.cmp(&0) {
        Ordering::Greater => {
            let count = top as usize;
            grid.scroll_up::<Color>(&(Line(0)..Line(rows as i32)), count);
            grid.cursor.point.line -= count;
            grid.saved_cursor.point.line = max(grid.saved_cursor.point.line - count, Line(0));
        }
        Ordering::Less => {
            let columns = grid.columns();
            grid.resize::<Color>(false, rows + (-top) as usize, columns);
        }
        Ordering::Equal => {}
    }
}
