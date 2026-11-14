//! The model of the windows and their tabs: the order of the tabs, the active
//! one and the focus, as pure logic without any interface, as decided in ADR
//! 0018 (`docs/adr/0018-o-modelo-das-tabs.md`).
//!
//! A [`Window`] keeps its tabs in the order of the tab bar. It always has at
//! least one, and exactly one of them is active. Each tab carries a content of
//! the type that the program chooses, such as its session and its view, so
//! that the order of the tabs lives in a single place.
//!
//! Three questions have separate answers, although for now they name the same
//! tab:
//!
//! - the active tab, the one that the tab bar marks: [`Window::active`];
//! - the focused tab, the one that receives the keyboard: [`Window::focused`];
//! - the visible tabs, the ones on the screen: [`Window::is_visible`].
//!
//! With the split view (F1), a window will show several tabs at once and the
//! focus will be on one of them: the model will keep the layout, and the
//! program will draw each visible tab in its own place. What the program
//! already asks by name keeps its meaning.

mod window;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

pub use window::{Close, Switch, Tab, Window};

/// Identifies a tab while meiga runs. Two tabs never get the same identifier,
/// not even in different windows, so a tab could keep its own if it moved to
/// another window (F6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TabId(u64);

impl TabId {
    fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(take(&NEXT))
    }

    /// The number of the identifier, for example to send it in a message to
    /// another thread.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// The identifier with this number, for example the one that came in a
    /// message from another thread. It only serves to look a tab up: the
    /// model gives each new tab a number from its counter, so it cannot make
    /// two tabs share one. A number that is not of a tab of the window finds
    /// nothing.
    pub const fn from_raw(number: u64) -> Self {
        Self(number)
    }
}

/// Identifies a window while meiga runs, as [`TabId`] identifies a tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(u64);

impl WindowId {
    fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(take(&NEXT))
    }

    /// The number of the identifier.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Takes the next number of a counter. Only that a number never comes out
/// twice matters, not its order with respect to other memory, so `Relaxed` is
/// enough. At a million tabs per second, 64 bits would last over 500 000
/// years.
fn take(counter: &AtomicU64) -> u64 {
    counter.fetch_add(1, Ordering::Relaxed)
}

/// Why a window refused a change. The window stays as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The window has no tab with this identifier: for example, it closed.
    NoSuchTab(TabId),
    /// The position, counted from 0, is past the last tab.
    OutOfRange { index: usize, count: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchTab(id) => write!(f, "the window has no tab {}", id.get()),
            Self::OutOfRange { index, count } => {
                let tabs = if *count == 1 { "tab" } else { "tabs" };
                write!(f, "position {index} is out of range for {count} {tabs}")
            }
        }
    }
}

impl std::error::Error for Error {}
