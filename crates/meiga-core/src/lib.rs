//! Platform-independent core of meiga.
//!
//! This crate holds the logic that does not depend on Windows: the window and
//! tab model, the terminal facade, input encoding, search and configuration.
//! It must stay free of `unsafe` code and of platform APIs, so that it is easy
//! to test and fast to compile.

#![forbid(unsafe_code)]

pub mod color;
pub mod config;
pub mod damage;
pub mod environment;
pub mod geometry;
pub mod input;
pub mod model;
pub mod mouse;
pub mod pacing;
pub mod paste;
pub mod profile;
pub mod shell;
pub mod shortcut;
pub mod sort;
pub mod term;
pub mod text;
pub mod ui;
