//! The terminal in the window: its copy of the screen, brought up to date
//! from the terminal of the session before each frame, and the drawing of
//! the cells, their lines and the cursor.

use std::mem;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use meiga_core::color::{Palette, Rgb};
use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::input::{KeyEncoder, KeyEvent, focus_sequence};
use meiga_core::term::{Cell, CellFlags, Columns, Cursor, CursorShape, Screen};
use meiga_core::text::CellChar;
use meiga_render::{Canvas, Decorations, Font, FontStyle};
use meiga_win::conpty::Input;
use meiga_win::window::{self, CaretSettings, Waker};

use crate::frames::Scene;
use crate::session::{Shared, lock};

/// The font: the first of these families that is installed, at 12 points,
/// as in Windows Terminal.
pub const FONT_FAMILIES: [&str; 2] = ["Cascadia Mono", "Consolas"];
pub const FONT_POINTS: f32 = 12.0;

/// How far dim text (SGR 2) goes from its colour towards the one of its
/// background: a third of the way, out of 255.
const DIM: u8 = 85;

/// The dots per inch at a scale of 100 %.
const BASE_DPI: u32 = 96;

/// The terminal of the session, as the window shows it.
pub struct TerminalView {
    shared: Arc<Mutex<Shared>>,
    /// Sends to the program the keys, and the answers that the window finds
    /// when it ends a synchronized update. `None` if the program did not
    /// start.
    input: Option<Input>,
    waker: Waker,
    font: Font,
    cell: Size,
    screen: Screen,
    caret: Caret,
    /// The characters of a run of cells, kept to avoid an allocation for
    /// each one.
    run: Vec<CellChar>,
    /// The answers for the program, kept for the same reason.
    answers: Vec<u8>,
    /// Turns the keys into what the program reads.
    encoder: KeyEncoder,
    /// What the program reads for a key, kept for the same reason.
    keys: Vec<u8>,
}

impl TerminalView {
    /// The view of the terminal of `shared`, drawn with `font` at `dpi`.
    pub fn new(
        shared: Arc<Mutex<Shared>>,
        input: Option<Input>,
        waker: Waker,
        font: Font,
        dpi: u32,
    ) -> Self {
        Self {
            shared,
            input,
            waker,
            cell: font.cell(),
            font,
            screen: Screen::new(),
            caret: Caret::new(dpi, Instant::now()),
            run: Vec::new(),
            answers: Vec::new(),
            encoder: KeyEncoder::new(),
            keys: Vec::new(),
        }
    }

    pub const fn shared(&self) -> &Arc<Mutex<Shared>> {
        &self.shared
    }

    /// Sends a key to the program, as the modes that it turned on say.
    pub fn key(&mut self, event: &KeyEvent) {
        let Some(input) = &self.input else {
            return;
        };
        let modes = lock(&self.shared).terminal.modes();
        self.keys.clear();
        self.encoder.encode(event, &modes, &mut self.keys);
        input.write(&self.keys);
    }

    /// The window gained or lost the focus, at `now`. With it, the cursor
    /// starts blinking again; without it, it is an outline that does not
    /// blink. The next update gives its cells as changed. The program
    /// learns it too, if it asked.
    pub fn focus(&mut self, focused: bool, now: Instant) {
        if focused == self.caret.focused {
            return;
        }
        if focused {
            // The settings can change while meiga runs: for example, when
            // the user connects to the session through Remote Desktop.
            self.caret.read_settings();
            self.caret.since = now;
        }
        self.caret.focused = focused;
        self.screen.mark_cursor_changed();
        if let Some(input) = &self.input {
            let modes = lock(&self.shared).terminal.modes();
            input.write(focus_sequence(focused, &modes));
        }
    }

    /// Sends the program the answers that ending a synchronized update gave,
    /// and wakes the window up for the title, if it changed.
    fn send(&mut self, title: bool) {
        if !self.answers.is_empty() {
            if let Some(input) = &self.input {
                input.write(&self.answers);
            }
            self.answers.clear();
        }
        if title {
            // The window takes the title when it wakes up.
            self.waker.wake();
        }
    }

    /// The pixels of `columns` of `row`.
    fn area(&self, row: u16, columns: Columns) -> Rect {
        let (width, height) = cell_size(self.cell);
        Rect::new(
            i32::from(columns.start) * width,
            i32::from(row) * height,
            i32::from(columns.end) * width,
            (i32::from(row) + 1) * height,
        )
    }
}

