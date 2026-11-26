//! Where the caret of a text field stops, and where its words start.
//!
//! The caret stops between the grapheme clusters of Unicode (UAX #29), the
//! extended ones, as in Chrome: never between a letter and its accent, nor
//! inside an emoji with a skin tone, a flag or a sequence joined with a
//! zero-width joiner (ZWJ).
//!
//! A word is a run of clusters of the same class: letters, digits and `_`
//! make words; each run of other signs is another word; and the spaces
//! separate them. So `main.rs` is three words, and `C:\Users\x` five. Each
//! cluster has the class of its first character.

use std::ops::Range;

use unicode_segmentation::GraphemeCursor;
use unicode_width::UnicodeWidthChar;

/// The end of the cluster that starts at `at`, or that has `at` inside it:
/// the next place where the caret stops, or the end of `text`.
pub(super) fn next_cluster(text: &str, at: usize) -> usize {
    // With the whole text, the cursor never asks for more of it.
    GraphemeCursor::new(at, text.len(), true)
        .next_boundary(text, 0)
        .ok()
        .flatten()
        .unwrap_or(text.len())
}

/// The start of the cluster that ends at `at`, or that has `at` inside it:
/// the previous place where the caret stops, or 0.
pub(super) fn previous_cluster(text: &str, at: usize) -> usize {
    GraphemeCursor::new(at, text.len(), true)
        .prev_boundary(text, 0)
        .ok()
        .flatten()
        .unwrap_or(0)
}

/// Whether the caret can stop at `at`: between two clusters, or at an end
/// of `text`.
pub(super) fn is_stop(text: &str, at: usize) -> bool {
    GraphemeCursor::new(at, text.len(), true)
        .is_boundary(text, 0)
        .unwrap_or(true)
}

/// What a cluster is in a word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Class {
    /// A space, which separates words.
    Space,
    /// A letter, a digit or `_`, which make words; and a character without
    /// width at the start of a cluster, such as an accent without a letter
    /// before it, or a zero-width space.
    Word,
    /// Any other sign, such as `.`, `\`, `-` or an emoji: each run of them is
    /// a word.
    Sign,
}

/// The class of `cluster`, which is that of its first character.
fn class(cluster: &str) -> Class {
    match cluster.chars().next() {
        Some(c) if c.is_whitespace() => Class::Space,
        Some(c) if c.is_alphanumeric() || c == '_' || c.width() == Some(0) => Class::Word,
        _ => Class::Sign,
    }
}

/// The class of the cluster that starts at `at`, if one does.
pub(super) fn class_after(text: &str, at: usize) -> Option<Class> {
    (at < text.len()).then(|| class(&text[at..next_cluster(text, at)]))
}

/// The class of the cluster that ends at `at`, if one does.
pub(super) fn class_before(text: &str, at: usize) -> Option<Class> {
    (at > 0).then(|| class(&text[previous_cluster(text, at)..at]))
}

/// Where Ctrl+Right goes from `at`: to the start of the next word, past the
/// rest of the word where `at` is and the spaces after it, as in Windows;
/// or to the end of `text`.
pub(super) fn next_word(text: &str, at: usize) -> usize {
    let mut at = at;
    if let Some(class) = class_after(text, at).filter(|&class| class != Class::Space) {
        while class_after(text, at) == Some(class) {
            at = next_cluster(text, at);
        }
    }
    while class_after(text, at) == Some(Class::Space) {
        at = next_cluster(text, at);
    }
    at
}

/// Where Ctrl+Left goes from `at`: back past the spaces before it, to the
/// start of the word before them; or to the start of `text`.
pub(super) fn previous_word(text: &str, at: usize) -> usize {
    let mut at = at;
    while class_before(text, at) == Some(Class::Space) {
        at = previous_cluster(text, at);
    }
    if let Some(class) = class_before(text, at) {
        while class_before(text, at) == Some(class) {
            at = previous_cluster(text, at);
        }
    }
    at
}

