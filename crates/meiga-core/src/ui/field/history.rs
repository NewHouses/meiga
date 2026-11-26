//! The steps that undoing and redoing go through in a text field, as Chrome
//! keeps them (`TextfieldModel`): what each edit removed and inserted, and
//! the selection before it.
//!
//! An edit joins the one before it while nothing else happens between them:
//! the characters typed in a row are one step; so are the Backspaces in a
//! row, and the Deletes in a row. A paste, a cut, and deleting a selection
//! or a word are a step each. Moving the caret, selecting and undoing close
//! the last step: the next edit starts another.

use std::collections::VecDeque;

/// The most steps that can be undone: one more drops the oldest.
pub(super) const STEPS: usize = 100;

/// What an edit did, which says what can join it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// Typing, also over a selection: the next character typed where it
    /// ended joins it.
    Typing,
    /// Backspace without a selection: the next one, just before it, joins
    /// it.
    Backspace,
    /// Delete without a selection: the next one, at the same place, joins
    /// it.
    Delete,
    /// Anything else: nothing joins it.
    Other,
}

/// An edit: at `at`, the text had `removed`, and has `inserted`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Edit {
    pub at: usize,
    pub removed: String,
    pub inserted: String,
    /// The anchor and the caret of the selection before the edit, which
    /// undoing it brings back.
    pub before: (usize, usize),
    /// The caret after the edit, which redoing it brings back.
    pub after: usize,
    pub kind: Kind,
}

/// The edits that can be undone, the oldest first, and the ones that were
/// undone and can be redone, the last one undone last.
#[derive(Clone, Debug, Default)]
pub(super) struct History {
    done: VecDeque<Edit>,
    undone: Vec<Edit>,
    /// Whether the last edit done can take the next one.
    open: bool,
}

impl History {
    /// Keeps `edit`, already done, joined to the last one if it can. The
    /// edits undone cannot be redone any more.
    pub fn record(&mut self, edit: Edit) {
        self.undone.clear();
        let open = edit.kind != Kind::Other;
        if self.open
            && let Some(last) = self.done.back_mut()
            && join(last, &edit)
        {
            return;
        }
        if self.done.len() == STEPS {
            self.done.pop_front();
        }
        self.done.push_back(edit);
        self.open = open;
    }

    /// The next edit starts another step.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Takes the last edit done, to undo it: it can be redone later.
    pub fn undo(&mut self) -> Option<&Edit> {
        self.open = false;
        let edit = self.done.pop_back()?;
        self.undone.push(edit);
        self.undone.last()
    }

    /// Takes the last edit undone, to do it again: it can be undone again.
    pub fn redo(&mut self) -> Option<&Edit> {
        self.open = false;
        let edit = self.undone.pop()?;
        self.done.push_back(edit);
        self.done.back()
    }

