//! The shells that are installed, found without starting any program (ADR
//! 0022): in the folders where their installers put them, and in the
//! registry, where WSL keeps its distributions and Git for Windows its
//! folder. Also the environment of meiga, and the folder of the system,
//! which the profiles need.
//!
//! The profiles are, in this order, when their programs are there:
//!
//! - PowerShell: the `pwsh.exe` of the newest version that has one in
//!   `%ProgramFiles%\PowerShell`, the one of the Microsoft Store, the one of
//!   scoop, the global tool of .NET, the newest preview in
//!   `%ProgramFiles%\PowerShell`, and the preview of the Microsoft Store;
//! - Windows PowerShell and Command Prompt, in the folder of the system;
//! - each distribution of WSL, the default one first and then the others
//!   by name, if the folder of the system has `wsl.exe`;
//! - Git Bash, in the folder of Git for Windows, or in the one of scoop.
//!
//! A file is there if `GetFileAttributesW` finds it: the programs of the
//! Microsoft Store are aliases, links that only Windows can open.

use std::{fs, ptr, slice};

use meiga_core::environment::Environment;
use meiga_core::profile::{
    COMMAND_PROMPT, Folder, GIT_BASH, POWERSHELL, Profile, WINDOWS_POWERSHELL,
};
use meiga_core::shell;
use meiga_core::sort::insertion_sort_by;
use windows_sys::Win32::Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DIRECTORY, GetFileAttributesW, INVALID_FILE_ATTRIBUTES,
};
use windows_sys::Win32::System::Environment::{FreeEnvironmentStringsW, GetEnvironmentStringsW};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_ENUMERATE_SUB_KEYS, RRF_RT_REG_DWORD,
    RRF_RT_REG_SZ, RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW,
};
use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;

use crate::wide;

/// The key of the distributions of WSL, under the one of the user.
const LXSS: &str = r"Software\Microsoft\Windows\CurrentVersion\Lxss";

/// The state of a distribution that is installed, and not being installed
/// or removed.
const INSTALLED: u32 = 1;

/// The distributions of WSL that run the engines of Docker Desktop and of
/// Rancher Desktop, not shells, as Windows Terminal leaves out.
const ENGINES: [&str; 2] = ["docker-desktop", "rancher-desktop"];

/// The longest name of a key of the registry, with its null character.
const KEY_NAME: usize = 256;

/// The environment of meiga, as Windows gives it to the programs that it
/// starts.
pub fn environment() -> Environment {
    // SAFETY: it has no preconditions, and returns null if it fails.
    let block = unsafe { GetEnvironmentStringsW() };
    if block.is_null() {
        return Environment::default();
    }
    // The block is a series of null-terminated strings that ends with an
    // empty one.
    let mut end = 0;
    // SAFETY: each string ends with a null character, and the block with an
    // empty string, so the reads stop at its last character.
    unsafe {
        while *block.add(end) != 0 {
            while *block.add(end) != 0 {
                end += 1;
            }
            end += 1;
        }
    }
    // SAFETY: `end` is the last character of the block, which stays valid
    // until it is freed below.
    let environment = Environment::from_block(unsafe { slice::from_raw_parts(block, end + 1) });
    // SAFETY: the block came from GetEnvironmentStringsW, and is not used
    // after this.
    unsafe { FreeEnvironmentStringsW(block) };
    environment
}

/// The folder of the system, as `C:\WINDOWS\system32`, where cmd.exe and
/// wsl.exe are.
pub fn system_folder() -> Option<String> {
    let mut buffer = [0_u16; 261];
    // SAFETY: the buffer has room for the number of characters that it says.
    let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };
    // 0 if it fails, and more than the buffer if it does not fit, which no
    // folder of the system needs.
    let length = usize::try_from(length).ok()?;
    (1..buffer.len())
        .contains(&length)
        .then(|| String::from_utf16_lossy(&buffer[..length]))
}

