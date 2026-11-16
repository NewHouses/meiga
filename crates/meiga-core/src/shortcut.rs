//! The keyboard shortcuts (ADR 0023): the keys that meiga takes for itself,
//! and what each one does.
//!
//! A shortcut is a chord, some modifier keys and a key, and an action. meiga
//! has its own, as Windows Terminal and Chrome, and the table `[shortcuts]`
//! of `config.toml` adds, changes or frees them:
//!
//! ```toml
//! [shortcuts]
//! "ctrl+t" = "new_tab"            # another shortcut for an action
//! "ctrl+6" = "none"               # Ctrl+6 reaches the program again
//! "ctrl+shift+u" = "profile_4"
//! ```
//!
//! A chord is written as in Windows Terminal: the modifier keys `ctrl`, `alt`
//! and `shift`, in any order, and a key, joined with `+`, without telling
//! capital letters from small ones. A key is a letter, a digit, `f1` to
//! `f24`, or one of the names of [`NAMES`]. Keys go by their virtual-key
//! code, which says what the key is in the layout of the keyboard, and not by
//! the character that it writes: `ctrl+shift+2` is the same chord with any
//! layout, though the 2 with Shift writes `@` in some and `"` in others.
//!
//! AltGr, which Windows gives as the left Ctrl and the right Alt, writes
//! characters: no shortcut has it, as in Windows Terminal.

use std::fmt;

use crate::input::KeyEvent;

/// The virtual-key codes that the shortcuts of meiga use.
const VK_TAB: u16 = 0x09;
const VK_SPACE: u16 = 0x20;
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_UP: u16 = 0x26;
const VK_DOWN: u16 = 0x28;
const VK_APPS: u16 = 0x5d;
const VK_NUMPAD0: u16 = 0x60;
const VK_ADD: u16 = 0x6b;
const VK_SUBTRACT: u16 = 0x6d;
const VK_F1: u16 = 0x70;
const VK_F4: u16 = 0x73;
const VK_F10: u16 = 0x79;
const VK_OEM_PLUS: u16 = 0xbb;
const VK_OEM_MINUS: u16 = 0xbd;

/// The names of the keys that are not a letter, a digit nor a function key,
/// with their virtual-key codes: the ones of Windows Terminal, without the
/// keys of the browser. `plus`, `comma`, `minus` and `period` are the keys
/// that write those characters in any layout; the keys of the numeric
/// keypad also have names, from `numpad0` (or `numpad_0`) to `numpad9`.
pub const NAMES: [(&str, u16); 32] = [
    ("enter", 0x0d),
    ("tab", VK_TAB),
    ("space", VK_SPACE),
    ("backspace", 0x08),
    ("menu", VK_APPS),
    ("app", VK_APPS),
    ("insert", 0x2d),
    ("delete", 0x2e),
    ("home", VK_HOME),
    ("end", VK_END),
    ("pgdn", VK_NEXT),
    ("pagedown", VK_NEXT),
    ("pgup", VK_PRIOR),
    ("pageup", VK_PRIOR),
    ("esc", 0x1b),
    ("escape", 0x1b),
    ("left", 0x25),
    ("right", 0x27),
    ("up", VK_UP),
    ("down", VK_DOWN),
    ("numpad_plus", VK_ADD),
    ("numpad_add", VK_ADD),
    ("numpad_minus", VK_SUBTRACT),
    ("numpad_subtract", VK_SUBTRACT),
    ("numpad_multiply", 0x6a),
    ("numpad_divide", 0x6f),
    ("numpad_period", 0x6e),
    ("numpad_decimal", 0x6e),
    ("plus", VK_OEM_PLUS),
    ("comma", 0xbc),
    ("minus", VK_OEM_MINUS),
    ("period", 0xbe),
];

/// A chord: the modifier keys that are down, and the key that goes down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    /// [`Chord::CTRL`], [`Chord::ALT`] and [`Chord::SHIFT`], or none.
    modifiers: u8,
    /// The virtual-key code of the key.
    key: u16,
}

