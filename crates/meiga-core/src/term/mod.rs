//! The terminal: the facade over `alacritty_terminal` (ADR 0003 and ADR
//! 0012). meiga gives it what the program writes, and reads from it what to
//! show and what to answer, in its own types.
//!
//! Only this module uses the types of `alacritty_terminal`: replacing it, or
//! updating it to a version with another API, only changes this module.

pub mod recording;
mod scanner;
mod screen;
mod selection;

use std::mem;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::{Dimensions, Grid};
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::selection::SelectionRange;
use alacritty_terminal::term::cell::{Cell as TermCell, Flags};
use alacritty_terminal::term::{
    self, ClipboardType, Config, LineDamageBounds, Osc52, Term, TermDamage, TermMode,
};
use alacritty_terminal::vte::ansi::{self, CursorStyle, NamedColor, Processor};

pub use screen::{Cell, CellFlags, Color, Columns, Cursor, CursorShape, Screen};
pub use selection::{SelectionKind, Side};

use crate::color::{Palette, Rgb};
use crate::geometry::{GridSize, Size};
use crate::text::CellChar;
use scanner::Scanner;

/// Lines of history kept above the screen, which the scrollback will show
/// (subphase 1.9).
pub const SCROLLBACK_LINES: usize = 10_000;

/// The smallest grid that `alacritty_terminal` supports: a wide character
/// needs two columns.
pub const MIN_SIZE: GridSize =
    GridSize::new(term::MIN_COLUMNS as u16, term::MIN_SCREEN_LINES as u16);

/// The cursor when the program does not choose one: a bar that blinks, as
/// in Windows Terminal.
const DEFAULT_CURSOR: CursorStyle = CursorStyle {
    shape: ansi::CursorShape::Beam,
    blinking: true,
};

/// What happened when the terminal read output, besides changes on the
/// screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    /// There are answers for the program: [`Terminal::take_responses`].
    pub responses: bool,
    /// The title changed: [`Terminal::title`].
    pub title: bool,
    /// The program rang the bell.
    pub bell: bool,
    /// The program copied text to the clipboard (OSC 52):
    /// [`Terminal::take_clipboard`].
    pub clipboard: bool,
}

/// The modes that the program turned on, which change what meiga sends it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    /// win32-input-mode (`?9001`): keys as whole key events.
    pub win32_input: bool,
    /// Focus events (`?1004`): tell the program when the window gains or
    /// loses the focus.
    pub focus_events: bool,
    /// Bracketed paste (`?2004`): mark the start and end of pasted text.
    pub bracketed_paste: bool,
    /// Application cursor keys (`?1`).
    pub app_cursor: bool,
    /// Application keypad (`ESC =`).
    pub app_keypad: bool,
    /// The program wants the clicks of the mouse (`?1000`).
    pub mouse_clicks: bool,
    /// And also the moves while a button is down (`?1002`).
    pub mouse_drag: bool,
    /// And also every move (`?1003`).
    pub mouse_motion: bool,
    /// Mouse reports in the SGR format (`?1006`).
    pub sgr_mouse: bool,
    /// Mouse reports in UTF-8 (`?1005`).
    pub utf8_mouse: bool,
    /// The wheel sends arrow keys on the alternate screen (`?1007`).
    pub alternate_scroll: bool,
    /// The alternate screen is shown (`?1049`), as in full-screen programs.
    pub alt_screen: bool,
    /// The cursor is shown (`?25`).
    pub show_cursor: bool,
    /// Line feed also returns to the first column (LNM, mode 20).
    pub line_feed_new_line: bool,
}

/// The events of `alacritty_terminal`, waiting for the terminal to take
/// them after reading output.
#[derive(Default)]
struct Pending {
    changes: Changes,
    responses: Vec<u8>,
    /// The questions that only the terminal can answer, with where their
    /// answers go among `responses`.
    questions: Vec<(usize, Question)>,
    title: Option<String>,
    clipboard: Option<String>,
}

