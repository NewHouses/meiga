//! What the program reads when the user pastes text.
//!
//! As in Windows Terminal: the text loses its control characters, except
//! the tab and the ends of line, so that it cannot send a key such as Esc
//! nor end a bracketed paste before its time; each end of line becomes a
//! CR, the key Enter; and, if the program did not ask for bracketed paste
//! (`?2004`), a text of a single line loses the blanks at its end, so that
//! a command copied with its end of line does not run on its own.

use crate::term::Modes;

/// Text ready to paste.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paste {
    /// The text, without control characters other than the tab and CR, at
    /// the end of each line.
    pub text: String,
    /// Whether it goes marked as pasted text (bracketed paste).
    pub bracketed: bool,
}

impl Paste {
    /// The text that `clipboard` gives to a program with `modes`.
    pub fn new(clipboard: &str, modes: &Modes) -> Self {
        let bracketed = modes.bracketed_paste;
        let clipboard = if bracketed {
            clipboard
        } else {
            trim(clipboard)
        };
        let mut text = String::with_capacity(clipboard.len());
        let mut after_cr = false;
        for character in clipboard.chars() {
            match character {
                // CR and LF, CRLF and LF CR... each line ends with one CR.
                '\n' if after_cr => {}
                '\r' | '\n' => text.push('\r'),
                '\t' => text.push('\t'),
                // C0, DEL and C1.
                '\0'..='\x1f' | '\x7f'..='\u{9f}' => {}
                _ => text.push(character),
            }
            after_cr = character == '\r';
        }
        Self { text, bracketed }
    }

    /// Whether it has more than one line. Without bracketed paste, a shell
    /// runs each line as it reads it.
    pub fn is_multiline(&self) -> bool {
        self.text.contains('\r')
    }

    /// Whether there is nothing to send. A bracketed paste without text
    /// still goes: some programs read an image from the clipboard when
    /// they get it.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && !self.bracketed
    }

    /// Adds to `out` what the program reads.
    pub fn write_to(&self, out: &mut Vec<u8>) {
        if self.bracketed {
            out.extend_from_slice(b"\x1b[200~");
        }
        out.extend_from_slice(self.text.as_bytes());
        if self.bracketed {
            out.extend_from_slice(b"\x1b[201~");
        }
    }
}

/// `text` without the blanks at its end, if it has a single line: nothing
/// if it only has blanks.
fn trim(text: &str) -> &str {
    let trimmed = text.trim_end_matches(['\t', '\n', '\x0b', '\x0c', '\r', ' ']);
    if trimmed.contains(['\n', '\r']) {
        text
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain() -> Modes {
        Modes::default()
    }

    fn bracketed() -> Modes {
        Modes {
            bracketed_paste: true,
            ..Modes::default()
        }
    }

    fn bytes(clipboard: &str, modes: &Modes) -> Vec<u8> {
        let mut out = Vec::new();
        Paste::new(clipboard, modes).write_to(&mut out);
        out
    }

    #[test]
    fn each_end_of_line_becomes_a_cr() {
        let paste = Paste::new("un\r\ndous\ntres\rcatro\n\rcinco", &plain());
        assert_eq!(paste.text, "un\rdous\rtres\rcatro\r\rcinco");
        assert!(paste.is_multiline());
        assert!(!Paste::new("só unha", &plain()).is_multiline());
    }

    #[test]
    fn control_characters_go_away_but_tabs_and_text_stay() {
        assert_eq!(
            bytes("a\x1b[201~b\x07\tc\x7f\u{85}\u{9b}ñ€😀", &bracketed()),
            "\x1b[200~a[201~b\tcñ€😀\x1b[201~".as_bytes()
        );
    }

    #[test]
    fn without_bracketed_paste_a_single_line_loses_the_blanks_at_its_end() {
        assert_eq!(bytes("  ls -l \t\r\n", &plain()), b"  ls -l");
        assert!(Paste::new("   \r\n", &plain()).is_empty());
        // More lines: as they are.
        assert_eq!(bytes("ls\r\npwd\r\n", &plain()), b"ls\rpwd\r");
        // With bracketed paste, the program decides.
        assert_eq!(bytes("ls \n", &bracketed()), b"\x1b[200~ls \r\x1b[201~");
    }

    #[test]
    fn an_empty_clipboard_only_goes_with_bracketed_paste() {
        assert!(Paste::new("", &plain()).is_empty());
        let empty = Paste::new("", &bracketed());
        assert!(!empty.is_empty());
        assert_eq!(bytes("", &bracketed()), b"\x1b[200~\x1b[201~");
    }
}
