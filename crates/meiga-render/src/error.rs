//! Errors from GDI and DirectWrite.

#![forbid(unsafe_code)]

use std::{fmt, io};

/// Something that the renderer could not do: a call to GDI or DirectWrite
/// that failed, or fonts that are not installed.
#[derive(Debug)]
pub struct Error {
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    /// A function failed, with the reason when Windows gives one.
    Call {
        function: &'static str,
        cause: Option<Cause>,
    },
    /// None of these font families is installed.
    NoFont(String),
}

#[derive(Debug)]
enum Cause {
    /// An error code of Windows, from `GetLastError`.
    Os(io::Error),
    /// The HRESULT that a COM method returned.
    Com(windows_core::Error),
}

impl Error {
    /// Builds the error of `function` from the last error code of the calling
    /// thread (`GetLastError`), so it must be called right after the failure.
    pub(crate) fn last(function: &'static str) -> Self {
        let error = io::Error::last_os_error();
        // Many GDI functions fail without setting an error code, and "The
        // operation completed successfully" would mislead.
        let cause = (error.raw_os_error() != Some(0)).then_some(Cause::Os(error));
        Self::call(function, cause)
    }

    /// The error of `function` when it is not even called, because what it
    /// should allocate does not fit in memory.
    pub(crate) fn too_big(function: &'static str) -> Self {
        Self::call(function, Some(Cause::Os(io::ErrorKind::OutOfMemory.into())))
    }

    /// The error of a COM method, `function`, from its HRESULT.
    pub(crate) fn com(function: &'static str, error: windows_core::Error) -> Self {
        Self::call(function, Some(Cause::Com(error)))
    }

    /// None of `families` is installed.
    pub(crate) fn no_font(families: &[&str]) -> Self {
        Self {
            kind: Kind::NoFont(families.join(", ")),
        }
    }

    const fn call(function: &'static str, cause: Option<Cause>) -> Self {
        Self {
            kind: Kind::Call { function, cause },
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::Call {
                function,
                cause: Some(Cause::Os(error)),
            } => write!(f, "{function} failed: {error}"),
            Kind::Call {
                function,
                cause: Some(Cause::Com(error)),
            } => write!(f, "{function} failed: {error}"),
            Kind::Call {
                function,
                cause: None,
            } => write!(f, "{function} failed"),
            Kind::NoFont(families) => {
                write!(f, "None of these fonts is installed: {families}.")
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_name_the_function_and_the_reason() {
        let com = Error::com(
            "IDWriteFactory::CreateFontFace",
            windows_core::Error::from_hresult(windows_core::HRESULT(0x8007_000E_u32 as i32)),
        );
        let text = com.to_string();
        assert!(text.starts_with("IDWriteFactory::CreateFontFace failed: "));
        assert!(text.ends_with("(0x8007000E)"), "{text}");
        assert_eq!(
            Error::no_font(&["Cascadia Mono", "Consolas"]).to_string(),
            "None of these fonts is installed: Cascadia Mono, Consolas."
        );
        assert_eq!(Error::call("BitBlt", None).to_string(), "BitBlt failed");
    }
}
