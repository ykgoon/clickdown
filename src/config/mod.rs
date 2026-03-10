//! Configuration management module

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Configuration manager - provides utility functions for config/cache paths
pub struct ConfigManager;

impl ConfigManager {
    /// Get the cache directory path
    pub fn cache_dir() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?
            .join("clickdown");

        std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(cache_dir)
    }

    /// Get the database path
    pub fn database_path() -> Result<PathBuf> {
        Ok(Self::cache_dir()?.join("cache.db"))
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        ConfigManager
    }
}
