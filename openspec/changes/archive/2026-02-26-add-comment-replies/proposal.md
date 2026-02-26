## Why

The current comment threading UI and navigation infrastructure exists in the codebase, but the ability to actually create replies to comments is not working. Users can view threads and navigate them, but cannot respond to comments, which breaks the core collaboration workflow.

## What Changes

- **Fix reply creation**: Enable the 'r' key in thread view to create replies with proper `parent_id` linkage
- **Fix API integration**: Ensure reply creation sends the correct `parent_id` field to ClickUp API
- **Fix UI feedback**: Add proper status messages and visual indicators when creating replies
- **Improve error handling**: Handle API errors gracefully when reply creation fails
- **Add validation**: Prevent empty replies and provide clear error messages

## Capabilities

### New Capabilities

- `comment-reply-creation`: Enable creating replies to existing comments with proper parent_id linkage
- `comment-reply-api-integration`: API client methods and request handling for threaded comment creation

### Modified Capabilities

- `task-comments`: Update comment creation logic to properly handle parent_id for replies vs top-level comments
- `comment-threading-ui`: Fix reply form UI and integrate reply creation action into thread view

## Impact

- **Code**: `src/tui/app.rs` (comment creation logic), `src/api/client.rs` (API endpoints), `src/models/comment.rs` (request models)
- **API**: ClickUp API comment creation endpoint with parent_id parameter
- **UI**: Comment panel in task detail view, reply form behavior
- **Existing features**: No breaking changes - enhances existing comment threading infrastructure
