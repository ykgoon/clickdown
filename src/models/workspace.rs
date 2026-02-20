//! Workspace, Space, Folder, and List models

use serde::{Deserialize, Serialize};
use crate::models::Priority;

/// A ClickUp Workspace (also called Team in the API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub avatar: Option<Avatar>,
    #[serde(default)]
    pub member_count: Option<u32>,
}

/// A Space within a Workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub status: Option<SpaceStatus>,
    #[serde(default)]
    pub folders: Vec<Folder>,
    #[serde(default)]
    pub lists: Vec<List>,
}

/// Status of a Space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceStatus {
    pub status: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub type_field: Option<String>,
}

/// A Folder within a Space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub space: Option<SpaceReference>,
    #[serde(default)]
    pub lists: Vec<List>,
}

/// Reference to a Space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceReference {
    pub id: String,
    pub name: String,
}

/// A List within a Folder or Space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub orderindex: Option<u32>,
    #[serde(default)]
    pub space: Option<SpaceReference>,
    #[serde(default)]
    pub folder: Option<FolderReference>,
    #[serde(default)]
    pub status: Option<ListStatus>,
    #[serde(default)]
    pub priority: Option<Priority>,
}

/// Reference to a Folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderReference {
    pub id: String,
    pub name: Option<String>,
}

/// Status of a List
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStatus {
    pub status: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub type_field: Option<String>,
}

/// Avatar image reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avatar {
    pub attachment_id: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// API response for getting workspaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacesResponse {
    pub teams: Vec<Workspace>,
}

/// API response for getting spaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacesResponse {
    pub spaces: Vec<Space>,
}

/// API response for getting folders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldersResponse {
    pub folders: Vec<Folder>,
}

/// API response for getting lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListsResponse {
    pub lists: Vec<List>,
}