/// The profiles of the shells that are installed, in the order of the menu,
/// with `environment` for the folders of the user and of the programs, and
/// `system` as the folder of the system. Each name only once: a
/// distribution of WSL with the name of another profile is left out.
pub fn detect(environment: &Environment, system: &str) -> Vec<Profile> {
    let mut profiles: Vec<Profile> = Vec::new();
    let mut add = |name: &str, program: &str, args: &[&str]| {
        if profiles.iter().any(|profile| profile.name == name) {
            return;
        }
        // Only a program with a quote has no command line, and no file
        // has one.
        if let Some(command) = shell::command_line(program, args) {
            profiles.push(Profile {
                name: name.to_owned(),
                command,
                folder: Folder::Home,
            });
        }
    };
    if let Some(pwsh) = powershell(environment) {
        add(POWERSHELL, &pwsh, &[]);
    }
    let windows_powershell = format!(r"{system}\WindowsPowerShell\v1.0\powershell.exe");
    if is_file(&windows_powershell) {
        add(WINDOWS_POWERSHELL, &windows_powershell, &[]);
    }
    let cmd = format!(r"{system}\cmd.exe");
    if is_file(&cmd) {
        add(COMMAND_PROMPT, &cmd, &[]);
    }
    let wsl = format!(r"{system}\wsl.exe");
    if is_file(&wsl) {
        for name in distributions() {
            add(&name, &wsl, &["-d", &name]);
        }
    }
    if let Some(bash) = git_bash(environment) {
        add(GIT_BASH, &bash, &["-i", "-l"]);
    }
    profiles
}

/// The first PowerShell 7 that is installed, in the order of the module.
fn powershell(environment: &Environment) -> Option<String> {
    let installed = environment
        .get("ProgramFiles")
        .map(|folder| format!(r"{folder}\PowerShell"));
    let versions = installed.as_deref().map(subfolders).unwrap_or_default();
    let in_versions = |preview| {
        let installed = installed.as_deref().unwrap_or_default();
        newest_first(&versions, preview)
            .into_iter()
            .map(move |version| format!(r"{installed}\{version}\pwsh.exe"))
    };
    let local = environment.get("LOCALAPPDATA");
    let store = |package: &str| {
        local
            .as_deref()
            .map(|local| format!(r"{local}\Microsoft\WindowsApps\{package}\pwsh.exe"))
    };
    let user = environment.get("USERPROFILE");
    let in_user = |path: &str| user.as_deref().map(|user| format!(r"{user}\{path}"));
    in_versions(false)
        .chain(store("Microsoft.PowerShell_8wekyb3d8bbwe"))
        .chain(in_user(r"scoop\apps\pwsh\current\pwsh.exe"))
        .chain(in_user(r".dotnet\tools\pwsh.exe"))
        .chain(in_versions(true))
        .chain(store("Microsoft.PowerShellPreview_8wekyb3d8bbwe"))
        .find(|path| is_file(path))
}

/// The folders of the versions of PowerShell in `folders`, the newest
/// first: the ones named with a number, as `7`, or, if `preview`, with a
/// number and `-preview`, as `7-preview`.
fn newest_first(folders: &[String], preview: bool) -> Vec<&str> {
    let mut versions: Vec<(u32, &str)> = folders
        .iter()
        .filter_map(|folder| {
            let number = if preview {
                folder.strip_suffix("-preview")?
            } else {
                folder
            };
            let all_digits = !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit());
            all_digits
                .then(|| number.parse().ok())
                .flatten()
                .map(|version| (version, folder.as_str()))
        })
        .collect();
    insertion_sort_by(&mut versions, |a, b| b.cmp(a));
    versions.into_iter().map(|(_, folder)| folder).collect()
}