impl Chord {
    pub const CTRL: u8 = 1;
    pub const ALT: u8 = 2;
    pub const SHIFT: u8 = 4;

    pub const fn new(modifiers: u8, key: u16) -> Self {
        Self {
            modifiers: modifiers & (Self::CTRL | Self::ALT | Self::SHIFT),
            key,
        }
    }

    /// The chord of a key that goes down, or `None` if no shortcut can have
    /// it: a character without a key, from an IME or the emoji panel, or a
    /// key with AltGr.
    pub const fn of(event: &KeyEvent) -> Option<Self> {
        let controls = event.controls;
        if event.key == 0 || controls.alt_gr() {
            return None;
        }
        let mut modifiers = 0;
        if controls.ctrl() {
            modifiers |= Self::CTRL;
        }
        if controls.alt() {
            modifiers |= Self::ALT;
        }
        if controls.shift() {
            modifiers |= Self::SHIFT;
        }
        Some(Self::new(modifiers, event.key))
    }

    /// Reads a chord written as in Windows Terminal, such as `ctrl+shift+t`.
    /// The blanks around each part do not count.
    pub fn parse(text: &str) -> Result<Self, ChordError> {
        let mut modifiers = 0;
        let mut key = None;
        for part in text.split('+') {
            let part = part.trim();
            if part.is_empty() {
                return Err(ChordError::Empty);
            }
            if part.eq_ignore_ascii_case("ctrl") {
                modifiers |= Self::CTRL;
            } else if part.eq_ignore_ascii_case("alt") {
                modifiers |= Self::ALT;
            } else if part.eq_ignore_ascii_case("shift") {
                modifiers |= Self::SHIFT;
            } else if part.eq_ignore_ascii_case("win") {
                return Err(ChordError::Windows);
            } else {
                let code = key_code(part).ok_or_else(|| ChordError::Unknown(part.to_owned()))?;
                if key.replace(code).is_some() {
                    return Err(ChordError::TwoKeys);
                }
            }
        }
        let key = key.ok_or(ChordError::NoKey)?;
        // Windows closes the window with Alt+F4, and opens its menu with
        // Alt+Space: meiga never gets them (ADR 0014).
        if modifiers & (Self::ALT | Self::CTRL) == Self::ALT && matches!(key, VK_F4 | VK_SPACE) {
            return Err(ChordError::ForWindows);
        }
        Ok(Self::new(modifiers, key))
    }
}

/// Why some text is not a chord.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChordError {
    /// There is nothing between two `+`, or before or after one.
    Empty,
    /// It has only modifier keys.
    NoKey,
    /// It has more than one key, as `ctrl+a+b`.
    TwoKeys,
    /// This part is not a modifier key nor the name of a key.
    Unknown(String),
    /// It has a Windows key.
    Windows,
    /// It is Alt+F4 or Alt+Space, with Shift or without it.
    ForWindows,
}

impl fmt::Display for ChordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("a part is empty (the key + is `plus`)"),
            Self::NoKey => f.write_str("it has no key, only modifier keys"),
            Self::TwoKeys => f.write_str("it has more than one key"),
            Self::Unknown(part) => write!(f, "`{part}` is not a key"),
            Self::Windows => f.write_str("the Windows keys are for Windows"),
            Self::ForWindows => f.write_str("Alt+F4 and Alt+Space are for Windows"),
        }
    }
}

/// The virtual-key code of the key named `name`, without telling capital
/// letters from small ones.
fn key_code(name: &str) -> Option<u16> {
    let bytes = name.as_bytes();
    if let [byte] = bytes {
        return match byte.to_ascii_uppercase() {
            byte @ (b'0'..=b'9' | b'A'..=b'Z') => Some(u16::from(byte)),
            _ => None,
        };
    }
    if let Some(number) = strip_prefix_ignore_case(name, "f").and_then(number) {
        return match number {
            1..=24 => Some(VK_F1 + number - 1),
            _ => None,
        };
    }
    if let Some(digit) = strip_prefix_ignore_case(name, "numpad")
        .map(|rest| rest.strip_prefix('_').unwrap_or(rest))
        .and_then(number)
    {
        return match digit {
            0..=9 => Some(VK_NUMPAD0 + digit),
            _ => None,
        };
    }
    NAMES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
        .map(|&(_, code)| code)
}

