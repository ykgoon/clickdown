//! TUI widgets

pub mod sidebar;
pub mod task_list;
pub mod task_detail;
pub mod auth;
pub mod document;
pub mod dialog;
pub mod help;
pub mod comments;

pub use sidebar::{SidebarState, SidebarItem, render_sidebar, get_sidebar_hints};
pub use task_list::{TaskListState, render_task_list, get_task_list_hints};
pub use task_detail::{TaskDetailState, render_task_detail, get_task_detail_hints};
pub use auth::{AuthState, render_auth, get_auth_hints};
pub use document::{DocumentState, render_document, get_document_hints};
pub use dialog::{DialogState, DialogType, render_dialog, get_dialog_hints};
pub use help::{HelpState, render_help};
pub use comments::render_comments;
