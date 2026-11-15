//! The environment of the programs of the tabs (ADR 0022): the one of meiga,
//! as Windows gives it when meiga starts, with the variables that tell the
//! programs which terminal they run in, and without the ones of Windows
//! Terminal, which meiga inherits when Windows Terminal starts it.
//!
//! The variables are kept as Windows has them, in UTF-16: a name can have
//! any character but `=` and the null, and a value any but the null. Names do
//! not distinguish upper and lower case, as in Windows. A name can start with
//! `=`: Windows keeps the current folder of each drive in variables such as
//! `=C:`, which `cmd` uses.

use std::cmp::Ordering;

use crate::sort::insertion_sort_by;

/// The variables that tell a program which terminal it runs in, as in VS
/// Code and WezTerm: its name, its version, and that it shows 24-bit colour.
pub const TERM_PROGRAM: &str = "TERM_PROGRAM";
pub const TERM_PROGRAM_VERSION: &str = "TERM_PROGRAM_VERSION";
pub const COLORTERM: &str = "COLORTERM";

/// The variables of Windows Terminal: a program that sees them believes that
/// it runs there.
const WINDOWS_TERMINAL: [&str; 2] = ["WT_SESSION", "WT_PROFILE_ID"];

/// The variables that WSL passes from Windows to the programs of Linux,
/// separated by colons, each one with its flags after a slash.
const WSLENV: &str = "WSLENV";

/// The value of [`TERM_PROGRAM`].
const NAME: &str = "meiga";

/// A set of environment variables.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Environment {
    /// Each variable as `NAME=value`, without its null.
    vars: Vec<Vec<u16>>,
}

impl Environment {
    /// The variables of `block`, as `GetEnvironmentStringsW` gives them: each
    /// one ends with a null, and an empty one ends the block.
    pub fn from_block(block: &[u16]) -> Self {
        let vars = block
            .split(|&unit| unit == 0)
            .take_while(|var| !var.is_empty())
            .map(<[u16]>::to_vec)
            .collect();
        Self { vars }
    }

    /// The value of the variable `name`, if there is one. A character that
    /// is not valid UTF-16 becomes U+FFFD.
    pub fn get(&self, name: &str) -> Option<String> {
        let var = &self.vars[self.position(name)?];
        let value = var.get(name_length(var) + 1..).unwrap_or_default();
        Some(String::from_utf16_lossy(value))
    }

    /// Sets the variable `name` to `value`, instead of the one that has the
    /// same name, whatever its case.
    pub fn set(&mut self, name: &str, value: &str) {
        let var = name
            .encode_utf16()
            .chain(Some(u16::from(b'=')))
            .chain(value.encode_utf16())
            .collect();
        match self.position(name) {
            Some(index) => self.vars[index] = var,
            None => self.vars.push(var),
        }
    }

    /// Removes the variable `name`, if there is one.
    pub fn remove(&mut self, name: &str) {
        if let Some(index) = self.position(name) {
            self.vars.remove(index);
        }
    }

    /// `text` with each `%NAME%` replaced by the value of the variable, as
    /// `ExpandEnvironmentStringsW` does: a name without a variable stays as
    /// it is, with its percent signs, and its second one can start the next
    /// name.
    pub fn expand(&self, text: &str) -> String {
        let mut expanded = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(start) = rest.find('%') {
            expanded.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            let Some(end) = after.find('%') else {
                rest = &rest[start..];
                break;
            };
            let name = &after[..end];
            let value = if name.is_empty() {
                None
            } else {
                self.get(name)
            };
            match value {
                Some(value) => {
                    expanded.push_str(&value);
                    rest = &after[end + 1..];
                }
                None => {
                    expanded.push('%');
                    rest = after;
                }
            }
        }
        expanded.push_str(rest);
        expanded
    }