/// A question of the program that needs what `alacritty_terminal` does not
/// know.
enum Question {
    /// A colour (OSC 4, 10, 11 or 12 with `?`): its index in the colours of
    /// `alacritty_terminal`, and how to write the answer.
    Color(usize, Arc<dyn Fn(ansi::Rgb) -> String + Send + Sync>),
    /// The size of the text area in pixels (`CSI 14 t`).
    TextAreaSize,
}

/// Receives the events of `alacritty_terminal` while it reads output.
///
/// They arrive through `&self`, so they go to a queue that the terminal
/// shares with it. The terminal moves between threads, so the queue is a
/// `Mutex`, which nobody else locks.
struct Listener(Arc<Mutex<Pending>>);

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        let mut pending = lock(&self.0);
        match event {
            // Answers to the program: device attributes (DA1), status and
            // cursor position (DSR), size in cells…
            Event::PtyWrite(text) => {
                pending.responses.extend_from_slice(text.as_bytes());
                pending.changes.responses = true;
            }
            // The terminal answers them once the output is read, in the
            // same order: some programs ask the attributes (DA1) after
            // another question, and take an answer to DA1 first as a
            // terminal that does not know the other one.
            Event::ColorRequest(index, answer) => {
                let at = pending.responses.len();
                pending.questions.push((at, Question::Color(index, answer)));
            }
            Event::TextAreaSizeRequest(_) => {
                let at = pending.responses.len();
                pending.questions.push((at, Question::TextAreaSize));
            }
            Event::Title(title) => {
                // An empty title is no title: the tab shows its own.
                pending.title = Some(title).filter(|title| !title.is_empty());
                pending.changes.title = true;
            }
            Event::ResetTitle => {
                pending.title = None;
                pending.changes.title = true;
            }
            Event::Bell => pending.changes.bell = true,
            // Windows has no primary selection: only the clipboard.
            Event::ClipboardStore(ClipboardType::Clipboard, text) => {
                pending.clipboard = Some(text);
                pending.changes.clipboard = true;
            }
            // Reading the clipboard is never allowed (Osc52::OnlyCopy). The
            // rest is for the event loop of Alacritty, which meiga does not
            // use.
            _ => {}
        }
    }
}

/// The size of the grid as `alacritty_terminal` asks for it.
struct TermSize(GridSize);

impl Dimensions for TermSize {
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }

    fn screen_lines(&self) -> usize {
        usize::from(self.0.rows)
    }

    fn columns(&self) -> usize {
        usize::from(self.0.columns)
    }
}

/// A terminal: the screen, its history, and the state that the program
/// changes with control sequences.
pub struct Terminal {
    term: Term<Listener>,
    parser: Processor,
    scanner: Scanner,
    pending: Arc<Mutex<Pending>>,
    size: GridSize,
    title: Option<String>,
    /// The colours of the theme, which the program can change.
    theme: Palette,
    /// The size of a cell in pixels, or 0 × 0 while it is not known.
    cell: Size,
    /// The damaged rows, reused at every copy.
    damaged: Vec<LineDamageBounds>,
    /// The selection that the screen shows, as the last copy left it:
    /// `alacritty_terminal` does not damage the cells of the selection when
    /// it changes.
    shown_selection: Option<SelectionRange>,
}

// The reader thread of a tab feeds its terminal, and the window thread copies
// from it: it has to be able to go from one thread to another.
const _: () = {
    const fn send<T: Send>() {}
    send::<Terminal>();
};

