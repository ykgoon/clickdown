//! Task list widget

use crate::models::task::{get_status_group_priority, resolve_status_group, sort_tasks, StatusGroupPriority};
use crate::models::Task;
use crate::tui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
    Frame,
};

/// A row in the task list — either a status group header or a task
#[derive(Debug, Clone)]
pub enum ListRow {
    Header { label: String, #[allow(dead_code)] count: usize },
    Task(Task),
}

/// Grouped task list state, replacing the flat `TaskListState`
///
/// Rows are a flat sequence of `ListRow::Header` and `ListRow::Task` items.
/// Navigation (j/k) skips header rows and only selects task rows.
#[derive(Debug, Clone)]
pub struct GroupedTaskList {
    rows: Vec<ListRow>,
    list: ListState,
}

impl GroupedTaskList {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            list: ListState::default(),
        }
    }

    /// Build a grouped task list from a flat vector of sorted tasks.
    ///
    /// Tasks are grouped by their resolved status group name (falling back to
    /// `status.status` when `status_group` is `None`), sorted within groups by
    /// `updated_at` descending. Known status groups (in_progress, todo, done)
    /// appear first in priority order; unknown groups appear after.
    pub fn from_tasks(mut tasks: Vec<Task>) -> Self {
        // Ensure tasks are sorted by status priority and recency
        tasks = sort_tasks(tasks);

        // Group tasks by resolved status group name (String key)
        // Each group tracks its priority (for ordering) and the task list
        let mut groups: Vec<(String, StatusGroupPriority, Vec<Task>)> = Vec::new();
        for task in tasks {
            let group_name = resolve_status_group(task.status.as_ref());
            let priority = get_status_group_priority(&group_name);
            // Find existing group or create new one
            if let Some((_, _, group_tasks)) = groups.iter_mut().find(|(name, _, _)| *name == group_name) {
                group_tasks.push(task);
            } else {
                groups.push((group_name, priority, vec![task]));
            }
        }

        // Sort groups: known priorities first (InProgress < ToDo < Done), then unknown groups alphabetically
        groups.sort_by(|(name_a, prio_a, _), (name_b, prio_b, _)| {
            prio_a.cmp(prio_b).then_with(|| name_a.cmp(name_b))
        });

        // Build rows: header + tasks for each non-empty group
        let mut rows: Vec<ListRow> = Vec::new();
        for (group_name, _priority, group_tasks) in &groups {
            let label = format_group_label(group_name, group_tasks.len());
            rows.push(ListRow::Header {
                label,
                count: group_tasks.len(),
            });
            for task in group_tasks {
                rows.push(ListRow::Task(task.clone()));
            }
        }

        // Select first task (skip headers)
        let first_task_index = rows
            .iter()
            .position(|r| matches!(r, ListRow::Task(_)));

        let mut list = ListState::default();
        list.select(first_task_index);

        Self { rows, list }
    }

    /// Move selection to the next task row, skipping header rows.
    pub fn select_next(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let current = self.list.selected().unwrap_or(0);
        let len = self.rows.len();

        // Find next task row, skipping headers, with wrapping
        for offset in 1..=len {
            let next = (current + offset) % len;
            if matches!(self.rows[next], ListRow::Task(_)) {
                self.list.select(Some(next));
                return;
            }
        }
    }

    /// Move selection to the previous task row, skipping header rows.
    pub fn select_previous(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let current = self.list.selected().unwrap_or(0);
        let len = self.rows.len();

        // Find previous task row, skipping headers, with wrapping
        for offset in 1..=len {
            let prev = if current >= offset {
                current - offset
            } else {
                len - (offset - current)
            };
            if matches!(self.rows[prev], ListRow::Task(_)) {
                self.list.select(Some(prev));
                return;
            }
        }
    }

    /// Select the first task row, skipping header rows.
    pub fn select_first(&mut self) {
        for (i, row) in self.rows.iter().enumerate() {
            if matches!(row, ListRow::Task(_)) {
                self.list.select(Some(i));
                return;
            }
        }
    }

    /// Get the currently selected task, or None if selection is on a header or empty.
    pub fn selected_task(&self) -> Option<&Task> {
        self.list
            .selected()
            .and_then(|i| self.rows.get(i))
            .and_then(|row| match row {
                ListRow::Task(task) => Some(task),
                ListRow::Header { .. } => None,
            })
    }

    /// Select item by index (raw index into rows, may select a header — callers should prefer navigation methods)
    pub fn select(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i < self.rows.len() {
                self.list.select(Some(i));
            }
        } else {
            self.list.select(None);
        }
    }

    /// Get all rows (for rendering)
    pub fn rows(&self) -> &[ListRow] {
        &self.rows
    }

    /// Get all rows (mutable, for rendering)
    #[allow(dead_code)]
    pub fn rows_mut(&mut self) -> &mut Vec<ListRow> {
        &mut self.rows
    }

    /// Get the internal ListState for rendering
    pub fn state(&self) -> &ListState {
        &self.list
    }

    /// Get mutable access to the list state for rendering
    #[allow(dead_code)]
    pub fn state_mut(&mut self) -> &mut ListState {
        &mut self.list
    }
}

