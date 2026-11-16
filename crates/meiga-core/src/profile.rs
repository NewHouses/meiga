//! The profiles of the tabs (ADR 0022): the shells that meiga finds on the
//! computer, with what the configuration changes, hides or adds, and how a
//! session starts each of them.
//!
//! A profile has a name, which the menu shows and the tab too, until its
//! program gives a title; a command line; and the folder where it starts:
//! the home one, unless the configuration says another. The programs of WSL
//! start in the home folder of Linux: as in Windows Terminal, `wsl.exe`
//! gets the folder as `--cd`, and `~` is the home one.

use crate::config::{Config, Problem};
use crate::environment::Environment;
use crate::shell;

/// The names of the profiles that meiga finds, in the order of the menu.
/// The distributions of WSL, between Command Prompt and Git Bash, have their
/// own names.
pub const POWERSHELL: &str = "PowerShell";
pub const WINDOWS_POWERSHELL: &str = "Windows PowerShell";
pub const COMMAND_PROMPT: &str = "Command Prompt";
pub const GIT_BASH: &str = "Git Bash";

/// The folder where a profile starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Folder {
    /// The home folder: for WSL, the one of Linux; for the other programs,
    /// the one of the user in Windows, `%USERPROFILE%`.
    Home,
    /// A folder of Windows or, for WSL, any that `wsl.exe --cd` takes.
    Path(String),
}

impl Folder {
    /// The folder of a `directory` of the configuration: `~` is the home
    /// one.
    pub fn of(directory: String) -> Self {
        if directory == "~" {
            Self::Home
        } else {
            Self::Path(directory)
        }
    }
}

/// A profile of the menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// The program and its arguments, quoted as the program reads them.
    pub command: String,
    pub folder: Folder,
}

impl Profile {
    /// How a session starts it, with `home` as the home folder of the user,
    /// if Windows gives one, and `system` as the folder of the system, where
    /// `wsl.exe` is.
    pub fn launch(&self, home: Option<&str>, system: &str) -> Launch {
        let command = wsl_command(&self.command, &self.folder, system);
        let folder = match (&self.folder, &command) {
            // WSL has the folder in its command line, and starts in the home
            // folder, which no folder of Linux keeps in use.
            (_, Some(_)) | (Folder::Home, None) => home.map(str::to_owned),
            (Folder::Path(path), None) => Some(path.clone()),
        };
        Launch {
            name: self.name.clone(),
            command: command.unwrap_or_else(|| self.command.clone()),
            folder,
        }
    }
}

/// What a session runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    /// What its tab shows until the program gives a title.
    pub name: String,
    /// The program and its arguments.
    pub command: String,
    /// The folder where it starts, or `None` for the one of meiga.
    pub folder: Option<String>,
}

impl Launch {
    /// The program of the command line of meiga, `command`, which starts in
    /// the folder of meiga, and whose tab shows its name.
    pub fn command_line(command: &str) -> Self {
        Self {
            name: shell::program_name(command).to_owned(),
            command: command.to_owned(),
            folder: None,
        }
    }
}

/// The profiles of the menu, and the default one, which the button + and
/// Ctrl+Shift+T open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profiles {
    /// The visible ones, never empty.
    list: Vec<Profile>,
    default: usize,
}