    /// The variables as `CreateProcessW` takes them with
    /// `CREATE_UNICODE_ENVIRONMENT`: each one ends with a null, and one more
    /// null ends the block. They go in the order of their names, without
    /// distinguishing upper and lower case, as Windows asks.
    pub fn block(&self) -> Vec<u16> {
        let mut vars: Vec<&[u16]> = self.vars.iter().map(Vec::as_slice).collect();
        insertion_sort_by(&mut vars, |first, second| compare_names(first, second));
        let length = vars.iter().map(|var| var.len() + 1).sum::<usize>();
        let mut block = Vec::with_capacity(length + 2);
        for var in vars {
            block.extend_from_slice(var);
            block.push(0);
        }
        // An empty block has two nulls too.
        if block.is_empty() {
            block.push(0);
        }
        block.push(0);
        block
    }

    /// Where the variable `name` is, if there is one.
    fn position(&self, name: &str) -> Option<usize> {
        self.vars
            .iter()
            .position(|var| uppercase(&var[..name_length(var)]).eq(uppercase_str(name)))
    }
}

/// Makes `environment` the one of the programs of the tabs: with
/// `TERM_PROGRAM=meiga`, `TERM_PROGRAM_VERSION=version` and
/// `COLORTERM=truecolor`, which WSL passes to Linux too, and without the
/// variables of Windows Terminal, also in `WSLENV`. It does not set `TERM`:
/// the ConPTY talks to the programs of Windows, and WSL sets it for Linux.
pub fn prepare(environment: &mut Environment, version: &str) {
    for name in WINDOWS_TERMINAL {
        environment.remove(name);
    }
    environment.set(TERM_PROGRAM, NAME);
    environment.set(TERM_PROGRAM_VERSION, version);
    environment.set(COLORTERM, "truecolor");
    let current = environment.get(WSLENV);
    let wslenv = wslenv(
        current.as_deref(),
        &[TERM_PROGRAM, TERM_PROGRAM_VERSION, COLORTERM],
        &WINDOWS_TERMINAL,
    );
    environment.set(WSLENV, &wslenv);
}

/// `current`, the value of `WSLENV`, with the variables of `add` that it does
/// not have yet, with or without flags, at its end, and without the ones of
/// `drop`, nor empty items.
fn wslenv(current: Option<&str>, add: &[&str], drop: &[&str]) -> String {
    let named = |item: &str, name: &str| {
        item.split_once('/')
            .map_or(item, |(item, _)| item)
            .eq_ignore_ascii_case(name)
    };
    let mut items: Vec<&str> = current
        .unwrap_or_default()
        .split(':')
        .filter(|item| !item.is_empty() && !drop.iter().any(|name| named(item, name)))
        .collect();
    for name in add {
        if !items.iter().any(|item| named(item, name)) {
            items.push(name);
        }
    }
    items.join(":")
}

/// The length of the name of `var`, a `NAME=value`: up to its first `=`,
/// which is part of the name if it is the first character.
fn name_length(var: &[u16]) -> usize {
    var.iter()
        .skip(1)
        .position(|&unit| unit == u16::from(b'='))
        .map_or(var.len(), |at| at + 1)
}

/// Orders two variables by their names, as uppercase characters.
fn compare_names(first: &[u16], second: &[u16]) -> Ordering {
    let first = uppercase(&first[..name_length(first)]);
    first.cmp(uppercase(&second[..name_length(second)]))
}

/// The characters of `name` in uppercase. A unit that is not valid UTF-16
/// becomes U+FFFD.
fn uppercase(name: &[u16]) -> impl Iterator<Item = char> + '_ {
    char::decode_utf16(name.iter().copied())
        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
        .flat_map(char::to_uppercase)
}