/// The run of clusters of the same class as the one that starts at `at`,
/// which a double click on it selects: a word, a run of signs or a run of
/// spaces. Nothing at the end of `text`.
pub(super) fn word_at(text: &str, at: usize) -> Range<usize> {
    let Some(class) = class_after(text, at) else {
        return at..at;
    };
    let mut start = at;
    while class_before(text, start) == Some(class) {
        start = previous_cluster(text, start);
    }
    let mut end = at;
    while class_after(text, end) == Some(class) {
        end = next_cluster(text, end);
    }
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The places where the caret stops in `text`, from its start to its
    /// end, one cluster at a time.
    fn stops(text: &str) -> Vec<usize> {
        let mut stops = vec![0];
        while let Some(&last) = stops.last().filter(|&&last| last < text.len()) {
            stops.push(next_cluster(text, last));
        }
        stops
    }

    /// The same, from the end to the start.
    fn stops_back(text: &str) -> Vec<usize> {
        let mut stops = vec![text.len()];
        while let Some(&last) = stops.last().filter(|&&last| last > 0) {
            stops.push(previous_cluster(text, last));
        }
        stops.reverse();
        stops
    }

    #[test]
    fn the_caret_stops_between_clusters_in_both_directions() {
        for (text, expected) in [
            ("", &[0][..]),
            ("ab", &[0, 1, 2]),
            // Precomposed letters are a single character.
            ("olá", &[0, 1, 2, 4]),
            // "e" and a combining acute accent; "n" and a combining tilde.
            ("e\u{301}n\u{303}", &[0, 3, 6]),
            ("中文", &[0, 3, 6]),
            // A thumbs up with a skin tone.
            ("👍🏽!", &[0, 8, 9]),
            // A family: man, ZWJ, woman, ZWJ, girl.
            ("👨\u{200d}👩\u{200d}👧x", &[0, 18, 19]),
            // Two flags: the regional indicators go in pairs.
            ("🇪🇸🇬🇧", &[0, 8, 16]),
            // A heart with the variation selector of emoji.
            ("❤\u{fe0f}a", &[0, 6, 7]),
            // An Indic conjunct (Devanagari KSSA: KA, VIRAMA, SSA).
            ("क्ष", &[0, 9]),
            // Hangul jamo that make one syllable.
            ("\u{1100}\u{1161}\u{11a8}", &[0, 9]),
        ] {
            assert_eq!(stops(text), expected, "{text:?}");
            assert_eq!(stops_back(text), expected, "{text:?}");
            for at in 0..=text.len() {
                if text.is_char_boundary(at) {
                    assert_eq!(
                        is_stop(text, at),
                        expected.contains(&at),
                        "{text:?} at {at}"
                    );
                }
            }
        }
    }

    #[test]
    fn inside_a_cluster_the_caret_goes_to_its_ends() {
        // Inside the family, after the man.
        let family = "a👨\u{200d}👩\u{200d}👧";
        assert_eq!(previous_cluster(family, 5), 1);
        assert_eq!(next_cluster(family, 5), family.len());
        // Between a letter and its accent.
        assert_eq!(previous_cluster("e\u{301}", 1), 0);
        assert_eq!(next_cluster("e\u{301}", 1), 3);
    }

    #[test]
    fn letters_digits_and_underscores_make_words_and_other_signs_another() {
        for (text, class) in [
            (" ", Class::Space),
            ("\t", Class::Space),
            // A no-break space and an ideographic space.
            ("\u{a0}", Class::Space),
            ("\u{3000}", Class::Space),
            ("a", Class::Word),
            ("Ñ", Class::Word),
            ("7", Class::Word),
            ("_", Class::Word),
            ("中", Class::Word),
            ("ά", Class::Word),
            // A combining accent with no letter before it.
            ("\u{301}", Class::Word),
            (".", Class::Sign),
            ("\\", Class::Sign),
            ("-", Class::Sign),
            ("😀", Class::Sign),
            ("€", Class::Sign),
        ] {
            assert_eq!(class_after(text, 0), Some(class), "{text:?}");
            assert_eq!(class_before(text, text.len()), Some(class), "{text:?}");
        }
        assert_eq!(class_after("ab", 2), None);
        assert_eq!(class_before("ab", 0), None);
    }

    /// Where Ctrl+Right stops, from the start of `text` to its end, and
    /// where Ctrl+Left stops, from its end to its start.
    fn words(text: &str) -> (Vec<usize>, Vec<usize>) {
        let mut forward = vec![0];
        while let Some(&last) = forward.last().filter(|&&last| last < text.len()) {
            forward.push(next_word(text, last));
        }
        let mut back = vec![text.len()];
        while let Some(&last) = back.last().filter(|&&last| last > 0) {
            back.push(previous_word(text, last));
        }
        back.reverse();
        (forward, back)
    }

    #[test]
    fn ctrl_and_the_arrows_stop_at_the_start_of_each_word() {
        for (text, starts) in [
            ("", &[0][..]),
            ("foo bar", &[0, 4, 7]),
            ("  foo   bar  ", &[0, 2, 8, 13]),
            ("main.rs", &[0, 4, 5, 7]),
            ("C:\\Users\\x", &[0, 1, 3, 8, 9, 10]),
            ("foo_bar-baz", &[0, 7, 8, 11]),
            ("...!", &[0, 4]),
            ("olá, mundo", &[0, 4, 6, 11]),
            ("a 😀😀 b", &[0, 2, 11, 12]),
        ] {
            let (forward, back) = words(text);
            assert_eq!(forward, starts, "{text:?} forward");
            assert_eq!(back, starts, "{text:?} back");
        }
    }

    #[test]
    fn ctrl_and_the_arrows_go_from_the_middle_of_a_word_to_its_ends() {
        let text = "foo bar";
        assert_eq!(next_word(text, 1), 4);
        assert_eq!(previous_word(text, 6), 4);
        // From the spaces between two words.
        let text = "foo   bar";
        assert_eq!(next_word(text, 4), 6);
        assert_eq!(previous_word(text, 4), 0);
    }

    #[test]
    fn a_double_click_takes_the_run_of_its_class() {
        let text = "foo  bar.rs";
        for (at, word) in [
            (0, 0..3),
            (2, 0..3),
            (3, 3..5),
            (4, 3..5),
            (5, 5..8),
            (8, 8..9),
            (9, 9..11),
            (11, 11..11),
        ] {
            assert_eq!(word_at(text, at), word, "at {at}");
        }
        // A cluster is never split.
        assert_eq!(word_at("x e\u{301}e\u{301} y", 2), 2..8);
        assert_eq!(word_at("", 0), 0..0);
    }
}
