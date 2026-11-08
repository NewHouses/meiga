//! A demo that shows the same text at three scales, and draws rectangles
//! with the mouse on top of it, to see which areas of the window each frame
//! presents. It goes away when meiga shows a terminal (subphase 1.6).

use meiga_core::color::Rgb;
use meiga_core::geometry::{Point, Rect, Size};
use meiga_core::text::{CellChar, cells};
use meiga_render::{Canvas, Error, Font, FontSystem};
use meiga_win::event::{Event, Key, MouseButton};

/// The colour of the empty window.
pub const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);
/// The colour of the text.
pub const TEXT: Rgb = Rgb::new(0xd4, 0xd7, 0xdd);
/// The colour of the text that explains the rest.
const DIM: Rgb = Rgb::new(0x80, 0x87, 0x94);

/// The font: the first of these families that is installed, at 12 points,
/// as in Windows Terminal.
pub const FONT_FAMILIES: [&str; 2] = ["Cascadia Mono", "Consolas"];
pub const FONT_POINTS: f32 = 12.0;

/// The scales of the text, in percent, and their dots per inch.
const SCALES: [(u32, u32); 3] = [(100, 96), (125, 120), (150, 144)];

/// The space around and between the blocks of text, in pixels.
const MARGIN: i32 = 12;

/// The colours of the rectangles, one after the other. None is magenta, the
/// colour of the marks.
const PALETTE: [Rgb; 8] = [
    Rgb::new(0x3b, 0x82, 0xf6),
    Rgb::new(0x10, 0xb9, 0x81),
    Rgb::new(0xf5, 0x9e, 0x0b),
    Rgb::new(0xef, 0x44, 0x44),
    Rgb::new(0x8b, 0x5c, 0xf6),
    Rgb::new(0x06, 0xb6, 0xd4),
    Rgb::new(0xf9, 0x73, 0x16),
    Rgb::new(0x84, 0xcc, 0x16),
];

/// The colours of a span of text.
#[derive(Clone, Copy)]
enum Style {
    Plain,
    Dim,
    Color(Rgb),
    /// The colours of the window swapped: dark text on a light background.
    Inverse,
}

/// How to use the demo: the first line of the window.
const HINT: &str = "Drag with the left button to draw a rectangle · right button: clear · \
                    Esc: cancel · P: marks";

/// The text of each scale: lines of spans, each with its colours.
const SAMPLE: &[&[(&str, Style)]] = &[
    &[
        ("ASCII    ", Style::Dim),
        (
            "The quick brown fox jumps over the lazy dog. 0123456789",
            Style::Plain,
        ),
    ],
    &[
        ("Latin    ", Style::Dim),
        (
            "á é í ó ú à ü ñ ç Á É Ñ Ç · «Ola, mundo!» ¿Que tal? € £ ¥ ° ± × ÷",
            Style::Plain,
        ),
    ],
    &[
        ("Symbols  ", Style::Dim),
        ("✓ ✗ ★ ☆ ♥ ♦ → ← ↑ ↓ ⇒ ∞ ≠ ≤ ≥ λ π Σ Ω √ ∑", Style::Plain),
    ],
    &[
        ("Boxes    ", Style::Dim),
        ("┌──┬──┐ ╔══╗ ░░▒▒▓▓██ ▀▄▌▐", Style::Plain),
    ],
    &[("         └──┴──┘ ╚══╝ ░░▒▒▓▓██ ▀▄▌▐", Style::Plain)],
    &[
        ("CJK      ", Style::Dim),
        ("中文 日本語 한국어 😀", Style::Plain),
    ],
    // Each pair of brackets is under a character that takes two cells.
    &[("         [][] [][][] [][][] []", Style::Dim)],
    &[
        ("Colours  ", Style::Dim),
        ("blue", Style::Color(PALETTE[0])),
        (" green", Style::Color(PALETTE[1])),
        (" amber", Style::Color(PALETTE[2])),
        (" red", Style::Color(PALETTE[3])),
        (" violet", Style::Color(PALETTE[4])),
        (" cyan", Style::Color(PALETTE[5])),
        (" orange", Style::Color(PALETTE[6])),
        (" lime", Style::Color(PALETTE[7])),
        ("  ", Style::Plain),
        (" dark text on a light background ", Style::Inverse),
    ],
];