    /// Forgets every edit.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Joins `next` to `last`, if they continue each other: typing where the
/// last typing ended, a Backspace just before the last one, or a Delete at
/// the same place. Returns whether it did.
fn join(last: &mut Edit, next: &Edit) -> bool {
    match (last.kind, next.kind) {
        (Kind::Typing, Kind::Typing)
            if next.removed.is_empty() && last.at + last.inserted.len() == next.at =>
        {
            last.inserted.push_str(&next.inserted);
            last.after = next.after;
        }
        (Kind::Backspace, Kind::Backspace) if next.at + next.removed.len() == last.at => {
            last.removed.insert_str(0, &next.removed);
            last.at = next.at;
            last.after = next.after;
        }
        (Kind::Delete, Kind::Delete) if next.at == last.at => {
            last.removed.push_str(&next.removed);
            last.after = next.after;
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(kind: Kind, at: usize, removed: &str, inserted: &str) -> Edit {
        Edit {
            at,
            removed: removed.to_owned(),
            inserted: inserted.to_owned(),
            before: (at, at),
            after: at + inserted.len(),
            kind,
        }
    }

    /// The steps that can be undone, the last one first, as (at, removed,
    /// inserted).
    fn steps(history: &mut History) -> Vec<(usize, String, String)> {
        let mut steps = Vec::new();
        while let Some(edit) = history.undo() {
            steps.push((edit.at, edit.removed.clone(), edit.inserted.clone()));
        }
        steps
    }

    fn step(at: usize, removed: &str, inserted: &str) -> (usize, String, String) {
        (at, removed.to_owned(), inserted.to_owned())
    }

    #[test]
    fn typing_in_a_row_is_one_step() {
        let mut history = History::default();
        history.record(edit(Kind::Typing, 0, "", "o"));
        history.record(edit(Kind::Typing, 1, "", "l"));
        history.record(edit(Kind::Typing, 2, "", "á"));
        assert_eq!(steps(&mut history), [step(0, "", "olá")]);
    }

    #[test]
    fn typing_over_a_selection_takes_the_next_characters() {
        let mut history = History::default();
        history.record(Edit {
            before: (0, 3),
            ..edit(Kind::Typing, 0, "abc", "x")
        });
        history.record(edit(Kind::Typing, 1, "", "y"));
        let undone = history.undo().cloned();
        assert_eq!(
            undone,
            Some(Edit {
                after: 2,
                before: (0, 3),
                ..edit(Kind::Typing, 0, "abc", "xy")
            })
        );
    }

    #[test]
    fn typing_somewhere_else_or_after_a_move_is_another_step() {
        let mut history = History::default();
        history.record(edit(Kind::Typing, 0, "", "a"));
        // Not where the last one ended.
        history.record(edit(Kind::Typing, 0, "", "b"));
        history.record(edit(Kind::Typing, 1, "", "c"));
        history.close();
        history.record(edit(Kind::Typing, 2, "", "d"));
        assert_eq!(
            steps(&mut history),
            [step(2, "", "d"), step(0, "", "bc"), step(0, "", "a")]
        );
    }

    #[test]
    fn backspaces_in_a_row_are_one_step_and_deletes_another() {
        let mut history = History::default();
        // "abcdef|": Backspace three times.
        history.record(edit(Kind::Backspace, 5, "f", ""));
        history.record(edit(Kind::Backspace, 4, "e", ""));
        history.record(edit(Kind::Backspace, 3, "d", ""));
        // "ab|c": Delete twice.
        history.record(edit(Kind::Delete, 2, "c", ""));
        // Not where the last one was.
        history.record(edit(Kind::Delete, 1, "b", ""));
        history.record(edit(Kind::Delete, 1, "x", ""));
        // A Backspace after the Deletes.
        history.record(edit(Kind::Backspace, 0, "a", ""));
        assert_eq!(
            steps(&mut history),
            [
                step(0, "a", ""),
                step(1, "bx", ""),
                step(2, "c", ""),
                step(3, "def", ""),
            ]
        );
    }

    #[test]
    fn a_backspace_that_does_not_end_where_the_last_one_started_is_another_step() {
        let mut history = History::default();
        history.record(edit(Kind::Backspace, 5, "f", ""));
        history.record(edit(Kind::Backspace, 2, "c", ""));
        assert_eq!(steps(&mut history), [step(2, "c", ""), step(5, "f", "")]);
    }

    #[test]
    fn other_edits_are_one_step_each_and_nothing_joins_them() {
        let mut history = History::default();
        history.record(edit(Kind::Typing, 0, "", "a"));
        history.record(edit(Kind::Other, 1, "", "pasted"));
        history.record(edit(Kind::Typing, 7, "", "b"));
        history.record(edit(Kind::Other, 0, "apastedb", ""));
        history.record(edit(Kind::Other, 0, "", "x"));
        assert_eq!(
            steps(&mut history),
            [
                step(0, "", "x"),
                step(0, "apastedb", ""),
                step(7, "", "b"),
                step(1, "", "pasted"),
                step(0, "", "a"),
            ]
        );
    }

    #[test]
    fn different_kinds_do_not_join() {
        for (first, second) in [
            (Kind::Typing, Kind::Backspace),
            (Kind::Backspace, Kind::Typing),
            (Kind::Delete, Kind::Typing),
            (Kind::Typing, Kind::Delete),
        ] {
            let mut history = History::default();
            history.record(edit(first, 1, "", ""));
            history.record(edit(second, 1, "", ""));
            assert_eq!(steps(&mut history).len(), 2, "{first:?} {second:?}");
        }
    }

    #[test]
    fn undone_steps_can_be_redone_until_another_edit() {
        let mut history = History::default();
        history.record(edit(Kind::Other, 0, "", "a"));
        history.record(edit(Kind::Other, 1, "", "b"));
        assert_eq!(
            history.undo().map(|edit| edit.inserted.clone()),
            Some("b".to_owned())
        );
        assert_eq!(
            history.undo().map(|edit| edit.inserted.clone()),
            Some("a".to_owned())
        );
        assert_eq!(history.undo(), None);
        assert_eq!(
            history.redo().map(|edit| edit.inserted.clone()),
            Some("a".to_owned())
        );
        // Another edit: "b" cannot be redone.
        history.record(edit(Kind::Other, 1, "", "c"));
        assert_eq!(history.redo(), None);
        assert_eq!(steps(&mut history), [step(1, "", "c"), step(0, "", "a")]);
    }

    #[test]
    fn undoing_and_redoing_close_the_last_step() {
        let mut history = History::default();
        history.record(edit(Kind::Typing, 0, "", "a"));
        history.record(edit(Kind::Typing, 1, "", "b"));
        history.undo();
        history.redo();
        history.record(edit(Kind::Typing, 2, "", "c"));
        assert_eq!(steps(&mut history), [step(2, "", "c"), step(0, "", "ab")]);
        let mut history = History::default();
        history.record(edit(Kind::Typing, 0, "", "a"));
        history.redo();
        history.record(edit(Kind::Typing, 1, "", "b"));
        assert_eq!(steps(&mut history), [step(1, "", "b"), step(0, "", "a")]);
    }

    #[test]
    fn only_the_last_100_steps_can_be_undone() {
        let mut history = History::default();
        for at in 0..=STEPS {
            history.record(edit(Kind::Other, at, "", "x"));
        }
        let steps = steps(&mut history);
        assert_eq!(steps.len(), STEPS);
        // The first one went away.
        assert_eq!(steps.last(), Some(&step(1, "", "x")));
    }

    #[test]
    fn clearing_forgets_everything() {
        let mut history = History::default();
        history.record(edit(Kind::Typing, 0, "", "a"));
        history.record(edit(Kind::Typing, 1, "", "b"));
        history.undo();
        history.clear();
        assert_eq!(history.redo(), None);
        assert_eq!(history.undo(), None);
        // And the next typing does not join anything.
        history.record(edit(Kind::Typing, 1, "", "c"));
        assert_eq!(steps(&mut history), [step(1, "", "c")]);
    }
}
