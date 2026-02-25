# Fix Comment API Field Mapping

## Why

The Comment model in `src/models/comment.rs` uses incorrect field names that don't match the actual ClickUp API response. This causes all comment data to deserialize as null/empty values:

- `text` field receives `""` because API returns `comment_text`
- `commenter` field receives `null` because API returns `user`
- `created_at` field receives `null` because API returns `date` (as string, not i64)
- `assigned_commenter` field receives `null` because API returns `assignee`

Users see "Anonymous - Unknown date" with empty comment bodies even when comments exist in ClickUp.

## What Changes

### 1. Update Comment Model Field Mappings

Fix serde rename attributes to match actual ClickUp API response:

| Current Field | Current Rename | Correct API Field | Fix |
|--------------|----------------|-------------------|-----|
| `text` | (none) | `comment_text` | Add `#[serde(rename = "comment_text")]` |
| `commenter` | (none) | `user` | Add `#[serde(rename = "user")]` |
| `created_at` | `date_created` | `date` (string) | Change to `#[serde(rename = "date")]` + parse string |
| `updated_at` | `date_updated` | (not in API) | Keep for edited comments, may need different field |
| `assigned_commenter` | `assigned_commenter` | `assignee` | Change to `#[serde(rename = "assignee")]` |
| `reaction` | (none) | `reactions` (array) | Handle array → string conversion |

### 2. Handle Nested Comment Structure

API returns `comment` array: `"comment": [{"text": "..."}]`
May need to extract from nested structure or prefer `comment_text` field.

### 3. Update Timestamp Parsing

API returns `date` as string timestamp (e.g., `"1568036964079"`), not i64.
Need flexible string/i64 deserializer.

### 4. Update Tests

Fix test fixtures and add tests for:
- Correct field deserialization from API response format
- String timestamp parsing
- Null/missing field handling

## Capabilities

### Modified Capabilities
- `api-models`: Comment model field mappings corrected to match ClickUp API v2 specification

## Impact

- **Models**: `src/models/comment.rs` - Comment struct field rename attributes updated
- **Tests**: `tests/fixtures.rs` - Test fixtures updated with correct field names
- **Tests**: `tests/app_test.rs` - Comment-related tests updated
- **Tests**: `tests/tui_test.rs` - TUI comment tests updated
- **API**: No breaking changes - this fixes existing broken functionality
