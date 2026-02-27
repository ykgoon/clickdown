//! URL generation utilities for ClickUp elements
//!
//! This module provides functionality to generate ClickUp web app URLs
//! for various element types (workspaces, spaces, folders, lists, tasks,
//! comments, and documents).
//!
//! # URL Patterns
//!
//! ClickUp uses consistent URL patterns:
//! - Workspace: `https://app.clickup.com/{workspace_id}`
//! - Space: `https://app.clickup.com/{workspace_id}/s/{space_id}`
//! - Folder: `https://app.clickup.com/{workspace_id}/f/{folder_id}`
//! - List: `https://app.clickup.com/{workspace_id}/l/{list_id}`
//! - Task: `https://app.clickup.com/{workspace_id}/l/{list_id}/t/{task_id}`
//! - Comment: `https://app.clickup.com/{workspace_id}/l/{list_id}/t/{task_id}/comment/{comment_id}`
//! - Document: `https://app.clickup.com/{workspace_id}/d/{doc_id}`

/// Base URL for ClickUp web app
const CLICKUP_BASE_URL: &str = "https://app.clickup.com";

/// Result type for URL generation operations
pub type UrlResult<T> = Result<T, UrlError>;

/// Error type for URL generation failures
#[derive(Debug, Clone, PartialEq)]
pub enum UrlError {
    /// Missing required workspace ID
    MissingWorkspace,
    /// Missing required list ID
    MissingList,
    /// Missing required task ID
    MissingTask,
    /// Missing required comment ID
    MissingComment,
    /// Missing required document ID
    MissingDocument,
    /// Missing required space ID
    MissingSpace,
    /// Missing required folder ID
    MissingFolder,
}

impl std::fmt::Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlError::MissingWorkspace => write!(f, "missing workspace ID"),
            UrlError::MissingList => write!(f, "missing list ID"),
            UrlError::MissingTask => write!(f, "missing task ID"),
            UrlError::MissingComment => write!(f, "missing comment ID"),
            UrlError::MissingDocument => write!(f, "missing document ID"),
            UrlError::MissingSpace => write!(f, "missing space ID"),
            UrlError::MissingFolder => write!(f, "missing folder ID"),
        }
    }
}

impl std::error::Error for UrlError {}

/// URL generator for ClickUp elements
///
/// This trait provides methods to generate URLs for different ClickUp element types.
/// ClickUp supports both long-form URLs (with workspace/list context) and short-form
/// URLs (just the element ID). Short-form URLs are preferred as they're cleaner and
/// work across workspaces.
///
/// # URL Formats
///
/// ## Short-form (preferred):
/// - Task: `https://app.clickup.com/t/{task_id}`
/// - Comment: `https://app.clickup.com/t/{task_id}/comment/{comment_id}`
/// - Document: `https://app.clickup.com/d/{doc_id}`
///
/// ## Long-form (legacy, still supported):
/// - Workspace: `https://app.clickup.com/{workspace_id}`
/// - Space: `https://app.clickup.com/{workspace_id}/s/{space_id}`
/// - Folder: `https://app.clickup.com/{workspace_id}/f/{folder_id}`
/// - List: `https://app.clickup.com/{workspace_id}/l/{list_id}`
pub trait UrlGenerator {
    /// Generate URL for a workspace
    ///
    /// # Arguments
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    /// The workspace URL
    fn workspace_url(workspace_id: &str) -> UrlResult<String>;

    /// Generate URL for a space
    ///
    /// # Arguments
    /// * `workspace_id` - The workspace ID
    /// * `space_id` - The space ID
    ///
    /// # Returns
    /// The space URL or error if missing required IDs
    fn space_url(workspace_id: &str, space_id: &str) -> UrlResult<String>;

    /// Generate URL for a folder
    ///
    /// # Arguments
    /// * `workspace_id` - The workspace ID
    /// * `folder_id` - The folder ID
    ///
    /// # Returns
    /// The folder URL or error if missing required IDs
    fn folder_url(workspace_id: &str, folder_id: &str) -> UrlResult<String>;

    /// Generate URL for a list
    ///
    /// # Arguments
    /// * `workspace_id` - The workspace ID
    /// * `list_id` - The list ID
    ///
    /// # Returns
    /// The list URL or error if missing required IDs
    fn list_url(workspace_id: &str, list_id: &str) -> UrlResult<String>;

    /// Generate URL for a task (short-form: no workspace/list needed)
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    ///
    /// # Returns
    /// The task URL
    fn task_url(_workspace_id: &str, _list_id: &str, task_id: &str) -> UrlResult<String>;

