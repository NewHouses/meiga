//! Tries the model of the tabs by hand: reads commands, one per line, and
//! shows the tab bar as text after each one. There is no window and no
//! program in the tabs, only the logic that decides their order and which
//! one is active.
//!
//! ```text
//! cargo run -p meiga-core --example tabs
//! ```
//!
//! Each tab gets a name, `a`, `b`, `c`…, in the order in which it opened, and
//! the bar shows the active tab between brackets: `a [b] c`. The commands:
//!
//! - `new` opens a tab at the end and activates it, as the button + will;
//! - `close TAB` closes a tab; closing the only one closes the window, and
//!   the example ends;
//! - `move TAB POSITION` moves a tab, counting the positions from 1;
//! - `activate TAB`, `next` and `previous` activate a tab;
//! - `help` shows the commands, and `quit` ends.
//!
//! The commands can also come from a pipe. Then each one is shown after the
//! prompt, so that the output reads as a session.

use std::io::{self, BufRead, IsTerminal, Write};

use meiga_core::model::{Close, Error, Tab, TabId, Window};

const HELP: &str =
    "Commands: new, close TAB, move TAB POSITION, activate TAB, next, previous, help, quit.
TAB is the name of a tab, and POSITION counts the tabs from 1.";

/// What to show after a command.
enum Outcome {
    /// The bar.
    Bar,
    /// A message.
    Message(String),
    /// Nothing.
    Nothing,
    /// A last message, and the example ends.
    End(String),
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let piped = !stdin.is_terminal();
    let mut out = io::stdout().lock();
    let mut names = Names::default();
    let mut window = Window::new(names.fresh());
    writeln!(out, "{HELP}")?;
    writeln!(out, "{}", bar(&window))?;
    let mut lines = stdin.lock().lines();
    loop {
        if !piped {
            write!(out, "> ")?;
            out.flush()?;
        }
        let Some(line) = lines.next().transpose()? else {
            break;
        };
        if piped {
            writeln!(out, "> {line}")?;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        match run(&mut window, &mut names, &words) {
            Outcome::Bar => writeln!(out, "{}", bar(&window))?,
            Outcome::Message(message) => writeln!(out, "{message}")?,
            Outcome::Nothing => {}
            Outcome::End(message) => {
                writeln!(out, "{message}")?;
                break;
            }
        }
    }
    Ok(())
}

fn run(window: &mut Window<String>, names: &mut Names, words: &[&str]) -> Outcome {
    match words {
        [] => Outcome::Nothing,
        ["new"] => {
            let id = window.open(names.fresh());
            answer(window.activate(id).map(|_| ()))
        }
        ["close", name] => {
            let Some(id) = find(window, name) else {
                return no_tab(name);
            };
            match window.close(id) {
                Ok(Close::Removed { .. }) => Outcome::Bar,
                Ok(Close::LastTab) => Outcome::End(format!(
                    "{name} is the only tab: the window closes with it."
                )),
                Err(error) => Outcome::Message(error.to_string()),
            }
        }
        ["move", name, position] => {
            let Some(id) = find(window, name) else {
                return no_tab(name);
            };
            let Some(position) = position
                .parse::<usize>()
                .ok()
                .filter(|&position| position > 0)
            else {
                return Outcome::Message(format!(
                    "{position} is not a position: they count the tabs from 1."
                ));
            };
            match window.move_tab(id, position - 1) {
                Err(Error::OutOfRange { count, .. }) => {
                    let tabs = if count == 1 { "tab" } else { "tabs" };
                    Outcome::Message(format!(
                        "There is no position {position}: the bar has {count} {tabs}."
                    ))
                }
                result => answer(result),
            }
        }
        ["activate", name] => match find(window, name) {
            Some(id) => answer(window.activate(id).map(|_| ())),
            None => no_tab(name),
        },
        ["next"] => {
            let _ = window.activate_next();
            Outcome::Bar
        }
        ["previous"] => {
            let _ = window.activate_previous();
            Outcome::Bar
        }
        ["help"] => Outcome::Message(HELP.to_owned()),
        ["quit"] => Outcome::End(String::new()),
        _ => Outcome::Message(format!(
            "I do not understand \"{}\".\n{HELP}",
            words.join(" ")
        )),
    }
}

fn answer(result: Result<(), Error>) -> Outcome {
    match result {
        Ok(()) => Outcome::Bar,
        Err(error) => Outcome::Message(error.to_string()),
    }
}

fn find(window: &Window<String>, name: &str) -> Option<TabId> {
    window
        .tabs()
        .iter()
        .find(|tab| tab.content() == name)
        .map(Tab::id)
}

fn no_tab(name: &str) -> Outcome {
    Outcome::Message(format!("There is no tab {name}."))
}

/// The tab bar as text, with the active tab between brackets.
fn bar(window: &Window<String>) -> String {
    let active = window.active().id();
    let tabs: Vec<String> = window
        .tabs()
        .iter()
        .map(|tab| {
            if tab.id() == active {
                format!("[{}]", tab.content())
            } else {
                tab.content().clone()
            }
        })
        .collect();
    tabs.join(" ")
}

/// The names of the tabs, in the order in which they open: `a` to `z`, then
/// `aa`, `ab`…
#[derive(Default)]
struct Names {
    opened: usize,
}

impl Names {
    fn fresh(&mut self) -> String {
        let mut number = self.opened;
        self.opened += 1;
        let mut name = String::new();
        loop {
            name.insert(0, char::from(b'a' + (number % 26) as u8));
            if number < 26 {
                return name;
            }
            number = number / 26 - 1;
        }
    }
}
