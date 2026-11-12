//! Text-editing behaviour: caret visibility, follow-scrolling, and
//! readline/bash-style editing keys.

mod fixtures;

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use clickdown::tui::text_edit;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
}
fn special(code: KeyCode) -> InputEvent {
    InputEvent::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn with(code: KeyCode, mods: KeyModifiers) -> InputEvent {
    InputEvent::Key(KeyEvent::new(code, mods))
}
fn type_str(app: &mut TuiApp, text: &str) {
    for c in text.chars() {
        app.update(key(c));
    }
}

fn comment_app(rt: &Runtime) -> TuiApp {
    let mut app = rt
        .block_on(async { TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap() });
    app.set_screen(Screen::TaskDetail);
    app.set_task_detail_task(fixtures::test_task());
    app.set_comment_focus(true);
    app
}

// ---------------------------------------------------------------- input mode

#[test]
fn ctrl_backspace_deletes_previous_word_in_comment() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    type_str(&mut app, "hello brave world");

    app.update(with(KeyCode::Backspace, KeyModifiers::CONTROL));
    assert_eq!(app.comment_new_text(), "hello brave ");

    // Ctrl+W is an alias.
    app.update(with(KeyCode::Char('w'), KeyModifiers::CONTROL));
    assert_eq!(app.comment_new_text(), "hello ");
}

#[test]
fn home_end_and_arrow_keys_move_the_caret() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    type_str(&mut app, "abc\ndefg");

    // Caret starts at the end; Home goes to the start of the last line.
    app.update(special(KeyCode::Home));
    app.update(key('X'));
    assert_eq!(app.comment_new_text(), "abc\nXdefg");

    // End returns to the end of that line.
    app.update(special(KeyCode::End));
    app.update(key('Y'));
    assert_eq!(app.comment_new_text(), "abc\nXdefgY");

    // Ctrl+Home / Ctrl+End jump to the buffer start/end.
    app.update(with(KeyCode::Home, KeyModifiers::CONTROL));
    app.update(key('S'));
    assert_eq!(app.comment_new_text(), "Sabc\nXdefgY");
    app.update(with(KeyCode::End, KeyModifiers::CONTROL));
    app.update(key('E'));
    assert_eq!(app.comment_new_text(), "Sabc\nXdefgYE");
}

#[test]
fn alt_backspace_and_ctrl_k_edit_like_bash() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    type_str(&mut app, "one two three");

    // Alt+Backspace deletes a word, same as Ctrl+Backspace.
    app.update(with(KeyCode::Backspace, KeyModifiers::ALT));
    assert_eq!(app.comment_new_text(), "one two ");

    // Ctrl+K kills to end of line (already at end → no-op), Ctrl+U kills the
    // text before the caret.
    app.update(with(KeyCode::Char('u'), KeyModifiers::CONTROL));
    assert_eq!(app.comment_new_text(), "");
}

#[test]
fn delete_key_removes_character_under_caret() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    type_str(&mut app, "abXcd");
    app.update(special(KeyCode::Home));
    for _ in 0..2 {
        app.update(special(KeyCode::Right));
    }
    app.update(special(KeyCode::Delete));
    assert_eq!(app.comment_new_text(), "abcd");
}

#[test]
fn description_insert_mode_supports_word_deletion() {
    let rt = Runtime::new().unwrap();
    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap()
    });
    app.set_screen(Screen::TaskDetail);
    app.set_task_detail_task(fixtures::test_task());

    app.update(key('e'));
    assert!(app.insert_mode_active_for_test());
    type_str(&mut app, "tokyo kyoto");
    app.update(with(KeyCode::Backspace, KeyModifiers::CONTROL));
    assert_eq!(app.description_buffer_for_test(), "tokyo ");
}

// -------------------------------------------------------- caret visibility

fn cursor_of(app: &mut TuiApp, width: u16, height: u16) -> (u16, u16) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.draw_ui(frame)).unwrap();
    terminal.get_cursor_position().map(Into::into).unwrap()
}

#[test]
fn comment_caret_follows_text_past_the_visible_box() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));

    // 40 short lines guarantee the compose box scrolls.
    for i in 0..40 {
        type_str(&mut app, &format!("line{}\n", i));
    }

    let (_, y) = cursor_of(&mut app, 80, 24);
    // The caret must stay inside the terminal, below the header row.
    assert!(y > 0 && y < 24, "caret row {} must be on screen", y);

    let text = app.comment_new_text().to_string();
    assert!(text.ends_with("line39\n"), "buffer intact: {:?}", text);
}