/// `text` without `prefix` at its start, if it starts with it, telling no
/// capital letters from small ones.
fn strip_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let start = text.get(..prefix.len())?;
    start
        .eq_ignore_ascii_case(prefix)
        .then(|| &text[prefix.len()..])
}

/// The number that `digits` writes, if they are only digits, and not too
/// many.
fn number(digits: &str) -> Option<u16> {
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// What a shortcut does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Open a tab with the default profile, as the button +.
    NewTab,
    /// Close the tab that is shown.
    CloseTab,
    /// Show the next tab, or the first after the last one.
    NextTab,
    /// Show the previous tab, or the last before the first one.
    PreviousTab,
    /// Show the tab at this place of the bar, from 0: `tab_1` is `Tab(0)`.
    /// If there is no tab there, nothing happens.
    Tab(u16),
    /// Show the last tab.
    LastTab,
    /// Open a tab with the profile at this place of the menu of the
    /// profiles, from 0: `profile_1` is `Profile(0)`.
    Profile(u16),
    /// Open the menu of the profiles, as the button next to +.
    ProfileMenu,
    /// Open the menu of the tab that is shown, below its tab.
    TabMenu,
    /// Duplicate the tab that is shown, as its menu.
    DuplicateTab,
    /// Close all the tabs but the one that is shown, as its menu.
    CloseOtherTabs,
    /// Close the tabs on the right of the one that is shown, as its menu.
    CloseTabsToTheRight,
    /// Copy the selection, if there is one: without it, the key goes to the
    /// program.
    Copy,
    /// Paste the clipboard.
    Paste,
    /// Move the view of the history a row up, or down, a page up, or down,
    /// or to its start, or to its end. On the alternate screen, which has no
    /// history, the key goes to the program.
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollToTop,
    ScrollToBottom,
    /// Make the font a point bigger or smaller, or bring it back to its
    /// size.
    ZoomIn,
    ZoomOut,
    ZoomReset,
}

/// The names of the actions without a number, as `config.toml` writes them.
const ACTIONS: [(&str, Action); 21] = [
    ("new_tab", Action::NewTab),
    ("close_tab", Action::CloseTab),
    ("next_tab", Action::NextTab),
    ("previous_tab", Action::PreviousTab),
    ("last_tab", Action::LastTab),
    ("profile_menu", Action::ProfileMenu),
    ("tab_menu", Action::TabMenu),
    ("duplicate_tab", Action::DuplicateTab),
    ("close_other_tabs", Action::CloseOtherTabs),
    ("close_tabs_to_the_right", Action::CloseTabsToTheRight),
    ("copy", Action::Copy),
    ("paste", Action::Paste),
    ("scroll_up", Action::ScrollUp),
    ("scroll_down", Action::ScrollDown),
    ("scroll_page_up", Action::ScrollPageUp),
    ("scroll_page_down", Action::ScrollPageDown),
    ("scroll_to_top", Action::ScrollToTop),
    ("scroll_to_bottom", Action::ScrollToBottom),
    ("zoom_in", Action::ZoomIn),
    ("zoom_out", Action::ZoomOut),
    ("zoom_reset", Action::ZoomReset),
];

/// The name of the action that frees a chord: the key goes to the program.
pub const NONE: &str = "none";

impl Action {
    /// Reads the name of an action, as `config.toml` writes it: `none`, which
    /// frees the chord, is `Ok(None)`.
    pub fn parse(name: &str) -> Result<Option<Self>, ActionError> {
        if name == NONE {
            return Ok(None);
        }
        if let Some((_, action)) = ACTIONS.iter().find(|(known, _)| *known == name) {
            return Ok(Some(*action));
        }
        let numbered = |prefix: &str, action: fn(u16) -> Self| {
            let digits = name.strip_prefix(prefix)?;
            Some(match number(digits) {
                Some(0) => Err(ActionError::FromOne),
                Some(place) => Ok(Some(action(place - 1))),
                None => Err(ActionError::Unknown),
            })
        };
        numbered("tab_", Self::Tab)
            .or_else(|| numbered("profile_", Self::Profile))
            .unwrap_or(Err(ActionError::Unknown))
    }