impl Scene for TerminalView {
    fn update(&mut self, now: Instant, damage: &mut dyn FnMut(Rect)) -> Option<Instant> {
        let (sync, title) = {
            let mut shared = lock(&self.shared);
            // If the program did not end its synchronized update in time, it
            // is shown as it is.
            let late = shared.end_late_sync(now, &mut self.answers);
            shared.terminal.copy_damage(&mut self.screen);
            shared.screen_changed = false;
            late
        };
        self.send(title);

        // What the program shows is activity: the cursor blinks again from
        // the start, shown.
        if self.screen.all_changed() || self.screen.changes().next().is_some() {
            self.caret.since = now;
        }
        let (shown, blink) = self.caret.phase(self.screen.cursor(), now);
        if mem::replace(&mut self.caret.shown, shown) != shown {
            self.screen.mark_cursor_changed();
        }

        if self.screen.all_changed() {
            // Also the margins, which have the colour of the background.
            damage(Rect::new(0, 0, i32::MAX, i32::MAX));
        } else {
            for (row, columns) in self.screen.changes() {
                damage(self.area(row, columns));
            }
        }
        self.screen.clear_changes();
        match (sync, blink) {
            (Some(sync), Some(blink)) => Some(sync.min(blink)),
            (sync, blink) => sync.or(blink),
        }
    }

    fn update_hidden(&mut self, now: Instant) -> Option<Instant> {
        // Only the synchronized update: the screen is copied when the
        // window is restored, and the cursor does not blink in a window
        // that is not shown.
        let (sync, title) = lock(&self.shared).end_late_sync(now, &mut self.answers);
        self.send(title);
        sync
    }

    fn draw(&mut self, canvas: &mut Canvas<'_>) {
        let Self {
            font,
            cell,
            screen,
            caret,
            run,
            ..
        } = self;
        let palette = screen.palette();
        let size = screen.size();
        let (width, height) = cell_size(*cell);
        let grid = Rect::new(
            0,
            0,
            i32::from(size.columns) * width,
            i32::from(size.rows) * height,
        );
        // The margins: the window can be larger than the grid.
        let clip = canvas.clip();
        canvas.fill(
            Rect::new(grid.right, clip.top, clip.right, clip.bottom),
            palette.background,
        );
        canvas.fill(
            Rect::new(clip.left, grid.bottom, grid.right, clip.bottom),
            palette.background,
        );

        let visible = clip.intersection(grid);
        if visible.is_empty() {
            return;
        }
        // `visible` is inside the grid, so the divisions are not negative.
        let columns = (visible.left / width) as usize..((visible.right - 1) / width + 1) as usize;
        for row in visible.top / height..=(visible.bottom - 1) / height {
            let cells = screen.row(row as u16);
            draw_row(
                canvas,
                font,
                run,
                cells,
                columns.clone(),
                row * height,
                palette,
            );
        }
        draw_cursor(canvas, font, screen, caret, *cell);
    }
}

/// The width and the height of `cell`, as coordinates. A cell is a few
/// dozen pixels.
fn cell_size(cell: Size) -> (i32, i32) {
    (cell.width as i32, cell.height as i32)
}

/// How a cell is drawn.
struct Style {
    font: FontStyle,
    text: Rgb,
    background: Rgb,
    /// The text is not drawn (SGR 8), only the background.
    hidden: bool,
}

impl Style {
    fn of(cell: &Cell, palette: &Palette) -> Self {
        let flags = cell.flags;
        let mut text = cell.fg.rgb(palette, palette.foreground);
        let mut background = cell.bg.rgb(palette, palette.background);
        if flags.contains(CellFlags::INVERSE) {
            mem::swap(&mut text, &mut background);
        }
        if flags.contains(CellFlags::DIM) {
            text = text.mix(background, DIM);
        }
        Self {
            font: FontStyle {
                bold: flags.contains(CellFlags::BOLD),
                italic: flags.contains(CellFlags::ITALIC),
            },
            text,
            background,
            hidden: flags.contains(CellFlags::HIDDEN),
        }
    }
}

