//! The command-line options.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use meiga_core::shell;

/// How to run meiga.
pub const USAGE: &str =
    "Usage: meiga [--trace-frames <file>] [--marks] [--] [<program> [<arguments>]]
       meiga [--trace-frames <file>] [--marks] --bench partial|full|text

  <program>              The program to run in the terminal, and its arguments.
                         Without it, PowerShell 7 (pwsh) or, if it is not
                         installed, Windows PowerShell.
  --                     The end of the options: what follows is the program.
  --trace-frames <file>  Write a line per frame to <file> (CSV).
  --marks                Draw a magenta border around the areas that each frame
                         presents.
  --bench partial        Instead of a terminal, move a small square without stopping.
  --bench full           Instead of a terminal, change the whole window without stopping.
  --bench text           Instead of a terminal, fill the whole window with text that
                         changes at every frame.";

/// What to draw instead of a terminal, to measure the cost of the frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BenchMode {
    /// A frame that presents a small part of the window.
    Partial,
    /// A frame that presents the whole window.
    Full,
    /// A frame that composes the whole window full of text, and presents
    /// it.
    Text,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// The file of the frame log.
    pub trace_frames: Option<PathBuf>,
    /// Whether the window shows the areas that each frame presents.
    pub marks: bool,
    pub bench: Option<BenchMode>,
    /// The command line of the program, quoted for Windows, or `None` for
    /// the default shell.
    pub command: Option<String>,
}

impl Options {
    /// Reads the options from `args`, without the name of the program. The
    /// first argument that is not an option, or the one after `--`, is the
    /// program: it and the ones after it are its command line.
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut options = Options::default();
        let mut args = args.into_iter();
        let mut program = None;
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--trace-frames") => {
                    let path = args
                        .next()
                        .ok_or("--trace-frames needs the path of a file.")?;
                    options.trace_frames = Some(PathBuf::from(path));
                }
                Some("--marks") => options.marks = true,
                Some("--bench") => {
                    options.bench = Some(match args.next().as_deref().and_then(OsStr::to_str) {
                        Some("partial") => BenchMode::Partial,
                        Some("full") => BenchMode::Full,
                        Some("text") => BenchMode::Text,
                        _ => return Err("--bench needs 'partial', 'full' or 'text'.".into()),
                    });
                }
                Some("--") => {
                    program = args.next();
                    break;
                }
                Some(option) if option.starts_with('-') => {
                    return Err(format!("Unknown option: {option}"));
                }
                _ => {
                    program = Some(arg);
                    break;
                }
            }
        }
        let Some(program) = program else {
            return Ok(options);
        };
        if options.bench.is_some() {
            return Err("--bench does not run a program.".into());
        }
        let text = |arg: OsString| {
            arg.into_string().map_err(|arg| {
                format!(
                    "The command line can only have valid Unicode: {}",
                    arg.to_string_lossy()
                )
            })
        };
        let program = text(program)?;
        let arguments = args.map(text).collect::<Result<Vec<_>, _>>()?;
        let command = shell::command_line(&program, &arguments)
            .ok_or_else(|| format!("The name of a program cannot have quotes: {program}"))?;
        options.command = Some(command);
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(args.iter().map(OsString::from))
    }

    fn command(args: &[&str]) -> Option<String> {
        parse(args).unwrap().command
    }

    #[test]
    fn no_options_run_the_default_shell() {
        assert_eq!(parse(&[]), Ok(Options::default()));
        assert_eq!(parse(&["--"]), Ok(Options::default()));
    }

    #[test]
    fn options_take_their_values() {
        assert_eq!(
            parse(&["--bench", "full", "--trace-frames", r"C:\perf\frames.csv"]),
            Ok(Options {
                trace_frames: Some(PathBuf::from(r"C:\perf\frames.csv")),
                marks: false,
                bench: Some(BenchMode::Full),
                command: None,
            })
        );
        assert!(parse(&["--marks", "cmd"]).unwrap().marks);
        assert_eq!(
            parse(&["--bench", "partial"]).unwrap().bench,
            Some(BenchMode::Partial)
        );
        assert_eq!(
            parse(&["--bench", "text"]).unwrap().bench,
            Some(BenchMode::Text)
        );
    }

    #[test]
    fn the_program_and_its_arguments_are_the_command_line() {
        let options = parse(&[
            "--trace-frames",
            "frames.csv",
            "pwsh",
            "-NoProfile",
            "-File",
            r"C:\my scripts\spinner.ps1",
        ])
        .unwrap();
        assert_eq!(options.trace_frames, Some(PathBuf::from("frames.csv")));
        assert_eq!(
            options.command.as_deref(),
            Some(r#"pwsh -NoProfile -File "C:\my scripts\spinner.ps1""#)
        );
        // After the program, the options are its own.
        assert_eq!(
            command(&["cmd", "--bench", "full"]).as_deref(),
            Some("cmd --bench full")
        );
        // After --, also a program that starts with a hyphen.
        assert_eq!(command(&["--", "-weird"]).as_deref(), Some("-weird"));
    }

    #[test]
    fn wrong_options_are_errors() {
        assert_eq!(parse(&["--fast"]), Err("Unknown option: --fast".into()));
        assert_eq!(
            parse(&["-NoProfile", "pwsh"]),
            Err("Unknown option: -NoProfile".into())
        );
        assert!(parse(&["--trace-frames"]).is_err());
        assert!(parse(&["--bench"]).is_err());
        assert!(parse(&["--bench", "half"]).is_err());
        assert_eq!(
            parse(&["--bench", "full", "pwsh"]),
            Err("--bench does not run a program.".into())
        );
        assert!(parse(&[r#"C:\a"b.exe"#]).is_err());
    }
}
