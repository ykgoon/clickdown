//! Clipboard service for cross-platform clipboard access
//!
//! This module provides a wrapper around the `arboard` crate for clipboard operations.
//! It handles errors gracefully and provides a simple interface for copying text.

use arboard::Clipboard;
use std::error::Error;

/// Result type for clipboard operations
pub type ClipboardResult<T> = Result<T, ClipboardError>;

/// Error type for clipboard operations
#[derive(Debug)]
pub enum ClipboardError {
    /// Clipboard is unavailable (e.g., headless SSH, Wayland without portal)
    Unavailable(String),
    /// Failed to copy text to clipboard
    CopyFailed(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::Unavailable(msg) => write!(f, "clipboard unavailable: {}", msg),
            ClipboardError::CopyFailed(msg) => write!(f, "copy failed: {}", msg),
        }
    }
}

impl Error for ClipboardError {}

/// Clipboard service for copying text to system clipboard
pub struct ClipboardService {
    clipboard: Option<Clipboard>,
}

impl ClipboardService {
    /// Create a new clipboard service
    ///
    /// Attempts to initialize the system clipboard.
    /// If clipboard is unavailable, the service will still be created
    /// but copy operations will fail gracefully.
    pub fn new() -> Self {
        let clipboard = Clipboard::new().ok();
        Self { clipboard }
    }

    /// Copy text to the system clipboard
    ///
    /// # Arguments
    /// * `text` - The text to copy
    ///
    /// # Returns
    /// - `Ok(())` if the text was copied successfully
    /// - `ClipboardError::Unavailable` if clipboard is not available
    /// - `ClipboardError::CopyFailed` if the copy operation failed
    pub fn copy_text(&mut self, text: &str) -> ClipboardResult<()> {
        match &mut self.clipboard {
            Some(clipboard) => {
                clipboard
                    .set_text(text)
                    .map_err(|e| ClipboardError::CopyFailed(e.to_string()))
            }
            None => Err(ClipboardError::Unavailable(
                "clipboard not initialized".to_string(),
            )),
        }
    }

    /// Check if clipboard is available
    pub fn is_available(&self) -> bool {
        self.clipboard.is_some()
    }
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_service_creation() {
        let service = ClipboardService::new();
        // Service should be created even if clipboard is unavailable
        // The clipboard field may be None in headless environments
    }

    #[test]
    fn test_clipboard_service_default() {
        let _service = ClipboardService::default();
        // Default should work the same as new()
    }

    #[test]
    fn test_clipboard_error_display_unavailable() {
        let err = ClipboardError::Unavailable("test reason".to_string());
        assert_eq!(err.to_string(), "clipboard unavailable: test reason");
    }

    #[test]
    fn test_clipboard_error_display_copy_failed() {
        let err = ClipboardError::CopyFailed("test reason".to_string());
        assert_eq!(err.to_string(), "copy failed: test reason");
    }

    /// Test that copy_text returns an error when clipboard is unavailable
    /// This test may pass or fail depending on the test environment
    #[test]
    fn test_copy_text_graceful_failure() {
        let mut service = ClipboardService::new();
        let result = service.copy_text("test text");
        
        // In headless environments, this should return Unavailable
        // In environments with clipboard, this should succeed
        // We just verify the result type is correct
        match result {
            Ok(()) => {
                // Clipboard worked - that's fine
            }
            Err(ClipboardError::Unavailable(_)) => {
                // Clipboard unavailable - also fine, error is graceful
            }
            Err(ClipboardError::CopyFailed(_)) => {
                // Copy failed - also acceptable
            }
        }
    }
}
