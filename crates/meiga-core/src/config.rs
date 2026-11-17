//! The configuration file, `config.toml` (ADR 0022): the default profile,
//! the entries that change, hide or add to the profiles that meiga finds,
//! and the shortcuts that change the ones of meiga (ADR 0023).
//!
//! ```toml
//! default_profile = "Ubuntu"
//!
//! [[profiles]]
//! name = "Command Prompt"
//! hidden = true
//!
//! [[profiles]]
//! name = "Build"
//! command = 'pwsh -NoExit -File "%USERPROFILE%\build.ps1"'
//! directory = 'C:\src\meiga'
//!
//! [shortcuts]
//! "ctrl+t" = "new_tab"
//! "ctrl+6" = "none"
//! ```
//!
//! The file is optional, and meiga reads it when it starts. A file that is
//! not TOML, that is not UTF-8 text, or that has more than 1 MiB, is
//! ignored whole; a value that is wrong, or a key that meiga does not know,
//! only itself; a profile without a name, or with the name of another
//! one, all of it; and a shortcut that is wrong, or whose chord an earlier
//! one has, only itself. Each of them is a [`Problem`], with its line and
//! its column, which the window shows.
//!
//! The file is read without serde: the parser of the `toml` crate gives each
//! key and each value with its place in the file, and this module takes the
//! ones that it knows.

use std::fmt;
use std::ops::Range;
use std::str;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::shortcut::{self, Action, Chord};
use crate::sort::insertion_sort_by;

/// The largest file that meiga reads: a configuration has a few hundred
/// bytes.
pub const MAX_SIZE: usize = 1 << 20;

/// The byte order mark of UTF-8, which some editors write at the start.
const BOM: &[u8] = b"\xef\xbb\xbf";

/// A place in the file: its line and its column, both from 1. The column
/// counts characters, as editors do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// Something wrong in the configuration, which meiga ignores.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// Where it is, if it is in one place of the file.
    pub at: Option<Position>,
    pub message: String,
}

impl Problem {
    pub fn new(at: Option<Position>, message: impl Into<String>) -> Self {
        Self {
            at,
            message: message.into(),
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.at {
            Some(at) => write!(
                f,
                "Line {}, column {}: {}",
                at.line, at.column, self.message
            ),
            None => f.write_str(&self.message),
        }
    }
}

/// A value of the file, and where it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting<T> {
    pub value: T,
    pub at: Position,
}

/// An entry of `[[profiles]]`: what changes in the profile `name`, which
/// meiga found, or a new profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileEntry {
    pub name: String,
    /// The command line, as the program reads it, with `%NAME%` variables.
    pub command: Option<String>,
    /// The folder where it starts, with `%NAME%` variables: `~` is the home
    /// folder.
    pub directory: Option<String>,
    /// Whether the menu leaves it out.
    pub hidden: bool,
}

/// What the configuration file says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// The name of the profile that the button + opens.
    pub default_profile: Option<Setting<String>>,
    /// The entries of `[[profiles]]`, in the order of the file.
    pub profiles: Vec<ProfileEntry>,
    /// The entries of `[shortcuts]`, in the order of the file, each chord
    /// once.
    pub shortcuts: Vec<shortcut::Entry>,
}

/// Reads the configuration in `bytes`, the contents of the file. Returns it,
/// with the problems of what it ignored, in the order of the file; or the
/// problem that makes it ignore all of it.
pub fn parse(bytes: &[u8]) -> Result<(Config, Vec<Problem>), Problem> {
    if bytes.len() > MAX_SIZE {
        return Err(Problem::new(None, "the file has more than 1 MiB"));
    }
    let bytes = bytes.strip_prefix(BOM).unwrap_or(bytes);
    let text = str::from_utf8(bytes).map_err(|error| {
        // The text is valid up to where the error is.
        let valid = str::from_utf8(&bytes[..error.valid_up_to()]).unwrap_or_default();
        Problem::new(
            Some(position(valid, valid.len())),
            "the file is not UTF-8 text",
        )
    })?;
    let table = DeTable::parse(text).map_err(|error| {
        let at = error.span().map(|span| position(text, span.start));
        Problem::new(at, error.message())
    })?;
    let mut reader = Reader {
        text,
        problems: Vec::new(),
    };
    let config = reader.config(table.get_ref());
    let mut problems = reader.problems;
    sort(&mut problems);
    Ok((config, problems))
}

