//! Points, sizes and rectangles, in pixels, and the size of the terminal
//! grid, in cells.

/// A point, in pixels from the top-left corner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A size, in pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Number of pixels.
    pub const fn area(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Whether it has no pixels.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// A rectangle of pixels.
///
/// As a Win32 `RECT`, it goes from `left` and `top`, included, to `right` and
/// `bottom`, excluded: its width is `right - left`, and two rectangles that
/// share a side do not overlap. When `right <= left` or `bottom <= top`, it
/// is empty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    /// The empty rectangle that the operations return when there are no
    /// pixels.
    pub const EMPTY: Rect = Rect::new(0, 0, 0, 0);

    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// The rectangle of `size` with its top-left corner at (0, 0).
    pub const fn from_size(size: Size) -> Self {
        Self::new(0, 0, saturate(size.width), saturate(size.height))
    }

    /// The rectangle of `size` with its top-left corner at `origin`.
    pub const fn at(origin: Point, size: Size) -> Self {
        Self::from_size(size).offset(origin)
    }

    /// The smallest rectangle that includes the pixels `a` and `b`.
    pub fn spanning(a: Point, b: Point) -> Self {
        Self::new(
            a.x.min(b.x),
            a.y.min(b.y),
            a.x.max(b.x).saturating_add(1),
            a.y.max(b.y).saturating_add(1),
        )
    }

    pub const fn width(self) -> u32 {
        span(self.left, self.right)
    }

    pub const fn height(self) -> u32 {
        span(self.top, self.bottom)
    }

    pub const fn size(self) -> Size {
        Size::new(self.width(), self.height())
    }

    /// Number of pixels.
    pub const fn area(self) -> u64 {
        self.size().area()
    }

    pub const fn is_empty(self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    /// The pixels that are in both rectangles.
    pub fn intersection(self, other: Rect) -> Rect {
        let common = Rect::new(
            self.left.max(other.left),
            self.top.max(other.top),
            self.right.min(other.right),
            self.bottom.min(other.bottom),
        );
        if common.is_empty() {
            Rect::EMPTY
        } else {
            common
        }
    }

    /// The smallest rectangle that includes both. An empty rectangle adds no
    /// pixels.
    pub fn union(self, other: Rect) -> Rect {
        match (self.is_empty(), other.is_empty()) {
            (true, true) => Rect::EMPTY,
            (true, false) => other,
            (false, true) => self,
            (false, false) => Rect::new(
                self.left.min(other.left),
                self.top.min(other.top),
                self.right.max(other.right),
                self.bottom.max(other.bottom),
            ),
        }
    }

    /// Whether they have pixels in common.
    pub fn intersects(self, other: Rect) -> bool {
        !self.intersection(other).is_empty()
    }

    /// Whether every pixel of `other` is in this rectangle. An empty
    /// rectangle is in any rectangle.
    pub const fn contains(self, other: Rect) -> bool {
        other.is_empty()
            || (self.left <= other.left
                && self.top <= other.top
                && self.right >= other.right
                && self.bottom >= other.bottom)
    }

    /// The same rectangle, moved by `by`.
    pub const fn offset(self, by: Point) -> Rect {
        Rect::new(
            self.left.saturating_add(by.x),
            self.top.saturating_add(by.y),
            self.right.saturating_add(by.x),
            self.bottom.saturating_add(by.y),
        )
    }

    /// The sides of a border `thickness` pixels wide, inside the rectangle:
    /// top, bottom, left and right. They do not overlap, and a side is empty
    /// where the rectangle is too small for it.
    pub fn border(self, thickness: u32) -> [Rect; 4] {
        if self.is_empty() || thickness == 0 {
            return [Rect::EMPTY; 4];
        }
        let thickness = saturate(thickness);
        let top_bottom = self.top.saturating_add(thickness).min(self.bottom);
        let bottom_top = self.bottom.saturating_sub(thickness).max(top_bottom);
        let left_right = self.left.saturating_add(thickness).min(self.right);
        let right_left = self.right.saturating_sub(thickness).max(left_right);
        [
            Rect::new(self.left, self.top, self.right, top_bottom),
            Rect::new(self.left, bottom_top, self.right, self.bottom),
            Rect::new(self.left, top_bottom, left_right, bottom_top),
            Rect::new(right_left, top_bottom, self.right, bottom_top),
        ]
        .map(|side| if side.is_empty() { Rect::EMPTY } else { side })
    }
}

/// A size of the terminal grid, in cells: how many columns and rows of
/// characters fit in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GridSize {
    pub columns: u16,
    pub rows: u16,
}

impl GridSize {
    pub const fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }
}

