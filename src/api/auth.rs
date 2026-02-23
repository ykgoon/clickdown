//! Authentication management for ClickUp API

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Manages API token storage and retrieval
pub struct AuthManager {
    config_dir: PathBuf,
}

impl AuthManager {
    /// Create a new AuthManager with the default config directory
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("clickdown");

        Ok(Self { config_dir })
    }

    /// Get the path to the token file
    fn token_path(&self) -> PathBuf {
        self.config_dir.join("token")
    }

    /// Save the API token
    pub fn save_token(&self, token: &str) -> Result<()> {
        // Create config directory if it doesn't exist
        std::fs::create_dir_all(&self.config_dir)
            .context("Failed to create config directory")?;

        // Write token to file
        std::fs::write(self.token_path(), token)
            .context("Failed to save token")?;

        // Set restrictive permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(self.token_path())?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(self.token_path(), perms)?;
        }

        Ok(())
    }

    /// Load the stored API token
    pub fn load_token(&self) -> Result<Option<String>> {
        let token_path = self.token_path();

        if !token_path.exists() {
            return Ok(None);
        }

        let token = std::fs::read_to_string(&token_path)
            .context("Failed to read token file")?;

        Ok(Some(token.trim().to_string()))
    }

    /// Remove the stored token
    pub fn clear_token(&self) -> Result<()> {
        let token_path = self.token_path();

        if token_path.exists() {
            std::fs::remove_file(&token_path)
                .context("Failed to remove token file")?;
        }

        Ok(())
    }

    /// Check if a token is stored
    pub fn has_token(&self) -> bool {
        self.token_path().exists()
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        // Return an AuthManager with default paths if initialization fails
        // This allows the app to start even if config directory can't be created
        use tracing::warn;
        match Self::new() {
            Ok(manager) => manager,
            Err(e) => {
                warn!("Failed to initialize AuthManager: {}", e);
                AuthManager {
                    config_dir: PathBuf::new(),
                }
            }
        }
    }
}
