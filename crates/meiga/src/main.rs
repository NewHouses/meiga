//! meiga: a lightweight terminal for Windows.

// Release builds are GUI applications without a console window. Debug builds
// keep the console so that diagnostics are visible during development.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

use std::process::ExitCode;

use meiga_core::color::Rgb;
use meiga_win::window::{self, Window, WindowOptions};

/// Colour of the empty window.
const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let text = format!("meiga stopped because of an unexpected error.\n\n{error}");
            // Release builds have no console, so the message box is the only
            // way to tell the user.
            eprintln!("{text}");
            window::show_error("meiga", &text);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), meiga_win::Error> {
    let window = Window::create(&WindowOptions {
        title: "meiga",
        background: BACKGROUND,
    })?;
    window.show()?;
    window::run_message_loop()
}