/// A cell of the terminal grid: its column and its row, from the top-left
/// cell, which is (0, 0).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GridPoint {
    pub column: u16,
    pub row: u16,
}

impl GridPoint {
    pub const fn new(column: u16, row: u16) -> Self {
        Self { column, row }
    }
}

/// Number of pixels from `from` to `to`, or 0 if `to` is not after `from`.
const fn span(from: i32, to: i32) -> u32 {
    if to > from {
        // The difference of two i32 always fits in a u32.
        (to as i64 - from as i64) as u32
    } else {
        0
    }
}

/// Converts a size to a coordinate, saturating at `i32::MAX`.
const fn saturate(value: u32) -> i32 {
    if value > i32::MAX as u32 {
        i32::MAX
    } else {
        value as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_and_bottom_are_not_included() {
        let rect = Rect::new(10, 20, 30, 25);
        assert_eq!(rect.size(), Size::new(20, 5));
        assert_eq!(rect.area(), 100);
        assert!(!Rect::new(0, 0, 10, 10).intersects(Rect::new(10, 0, 20, 10)));
    }

    #[test]
    fn inverted_rectangles_are_empty() {
        let rect = Rect::new(30, 20, 10, 25);
        assert!(rect.is_empty());
        assert_eq!(rect.width(), 0);
        assert_eq!(rect.area(), 0);
    }

    #[test]
    fn from_size_starts_at_the_origin() {
        assert_eq!(
            Rect::from_size(Size::new(640, 480)),
            Rect::new(0, 0, 640, 480)
        );
        assert_eq!(Rect::from_size(Size::new(u32::MAX, 1)).right, i32::MAX);
    }

    #[test]
    fn spanning_includes_both_pixels_in_any_order() {
        let expected = Rect::new(10, 20, 31, 41);
        assert_eq!(
            Rect::spanning(Point::new(10, 20), Point::new(30, 40)),
            expected
        );
        assert_eq!(
            Rect::spanning(Point::new(30, 20), Point::new(10, 40)),
            expected
        );
        assert_eq!(Rect::spanning(Point::new(5, 5), Point::new(5, 5)).area(), 1);
    }

    #[test]
    fn intersection_and_union() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 20, 20);
        assert_eq!(a.intersection(b), Rect::new(5, 5, 10, 10));
        assert_eq!(a.union(b), Rect::new(0, 0, 20, 20));
        assert_eq!(a.intersection(Rect::new(50, 50, 60, 60)), Rect::EMPTY);
        assert_eq!(a.union(Rect::EMPTY), a);
        assert_eq!(Rect::new(7, 7, 7, 9).union(b), b);
        assert_eq!(Rect::EMPTY.union(Rect::EMPTY), Rect::EMPTY);
    }

    #[test]
    fn contains_other_rectangles() {
        let outer = Rect::new(0, 0, 100, 100);
        assert!(outer.contains(Rect::new(10, 10, 100, 100)));
        assert!(!outer.contains(Rect::new(10, 10, 101, 100)));
        assert!(outer.contains(Rect::EMPTY));
    }

    #[test]
    fn rectangles_move_and_start_anywhere() {
        let cell = Rect::at(Point::new(-3, 40), Size::new(9, 19));
        assert_eq!(cell, Rect::new(-3, 40, 6, 59));
        assert_eq!(cell.offset(Point::new(9, -40)), Rect::new(6, 0, 15, 19));
        assert_eq!(
            Rect::at(Point::new(i32::MAX - 1, 0), Size::new(5, 1)).right,
            i32::MAX
        );
    }

    #[test]
    fn border_sides_cover_the_edge_without_overlapping() {
        let rect = Rect::new(10, 10, 30, 20);
        let sides = rect.border(2);
        assert_eq!(
            sides,
            [
                Rect::new(10, 10, 30, 12),
                Rect::new(10, 18, 30, 20),
                Rect::new(10, 12, 12, 18),
                Rect::new(28, 12, 30, 18),
            ]
        );
        assert_eq!(
            sides.iter().map(|side| side.area()).sum::<u64>(),
            rect.area() - Rect::new(12, 12, 28, 18).area()
        );
    }

    #[test]
    fn border_of_a_thin_rectangle_does_not_overflow_it() {
        let rect = Rect::new(0, 0, 3, 50);
        let sides = rect.border(4);
        for side in sides {
            assert!(rect.contains(side));
        }
        assert_eq!(sides.iter().map(|side| side.area()).sum::<u64>(), 150);
        assert_eq!(Rect::EMPTY.border(1), [Rect::EMPTY; 4]);
        assert_eq!(rect.border(0), [Rect::EMPTY; 4]);
    }
}
