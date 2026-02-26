## Why

The current comment panel displays all comments in a flat list, making it difficult to follow threaded conversations. Users need to enter specific top-level comments to view and participate in comment threads, improving conversation clarity and usability.

## What Changes

- **Top-level comments view**: Show only top-level comments initially in the comment panel
- **Comment thread navigation**: Users can select and "enter" any top-level comment to view its thread
- **Second-level thread view**: Display nested replies when inside a comment thread
- **Breadcrumb navigation**: Show navigation path (e.g., "Comments > Comment by John") when viewing a thread
- **Back navigation**: Allow users to exit a thread and return to top-level comments view
- **Thread-aware actions**: "New comment" creates a top-level comment; "Reply" creates a nested reply in current thread

## Capabilities

### New Capabilities
- `comment-threading-ui`: UI components and state management for displaying and navigating comment threads
- `comment-thread-navigation`: Keyboard navigation for entering/exiting comment threads and breadcrumb display

### Modified Capabilities
- `task-comments`: Extend comment model to support parent-child relationships and thread viewing

## Impact

- **UI**: Comment panel layout changes to support thread navigation and breadcrumb
- **State**: New state tracking for current thread context (viewing top-level vs. inside thread)
- **Navigation**: Additional keyboard commands for thread navigation (Enter to view thread, Esc to go back)
- **API**: May require fetching comments with thread relationships or nested structure
- **Models**: Comment model may need parent_id field or thread_id for organizing threads
