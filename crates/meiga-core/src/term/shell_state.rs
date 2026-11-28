//! What the shell of a tab says about itself (ADR 0007 and ADR 0032): its
//! folder, and where it is between its prompt and the output of a command,
//! with the exit code of the last one. The scanner finds it in the output
//! ([`super::scanner`]), and the terminal keeps it here, until a new program
//! starts in the tab.

use std::mem;

use super::scanner::Report;

/// The folder of the shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Folder {
    /// A path of Windows, from OSC 9;9, as PowerShell and cmd give it.
    Windows(String),
    /// A URL with the computer and the path, from OSC 7, as the shells of
    /// Linux give it: in WSL, the path is one of Linux. The computer can be
    /// another one, as in a session of SSH.
    Url { host: String, path: String },
}

/// The marks of OSC 133, which the shell writes around its prompt and the
/// commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// A: the prompt starts.
    Prompt,
    /// B: the prompt ends, and the command that is typed starts.
    Command,
    /// C: the command runs, and its output starts.
    Output,
    /// D: the command ended, with its exit code, if the shell gave one.
    Done(Option<i32>),
}

/// What the shell said last.
#[derive(Clone, Debug, Default)]
pub struct ShellState {
    folder: Option<Folder>,
    mark: Option<Mark>,
    exit_code: Option<i32>,
    /// Something changed since the terminal last took its changes.
    changed: bool,
}

impl ShellState {
    /// The last folder that the shell gave.
    pub const fn folder(&self) -> Option<&Folder> {
        self.folder.as_ref()
    }

    /// The last mark that the shell wrote.
    pub const fn mark(&self) -> Option<Mark> {
        self.mark
    }

    /// The exit code of the last command that ended (D), if the shell gave
    /// it. It stays while the next prompt is shown and a new command runs.
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Takes what the shell said. Saying the same again is not a change.
    pub(super) fn apply(&mut self, report: Report) {
        match report {
            Report::Folder(folder) => {
                if self.folder.as_ref() != Some(&folder) {
                    self.folder = Some(folder);
                    self.changed = true;
                }
            }
            Report::Mark(mark) => {
                if self.mark != Some(mark) {
                    self.mark = Some(mark);
                    self.changed = true;
                }
                if let Mark::Done(code) = mark
                    && self.exit_code != code
                {
                    self.exit_code = code;
                    self.changed = true;
                }
            }
        }
    }

    /// Whether something changed since the last time that this was asked.
    pub(super) fn take_changed(&mut self) -> bool {
        mem::take(&mut self.changed)
    }
}