/// Draws `columns` of the row of `cells` whose top is at `top`: in runs of
/// cells with the same colours and styles, each with one call to the font.
fn draw_row(
    canvas: &mut Canvas<'_>,
    font: &mut Font,
    run: &mut Vec<CellChar>,
    cells: &[Cell],
    columns: Range<usize>,
    top: i32,
    palette: &Palette,
) {
    let (width, height) = cell_size(font.cell());
    let end = columns.end.min(cells.len());
    let mut start = columns.start;
    while start < end {
        let first = &cells[start];
        let same =
            |cell: &Cell| cell.fg == first.fg && cell.bg == first.bg && cell.flags == first.flags;
        let stop = cells[start + 1..end]
            .iter()
            .position(|cell| !same(cell))
            .map_or(end, |count| start + 1 + count);
        let style = Style::of(first, palette);
        run.clear();
        if style.hidden {
            run.resize(stop - start, CellChar::Blank);
        } else {
            run.extend(cells[start..stop].iter().map(|cell| cell.char));
        }
        // The columns of a row fit in a u16, and so their pixels in an i32.
        let left = start as i32 * width;
        font.draw(
            canvas,
            Point::new(left, top),
            run,
            style.font,
            style.text,
            style.background,
        );
        if !style.hidden {
            let area = Rect::new(left, top, stop as i32 * width, top + height);
            draw_lines(
                canvas,
                first.flags,
                area,
                font.decorations(),
                style.text,
                font.cell(),
            );
        }
        start = stop;
    }
}

/// Draws the lines of the cells of `area` with `flags`: the underlines and
/// the strikeout, in `color`.
fn draw_lines(
    canvas: &mut Canvas<'_>,
    flags: CellFlags,
    area: Rect,
    decorations: Decorations,
    color: Rgb,
    cell: Size,
) {
    let Decorations {
        underline,
        strikeout,
    } = decorations;
    // A cell is a few dozen pixels.
    let thickness = underline.thickness as i32;
    let line = |canvas: &mut Canvas<'_>, top: u32, height: u32| {
        let top = area.top + top as i32;
        canvas.fill(
            Rect::new(area.left, top, area.right, top + height as i32),
            color,
        );
    };
    if flags.contains(CellFlags::UNDERLINE) {
        line(canvas, underline.top, underline.thickness);
    }
    if flags.contains(CellFlags::DOUBLE_UNDERLINE) {
        // Two lines with a gap as thick as them, higher if they do not fit.
        let top = underline
            .top
            .min(cell.height.saturating_sub(3 * underline.thickness));
        line(canvas, top, underline.thickness);
        line(canvas, top + 2 * underline.thickness, underline.thickness);
    }
    let top = area.top + underline.top as i32;
    if flags.contains(CellFlags::DOTTED_UNDERLINE) {
        // Dots as long as the line is thick, with gaps as long, at the same
        // places of the row however much of it a frame draws: from the left
        // of the window, not of the area.
        let period = 2 * thickness;
        let first = area.left - area.left.rem_euclid(period);
        for left in (first..area.right).step_by(period as usize) {
            let dot = Rect::new(
                left.max(area.left),
                top,
                (left + thickness).min(area.right),
                top + thickness,
            );
            canvas.fill(dot, color);
        }
    }
    if flags.contains(CellFlags::DASHED_UNDERLINE) {
        // A dash in the middle of each cell, two thirds of its width.
        let width = cell.width as i32;
        let dash = (width * 2 / 3).max(1);
        for left in (area.left..area.right).step_by(width.max(1) as usize) {
            let left = left + (width - dash) / 2;
            canvas.fill(Rect::new(left, top, left + dash, top + thickness), color);
        }
    }
    if flags.contains(CellFlags::UNDERCURL) {
        // A zigzag with a period of a cell, as high as the space below the
        // underline allows.
        let period = (cell.width as i32).max(4);
        let height = (cell.height as i32 / 8).max(thickness);
        let top = top.min(area.top + cell.height as i32 - height - thickness);
        let half = period / 2;
        for x in area.left..area.right {
            let phase = x.rem_euclid(period);
            let rise = if phase < half { phase } else { period - phase };
            let y = top + rise * height / half;
            canvas.fill(Rect::new(x, y, x + 1, y + thickness), color);
        }
    }
    if flags.contains(CellFlags::STRIKEOUT) {
        line(canvas, strikeout.top, strikeout.thickness);
    }
}

