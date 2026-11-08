//! Text as cells of a grid: which characters take one cell, which take two,
//! and which take none.

use unicode_width::UnicodeWidthChar;

/// What a cell of the grid shows.
///
/// A character that takes two cells, such as most CJK ideographs, is split
/// in two halves, so that each cell can be drawn on its own: when only one of
/// the two changes, only that one is drawn again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CellChar {
    /// Only the background, as a space.
    #[default]
    Blank,
    /// A character that takes this cell.
    Narrow(char),
    /// The left half of a character that takes this cell and the next one.
    WideLeft(char),
    /// The right half of a character that takes the previous cell and this
    /// one.
    WideRight(char),
}

/// The cells of `text`, one after the other.
///
/// The width of each character is the one that `unicode-width` gives, which
/// is the one that `alacritty_terminal` uses for the grid of the terminal.
/// Control characters and the characters without width (such as combining
/// accents) take no cell: joining them to the previous character, as part of
/// a grapheme, comes in subphase 8.1. Precomposed letters, such as `á`, are a
/// single character.
pub fn cells(text: &str) -> impl Iterator<Item = CellChar> + '_ {
    text.chars().flat_map(|c| {
        let (first, second) = match c.width() {
            Some(1) if c == ' ' => (Some(CellChar::Blank), None),
            Some(1) => (Some(CellChar::Narrow(c)), None),
            Some(2) => (Some(CellChar::WideLeft(c)), Some(CellChar::WideRight(c))),
            _ => (None, None),
        };
        first.into_iter().chain(second)
    })
}

/// How many cells `text` takes.
pub fn width(text: &str) -> usize {
    cells(text).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(text: &str) -> Vec<CellChar> {
        cells(text).collect()
    }

    #[test]
    fn letters_take_one_cell_and_spaces_are_blank() {
        assert_eq!(
            layout("a ñ€"),
            [
                CellChar::Narrow('a'),
                CellChar::Blank,
                CellChar::Narrow('ñ'),
                CellChar::Narrow('€'),
            ]
        );
    }

    #[test]
    fn wide_characters_take_two_cells() {
        assert_eq!(
            layout("中a😀"),
            [
                CellChar::WideLeft('中'),
                CellChar::WideRight('中'),
                CellChar::Narrow('a'),
                CellChar::WideLeft('😀'),
                CellChar::WideRight('😀'),
            ]
        );
        assert_eq!(width("日本語 abc"), 10);
    }

    #[test]
    fn control_and_zero_width_characters_take_no_cell() {
        // "e" followed by a combining acute accent, a tab and a zero-width
        // space.
        assert_eq!(
            layout("e\u{301}\t\u{200b}x"),
            [CellChar::Narrow('e'), CellChar::Narrow('x'),]
        );
        assert_eq!(width(""), 0);
    }
}
