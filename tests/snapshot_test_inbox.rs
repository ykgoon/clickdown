//! Snapshot tests for inbox/activity feed feature
//!
//! These tests verify that the inbox displays activities correctly in the UI
//! with mocked data for deterministic, reproducible results.
//!
//! Run tests: `cargo test --test snapshot_test_inbox`
//! Review snapshots: `cargo insta review`
//! Accept changes: `cargo insta accept`

mod fixtures;

use clickdown::tui::widgets::{render_inbox_list, InboxListState};
use insta::assert_snapshot;
use ratatui::{backend::TestBackend, layout::Rect, Terminal};

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test terminal with the given size
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).unwrap()
}

/// Render inbox widget and capture as string for snapshot
fn capture_inbox(inbox: &InboxListState, width: u16, height: u16) -> String {
    let mut terminal = create_test_terminal(width, height);
    let mut inbox_copy = inbox.clone();

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, width, height);
            render_inbox_list(frame, area, &mut inbox_copy, false);
        })
        .unwrap();

    // Get buffer contents
    let mut snapshot = String::new();
    for y in 0..height {
        for x in 0..width {
            let cell = &terminal.backend().buffer()[ratatui::layout::Position::new(x, y)];
            snapshot.push(cell.symbol().chars().next().unwrap_or(' '));
        }
        snapshot.push('\n');
    }

    snapshot
}

// ============================================================================
// Inbox Widget Snapshot Tests
// ============================================================================

/// Test inbox widget with loaded activities
#[test]
fn test_inbox_widget_with_activities() {
    let mut inbox = InboxListState::new();
    inbox.set_activities(fixtures::test_inbox_activities());

    let snapshot = capture_inbox(&inbox, 60, 15);
    assert_snapshot!("inbox_widget_with_activities", snapshot);
}

/// Test inbox widget with selection on first item
#[test]
fn test_inbox_widget_with_first_selected() {
    let mut inbox = InboxListState::new();
    inbox.set_activities(fixtures::test_inbox_activities());
    inbox.list_state.select(Some(0));

    let snapshot = capture_inbox(&inbox, 60, 15);
    assert_snapshot!("inbox_widget_with_first_selected", snapshot);
}

/// Test inbox widget with selection on last item
#[test]
fn test_inbox_widget_with_last_selected() {
    let mut inbox = InboxListState::new();
    inbox.set_activities(fixtures::test_inbox_activities());
    inbox.list_state.select(Some(3));

    let snapshot = capture_inbox(&inbox, 60, 15);
    assert_snapshot!("inbox_widget_with_last_selected", snapshot);
}

/// Test inbox widget empty state
#[test]
fn test_inbox_widget_empty() {
    let inbox = InboxListState::new();

    let snapshot = capture_inbox(&inbox, 60, 15);
    assert_snapshot!("inbox_widget_empty", snapshot);
}

/// Test inbox widget with single activity type (assignment)
#[test]
fn test_inbox_widget_assignment_only() {
    let mut inbox = InboxListState::new();
    inbox.set_activities(vec![fixtures::test_inbox_activity_assignment()]);

    let snapshot = capture_inbox(&inbox, 60, 15);
    assert_snapshot!("inbox_widget_assignment_only", snapshot);
}

/// Test inbox widget with comment activity
#[test]
fn test_inbox_widget_comment_only() {
    let mut inbox = InboxListState::new();
    inbox.set_activities(vec![fixtures::test_inbox_activity_comment()]);

    let snapshot = capture_inbox(&inbox, 60, 15);
    assert_snapshot!("inbox_widget_comment_only", snapshot);
}

/// Test inbox widget with status change activity
#[test]
fn test_inbox_widget_status_change_only() {
    let mut inbox = InboxListState::new();
    inbox.set_activities(vec![fixtures::test_inbox_activity_status_change()]);

    let snapshot = capture_inbox(&inbox, 60, 15);
    assert_snapshot!("inbox_widget_status_change_only", snapshot);
}

/// Test inbox widget with due date activity
#[test]
fn test_inbox_widget_due_date_only() {
    let mut inbox = InboxListState::new();
    inbox.set_activities(vec![fixtures::test_inbox_activity_due_date()]);

    let snapshot = capture_inbox(&inbox, 60, 15);
    assert_snapshot!("inbox_widget_due_date_only", snapshot);
}

// ============================================================================
// Additional Inbox Activity Display Tests
// ============================================================================

/// Test inbox with all activity types displayed together
#[test]
fn test_inbox_widget_all_activity_types() {
    use clickdown::models::inbox_activity::{InboxActivity, ActivityType};
    
    let activities = vec![
        InboxActivity::assignment(
            "task_1".to_string(),
            "Review PR #123".to_string(),
            "ws_1".to_string(),
            1704067200000,
            Some("Open".to_string()),
        ),
        InboxActivity::comment(
            "comment_1".to_string(),
            "task_2".to_string(),
            "Fix bug in auth".to_string(),
            "ws_1".to_string(),
            1704067300000,
            "I've reviewed the changes and they look good.".to_string(),
        ),
        InboxActivity::status_change(
            "task_3".to_string(),
            "Deploy to staging".to_string(),
            "ws_1".to_string(),
            1704067400000,
            "In Progress".to_string(),
            "Complete".to_string(),
        ),
        InboxActivity::due_date(
            "task_4".to_string(),
            "Submit quarterly report".to_string(),
            "ws_1".to_string(),
            1704153600000,
        ),
    ];
    
    let mut inbox = InboxListState::new();
    inbox.set_activities(activities);
    inbox.list_state.select(Some(1));

    let snapshot = capture_inbox(&inbox, 70, 18);
    assert_snapshot!("inbox_widget_all_activity_types", snapshot);
}

