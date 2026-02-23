## Why

The codebase currently uses `.unwrap()` and `.expect()` in several places where errors should be handled gracefully. While these calls are unlikely to panic in normal operation, they represent potential crash points that violate Rust best practices for application robustness.

## What Changes

- Replace `.unwrap()` calls with `?` operator or proper error handling
- Convert panic-prone code to graceful error propagation
- Maintain existing behavior while improving safety

## Capabilities

### Modified Capabilities
- `app.rs`: Error handling in load_spaces, load_folders, load_lists_in_folder, load_tasks
- `config/mod.rs`: Error handling in ConfigManager::default()
- `api/auth.rs`: Error handling in AuthManager::default()

## Impact

- `src/app.rs`: 4 unwrap() calls replaced with ? operator
- `src/config/mod.rs`: 1 expect() call replaced with Result propagation
- `src/api/auth.rs`: 1 expect() call replaced with Result propagation
