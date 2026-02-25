## Why

The current comment section in the task detail view lacks sufficient vertical space and independent scrolling behavior, making it difficult to browse and read multiple comments. Users need better visibility and navigation when working with tasks that have many comments.

## What Changes

- **Comment section height ratio adjusted**: The vertical space allocation between description panel and comment panel will change from the current ratio to 3:7 (description:comments), giving comments significantly more screen real estate.
- **Independent scrolling panels**: Both description and comment panels will become independently scrollable when content exceeds their allocated space, preventing one panel's overflow from affecting the other.
- **Auto-scroll with selection**: When navigating comments with j/k keys, the comment list will automatically scroll to keep the selected comment visible, improving navigation usability.
- **Scroll indicators**: Visual scroll indicators will be added to both panels to show when content extends beyond the visible area.

## Capabilities

### New Capabilities

- `comment-panel-scrolling`: Independent scrolling behavior for the comment panel with auto-scroll to maintain selection visibility and proper height ratio management.

### Modified Capabilities

- `task-comments`: Requirements updated to specify the 3:7 height ratio between description and comment panels, independent scrolling behavior, and auto-scroll functionality during keyboard navigation.

## Impact

- **TUI Layout**: The `tui/layout.rs` file will need updates to support the new 3:7 height ratio and independent scrollable areas.
- **Task Detail View**: The `tui/widgets/task_detail.rs` component requires modifications to implement independent scrolling and auto-scroll behavior.
- **Navigation Logic**: Keyboard navigation handlers need to trigger auto-scroll when selection moves outside visible bounds.
- **Existing Tests**: Layout and navigation tests may need updates to reflect the new height ratios and scrolling behavior.
