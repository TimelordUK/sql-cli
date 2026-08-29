//! User interface layer
//!
//! This module contains the main TUI application and related UI components.
//!
//! Nothing here may write to stdout/stderr directly. The TUI owns the terminal
//! via the alternate screen, and a stray `println!`/`eprintln!` injects a line
//! that scrolls the display out from under ratatui's diff - leaving artifacts
//! until something forces a full repaint. Use `tracing` instead; those records
//! reach the log file and the F5 debug view.
#![deny(clippy::print_stdout, clippy::print_stderr)]

pub mod behaviors;
pub mod debug;
pub mod enhanced_tui;
pub mod input;
pub mod key_handling;
pub mod operations;
pub mod rendering;
pub mod search;
pub mod state;
pub mod traits;
pub mod tui_app;
pub mod utils;
pub mod viewport;
pub mod viewport_manager;
