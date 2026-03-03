//! ClickUp API data models

pub mod comment;
pub mod document;
pub mod session;
pub mod task;
pub mod workspace;

// Export specific types to avoid name conflicts with iced
pub use comment::{
    Comment, CommentsResponse, CreateCommentRequest, UpdateCommentRequest, User as CommentUser,
};
pub use document::{
    Document, DocumentFilters, DocumentPagesResponse, DocumentsResponse, Page, PageResponse,
};
pub use session::SessionState;
pub use task::*;
pub use workspace::{
    Folder, FolderReference as ClickUpFolderReference, FoldersResponse, List, ListsResponse,
    Space as ClickUpSpace, SpaceReference, SpacesResponse, Workspace, WorkspacesResponse,
};
