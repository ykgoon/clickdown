## 1. API Layer - Credential Authentication

- [x] 1.1 Add new `authenticate_with_credentials(username: &str, password: &str)` method to `ClickUpApi` trait in `src/api/client_trait.rs`
- [x] 1.2 Implement credential authentication in `ClickUpClient` (`src/api/client.rs`) using ClickUp's token exchange endpoint
- [x] 1.3 Add `MockClickUpClient` support for credential authentication in `src/api/mock_client.rs` with configurable success/failure responses
- [x] 1.4 Add authentication error types to handle invalid credentials, network errors, and account issues

## 2. UI - Login View Component

- [x] 2.1 Create new `login_view.rs` module with username and password input fields
- [x] 2.2 Add `LoginState` struct to track username, password, and loading state
- [x] 2.3 Implement login form rendering with "Login" button and loading indicator
- [x] 2.4 Add error message display for authentication failures
- [x] 2.5 Export login view from `src/ui/mod.rs`

## 3. Application State and Messages

- [x] 3.1 Add new message variants: `CredentialsEntered(String, String)`, `LoginRequested`, `LoginSuccess`, `LoginError(String)`
- [x] 3.2 Update `ClickDown` struct to use `login_view::State` instead of `auth_view::State`
- [x] 3.3 Update `Message::TokenEntered` handling to use credential flow instead
- [x] 3.4 Implement credential authentication in `update()` method, calling new API endpoint
- [x] 3.5 Update view routing to show login screen instead of token entry screen

## 4. Integration and Testing

- [x] 4.1 Update integration tests in `tests/app_test.rs` to use credential authentication flow
- [x] 4.2 Add test fixtures for successful and failed credential authentication
- [x] 4.3 Test error scenarios: invalid credentials, network failure, account locked
- [x] 4.4 Verify token storage still works correctly after credential exchange

## 5. Cleanup and Polish

- [x] 5.1 Remove or deprecate token entry UI components from `auth_view.rs`
- [x] 5.2 Update help text and error messages to be user-friendly
- [x] 5.3 Add "Show password" toggle to password input field (optional enhancement)
- [x] 5.4 Update README.md with new authentication flow documentation
