//! CLI module for ClickDown debug mode
//!
//! Provides command-line interface for headless debugging operations.

pub mod args;
pub mod run;

pub use args::{CliArgs, DebugCommand, DebugOperation};
pub use run::run_cli;
