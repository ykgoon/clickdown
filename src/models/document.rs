//! Document models

use serde::{Deserialize, Serialize};
use crate::models::{ClickUpSpace as SpaceReference, ClickUpFolderReference as FolderReference};

/// A ClickUp Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
    #[serde(default)]
    pub created_by: Option<User>,
    #[serde(default)]
    pub updated_by: Option<User>,
    #[serde(default)]
    pub space: Option<SpaceReference>,
    #[serde(default)]
    pub folder: Option<FolderReference>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub pages: Vec<Page>,
}

/// A Page within a Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub content_markdown: Option<String>,
    #[serde(default)]
    pub order: Option<u32>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
    #[serde(default)]
    pub children: Vec<Page>,
}

/// User reference for documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub profile_picture: Option<String>,
}

/// API response for searching documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentsResponse {
    pub docs: Vec<Document>,
}

/// API response for getting document pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPagesResponse {
    pub pages: Vec<Page>,
}

/// API response for getting a single page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse {
    pub page: Page,
}

/// Parameters for searching documents
#[derive(Debug, Clone, Default)]
pub struct DocumentFilters {
    pub query: Option<String>,
    pub space_id: Option<String>,
    pub folder_id: Option<String>,
}

impl DocumentFilters {
    /// Convert filters to query parameters
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(ref query) = self.query {
            params.push(format!("query={}", urlencoding::encode(query)));
        }
        if let Some(ref space_id) = self.space_id {
            params.push(format!("space_id={}", space_id));
        }
        if let Some(ref folder_id) = self.folder_id {
            params.push(format!("folder_id={}", folder_id));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