impl Terminal {
    /// A terminal of `size`, with at least [`MIN_SIZE`], and the colours of
    /// meiga.
    pub fn new(size: GridSize) -> Self {
        let size = at_least_min_size(size);
        let pending = Arc::new(Mutex::new(Pending::default()));
        let config = Config {
            scrolling_history: SCROLLBACK_LINES,
            default_cursor_style: DEFAULT_CURSOR,
            osc52: Osc52::OnlyCopy,
            kitty_keyboard: false,
            semantic_escape_chars: selection::WORD_DELIMITERS.to_owned(),
            ..Config::default()
        };
        Self {
            term: Term::new(config, &TermSize(size), Listener(Arc::clone(&pending))),
            parser: Processor::new(),
            scanner: Scanner::new(),
            pending,
            size,
            title: None,
            theme: Palette::MEIGA,
            cell: Size::new(0, 0),
            damaged: Vec::new(),
            shown_selection: None,
        }
    }

    pub const fn size(&self) -> GridSize {
        self.size
    }

    /// Changes the colours of the theme. The ones that the program changed
    /// keep its colour.
    pub fn set_theme(&mut self, theme: Palette) {
        self.theme = theme;
    }

    /// Gives the size of a cell in pixels, with which the terminal answers
    /// the programs that ask for the size of the text area in pixels (`CSI
    /// 14 t`). Until it has it, it does not answer them.
    pub fn set_cell_size(&mut self, cell: Size) {
        self.cell = cell;
    }

    /// The colours with which the cells are drawn: the ones of the theme,
    /// except those that the program changed (OSC 4, 10, 11 and 12).
    pub fn palette(&self) -> Palette {
        let mut palette = self.theme.clone();
        let colors = self.term.colors();
        for (index, color) in palette.colors.iter_mut().enumerate() {
            if let Some(changed) = colors[index] {
                *color = rgb(changed);
            }
        }
        for (name, color) in [
            (NamedColor::Foreground, &mut palette.foreground),
            (NamedColor::Background, &mut palette.background),
            (NamedColor::Cursor, &mut palette.cursor),
        ] {
            if let Some(changed) = colors[name] {
                *color = rgb(changed);
            }
        }
        palette
    }

    /// Reads what the program wrote.
    pub fn feed(&mut self, bytes: &[u8]) -> Changes {
        self.scanner.scan(bytes);
        self.parser.advance(&mut self.term, bytes);
        self.take_changes()
    }

    /// Moves the answers for the program to the end of `into`, to send them
    /// to it.
    pub fn take_responses(&mut self, into: &mut Vec<u8>) {
        into.append(&mut lock(&self.pending).responses);
    }

    /// The text that the program last copied to the clipboard.
    pub fn take_clipboard(&mut self) -> Option<String> {
        lock(&self.pending).clipboard.take()
    }

