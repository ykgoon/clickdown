//! TUI widgets

pub mod assignee_picker;
pub mod auth;
pub mod comments;
pub mod dialog;
pub mod document;
pub mod help;
pub mod sidebar;
pub mod status_picker;
pub mod task_detail;
pub mod task_list;

pub use assignee_picker::render_assignee_picker;
pub use auth::{render_auth, AuthState};
pub use comments::render_comments;
pub use dialog::{get_dialog_hints, render_dialog, DialogState, DialogType};
pub use document::{render_document, DocumentState};
pub use help::{render_help, HelpState};
pub use sidebar::{render_sidebar, SidebarItem, SidebarState};
pub use status_picker::render_status_picker;
pub use task_detail::{render_task_detail, TaskDetailState};
pub use task_list::{render_task_list, TaskListState};
