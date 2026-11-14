//! A window and its tabs.

use std::cmp::Ordering;

use super::{Error, TabId, WindowId};

/// A tab of a window: its identifier and the content that the program keeps
/// for it.
#[derive(Debug)]
pub struct Tab<T> {
    id: TabId,
    content: T,
}

impl<T> Tab<T> {
    fn new(content: T) -> Self {
        Self {
            id: TabId::next(),
            content,
        }
    }

    /// A tab whose content is made knowing its identifier.
    fn try_with<E>(content: impl FnOnce(TabId) -> Result<T, E>) -> Result<Self, E> {
        let id = TabId::next();
        Ok(Self {
            id,
            content: content(id)?,
        })
    }

    pub const fn id(&self) -> TabId {
        self.id
    }

    pub const fn content(&self) -> &T {
        &self.content
    }
}

/// The active tab changed: `from` stopped being active, and `to` is active
/// now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Switch {
    pub from: TabId,
    pub to: TabId,
}

/// What closing a tab did. It has to be looked at: it may bring the content
/// of the tab, such as its session, and the tab that is active now.
#[derive(Debug, PartialEq, Eq)]
#[must_use = "the program has to close the content of the tab and show the tab that is active now"]
pub enum Close<T> {
    /// The tab left the window, and this is its content. If it was the active
    /// tab, `activated` is the tab that took its place.
    Removed {
        content: T,
        activated: Option<TabId>,
    },
    /// It is the only tab of the window: it stays, and the window has to close
    /// with it.
    LastTab,
}

/// A window: its tabs, in the order of the tab bar, of which one is active.
///
/// It always has at least one tab, and the active tab is always one of them.
/// No method can break this: they refuse what would break it with an
/// [`Error`], which leaves the window as it was.
#[derive(Debug)]
pub struct Window<T> {
    id: WindowId,
    /// Never empty.
    tabs: Vec<Tab<T>>,
    /// The position of the active tab in `tabs`.
    active: usize,
}

impl<T> Window<T> {
    /// A window with a single tab, which is the active one.
    pub fn new(content: T) -> Self {
        Self::with_tab(Tab::new(content))
    }

    /// A window with a single tab, whose content is made knowing the
    /// identifier of the tab: from subphase 2.2, its session, so that its
    /// reader thread sends the identifier with its wake-ups. If making the
    /// content fails, there is no window, and the error comes back.
    pub fn try_new_with<E>(content: impl FnOnce(TabId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self::with_tab(Tab::try_with(content)?))
    }

    fn with_tab(tab: Tab<T>) -> Self {
        Self {
            id: WindowId::next(),
            tabs: vec![tab],
            active: 0,
        }
    }

    pub const fn id(&self) -> WindowId {
        self.id
    }

    /// The tabs, in the order of the tab bar.
    pub fn tabs(&self) -> &[Tab<T>] {
        &self.tabs
    }