impl Default for GroupedTaskList {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a group label for display from the resolved status group name.
/// e.g., `"in progress"` with count 3 → "▸ IN PROGRESS (3)"
///       `"review"` with count 2 → "▸ REVIEW (2)"
fn format_group_label(group_name: &str, count: usize) -> String {
    format!("▸ {} ({})", group_name.to_uppercase(), count)
}

/// Type alias for backwards compatibility — use `GroupedTaskList` directly.
#[allow(dead_code)]
#[deprecated(note = "Use GroupedTaskList instead")]
pub type TaskListState = GroupedTaskList;

/// Get status color
#[allow(dead_code)]
fn get_status_color(status: &Option<crate::models::TaskStatus>) -> Color {
    match status {
        Some(s) => match s.status.to_lowercase().as_str() {
            "complete" | "done" => Theme::TASK_STATUS_COMPLETE,
            "in progress" => Theme::TASK_STATUS_IN_PROGRESS,
            "todo" => Theme::TASK_STATUS_TODO,
            _ => Theme::TASK_STATUS_OTHER,
        },
        None => Theme::TASK_STATUS_OTHER,
    }
}

/// Get priority indicator
fn get_priority_indicator(priority: &Option<crate::models::Priority>) -> &'static str {
    match priority {
        Some(p) => match p.priority.as_str() {
            "urgent" => "⚡",
            "high" => "↑",
            "low" => "↓",
            _ => "•",
        },
        None => "•",
    }
}

pub fn render_task_list(frame: &mut Frame, state: &GroupedTaskList, area: Rect, loading: bool) {
    // Show loading indicator if loading
    if loading {
        use ratatui::widgets::Paragraph;
        let loading_text = Paragraph::new("Loading assigned tasks...")
            .style(Style::default().fg(Theme::WARNING))
            .block(crate::tui::layout::titled_block(" Tasks "));
        frame.render_widget(loading_text, area);
        return;
    }

    let items: Vec<ListItem> = state
        .rows()
        .iter()
        .map(|row| match row {
            ListRow::Header { label, .. } => {
                // Header rows: dimmed, bold text, no highlight symbol
                ListItem::new(Line::from(vec![Span::styled(
                    label.as_str(),
                    Style::default()
                        .fg(Theme::TEXT_DIM)
                        .add_modifier(Modifier::BOLD),
                )]))
            }
            ListRow::Task(task) => {
                let priority = get_priority_indicator(&task.priority);

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", priority),
                        Style::default().fg(Theme::WARNING),
                    ),
                    Span::raw(task.name.as_str()),
                ]))
            }
        })
        .collect();

    let list = List::new(items)
        .block(crate::tui::layout::titled_block(" Tasks "))
        .highlight_style(
            Style::default()
                .bg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut state.state().clone());
}

