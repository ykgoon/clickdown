//! TUI widgets

pub mod auth;
pub mod comments;
pub mod dialog;
pub mod document;
pub mod help;
pub mod inbox_view;
pub mod sidebar;
pub mod task_detail;
pub mod task_list;

pub use auth::{render_auth, AuthState};
pub use comments::render_comments;
pub use dialog::{get_dialog_hints, render_dialog, DialogState, DialogType};
pub use document::{render_document, DocumentState};
pub use help::{render_help, HelpState};
pub use inbox_view::{render_inbox_list, render_notification_detail, InboxListState};
pub use sidebar::{render_sidebar, SidebarItem, SidebarState};
pub use task_detail::{render_task_detail, TaskDetailState};
pub use task_list::{render_task_list, TaskListState};