impl Profiles {
    /// The profiles that meiga found, `detected`, in their order, with what
    /// `config` changes, hides or adds after them, and its `%NAME%`
    /// variables expanded with `environment`. Returns them, and the problems
    /// of the configuration that only show here.
    ///
    /// An entry of the configuration changes or hides the profile with its
    /// name, or adds a profile if it has a command. Without a command nor
    /// a profile of its name, it is ignored, without a problem: the same
    /// file can go to computers without the same shells. The default profile
    /// is the one that the configuration says, if it is visible, and
    /// otherwise PowerShell, Windows PowerShell or the first one. If all of
    /// them are hidden, they are all shown.
    pub fn merge(
        detected: Vec<Profile>,
        config: &Config,
        environment: &Environment,
    ) -> (Self, Vec<Problem>) {
        let mut profiles: Vec<(Profile, bool)> = detected
            .into_iter()
            .map(|profile| (profile, false))
            .collect();
        for entry in &config.profiles {
            let command = entry
                .command
                .as_deref()
                .map(|text| environment.expand(text));
            let folder = entry
                .directory
                .as_deref()
                .map(|text| Folder::of(environment.expand(text)));
            if let Some((profile, hidden)) = profiles
                .iter_mut()
                .find(|(profile, _)| profile.name == entry.name)
            {
                if let Some(command) = command {
                    profile.command = command;
                }
                if let Some(folder) = folder {
                    profile.folder = folder;
                }
                *hidden = entry.hidden;
            } else if let Some(command) = command {
                let name = entry.name.clone();
                let folder = folder.unwrap_or(Folder::Home);
                profiles.push((
                    Profile {
                        name,
                        command,
                        folder,
                    },
                    entry.hidden,
                ));
            }
        }

        let mut problems = Vec::new();
        let mut list: Vec<Profile> = profiles
            .iter()
            .filter(|(_, hidden)| !hidden)
            .map(|(profile, _)| profile.clone())
            .collect();
        if list.is_empty() && !profiles.is_empty() {
            problems.push(Problem::new(
                None,
                "all the profiles are hidden, so meiga shows them all",
            ));
            list = profiles
                .iter()
                .map(|(profile, _)| profile.clone())
                .collect();
        }
        if list.is_empty() {
            // No shell was found: Command Prompt, which every Windows has.
            list.push(Profile {
                name: COMMAND_PROMPT.to_owned(),
                command: "cmd.exe".to_owned(),
                folder: Folder::Home,
            });
        }

        let index_of = |name: &str| list.iter().position(|profile| profile.name == name);
        let chosen = config.default_profile.as_ref().and_then(|setting| {
            let index = index_of(&setting.value);
            if index.is_none() {
                let name = &setting.value;
                let message = if profiles.iter().any(|(profile, _)| profile.name == *name) {
                    format!("the profile `{name}` is hidden, so it cannot be the default one")
                } else {
                    let names: Vec<&str> =
                        list.iter().map(|profile| profile.name.as_str()).collect();
                    format!(
                        "there is no profile named `{name}`; the profiles are: {}",
                        names.join(", ")
                    )
                };
                problems.push(Problem::new(Some(setting.at), message));
            }
            index
        });
        let default = chosen
            .or_else(|| index_of(POWERSHELL))
            .or_else(|| index_of(WINDOWS_POWERSHELL))
            .unwrap_or(0);
        (Self { list, default }, problems)
    }

    /// The profiles of the menu, in its order.
    pub fn list(&self) -> &[Profile] {
        &self.list
    }

    pub const fn default_index(&self) -> usize {
        self.default
    }

    pub fn default_profile(&self) -> &Profile {
        &self.list[self.default]
    }
}

