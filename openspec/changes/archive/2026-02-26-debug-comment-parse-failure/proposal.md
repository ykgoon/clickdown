## Why

When saving comment replies in the TUI, users may encounter a "failed to parse" error after the comment has been successfully posted to the ClickUp API. This prevents the reply from appearing in the UI despite being created, causing confusion and data loss concerns. We need to diagnose and fix the root cause of this parsing failure.

## What Changes

- **Investigation**: Use the newly added CLI debug commands to reproduce and diagnose the parsing failure
- **Root Cause Analysis**: Identify which Comment field(s) are causing serde deserialization to fail
- **Fix**: Update the Comment model's deserializers to handle the actual API response format
- **Testing**: Add test cases for the specific API response format that was failing
- **Documentation**: Document known API response variations and how they're handled

## Capabilities

### New Capabilities

- `comment-parsing-robustness`: Improved error handling and logging for comment deserialization failures, including better diagnostics when parsing fails

### Modified Capabilities

- `comment-data-model`: Update Comment struct deserializers to handle edge cases in ClickUp API responses (e.g., missing fields, type variations, unexpected null values)

## Impact

- **Models**: `src/models/comment.rs` - May need additional deserializers or field type changes
- **Error Handling**: Improved error messages to help diagnose future parsing issues
- **Testing**: New test cases for edge cases discovered during investigation
- **Logging**: Enhanced debug logging for comment API responses (already in place via CLI debug mode)
