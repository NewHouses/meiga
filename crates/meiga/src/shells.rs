//! The shells that the tabs run (ADR 0022): the profiles, which meiga finds
//! and `config.toml` changes, hides or adds; the default one, which the
//! button + opens; and the environment of their programs.
//!
//! meiga reads `%APPDATA%\meiga\config.toml` once, when it starts, and
//! finds the shells then and each time that the menu of the profiles opens,
//! so that it shows the ones that were installed meanwhile. If the file has
//! problems, the window tells them once it is on the screen; meiga ignores
//! what is wrong, or the whole file if it cannot read it, and goes on.

use std::fs::File;
use std::io::{self, Read};

use meiga_core::config::{self, Config, MAX_SIZE, Problem};
use meiga_core::environment::{self, Environment};
use meiga_core::profile::{Launch, Profiles};
use meiga_win::profiles;

/// The most problems that the message box shows: the others are only
/// counted.
const MAX_PROBLEMS: usize = 20;

pub struct Shells {
    config: Config,
    /// The environment of meiga, which the configuration and the folders of
    /// the shells use.
    environment: Environment,
    /// The environment of the programs of the tabs, as an environment
    /// block: the one of meiga, with the variables of a terminal.
    block: Vec<u16>,
    /// The home folder of the user, where the profiles start.
    home: Option<String>,
    /// The folder of the system, where `wsl.exe` is.
    system: String,
    profiles: Profiles,
}

impl Shells {
    /// Reads the configuration and finds the shells. Returns them, and the
    /// problems of the configuration, if it has any.
    pub fn load() -> (Self, Option<Report>) {
        let environment = profiles::environment();
        let system = profiles::system_folder()
            .or_else(|| {
                environment
                    .get("SystemRoot")
                    .map(|root| format!(r"{root}\System32"))
            })
            .unwrap_or_else(|| r"C:\Windows\System32".to_owned());
        let (config, mut report) = read_config(&environment);
        let detected = profiles::detect(&environment, &system);
        let (profiles, problems) = Profiles::merge(detected, &config, &environment);
        if let Some(report) = &mut report {
            report.problems.extend(problems);
        } else if !problems.is_empty() {
            report = config_path(&environment).map(|path| Report {
                path,
                problems,
                ignored: false,
            });
        }
        if let Some(report) = &mut report {
            config::sort(&mut report.problems);
        }
        let mut terminal = environment.clone();
        environment::prepare(&mut terminal, env!("CARGO_PKG_VERSION"));
        let shells = Self {
            config,
            block: terminal.block(),
            home: environment.get("USERPROFILE"),
            environment,
            system,
            profiles,
        };
        (shells, report)
    }

    /// Finds the shells again, which may have been installed or removed
    /// meanwhile. The problems of the configuration were already told.
    pub fn refresh(&mut self) {
        let detected = profiles::detect(&self.environment, &self.system);
        self.profiles = Profiles::merge(detected, &self.config, &self.environment).0;
    }

    pub const fn profiles(&self) -> &Profiles {
        &self.profiles
    }

    /// What a tab of the profile at `index` of the menu runs.
    pub fn launch(&self, index: usize) -> Option<Launch> {
        let profile = self.profiles.list().get(index)?;
        Some(profile.launch(self.home.as_deref(), &self.system))
    }

    /// What a tab of the default profile runs.
    pub fn default_launch(&self) -> Launch {
        self.profiles
            .default_profile()
            .launch(self.home.as_deref(), &self.system)
    }

    /// The environment block of the programs of the tabs.
    pub fn environment(&self) -> &[u16] {
        &self.block
    }
}

/// The problems of the configuration, which meiga tells once its window is
/// on the screen.
#[derive(Debug)]
pub struct Report {
    path: String,
    problems: Vec<Problem>,
    /// Whether meiga ignored the whole file.
    ignored: bool,
}

impl Report {
    /// The text of the message box.
    pub fn text(&self) -> String {
        let mut text = format!("There are errors in {}:\n\n", self.path);
        for problem in self.problems.iter().take(MAX_PROBLEMS) {
            text.push_str(&problem.to_string());
            text.push('\n');
        }
        if let Some(more) = self.problems.len().checked_sub(MAX_PROBLEMS)
            && more > 0
        {
            text.push_str(&format!("…and {more} more\n"));
        }
        text.push('\n');
        text.push_str(if self.ignored {
            "meiga ignored the file, and uses its default values."
        } else {
            "meiga ignored them, and uses its default values instead."
        });
        text
    }
}

/// The path of the configuration file, if the user has a folder of
/// application data.
fn config_path(environment: &Environment) -> Option<String> {
    environment
        .get("APPDATA")
        .map(|folder| format!(r"{folder}\meiga\config.toml"))
}

/// Reads the configuration file, if there is one. Returns what it says, or
/// the default values, and its problems, if it has any.
fn read_config(environment: &Environment) -> (Config, Option<Report>) {
    let Some(path) = config_path(environment) else {
        return (Config::default(), None);
    };
    let ignored = |problem| Report {
        path: path.clone(),
        problems: vec![problem],
        ignored: true,
    };
    let bytes = match read_file(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return (Config::default(), None);
        }
        Err(error) => {
            let problem = Problem::new(None, format!("could not read the file: {error}"));
            return (Config::default(), Some(ignored(problem)));
        }
    };
    match config::parse(&bytes) {
        Ok((config, problems)) if problems.is_empty() => (config, None),
        Ok((config, problems)) => {
            let report = Report {
                path,
                problems,
                ignored: false,
            };
            (config, Some(report))
        }
        Err(problem) => (Config::default(), Some(ignored(problem))),
    }
}

/// The contents of the file at `path`, up to a byte more than the largest
/// configuration, which is enough to know that it is too large.
fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_SIZE as u64 + 1)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use meiga_core::config::Position;

    use super::*;

    #[test]
    fn the_report_says_where_each_problem_is_and_what_meiga_did() {
        let problem =
            |line| Problem::new(Some(Position { line, column: 3 }), "unknown key `colour`");
        let report = Report {
            path: r"C:\Users\ana\AppData\Roaming\meiga\config.toml".to_owned(),
            problems: vec![
                problem(2),
                Problem::new(None, "all the profiles are hidden"),
            ],
            ignored: false,
        };
        assert_eq!(
            report.text(),
            "There are errors in C:\\Users\\ana\\AppData\\Roaming\\meiga\\config.toml:\n\n\
             Line 2, column 3: unknown key `colour`\n\
             all the profiles are hidden\n\n\
             meiga ignored them, and uses its default values instead."
        );
        // Only the first ones, and how many more.
        let report = Report {
            problems: (1..=23).map(problem).collect(),
            ignored: true,
            ..report
        };
        let text = report.text();
        assert!(text.contains("Line 20, column 3"), "{text}");
        assert!(!text.contains("Line 21, column 3"), "{text}");
        assert!(
            text.ends_with("…and 3 more\n\nmeiga ignored the file, and uses its default values."),
            "{text}"
        );
    }
}
