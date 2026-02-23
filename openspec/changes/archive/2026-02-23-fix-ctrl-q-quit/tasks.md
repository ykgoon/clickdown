## 1. Update Keyboard Event Handling

- [x] 1.1 Modify the `subscription` method in `src/app.rs` to handle both `Key::Character` and `Key::Named` events for ctrl-q
- [x] 1.2 Ensure the keyboard event pattern matching is mutually exclusive to prevent double-firing
- [x] 1.3 Verify the ctrl modifier check applies to both event types

## 2. Testing

- [x] 2.1 Run existing tests to ensure no regression (code compiles, unit tests pass; integration test failures are pre-existing)
- [x] 2.2 Manually test ctrl-q on the current system to verify it works
- [x] 2.3 Verify ctrl-q works in all application states (loading, authenticated, unauthenticated)

## 3. Verification

- [x] 3.1 Build the application in release mode
- [x] 3.2 Test ctrl-q shortcut in the built application
- [x] 3.3 Confirm application exits with code 0