/// The command line that starts `command` in `folder`, if it is `wsl.exe`
/// from `system`, as Windows Terminal does (`MangleStartingDirectoryForWSL`):
/// with the folder as `--cd`, since WSL can start in a folder of Linux,
/// which `CreateProcessW` cannot. `None` for other programs, and for a
/// `wsl.exe` that already has `--cd`, or `~` as an argument, which starts in
/// the home folder.
fn wsl_command(command: &str, folder: &Folder, system: &str) -> Option<String> {
    // The program ends at the first space or quote after its first
    // character, which can be the opening quote.
    let start = usize::from(command.starts_with('"'));
    let end = command
        .get(1..)?
        .find([' ', '"'])
        .map_or(command.len(), |at| at + 1);
    let program = command.get(start..end)?;
    let (parent, file) = match program.rsplit_once(['\\', '/']) {
        Some((parent, file)) => (Some(parent), file),
        None => (None, program),
    };
    if !file.eq_ignore_ascii_case("wsl") && !file.eq_ignore_ascii_case("wsl.exe") {
        return None;
    }
    // Another program with that name.
    if parent.is_some_and(|parent| !parent.eq_ignore_ascii_case(system)) {
        return None;
    }
    let arguments = command.get(end + 1..).unwrap_or_default();
    if arguments.contains("--cd") {
        return None;
    }
    if let Some(tilde) = arguments.find('~')
        && arguments[tilde + 1..]
            .chars()
            .next()
            .is_none_or(|c| c == ' ')
    {
        return None;
    }
    let directory = match folder {
        Folder::Home => "~".to_owned(),
        // `wsl.exe --cd` takes these as folders of Linux.
        Folder::Path(path) if path.starts_with("//wsl$") || path.starts_with("//wsl.localhost") => {
            path.replace('/', "\\")
        }
        Folder::Path(path) => path.clone(),
    };
    Some(format!(r#""{program}" --cd "{directory}" {arguments}"#))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Position, ProfileEntry, Setting};

    const SYSTEM: &str = r"C:\WINDOWS\system32";
    const HOME: Option<&str> = Some(r"C:\Users\ana");

    fn profile(name: &str, command: &str) -> Profile {
        Profile {
            name: name.to_owned(),
            command: command.to_owned(),
            folder: Folder::Home,
        }
    }

    /// The profiles of a computer with all the shells.
    fn detected() -> Vec<Profile> {
        vec![
            profile(POWERSHELL, r#""C:\Program Files\PowerShell\7\pwsh.exe""#),
            profile(
                WINDOWS_POWERSHELL,
                r"C:\WINDOWS\system32\WindowsPowerShell\v1.0\powershell.exe",
            ),
            profile(COMMAND_PROMPT, r"C:\WINDOWS\system32\cmd.exe"),
            profile("Ubuntu", r"C:\WINDOWS\system32\wsl.exe -d Ubuntu"),
            profile(GIT_BASH, r#""C:\Program Files\Git\bin\bash.exe" -i -l"#),
        ]
    }

    fn entry(name: &str) -> ProfileEntry {
        ProfileEntry {
            name: name.to_owned(),
            command: None,
            directory: None,
            hidden: false,
        }
    }

    fn names(profiles: &Profiles) -> Vec<&str> {
        profiles
            .list()
            .iter()
            .map(|profile| profile.name.as_str())
            .collect()
    }

    fn default_named(name: &str) -> Option<Setting<String>> {
        Some(Setting {
            value: name.to_owned(),
            at: Position {
                line: 1,
                column: 19,
            },
        })
    }

    fn environment() -> Environment {
        let mut environment = Environment::default();
        environment.set("USERPROFILE", r"C:\Users\ana");
        environment
    }

    #[test]
    fn without_a_configuration_powershell_is_the_default() {
        let (profiles, problems) =
            Profiles::merge(detected(), &Config::default(), &Environment::default());
        assert!(problems.is_empty());
        assert_eq!(profiles.list(), detected());
        assert_eq!(profiles.default_profile().name, POWERSHELL);
        // Without PowerShell, Windows PowerShell; without both, the first.
        let without = detected()[1..].to_vec();
        let (profiles, _) = Profiles::merge(without, &Config::default(), &Environment::default());
        assert_eq!(profiles.default_index(), 0);
        let without = detected()[2..].to_vec();
        let (profiles, _) = Profiles::merge(without, &Config::default(), &Environment::default());
        assert_eq!(profiles.default_profile().name, COMMAND_PROMPT);
        // Without any shell, Command Prompt.
        let (profiles, _) =
            Profiles::merge(Vec::new(), &Config::default(), &Environment::default());
        assert_eq!(profiles.list(), [profile(COMMAND_PROMPT, "cmd.exe")]);
    }

    #[test]
    fn the_configuration_changes_hides_and_adds_profiles() {
        let config = Config {
            default_profile: default_named("Build"),
            profiles: vec![
                ProfileEntry {
                    hidden: true,
                    ..entry(COMMAND_PROMPT)
                },
                ProfileEntry {
                    directory: Some(r"%USERPROFILE%\src".to_owned()),
                    ..entry("Ubuntu")
                },
                ProfileEntry {
                    command: Some(r"pwsh -File %USERPROFILE%\build.ps1".to_owned()),
                    ..entry("Build")
                },
                ProfileEntry {
                    command: Some("pwsh -NoProfile".to_owned()),
                    directory: Some("~".to_owned()),
                    ..entry(POWERSHELL)
                },
                // Neither a command nor a profile of its name: another
                // computer has it.
                ProfileEntry {
                    hidden: true,
                    ..entry("Debian")
                },
            ],
        };
        let (profiles, problems) = Profiles::merge(detected(), &config, &environment());
        assert!(problems.is_empty());
        assert_eq!(
            names(&profiles),
            [POWERSHELL, WINDOWS_POWERSHELL, "Ubuntu", GIT_BASH, "Build"]
        );
        assert_eq!(profiles.list()[0], profile(POWERSHELL, "pwsh -NoProfile"));
        assert_eq!(
            profiles.list()[2].folder,
            Folder::Path(r"C:\Users\ana\src".to_owned())
        );
        assert_eq!(
            profiles.list()[4],
            profile("Build", r"pwsh -File C:\Users\ana\build.ps1")
        );
        assert_eq!(profiles.default_profile().name, "Build");
    }

    #[test]
    fn a_default_profile_that_is_hidden_or_not_there_is_a_problem() {
        let messages = |name: &str| {
            let config = Config {
                default_profile: default_named(name),
                profiles: vec![ProfileEntry {
                    hidden: true,
                    ..entry(COMMAND_PROMPT)
                }],
            };
            // Without PowerShell, the default one is Windows PowerShell.
            let (profiles, problems) =
                Profiles::merge(detected()[1..].to_vec(), &config, &environment());
            assert_eq!(profiles.default_profile().name, WINDOWS_POWERSHELL);
            problems.iter().map(ToString::to_string).collect::<Vec<_>>()
        };
        assert_eq!(
            messages(COMMAND_PROMPT),
            [
                "Line 1, column 19: the profile `Command Prompt` is hidden, so it cannot be the default one"
            ]
        );
        // Names distinguish case.
        assert_eq!(
            messages("ubuntu"),
            ["Line 1, column 19: there is no profile named `ubuntu`; \
                 the profiles are: Windows PowerShell, Ubuntu, Git Bash"]
        );
    }

    #[test]
    fn if_all_the_profiles_are_hidden_they_are_all_shown() {
        let config = Config {
            default_profile: None,
            profiles: detected()
                .iter()
                .map(|profile| ProfileEntry {
                    hidden: true,
                    ..entry(&profile.name)
                })
                .collect(),
        };
        let (profiles, problems) = Profiles::merge(detected(), &config, &environment());
        assert_eq!(
            problems.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["all the profiles are hidden, so meiga shows them all"]
        );
        assert_eq!(profiles.list(), detected());
        assert_eq!(profiles.default_profile().name, POWERSHELL);
    }

    #[test]
    fn profiles_start_in_the_home_folder_or_in_theirs() {
        let launch = profile(COMMAND_PROMPT, "cmd.exe").launch(HOME, SYSTEM);
        assert_eq!(
            launch,
            Launch {
                name: COMMAND_PROMPT.to_owned(),
                command: "cmd.exe".to_owned(),
                folder: HOME.map(str::to_owned),
            }
        );
        let src = Profile {
            folder: Folder::Path(r"C:\src".to_owned()),
            ..profile("src", "cmd.exe")
        };
        assert_eq!(src.launch(HOME, SYSTEM).folder.as_deref(), Some(r"C:\src"));
        assert_eq!(profile("x", "cmd.exe").launch(None, SYSTEM).folder, None);
        // The program of the command line of meiga starts in its folder.
        assert_eq!(
            Launch::command_line(r#""C:\tools\my app.exe" -x"#),
            Launch {
                name: "my app".to_owned(),
                command: r#""C:\tools\my app.exe" -x"#.to_owned(),
                folder: None,
            }
        );
    }

    #[test]
    fn wsl_gets_its_folder_as_cd_as_in_windows_terminal() {
        let wsl = |command: &str, folder: Folder| {
            Profile {
                folder,
                ..profile("wsl", command)
            }
            .launch(HOME, SYSTEM)
        };
        let launch = wsl(r"C:\WINDOWS\system32\wsl.exe -d Ubuntu", Folder::Home);
        assert_eq!(
            launch.command,
            r#""C:\WINDOWS\system32\wsl.exe" --cd "~" -d Ubuntu"#
        );
        assert_eq!(launch.folder.as_deref(), HOME);
        // Without a folder, and quoted, in another case.
        assert_eq!(
            wsl("wsl", Folder::Path(r"C:\src".to_owned())).command,
            r#""wsl" --cd "C:\src" "#
        );
        assert_eq!(
            wsl(r#""c:\windows\SYSTEM32\WSL.EXE" -e bash"#, Folder::Home).command,
            r#""c:\windows\SYSTEM32\WSL.EXE" --cd "~"  -e bash"#
        );
        assert_eq!(
            wsl("wsl.exe", Folder::Path("//wsl$/Ubuntu/home".to_owned())).command,
            r#""wsl.exe" --cd "\\wsl$\Ubuntu\home" "#
        );
        // What stays as it is: other programs, a wsl.exe from another
        // folder, and one with a folder already.
        for command in [
            "cmd.exe /c wsl.exe",
            r"C:\tools\wsl.exe -d Ubuntu",
            "wsl --cd /tmp",
            "wsl ~",
            "wsl ~ -d Ubuntu",
        ] {
            let launch = wsl(command, Folder::Home);
            assert_eq!(launch.command, command);
            assert_eq!(launch.folder.as_deref(), HOME);
        }
        let launch = wsl("wsl --cd /tmp", Folder::Path(r"C:\src".to_owned()));
        assert_eq!(launch.folder.as_deref(), Some(r"C:\src"));
        // A ~ inside an argument is not the home folder.
        assert_eq!(
            wsl("wsl ls ~/src", Folder::Home).command,
            r#""wsl" --cd "~" ls ~/src"#
        );
    }
}
