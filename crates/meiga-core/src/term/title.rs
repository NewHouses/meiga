//! The title that the program gives to its tab, and the ones that it saves
//! to give again later (ADR 0029). The terminal keeps them, and not
//! `alacritty_terminal`, which empties them on a reset (`ESC c`) without
//! telling it: the tab kept its title, but a program that then saved it
//! and gave it again got none, and the tab showed the name of its profile.
//!
//! As in Windows Terminal and the other terminals, a reset keeps the title,
//! and, as in xterm and Kitty, the saved ones too: they are of the window,
//! not of the screen. A new program in the tab (ADR 0024) starts without
//! them, as in a new tab.

use std::collections::VecDeque;
use std::mem;

/// The most titles that the program can save (`CSI 22 t`), as in xterm and
/// Kitty: saving another one drops the oldest.
pub(super) const SAVED_TITLES: usize = 10;

/// The longest title, in UTF-16 code units, as Windows counts them. A longer
/// one is ignored, as in Windows Terminal and in the console host of ConPTY,
/// which keeps the title that it had; xterm ignores titles of more than 1000
/// bytes.
pub(super) const MAX_TITLE: usize = 1000;

/// The title of the program, and the ones that it saved.
#[derive(Default)]
pub(super) struct Titles {
    /// The title that the program gave, if it gave one.
    current: Option<String>,
    /// The titles that it saved, the newest last.
    saved: VecDeque<Option<String>>,
    /// The title changed since the terminal last took its changes.
    changed: bool,
}

impl Titles {
    /// The title that the program gave, if it gave one.
    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// OSC 0 and 2, whose title `vte` gives without the spaces at its ends.
    /// An empty one is no title: the tab shows the name of its profile, as
    /// in Windows Terminal. One longer than [`MAX_TITLE`] is ignored.
    pub fn set(&mut self, title: Option<String>) {
        let title = title.filter(|title| !title.is_empty());
        if title.as_deref().is_some_and(too_long) {
            return;
        }
        self.replace(title);
    }

    /// `CSI 22 t`: saves the title, also when there is none, which giving it
    /// again brings back.
    pub fn push(&mut self) {
        if self.saved.len() == SAVED_TITLES {
            self.saved.pop_front();
        }
        self.saved.push_back(self.current.clone());
    }

    /// `CSI 23 t`: gives again the last title saved, if there is one.
    pub fn pop(&mut self) {
        if let Some(title) = self.saved.pop_back() {
            self.replace(title);
        }
    }

    /// Whether the title changed since the last time that this was asked.
    /// Giving the same title again is not a change.
    pub fn take_changed(&mut self) -> bool {
        mem::take(&mut self.changed)
    }

    fn replace(&mut self, title: Option<String>) {
        if self.current != title {
            self.current = title;
            self.changed = true;
        }
    }
}

/// Whether `title` has more than [`MAX_TITLE`] UTF-16 code units. It counts
/// at most one more.
fn too_long(title: &str) -> bool {
    title.encode_utf16().nth(MAX_TITLE).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titled(title: &str) -> Titles {
        let mut titles = Titles::default();
        titles.set(Some(title.to_owned()));
        titles.take_changed();
        titles
    }

    #[test]
    fn a_title_is_a_change_only_when_it_is_another_one() {
        let mut titles = Titles::default();
        assert_eq!(titles.current(), None);
        assert!(!titles.take_changed());
        titles.set(Some("pwsh".to_owned()));
        assert_eq!(titles.current(), Some("pwsh"));
        assert!(titles.take_changed());
        assert!(!titles.take_changed());
        titles.set(Some("pwsh".to_owned()));
        assert!(!titles.take_changed());
        // Another one and back before the changes are taken: it changed.
        titles.set(Some("vim".to_owned()));
        titles.set(Some("pwsh".to_owned()));
        assert!(titles.take_changed());
        assert_eq!(titles.current(), Some("pwsh"));
    }

    #[test]
    fn an_empty_title_is_no_title() {
        for empty in [Some(String::new()), None] {
            let mut titles = titled("pwsh");
            titles.set(empty.clone());
            assert_eq!(titles.current(), None, "{empty:?}");
            assert!(titles.take_changed());
            titles.set(empty);
            assert!(!titles.take_changed());
        }
    }

    #[test]
    fn a_title_of_more_than_1000_utf16_code_units_is_ignored() {
        // One, two, three and four bytes in UTF-8; the last one, two code
        // units in UTF-16.
        for (character, units) in [("x", 1), ("ñ", 1), ("中", 1), ("😀", 2)] {
            let fits = character.repeat(MAX_TITLE / units);
            let mut titles = titled("pwsh");
            titles.set(Some(fits.clone()));
            assert_eq!(titles.current(), Some(&*fits), "{character}");
            assert!(titles.take_changed());
            for longer in [format!("{fits}x"), fits.repeat(3)] {
                titles.set(Some(longer));
                assert_eq!(titles.current(), Some(&*fits), "{character}");
                assert!(!titles.take_changed());
            }
        }
    }

    #[test]
    fn saved_titles_come_back_newest_first() {
        let mut titles = titled("pwsh");
        titles.push();
        titles.set(Some("vim".to_owned()));
        titles.push();
        titles.set(Some("less".to_owned()));
        titles.take_changed();
        titles.pop();
        assert_eq!(titles.current(), Some("vim"));
        assert!(titles.take_changed());
        titles.pop();
        assert_eq!(titles.current(), Some("pwsh"));
        assert!(titles.take_changed());
        // None left: nothing changes.
        titles.pop();
        assert_eq!(titles.current(), Some("pwsh"));
        assert!(!titles.take_changed());
    }

    #[test]
    fn no_title_can_be_saved_and_given_again() {
        let mut titles = Titles::default();
        titles.push();
        titles.set(Some("vim".to_owned()));
        titles.take_changed();
        titles.pop();
        assert_eq!(titles.current(), None);
        assert!(titles.take_changed());
    }

    #[test]
    fn giving_again_the_same_title_is_not_a_change() {
        let mut titles = titled("pwsh");
        titles.push();
        titles.pop();
        assert_eq!(titles.current(), Some("pwsh"));
        assert!(!titles.take_changed());
    }

    #[test]
    fn saving_more_than_10_titles_drops_the_oldest() {
        let mut titles = Titles::default();
        for n in 0..=SAVED_TITLES {
            titles.set(Some(n.to_string()));
            titles.push();
        }
        titles.set(Some("last".to_owned()));
        for n in (1..=SAVED_TITLES).rev() {
            titles.pop();
            assert_eq!(titles.current(), Some(&*n.to_string()));
        }
        // The first one, 0, went away.
        titles.take_changed();
        titles.pop();
        assert_eq!(titles.current(), Some("1"));
        assert!(!titles.take_changed());
    }
}