/// Puts `problems` in the order of the file: by their place, and the ones
/// that have none at the end.
pub fn sort(problems: &mut [Problem]) {
    let key = |problem: &Problem| (problem.at.is_none(), problem.at);
    insertion_sort_by(problems, |a, b| key(a).cmp(&key(b)));
}

/// Takes the values that meiga knows from the tables of a file, and notes
/// the problems of the others.
struct Reader<'a> {
    text: &'a str,
    problems: Vec<Problem>,
}

type Key<'i> = Spanned<std::borrow::Cow<'i, str>>;
type Value<'i> = Spanned<DeValue<'i>>;

impl Reader<'_> {
    fn config(&mut self, table: &DeTable<'_>) -> Config {
        let mut config = Config::default();
        for (key, value) in table {
            match key.get_ref().as_ref() {
                "default_profile" => {
                    config.default_profile = self.string(key, value).map(|name| Setting {
                        value: name,
                        at: self.at(value.span()),
                    });
                }
                "profiles" => config.profiles = self.profiles(key, value),
                "shortcuts" => config.shortcuts = self.shortcuts(key, value),
                _ => self.unknown(key),
            }
        }
        config
    }

    /// The entries of `[shortcuts]`: a chord, and the name of its action.
    fn shortcuts(&mut self, key: &Key<'_>, value: &Value<'_>) -> Vec<shortcut::Entry> {
        let DeValue::Table(table) = value.get_ref() else {
            self.problem(
                value.span(),
                format!(
                    "`{}` has to be a table ([shortcuts]), not {}",
                    key.get_ref(),
                    kind(value.get_ref())
                ),
            );
            return Vec::new();
        };
        // The table gives its keys in the order of their text: the entries
        // go in the order of the file, so that the first one of a chord
        // counts.
        let mut items: Vec<(&Key<'_>, &Value<'_>)> = table.iter().collect();
        insertion_sort_by(&mut items, |a, b| a.0.span().start.cmp(&b.0.span().start));
        let mut entries = Vec::<(shortcut::Entry, u32)>::with_capacity(items.len());
        for (key, value) in items {
            let text = key.get_ref();
            let chord = match Chord::parse(text) {
                Ok(chord) => chord,
                Err(_) if text.trim().is_empty() => {
                    self.problem(key.span(), "a shortcut cannot be empty");
                    continue;
                }
                Err(error) => {
                    self.problem(key.span(), format!("`{text}` is not a shortcut: {error}"));
                    continue;
                }
            };
            let Some(name) = self.string(key, value) else {
                continue;
            };
            let action = match Action::parse(&name) {
                Ok(action) => action,
                Err(error) => {
                    self.problem(value.span(), format!("`{name}` {error}"));
                    continue;
                }
            };
            if let Some((_, line)) = entries.iter().find(|(entry, _)| entry.chord == chord) {
                let message = format!("`{text}` repeats the shortcut of line {line}");
                self.problem(key.span(), message);
                continue;
            }
            let line = self.at(key.span()).line;
            entries.push((shortcut::Entry { chord, action }, line));
        }
        entries.into_iter().map(|(entry, _)| entry).collect()
    }

    /// The entries of `[[profiles]]`.
    fn profiles(&mut self, key: &Key<'_>, value: &Value<'_>) -> Vec<ProfileEntry> {
        let DeValue::Array(array) = value.get_ref() else {
            self.problem(
                value.span(),
                format!(
                    "`{}` has to be an array of tables ([[profiles]]), not {}",
                    key.get_ref(),
                    kind(value.get_ref())
                ),
            );
            return Vec::new();
        };
        let mut entries = Vec::<ProfileEntry>::with_capacity(array.len());
        for item in array {
            let DeValue::Table(table) = item.get_ref() else {
                self.problem(
                    item.span(),
                    format!("a profile has to be a table, not {}", kind(item.get_ref())),
                );
                continue;
            };
            let Some((entry, name)) = self.profile(table, item.span()) else {
                continue;
            };
            if entries.iter().any(|other| other.name == entry.name) {
                self.problem(
                    name,
                    format!("another profile is already named `{}`", entry.name),
                );
            } else {
                entries.push(entry);
            }
        }
        entries
    }

    /// An entry of `[[profiles]]`, which is `span` in the file, and where its
    /// name is, or `None` if it has no name.
    fn profile(
        &mut self,
        table: &DeTable<'_>,
        span: Range<usize>,
    ) -> Option<(ProfileEntry, Range<usize>)> {
        let mut name = None;
        let mut named = false;
        let (mut command, mut directory, mut hidden) = (None, None, false);
        for (key, value) in table {
            match key.get_ref().as_ref() {
                "name" => {
                    named = true;
                    name = self.string(key, value).map(|name| (name, value.span()));
                }
                "command" => command = self.string(key, value),
                "directory" => directory = self.string(key, value),
                "hidden" => hidden = self.boolean(key, value).unwrap_or(hidden),
                _ => self.unknown(key),
            }
        }
        // A name that is wrong already has its problem.
        if !named {
            self.problem(span, "a profile needs a `name`");
        }
        let (name, at) = name?;
        Some((
            ProfileEntry {
                name,
                command,
                directory,
                hidden,
            },
            at,
        ))
    }

    /// The text of `value`, if it is a string that is not empty.
    fn string(&mut self, key: &Key<'_>, value: &Value<'_>) -> Option<String> {
        match value.get_ref() {
            DeValue::String(text) if text.is_empty() => {
                self.problem(value.span(), format!("`{}` cannot be empty", key.get_ref()));
                None
            }
            DeValue::String(text) => Some(text.to_string()),
            other => {
                self.problem(
                    value.span(),
                    format!(
                        "`{}` has to be a string, not {}",
                        key.get_ref(),
                        kind(other)
                    ),
                );
                None
            }
        }
    }

    /// The value of `value`, if it is a boolean.
    fn boolean(&mut self, key: &Key<'_>, value: &Value<'_>) -> Option<bool> {
        match value.get_ref() {
            DeValue::Boolean(value) => Some(*value),
            other => {
                self.problem(
                    value.span(),
                    format!(
                        "`{}` has to be true or false, not {}",
                        key.get_ref(),
                        kind(other)
                    ),
                );
                None
            }
        }
    }

    fn unknown(&mut self, key: &Key<'_>) {
        self.problem(key.span(), format!("unknown key `{}`", key.get_ref()));
    }

    fn problem(&mut self, span: Range<usize>, message: impl Into<String>) {
        let at = self.at(span);
        self.problems.push(Problem::new(Some(at), message));
    }

    /// Where `span` starts in the file.
    fn at(&self, span: Range<usize>) -> Position {
        position(self.text, span.start)
    }
}