/// The names of the folders in `folder`, or none if it cannot be read.
fn subfolders(folder: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

/// A distribution of WSL, as the registry has it.
struct Distribution {
    /// Its key, a GUID, which the default one is named by.
    key: String,
    name: String,
    state: u32,
}

/// The names of the distributions of WSL of the user, in the order of the
/// menu.
fn distributions() -> Vec<String> {
    let default = string_value(HKEY_CURRENT_USER, LXSS, "DefaultDistribution");
    let found = subkeys(HKEY_CURRENT_USER, LXSS)
        .into_iter()
        .filter_map(|key| {
            let path = format!(r"{LXSS}\{key}");
            let name = string_value(HKEY_CURRENT_USER, &path, "DistributionName")?;
            let state = dword_value(HKEY_CURRENT_USER, &path, "State")?;
            Some(Distribution { key, name, state })
        })
        .collect();
    in_order(found, default.as_deref())
}

/// The names of `distributions` in the order of the menu: the default one,
/// whose key is `default`, first, and then the others by name, without
/// distinguishing case; without the ones that are not installed, nor the
/// engines of Docker Desktop and Rancher Desktop.
fn in_order(mut distributions: Vec<Distribution>, default: Option<&str>) -> Vec<String> {
    distributions.retain(|distribution| {
        distribution.state == INSTALLED
            && !ENGINES.iter().any(|engine| {
                distribution
                    .name
                    .get(..engine.len())
                    .is_some_and(|start| start.eq_ignore_ascii_case(engine))
            })
    });
    let is_default = |distribution: &Distribution| {
        default.is_some_and(|key| key.eq_ignore_ascii_case(&distribution.key))
    };
    fn lowercase(name: &str) -> impl Iterator<Item = char> + '_ {
        name.chars().flat_map(char::to_lowercase)
    }
    insertion_sort_by(&mut distributions, |a, b| {
        is_default(b)
            .cmp(&is_default(a))
            .then_with(|| lowercase(&a.name).cmp(lowercase(&b.name)))
    });
    distributions
        .into_iter()
        .map(|distribution| distribution.name)
        .collect()
}

/// The `bash.exe` of Git for Windows, from the folder where the registry
/// says that it is installed, for all the users or for this one, or in the
/// folder of scoop.
fn git_bash(environment: &Environment) -> Option<String> {
    [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER]
        .into_iter()
        .filter_map(|root| string_value(root, r"SOFTWARE\GitForWindows", "InstallPath"))
        .map(|folder| format!(r"{}\bin\bash.exe", folder.trim_end_matches('\\')))
        .chain(
            environment
                .get("USERPROFILE")
                .map(|user| format!(r"{user}\scoop\apps\git\current\bin\bash.exe")),
        )
        .find(|path| is_file(path))
}

/// Whether `path` is a file that Windows can find, also an alias of a
/// program of the Microsoft Store.
fn is_file(path: &str) -> bool {
    let path = wide(path);
    // SAFETY: the path is null-terminated.
    let attributes = unsafe { GetFileAttributesW(path.as_ptr()) };
    attributes != INVALID_FILE_ATTRIBUTES && attributes & FILE_ATTRIBUTE_DIRECTORY == 0
}

/// The text of the value `value` of the key `path` under `root`, if it is a
/// string, with the variables that it may have expanded.
fn string_value(root: HKEY, path: &str, value: &str) -> Option<String> {
    let (path, value) = (wide(path), wide(value));
    let mut buffer: Vec<u16> = Vec::new();
    // The first time asks for the size, and the others read it: again if the
    // value grew meanwhile, or is longer once its variables are expanded.
    for _ in 0..4 {
        let mut size = u32::try_from(buffer.len() * 2).ok()?;
        let data = if buffer.is_empty() {
            ptr::null_mut()
        } else {
            buffer.as_mut_ptr().cast()
        };
        // SAFETY: both names are null-terminated, and `data` is null, to ask
        // for the size, or has the `size` bytes of `buffer`. RRF_RT_REG_SZ
        // also takes the values with variables, expanded.
        let result = unsafe {
            RegGetValueW(
                root,
                path.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                ptr::null_mut(),
                data,
                &mut size,
            )
        };
        match result {
            ERROR_SUCCESS if !buffer.is_empty() => {
                let text = &buffer[..(size as usize / 2).min(buffer.len())];
                let end = text
                    .iter()
                    .position(|&unit| unit == 0)
                    .unwrap_or(text.len());
                return Some(String::from_utf16_lossy(&text[..end]));
            }
            ERROR_SUCCESS | ERROR_MORE_DATA => buffer = vec![0; (size as usize).div_ceil(2).max(1)],
            _ => return None,
        }
    }
    None
}