    /// The title that the program gave, if it gave one.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn modes(&self) -> Modes {
        let mode = *self.term.mode();
        Modes {
            win32_input: self.scanner.win32_input_mode(),
            focus_events: mode.contains(TermMode::FOCUS_IN_OUT),
            bracketed_paste: mode.contains(TermMode::BRACKETED_PASTE),
            app_cursor: mode.contains(TermMode::APP_CURSOR),
            app_keypad: mode.contains(TermMode::APP_KEYPAD),
            mouse_clicks: mode.contains(TermMode::MOUSE_REPORT_CLICK),
            mouse_drag: mode.contains(TermMode::MOUSE_DRAG),
            mouse_motion: mode.contains(TermMode::MOUSE_MOTION),
            sgr_mouse: mode.contains(TermMode::SGR_MOUSE),
            utf8_mouse: mode.contains(TermMode::UTF8_MOUSE),
            alternate_scroll: mode.contains(TermMode::ALTERNATE_SCROLL),
            alt_screen: mode.contains(TermMode::ALT_SCREEN),
            show_cursor: mode.contains(TermMode::SHOW_CURSOR),
            line_feed_new_line: mode.contains(TermMode::LINE_FEED_NEW_LINE),
        }
    }

    /// Changes the size of the grid, with at least [`MIN_SIZE`].
    pub fn resize(&mut self, size: GridSize) {
        self.size = at_least_min_size(size);
        self.term.resize(TermSize(self.size));
    }

    /// When the synchronized update that the program started (`?2026`) has
    /// to be shown, even if the program does not end it. Until then, the
    /// output after it does not reach the screen.
    pub fn sync_deadline(&self) -> Option<Instant> {
        self.parser.sync_timeout().sync_timeout()
    }

    /// Shows the output of the synchronized update that is waiting. It is
    /// for when its deadline passes: the terminal does not have a timer.
    pub fn end_sync(&mut self) -> Changes {
        self.parser.stop_sync(&mut self.term);
        self.take_changes()
    }

    /// Copies to `screen` the rows that changed since the last copy, the
    /// cursor and the palette, and adds to the changes of `screen` the cells
    /// that are now different. When the cursor moves, or changes its shape,
    /// the changes also have its cells, where it was and where it is, if it
    /// is drawn: the renderer draws it over them. When the palette changes,
    /// every cell changes. The cells of the selection have
    /// [`CellFlags::SELECTED`].
    pub fn copy_damage(&mut self, screen: &mut Screen) {
        let resized = screen.size() != self.size;
        if resized {
            screen.resize(self.size);
        }
        self.damaged.clear();
        let full = match self.term.damage() {
            TermDamage::Full => true,
            TermDamage::Partial(lines) => {
                self.damaged.extend(lines);
                resized
            }
        };
        let selection = self.selection_range();
        let grid = self.term.grid();
        if full {
            for row in 0..self.size.rows {
                copy_row(grid, screen, row, selection.as_ref());
            }
        } else {
            // alacritty_terminal damages the cells that it writes, but not
            // always others of the same row that change with them, like the
            // other half of a wide character: the whole row is compared.
            for damage in &self.damaged {
                match u16::try_from(damage.line) {
                    Ok(row) if row < self.size.rows => {
                        copy_row(grid, screen, row, selection.as_ref());
                    }
                    _ => {}
                }
            }
            // Nor does it damage the cells of the selection when the
            // selection changes. When it moves with the text, everything is
            // damaged.
            if selection != self.shown_selection {
                let display_offset = grid.display_offset();
                for range in [&self.shown_selection, &selection].into_iter().flatten() {
                    for row in selection::shown_rows(range, display_offset, self.size.rows) {
                        copy_row(grid, screen, row, selection.as_ref());
                    }
                }
            }
        }
        self.shown_selection = selection;
        let cursor = self.cursor();
        if cursor != screen.cursor() {
            screen.mark_cursor_changed();
            screen.set_cursor(cursor);
            screen.mark_cursor_changed();
        }
        // When a colour changes, alacritty_terminal damages every row, but
        // the cells keep the index of their colour, so they compare equal:
        // the palette is compared on its own.
        screen.set_palette(&self.palette());
        self.term.reset_damage();
    }

    /// Where the cursor is now, and how the program wants it.
    pub fn cursor(&self) -> Cursor {
        let style = self.term.cursor_style();
        let shape = if self.term.mode().contains(TermMode::SHOW_CURSOR) {
            match style.shape {
                ansi::CursorShape::Block => CursorShape::Block,
                ansi::CursorShape::Underline => CursorShape::Underline,
                ansi::CursorShape::Beam => CursorShape::Beam,
                ansi::CursorShape::HollowBlock => CursorShape::HollowBlock,
                ansi::CursorShape::Hidden => CursorShape::Hidden,
            }
        } else {
            CursorShape::Hidden
        };
        // Not from renderable_content(): on a right half in the first column,
        // left alone when deleting characters, it subtracts below zero.
        let grid = self.term.grid();
        let mut point = grid.cursor.point;
        // On the right half of a wide character, it is drawn over all of it,
        // from its left half, as copy_row shows them.
        if point.column.0 > 0
            && grid[point].flags.contains(Flags::WIDE_CHAR_SPACER)
            && grid[point.line][point.column - 1]
                .flags
                .contains(Flags::WIDE_CHAR)
        {
            point.column -= 1;
        }
        let point = term::point_to_viewport(grid.display_offset(), point);
        match point.and_then(|point| {
            Some((
                u16::try_from(point.line).ok()?,
                u16::try_from(point.column.0).ok()?,
            ))
        }) {
            Some((row, column)) if row < self.size.rows => Cursor {
                row,
                column,
                shape,
                blinking: style.blinking,
            },
            // Above or below what is shown.
            _ => Cursor {
                shape: CursorShape::Hidden,
                blinking: style.blinking,
                ..Cursor::default()
            },
        }
    }

    fn take_changes(&mut self) -> Changes {
        let mut pending = lock(&self.pending);
        if !pending.questions.is_empty() {
            let palette = self.palette();
            let questions = mem::take(&mut pending.questions);
            let responses = mem::take(&mut pending.responses);
            let mut answered = Vec::with_capacity(responses.len() + 32 * questions.len());
            let mut from = 0;
            for (at, question) in &questions {
                // The positions only grow, and the answers only went to the
                // end while the output was read.
                let at = (*at).clamp(from, responses.len());
                answered.extend_from_slice(&responses[from..at]);
                from = at;
                if let Some(answer) = self.answer(question, &palette) {
                    answered.extend_from_slice(answer.as_bytes());
                    pending.changes.responses = true;
                }
            }
            answered.extend_from_slice(&responses[from..]);
            pending.responses = answered;
        }
        let changes = mem::take(&mut pending.changes);
        if changes.title {
            self.title = pending.title.take();
        }
        changes
    }

    /// The answer to `question`, with the colours of `palette`, or `None`
    /// if the terminal does not know it.
    fn answer(&self, question: &Question, palette: &Palette) -> Option<String> {
        match question {
            Question::Color(index, answer) => {
                let color = match *index {
                    index @ 0..256 => palette.colors[index],
                    index if index == NamedColor::Foreground as usize => palette.foreground,
                    index if index == NamedColor::Background as usize => palette.background,
                    index if index == NamedColor::Cursor as usize => palette.cursor,
                    _ => return None,
                };
                Some(answer(ansi::Rgb {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                }))
            }
            // As alacritty_terminal writes it, but without its multiplication
            // of 16-bit numbers, which can overflow.
            Question::TextAreaSize => (!self.cell.is_empty()).then(|| {
                let height = u64::from(self.size.rows) * u64::from(self.cell.height);
                let width = u64::from(self.size.columns) * u64::from(self.cell.width);
                format!("\x1b[4;{height};{width}t")
            }),
        }
    }
}

