//! Copying the selection to the clipboard, and pasting the clipboard into
//! the program.

use std::sync::Mutex;

use meiga_core::paste::Paste;
use meiga_win::clipboard;
use meiga_win::conpty::Input;
use meiga_win::window::Window;

use crate::app::TITLE;
use crate::session::{Shared, lock};

/// The most characters of the text that the question before pasting shows,
/// and the most lines.
const PREVIEW_CHARS: usize = 512;
const PREVIEW_LINES: usize = 10;

/// Copies the text of the selection of the terminal of `shared` to the
/// clipboard, and removes the selection if `clear`. Returns whether there
/// was a selection.
pub fn copy(window: &Window, shared: &Mutex<Shared>, clear: bool) -> bool {
    let text = {
        let mut shared = lock(shared);
        if !shared.terminal.has_selection() {
            return false;
        }
        let text = shared.terminal.selection_text();
        if clear {
            shared.terminal.clear_selection();
        }
        text
    };
    if let Some(text) = text {
        // If another program keeps the clipboard open, the text is not
        // copied: as in Windows Terminal, nobody is told.
        let _ = clipboard::write_text(window, &text);
    }
    true
}

/// Pastes the text of the clipboard into the program of `input`, as the
/// modes of the terminal of `shared` say. If the text has more than one
/// line and the program did not ask for bracketed paste, it asks the user
/// first, in a message box. Returns whether it showed one.
pub fn paste(window: &Window, shared: &Mutex<Shared>, input: &Input) -> bool {
    // A clipboard that another program keeps open, or without text.
    let text = clipboard::read_text(window)
        .ok()
        .flatten()
        .unwrap_or_default();
    let modes = lock(shared).terminal.modes();
    let paste = Paste::new(&text, &modes);
    if paste.is_empty() {
        return false;
    }
    let asked = !paste.bracketed && paste.is_multiline();
    if asked && !window.confirm(TITLE, &question(&paste.text)) {
        return true;
    }
    let mut bytes = Vec::with_capacity(paste.text.len() + 12);
    paste.write_to(&mut bytes);
    input.write(&bytes);
    asked
}

/// The question before pasting `text`, which has more than one line, each
/// one ending with CR, except maybe the last one.
fn question(text: &str) -> String {
    let lines = text.split_terminator('\r').count();
    let mut preview = String::new();
    for (index, line) in text.split_terminator('\r').enumerate() {
        if index == PREVIEW_LINES || preview.chars().count() + line.chars().count() > PREVIEW_CHARS
        {
            preview.push('…');
            break;
        }
        preview.push_str(line);
        preview.push('\n');
    }
    format!(
        "You are about to paste {lines} lines, and the program may run each one as a command as \
         soon as it gets it.\n\n{}\n\nPaste them?",
        preview.trim_end_matches('\n')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_question_counts_the_lines_and_shows_the_first_ones() {
        assert_eq!(
            question("ls\rpwd\r"),
            "You are about to paste 2 lines, and the program may run each one as a command as \
             soon as it gets it.\n\nls\npwd\n\nPaste them?"
        );
        let many = "echo\r".repeat(12);
        let asked = question(&many);
        assert!(
            asked.starts_with("You are about to paste 12 lines,"),
            "{asked}"
        );
        assert!(
            asked.contains(&format!("{}…\n\nPaste them?", "echo\n".repeat(10))),
            "{asked}"
        );
        let long = format!("{}\r{}", "a".repeat(600), "b");
        let asked = question(&long);
        assert!(asked.contains("\n\n…\n\nPaste them?"), "{asked}");
    }
}
