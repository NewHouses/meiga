//! The parts of the window that changed since the last frame.

use crate::geometry::Rect;

/// Most rectangles that a [`Damage`] keeps. Each one is copied to the screen
/// separately.
pub const MAX_RECTS: usize = 8;

/// What copying one more rectangle to the screen costs, counted in pixels.
///
/// Two rectangles are merged into the one that contains both when it has at
/// most this many pixels more than the two together. The value, a square of
/// 64 × 64, is an estimate, to adjust with measurements.
pub const MERGE_SLACK: u64 = 64 * 64;

/// The parts of the window that changed since the last frame, as at most
/// [`MAX_RECTS`] rectangles.
///
/// Each rectangle that is added ends up inside one of the rectangles that it
/// keeps: a rectangle is never cut, only merged with others. It has a fixed
/// capacity, so adding never allocates memory.
#[derive(Clone, Debug)]
pub struct Damage {
    rects: [Rect; MAX_RECTS],
    len: usize,
}

impl Damage {
    pub const fn new() -> Self {
        Self {
            rects: [Rect::EMPTY; MAX_RECTS],
            len: 0,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The rectangles, none of them empty.
    pub fn rects(&self) -> &[Rect] {
        &self.rects[..self.len]
    }

    /// Forgets every change, for example after drawing them.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Adds a rectangle that changed.
    ///
    /// While the rectangle that contains it and the cheapest of the others
    /// has at most [`MERGE_SLACK`] pixels more than the two, they are merged.
    /// When there is no room for one more, it is merged anyway.
    pub fn add(&mut self, rect: Rect) {
        if rect.is_empty() {
            return;
        }
        let mut rect = rect;
        while let Some((index, extra)) = self.cheapest_merge(rect) {
            if extra > i128::from(MERGE_SLACK) && self.len < MAX_RECTS {
                break;
            }
            rect = rect.union(self.rects[index]);
            self.len -= 1;
            self.rects[index] = self.rects[self.len];
        }
        self.rects[self.len] = rect;
        self.len += 1;
    }

    /// The rectangle that grows the least when merged with `rect`, and how
    /// many pixels it would grow.
    fn cheapest_merge(&self, rect: Rect) -> Option<(usize, i128)> {
        self.rects()
            .iter()
            .enumerate()
            .map(|(index, other)| (index, extra_pixels(*other, rect)))
            .min_by_key(|&(_, extra)| extra)
    }
}

impl Default for Damage {
    fn default() -> Self {
        Self::new()
    }
}

/// How many pixels the rectangle that contains `a` and `b` has more than the
/// two together. It is negative when they overlap, because the pixels in
/// common would be copied twice.
fn extra_pixels(a: Rect, b: Rect) -> i128 {
    i128::from(a.union(b).area()) - i128::from(a.area()) - i128::from(b.area())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn damage(rects: &[Rect]) -> Damage {
        let mut damage = Damage::new();
        for rect in rects {
            damage.add(*rect);
        }
        damage
    }

    #[test]
    fn ignores_empty_rectangles() {
        let damage = damage(&[Rect::EMPTY, Rect::new(5, 5, 5, 9)]);
        assert!(damage.is_empty());
        assert_eq!(damage.rects(), []);
    }

    #[test]
    fn a_rectangle_inside_another_adds_nothing() {
        let big = Rect::new(0, 0, 500, 500);
        let small = Rect::new(10, 10, 20, 20);
        assert_eq!(damage(&[big, small]).rects(), [big]);
        assert_eq!(damage(&[small, big]).rects(), [big]);
    }

    #[test]
    fn rows_one_after_another_become_one_rectangle() {
        let damage = damage(&[
            Rect::new(0, 0, 1920, 20),
            Rect::new(0, 40, 1920, 60),
            Rect::new(0, 20, 1920, 40),
        ]);
        assert_eq!(damage.rects(), [Rect::new(0, 0, 1920, 60)]);
    }

    #[test]
    fn distant_changes_stay_apart() {
        let cursor = Rect::new(10, 10, 20, 30);
        let spinner = Rect::new(1500, 900, 1510, 920);
        assert_eq!(damage(&[cursor, spinner]).rects(), [cursor, spinner]);
    }

    #[test]
    fn close_changes_are_merged() {
        let damage = damage(&[Rect::new(0, 0, 10, 20), Rect::new(30, 0, 40, 20)]);
        assert_eq!(damage.rects(), [Rect::new(0, 0, 40, 20)]);
    }

    #[test]
    fn overlapping_changes_are_merged() {
        let before = Rect::new(100, 100, 400, 300);
        let after = Rect::new(110, 105, 410, 305);
        assert_eq!(
            damage(&[before, after]).rects(),
            [Rect::new(100, 100, 410, 305)]
        );
    }

    #[test]
    fn clear_forgets_every_change() {
        let mut damage = damage(&[Rect::new(0, 0, 10, 10)]);
        damage.clear();
        assert!(damage.is_empty());
    }

    #[test]
    fn every_change_stays_covered_within_the_capacity() {
        // A fixed pseudo-random sequence (xorshift), so that a failure can be
        // repeated.
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let mut next = |limit: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            i32::try_from(state % limit).unwrap_or(0)
        };
        for _ in 0..200 {
            let mut damage = Damage::new();
            let mut added = Vec::new();
            for _ in 0..1 + next(40) {
                let (left, top) = (next(2000) - 50, next(1200) - 50);
                let rect = Rect::new(left, top, left + next(300), top + next(100));
                damage.add(rect);
                added.push(rect);
            }
            assert!(damage.rects().len() <= MAX_RECTS);
            assert!(damage.rects().iter().all(|rect| !rect.is_empty()));
            for rect in added.iter().filter(|rect| !rect.is_empty()) {
                assert!(
                    damage.rects().iter().any(|kept| kept.contains(*rect)),
                    "{rect:?} is not inside any of {:?}",
                    damage.rects()
                );
            }
        }
    }
}
