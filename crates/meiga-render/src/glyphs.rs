//! The glyph cache: the coverage of each glyph, rasterized once, in slots of
//! the size of a cell.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::mem;

use meiga_core::geometry::{Rect, Size};
use meiga_core::text::CellChar;

/// The slots of a cache, unless it is created with another capacity. At
/// 150 % (cells of 14 × 28 pixels), 4096 slots are 1.6 MB, but the memory
/// grows only with the glyphs that are used: a few hundred in a usual
/// screen, which uses many fewer different characters than cells.
pub const CAPACITY: usize = 4096;

/// A glyph in the cache.
#[derive(Clone, Copy, Debug)]
pub struct Glyph<'a> {
    /// The coverage of each pixel of the cell, from 0 (none) to 255 (all),
    /// row after row.
    pub coverage: &'a [u8],
    /// The part of the cell that has coverage, which is the only part to
    /// draw. It is empty for a glyph without ink, such as a space.
    pub ink: Rect,
}

/// The place of a glyph in the cache. It is valid until the next insertion,
/// which can give its slot to another glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotId(u32);

/// The coverage of the glyphs already rasterized, each in a slot of the size
/// of a cell.
///
/// A character that takes two cells uses two slots, one for each half, so
/// every slot has the same size and every cell is drawn the same way. When
/// the cache is full, a new glyph takes the slot of one that was not used
/// recently, chosen with the clock algorithm ("second chance"): each slot
/// has a mark that finding it sets, and the clock hand goes round the slots,
/// removing the marks, until it finds one without the mark. Finding a glyph
/// costs a lookup in a hash table, and nothing else.
pub struct GlyphCache {
    cell: Size,
    /// The bytes of a slot: one per pixel of a cell.
    slot_len: usize,
    /// The coverage of all the slots, one after the other.
    coverage: Vec<u8>,
    slots: Vec<Slot>,
    /// The slot of each glyph, by [`key`].
    index: HashMap<u32, u32, BuildHasherDefault<KeyHasher>>,
    capacity: usize,
    /// The next slot that the clock looks at when the cache is full.
    hand: usize,
}

struct Slot {
    key: u32,
    ink: Rect,
    /// Whether it was used since the clock hand last passed.
    used: bool,
}

impl GlyphCache {
    /// An empty cache for cells of `cell`, with room for `capacity` slots
    /// (at least two, the halves of a wide character).
    pub fn new(cell: Size, capacity: usize) -> Self {
        Self {
            cell,
            slot_len: cell.width as usize * cell.height as usize,
            coverage: Vec::new(),
            slots: Vec::new(),
            index: HashMap::default(),
            capacity: capacity.max(2),
            hand: 0,
        }
    }

    pub const fn cell(&self) -> Size {
        self.cell
    }

    /// How many glyphs it holds.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// The slot of `glyph` in `style`, if it is in the cache. Marks it as
    /// used, so that the next insertions keep it.
    pub fn find(&mut self, glyph: CellChar, style: u8) -> Option<SlotId> {
        let slot = *self.index.get(&key(glyph, style))?;
        self.slots[slot as usize].used = true;
        Some(SlotId(slot))
    }

    /// Puts `glyph` in `style` in the cache, with the coverage that
    /// `rasterize` writes in its slot: a cell of zeros, row after row.
    pub fn insert(
        &mut self,
        glyph: CellChar,
        style: u8,
        rasterize: impl FnOnce(&mut [u8]),
    ) -> SlotId {
        let key = key(glyph, style);
        let slot = match self.index.get(&key) {
            Some(&slot) => slot as usize,
            None => self.free_slot(),
        };
        let coverage = &mut self.coverage[slot * self.slot_len..][..self.slot_len];
        coverage.fill(0);
        rasterize(coverage);
        self.slots[slot] = Slot {
            key,
            ink: ink(coverage, self.cell.width as usize),
            used: true,
        };
        self.index.insert(key, slot as u32);
        SlotId(slot as u32)
    }

    pub fn glyph(&self, slot: SlotId) -> Glyph<'_> {
        let slot = slot.0 as usize;
        Glyph {
            coverage: &self.coverage[slot * self.slot_len..][..self.slot_len],
            ink: self.slots[slot].ink,
        }
    }

    /// A slot for a new glyph: a new one while there is room, and then the
    /// first one without the mark after the clock hand.
    fn free_slot(&mut self) -> usize {
        if self.slots.len() < self.capacity {
            self.slots.push(Slot {
                key: 0,
                ink: Rect::EMPTY,
                used: false,
            });
            self.coverage.resize(self.coverage.len() + self.slot_len, 0);
            return self.slots.len() - 1;
        }
        // It ends in at most two rounds: the first one removes every mark.
        loop {
            let slot = self.hand;
            self.hand = (slot + 1) % self.slots.len();
            if !mem::take(&mut self.slots[slot].used) {
                self.index.remove(&self.slots[slot].key);
                return slot;
            }
        }
    }
}

/// The key of a glyph in the index: the character; its style, from 0 to 3,
/// in the next two bits; and, in the two low bits, which cell it is.
/// Characters go up to 0x10FFFF, so it fits in 25 bits.
fn key(glyph: CellChar, style: u8) -> u32 {
    let (c, cell) = match glyph {
        CellChar::Blank => return 0,
        CellChar::Narrow(c) => (c, 1),
        CellChar::WideLeft(c) => (c, 2),
        CellChar::WideRight(c) => (c, 3),
    };
    u32::from(c) << 4 | u32::from(style & 3) << 2 | cell
}

