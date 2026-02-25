# Tasks: Fix Comment API Field Mapping

## Implementation Tasks

### 1. Update Comment Model ✅
- [x] Add `flexible_string_timestamp` deserializer function (already had `flexible_timestamp`)
- [x] Add `reactions_to_string` deserializer function (simplified to skip for now)
- [x] Update `Comment` struct field rename attributes:
  - [x] `text` → `#[serde(rename = "comment_text")]`
  - [x] `commenter` → `#[serde(rename = "user")]`
  - [x] `created_at` → `#[serde(rename = "date")]`
  - [x] `updated_at` → keep `date_updated`
  - [x] `assigned_commenter` → `#[serde(rename = "assignee")]`
  - [x] `assigned` → `#[serde(rename = "resolved")]`
  - [x] `reaction` → keep as-is (reactions deserialization deferred)
- [x] Run `cargo check` to verify compilation

### 2. Update Test Fixtures ✅
- [x] Test fixtures work correctly (they create Rust objects directly)

### 3. Update Unit Tests ✅
- [x] Update comment deserialization tests in `src/models/comment.rs`
- [x] Add test for string timestamp parsing
- [x] Add test for API response format deserialization
- [x] Update reactions test to document future enhancement

### 4. Update Integration Tests ✅
- [x] All existing tests pass with new field mappings

### 5. Verification ✅
- [x] Run `cargo build` - verify compilation
- [x] Run `cargo test` - all 128 tests pass
- [x] Test with real API: `clickdown debug comments 86d1pk5tc --json`
- [x] Verify comments display correctly in CLI output

## Test Commands

```bash
# Build
cargo build

# Run all tests
cargo test

# Test comment model specifically
cargo test --lib comment

# Test with real API
clickdown debug comments 86d1pk5tc --json

# Run TUI and check comments
cargo run
```

## Acceptance Criteria ✅

- [x] Comments fetched from API have non-empty `text` field
- [x] Comments show actual author username instead of "Anonymous"
- [x] Comments show actual date instead of "Unknown date"
- [x] All existing tests pass (128 tests)
- [x] New tests cover API response format deserialization

## Test Results

```
Comments for task 86d1pk5tc:

[1] Amir Syazwan - Jan 23, 2026 15:08
    This is to ensure OT mode displayed in Present Table and calculated correctly.
```
