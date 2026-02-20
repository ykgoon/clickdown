//! Configuration management module

pub mod storage;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API token for ClickUp
    #[serde(default)]
    pub api_token: Option<String>,

    /// Last selected workspace ID
    #[serde(default)]
    pub last_workspace_id: Option<String>,

    /// Last selected space ID
    #[serde(default)]
    pub last_space_id: Option<String>,

    /// Window dimensions
    #[serde(default = "default_window_width")]
    pub window_width: u32,

    #[serde(default = "default_window_height")]
    pub window_height: u32,

    /// UI preferences
    #[serde(default)]
    pub ui: UiConfig,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    /// Sidebar width
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,

    /// Task detail panel width
    #[serde(default = "default_detail_width")]
    pub detail_width: u32,

    /// Theme (light/dark)
    #[serde(default)]
    pub theme: Theme,

    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: f32,
}

/// Color theme
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

fn default_window_width() -> u32 { 1200 }
fn default_window_height() -> u32 { 800 }
fn default_sidebar_width() -> u32 { 280 }
fn default_detail_width() -> u32 { 400 }
fn default_font_size() -> f32 { 14.0 }

impl Default for Config {
    fn default() -> Self {
        Self {
            api_token: None,
            last_workspace_id: None,
            last_space_id: None,
            window_width: default_window_width(),
            window_height: default_window_height(),
            ui: UiConfig::default(),
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config_path: PathBuf,
    config: Config,
}

impl ConfigManager {
    /// Create a new ConfigManager
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("clickdown");

        let config_path = config_dir.join("config.toml");

        // Create config directory if needed
        std::fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        // Load or create config
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            Config::default()
        };

        Ok(Self { config_path, config })
    }

    /// Load configuration from file
    fn load_config(path: &PathBuf) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read config file")?;

        toml::from_str(&content)
            .context("Failed to parse config file")
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;

        std::fs::write(&self.config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Get the configuration
    pub fn get(&self) -> &Config {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn get_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Set the API token
    pub fn set_api_token(&mut self, token: Option<String>) -> Result<()> {
        self.config.api_token = token;
        self.save()
    }

    /// Get the API token
    pub fn api_token(&self) -> Option<&String> {
        self.config.api_token.as_ref()
    }

    /// Set the last workspace ID
    pub fn set_last_workspace(&mut self, id: Option<String>) -> Result<()> {
        self.config.last_workspace_id = id;
        self.save()
    }

    /// Set the last space ID
    pub fn set_last_space(&mut self, id: Option<String>) -> Result<()> {
        self.config.last_space_id = id;
        self.save()
    }

    /// Get the cache directory path
    pub fn cache_dir() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?
            .join("clickdown");

        std::fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        Ok(cache_dir)
    }

    /// Get the database path
    pub fn database_path() -> Result<PathBuf> {
        Ok(Self::cache_dir()?.join("cache.db"))
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigManager")
    }
}
