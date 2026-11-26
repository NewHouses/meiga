//! What the keys do in a text field, as in the text fields of Chrome on
//! Windows (ADR 0030).
//!
//! | Keys | What they do |
//! |---|---|
//! | ← → | move the caret a cluster; without Shift, a selection goes away from its edge on that side |
//! | Ctrl+← Ctrl+→ | move it to the start of the previous or the next word |
//! | Home End | move it to the start or the end of the text |
//! | Shift and the keys above, Shift+↑ Shift+↓ | select up to there |
//! | Backspace Delete | delete the selection, or the last code point before the caret, or the cluster after it |
//! | Ctrl+Backspace Ctrl+Delete | delete the selection, or up to the start of the previous or the next word |
//! | Ctrl+A | select all |
//! | Ctrl+C Ctrl+Insert | copy the selection |
//! | Ctrl+X Shift+Delete | cut the selection |
//! | Ctrl+V Shift+Insert | paste |
//! | Ctrl+Z Alt+Backspace | undo |
//! | Ctrl+Y Ctrl+Shift+Z Alt+Shift+Backspace | redo |
//!
//! The other keys write their characters, also with AltGr, or with Ctrl+Alt,
//! which Windows takes as AltGr; with Alt alone, they write nothing, but Alt
//! writes the character of the code typed with it on the numeric keypad,
//! when it goes up. Control characters never go in. A key that the keyboard
//! repeats faster than the window takes it comes once with the count of its
//! repeats, and does its work that many times. Enter, Esc, Tab, ↑, ↓ and
//! the keys without a character do nothing: they are for the user of the
//! field.

use std::ops::Range;

use super::history::Kind;
use super::segment::{next_cluster, next_word, previous_cluster, previous_word};
use super::{Field, Reply};
use crate::input::{KeyEvent, own_character};

/// The virtual-key codes of the keys of a field.
const VK_BACK: u16 = 0x08;
const VK_MENU: u16 = 0x12;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_INSERT: u16 = 0x2d;
const VK_DELETE: u16 = 0x2e;
const VK_A: u16 = 0x41;
const VK_C: u16 = 0x43;
const VK_V: u16 = 0x56;
const VK_X: u16 = 0x58;
const VK_Y: u16 = 0x59;
const VK_Z: u16 = 0x5a;

impl Field {
    /// A key that went down or up while the field has the keyboard.
    pub fn key(&mut self, event: &KeyEvent) -> Reply {
        if !event.down {
            // Of the keys that go up, only Alt writes a character: the one
            // of the code typed with it on the numeric keypad.
            if event.key == VK_MENU && event.character != 0 {
                return self.typed(event.character, 1);
            }
            return Reply::Ignored;
        }
        // A character that is not of its key goes in as a character: the
        // one of a dead key that does not join with Backspace, Enter, Tab or
        // Esc, which Windows writes with them. The key comes next, with its
        // own character.
        if event.character != 0 && !own_character(event.key, event.character) {
            return self.typed(event.character, event.repeat);
        }
        let controls = event.controls;
        // AltGr, which is Ctrl+Alt for Windows, writes characters, and has
        // no commands.
        if !(controls.ctrl() && controls.alt())
            && let Some(reply) = self.command(event)
        {
            self.high_surrogate = None;
            return reply;
        }
        // The characters with Alt alone are for the menus of the window;
        // with Ctrl too, Windows takes Alt as AltGr.
        if event.character == 0 || (controls.alt() && !controls.ctrl()) {
            return Reply::Ignored;
        }
        self.typed(event.character, event.repeat)
    }

    /// What `event`, a key that goes down without Ctrl+Alt, does, if it is a
    /// command.
    fn command(&mut self, event: &KeyEvent) -> Option<Reply> {
        let controls = event.controls;
        let (ctrl, alt, shift) = (controls.ctrl(), controls.alt(), controls.shift());
        let times = event.repeat;
        let reply = match event.key {
            VK_LEFT | VK_RIGHT if !alt => {
                let right = event.key == VK_RIGHT;
                self.repeat(times, |field| field.step(right, ctrl, shift))
            }
            VK_HOME | VK_END if !alt => {
                self.go_to_end(event.key == VK_END, shift);
                Reply::Taken
            }
            VK_UP | VK_DOWN if shift && !alt => {
                self.go_to_end(event.key == VK_DOWN, true);
                Reply::Taken
            }
            VK_BACK if alt && !ctrl => self.repeat(times, |field| field.undo_or_redo(shift)),
            VK_BACK if !alt => self.repeat(times, |field| {
                if ctrl {
                    field.delete_word(false)
                } else {
                    field.backspace()
                }
            }),
            VK_DELETE if !alt && ctrl => self.repeat(times, |field| field.delete_word(true)),
            VK_DELETE if !alt && shift => self.cut(),
            VK_DELETE if !alt => self.repeat(times, Self::delete_forward),
            VK_INSERT if !alt && ctrl && !shift => self.copy(),
            VK_INSERT if !alt && shift && !ctrl => Reply::Paste,
            VK_A if ctrl && !alt => {
                self.select_all();
                Reply::Taken
            }
            VK_C if ctrl && !alt => self.copy(),
            VK_X if ctrl && !alt => self.cut(),
            VK_V if ctrl && !alt => Reply::Paste,
            VK_Z if ctrl && !alt => self.repeat(times, |field| field.undo_or_redo(shift)),
            VK_Y if ctrl && !alt => self.repeat(times, |field| field.undo_or_redo(true)),
            _ => return None,
        };
        Some(reply)
    }

