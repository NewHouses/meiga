//! The command-line options.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// How to run meiga, without options.
pub const USAGE: &str = "Usage: meiga [--trace-frames <file>] [--bench partial|full|text]

  --trace-frames <file>  Write a line per frame to <file> (CSV).
  --bench partial        Instead of the demo, move a small square without stopping.
  --bench full           Instead of the demo, change the whole window without stopping.
  --bench text           Instead of the demo, fill the whole window with text that
                         changes at every frame.";

/// What to draw instead of the demo, to measure the cost of the frames.
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
    pub bench: Option<BenchMode>,
}

impl Options {
    /// Reads the options from `args`, without the name of the program.
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut options = Options::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--trace-frames") => {
                    let path = args
                        .next()
                        .ok_or("--trace-frames needs the path of a file.")?;
                    options.trace_frames = Some(PathBuf::from(path));
                }
                Some("--bench") => {
                    options.bench = Some(match args.next().as_deref().and_then(OsStr::to_str) {
                        Some("partial") => BenchMode::Partial,
                        Some("full") => BenchMode::Full,
                        Some("text") => BenchMode::Text,
                        _ => return Err("--bench needs 'partial', 'full' or 'text'.".into()),
                    });
                }
                _ => return Err(format!("Unknown option: {}", arg.to_string_lossy())),
            }
        }
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(args.iter().map(OsString::from))
    }

    #[test]
    fn no_options_run_the_demo() {
        assert_eq!(parse(&[]), Ok(Options::default()));
    }

    #[test]
    fn options_take_their_values() {
        assert_eq!(
            parse(&["--bench", "full", "--trace-frames", r"C:\perf\frames.csv"]),
            Ok(Options {
                trace_frames: Some(PathBuf::from(r"C:\perf\frames.csv")),
                bench: Some(BenchMode::Full),
            })
        );
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
    fn wrong_options_are_errors() {
        assert_eq!(parse(&["--fast"]), Err("Unknown option: --fast".into()));
        assert!(parse(&["--trace-frames"]).is_err());
        assert!(parse(&["--bench"]).is_err());
        assert!(parse(&["--bench", "half"]).is_err());
    }
}
