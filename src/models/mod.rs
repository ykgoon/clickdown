//! ClickUp API data models

pub mod workspace;
pub mod task;
pub mod document;

// Export specific types to avoid name conflicts with iced
pub use workspace::{Workspace, Space as ClickUpSpace, Folder, List, FolderReference as ClickUpFolderReference, SpaceReference, WorkspacesResponse, SpacesResponse, FoldersResponse, ListsResponse};
pub use task::*;
pub use document::{Document, Page, DocumentFilters, DocumentsResponse, DocumentPagesResponse, PageResponse};