    /// Whether it goes on while its key is held down: the next and the
    /// previous tab, the history and the zoom. Opening or closing tabs does
    /// not, nor pasting.
    pub const fn repeats(self) -> bool {
        matches!(
            self,
            Self::NextTab
                | Self::PreviousTab
                | Self::ScrollUp
                | Self::ScrollDown
                | Self::ScrollPageUp
                | Self::ScrollPageDown
                | Self::ScrollToTop
                | Self::ScrollToBottom
                | Self::ZoomIn
                | Self::ZoomOut
        )
    }
}

/// Why a name is not an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionError {
    Unknown,
    /// `tab_0` or `profile_0`.
    FromOne,
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unknown => "is not an action",
            Self::FromOne => "is not an action: the tabs and the profiles count from 1",
        })
    }
}

/// A shortcut of `config.toml`: a chord, and its action, or `None` if the
/// chord is free, and its key goes to the program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub chord: Chord,
    pub action: Option<Action>,
}

/// The shortcuts of the window: an action for each chord that meiga takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcuts {
    list: Vec<(Chord, Action)>,
}

impl Default for Shortcuts {
    /// The shortcuts of meiga, without a file.
    fn default() -> Self {
        Self { list: defaults() }
    }
}

impl Shortcuts {
    /// The shortcuts of meiga with the ones of `entries` over them: an entry
    /// with an action gives it to its chord, and one without frees it.
    pub fn new(entries: &[Entry]) -> Self {
        let mut list = defaults();
        for entry in entries {
            list.retain(|(chord, _)| *chord != entry.chord);
            if let Some(action) = entry.action {
                list.push((entry.chord, action));
            }
        }
        Self { list }
    }

    /// The action of `chord`, if meiga takes it.
    pub fn action(&self, chord: Chord) -> Option<Action> {
        self.list
            .iter()
            .find(|(known, _)| *known == chord)
            .map(|&(_, action)| action)
    }

    /// All the chords that meiga takes, and their actions.
    pub fn list(&self) -> &[(Chord, Action)] {
        &self.list
    }
}