/// Draws the cursor, if it is shown, over the cells that are already drawn.
fn draw_cursor(
    canvas: &mut Canvas<'_>,
    font: &mut Font,
    screen: &Screen,
    caret: &Caret,
    cell: Size,
) {
    let cursor = screen.cursor();
    if cursor.shape == CursorShape::Hidden || !caret.shown {
        return;
    }
    let cells = screen.row(cursor.row);
    let column = usize::from(cursor.column);
    let Some(under) = cells.get(column) else {
        return;
    };
    // Over a wide character, it takes its two cells.
    let wide = matches!(under.char, CellChar::WideLeft(_));
    let (width, height) = cell_size(cell);
    let (x, y) = (
        i32::from(cursor.column) * width,
        i32::from(cursor.row) * height,
    );
    let area = Rect::new(x, y, x + if wide { 2 * width } else { width }, y + height);
    if !area.intersects(canvas.clip()) {
        return;
    }
    let palette = screen.palette();
    let color = palette.cursor;
    let thickness = caret.thickness as i32;
    let shape = if caret.focused {
        cursor.shape
    } else {
        CursorShape::HollowBlock
    };
    match shape {
        CursorShape::Beam => canvas.fill(Rect::new(x, y, x + thickness, y + height), color),
        CursorShape::Underline => canvas.fill(
            Rect::new(x, area.bottom - thickness, area.right, area.bottom),
            color,
        ),
        CursorShape::HollowBlock => canvas.outline(area, caret.thickness, color),
        CursorShape::Block => {
            // The cells under it, with the colour of their background on the
            // one of the cursor.
            let style = Style::of(under, palette);
            let count = if wide { 2 } else { 1 };
            let mut chars = [CellChar::Blank; 2];
            if !style.hidden {
                for (char, cell) in chars.iter_mut().zip(&cells[column..]) {
                    *char = cell.char;
                }
            }
            font.draw(
                canvas,
                Point::new(x, y),
                &chars[..count],
                style.font,
                style.background,
                color,
            );
        }
        CursorShape::Hidden => {}
    }
}

/// The width of the caret of `settings` at `dpi`, rounded, and at least one
/// pixel.
fn thickness(settings: CaretSettings, dpi: u32) -> u32 {
    let scaled = (settings.width.saturating_mul(dpi) + BASE_DPI / 2) / BASE_DPI;
    scaled.max(1)
}

/// How the cursor looks and blinks.
struct Caret {
    settings: CaretSettings,
    dpi: u32,
    /// The thickness of the bar, of the underline and of the outline, in
    /// pixels: the width of the caret of Windows, at the scale of the window.
    thickness: u32,
    focused: bool,
    /// When it started to blink: at the last activity. It stops after the
    /// timeout of the caret, shown.
    since: Instant,
    /// Whether it is drawn now: not in the hidden half of a blink.
    shown: bool,
}

impl Caret {
    fn new(dpi: u32, now: Instant) -> Self {
        let settings = window::caret_settings();
        Self {
            settings,
            dpi,
            thickness: thickness(settings, dpi),
            focused: false,
            since: now,
            shown: true,
        }
    }

    fn read_settings(&mut self) {
        self.settings = window::caret_settings();
        self.thickness = thickness(self.settings, self.dpi);
    }

