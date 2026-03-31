//! Assigned items view widget (tasks + comments)

use crate::models::{AssignedItem, AssignedItemsFilter};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the assigned items view
pub fn render_assigned_view(
    frame: &mut Frame,
    items: &[AssignedItem],
    selected_index: usize,
    filter: AssignedItemsFilter,
    area: Rect,
    loading: bool,
    error: Option<&str>,
    count: usize,
) {
    // Show loading indicator
    if loading {
        let loading_text = Paragraph::new("Loading assigned items...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Assigned to Me"));
        frame.render_widget(loading_text, area);
        return;
    }

    // Show error message
    if let Some(err) = error {
        let error_text = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Assigned to Me"));
        frame.render_widget(error_text, area);
        return;
    }

    // Filter items based on current filter
    let filtered_items: Vec<&AssignedItem> = match filter {
        AssignedItemsFilter::All => items.iter().collect(),
        AssignedItemsFilter::TasksOnly => {
            items.iter().filter(|item| item.is_task()).collect()
        }
        AssignedItemsFilter::CommentsOnly => {
            items.iter().filter(|item| item.is_comment()).collect()
        }
    };

    // Empty state
    if filtered_items.is_empty() {
        let empty_text = if items.is_empty() {
            "No tasks or comments assigned to you"
        } else {
            match filter {
                AssignedItemsFilter::TasksOnly => "No assigned tasks",
                AssignedItemsFilter::CommentsOnly => "No assigned comments",
                AssignedItemsFilter::All => unreachable!(),
            }
        };

        let empty_paragraph = Paragraph::new(empty_text)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Assigned to Me ({} items)", count)),
            );
        frame.render_widget(empty_paragraph, area);
        return;
    }

    // Build list items
    let list_items: Vec<ListItem> = filtered_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let item_span = match item {
                AssignedItem::Task(task) => {
                    // Task item: ✓ icon + name + status + priority
                    let status_color = get_task_status_color(&task.status);
                    let priority_icon = get_task_priority_icon(&task.priority);
                    
                    Line::from(vec![
                        Span::styled("✓ ", Style::default().fg(Color::Green)),
                        Span::raw(format!("{} ", task.name)),
                        Span::styled(
                            format!("[{}] ", task.status.as_ref().map(|s| s.status.as_str()).unwrap_or("")),
                            Style::default().fg(status_color),
                        ),
                        Span::raw(priority_icon),
                    ])
                }
                AssignedItem::AssignedComment(ac) => {
                    // Comment item: 💬 icon + preview + parent task name
                    let preview = ac.preview(40);
                    let parent_task = ac.task.name.as_deref().unwrap_or("Unknown task");
                    
                    Line::from(vec![
                        Span::styled("💬 ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{} ", preview)),
                        Span::styled(
                            format!("(in: {})", parent_task),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ])
                }
            };

            let style = if idx == selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(item_span).style(style)
        })
        .collect();

    // Create filter tabs
    let filter_label = filter.label();
    let title = format!("Assigned to Me ({} items) | Filter: {}", count, filter_label);

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(list, area);
}

/// Get status color for tasks
fn get_task_status_color(status: &Option<crate::models::TaskStatus>) -> Color {
    match status {
        Some(s) => match s.status.to_lowercase().as_str() {
            "complete" | "done" => Color::Green,
            "in progress" => Color::Yellow,
            "todo" => Color::White,
            _ => Color::Gray,
        },
        None => Color::Gray,
    }
}

/// Get priority icon for tasks
fn get_task_priority_icon(priority: &Option<crate::models::Priority>) -> &'static str {
    match priority {
        Some(p) => match p.priority.as_str() {
            "urgent" => " ⚡",
            "high" => " ↑",
            "low" => " ↓",
            _ => "",
        },
        None => "",
    }
}