/// What the window must do with the mouse after an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mouse {
    /// Receive all of it, also outside the window, while dragging.
    Capture,
    Release,
}

/// The text, and the rectangles on top of it.
pub struct Demo {
    blocks: Vec<Block>,
    rectangles: Rectangles,
}

impl Demo {
    /// The demo, with a font for each scale.
    pub fn new(system: &FontSystem) -> Result<Self, Error> {
        let mut blocks = Vec::new();
        let mut top = MARGIN;
        for (percent, dpi) in SCALES {
            let font = Font::new(system, &FONT_FAMILIES, FONT_POINTS, dpi)?;
            let cell = font.cell();
            let header = format!(
                "{percent} % · {dpi} DPI · {} {FONT_POINTS} pt · cell {} × {} pixels",
                font.family(),
                cell.width,
                cell.height
            );
            let mut lines = Vec::new();
            if blocks.is_empty() {
                lines.push(line(&[(HINT, Style::Dim)]));
                lines.push(Vec::new());
            }
            lines.push(line(&[(&header, Style::Dim)]));
            lines.extend(SAMPLE.iter().map(|spans| line(spans)));
            let block = Block {
                font,
                origin: Point::new(MARGIN, top),
                lines,
            };
            top = block.bounds().bottom + MARGIN;
            blocks.push(block);
        }
        Ok(Self {
            blocks,
            rectangles: Rectangles::new(),
        })
    }

    /// Reacts to `event` in a window of `size`, and gives to `damage` the
    /// areas that change.
    pub fn event(&mut self, event: Event, size: Size, damage: impl FnMut(Rect)) -> Option<Mouse> {
        self.rectangles.event(event, size, damage)
    }

    pub fn draw(&mut self, canvas: &mut Canvas<'_>) {
        canvas.fill(canvas.clip(), BACKGROUND);
        for block in &mut self.blocks {
            block.draw(canvas);
        }
        self.rectangles.draw(canvas);
    }
}

/// The text at one scale.
struct Block {
    font: Font,
    /// The top-left corner of its first line.
    origin: Point,
    lines: Vec<Vec<Run>>,
}

impl Block {
    /// The area of the window that its lines take, as wide as the longest.
    fn bounds(&self) -> Rect {
        let cell = self.font.cell();
        let columns = self
            .lines
            .iter()
            .filter_map(|runs| runs.last())
            .map(|run| run.column + run.cells.len() as i32)
            .max()
            .unwrap_or(0);
        Rect::at(
            self.origin,
            Size::new(
                columns as u32 * cell.width,
                self.lines.len() as u32 * cell.height,
            ),
        )
    }

    /// Draws the lines that are inside the clip rectangle of `canvas`: the
    /// font skips the cells that are not.
    fn draw(&mut self, canvas: &mut Canvas<'_>) {
        let cell = self.font.cell();
        if !self.bounds().intersects(canvas.clip()) {
            return;
        }
        for (row, runs) in self.lines.iter().enumerate() {
            let y = self.origin.y + row as i32 * cell.height as i32;
            for run in runs {
                let x = self.origin.x + run.column * cell.width as i32;
                self.font.draw(
                    canvas,
                    Point::new(x, y),
                    &run.cells,
                    run.text,
                    run.background,
                );
            }
        }
    }
}

/// Cells of a line with the same colours.
struct Run {
    /// The cell where it starts.
    column: i32,
    cells: Vec<CellChar>,
    text: Rgb,
    background: Rgb,
}

/// The runs of a line of `spans`, one after the other.
fn line(spans: &[(&str, Style)]) -> Vec<Run> {
    let mut column = 0;
    spans
        .iter()
        .map(|&(text, style)| {
            let (color, background) = match style {
                Style::Plain => (TEXT, BACKGROUND),
                Style::Dim => (DIM, BACKGROUND),
                Style::Color(color) => (color, BACKGROUND),
                Style::Inverse => (BACKGROUND, TEXT),
            };
            let cells: Vec<CellChar> = cells(text).collect();
            let run = Run {
                column,
                text: color,
                background,
                cells,
            };
            column += run.cells.len() as i32;
            run
        })
        .collect()
}

/// The rectangles: dragging with the left button draws one, the right
/// button removes them all, and Esc cancels the one being drawn.
struct Rectangles {
    rects: Vec<(Rect, Rgb)>,
    drag: Option<Drag>,
    next_color: usize,
}