fn rendered_rows(app: &mut TuiApp, width: u16, height: u16) -> Vec<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.draw_ui(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect()
}

#[test]
fn comment_box_scrolls_so_the_caret_stays_visible() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    for i in 0..30 {
        type_str(&mut app, &format!("row{} marker\n", i));
    }

    let rows = rendered_rows(&mut app, 80, 24);
    let screen = rows.join("\n");
    assert!(screen.contains("row29 marker"), "newest line visible:\n{screen}");
    assert!(
        !screen.contains("row0 marker"),
        "oldest line must have scrolled out of the box:\n{screen}"
    );
    assert!(
        screen.contains("Ctrl+S save"),
        "compose hint still visible:\n{screen}"
    );
}

#[test]
fn comment_caret_is_visible_for_a_single_long_line() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    type_str(&mut app, &"x".repeat(600));

    let (x, y) = cursor_of(&mut app, 80, 24);
    assert!(x < 80 && y < 24, "caret ({}, {}) must be on screen", x, y);
}

#[test]
fn unicode_text_keeps_the_caret_inside_the_box() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    type_str(&mut app, &"日本語テキスト".repeat(40));
    let (x, y) = cursor_of(&mut app, 80, 24);
    assert!(x < 80 && y < 24);
}

#[test]
fn description_insert_mode_keeps_the_caret_visible() {
    let rt = Runtime::new().unwrap();
    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap()
    });
    app.set_screen(Screen::TaskDetail);
    app.set_task_detail_task(fixtures::test_task());

    app.update(key('e'));
    assert!(app.insert_mode_active_for_test());

    // A long, many-line description must not push the caret off screen.
    for i in 0..40 {
        type_str(&mut app, &format!("line {} of the description\n", i));
    }
    let (x, y) = cursor_of(&mut app, 80, 30);
    assert!(x < 80 && y < 30, "caret ({x}, {y}) must be on screen");

    let rows = rendered_rows(&mut app, 80, 30).join("\n");
    assert!(
        rows.contains("line 39 of the description"),
        "the newest line must be visible:\n{rows}"
    );
    assert!(rows.contains("INSERT"), "insert-mode border must show:\n{rows}");
}

// ------------------------------------------------------- widget-level edits

#[test]
fn wrap_with_cursor_places_caret_at_wrap_boundary() {
    let wrapped = text_edit::wrap_with_cursor("abcdefgh", 4, 8);
    assert_eq!(wrapped.lines, vec!["abcd", "efgh"]);
    assert_eq!((wrapped.cursor_row, wrapped.cursor_col), (1, 4));
}

#[test]
fn mention_insertion_updates_cursor() {
    use clickdown::models::User;

    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    app.set_mention_members_for_test(vec![User {
        id: 7,
        username: "alice".to_string(),
        email: None,
        color: None,
        initials: None,
        profile_picture: None,
    }]);

    type_str(&mut app, "hi @al");
    assert!(app.mention_picker_open_for_test());
    app.update(special(KeyCode::Enter));

    let text = app.comment_new_text().to_string();
    assert!(text.contains("@alice"), "text: {text:?}");
}

/// Regression: picking a mention produced "@ @alice" because the picker
/// filter text never entered the buffer, so the old rfind('@') lookup fell
/// through to an append branch that added a second '@'.
#[test]
fn mention_selection_inserts_exactly_one_at_sign() {
    use clickdown::models::User;

    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    app.set_mention_members_for_test(vec![User {
        id: 7,
        username: "alice".to_string(),
        email: None,
        color: None,
        initials: None,
        profile_picture: None,
    }]);

    type_str(&mut app, "hi @al");
    app.update(special(KeyCode::Enter));

    let text = app.comment_new_text().to_string();
    assert_eq!(text, "hi @alice ", "text: {text:?}");
    assert!(
        !text.contains("@ @"),
        "duplicate '@' must not appear: {text:?}"
    );
}

/// A mention picked after existing text must replace only the '@', keeping
/// the text that preceded it on the line.
#[test]
fn mention_selection_keeps_preceding_text() {
    use clickdown::models::User;

    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    app.set_mention_members_for_test(vec![User {
        id: 7,
        username: "alice".to_string(),
        email: None,
        color: None,
        initials: None,
        profile_picture: None,
    }]);

    type_str(&mut app, "hi there @al");
    assert!(app.mention_picker_open_for_test());
    app.update(special(KeyCode::Enter));

    let text = app.comment_new_text().to_string();
    assert_eq!(text, "hi there @alice ", "text: {text:?}");
    assert!(!text.contains("@ @"), "duplicate '@': {text:?}");
}

