//! ClickDown - A fast ClickUp desktop client library
//!
//! This library provides the core functionality for the ClickDown application,
//! including API client, models, UI components, and application state management.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod api;
pub mod models;
pub mod cache;
pub mod config;
pub mod ui;
pub mod app;

pub use app::ClickDown;
pub use api::{ClickUpClient, AuthManager, ClickUpApi, MockClickUpClient};