#[allow(dead_code)]
pub fn get_task_list_hints() -> &'static str {
    "j/k: Navigate | Enter: View | n: New | e: Edit | d: Delete"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskStatus;

    fn make_task(id: &str, status_group: Option<&str>, updated_at: Option<i64>) -> Task {
        Task {
            id: id.to_string(),
            name: format!("Task {}", id),
            status: Some(TaskStatus {
                id: None,
                status: "test".to_string(),
                color: None,
                type_field: None,
                orderindex: None,
                status_group: status_group.map(|s| s.to_string()),
            }),
            updated_at,
            ..Default::default()
        }
    }

    /// Test 5.1: from_tasks groups tasks correctly by status_group with proper order
    #[test]
    fn test_from_tasks_groups_correctly() {
        let tasks = vec![
            make_task("t1", Some("todo"), Some(1000)),
            make_task("t2", Some("in_progress"), Some(2000)),
            make_task("t3", Some("done"), Some(1500)),
            make_task("t4", Some("in_progress"), Some(1000)),
        ];

        let grouped = GroupedTaskList::from_tasks(tasks);
        let rows = grouped.rows();

        // Expect: IN_PROGRESS header, t2, t4, TODO header, t1, DONE header, t3
        assert_eq!(rows.len(), 7);

        // First group: In Progress (highest priority)
        assert!(matches!(&rows[0], ListRow::Header { label, .. } if label.contains("IN_PROGRESS")));
        assert!(matches!(&rows[1], ListRow::Task(t) if t.id == "t2"));
        assert!(matches!(&rows[2], ListRow::Task(t) if t.id == "t4"));

        // Second group: Todo
        assert!(matches!(&rows[3], ListRow::Header { label, .. } if label.contains("TODO")));
        assert!(matches!(&rows[4], ListRow::Task(t) if t.id == "t1"));

        // Third group: Done
        assert!(matches!(&rows[5], ListRow::Header { label, .. } if label.contains("DONE")));
        assert!(matches!(&rows[6], ListRow::Task(t) if t.id == "t3"));
    }

    /// Test 5.2: empty groups are not rendered
    #[test]
    fn test_from_tasks_skips_empty_groups() {
        let tasks = vec![
            make_task("t1", Some("done"), Some(1000)),
            make_task("t2", Some("todo"), Some(1000)),
        ];

        let grouped = GroupedTaskList::from_tasks(tasks);
        let rows = grouped.rows();

        // Only Todo and Done groups (no In Progress or Other)
        assert_eq!(rows.len(), 4);
        assert!(matches!(&rows[0], ListRow::Header { label, .. } if label.contains("TODO")));
        assert!(matches!(&rows[2], ListRow::Header { label, .. } if label.contains("DONE")));
    }

    #[test]
    fn test_from_tasks_empty_list() {
        let tasks: Vec<Task> = vec![];
        let grouped = GroupedTaskList::from_tasks(tasks);
        assert!(grouped.rows().is_empty());
        assert!(grouped.selected_task().is_none());
    }

    /// Test 5.3: select_next skips header rows
    #[test]
    fn test_select_next_skips_headers() {
        let tasks = vec![
            make_task("t1", Some("in_progress"), Some(1000)),
            make_task("t2", Some("todo"), Some(1000)),
        ];

        let mut grouped = GroupedTaskList::from_tasks(tasks);

        // t1 is selected (first task in In Progress group)
        assert!(matches!(grouped.selected_task(), Some(t) if t.id == "t1"));

        // Move next — should skip Todo header and select t2
        grouped.select_next();
        assert!(matches!(grouped.selected_task(), Some(t) if t.id == "t2"));
    }

    /// Test 5.4: select_previous skips header rows and wraps correctly
    #[test]
    fn test_select_previous_skips_headers_and_wraps() {
        let tasks = vec![
            make_task("t1", Some("in_progress"), Some(1000)),
            make_task("t2", Some("todo"), Some(1000)),
        ];

        let mut grouped = GroupedTaskList::from_tasks(tasks);

        // Move next to t2
        grouped.select_next();
        assert!(matches!(grouped.selected_task(), Some(t) if t.id == "t2"));

        // Move previous — should skip Todo->Done boundary (just In Progress header)
        // and go back to t1
        grouped.select_previous();
        assert!(matches!(grouped.selected_task(), Some(t) if t.id == "t1"));

        // Move previous from first task — should wrap to last task (t2)
        grouped.select_previous();
        assert!(matches!(grouped.selected_task(), Some(t) if t.id == "t2"));
    }

    /// Test 5.5: selected_task returns None when selection is on a header
    #[test]
    fn test_selected_task_none_on_header() {
        let tasks = vec![make_task("t1", Some("todo"), Some(1000))];
        let mut grouped = GroupedTaskList::from_tasks(tasks);

        // t1 is selected
        assert!(grouped.selected_task().is_some());

        // Manually select the header (index 0)
        grouped.select(Some(0));
        assert!(grouped.selected_task().is_none());

        // select_next should still work and select t1
        grouped.select_next();
        assert!(matches!(grouped.selected_task(), Some(t) if t.id == "t1"));
    }

    #[test]
    fn test_select_first_skips_headers() {
        let tasks = vec![
            make_task("t1", Some("in_progress"), Some(2000)),
            make_task("t2", Some("todo"), Some(1000)),
        ];

        let mut grouped = GroupedTaskList::new();
        // Start with no selection
        assert!(grouped.selected_task().is_none());

        grouped = GroupedTaskList::from_tasks(tasks);
        // from_tasks calls select_first internally, so t1 should be selected
        assert!(matches!(grouped.selected_task(), Some(t) if t.id == "t1"));
    }

    #[test]
    fn test_single_group_list() {
        let tasks = vec![
            make_task("t1", Some("todo"), Some(2000)),
            make_task("t2", Some("todo"), Some(1000)),
        ];

        let grouped = GroupedTaskList::from_tasks(tasks);
        let rows = grouped.rows();

        // One header + two tasks
        assert_eq!(rows.len(), 3);
        assert!(matches!(&rows[0], ListRow::Header { label, count } if label.contains("TODO") && *count == 2));
        assert!(matches!(&rows[1], ListRow::Task(t) if t.id == "t1"));
        assert!(matches!(&rows[2], ListRow::Task(t) if t.id == "t2"));
    }

    #[test]
    fn test_wrapping_past_last_task() {
        let tasks = vec![
            make_task("t1", Some("in_progress"), Some(1000)),
            make_task("t2", Some("todo"), Some(1000)),
        ];

        let mut grouped = GroupedTaskList::from_tasks(tasks);
        // t1 selected
        grouped.select_next(); // t2 selected

        // From t2, next should wrap to t1 (skipping headers)
        grouped.select_next();
        assert!(matches!(grouped.selected_task(), Some(t) if t.id == "t1"));
    }

    #[test]
    fn test_fallback_group() {
        let tasks = vec![
            make_task("t1", Some("unknown_status"), Some(1000)),
            make_task("t2", None, Some(1000)),
        ];

        let grouped = GroupedTaskList::from_tasks(tasks);
        let rows = grouped.rows();

        // Each unknown status gets its own header (no longer collapsed into "OTHER")
        // t1 has status_group="unknown_status", t2 has status_group=None → resolves to "test"
        // Both have Fallback priority so they sort alphabetically: "test" < "unknown_status"
        assert_eq!(rows.len(), 4); // 2 headers + 2 tasks
        assert!(matches!(&rows[0], ListRow::Header { label, .. } if label.contains("TEST")));
        assert!(matches!(&rows[2], ListRow::Header { label, .. } if label.contains("UNKNOWN_STATUS")));
    }

    /// Test that tasks with `status_group: None` but different `status.status` values
    /// are grouped into distinct headers (using status.status as fallback), and that
    /// arbitrary/unknown status groups also get their own headers rather than collapsing
    /// into a single "OTHER" bucket.
    #[test]
    fn test_grouping_uses_status_fallback_and_preserves_distinct_groups() {
        use crate::models::task::TaskStatus;

        fn make_task_with_status(id: &str, status_name: &str, updated_at: i64) -> Task {
            Task {
                id: id.to_string(),
                name: format!("Task {}", id),
                status: Some(TaskStatus {
                    id: None,
                    status: status_name.to_string(),
                    color: None,
                    type_field: None,
                    orderindex: None,
                    status_group: None, // status_group is None, only status.status is set
                }),
                updated_at: Some(updated_at),
                ..Default::default()
            }
        }

        let tasks = vec![
            make_task_with_status("t1", "todo", 1000),
            make_task_with_status("t2", "in progress", 2000),
            make_task_with_status("t3", "done", 1500),
            make_task_with_status("t4", "review", 1800),
            make_task_with_status("t5", "blocked", 900),
        ];

        let grouped = GroupedTaskList::from_tasks(tasks);
        let rows = grouped.rows();

        // Should have 5 distinct status headers, NOT collapsed into one "OTHER"
        let header_count = rows
            .iter()
            .filter(|r| matches!(r, ListRow::Header { .. }))
            .count();

        assert_eq!(
            header_count, 5,
            "Expected 5 distinct status headers (IN PROGRESS, REVIEW, TODO, DONE, BLOCKED), \
             not collapsed into a single OTHER header. Rows: {:?}",
            rows.iter()
                .filter_map(|r| match r {
                    ListRow::Header { label, .. } => Some(label.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        );

        // Verify known-status headers appear in priority order before unknown ones
        // In Progress (priority 1) should come before Todo (priority 2), before Done (priority 3)
        let headers: Vec<&str> = rows
            .iter()
            .filter_map(|r| match r {
                ListRow::Header { label, .. } => Some(label.as_str()),
                _ => None,
            })
            .collect();

        // Known groups should be in priority order: IN PROGRESS < TODO < DONE
        let in_progress_idx = headers.iter().position(|h| h.contains("IN PROGRESS")).unwrap();
        let todo_idx = headers.iter().position(|h| h.contains("TODO")).unwrap();
        let done_idx = headers.iter().position(|h| h.contains("DONE")).unwrap();

        assert!(
            in_progress_idx < todo_idx,
            "IN PROGRESS should appear before TODO"
        );
        assert!(
            todo_idx < done_idx,
            "TODO should appear before DONE"
        );

        // Unknown groups (REVIEW, BLOCKED) should appear after known groups
        let review_idx = headers.iter().position(|h| h.contains("REVIEW")).unwrap();
        let blocked_idx = headers.iter().position(|h| h.contains("BLOCKED")).unwrap();
        let last_known = done_idx.max(todo_idx.max(in_progress_idx));
        assert!(
            review_idx > last_known,
            "REVIEW should appear after known status groups"
        );
        assert!(
            blocked_idx > last_known,
            "BLOCKED should appear after known status groups"
        );
    }
}