    /// Generate URL for a comment (includes task context)
    ///
    /// # Arguments
    /// * `task_id` - The task ID
    /// * `comment_id` - The comment ID
    ///
    /// # Returns
    /// The comment URL
    fn comment_url(
        _workspace_id: &str,
        _list_id: &str,
        task_id: &str,
        comment_id: &str,
    ) -> UrlResult<String>;

    /// Generate URL for a document (short-form: no workspace needed)
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    ///
    /// # Returns
    /// The document URL
    fn document_url(_workspace_id: &str, doc_id: &str) -> UrlResult<String>;
}

/// Concrete implementation of the UrlGenerator trait
pub struct ClickUpUrlGenerator;

impl UrlGenerator for ClickUpUrlGenerator {
    fn workspace_url(workspace_id: &str) -> UrlResult<String> {
        if workspace_id.is_empty() {
            return Err(UrlError::MissingWorkspace);
        }
        Ok(format!("{}/{}", CLICKUP_BASE_URL, workspace_id))
    }

    fn space_url(workspace_id: &str, space_id: &str) -> UrlResult<String> {
        if workspace_id.is_empty() {
            return Err(UrlError::MissingWorkspace);
        }
        if space_id.is_empty() {
            return Err(UrlError::MissingSpace);
        }
        // ClickUp space URLs use the format:
        // https://app.clickup.com/{workspace_id}/v/o/s/{space_id}
        Ok(format!("{}/{}/v/o/s/{}", CLICKUP_BASE_URL, workspace_id, space_id))
    }

    fn folder_url(workspace_id: &str, folder_id: &str) -> UrlResult<String> {
        if workspace_id.is_empty() {
            return Err(UrlError::MissingWorkspace);
        }
        if folder_id.is_empty() {
            return Err(UrlError::MissingFolder);
        }
        // ClickUp folder URLs use the format:
        // https://app.clickup.com/{workspace_id}/v/o/f/{folder_id}
        Ok(format!("{}/{}/v/o/f/{}", CLICKUP_BASE_URL, workspace_id, folder_id))
    }

    fn list_url(workspace_id: &str, list_id: &str) -> UrlResult<String> {
        if workspace_id.is_empty() {
            return Err(UrlError::MissingWorkspace);
        }
        if list_id.is_empty() {
            return Err(UrlError::MissingList);
        }
        // ClickUp list URLs use the format:
        // https://app.clickup.com/{workspace_id}/v/l/{view}-{list_id}-{suffix}
        // where view is typically "6" (list view) and suffix is typically "1"
        // Note: This format may vary based on the workspace configuration
        Ok(format!("{}/{}/v/l/6-{}-1", CLICKUP_BASE_URL, workspace_id, list_id))
    }

    fn task_url(_workspace_id: &str, _list_id: &str, task_id: &str) -> UrlResult<String> {
        // Use short-form URL: https://app.clickup.com/t/{task_id}
        if task_id.is_empty() {
            return Err(UrlError::MissingTask);
        }
        Ok(format!("{}/t/{}", CLICKUP_BASE_URL, task_id))
    }

    fn comment_url(
        _workspace_id: &str,
        _list_id: &str,
        task_id: &str,
        comment_id: &str,
    ) -> UrlResult<String> {
        // Use short-form URL with query parameter: https://app.clickup.com/t/{task_id}?comment={comment_id}
        if task_id.is_empty() {
            return Err(UrlError::MissingTask);
        }
        if comment_id.is_empty() {
            return Err(UrlError::MissingComment);
        }
        Ok(format!("{}/t/{}?comment={}", CLICKUP_BASE_URL, task_id, comment_id))
    }

    fn document_url(_workspace_id: &str, doc_id: &str) -> UrlResult<String> {
        // Use short-form URL: https://app.clickup.com/d/{doc_id}
        if doc_id.is_empty() {
            return Err(UrlError::MissingDocument);
        }
        Ok(format!("{}/d/{}", CLICKUP_BASE_URL, doc_id))
    }
}

