//! Overlays must fully erase what is behind them.
//!
//! A background colour alone only recolours cells; the glyphs underneath stay
//! and read as a translucent dialog. These tests paint a marker glyph across
//! the whole screen and assert that none survives inside each dialog's rect.

use clickdown::models::User;
use clickdown::models::task::TaskStatus;
use clickdown::tui::widgets::status_picker::render_status_picker;
use clickdown::tui::widgets::{render_assignee_picker, render_mention_picker};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

/// Marker glyph: no dialog renders this, so any survivor is bleed-through.
const MARK: char = '\u{00a7}';

fn statuses() -> Vec<TaskStatus> {
    ["To Do", "In Progress", "Done"]
        .iter()
        .enumerate()
        .map(|(i, n)| TaskStatus {
            id: None,
            status: n.to_string(),
            color: None,
            type_field: None,
            orderindex: Some(i as u32),
            status_group: None,
        })
        .collect()
}

fn users() -> Vec<User> {
    vec![User {
        id: 1,
        username: "alice".to_string(),
        color: None,
        email: Some("a@x.com".to_string()),
        profile_picture: None,
        initials: None,
    }]
}

/// Rendering the background then the overlay must leave no marker glyph
/// inside `inner` (the region the dialog must own).
fn assert_erases(inner: Rect, draw: impl Fn(&mut ratatui::Frame), label: &str) {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 80, 24);
            let rows: Vec<ratatui::text::Line> = (0..24)
                .map(|_| ratatui::text::Line::from(MARK.to_string().repeat(80)))
                .collect();
            frame.render_widget(Paragraph::new(rows).style(Style::default()), area);
            draw(frame);
        })
        .unwrap();

    let buf = terminal.backend().buffer();
    let mut bleed = Vec::new();
    for y in inner.top()..inner.bottom() {
        for x in inner.left()..inner.right() {
            if buf[(x, y)].symbol().starts_with(MARK) {
                bleed.push((x, y));
            }
        }
    }
    assert!(
        bleed.is_empty(),
        "{label} left {} background cell(s) visible, e.g. {:?}",
        bleed.len(),
        &bleed[..bleed.len().min(6)]
    );
}

/// Compute the rect ratatui's percentage layout actually produces, so tests
/// assert against real geometry instead of hand-derived guesses.
fn percent_split(area: Rect, percent: u16, height: u16) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent) / 2),
            Constraint::Percentage(percent),
            Constraint::Percentage((100 - percent) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

#[test]
fn status_picker_is_opaque() {
    // Mirrors render_status_picker: width 40 (capped), height len+4 (min 8).
    let w = 40; // 40 <= width-4 at 80 cols
    let h = (3u16 + 4).clamp(8, 24 - 4);
    let inner = Rect::new((80 - w) / 2 + 1, (24 - h) / 2 + 1, w - 2, h - 2);
    assert_erases(
        inner,
        |frame| {
            render_status_picker(
                frame,
                Rect::new(0, 0, 80, 24),
                &statuses(),
                0,
                Some("To Do"),
                true,
            )
        },
        "status picker",
    );
}

#[test]
fn assignee_picker_is_opaque() {
    // Mirrors render_assignee_picker: width 60, height len+4 (min 8).
    let w = 60; // 60 <= width-4 at 80 cols
    let h = (1u16 + 4).clamp(8, 24 - 4);
    let inner = Rect::new((80 - w) / 2 + 1, (24 - h) / 2 + 1, w - 2, h - 2);
    assert_erases(
        inner,
        |frame| {
            render_assignee_picker(
                frame,
                Rect::new(0, 0, 80, 24),
                &users(),
                &std::collections::HashSet::new(),
                0,
            )
        },
        "assignee picker",
    );
}

#[test]
fn mention_picker_is_opaque() {
    // Mirrors the picker's centred(area, 46, 14) helper.
    let popup = percent_split(Rect::new(0, 0, 80, 24), 46, 14);
    let inner = Rect::new(
        popup.x + 1,
        popup.y + 1,
        popup.width - 2,
        popup.height - 2,
    );
    assert_erases(
        inner,
        |frame| render_mention_picker(frame, Rect::new(0, 0, 80, 24), &users(), 0, ""),
        "mention picker",
    );
}
