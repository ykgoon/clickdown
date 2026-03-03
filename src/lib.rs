//! ClickDown - A TUI ClickUp client library
//!
//! This library provides the core functionality for the ClickDown TUI application,
//! including API client, models, caching, configuration, and TUI components.

pub mod api;
pub mod cache;
pub mod cli;
pub mod commands;
pub mod config;
pub mod models;
pub mod tui;
pub mod utils;

pub use api::{AuthManager, ClickUpApi, ClickUpClient, MockClickUpClient};
pub use tui::app::TuiApp;
