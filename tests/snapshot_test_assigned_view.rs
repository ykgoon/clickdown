//! Snapshot tests for assigned view rendering with tasks and comments
//!
//! These tests verify the unified "Assigned to Me" view displays both tasks
//! and comments correctly. Tests use the `insta` crate for snapshot comparisons
//! with mocked data for deterministic, reproducible results.
//!
//! Run tests: `cargo test --test snapshot_test_assigned_view`
//! Review snapshots: `cargo insta review`
//! Accept changes: `cargo insta accept`

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::models::workspace::List;
use clickdown::models::{AssignedComment, AssignedItem, TaskReference};
use clickdown::tui::widgets::assigned_view::render_assigned_view;
use clickdown::tui::widgets::SidebarItem;
use clickdown::{models::AssignedItemsFilter, tui::app::TuiApp};
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use std::sync::Arc;
use tokio::runtime::Runtime;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test terminal with the given size
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).unwrap()
}

/// Render assigned view widget and capture as string for snapshot
fn capture_assigned_view(
    items: &[AssignedItem],
    selected_index: usize,
    filter: AssignedItemsFilter,
    count: usize,
    width: u16,
    height: u16,
    loading: bool,
    error: Option<&str>,
) -> String {
    let mut terminal = create_test_terminal(width, height);

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, width, height);
            render_assigned_view(
                frame,
                items,
                selected_index,
                filter,
                area,
                loading,
                error,
                count,
            );
        })
        .unwrap();

    // Get buffer contents
    let mut snapshot = String::new();
    for y in 0..height {
        for x in 0..width {
            let cell = terminal.backend().buffer().get(x, y);
            snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
        }
        snapshot.push('\n');
    }

    snapshot
}

/// Create test assigned items (both tasks and comments)
fn create_test_assigned_items() -> Vec<AssignedItem> {
    let tasks = fixtures::test_tasks_with_assignees();
    let comments = fixtures::test_assigned_comments();

    let mut items = Vec::new();

    // Add tasks
    for task in tasks {
        items.push(AssignedItem::Task(task));
    }

    // Add comments
    for comment in comments {
        items.push(AssignedItem::AssignedComment(comment));
    }

    items
}

// ============================================================================
// Assigned View Snapshot Tests
// ============================================================================

/// Test assigned view empty state
#[test]
fn test_assigned_view_empty() {
    let items = vec![];
    let snapshot = capture_assigned_view(&items, 0, AssignedItemsFilter::All, 0, 60, 15, false, None);
    assert_snapshot!("assigned_view_empty", snapshot);
}

/// Test assigned view loading state
#[test]
fn test_assigned_view_loading() {
    let items = vec![];
    let snapshot = capture_assigned_view(&items, 0, AssignedItemsFilter::All, 0, 60, 15, true, None);
    assert_snapshot!("assigned_view_loading", snapshot);
}

/// Test assigned view error state
#[test]
fn test_assigned_view_error() {
    let items = vec![];
    let snapshot = capture_assigned_view(
        &items,
        0,
        AssignedItemsFilter::All,
        0,
        60,
        15,
        false,
        Some("Failed to fetch assigned items"),
    );
    assert_snapshot!("assigned_view_error", snapshot);
}

/// Test assigned view with tasks only (filter)
#[test]
fn test_assigned_view_filter_tasks_only() {
    let items = create_test_assigned_items();
    let snapshot = capture_assigned_view(
        &items,
        0,
        AssignedItemsFilter::TasksOnly,
        items.len(),
        60,
        15,
        false,
        None,
    );
    assert_snapshot!("assigned_view_filter_tasks_only", snapshot);
}

/// Test assigned view with comments only (filter)
#[test]
fn test_assigned_view_filter_comments_only() {
    let items = create_test_assigned_items();
    let snapshot = capture_assigned_view(
        &items,
        0,
        AssignedItemsFilter::CommentsOnly,
        items.len(),
        60,
        15,
        false,
        None,
    );
    assert_snapshot!("assigned_view_filter_comments_only", snapshot);
}

/// Test assigned view with all items (tasks + comments mixed)
#[test]
fn test_assigned_view_all_items() {
    let items = create_test_assigned_items();
    let snapshot = capture_assigned_view(
        &items,
        0,
        AssignedItemsFilter::All,
        items.len(),
        60,
        15,
        false,
        None,
    );
    assert_snapshot!("assigned_view_all_items", snapshot);
}

/// Test assigned view with selection
#[test]
fn test_assigned_view_with_selection() {
    let items = create_test_assigned_items();
    let snapshot = capture_assigned_view(
        &items,
        2, // Select third item
        AssignedItemsFilter::All,
        items.len(),
        60,
        15,
        false,
        None,
    );
    assert_snapshot!("assigned_view_with_selection", snapshot);
}

/// Test assigned view visual distinction between tasks and comments
#[test]
fn test_assigned_view_visual_distinction() {
    // Create items with clear visual differences
    let mut items = vec![
        AssignedItem::Task(fixtures::test_task_with_assignee()),
        AssignedItem::AssignedComment(fixtures::test_assigned_comment()),
    ];

    let snapshot = capture_assigned_view(
        &items,
        0,
        AssignedItemsFilter::All,
        items.len(),
        60,
        15,
        false,
        None,
    );
    assert_snapshot!("assigned_view_visual_distinction", snapshot);
}

/// Test assigned view with large item count
#[test]
fn test_assigned_view_large_count() {
    // Create many items to test scrolling/pagination appearance
    let mut items = Vec::new();
    for i in 0..10 {
        let mut task = fixtures::test_task();
        task.id = format!("task-{}", i);
        task.name = format!("Task {}", i);
        items.push(AssignedItem::Task(task));
    }

    let snapshot = capture_assigned_view(
        &items,
        5, // Select middle item
        AssignedItemsFilter::All,
        items.len(),
        60,
        15,
        false,
        None,
    );
    assert_snapshot!("assigned_view_large_count", snapshot);
}

/// Test assigned view with mixed tasks and comments sorted by updated_at
#[test]
fn test_assigned_view_sorted_mixed_items() {
    use clickdown::models::assigned_item::sort_assigned_items;

    let mut items = create_test_assigned_items();
    items = sort_assigned_items(items);

    let snapshot = capture_assigned_view(
        &items,
        0,
        AssignedItemsFilter::All,
        items.len(),
        60,
        15,
        false,
        None,
    );
    assert_snapshot!("assigned_view_sorted_mixed_items", snapshot);
}
