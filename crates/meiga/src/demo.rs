//! A demo that draws rectangles with the mouse, to see which areas of the
//! window each frame presents. It goes away when meiga shows a terminal
//! (subphase 1.6).

use meiga_core::color::Rgb;
use meiga_core::geometry::{Point, Rect, Size};
use meiga_render::Canvas;
use meiga_win::event::{Event, Key, MouseButton};

/// The colour of the empty window.
pub const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);

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

/// What the window must do with the mouse after an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mouse {
    /// Receive all of it, also outside the window, while dragging.
    Capture,
    Release,
}

/// The rectangles: dragging with the left button draws one, the right
/// button removes them all, and Esc cancels the one being drawn.
pub struct Demo {
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

impl Demo {
    pub fn new() -> Self {
        Self {
            rects: Vec::new(),
            drag: None,
            next_color: 0,
        }
    }

    /// Reacts to `event` in a window of `size`, and gives to `damage` the
    /// areas that change.
    pub fn event(
        &mut self,
        event: Event,
        size: Size,
        mut damage: impl FnMut(Rect),
    ) -> Option<Mouse> {
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

    pub fn draw(&self, canvas: &mut Canvas<'_>) {
        canvas.fill(canvas.clip(), BACKGROUND);
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
    fn event(demo: &mut Demo, event: Event) -> (Vec<Rect>, Option<Mouse>) {
        let mut areas = Vec::new();
        let mouse = demo.event(event, SIZE, |area| areas.push(area));
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

    /// The colour of the pixel at (`x`, `y`) after drawing the whole demo.
    fn color_at(demo: &Demo, x: u32, y: u32) -> u32 {
        let mut pixels = vec![0; SIZE.area() as usize];
        let mut canvas = Canvas::new(&mut pixels, SIZE, SIZE.width as usize).unwrap();
        demo.draw(&mut canvas);
        canvas.get(x, y).unwrap()
    }

    #[test]
    fn dragging_draws_a_rectangle_and_changes_only_its_area() {
        let mut demo = Demo::new();
        assert_eq!(
            event(&mut demo, left(true, 10, 10)),
            (vec![Rect::new(10, 10, 11, 11)], Some(Mouse::Capture))
        );
        assert_eq!(
            event(&mut demo, drag_to(30, 20)),
            (
                vec![Rect::new(10, 10, 11, 11), Rect::new(10, 10, 31, 21)],
                None
            )
        );
        // The same rectangle: nothing changes.
        assert_eq!(event(&mut demo, drag_to(30, 20)), (vec![], None));
        assert_eq!(
            event(&mut demo, left(false, 30, 20)),
            (vec![], Some(Mouse::Release))
        );
        assert_eq!(color_at(&demo, 20, 15), pixel(PALETTE[0]));
        assert_eq!(color_at(&demo, 31, 15), pixel(BACKGROUND));
    }

    #[test]
    fn moves_without_a_drag_or_the_left_button_change_nothing() {
        let mut demo = Demo::new();
        assert_eq!(event(&mut demo, drag_to(5, 5)), (vec![], None));
        event(&mut demo, left(true, 10, 10));
        let hover = Event::MouseMoved {
            at: Point::new(50, 50),
            left_down: false,
        };
        assert_eq!(event(&mut demo, hover), (vec![], None));
    }

    #[test]
    fn escape_and_a_lost_capture_cancel_the_drag() {
        let mut demo = Demo::new();
        event(&mut demo, left(true, 10, 10));
        event(&mut demo, drag_to(20, 20));
        let escape = Event::KeyDown {
            key: Key::ESCAPE,
            repeat: false,
        };
        assert_eq!(
            event(&mut demo, escape),
            (vec![Rect::new(10, 10, 21, 21)], Some(Mouse::Release))
        );
        assert_eq!(event(&mut demo, left(false, 20, 20)), (vec![], None));

        event(&mut demo, left(true, 40, 40));
        assert_eq!(
            event(&mut demo, Event::CaptureLost),
            (vec![Rect::new(40, 40, 41, 41)], None)
        );
        assert_eq!(color_at(&demo, 40, 40), pixel(BACKGROUND));
    }

    #[test]
    fn the_right_button_clears_the_whole_window() {
        let mut demo = Demo::new();
        event(&mut demo, left(true, 10, 10));
        event(&mut demo, left(false, 20, 20));
        let right = Event::MouseDown {
            button: MouseButton::Right,
            at: Point::new(1, 1),
        };
        assert_eq!(event(&mut demo, right), (vec![Rect::from_size(SIZE)], None));
        assert_eq!(color_at(&demo, 15, 15), pixel(BACKGROUND));
    }

    #[test]
    fn each_rectangle_takes_the_next_colour() {
        let mut demo = Demo::new();
        for x in [0, 10] {
            event(&mut demo, left(true, x, 0));
            event(&mut demo, left(false, x + 5, 5));
        }
        assert_eq!(color_at(&demo, 2, 2), pixel(PALETTE[0]));
        assert_eq!(color_at(&demo, 12, 2), pixel(PALETTE[1]));
        assert!(!PALETTE.contains(&Rgb::new(0xff, 0x00, 0xff)));
    }
}