    /// How many tabs the window has: at least one.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// The position of a tab in the tab bar, counted from 0.
    pub fn index_of(&self, id: TabId) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.id == id)
    }

    /// The content of a tab.
    pub fn get(&self, id: TabId) -> Option<&T> {
        self.tabs
            .iter()
            .find(|tab| tab.id == id)
            .map(|tab| &tab.content)
    }

    /// The content of a tab, to change it.
    pub fn get_mut(&mut self, id: TabId) -> Option<&mut T> {
        self.tabs
            .iter_mut()
            .find(|tab| tab.id == id)
            .map(|tab| &mut tab.content)
    }

    /// The contents of all the tabs, with their identifiers, in the order
    /// of the tab bar, to change them: for example, from subphase 2.2, the
    /// ones of the tabs that are not shown. The tabs themselves do not
    /// change.
    pub fn contents_mut(&mut self) -> impl Iterator<Item = (TabId, &mut T)> {
        self.tabs.iter_mut().map(|tab| (tab.id, &mut tab.content))
    }

    /// The active tab: the one that the tab bar marks.
    pub fn active(&self) -> &Tab<T> {
        &self.tabs[self.active]
    }

    /// The position of the active tab, counted from 0.
    pub const fn active_index(&self) -> usize {
        self.active
    }

    /// The focused tab: the one that receives the keyboard. For now it is
    /// always the active tab; with the split view (F1), it will be one of the
    /// visible tabs.
    pub fn focused(&self) -> &Tab<T> {
        self.active()
    }

    /// The content of the focused tab, to give it the keyboard.
    pub fn focused_mut(&mut self) -> &mut T {
        &mut self.tabs[self.active].content
    }

    /// Whether a tab is on the screen. For now, only the active tab is: the
    /// others only process the output of their programs (ADR 0005).
    pub fn is_visible(&self, id: TabId) -> bool {
        self.active().id == id
    }

    /// Opens a tab at the end of the tab bar, and answers its identifier. It
    /// does not activate it: the button + will activate it, but a restored
    /// session opens its tabs without activating them (subphase 7.2).
    pub fn open(&mut self, content: T) -> TabId {
        self.push(Tab::new(content))
    }

    /// Opens a tab at the end of the tab bar, as [`open`](Self::open), with
    /// a content made knowing the identifier of the tab, as
    /// [`try_new_with`](Self::try_new_with). If making the content fails, the
    /// window does not change, and the error comes back.
    pub fn try_open_with<E>(
        &mut self,
        content: impl FnOnce(TabId) -> Result<T, E>,
    ) -> Result<TabId, E> {
        Ok(self.push(Tab::try_with(content)?))
    }

    fn push(&mut self, tab: Tab<T>) -> TabId {
        let id = tab.id;
        self.tabs.push(tab);
        id
    }

    /// Closes a tab, and answers its content. If it was the active tab, the
    /// one on its right takes its place or, if it was the last one, the one on
    /// its left, as in Chrome.
    ///
    /// The window never stays without tabs: its only tab stays where it is,
    /// and the answer is [`Close::LastTab`], so that the window closes.
    pub fn close(&mut self, id: TabId) -> Result<Close<T>, Error> {
        let index = self.find(id)?;
        if self.tabs.len() == 1 {
            return Ok(Close::LastTab);
        }
        let tab = self.tabs.remove(index);
        let activated = match index.cmp(&self.active) {
            Ordering::Less => {
                self.active -= 1;
                None
            }
            Ordering::Greater => None,
            Ordering::Equal => {
                // The tab on its right moved into its position; without one,
                // the tab on its left is the last one now.
                self.active = index.min(self.tabs.len() - 1);
                Some(self.tabs[self.active].id)
            }
        };
        Ok(Close::Removed {
            content: tab.content,
            activated,
        })
    }

    /// Moves a tab to the position `to` of the tab bar, counted from 0. The
    /// tabs between its old and its new position move one place to fill the
    /// gap, and the active tab stays the same tab.
    pub fn move_tab(&mut self, id: TabId, to: usize) -> Result<(), Error> {
        let from = self.find(id)?;
        let count = self.tabs.len();
        if to >= count {
            return Err(Error::OutOfRange { index: to, count });
        }
        if from < to {
            self.tabs[from..=to].rotate_left(1);
        } else {
            self.tabs[to..=from].rotate_right(1);
        }
        self.active = moved(self.active, from, to);
        Ok(())
    }

    /// Activates a tab, and answers the change, or `None` if it was already
    /// the active tab.
    pub fn activate(&mut self, id: TabId) -> Result<Option<Switch>, Error> {
        let index = self.find(id)?;
        Ok(self.activate_at(index))
    }

    /// Activates the tab on the right of the active one or, from the last
    /// one, the first, as Ctrl+Tab does in Chrome. With a single tab, nothing
    /// changes.
    #[must_use = "the program has to show the tab that is active now"]
    pub fn activate_next(&mut self) -> Option<Switch> {
        self.activate_at((self.active + 1) % self.tabs.len())
    }

    /// Activates the tab on the left of the active one or, from the first
    /// one, the last.
    #[must_use = "the program has to show the tab that is active now"]
    pub fn activate_previous(&mut self) -> Option<Switch> {
        let count = self.tabs.len();
        self.activate_at((self.active + count - 1) % count)
    }

    fn activate_at(&mut self, index: usize) -> Option<Switch> {
        if index == self.active {
            return None;
        }
        let from = self.tabs[self.active].id;
        self.active = index;
        Some(Switch {
            from,
            to: self.tabs[index].id,
        })
    }

    fn find(&self, id: TabId) -> Result<usize, Error> {
        self.index_of(id).ok_or(Error::NoSuchTab(id))
    }
}

/// Where the tab at `index` ends when the tab at `from` moves to `to`.
const fn moved(index: usize, from: usize, to: usize) -> usize {
    if index == from {
        to
    } else if from < index && index <= to {
        index - 1
    } else if to <= index && index < from {
        index + 1
    } else {
        index
    }
}