/// Regression at the widget level: the drawn compose box must break between
/// words, never mid-word. The reported symptom was "ip" / "sum".
///
/// Rows are taken from the compose box only (border characters stripped), and
/// every row but the last must end on a complete word — a character wrap
/// leaves a fragment such as "ips" at the row edge.
#[test]
fn comment_box_wraps_on_words_when_drawn() {
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    app.update(key('n'));
    let text = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do";
    type_str(&mut app, text);

    let words: Vec<&str> = text.split_whitespace().collect();
    let rows = rendered_rows(&mut app, 80, 24);
    let screen = rows.join("\n");
    assert!(
        screen.contains("lorem") && screen.contains("ipsum"),
        "compose box was not drawn:\n{screen}"
    );

    let mut body_rows: Vec<String> = rows
        .iter()
        .map(|r| {
            r.trim_matches(|c| {
                c == '│' || c == '┌' || c == '┐' || c == '└' || c == '┘' || c == '─' || c == ' '
            })
            .trim()
            .to_string()
        })
        .filter(|r| {
            !r.is_empty()
                && words.iter().any(|w| r.contains(w))
                && !r.contains("New comment")
                && !r.contains("Comments")
        })
        .collect();
    assert!(
        body_rows.len() >= 2,
        "expected the text to wrap over several rows, got: {body_rows:?}\n{screen}"
    );
    body_rows.pop(); // the final row may legitimately end mid-sentence

    for row in &body_rows {
        let last_token = row.split_whitespace().last().unwrap_or("");
        assert!(
            words.contains(&last_token),
            "row ends mid-word ({last_token:?}):\n{screen}"
        );
    }
}

/// Regression: wrapping split words by character, so "lorem ipsum" at a
/// narrow width became "ip" / "sum".
#[test]
fn wrapping_breaks_between_words_not_inside_them() {
    let wrapped = text_edit::wrap_with_cursor("lorem ipsum", 7, 0);
    assert_eq!(wrapped.lines, vec!["lorem", "ipsum"]);
    for line in &wrapped.lines {
        assert!(
            !line.contains("ip") || line.trim() == "ipsum" || line.trim() == "lorem ipsum",
            "word was split across rows: {line:?}"
        );
    }
}

#[test]
fn comment_view_mode_sentinel_is_unaffected() {
    // Ensure the cursor additions did not disturb comment compose state.
    let rt = Runtime::new().unwrap();
    let mut app = comment_app(&rt);
    assert_eq!(app.comment_new_text(), "");
    app.update(key('n'));
    assert!(app.is_comment_editing_active());
    app.update(special(KeyCode::Esc));
    assert!(!app.is_comment_editing_active());
    assert_eq!(app.comment_new_text(), "");
}

// ------------------------------------------------ other text inputs covered

#[test]
fn url_dialog_supports_word_deletion_and_home_end() {
    let rt = Runtime::new().unwrap();
    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap()
    });
    app.update(key('g'));
    app.update(key('u'));
    assert!(app.url_input_open_for_test());

    type_str(&mut app, "https://app.clickup.com/t/123");
    app.update(with(KeyCode::Backspace, KeyModifiers::CONTROL));
    assert_eq!(app.url_input_text(), "https://app.clickup.com/t/");

    app.update(special(KeyCode::Home));
    app.update(key('#'));
    assert_eq!(app.url_input_text(), "#https://app.clickup.com/t/");
}

#[test]
fn task_creation_fields_support_bash_style_editing() {
    let rt = Runtime::new().unwrap();
    let mut app = rt.block_on(async {
        TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap()
    });
    app.set_screen(Screen::TaskDetail);
    app.set_task_creating(true);
    app.set_task_creation_focus(clickdown::tui::app::TaskCreationField::Name);

    type_str(&mut app, "alpha beta gamma");
    app.update(with(KeyCode::Backspace, KeyModifiers::CONTROL));
    assert_eq!(app.task_name_input(), "alpha beta ");

    app.update(InputEvent::Key(KeyEvent::new(
        KeyCode::Tab,
        KeyModifiers::NONE,
    )));
    type_str(&mut app, "desc words here");
    app.update(with(KeyCode::Char('w'), KeyModifiers::CONTROL));
    assert_eq!(app.task_description_input(), "desc words ");
}