/// The rectangle being drawn.
struct Drag {
    from: Point,
    rect: Rect,
    color: Rgb,
}

impl Rectangles {
    fn new() -> Self {
        Self {
            rects: Vec::new(),
            drag: None,
            next_color: 0,
        }
    }

    /// Reacts to `event` in a window of `size`, and gives to `damage` the
    /// areas that change.
    fn event(&mut self, event: Event, size: Size, mut damage: impl FnMut(Rect)) -> Option<Mouse> {
        match event {
            Event::MouseDown {
                button: MouseButton::Left,
                at,
            } => {
                if let Some(drag) = self.drag.take() {
                    damage(drag.rect);
                }
                let rect = Rect::spanning(at, at);
                self.drag = Some(Drag {
                    from: at,
                    rect,
                    color: PALETTE[self.next_color % PALETTE.len()],
                });
                self.next_color += 1;
                damage(rect);
                Some(Mouse::Capture)
            }
            // Without the left button, the move is not part of a drag.
            Event::MouseMoved {
                at,
                left_down: true,
            } => {
                let drag = self.drag.as_mut()?;
                move_to(drag, at, &mut damage);
                None
            }
            Event::MouseUp {
                button: MouseButton::Left,
                at,
            } => {
                let mut drag = self.drag.take()?;
                move_to(&mut drag, at, &mut damage);
                self.rects.push((drag.rect, drag.color));
                Some(Mouse::Release)
            }
            // Another window took the mouse: the drag ends without a
            // rectangle.
            Event::CaptureLost => {
                damage(self.drag.take()?.rect);
                None
            }
            Event::KeyDown {
                key: Key::ESCAPE, ..
            } => {
                damage(self.drag.take()?.rect);
                Some(Mouse::Release)
            }
            // Always the whole window, even if it is empty: it is how to see
            // a full frame.
            Event::MouseDown {
                button: MouseButton::Right,
                ..
            } => {
                self.rects.clear();
                let dragging = self.drag.take().is_some();
                damage(Rect::from_size(size));
                dragging.then_some(Mouse::Release)
            }
            _ => None,
        }
    }

    /// Draws the rectangles, on top of what is already drawn.
    fn draw(&self, canvas: &mut Canvas<'_>) {
        for &(rect, color) in &self.rects {
            canvas.fill(rect, color);
        }
        if let Some(drag) = &self.drag {
            canvas.fill(drag.rect, drag.color);
        }
    }
}

/// The rectangle of `drag` goes to `at`: both the old one and the new one
/// change.
fn move_to(drag: &mut Drag, at: Point, damage: &mut impl FnMut(Rect)) {
    let rect = Rect::spanning(drag.from, at);
    if rect != drag.rect {
        damage(drag.rect);
        damage(rect);
        drag.rect = rect;
    }
}

#[cfg(test)]
mod tests {
    use meiga_render::pixel;

    use super::*;

    const SIZE: Size = Size::new(100, 80);

    /// The areas that change with `event`, and what to do with the mouse.
    fn event(rectangles: &mut Rectangles, event: Event) -> (Vec<Rect>, Option<Mouse>) {
        let mut areas = Vec::new();
        let mouse = rectangles.event(event, SIZE, |area| areas.push(area));
        (areas, mouse)
    }

    fn left(down: bool, x: i32, y: i32) -> Event {
        let at = Point::new(x, y);
        if down {
            Event::MouseDown {
                button: MouseButton::Left,
                at,
            }
        } else {
            Event::MouseUp {
                button: MouseButton::Left,
                at,
            }
        }
    }

    fn drag_to(x: i32, y: i32) -> Event {
        Event::MouseMoved {
            at: Point::new(x, y),
            left_down: true,
        }
    }

    /// The colour of the pixel at (`x`, `y`) after drawing the rectangles on
    /// the background.
    fn color_at(rectangles: &Rectangles, x: u32, y: u32) -> u32 {
        let mut pixels = vec![0; SIZE.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, SIZE, SIZE.width as usize).unwrap();
        canvas.fill(canvas.clip(), BACKGROUND);
        rectangles.draw(&mut canvas);
        canvas.get(x, y).unwrap()
    }

