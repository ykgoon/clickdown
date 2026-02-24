## 1. Remove Credential Authentication Code

- [x] 1.1 Remove `login_view.rs` file from `src/ui/`
- [x] 1.2 Remove `login: login_view::State` field from `ClickDown` struct in `app.rs`
- [x] 1.3 Remove credential-related `Message` variants from enum (`UsernameEntered`, `PasswordEntered`, `ShowPasswordToggled`, `LoginRequested`, `LoginSuccess`, `LoginError`)
- [x] 1.4 Remove credential auth message handlers from `update()` method in `app.rs`
- [x] 1.5 Remove `authenticate_with_credentials()` method from `ClickUpApi` trait in `client_trait.rs`
- [x] 1.6 Remove `authenticate_with_credentials()` implementation from `ClickUpClient` in `client.rs`
- [x] 1.7 Remove `authenticate_with_credentials()` implementation from `MockClickUpClient` in `mock_client.rs`
- [x] 1.8 Remove `auth_credentials_response` field from `MockClickUpClient` struct

## 2. Update Module Exports

- [x] 2.1 Remove `login_view` from `ui/mod.rs` exports
- [x] 2.2 Remove unused imports in `app.rs` related to login_view

## 3. Update Tests

- [x] 3.1 Remove credential-based test fixtures from `tests/fixtures.rs`
- [x] 3.2 Update integration tests in `tests/app_test.rs` to remove credential authentication test cases
- [x] 3.3 Remove mock client credential configuration methods

## 4. Update Documentation

- [x] 4.1 Update `README.md` authentication instructions to reference Personal API Token only
- [x] 4.2 Update `AGENTS.md` authentication context
- [x] 4.3 Add code comments in `client.rs` explaining ClickUp doesn't support password grant

## 5. Verification

- [x] 5.1 Run `cargo build` to ensure no compilation errors
- [x] 5.2 Run `cargo test` to ensure all tests pass
- [x] 5.3 Run `cargo clippy` to ensure no warnings about dead code
- [x] 5.4 Manually test authentication flow with Personal API Token