/// Test inbox with long activity descriptions (truncation)
#[test]
fn test_inbox_widget_long_descriptions() {
    use clickdown::models::inbox_activity::InboxActivity;
    
    let activities = vec![
        InboxActivity::comment(
            "comment_1".to_string(),
            "task_1".to_string(),
            "Code Review".to_string(),
            "ws_1".to_string(),
            1704067200000,
            "This is a very long comment that should be truncated in the display. The comment contains important feedback about the implementation approach and suggests some improvements that should be considered before merging this pull request.".to_string(),
        ),
        InboxActivity::comment(
            "comment_2".to_string(),
            "task_2".to_string(),
            "Another Task".to_string(),
            "ws_1".to_string(),
            1704067300000,
            "Short comment".to_string(),
        ),
    ];
    
    let mut inbox = InboxListState::new();
    inbox.set_activities(activities);

    let snapshot = capture_inbox(&inbox, 60, 12);
    assert_snapshot!("inbox_widget_long_descriptions", snapshot);
}

/// Test inbox with many activities (scrolling state)
#[test]
fn test_inbox_widget_many_activities() {
    use clickdown::models::inbox_activity::InboxActivity;
    
    let mut activities = Vec::new();
    for i in 0..15 {
        activities.push(InboxActivity::assignment(
            format!("task_{}", i),
            format!("Task number {}", i),
            "ws_1".to_string(),
            1704067200000 + (i as i64 * 1000),
            Some("Open".to_string()),
        ));
    }
    
    let mut inbox = InboxListState::new();
    inbox.set_activities(activities);
    inbox.list_state.select(Some(7)); // Select middle item

    let snapshot = capture_inbox(&inbox, 60, 10);
    assert_snapshot!("inbox_widget_many_activities", snapshot);
}

/// Test inbox with activity icons display
#[test]
fn test_inbox_widget_icons_display() {
    use clickdown::models::inbox_activity::{InboxActivity, ActivityType};
    
    // Create activities with each type to verify icons render correctly
    let activities = vec![
        InboxActivity {
            id: "assignment_1".to_string(),
            activity_type: ActivityType::Assignment,
            title: "📋 Task assigned".to_string(),
            description: "You were assigned to a task".to_string(),
            timestamp: 1704067200000,
            task_id: Some("task_1".to_string()),
            comment_id: None,
            workspace_id: "ws_1".to_string(),
            task_name: "Task 1".to_string(),
            previous_status: None,
            new_status: None,
            due_date: None,
        },
        InboxActivity {
            id: "comment_1".to_string(),
            activity_type: ActivityType::Comment,
            title: "💬 New comment".to_string(),
            description: "Someone commented on your task".to_string(),
            timestamp: 1704067300000,
            task_id: Some("task_2".to_string()),
            comment_id: Some("comment_1".to_string()),
            workspace_id: "ws_1".to_string(),
            task_name: "Task 2".to_string(),
            previous_status: None,
            new_status: None,
            due_date: None,
        },
        InboxActivity {
            id: "status_1".to_string(),
            activity_type: ActivityType::StatusChange,
            title: "🔄 Status changed".to_string(),
            description: "Task status was updated".to_string(),
            timestamp: 1704067400000,
            task_id: Some("task_3".to_string()),
            comment_id: None,
            workspace_id: "ws_1".to_string(),
            task_name: "Task 3".to_string(),
            previous_status: Some("Open".to_string()),
            new_status: Some("Closed".to_string()),
            due_date: None,
        },
        InboxActivity {
            id: "due_1".to_string(),
            activity_type: ActivityType::DueDate,
            title: "⏰ Due soon".to_string(),
            description: "Task is due in 2 days".to_string(),
            timestamp: 1704067500000,
            task_id: Some("task_4".to_string()),
            comment_id: None,
            workspace_id: "ws_1".to_string(),
            task_name: "Task 4".to_string(),
            previous_status: None,
            new_status: None,
            due_date: Some(1704153600000),
        },
    ];
    
    let mut inbox = InboxListState::new();
    inbox.set_activities(activities);

    let snapshot = capture_inbox(&inbox, 65, 15);
    assert_snapshot!("inbox_widget_icons_display", snapshot);
}

/// Test inbox with activity showing status details
#[test]
fn test_inbox_widget_status_details() {
    use clickdown::models::inbox_activity::InboxActivity;
    
    let activities = vec![
        InboxActivity::status_change(
            "task_1".to_string(),
            "Bug fix #456".to_string(),
            "ws_1".to_string(),
            1704067200000,
            "To Do".to_string(),
            "In Progress".to_string(),
        ),
        InboxActivity::status_change(
            "task_2".to_string(),
            "Feature request".to_string(),
            "ws_1".to_string(),
            1704067300000,
            "In Progress".to_string(),
            "Done".to_string(),
        ),
    ];
    
    let mut inbox = InboxListState::new();
    inbox.set_activities(activities);
    inbox.list_state.select(Some(0));

    let snapshot = capture_inbox(&inbox, 60, 12);
    assert_snapshot!("inbox_widget_status_details", snapshot);
}

/// Test inbox with overdue task display
#[test]
fn test_inbox_widget_overdue_task() {
    use clickdown::models::inbox_activity::InboxActivity;
    
    let now = chrono::Utc::now().timestamp_millis();
    let yesterday = now - (24 * 60 * 60 * 1000);
    
    let activities = vec![
        InboxActivity::due_date(
            "task_1".to_string(),
            "Overdue report".to_string(),
            "ws_1".to_string(),
            yesterday,
        ),
    ];
    
    let mut inbox = InboxListState::new();
    inbox.set_activities(activities);

    let snapshot = capture_inbox(&inbox, 60, 10);
    assert_snapshot!("inbox_widget_overdue_task", snapshot);
}