    /// Whether `cursor` is shown at `now`, and when that changes next, while
    /// it blinks. It only blinks if Windows allows it, if the window has the
    /// focus and if the program did not ask for a steady cursor, and only
    /// until the timeout of the caret after the last activity.
    fn phase(&self, cursor: Cursor, now: Instant) -> (bool, Option<Instant>) {
        let blinks = self.focused && cursor.blinking && cursor.shape != CursorShape::Hidden;
        let Some(period) = self.settings.blink.filter(|_| blinks) else {
            return (true, None);
        };
        let Some(end) = self.since.checked_add(self.settings.timeout) else {
            return (true, None);
        };
        if now >= end || period.is_zero() {
            return (true, None);
        }
        let elapsed = now.saturating_duration_since(self.since);
        // The halves of a blink so far: fewer than ten with the defaults of
        // Windows, and they fit in a u32 with any timeout of Windows, in
        // milliseconds.
        let halves = u32::try_from(elapsed.as_nanos() / period.as_nanos()).unwrap_or(u32::MAX);
        let next = period
            .checked_mul(halves.saturating_add(1))
            .and_then(|offset| self.since.checked_add(offset))
            .map_or(end, |next| next.min(end));
        (halves.is_multiple_of(2), Some(next))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use meiga_core::color::{BACKGROUND, FOREGROUND};
    use meiga_core::term::Color;
    use meiga_render::{Line, pixel};

    use super::*;

    fn caret(blink: Option<Duration>) -> (Caret, Instant) {
        let now = Instant::now();
        let caret = Caret {
            settings: CaretSettings {
                blink,
                timeout: Duration::from_millis(5000),
                width: 1,
            },
            dpi: 96,
            thickness: 1,
            focused: true,
            since: now,
            shown: true,
        };
        (caret, now)
    }

    const BAR: Cursor = Cursor {
        row: 0,
        column: 0,
        shape: CursorShape::Beam,
        blinking: true,
    };

    #[test]
    fn the_cursor_blinks_until_the_timeout_and_stays_shown() {
        let (caret, start) = caret(Some(Duration::from_millis(500)));
        let at = |ms| start + Duration::from_millis(ms);
        assert_eq!(caret.phase(BAR, at(0)), (true, Some(at(500))));
        assert_eq!(caret.phase(BAR, at(499)), (true, Some(at(500))));
        assert_eq!(caret.phase(BAR, at(500)), (false, Some(at(1000))));
        assert_eq!(caret.phase(BAR, at(1200)), (true, Some(at(1500))));
        assert_eq!(caret.phase(BAR, at(1700)), (false, Some(at(2000))));
        // The last change is the end, when it is shown for good.
        assert_eq!(caret.phase(BAR, at(4800)), (false, Some(at(5000))));
        assert_eq!(caret.phase(BAR, at(5000)), (true, None));
        assert_eq!(caret.phase(BAR, at(60_000)), (true, None));
    }

    #[test]
    fn the_cursor_does_not_blink_unless_everything_allows_it() {
        let (caret, now) = caret(Some(Duration::from_millis(500)));
        let steady = Cursor {
            blinking: false,
            ..BAR
        };
        assert_eq!(caret.phase(steady, now), (true, None));
        let hidden = Cursor {
            shape: CursorShape::Hidden,
            ..BAR
        };
        assert_eq!(caret.phase(hidden, now), (true, None));
        // Windows does not allow it, for example through Remote Desktop.
        let (still, now) = self::caret(None);
        assert_eq!(still.phase(BAR, now), (true, None));
        // Without the focus.
        let (mut unfocused, now) = self::caret(Some(Duration::from_millis(500)));
        unfocused.focused = false;
        assert_eq!(unfocused.phase(BAR, now), (true, None));
    }

    #[test]
    fn a_dotted_underline_keeps_its_dots_when_a_frame_draws_part_of_it() {
        // Four cells of 9 × 19 pixels, as at a scale of 100 %.
        let cell = Size::new(9, 19);
        let size = Size::new(36, 19);
        for thickness in [1, 2] {
            let decorations = Decorations {
                underline: Line { top: 15, thickness },
                strikeout: Line { top: 9, thickness },
            };
            let draw = |area: Rect| {
                let mut pixels = vec![0; 36 * 19];
                let mut canvas = Canvas::new(&mut pixels, size, 36).unwrap();
                let flags = CellFlags::DOTTED_UNDERLINE;
                draw_lines(&mut canvas, flags, area, decorations, FOREGROUND, cell);
                pixels[15 * 36..16 * 36].to_vec()
            };
            let whole = draw(Rect::new(0, 0, 36, 19));
            let dots: Vec<bool> = whole.iter().map(|&p| p == pixel(FOREGROUND)).collect();
            let expected: Vec<bool> = (0..36).map(|x| x % (2 * thickness) < thickness).collect();
            assert_eq!(dots, expected, "thickness {thickness}");
            // A frame that draws only the second and the third cell, which
            // start at an odd pixel, draws the same dots there.
            let part = draw(Rect::new(9, 0, 27, 19));
            assert_eq!(part[9..27], whole[9..27], "thickness {thickness}");
            assert!(part[..9].iter().chain(&part[27..]).all(|&p| p == 0));
        }
    }

    #[test]
    fn styles_give_the_colours_of_the_text_and_its_background() {
        let palette = Palette::MEIGA;
        let cell = |fg, bg, flags| Cell {
            char: CellChar::Narrow('a'),
            fg,
            bg,
            flags,
        };
        let plain = Style::of(
            &cell(Color::Default, Color::Default, CellFlags::empty()),
            &palette,
        );
        assert_eq!((plain.text, plain.background), (FOREGROUND, BACKGROUND));
        assert_eq!(plain.font, FontStyle::REGULAR);

        let red = Rgb::new(0xc5, 0x0f, 0x1f);
        let inverse = Style::of(
            &cell(
                Color::Indexed(1),
                Color::Default,
                CellFlags::INVERSE | CellFlags::BOLD,
            ),
            &palette,
        );
        assert_eq!((inverse.text, inverse.background), (BACKGROUND, red));
        assert!(inverse.font.bold && !inverse.font.italic);

        let rgb = Rgb::new(10, 20, 30);
        let dim = Style::of(
            &cell(
                Color::Rgb(rgb),
                Color::Indexed(15),
                CellFlags::DIM | CellFlags::HIDDEN,
            ),
            &palette,
        );
        assert_eq!(dim.text, rgb.mix(palette.colors[15], DIM));
        assert_eq!(dim.background, palette.colors[15]);
        assert!(dim.hidden);
    }
}