/// Helper function to truncate URL for display
///
/// Truncates URLs longer than `max_length` to fit in status bar.
/// Shows first N characters followed by "..."
///
/// # Arguments
/// * `url` - The URL to truncate
/// * `max_length` - Maximum length (default: 60)
///
/// # Returns
/// Truncated URL with "..." if needed
pub fn truncate_url(url: &str, max_length: usize) -> String {
    if url.len() <= max_length {
        url.to_string()
    } else {
        format!("{}...", &url[..max_length.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_url_generation() {
        let url = ClickUpUrlGenerator::workspace_url("workspace123").unwrap();
        assert_eq!(url, "https://app.clickup.com/workspace123");
    }

    #[test]
    fn test_workspace_url_empty_id() {
        let result = ClickUpUrlGenerator::workspace_url("");
        assert_eq!(result, Err(UrlError::MissingWorkspace));
    }

    #[test]
    fn test_space_url_generation() {
        let url = ClickUpUrlGenerator::space_url("ws123", "space456").unwrap();
        assert_eq!(url, "https://app.clickup.com/ws123/v/o/s/space456");
    }

    #[test]
    fn test_space_url_missing_workspace() {
        let result = ClickUpUrlGenerator::space_url("", "space456");
        assert_eq!(result, Err(UrlError::MissingWorkspace));
    }

    #[test]
    fn test_space_url_missing_space() {
        let result = ClickUpUrlGenerator::space_url("ws123", "");
        assert_eq!(result, Err(UrlError::MissingSpace));
    }

    #[test]
    fn test_folder_url_generation() {
        let url = ClickUpUrlGenerator::folder_url("ws123", "folder789").unwrap();
        assert_eq!(url, "https://app.clickup.com/ws123/v/o/f/folder789");
    }

    #[test]
    fn test_folder_url_missing_folder() {
        let result = ClickUpUrlGenerator::folder_url("ws123", "");
        assert_eq!(result, Err(UrlError::MissingFolder));
    }

    #[test]
    fn test_list_url_generation() {
        let url = ClickUpUrlGenerator::list_url("ws123", "list012").unwrap();
        assert_eq!(url, "https://app.clickup.com/ws123/v/l/6-list012-1");
    }

    #[test]
    fn test_list_url_missing_list() {
        let result = ClickUpUrlGenerator::list_url("ws123", "");
        assert_eq!(result, Err(UrlError::MissingList));
    }

    #[test]
    fn test_task_url_generation() {
        // Short-form URL: workspace and list params are ignored
        let url =
            ClickUpUrlGenerator::task_url("ws123", "list012", "task345").unwrap();
        assert_eq!(url, "https://app.clickup.com/t/task345");
    }

    #[test]
    fn test_task_url_missing_task() {
        let result = ClickUpUrlGenerator::task_url("ws123", "list012", "");
        assert_eq!(result, Err(UrlError::MissingTask));
    }

    #[test]
    fn test_comment_url_generation() {
        // Short-form URL with query parameter: workspace and list params are ignored
        let url = ClickUpUrlGenerator::comment_url(
            "ws123",
            "list012",
            "task345",
            "comment678",
        )
        .unwrap();
        assert_eq!(
            url,
            "https://app.clickup.com/t/task345?comment=comment678"
        );
    }

    #[test]
    fn test_comment_url_missing_comment() {
        let result = ClickUpUrlGenerator::comment_url(
            "ws123",
            "list012",
            "task345",
            "",
        );
        assert_eq!(result, Err(UrlError::MissingComment));
    }

    #[test]
    fn test_document_url_generation() {
        // Short-form URL: workspace param is ignored
        let url = ClickUpUrlGenerator::document_url("ws123", "doc901").unwrap();
        assert_eq!(url, "https://app.clickup.com/d/doc901");
    }

    #[test]
    fn test_document_url_missing_doc() {
        let result = ClickUpUrlGenerator::document_url("ws123", "");
        assert_eq!(result, Err(UrlError::MissingDocument));
    }

    #[test]
    fn test_truncate_url_short_url() {
        let url = "https://app.clickup.com/short";
        assert_eq!(truncate_url(url, 60), url);
    }

    #[test]
    fn test_truncate_url_long_url() {
        let url = "https://app.clickup.com/ws123/l/list012/t/task345/comment/comment678";
        let truncated = truncate_url(url, 60);
        assert_eq!(truncated.len(), 60);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_url_exact_length() {
        let url = "https://app.clickup.com/test";
        assert_eq!(truncate_url(url, url.len()), url);
    }

    #[test]
    fn test_url_error_display() {
        assert_eq!(UrlError::MissingWorkspace.to_string(), "missing workspace ID");
        assert_eq!(UrlError::MissingList.to_string(), "missing list ID");
        assert_eq!(UrlError::MissingTask.to_string(), "missing task ID");
        assert_eq!(UrlError::MissingComment.to_string(), "missing comment ID");
        assert_eq!(UrlError::MissingDocument.to_string(), "missing document ID");
        assert_eq!(UrlError::MissingSpace.to_string(), "missing space ID");
        assert_eq!(UrlError::MissingFolder.to_string(), "missing folder ID");
    }
}
