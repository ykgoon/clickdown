//! ClickUp API data models

pub mod comment;
pub mod document;
pub mod inbox_activity;
pub mod notification;
pub mod session;
pub mod task;
pub mod user;
pub mod workspace;

// Export specific types to avoid name conflicts with iced
pub use comment::{
    Comment, CommentsResponse, CreateCommentRequest, UpdateCommentRequest,
};
pub use document::{
    Document, DocumentFilters, DocumentPagesResponse, DocumentsResponse, Page, PageResponse,
};
pub use inbox_activity::{ActivityType, InboxActivity, InboxActivityResponse};
pub use notification::{Notification, NotificationsResponse};
pub use session::SessionState;
pub use task::*;
pub use user::User;
pub use workspace::{
    Folder, FolderReference as ClickUpFolderReference, FoldersResponse, List, ListsResponse,
    Space as ClickUpSpace, SpacesResponse, Workspace, WorkspacesResponse,
};
