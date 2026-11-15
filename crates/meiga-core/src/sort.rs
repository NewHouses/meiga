//! A sort for the short lists of meiga: the variables of the environment,
//! the problems of the configuration, the versions of PowerShell and the
//! distributions of WSL.
//!
//! The sorts of the standard library are made for long lists, and each
//! kind of list that they sort adds a few kilobytes of code to `meiga.exe`:
//! about 33 KB for those five, measured in subphase 2.5. An insertion sort
//! takes a few hundred bytes for each, and with a few dozen items it is as
//! fast; with a list that is nearly sorted already, as the environment that
//! Windows gives, faster.

use std::cmp::Ordering;

/// Sorts `items` as `compare` orders them. Items that it finds equal keep
/// their order, as with [`slice::sort_by`].
pub fn insertion_sort_by<T>(items: &mut [T], mut compare: impl FnMut(&T, &T) -> Ordering) {
    for end in 1..items.len() {
        // The items before `end` are sorted: the one at `end` goes back past
        // the ones that go after it.
        let mut at = end;
        while at > 0 && compare(&items[at - 1], &items[at]) == Ordering::Greater {
            items.swap(at - 1, at);
            at -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_sorts_as_the_stable_sort_of_the_standard_library() {
        // Keys from 0 to 7, with the place of each item: equal keys keep the
        // order of their places only if the sort is stable.
        let mut state = 0x2545_f491_u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        };
        for length in 0..40 {
            let items: Vec<(u32, usize)> = (0..length).map(|place| (next() % 8, place)).collect();
            let mut sorted = items.clone();
            insertion_sort_by(&mut sorted, |a, b| a.0.cmp(&b.0));
            let mut expected = items;
            expected.sort_by_key(|item| item.0);
            assert_eq!(sorted, expected);
        }
    }

    #[test]
    fn it_follows_the_order_that_it_is_given() {
        let mut items = [3, 1, 4, 1, 5, 9, 2, 6];
        insertion_sort_by(&mut items, |a, b| b.cmp(a));
        assert_eq!(items, [9, 6, 5, 4, 3, 2, 1, 1]);
    }
}
