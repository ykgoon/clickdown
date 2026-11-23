//! Configuration and cache locations.
//!
//! These are plain functions: there is no `ConfigManager` state to carry, and
//! a unit struct with only associated functions was pretending otherwise.

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Directory holding the cache database, created if missing.
pub fn cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .context("Failed to get cache directory")?
        .join("clickdown");

    std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

    Ok(cache_dir)
}

/// Path of the SQLite cache database.
pub fn database_path() -> Result<PathBuf> {
    Ok(cache_dir()?.join("cache.db"))
}
