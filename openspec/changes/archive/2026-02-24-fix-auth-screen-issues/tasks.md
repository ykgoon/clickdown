## 1. Fix Password Visibility Toggle

- [x] 1.1 Update `login_view.rs` to wire `state.show_password` to TextInput's `secure()` method
- [x] 1.2 Change password input from `.secure(true)` to `.secure(!state.show_password)`
- [x] 1.3 Test that checking "Show" checkbox reveals password characters
- [x] 1.4 Test that unchecking "Show" checkbox masks password characters

## 2. Fix Tab Navigation

- [x] 2.1 Analyze current widget hierarchy in `login_view.rs` for focus order issues
- [x] 2.2 Restructure password row to maintain proper tab order (input → checkbox → button)
- [x] 2.3 Ensure all inputs are in a single Column for natural focus flow
- [x] 2.4 Test tab order: username → password → show checkbox → login button
- [x] 2.5 Test that Enter key in password field submits the form

## 3. Fix UI Component Alignment

- [x] 3.1 Ensure all labels ("Email", "Password") have consistent left alignment
- [x] 3.2 Ensure all inputs (username, password) have equal width and alignment
- [x] 3.3 Fix password row to keep input aligned with username input above
- [x] 3.4 Ensure "Show" checkbox is vertically centered with password input
- [x] 3.5 Ensure Login button has same width and alignment as inputs
- [x] 3.6 Verify visual alignment on screen c08b

## 4. Fix Authentication Parsing Error

- [x] 4.1 Add logging to capture raw HTTP response in `authenticate_with_credentials()`
- [x] 4.2 Update error handling to catch and log parsing errors separately
- [x] 4.3 Improve error message for parsing failures: "Failed to process authentication response"
- [x] 4.4 Handle case where "token" field is missing from response
- [x] 4.5 Test authentication flow and verify parsing error is resolved
- [x] 4.6 If ClickUp doesn't support password grant, update to show appropriate error message

## 5. Verification

- [x] 5.1 Run `cargo build` and verify no compilation errors
- [x] 5.2 Run `cargo run` and test all four fixes on screen c08b
- [x] 5.3 Verify "Show password" reveals password in plain text
- [x] 5.4 Verify UI components are properly aligned
- [x] 5.5 Verify Tab key moves focus correctly
- [x] 5.6 Verify authentication succeeds without "parsing error"