fn rgb(color: ansi::Rgb) -> Rgb {
    Rgb::new(color.r, color.g, color.b)
}

fn lock(pending: &Mutex<Pending>) -> MutexGuard<'_, Pending> {
    // A panic while it was locked cannot leave the queue half changed.
    pending.lock().unwrap_or_else(PoisonError::into_inner)
}

fn at_least_min_size(size: GridSize) -> GridSize {
    GridSize::new(
        size.columns.max(MIN_SIZE.columns),
        size.rows.max(MIN_SIZE.rows),
    )
}

/// Copies the shown `row`, and adds to the changes of `screen` the cells that
/// are different. The cells of `selection` have [`CellFlags::SELECTED`].
fn copy_row(
    grid: &Grid<TermCell>,
    screen: &mut Screen,
    row: u16,
    selection: Option<&SelectionRange>,
) {
    let point = term::viewport_to_point(
        grid.display_offset(),
        Point::new(usize::from(row), Column(0)),
    );
    let sources = &grid[point.line][..];
    let columns = screen.size().columns;
    let selected = selection.map_or(Columns::default(), |range| {
        selected_cells(range, point.line, sources, columns)
    });
    let mut changed = Columns::default();
    // The character of the cell before, if it is the left half of a wide one.
    let mut wide = None;
    let cells = screen.cells_mut(row, Columns::new(0, columns));
    for ((cell, source), column) in cells.iter_mut().zip(sources).zip(0..columns) {
        let char = if source.flags.contains(Flags::WIDE_CHAR) {
            CellChar::WideLeft(source.c)
        } else if source.flags.contains(Flags::WIDE_CHAR_SPACER) {
            // Deleting or inserting characters can separate the halves.
            wide.map_or(CellChar::Blank, CellChar::WideRight)
        } else if source.flags.contains(Flags::LEADING_WIDE_CHAR_SPACER)
            // alacritty_terminal keeps a tab in the cell where it started.
            || matches!(source.c, ' ' | '\t')
        {
            CellChar::Blank
        } else {
            CellChar::Narrow(source.c)
        };
        wide = source.flags.contains(Flags::WIDE_CHAR).then_some(source.c);
        let mut flags = flags(source.flags);
        if selected.contains(column) {
            flags = flags | CellFlags::SELECTED;
        }
        let new = Cell {
            char,
            fg: color(source.fg),
            bg: color(source.bg),
            flags,
        };
        if *cell != new {
            *cell = new;
            if changed.is_empty() {
                changed.start = column;
            }
            changed.end = column + 1;
        }
    }
    screen.mark_changed(row, changed);
}