/// The number of the value `value` of the key `path` under `root`, if it is
/// a `REG_DWORD`.
fn dword_value(root: HKEY, path: &str, value: &str) -> Option<u32> {
    let (path, value) = (wide(path), wide(value));
    let mut data = 0_u32;
    let mut size = size_of::<u32>() as u32;
    // SAFETY: both names are null-terminated, and `data` has the `size`
    // bytes of a REG_DWORD.
    let result = unsafe {
        RegGetValueW(
            root,
            path.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            ptr::null_mut(),
            (&raw mut data).cast(),
            &mut size,
        )
    };
    (result == ERROR_SUCCESS).then_some(data)
}

/// An open key of the registry, which closes when it is dropped.
struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        // SAFETY: the key was opened by RegOpenKeyExW, and is closed only
        // here.
        unsafe { RegCloseKey(self.0) };
    }
}

/// The names of the keys in the key `path` under `root`.
fn subkeys(root: HKEY, path: &str) -> Vec<String> {
    let path = wide(path);
    let mut handle: HKEY = ptr::null_mut();
    // SAFETY: the name is null-terminated, and `handle` is valid for the
    // function to fill in.
    let opened =
        unsafe { RegOpenKeyExW(root, path.as_ptr(), 0, KEY_ENUMERATE_SUB_KEYS, &mut handle) };
    if opened != ERROR_SUCCESS {
        return Vec::new();
    }
    let key = Key(handle);
    let mut names = Vec::new();
    let mut name = [0_u16; KEY_NAME];
    for index in 0..u32::MAX {
        let mut length = KEY_NAME as u32;
        // SAFETY: the key is open, and `name` has room for the `length`
        // characters that it says, as the longest name of a key needs.
        let result = unsafe {
            RegEnumKeyExW(
                key.0,
                index,
                name.as_mut_ptr(),
                &mut length,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        match result {
            ERROR_SUCCESS => {
                let length = (length as usize).min(KEY_NAME);
                names.push(String::from_utf16_lossy(&name[..length]));
            }
            // ERROR_NO_MORE_ITEMS, or a key that changed meanwhile: what
            // was read is enough.
            _ => break,
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_version_of_powershell_comes_first() {
        let folders = [
            "6",
            "7",
            "7-preview",
            "10",
            "8-preview",
            "-preview",
            "7.1",
            "x",
            "",
        ]
        .map(String::from);
        assert_eq!(newest_first(&folders, false), ["10", "7", "6"]);
        assert_eq!(newest_first(&folders, true), ["8-preview", "7-preview"]);
    }

    #[test]
    fn the_default_distribution_comes_first_and_the_others_by_name() {
        let distribution = |key: &str, name: &str, state| Distribution {
            key: key.to_owned(),
            name: name.to_owned(),
            state,
        };
        let found = || {
            vec![
                distribution("{a}", "kali", INSTALLED),
                distribution("{b}", "Ubuntu", INSTALLED),
                distribution("{c}", "Docker-Desktop-Data", INSTALLED),
                distribution("{d}", "Debian", INSTALLED),
                distribution("{e}", "rancher-desktop", INSTALLED),
                distribution("{f}", "Arch", 3),
            ]
        };
        assert_eq!(in_order(found(), Some("{B}")), ["Ubuntu", "Debian", "kali"]);
        assert_eq!(in_order(found(), None), ["Debian", "kali", "Ubuntu"]);
        // A default one that is not installed, or not there.
        assert_eq!(in_order(found(), Some("{f}")), ["Debian", "kali", "Ubuntu"]);
    }
}
