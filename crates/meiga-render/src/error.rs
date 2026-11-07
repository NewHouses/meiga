//! Errors from GDI calls.

#![forbid(unsafe_code)]

use std::{fmt, io};

/// A GDI call that failed.
#[derive(Debug)]
pub struct Error {
    function: &'static str,
    os_error: Option<io::Error>,
}

impl Error {
    /// Builds the error of `function` from the last error code of the calling
    /// thread (`GetLastError`), so it must be called right after the failure.
    pub(crate) fn last(function: &'static str) -> Self {
        let error = io::Error::last_os_error();
        // Many GDI functions fail without setting an error code, and "The
        // operation completed successfully" would mislead.
        let os_error = (error.raw_os_error() != Some(0)).then_some(error);
        Self { function, os_error }
    }

    /// The error of `function` when it is not even called, because what it
    /// should allocate does not fit in memory.
    pub(crate) fn too_big(function: &'static str) -> Self {
        Self {
            function,
            os_error: Some(io::ErrorKind::OutOfMemory.into()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.os_error {
            Some(error) => write!(f, "{} failed: {error}", self.function),
            None => write!(f, "{} failed", self.function),
        }
    }
}

impl std::error::Error for Error {}
