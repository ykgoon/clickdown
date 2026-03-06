//! Document models

use crate::models::{ClickUpFolderReference as FolderReference, ClickUpSpace as SpaceReference};
use serde::{Deserialize, Serialize};

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
/// Re-exported from crate::models::User for backwards compatibility
pub use crate::models::user::User;

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
        use crate::utils::QueryParams;

        let mut params = QueryParams::new();
        params.add_opt_encoded("query", self.query.as_deref());
        params.add_opt("space_id", self.space_id.as_ref());
        params.add_opt("folder_id", self.folder_id.as_ref());

        params.to_query_string()
    }
}
