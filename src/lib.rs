//! ClickDown - A TUI ClickUp client library
//!
//! This library provides the core functionality for the ClickDown TUI application,
//! including API client, models, caching, configuration, and TUI components.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod api;
pub mod models;
pub mod cache;
pub mod config;
pub mod tui;
pub mod cli;
pub mod commands;

pub use tui::app::TuiApp;
pub use api::{ClickUpClient, AuthManager, ClickUpApi, MockClickUpClient};