    #[test]
    fn dragging_draws_a_rectangle_and_changes_only_its_area() {
        let mut rectangles = Rectangles::new();
        assert_eq!(
            event(&mut rectangles, left(true, 10, 10)),
            (vec![Rect::new(10, 10, 11, 11)], Some(Mouse::Capture))
        );
        assert_eq!(
            event(&mut rectangles, drag_to(30, 20)),
            (
                vec![Rect::new(10, 10, 11, 11), Rect::new(10, 10, 31, 21)],
                None
            )
        );
        // The same rectangle: nothing changes.
        assert_eq!(event(&mut rectangles, drag_to(30, 20)), (vec![], None));
        assert_eq!(
            event(&mut rectangles, left(false, 30, 20)),
            (vec![], Some(Mouse::Release))
        );
        assert_eq!(color_at(&rectangles, 20, 15), pixel(PALETTE[0]));
        assert_eq!(color_at(&rectangles, 31, 15), pixel(BACKGROUND));
    }

    #[test]
    fn moves_without_a_drag_or_the_left_button_change_nothing() {
        let mut rectangles = Rectangles::new();
        assert_eq!(event(&mut rectangles, drag_to(5, 5)), (vec![], None));
        event(&mut rectangles, left(true, 10, 10));
        let hover = Event::MouseMoved {
            at: Point::new(50, 50),
            left_down: false,
        };
        assert_eq!(event(&mut rectangles, hover), (vec![], None));
    }

    #[test]
    fn escape_and_a_lost_capture_cancel_the_drag() {
        let mut rectangles = Rectangles::new();
        event(&mut rectangles, left(true, 10, 10));
        event(&mut rectangles, drag_to(20, 20));
        let escape = Event::KeyDown {
            key: Key::ESCAPE,
            repeat: false,
        };
        assert_eq!(
            event(&mut rectangles, escape),
            (vec![Rect::new(10, 10, 21, 21)], Some(Mouse::Release))
        );
        assert_eq!(event(&mut rectangles, left(false, 20, 20)), (vec![], None));

        event(&mut rectangles, left(true, 40, 40));
        assert_eq!(
            event(&mut rectangles, Event::CaptureLost),
            (vec![Rect::new(40, 40, 41, 41)], None)
        );
        assert_eq!(color_at(&rectangles, 40, 40), pixel(BACKGROUND));
    }

    #[test]
    fn the_right_button_clears_the_whole_window() {
        let mut rectangles = Rectangles::new();
        event(&mut rectangles, left(true, 10, 10));
        event(&mut rectangles, left(false, 20, 20));
        let right = Event::MouseDown {
            button: MouseButton::Right,
            at: Point::new(1, 1),
        };
        assert_eq!(
            event(&mut rectangles, right),
            (vec![Rect::from_size(SIZE)], None)
        );
        assert_eq!(color_at(&rectangles, 15, 15), pixel(BACKGROUND));
    }

    #[test]
    fn each_rectangle_takes_the_next_colour() {
        let mut rectangles = Rectangles::new();
        for x in [0, 10] {
            event(&mut rectangles, left(true, x, 0));
            event(&mut rectangles, left(false, x + 5, 5));
        }
        assert_eq!(color_at(&rectangles, 2, 2), pixel(PALETTE[0]));
        assert_eq!(color_at(&rectangles, 12, 2), pixel(PALETTE[1]));
        assert!(!PALETTE.contains(&Rgb::new(0xff, 0x00, 0xff)));
    }

    #[test]
    fn the_three_scales_are_stacked_from_the_smallest() {
        let demo = Demo::new(&FontSystem::new().unwrap()).unwrap();
        let blocks: Vec<Rect> = demo.blocks.iter().map(Block::bounds).collect();
        for pair in demo.blocks.windows(2) {
            assert!(pair[1].font.cell().height > pair[0].font.cell().height);
        }
        for pair in blocks.windows(2) {
            assert_eq!(pair[1].top, pair[0].bottom + MARGIN);
        }
        // The hint is the first line, before the header of 100 %.
        let first = &demo.blocks[0].lines;
        assert_eq!(first[0][0].cells.len(), meiga_core::text::width(HINT));
        assert!(first[1].is_empty());
    }

    #[test]
    fn runs_start_where_the_previous_one_ends() {
        let runs = line(&[("中a", Style::Plain), ("bc", Style::Inverse)]);
        assert_eq!(runs[0].cells.len(), 3);
        assert_eq!(runs[1].column, 3);
        assert_eq!((runs[1].text, runs[1].background), (BACKGROUND, TEXT));
    }
}