fn uppercase_str(name: &str) -> impl Iterator<Item = char> + '_ {
    name.chars().flat_map(char::to_uppercase)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    /// An environment of `vars`, as Windows gives it.
    fn environment(vars: &[&str]) -> Environment {
        let mut block = Vec::new();
        for var in vars {
            block.extend(wide(var));
            block.push(0);
        }
        block.push(0);
        Environment::from_block(&block)
    }

    /// The variables of a block, as text.
    fn vars(block: &[u16]) -> Vec<String> {
        assert!(
            block.ends_with(&[0, 0]),
            "the block has to end with two nulls"
        );
        block[..block.len() - 1]
            .split(|&unit| unit == 0)
            .filter(|var| !var.is_empty())
            .map(String::from_utf16_lossy)
            .collect()
    }

    #[test]
    fn names_do_not_distinguish_case_and_can_start_with_an_equals_sign() {
        let mut env = environment(&["=C:=C:\\src", "Path=C:\\Windows", "EMPTY=", "x=a=b"]);
        assert_eq!(env.get("=c:").as_deref(), Some("C:\\src"));
        assert_eq!(env.get("PATH").as_deref(), Some("C:\\Windows"));
        assert_eq!(env.get("empty").as_deref(), Some(""));
        // The first equals sign after the name ends it.
        assert_eq!(env.get("X").as_deref(), Some("a=b"));
        assert_eq!(env.get("missing"), None);

        env.set("PATH", "C:\\bin");
        env.set("New", "1");
        env.remove("=C:");
        assert_eq!(
            vars(&env.block()),
            ["EMPTY=", "New=1", "PATH=C:\\bin", "x=a=b"]
        );
    }

    #[test]
    fn the_block_goes_in_the_order_of_the_names_and_ends_with_two_nulls() {
        let env = environment(&["b=2", "A_B=3", "a=1", "=D:=D:\\", "Ñ=4", "z=5"]);
        // `=` goes before the letters, and `_` after them.
        assert_eq!(
            vars(&env.block()),
            ["=D:=D:\\", "a=1", "A_B=3", "b=2", "z=5", "Ñ=4"]
        );
        assert_eq!(Environment::default().block(), [0, 0]);
        // The block of Windows ends at its first empty variable.
        let block = [wide("a=1"), vec![0, 0], wide("b=2"), vec![0]].concat();
        assert_eq!(vars(&Environment::from_block(&block).block()), ["a=1"]);
    }

    #[test]
    fn variables_expand_as_windows_expands_them() {
        let env = environment(&["USERPROFILE=C:\\Users\\ana", "A=x"]);
        assert_eq!(
            env.expand("%userprofile%\\src\\%A%"),
            "C:\\Users\\ana\\src\\x"
        );
        // A name without a variable stays, and its second sign can start
        // the next one.
        assert_eq!(env.expand("%NONE%A%"), "%NONEx");
        assert_eq!(env.expand("100%"), "100%");
        assert_eq!(env.expand("%%A%"), "%x");
        assert_eq!(env.expand("ñ%A%ñ"), "ñxñ");
        assert_eq!(env.expand(""), "");
    }

    #[test]
    fn the_programs_know_that_they_run_in_meiga_and_not_in_windows_terminal() {
        let mut env = environment(&[
            "Path=C:\\Windows",
            "WT_SESSION=0b5c",
            "wt_profile_id={61c5}",
            "WSLENV=WT_SESSION:WT_PROFILE_ID:",
            "COLORTERM=24bit",
        ]);
        prepare(&mut env, "0.1.0");
        assert_eq!(
            vars(&env.block()),
            [
                "COLORTERM=truecolor",
                "Path=C:\\Windows",
                "TERM_PROGRAM=meiga",
                "TERM_PROGRAM_VERSION=0.1.0",
                "WSLENV=TERM_PROGRAM:TERM_PROGRAM_VERSION:COLORTERM",
            ]
        );
        // Without WSLENV.
        let mut env = environment(&["Path=C:\\Windows"]);
        prepare(&mut env, "1.2.3");
        assert_eq!(
            env.get("WSLENV").as_deref(),
            Some("TERM_PROGRAM:TERM_PROGRAM_VERSION:COLORTERM")
        );
    }

    #[test]
    fn wslenv_keeps_what_it_has_and_its_flags() {
        let add = [TERM_PROGRAM, TERM_PROGRAM_VERSION, COLORTERM];
        assert_eq!(
            wslenv(Some("USERPROFILE/p:term_program/u"), &add, &[]),
            "USERPROFILE/p:term_program/u:TERM_PROGRAM_VERSION:COLORTERM"
        );
        assert_eq!(
            wslenv(Some("A::WT_SESSION/u:B"), &["C"], &WINDOWS_TERMINAL),
            "A:B:C"
        );
        assert_eq!(wslenv(None, &["C"], &[]), "C");
    }
}