/// The smallest rectangle with all the pixels that have coverage, in rows
/// of `width` bytes.
fn ink(coverage: &[u8], width: usize) -> Rect {
    let (mut left, mut top, mut right, mut bottom) = (usize::MAX, usize::MAX, 0, 0);
    for (y, row) in coverage.chunks_exact(width.max(1)).enumerate() {
        if let Some(first) = row.iter().position(|&c| c != 0) {
            let last = row.iter().rposition(|&c| c != 0).unwrap_or(first);
            left = left.min(first);
            right = right.max(last + 1);
            top = top.min(y);
            bottom = y + 1;
        }
    }
    if top == usize::MAX {
        return Rect::EMPTY;
    }
    // A cell is much smaller than i32::MAX pixels.
    Rect::new(left as i32, top as i32, right as i32, bottom as i32)
}

/// The hash of the index: a multiplication, as in the Fx hash of the Rust
/// compiler. The keys are small numbers that nobody outside meiga chooses,
/// so they need neither the protection against collisions of the standard
/// hash (SipHash) nor its cost.
#[derive(Default)]
struct KeyHasher(u64);

impl Hasher for KeyHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_u64(u64::from(byte));
        }
    }

    fn write_u32(&mut self, value: u32) {
        self.write_u64(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CELL: Size = Size::new(4, 3);

    /// Rasterizes a glyph with coverage 200 at the pixel (`x`, `y`).
    fn dot(x: usize, y: usize) -> impl FnOnce(&mut [u8]) {
        move |coverage| coverage[y * CELL.width as usize + x] = 200
    }

    #[test]
    fn a_glyph_is_rasterized_once_and_found_after() {
        let mut cache = GlyphCache::new(CELL, 8);
        let a = CellChar::Narrow('a');
        assert_eq!(cache.find(a, 0), None);
        let slot = cache.insert(a, 0, |coverage| {
            assert_eq!(coverage, [0; 12]);
            coverage[5] = 255;
            coverage[11] = 7;
        });
        assert_eq!(cache.find(a, 0), Some(slot));
        let glyph = cache.glyph(slot);
        assert_eq!(glyph.coverage[5], 255);
        assert_eq!(glyph.ink, Rect::new(1, 1, 4, 3));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn a_glyph_without_ink_has_an_empty_ink_box() {
        let mut cache = GlyphCache::new(CELL, 8);
        let slot = cache.insert(CellChar::Narrow('\u{a0}'), 0, |_| {});
        assert!(cache.glyph(slot).ink.is_empty());
    }

    #[test]
    fn each_half_of_a_wide_character_has_its_slot() {
        let mut cache = GlyphCache::new(CELL, 8);
        let left = cache.insert(CellChar::WideLeft('中'), 0, dot(0, 0));
        let right = cache.insert(CellChar::WideRight('中'), 0, dot(3, 2));
        assert_ne!(left, right);
        assert_eq!(cache.glyph(left).ink, Rect::new(0, 0, 1, 1));
        assert_eq!(cache.glyph(right).ink, Rect::new(3, 2, 4, 3));
        assert_eq!(cache.find(CellChar::Narrow('中'), 0), None);
    }

    #[test]
    fn each_style_of_a_character_has_its_slot() {
        let mut cache = GlyphCache::new(CELL, 8);
        let a = CellChar::Narrow('a');
        let regular = cache.insert(a, 0, dot(0, 0));
        let bold = cache.insert(a, 1, dot(1, 1));
        assert_ne!(regular, bold);
        assert_eq!(cache.find(a, 0), Some(regular));
        assert_eq!(cache.find(a, 1), Some(bold));
        assert_eq!(cache.find(a, 3), None);
        assert_eq!(cache.glyph(bold).ink, Rect::new(1, 1, 2, 2));
    }

    #[test]
    fn inserting_again_reuses_the_slot() {
        let mut cache = GlyphCache::new(CELL, 8);
        let first = cache.insert(CellChar::Narrow('x'), 0, dot(0, 0));
        let again = cache.insert(CellChar::Narrow('x'), 0, dot(1, 1));
        assert_eq!(first, again);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.glyph(again).ink, Rect::new(1, 1, 2, 2));
    }

    #[test]
    fn when_it_is_full_a_glyph_used_since_the_last_round_stays() {
        let mut cache = GlyphCache::new(CELL, 3);
        for c in ['a', 'b', 'c'] {
            cache.insert(CellChar::Narrow(c), 0, dot(0, 0));
        }
        // All three are marked: the first round removes the marks, and the
        // second one takes the slot of 'a'.
        cache.insert(CellChar::Narrow('d'), 0, dot(0, 0));
        assert_eq!(cache.find(CellChar::Narrow('a'), 0), None);
        // 'b' is used again, 'c' is not: 'e' takes the slot of 'c'.
        assert!(cache.find(CellChar::Narrow('b'), 0).is_some());
        cache.insert(CellChar::Narrow('e'), 0, dot(0, 0));
        assert!(cache.find(CellChar::Narrow('b'), 0).is_some());
        assert_eq!(cache.find(CellChar::Narrow('c'), 0), None);
        assert!(cache.find(CellChar::Narrow('d'), 0).is_some());
        assert!(cache.find(CellChar::Narrow('e'), 0).is_some());
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn a_reused_slot_is_cleared_first() {
        let mut cache = GlyphCache::new(CELL, 2);
        cache.insert(CellChar::Narrow('a'), 0, |coverage| coverage.fill(255));
        cache.insert(CellChar::Narrow('b'), 0, dot(0, 0));
        let c = cache.insert(CellChar::Narrow('c'), 0, |coverage| {
            assert_eq!(coverage, [0; 12]);
        });
        assert!(cache.glyph(c).ink.is_empty());
    }
}
