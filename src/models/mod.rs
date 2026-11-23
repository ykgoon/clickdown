//! ClickUp API data models

pub mod comment;
pub mod document;
pub mod entity_ref;
pub mod inbox;
pub mod session;
pub mod task;
pub mod user;
pub mod workspace;

// Export specific types to avoid name conflicts with iced
pub use comment::{Comment, CommentsResponse, CreateCommentRequest, UpdateCommentRequest};
pub use document::{Document, DocumentFilters, DocumentsResponse};
pub use entity_ref::EntityRef;
pub use inbox::{Inbox, InboxResponse, InboxTask};
pub use session::SessionState;
pub use task::*;
pub use user::MembersResponse;
pub use user::User;
pub use workspace::{
    Folder, FoldersResponse, List, ListsResponse, Space as ClickUpSpace, SpacesResponse,
    UserResponse, Workspace, WorkspacesResponse,
};