    /// Does `action` `times` times, or once with no count, while it changes
    /// something.
    fn repeat(&mut self, times: u16, mut action: impl FnMut(&mut Self) -> bool) -> Reply {
        for _ in 0..times.max(1) {
            if !action(self) {
                break;
            }
        }
        Reply::Taken
    }

    /// A UTF-16 code unit of a character typed `times` times. The first
    /// half of a character outside the Basic Multilingual Plane waits for
    /// the second one, and a half alone goes away. A control character is
    /// for the user of the field.
    fn typed(&mut self, unit: u16, times: u16) -> Reply {
        let character = match unit {
            0xd800..=0xdbff => {
                self.high_surrogate = Some(unit);
                return Reply::Taken;
            }
            0xdc00..=0xdfff => match self.high_surrogate.take() {
                Some(high) => char::decode_utf16([high, unit]).next().and_then(Result::ok),
                None => return Reply::Taken,
            },
            _ => {
                self.high_surrogate = None;
                char::from_u32(u32::from(unit))
            }
        };
        match character {
            Some(character) if !character.is_control() => {
                self.repeat(times, |field| field.type_character(character))
            }
            _ => Reply::Ignored,
        }
    }

    /// Types `character` in place of the selection, if it fits. Returns
    /// whether it did.
    fn type_character(&mut self, character: char) -> bool {
        if character.len_utf16() > self.room() {
            return false;
        }
        let mut buffer = [0; 4];
        let text = &*character.encode_utf8(&mut buffer);
        self.track(|field| field.replace(field.selection(), text, Kind::Typing));
        true
    }

    /// ← or →, by words with Ctrl, selecting with Shift. Returns whether
    /// the caret or the selection changed.
    fn step(&mut self, right: bool, word: bool, extend: bool) -> bool {
        let selection = self.selection();
        // Without Shift, a selection goes away, from its edge on that side.
        let from = match (extend || selection.is_empty(), right) {
            (true, _) => self.caret,
            (false, true) => selection.end,
            (false, false) => selection.start,
        };
        let text = &self.text;
        let at = match (word, right) {
            (true, true) => next_word(text, from),
            (true, false) => previous_word(text, from),
            _ if !extend && !selection.is_empty() => from,
            (false, true) => next_cluster(text, from),
            (false, false) => previous_cluster(text, from),
        };
        self.move_caret(at, extend)
    }

    /// Home or End, selecting with Shift.
    fn go_to_end(&mut self, end: bool, extend: bool) {
        let at = if end { self.text.len() } else { 0 };
        self.move_caret(at, extend);
    }

    /// Moves the caret to `at`, and the anchor with it unless `extend`.
    /// Returns whether the caret or the selection changed.
    fn move_caret(&mut self, at: usize, extend: bool) -> bool {
        let anchor = if extend { self.anchor } else { at };
        self.select(anchor, at)
    }

    /// Backspace. Returns whether it deleted something.
    fn backspace(&mut self) -> bool {
        let selection = self.selection();
        if !selection.is_empty() {
            return self.delete(selection, Kind::Other);
        }
        match self.text[..self.caret].chars().next_back() {
            Some(last) => self.delete(self.caret - last.len_utf8()..self.caret, Kind::Backspace),
            None => false,
        }
    }

    /// Delete. Returns whether it deleted something.
    fn delete_forward(&mut self) -> bool {
        let selection = self.selection();
        if !selection.is_empty() {
            return self.delete(selection, Kind::Other);
        }
        let end = next_cluster(&self.text, self.caret);
        self.delete(self.caret..end, Kind::Delete)
    }

    /// Ctrl+Backspace, or Ctrl+Delete if `forward`. Returns whether it
    /// deleted something.
    fn delete_word(&mut self, forward: bool) -> bool {
        let selection = self.selection();
        let range = if !selection.is_empty() {
            selection
        } else if forward {
            self.caret..next_word(&self.text, self.caret)
        } else {
            previous_word(&self.text, self.caret)..self.caret
        };
        self.delete(range, Kind::Other)
    }

    /// Deletes `range` of the text, as `kind`. Returns whether it was not
    /// empty.
    fn delete(&mut self, range: Range<usize>, kind: Kind) -> bool {
        if range.is_empty() {
            return false;
        }
        self.track(|field| field.replace(range, "", kind));
        true
    }

    /// Ctrl+C: the selection goes to the clipboard.
    fn copy(&self) -> Reply {
        match self.selected_text() {
            "" => Reply::Taken,
            selected => Reply::Copy(selected.to_owned()),
        }
    }

    /// Ctrl+X: the selection goes to the clipboard, and away from the
    /// field.
    fn cut(&mut self) -> Reply {
        let reply = self.copy();
        self.delete(self.selection(), Kind::Other);
        reply
    }

    /// Undoes the last step, or redoes the last one undone if `redo`.
    /// Returns whether there was one.
    fn undo_or_redo(&mut self, redo: bool) -> bool {
        if redo {
            self.track(Self::redo)
        } else {
            self.track(Self::undo)
        }
    }
}
