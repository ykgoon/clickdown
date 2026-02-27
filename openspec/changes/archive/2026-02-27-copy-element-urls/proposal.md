## Why

Users sometimes need to view ClickUp elements (tasks, comments, workspaces, etc.) in the web app when ClickDown doesn't provide all the details they need. Currently, there's no way to quickly access the web URL for these elements, forcing users to manually search for them in the browser. This change adds the ability to copy web app URLs for ClickUp elements, enabling seamless transition between terminal and web interfaces.

## What Changes

- Add URL generation logic for various ClickUp element types (workspaces, spaces, folders, lists, tasks, comments, documents)
- Add keyboard shortcut to copy element URL to clipboard in relevant views
- Add visual indicator showing when URL copying is available
- Display confirmation feedback when URL is copied
- Support deep links to specific elements (e.g., specific comment in a thread)

## Capabilities

### New Capabilities
- `element-url-copying`: Ability to generate and copy web app URLs for ClickUp elements (tasks, comments, workspaces, spaces, folders, lists, documents)

### Modified Capabilities
- `tui-navigation`: Add new keyboard action for copying URLs (extends navigation patterns)
- `keyboard-shortcuts`: Add new shortcut for URL copying action

## Impact

- **TUI Layer**: New keyboard handler for URL copying action in task detail, comment thread, and workspace navigation views
- **Models**: May need to store additional metadata (e.g., workspace ID) to construct URLs
- **Clipboard**: New dependency for clipboard operations (likely `arboard` crate for cross-platform clipboard access)
- **UI Feedback**: Status bar updates to show copy confirmation
- **No Breaking Changes**: Existing functionality remains unchanged