/// What kind of value `value` is, as the problems say it.
const fn kind(value: &DeValue<'_>) -> &'static str {
    match value {
        DeValue::String(_) => "a string",
        DeValue::Integer(_) => "an integer",
        DeValue::Float(_) => "a float",
        DeValue::Boolean(_) => "a boolean",
        DeValue::Datetime(_) => "a date",
        DeValue::Array(_) => "an array",
        DeValue::Table(_) => "a table",
    }
}

/// The line and the column of the byte at `offset` in `text`.
fn position(text: &str, offset: usize) -> Position {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &text[..offset];
    let start = before.rfind('\n').map_or(0, |newline| newline + 1);
    let line = before.bytes().filter(|&byte| byte == b'\n').count() + 1;
    let column = before[start..].chars().count() + 1;
    Position {
        line: u32::try_from(line).unwrap_or(u32::MAX),
        column: u32::try_from(column).unwrap_or(u32::MAX),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> (Config, Vec<String>) {
        let (config, problems) = parse(text.as_bytes()).unwrap();
        (config, problems.iter().map(ToString::to_string).collect())
    }

    fn fails(bytes: &[u8]) -> String {
        parse(bytes).unwrap_err().to_string()
    }

    fn entry(name: &str) -> ProfileEntry {
        ProfileEntry {
            name: name.to_owned(),
            command: None,
            directory: None,
            hidden: false,
        }
    }

    #[test]
    fn an_empty_file_says_nothing() {
        assert_eq!(read(""), (Config::default(), Vec::new()));
        assert_eq!(read("# Nothing yet.\r\n"), (Config::default(), Vec::new()));
    }

    #[test]
    fn it_takes_the_default_profile_and_the_profiles_in_their_order() {
        let (config, problems) = read(
            "default_profile = \"Ubuntu\"\n\
             \n\
             [[profiles]]\n\
             name = \"Command Prompt\"\n\
             hidden = true\n\
             \n\
             [[profiles]]\n\
             name = \"Build\"\n\
             command = 'pwsh -NoExit -File \"%USERPROFILE%\\build.ps1\"'\n\
             directory = 'C:\\src\\meiga'\n",
        );
        assert_eq!(problems, Vec::<String>::new());
        assert_eq!(
            config,
            Config {
                default_profile: Some(Setting {
                    value: "Ubuntu".to_owned(),
                    at: Position {
                        line: 1,
                        column: 19
                    },
                }),
                profiles: vec![
                    ProfileEntry {
                        hidden: true,
                        ..entry("Command Prompt")
                    },
                    ProfileEntry {
                        command: Some(r#"pwsh -NoExit -File "%USERPROFILE%\build.ps1""#.to_owned()),
                        directory: Some(r"C:\src\meiga".to_owned()),
                        ..entry("Build")
                    },
                ],
                shortcuts: Vec::new(),
            }
        );
        // Inline tables too.
        let (config, _) = read("profiles = [{ name = \"a\" }, { name = \"b\", hidden = false }]");
        assert_eq!(config.profiles, [entry("a"), entry("b")]);
    }

    #[test]
    fn wrong_values_and_unknown_keys_are_ignored_with_their_place() {
        let (config, problems) = read(
            "default_profile = 42\r\n\
             colour = \"red\"\r\n\
             [[profiles]]\r\n\
             name = \"Ubuntu\"\r\n\
             command = \"\"\r\n\
             hidden = \"yes\"\r\n\
             shortcut = \"ctrl+1\"\r\n\
             [[profiles]]\r\n\
             name = \"cmd\"\r\n\
             directory = ['C:\\']\r\n",
        );
        assert_eq!(
            problems,
            [
                "Line 1, column 19: `default_profile` has to be a string, not an integer",
                "Line 2, column 1: unknown key `colour`",
                "Line 5, column 11: `command` cannot be empty",
                "Line 6, column 10: `hidden` has to be true or false, not a string",
                "Line 7, column 1: unknown key `shortcut`",
                "Line 10, column 13: `directory` has to be a string, not an array",
            ]
        );
        // What is right stays.
        assert_eq!(config.default_profile, None);
        assert_eq!(config.profiles, [entry("Ubuntu"), entry("cmd")]);
    }

    #[test]
    fn a_profile_needs_a_name_of_its_own() {
        let (config, problems) = read(
            "[[profiles]]\n\
             command = \"cmd\"\n\
             [[profiles]]\n\
             name = 7\n\
             [[profiles]]\n\
             name = \"a\"\n\
             [[profiles]]\n\
             name = \"a\"\n\
             hidden = true\n",
        );
        assert_eq!(
            problems,
            [
                "Line 1, column 1: a profile needs a `name`",
                "Line 4, column 8: `name` has to be a string, not an integer",
                "Line 8, column 8: another profile is already named `a`",
            ]
        );
        assert_eq!(config.profiles, [entry("a")]);
        let (config, problems) = read("profiles = { name = \"a\" }\nprofiles2 = 1");
        assert_eq!(
            problems,
            [
                "Line 1, column 12: `profiles` has to be an array of tables ([[profiles]]), not a table",
                "Line 2, column 1: unknown key `profiles2`",
            ]
        );
        assert!(config.profiles.is_empty());
        let (_, problems) = read("profiles = [\"cmd\"]");
        assert_eq!(
            problems,
            ["Line 1, column 13: a profile has to be a table, not a string"]
        );
    }

    fn shortcut(text: &str, action: Option<Action>) -> shortcut::Entry {
        shortcut::Entry {
            chord: Chord::parse(text).unwrap(),
            action,
        }
    }

    #[test]
    fn shortcuts_go_in_the_order_of_the_file_and_the_first_of_a_chord_counts() {
        // In the order of their text, `Ctrl+T` would come first.
        let (config, problems) = read(
            "[shortcuts]\n\
             \"ctrl+t\" = \"new_tab\"\n\
             \"ctrl+6\" = \"none\"\n\
             \"alt+a\" = \"tab_2\"\n\
             \"Ctrl+T\" = \"close_tab\"\n",
        );
        assert_eq!(
            problems,
            ["Line 5, column 1: `Ctrl+T` repeats the shortcut of line 2"]
        );
        assert_eq!(
            config.shortcuts,
            [
                shortcut("ctrl+t", Some(Action::NewTab)),
                shortcut("ctrl+6", None),
                shortcut("alt+a", Some(Action::Tab(1))),
            ]
        );
        // An inline table too.
        let (config, _) = read("shortcuts = { \"ctrl+shift+u\" = \"profile_4\" }");
        assert_eq!(
            config.shortcuts,
            [shortcut("ctrl+shift+u", Some(Action::Profile(3)))]
        );
        // The same text twice is not TOML.
        assert!(
            fails(b"[shortcuts]\n\"ctrl+t\" = \"new_tab\"\n\"ctrl+t\" = \"none\"")
                .starts_with("Line 3, column 1: ")
        );
    }

    #[test]
    fn wrong_shortcuts_are_ignored_with_their_place() {
        let (config, problems) = read(
            "[shortcuts]\n\
             \"ctrl+a+b\" = \"new_tab\"\n\
             \"\" = \"new_tab\"\n\
             \"win+e\" = \"new_tab\"\n\
             \"alt+f4\" = \"close_tab\"\n\
             \"ctrl+q\" = \"quit\"\n\
             \"ctrl+0\" = \"tab_0\"\n\
             \"ctrl+e\" = 5\n\
             \"ctrl+r\" = \"\"\n\
             \"ctrl+y\" = \"nwe_tab\"\n\
             \"Ctrl+Y\" = \"duplicate_tab\"\n",
        );
        assert_eq!(
            problems,
            [
                "Line 2, column 1: `ctrl+a+b` is not a shortcut: it has more than one key",
                "Line 3, column 1: a shortcut cannot be empty",
                "Line 4, column 1: `win+e` is not a shortcut: the Windows keys are for Windows",
                "Line 5, column 1: `alt+f4` is not a shortcut: Alt+F4 and Alt+Space are for Windows",
                "Line 6, column 12: `quit` is not an action",
                "Line 7, column 12: `tab_0` is not an action: the tabs and the profiles count from 1",
                "Line 8, column 12: `ctrl+e` has to be a string, not an integer",
                "Line 9, column 12: `ctrl+r` cannot be empty",
                "Line 10, column 12: `nwe_tab` is not an action",
            ]
        );
        // The first right one of a chord counts.
        assert_eq!(
            config.shortcuts,
            [shortcut("ctrl+y", Some(Action::DuplicateTab))]
        );
        let (config, problems) = read("shortcuts = [\"ctrl+t\"]");
        assert_eq!(
            problems,
            ["Line 1, column 13: `shortcuts` has to be a table ([shortcuts]), not an array"]
        );
        assert!(config.shortcuts.is_empty());
    }

    #[test]
    fn a_file_that_is_not_toml_is_ignored_whole() {
        assert_eq!(
            fails(b"default_profile = \"Ubuntu\"\n[[profiles]]\nhidden = tru\n"),
            "Line 3, column 10: invalid boolean, expected `true`"
        );
        // The columns count characters: in bytes, the end would be the 22nd.
        assert_eq!(
            fails("# Galego: ñ\ndefault_profile = 'Ñ".as_bytes()),
            "Line 2, column 21: invalid literal string, expected `'`"
        );
        assert!(fails(b"a = 1\na = 2").starts_with("Line 2, column 1: "));
    }

    #[test]
    fn the_file_is_utf8_with_or_without_its_mark_and_not_too_large() {
        let (config, problems) = read("\u{feff}default_profile = \"ñ\"");
        assert!(problems.is_empty());
        assert_eq!(
            config.default_profile.map(|name| (name.value, name.at)),
            Some((
                "ñ".to_owned(),
                Position {
                    line: 1,
                    column: 19
                }
            ))
        );
        assert_eq!(
            fails(b"# ok\ndefault_profile = \"\xe1\""),
            "Line 2, column 20: the file is not UTF-8 text"
        );
        let large = vec![b'#'; MAX_SIZE + 1];
        assert_eq!(fails(&large), "the file has more than 1 MiB");
        assert!(parse(&large[..MAX_SIZE]).is_ok());
    }
}
