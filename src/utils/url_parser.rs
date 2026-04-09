//! URL parsing utilities for ClickUp resource URLs
//!
//! This module provides functionality to parse ClickUp web app URLs and extract
//! resource identifiers. It mirrors the URL generation patterns from
//! `ClickUpUrlGenerator` to ensure round-trip symmetry — any URL the app
//! generates can be parsed back to the same resource IDs.
//!
//! # Supported URL Patterns
//!
//! ## Long-form URLs:
//! - Workspace: `https://app.clickup.com/{workspace_id}`
//! - Space: `https://app.clickup.com/{workspace_id}/v/o/s/{space_id}`
//! - Folder: `https://app.clickup.com/{workspace_id}/v/o/f/{folder_id}`
//! - List: `https://app.clickup.com/{workspace_id}/v/l/{view}-{list_id}-{suffix}`
//!
//! ## Short-form URLs:
//! - Task: `https://app.clickup.com/t/{task_id}`
//! - Comment: `https://app.clickup.com/t/{task_id}?comment={comment_id}`
//! - Document: `https://app.clickup.com/d/{doc_id}`

/// Base URL for ClickUp web app
const CLICKUP_BASE_URL: &str = "https://app.clickup.com";

/// Result type for URL parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Error type for URL parsing failures
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// The input is not a valid URL
    InvalidUrl(String),
    /// The URL is not a recognized ClickUp URL
    NotClickUpUrl(String),
    /// The URL format is not recognized
    UnknownFormat(String),
    /// A required resource ID is missing in the URL
    MissingResourceId(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidUrl(msg) => write!(f, "invalid URL: {}", msg),
            ParseError::NotClickUpUrl(msg) => write!(f, "not a ClickUp URL: {}", msg),
            ParseError::UnknownFormat(msg) => {
                write!(f, "unrecognized ClickUp URL format: {}", msg)
            }
            ParseError::MissingResourceId(msg) => {
                write!(f, "missing resource ID: {}", msg)
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parsed ClickUp URL, identifying the resource type and its IDs
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedUrl {
    /// Workspace URL: `https://app.clickup.com/{workspace_id}`
    Workspace {
        workspace_id: String,
    },
    /// Space URL: `https://app.clickup.com/{workspace_id}/v/o/s/{space_id}`
    Space {
        workspace_id: String,
        space_id: String,
    },
    /// Folder URL: `https://app.clickup.com/{workspace_id}/v/o/f/{folder_id}`
    Folder {
        workspace_id: String,
        folder_id: String,
    },
    /// List URL: `https://app.clickup.com/{workspace_id}/v/l/{view}-{list_id}-{suffix}`
    List {
        workspace_id: String,
        list_id: String,
    },
    /// Task URL (short-form): `https://app.clickup.com/t/{task_id}`
    Task {
        task_id: String,
    },
    /// Comment URL (short-form): `https://app.clickup.com/t/{task_id}?comment={comment_id}`
    Comment {
        task_id: String,
        comment_id: String,
    },
    /// Document URL (short-form): `https://app.clickup.com/d/{doc_id}`
    Document {
        doc_id: String,
    },
}

/// URL parser for ClickUp resource URLs
pub struct UrlParser;

impl UrlParser {
    /// Parse a ClickUp URL and extract resource identifiers
    ///
    /// # Arguments
    /// * `url` - The URL string to parse
    ///
    /// # Returns
    /// A `ParsedUrl` enum variant with the extracted resource IDs, or a `ParseError`
    pub fn parse(url: &str) -> ParseResult<ParsedUrl> {
        let url = url.trim_end_matches('/');

        if !url.starts_with(CLICKUP_BASE_URL) {
            return Err(ParseError::NotClickUpUrl(format!(
                "expected base URL {}",
                CLICKUP_BASE_URL
            )));
        }

        let path = &url[CLICKUP_BASE_URL.len()..];

        // Separate path from query string
        let (path, query) = if let Some(pos) = path.find('?') {
            (&path[..pos], Some(&path[pos + 1..]))
        } else {
            (path, None)
        };

        // Remove leading slash
        let path = path.strip_prefix('/').unwrap_or(path);

        if path.is_empty() {
            return Err(ParseError::UnknownFormat(
                "empty path after base URL".to_string(),
            ));
        }

        // Try short-form patterns first: /t/{id}, /d/{id}
        if let Some(result) = Self::try_short_form(path, query)? {
            return Ok(result);
        }

        // Try long-form patterns
        Self::try_long_form(path)
    }

    /// Try to match short-form URL patterns: /t/{id}, /d/{id}
    fn try_short_form(path: &str, query: Option<&str>) -> ParseResult<Option<ParsedUrl>> {
        // Task or comment: /t/{task_id} or /t/{task_id}?comment={comment_id}
        if let Some(rest) = path.strip_prefix("t/") {
            let task_id = Self::extract_id(rest)?;

            // Check for comment query parameter
            if let Some(q) = query {
                if let Some(comment_id) = Self::extract_query_param(q, "comment") {
                    return Ok(Some(ParsedUrl::Comment {
                        task_id: task_id.to_string(),
                        comment_id,
                    }));
                }
            }

            return Ok(Some(ParsedUrl::Task {
                task_id: task_id.to_string(),
            }));
        }

        // Document: /d/{doc_id}
        if let Some(rest) = path.strip_prefix("d/") {
            let doc_id = Self::extract_id(rest)?;
            return Ok(Some(ParsedUrl::Document {
                doc_id: doc_id.to_string(),
            }));
        }

        Ok(None)
    }

    /// Try to match long-form URL patterns
    fn try_long_form(path: &str) -> ParseResult<ParsedUrl> {
        let segments: Vec<&str> = path.split('/').collect();

        if segments.is_empty() {
            return Err(ParseError::UnknownFormat("empty path".to_string()));
        }

        // Workspace: just the workspace ID (single segment)
        if segments.len() == 1 {
            let workspace_id = Self::extract_id(segments[0])?;
            return Ok(ParsedUrl::Workspace {
                workspace_id: workspace_id.to_string(),
            });
        }

        let workspace_id = Self::extract_id(segments[0])?;

        // Space: {workspace_id}/v/o/s/{space_id}
        // Path has 5 segments: [workspace_id, v, o, s, space_id]
        if segments.len() >= 5
            && segments[1] == "v"
            && segments[2] == "o"
            && segments[3] == "s"
        {
            let space_id = Self::extract_id(segments[4])?;
            return Ok(ParsedUrl::Space {
                workspace_id: workspace_id.to_string(),
                space_id: space_id.to_string(),
            });
        }

        // Folder: {workspace_id}/v/o/f/{folder_id}
        // Path has 5 segments: [workspace_id, v, o, f, folder_id]
        if segments.len() >= 5
            && segments[1] == "v"
            && segments[2] == "o"
            && segments[3] == "f"
        {
            let folder_id = Self::extract_id(segments[4])?;
            return Ok(ParsedUrl::Folder {
                workspace_id: workspace_id.to_string(),
                folder_id: folder_id.to_string(),
            });
        }

        // List: {workspace_id}/v/l/{view}-{list_id}-{suffix}
        // Path has 4 segments: [workspace_id, v, l, pattern]
        if segments.len() >= 4
            && segments[1] == "v"
            && segments[2] == "l"
        {
            let list_pattern = segments[3];
            if let Some(list_id) = Self::extract_list_id_from_pattern(list_pattern) {
                return Ok(ParsedUrl::List {
                    workspace_id: workspace_id.to_string(),
                    list_id,
                });
            }
        }

        Err(ParseError::UnknownFormat(format!(
            "path: {}",
            path
        )))
    }

    /// Extract a resource ID from a path segment, validating it's not empty
    fn extract_id(segment: &str) -> ParseResult<&str> {
        if segment.is_empty() {
            Err(ParseError::MissingResourceId(
                "empty resource ID segment".to_string(),
            ))
        } else {
            Ok(segment)
        }
    }

    /// Extract a query parameter value by key
    fn extract_query_param(query: &str, key: &str) -> Option<String> {
        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                if k == key && !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
        None
    }

    /// Extract list ID from the pattern `{view}-{list_id}-{suffix}`
    /// e.g., "6-list012-1" → "list012"
    fn extract_list_id_from_pattern(pattern: &str) -> Option<String> {
        // Pattern: {view}-{list_id}-{suffix}
        // We need to find the middle part. The view is typically a number,
        // and the suffix is also a number. Split on '-' and take the middle parts.
        let parts: Vec<&str> = pattern.split('-').collect();
        if parts.len() >= 3 {
            // Middle parts (excluding first and last)
            // e.g., ["6", "list012", "1"] → "list012"
            // e.g., ["6", "my", "list", "1"] → "my-list"
            let middle = &parts[1..parts.len() - 1];
            let list_id = middle.join("-");
            if !list_id.is_empty() {
                return Some(list_id);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Workspace URL parsing ==========

    #[test]
    fn test_parse_workspace_url() {
        let result = UrlParser::parse("https://app.clickup.com/workspace123").unwrap();
        match result {
            ParsedUrl::Workspace { workspace_id } => {
                assert_eq!(workspace_id, "workspace123");
            }
            _ => panic!("Expected Workspace variant"),
        }
    }

    #[test]
    fn test_parse_workspace_url_with_trailing_slash() {
        let result = UrlParser::parse("https://app.clickup.com/workspace123/").unwrap();
        match result {
            ParsedUrl::Workspace { workspace_id } => {
                assert_eq!(workspace_id, "workspace123");
            }
            _ => panic!("Expected Workspace variant"),
        }
    }

    // ========== Space URL parsing ==========

    #[test]
    fn test_parse_space_url() {
        let result = UrlParser::parse("https://app.clickup.com/ws123/v/o/s/space456").unwrap();
        match result {
            ParsedUrl::Space {
                workspace_id,
                space_id,
            } => {
                assert_eq!(workspace_id, "ws123");
                assert_eq!(space_id, "space456");
            }
            _ => panic!("Expected Space variant"),
        }
    }

    // ========== Folder URL parsing ==========

    #[test]
    fn test_parse_folder_url() {
        let result = UrlParser::parse("https://app.clickup.com/ws123/v/o/f/folder789").unwrap();
        match result {
            ParsedUrl::Folder {
                workspace_id,
                folder_id,
            } => {
                assert_eq!(workspace_id, "ws123");
                assert_eq!(folder_id, "folder789");
            }
            _ => panic!("Expected Folder variant"),
        }
    }

    // ========== List URL parsing ==========

    #[test]
    fn test_parse_list_url() {
        let result = UrlParser::parse("https://app.clickup.com/ws123/v/l/6-list012-1").unwrap();
        match result {
            ParsedUrl::List {
                workspace_id,
                list_id,
            } => {
                assert_eq!(workspace_id, "ws123");
                assert_eq!(list_id, "list012");
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_parse_list_url_with_hyphens_in_id() {
        let result = UrlParser::parse("https://app.clickup.com/ws123/v/l/6-my-list-name-1").unwrap();
        match result {
            ParsedUrl::List {
                workspace_id,
                list_id,
            } => {
                assert_eq!(workspace_id, "ws123");
                assert_eq!(list_id, "my-list-name");
            }
            _ => panic!("Expected List variant"),
        }
    }

    // ========== Task URL parsing ==========

    #[test]
    fn test_parse_task_url() {
        let result = UrlParser::parse("https://app.clickup.com/t/task345").unwrap();
        match result {
            ParsedUrl::Task { task_id } => {
                assert_eq!(task_id, "task345");
            }
            _ => panic!("Expected Task variant"),
        }
    }

    #[test]
    fn test_parse_task_url_with_trailing_slash() {
        let result = UrlParser::parse("https://app.clickup.com/t/task345/").unwrap();
        match result {
            ParsedUrl::Task { task_id } => {
                assert_eq!(task_id, "task345");
            }
            _ => panic!("Expected Task variant"),
        }
    }

    // ========== Comment URL parsing ==========

    #[test]
    fn test_parse_comment_url() {
        let result =
            UrlParser::parse("https://app.clickup.com/t/task345?comment=comment678").unwrap();
        match result {
            ParsedUrl::Comment {
                task_id,
                comment_id,
            } => {
                assert_eq!(task_id, "task345");
                assert_eq!(comment_id, "comment678");
            }
            _ => panic!("Expected Comment variant"),
        }
    }

    // ========== Document URL parsing ==========

    #[test]
    fn test_parse_document_url() {
        let result = UrlParser::parse("https://app.clickup.com/d/doc901").unwrap();
        match result {
            ParsedUrl::Document { doc_id } => {
                assert_eq!(doc_id, "doc901");
            }
            _ => panic!("Expected Document variant"),
        }
    }

    // ========== Error cases ==========

    #[test]
    fn test_reject_non_clickup_url() {
        let result = UrlParser::parse("https://example.com/something");
        assert!(matches!(result, Err(ParseError::NotClickUpUrl(_))));
    }

    #[test]
    fn test_reject_malformed_input() {
        let result = UrlParser::parse("not-a-url-at-all");
        assert!(matches!(result, Err(ParseError::NotClickUpUrl(_))));
    }

    #[test]
    fn test_reject_unknown_clickup_format() {
        let result = UrlParser::parse("https://app.clickup.com/unknown/path/here");
        assert!(matches!(result, Err(ParseError::UnknownFormat(_))));
    }

    // ========== Round-trip symmetry tests ==========

    use crate::utils::{ClickUpUrlGenerator, UrlGenerator};

    #[test]
    fn test_roundtrip_workspace_url() {
        let generated = ClickUpUrlGenerator::workspace_url("workspace123").unwrap();
        let parsed = UrlParser::parse(&generated).unwrap();
        match parsed {
            ParsedUrl::Workspace { workspace_id } => {
                assert_eq!(workspace_id, "workspace123");
            }
            _ => panic!("Expected Workspace, got {:?}", parsed),
        }
    }

    #[test]
    fn test_roundtrip_space_url() {
        let generated = ClickUpUrlGenerator::space_url("ws123", "space456").unwrap();
        let parsed = UrlParser::parse(&generated).unwrap();
        match parsed {
            ParsedUrl::Space {
                workspace_id,
                space_id,
            } => {
                assert_eq!(workspace_id, "ws123");
                assert_eq!(space_id, "space456");
            }
            _ => panic!("Expected Space, got {:?}", parsed),
        }
    }

    #[test]
    fn test_roundtrip_folder_url() {
        let generated = ClickUpUrlGenerator::folder_url("ws123", "folder789").unwrap();
        let parsed = UrlParser::parse(&generated).unwrap();
        match parsed {
            ParsedUrl::Folder {
                workspace_id,
                folder_id,
            } => {
                assert_eq!(workspace_id, "ws123");
                assert_eq!(folder_id, "folder789");
            }
            _ => panic!("Expected Folder, got {:?}", parsed),
        }
    }

    #[test]
    fn test_roundtrip_list_url() {
        let generated = ClickUpUrlGenerator::list_url("ws123", "list012").unwrap();
        let parsed = UrlParser::parse(&generated).unwrap();
        match parsed {
            ParsedUrl::List {
                workspace_id,
                list_id,
            } => {
                assert_eq!(workspace_id, "ws123");
                assert_eq!(list_id, "list012");
            }
            _ => panic!("Expected List, got {:?}", parsed),
        }
    }

    #[test]
    fn test_roundtrip_task_url() {
        let generated = ClickUpUrlGenerator::task_url("", "", "task345").unwrap();
        let parsed = UrlParser::parse(&generated).unwrap();
        match parsed {
            ParsedUrl::Task { task_id } => {
                assert_eq!(task_id, "task345");
            }
            _ => panic!("Expected Task, got {:?}", parsed),
        }
    }

    #[test]
    fn test_roundtrip_comment_url() {
        let generated =
            ClickUpUrlGenerator::comment_url("", "", "task345", "comment678").unwrap();
        let parsed = UrlParser::parse(&generated).unwrap();
        match parsed {
            ParsedUrl::Comment {
                task_id,
                comment_id,
            } => {
                assert_eq!(task_id, "task345");
                assert_eq!(comment_id, "comment678");
            }
            _ => panic!("Expected Comment, got {:?}", parsed),
        }
    }

    #[test]
    fn test_roundtrip_document_url() {
        let generated = ClickUpUrlGenerator::document_url("", "doc901").unwrap();
        let parsed = UrlParser::parse(&generated).unwrap();
        match parsed {
            ParsedUrl::Document { doc_id } => {
                assert_eq!(doc_id, "doc901");
            }
            _ => panic!("Expected Document, got {:?}", parsed),
        }
    }
}
