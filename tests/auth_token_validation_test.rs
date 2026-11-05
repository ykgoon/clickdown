//! Regression tests for auth token validation and dead-token recovery.
//!
//! Background (2026-11-05): the token file on a user machine was overwritten
//! with a pasted task URL, which silently broke every API call. ClickUp task
//! URLs reported as failing to open were actually failing at the API fetch
//! with 401, surfacing as a misleading "Resource not found". These tests pin
//! the new behavior:
//! - authenticate() rejects non-pk_ input before anything is saved
//! - the token is verified against the API before being persisted
//! - a dead stored token routes the user back to the auth screen
//! - URL navigation reports token problems as re-auth guidance

use clickdown::api::mock_client::MockClickUpClient;
use clickdown::tui::app::{Screen, TuiApp};
use clickdown::tui::input::InputEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Real-world task URL from the original failure report.
const TASK_URL: &str = "https://app.clickup.com/t/26408409/14zrc6qa6qb";

fn pump(app: &mut TuiApp, rounds: usize) {
    for _ in 0..rounds {
        std::thread::sleep(Duration::from_millis(20));
        app.process_async_messages();
    }
}

/// A pasted task URL is not a token: reject before any save or network call.
#[test]
fn test_authenticate_rejects_task_url_as_token() {
    let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();
    app.set_auth_token_for_test(TASK_URL);
    app.authenticate();
    let err = app.auth_error_for_test().expect("auth error must be set");
    assert!(err.contains("pk_"), "error should mention pk_ tokens: {}", err);
    assert_eq!(app.status_message(), "Authentication failed");
}

/// Arbitrary non-token text is rejected the same way.
#[test]
fn test_authenticate_rejects_non_pk_text() {
    let mut app = TuiApp::with_client(Arc::new(MockClickUpClient::new())).unwrap();
    app.set_auth_token_for_test("hello world");
    app.authenticate();
    assert!(app.auth_error_for_test().is_some());
    assert_eq!(app.status_message(), "Authentication failed");
}

/// A pk_-prefixed token is verified against the API first; on success the
/// app enters Main with workspaces loaded. The on-disk token file must NOT
/// be touched by test constructors (manage_stored_token=false).
#[test]
fn test_authenticate_valid_token_verifies_then_enters_main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock = MockClickUpClient::new().with_workspaces(Vec::new());
        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        app.set_auth_token_for_test("pk_test_valid_token");
        app.authenticate();
        assert!(app.auth_error_for_test().is_none());
        pump(&mut app, 30);
        assert_eq!(app.screen(), Screen::Workspaces);
        assert!(!app.auth_invalid_for_test());
        assert!(app.status_message().contains("Authenticated"));
    });
}

/// A token the API rejects is never persisted and the user lands back on
/// the auth screen with explicit guidance.
#[test]
fn test_authenticate_rejected_token_returns_to_auth() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock = MockClickUpClient::new().with_workspaces_error(
            "API error (401 Unauthorized): {\"err\":\"Oauth token not found\"}".to_string(),
        );
        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        app.set_auth_token_for_test("pk_dead_token");
        app.authenticate();
        pump(&mut app, 30);
        assert_eq!(app.screen(), Screen::Auth);
        assert!(app.auth_invalid_for_test());
        let err = app.auth_error_for_test().expect("auth error must be set");
        assert!(err.contains("rejected"), "error: {}", err);
    });
}

/// Startup with a dead stored token: the 401 is recognized, the user is
/// routed to the auth screen (token cleared in production), not stranded on
/// a generic error banner.
#[test]
fn test_startup_with_dead_token_recovers_to_auth_screen() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock = MockClickUpClient::new()
            .with_workspaces_error("Token invalid or expired: re-authentication required".to_string());
        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        app.load_workspaces();
        pump(&mut app, 30);
        assert_eq!(app.screen(), Screen::Auth);
        assert!(app.auth_invalid_for_test());
        let err = app.auth_error_for_test().expect("auth error must be set");
        assert!(err.contains("rejected"), "error: {}", err);
    });
}

/// Opening a task URL while the token is dead must tell the user to
/// re-authenticate, not claim the resource was not found.
#[test]
fn test_url_nav_with_dead_token_shows_reauth_message() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut mock = MockClickUpClient::new();
        mock.task_response =
            Some(Err(anyhow::anyhow!("Token invalid or expired: re-authentication required")));
        let mut app = TuiApp::with_client(Arc::new(mock)).unwrap();
        // Drive the app exactly like a user: g u -> paste URL -> Enter.
        app.update(InputEvent::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE)));
        app.update(InputEvent::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE)));
        assert!(app.is_url_input_open(), "g u must open the URL dialog");
        for c in TASK_URL.chars() {
            app.handle_url_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.update(InputEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
        pump(&mut app, 30);
        let status = app.status_message();
        assert!(status.contains("Token invalid"), "status: {}", status);
        assert!(!status.contains("Resource not found"), "status: {}", status);
    });
}
