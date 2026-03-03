//! TUI widgets

pub mod auth;
pub mod comments;
pub mod dialog;
pub mod document;
pub mod help;
pub mod sidebar;
pub mod task_detail;
pub mod task_list;

pub use auth::{get_auth_hints, render_auth, AuthState};
pub use comments::render_comments;
pub use dialog::{get_dialog_hints, render_dialog, DialogState, DialogType};
pub use document::{get_document_hints, render_document, DocumentState};
pub use help::{get_help_hints, render_help, HelpState};
pub use sidebar::{get_sidebar_hints, render_sidebar, SidebarItem, SidebarState};
pub use task_detail::{get_task_detail_hints, render_task_detail, TaskDetailState};
pub use task_list::{get_task_list_hints, render_task_list, TaskListState};