/// Handle keyboard input for assigned view
/// Returns true if the key was handled
pub fn handle_assigned_view_input(
    key: ratatui::crossterm::event::KeyCode,
    items: &[AssignedItem],
    selected_index: &mut usize,
    filter: &mut AssignedItemsFilter,
    item_count: usize,
) -> bool {
    use ratatui::crossterm::event::KeyCode;

    match key {
        KeyCode::Char('j') | KeyCode::Down => {
            if *selected_index < item_count.saturating_sub(1) {
                *selected_index += 1;
            }
            true
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if *selected_index > 0 {
                *selected_index -= 1;
            }
            true
        }
        KeyCode::Char('g') => {
            *selected_index = 0;
            true
        }
        KeyCode::Char('G') => {
            *selected_index = item_count.saturating_sub(1);
            true
        }
        KeyCode::Char('f') => {
            // Toggle filter
            *filter = filter.next();
            // Reset selection when filter changes
            *selected_index = 0;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssignedComment, Comment, Task, TaskReference};

    fn create_test_task(id: &str, name: &str) -> AssignedItem {
        AssignedItem::Task(Task {
            id: id.to_string(),
            name: name.to_string(),
            ..Default::default()
        })
    }

    fn create_test_comment(id: &str, text: &str) -> AssignedItem {
        AssignedItem::AssignedComment(AssignedComment {
            comment: Comment {
                id: id.to_string(),
                text: text.to_string(),
                text_preview: String::new(),
                commenter: None,
                created_at: None,
                updated_at: None,
                assigned_commenter: None,
                assigned_by: None,
                assigned: false,
                reaction: String::new(),
                parent_id: None,
            },
            task: TaskReference {
                id: "task123".to_string(),
                name: Some("Parent Task".to_string()),
            },
            assigned_at: None,
        })
    }

    #[test]
    fn test_filter_applies_correctly() {
        let items = vec![
            create_test_task("t1", "Task 1"),
            create_test_comment("c1", "Comment 1"),
            create_test_task("t2", "Task 2"),
        ];

        let all: Vec<_> = match AssignedItemsFilter::All {
            AssignedItemsFilter::All => items.iter().collect(),
            AssignedItemsFilter::TasksOnly => items.iter().filter(|i| i.is_task()).collect(),
            AssignedItemsFilter::CommentsOnly => items.iter().filter(|i| i.is_comment()).collect(),
        };
        assert_eq!(all.len(), 3);

        let tasks: Vec<_> = items.iter().filter(|i| i.is_task()).collect();
        assert_eq!(tasks.len(), 2);

        let comments: Vec<_> = items.iter().filter(|i| i.is_comment()).collect();
        assert_eq!(comments.len(), 1);
    }

    #[test]
    fn test_handle_input_navigation() {
        let items = vec![
            create_test_task("t1", "Task 1"),
            create_test_task("t2", "Task 2"),
            create_test_task("t3", "Task 3"),
        ];
        let mut selected = 0;
        let mut filter = AssignedItemsFilter::All;

        // Test down navigation
        handle_assigned_view_input(
            ratatui::crossterm::event::KeyCode::Char('j'),
            &items,
            &mut selected,
            &mut filter,
            items.len(),
        );
        assert_eq!(selected, 1);

        handle_assigned_view_input(
            ratatui::crossterm::event::KeyCode::Char('j'),
            &items,
            &mut selected,
            &mut filter,
            items.len(),
        );
        assert_eq!(selected, 2);

        // Test boundary
        handle_assigned_view_input(
            ratatui::crossterm::event::KeyCode::Char('j'),
            &items,
            &mut selected,
            &mut filter,
            items.len(),
        );
        assert_eq!(selected, 2); // Should not go past end

        // Test up navigation
        handle_assigned_view_input(
            ratatui::crossterm::event::KeyCode::Char('k'),
            &items,
            &mut selected,
            &mut filter,
            items.len(),
        );
        assert_eq!(selected, 1);
    }

    #[test]
    fn test_handle_input_filter_toggle() {
        let items = vec![create_test_task("t1", "Task 1")];
        let mut selected = 0;
        let mut filter = AssignedItemsFilter::All;

        handle_assigned_view_input(
            ratatui::crossterm::event::KeyCode::Char('f'),
            &items,
            &mut selected,
            &mut filter,
            items.len(),
        );
        assert_eq!(filter, AssignedItemsFilter::TasksOnly);
        assert_eq!(selected, 0); // Reset on filter change

        handle_assigned_view_input(
            ratatui::crossterm::event::KeyCode::Char('f'),
            &items,
            &mut selected,
            &mut filter,
            items.len(),
        );
        assert_eq!(filter, AssignedItemsFilter::CommentsOnly);

        handle_assigned_view_input(
            ratatui::crossterm::event::KeyCode::Char('f'),
            &items,
            &mut selected,
            &mut filter,
            items.len(),
        );
        assert_eq!(filter, AssignedItemsFilter::All);
    }
}