/// The columns of the row of the grid at `line`, with the cells `sources`,
/// that `range` selects: a wide character is selected whole if one of its
/// halves is.
fn selected_cells(
    range: &SelectionRange,
    line: Line,
    sources: &[TermCell],
    columns: u16,
) -> Columns {
    let mut selected = selection::selected_columns(range, line, columns);
    if selected.is_empty() {
        return selected;
    }
    let flags_at = |column: u16| {
        sources
            .get(usize::from(column))
            .map_or(Flags::empty(), |cell| cell.flags)
    };
    if selected.start > 0 && flags_at(selected.start).contains(Flags::WIDE_CHAR_SPACER) {
        selected.start -= 1;
    }
    if selected.end < columns && flags_at(selected.end - 1).contains(Flags::WIDE_CHAR) {
        selected.end += 1;
    }
    selected
}

fn color(color: ansi::Color) -> Color {
    match color {
        // The 16 named colours are the first 16 of the palette. The rest are
        // the default colours, `Foreground` and `Background`, and others that
        // only the renderer of Alacritty uses (the cursor, the dim ones…),
        // which no sequence gives to a cell.
        ansi::Color::Named(named) => match u8::try_from(named as usize) {
            Ok(index @ 0..16) => Color::Indexed(index),
            _ => Color::Default,
        },
        ansi::Color::Indexed(index) => Color::Indexed(index),
        ansi::Color::Spec(spec) => Color::Rgb(rgb(spec)),
    }
}

fn flags(flags: Flags) -> CellFlags {
    const MAP: [(Flags, CellFlags); 11] = [
        (Flags::BOLD, CellFlags::BOLD),
        (Flags::DIM, CellFlags::DIM),
        (Flags::ITALIC, CellFlags::ITALIC),
        (Flags::UNDERLINE, CellFlags::UNDERLINE),
        (Flags::DOUBLE_UNDERLINE, CellFlags::DOUBLE_UNDERLINE),
        (Flags::UNDERCURL, CellFlags::UNDERCURL),
        (Flags::DOTTED_UNDERLINE, CellFlags::DOTTED_UNDERLINE),
        (Flags::DASHED_UNDERLINE, CellFlags::DASHED_UNDERLINE),
        (Flags::STRIKEOUT, CellFlags::STRIKEOUT),
        (Flags::INVERSE, CellFlags::INVERSE),
        (Flags::HIDDEN, CellFlags::HIDDEN),
    ];
    MAP.iter()
        .filter(|(from, _)| flags.contains(*from))
        .fold(CellFlags::empty(), |all, &(_, to)| all | to)
}

#[cfg(test)]
mod tests;