/// The shortcuts of meiga: the ones of Windows Terminal for the tabs, the
/// profiles, the menus, copying and pasting, the history and the zoom; and
/// the ones of Chrome to show a tab by its place, the last one, the previous
/// one and the next one. Ctrl+T and Ctrl+W go to the program.
fn defaults() -> Vec<(Chord, Action)> {
    const CTRL: u8 = Chord::CTRL;
    const SHIFT: u8 = Chord::SHIFT;
    let key = |modifiers, key: u8| Chord::new(modifiers, u16::from(key));
    let mut list = vec![
        (key(CTRL | SHIFT, b'T'), Action::NewTab),
        (key(CTRL | SHIFT, b'W'), Action::CloseTab),
        (Chord::new(CTRL, VK_TAB), Action::NextTab),
        (Chord::new(CTRL | SHIFT, VK_TAB), Action::PreviousTab),
        (Chord::new(CTRL, VK_NEXT), Action::NextTab),
        (Chord::new(CTRL, VK_PRIOR), Action::PreviousTab),
        (key(CTRL, b'9'), Action::LastTab),
        (Chord::new(CTRL | SHIFT, VK_SPACE), Action::ProfileMenu),
        (Chord::new(0, VK_APPS), Action::TabMenu),
        (Chord::new(SHIFT, VK_APPS), Action::TabMenu),
        (Chord::new(SHIFT, VK_F10), Action::TabMenu),
        (key(CTRL | SHIFT, b'D'), Action::DuplicateTab),
        (key(CTRL, b'C'), Action::Copy),
        (key(CTRL | SHIFT, b'C'), Action::Copy),
        (key(CTRL, b'V'), Action::Paste),
        (key(CTRL | SHIFT, b'V'), Action::Paste),
        (Chord::new(CTRL | SHIFT, VK_UP), Action::ScrollUp),
        (Chord::new(CTRL | SHIFT, VK_DOWN), Action::ScrollDown),
        (Chord::new(SHIFT, VK_PRIOR), Action::ScrollPageUp),
        (Chord::new(CTRL | SHIFT, VK_PRIOR), Action::ScrollPageUp),
        (Chord::new(SHIFT, VK_NEXT), Action::ScrollPageDown),
        (Chord::new(CTRL | SHIFT, VK_NEXT), Action::ScrollPageDown),
        (Chord::new(CTRL | SHIFT, VK_HOME), Action::ScrollToTop),
        (Chord::new(CTRL | SHIFT, VK_END), Action::ScrollToBottom),
        (Chord::new(CTRL, VK_OEM_PLUS), Action::ZoomIn),
        (Chord::new(CTRL, VK_ADD), Action::ZoomIn),
        (Chord::new(CTRL, VK_OEM_MINUS), Action::ZoomOut),
        (Chord::new(CTRL, VK_SUBTRACT), Action::ZoomOut),
        (key(CTRL, b'0'), Action::ZoomReset),
        (Chord::new(CTRL, VK_NUMPAD0), Action::ZoomReset),
    ];
    for place in 0..8 {
        list.push((key(CTRL, b'1' + place), Action::Tab(u16::from(place))));
    }
    for place in 0..9 {
        list.push((
            key(CTRL | SHIFT, b'1' + place),
            Action::Profile(u16::from(place)),
        ));
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ControlKeys;

    const CTRL: u8 = Chord::CTRL;
    const ALT: u8 = Chord::ALT;
    const SHIFT: u8 = Chord::SHIFT;

    fn chord(text: &str) -> Chord {
        Chord::parse(text).unwrap()
    }

    fn event(key: u16, controls: ControlKeys) -> KeyEvent {
        KeyEvent {
            key,
            scan: 0,
            character: 0,
            down: true,
            controls,
            repeat: 1,
        }
    }

    #[test]
    fn chords_are_written_as_in_windows_terminal() {
        assert_eq!(chord("ctrl+shift+t"), Chord::new(CTRL | SHIFT, 0x54));
        // In any order, with capital letters and blanks.
        assert_eq!(chord("Shift + CTRL + T"), chord("ctrl+shift+t"));
        // A modifier key twice is the same.
        assert_eq!(chord("ctrl+ctrl+t"), chord("ctrl+t"));
        assert_eq!(chord("alt+shift+1"), Chord::new(ALT | SHIFT, 0x31));
        assert_eq!(chord("f1"), Chord::new(0, 0x70));
        assert_eq!(chord("ctrl+F24"), Chord::new(CTRL, 0x87));
        assert_eq!(chord("numpad7"), chord("numpad_7"));
        assert_eq!(chord("numpad_7"), Chord::new(0, 0x67));
        assert_eq!(chord("ctrl+pgup"), chord("ctrl+PageUp"));
        assert_eq!(chord("menu"), chord("app"));
        assert_eq!(chord("ctrl+plus"), Chord::new(CTRL, VK_OEM_PLUS));
        assert_eq!(chord("ctrl+numpad_add"), chord("ctrl+numpad_plus"));
        // Alt+F4 with Ctrl is not the one of Windows.
        assert_eq!(chord("ctrl+alt+f4"), Chord::new(CTRL | ALT, VK_F4));
        for (name, code) in NAMES {
            assert_eq!(chord(&name.to_ascii_uppercase()), Chord::new(0, code));
        }
    }

    #[test]
    fn wrong_chords_say_why() {
        let error = |text: &str| Chord::parse(text).unwrap_err();
        assert_eq!(error(""), ChordError::Empty);
        assert_eq!(error("ctrl+"), ChordError::Empty);
        assert_eq!(error("ctrl++"), ChordError::Empty);
        assert_eq!(error("ctrl+shift"), ChordError::NoKey);
        assert_eq!(error("ctrl+a+b"), ChordError::TwoKeys);
        assert_eq!(
            error("ctrl+hyper+t"),
            ChordError::Unknown("hyper".to_owned())
        );
        assert_eq!(error("ctrl+ñ"), ChordError::Unknown("ñ".to_owned()));
        assert_eq!(error("f0"), ChordError::Unknown("f0".to_owned()));
        assert_eq!(error("f25"), ChordError::Unknown("f25".to_owned()));
        assert_eq!(
            error("numpad10"),
            ChordError::Unknown("numpad10".to_owned())
        );
        // Numbers that do not fit next to the first code are not keys either.
        assert_eq!(error("f65535"), ChordError::Unknown("f65535".to_owned()));
        assert_eq!(
            error("numpad65535"),
            ChordError::Unknown("numpad65535".to_owned())
        );
        assert_eq!(error("win+e"), ChordError::Windows);
        assert_eq!(error("alt+f4"), ChordError::ForWindows);
        assert_eq!(error("alt+shift+space"), ChordError::ForWindows);
        assert_eq!(error("ctrl+hyper+t").to_string(), "`hyper` is not a key");
    }

    #[test]
    fn a_key_that_goes_down_has_the_chord_of_its_modifier_keys_but_altgr_none() {
        let left_ctrl = ControlKeys::LEFT_CTRL;
        let right_alt = ControlKeys::RIGHT_ALT;
        assert_eq!(
            Chord::of(&event(
                0x54,
                left_ctrl | ControlKeys::SHIFT | ControlKeys::NUM_LOCK
            )),
            Some(chord("ctrl+shift+t"))
        );
        assert_eq!(
            Chord::of(&event(
                0x31,
                ControlKeys::RIGHT_CTRL | ControlKeys::LEFT_ALT
            )),
            Some(chord("ctrl+alt+1"))
        );
        // AltGr, with Shift or without it.
        assert_eq!(Chord::of(&event(0x31, left_ctrl | right_alt)), None);
        assert_eq!(
            Chord::of(&event(0x31, left_ctrl | right_alt | ControlKeys::SHIFT)),
            None
        );
        // Both Ctrl and the right Alt are not only AltGr.
        assert_eq!(
            Chord::of(&event(
                0x31,
                left_ctrl | ControlKeys::RIGHT_CTRL | right_alt
            )),
            Some(chord("ctrl+alt+1"))
        );
        // A character without a key.
        assert_eq!(Chord::of(&event(0, ControlKeys::empty())), None);
    }

    #[test]
    fn actions_are_read_by_their_names() {
        assert_eq!(Action::parse("none"), Ok(None));
        assert_eq!(Action::parse("new_tab"), Ok(Some(Action::NewTab)));
        assert_eq!(Action::parse("tab_1"), Ok(Some(Action::Tab(0))));
        assert_eq!(Action::parse("tab_12"), Ok(Some(Action::Tab(11))));
        assert_eq!(Action::parse("profile_9"), Ok(Some(Action::Profile(8))));
        assert_eq!(Action::parse("tab_0"), Err(ActionError::FromOne));
        assert_eq!(Action::parse("profile_0"), Err(ActionError::FromOne));
        for wrong in [
            "tab_",
            "tab_+1",
            "tab_x",
            "tab_99999",
            "New_Tab",
            "None",
            "",
        ] {
            assert_eq!(Action::parse(wrong), Err(ActionError::Unknown), "{wrong}");
        }
        for (name, action) in ACTIONS {
            assert_eq!(Action::parse(name), Ok(Some(action)));
        }
    }

    #[test]
    fn the_shortcuts_of_meiga_leave_ctrl_t_and_ctrl_w_to_the_program() {
        let shortcuts = Shortcuts::default();
        let action = |text| shortcuts.action(chord(text));
        assert_eq!(action("ctrl+shift+t"), Some(Action::NewTab));
        assert_eq!(action("ctrl+shift+w"), Some(Action::CloseTab));
        assert_eq!(action("ctrl+t"), None);
        assert_eq!(action("ctrl+w"), None);
        assert_eq!(action("ctrl+tab"), Some(Action::NextTab));
        assert_eq!(action("ctrl+shift+tab"), Some(Action::PreviousTab));
        assert_eq!(action("ctrl+pagedown"), Some(Action::NextTab));
        assert_eq!(action("ctrl+pageup"), Some(Action::PreviousTab));
        assert_eq!(action("ctrl+1"), Some(Action::Tab(0)));
        assert_eq!(action("ctrl+8"), Some(Action::Tab(7)));
        assert_eq!(action("ctrl+9"), Some(Action::LastTab));
        assert_eq!(action("ctrl+shift+1"), Some(Action::Profile(0)));
        assert_eq!(action("ctrl+shift+9"), Some(Action::Profile(8)));
        assert_eq!(action("ctrl+alt+1"), None);
        assert_eq!(action("ctrl+shift+d"), Some(Action::DuplicateTab));
        assert_eq!(action("ctrl+shift+space"), Some(Action::ProfileMenu));
        assert_eq!(action("menu"), Some(Action::TabMenu));
        assert_eq!(action("shift+menu"), Some(Action::TabMenu));
        assert_eq!(action("shift+f10"), Some(Action::TabMenu));
        assert_eq!(action("f10"), None);
        assert_eq!(action("ctrl+c"), Some(Action::Copy));
        assert_eq!(action("ctrl+shift+v"), Some(Action::Paste));
        assert_eq!(action("shift+insert"), None);
        assert_eq!(action("shift+pageup"), Some(Action::ScrollPageUp));
        assert_eq!(action("ctrl+shift+pagedown"), Some(Action::ScrollPageDown));
        assert_eq!(action("ctrl+shift+home"), Some(Action::ScrollToTop));
        assert_eq!(action("ctrl+numpad_minus"), Some(Action::ZoomOut));
        assert_eq!(action("ctrl+numpad0"), Some(Action::ZoomReset));
        assert_eq!(action("ctrl+shift+plus"), None);
        // One action for each chord.
        let list = shortcuts.list();
        for (index, (chord, _)) in list.iter().enumerate() {
            assert!(!list[index + 1..].iter().any(|(other, _)| other == chord));
        }
    }

    #[test]
    fn the_file_adds_changes_and_frees_shortcuts() {
        let entry = |text, action| Entry {
            chord: chord(text),
            action,
        };
        let shortcuts = Shortcuts::new(&[
            entry("ctrl+t", Some(Action::NewTab)),
            entry("ctrl+6", None),
            entry("ctrl+shift+w", Some(Action::CloseOtherTabs)),
            entry("ctrl+alt+x", None),
        ]);
        let action = |text| shortcuts.action(chord(text));
        assert_eq!(action("ctrl+t"), Some(Action::NewTab));
        assert_eq!(action("ctrl+shift+t"), Some(Action::NewTab));
        assert_eq!(action("ctrl+6"), None);
        assert_eq!(action("ctrl+5"), Some(Action::Tab(4)));
        assert_eq!(action("ctrl+shift+w"), Some(Action::CloseOtherTabs));
        assert_eq!(action("ctrl+alt+x"), None);
        assert_eq!(shortcuts.list().len(), Shortcuts::default().list().len());
    }

    #[test]
    fn only_moving_between_tabs_the_history_and_the_zoom_repeat() {
        let repeating = [
            "next_tab",
            "previous_tab",
            "scroll_up",
            "scroll_down",
            "scroll_page_up",
            "scroll_page_down",
            "scroll_to_top",
            "scroll_to_bottom",
            "zoom_in",
            "zoom_out",
        ];
        for (name, action) in ACTIONS {
            assert_eq!(action.repeats(), repeating.contains(&name), "{name}");
        }
        assert!(!Action::Tab(0).repeats());
        assert!(!Action::Profile(0).repeats());
    }
}
