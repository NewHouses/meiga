//! Finds the shells of this computer, as the menu of the profiles does, and
//! reads the environment of this program, as the tabs get it.

// Clippy allows them in test functions (clippy.toml), but not in the helpers
// of an integration test, where a panic is also how a test fails.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use meiga_core::profile::{COMMAND_PROMPT, GIT_BASH, POWERSHELL, WINDOWS_POWERSHELL};
use meiga_win::profiles;

#[test]
fn the_shells_of_windows_are_in_the_folder_of_the_system() {
    let environment = profiles::environment();
    let system = profiles::system_folder().unwrap();
    let found = profiles::detect(&environment, &system);
    let command = |name: &str| {
        found
            .iter()
            .find(|profile| profile.name == name)
            .map(|profile| profile.command.clone())
    };
    assert_eq!(command(COMMAND_PROMPT), Some(format!(r"{system}\cmd.exe")));
    assert_eq!(
        command(WINDOWS_POWERSHELL),
        Some(format!(r"{system}\WindowsPowerShell\v1.0\powershell.exe"))
    );
    if let Some(pwsh) = command(POWERSHELL) {
        assert!(pwsh.trim_end_matches('"').ends_with(r"\pwsh.exe"), "{pwsh}");
    }
    // In the order of the menu, and each name once.
    let places: Vec<usize> = [POWERSHELL, WINDOWS_POWERSHELL, COMMAND_PROMPT, GIT_BASH]
        .iter()
        .filter_map(|name| found.iter().position(|profile| profile.name == *name))
        .collect();
    assert!(places.is_sorted(), "{found:?}");
    let mut names: Vec<&str> = found.iter().map(|profile| profile.name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), found.len(), "{found:?}");
}

#[test]
fn the_environment_is_the_one_of_this_program() {
    let environment = profiles::environment();
    for name in ["PATH", "SystemRoot", "USERPROFILE"] {
        assert_eq!(environment.get(name), std::env::var(name).ok(), "{name}");
    }
    // The names do not distinguish case, as in Windows.
    assert_eq!(environment.get("path"), std::env::var("PATH").ok());
}
