## Why

Users cannot navigate back up the workspace hierarchy after drilling down. After selecting a workspace → spaces → folders → lists, pressing Esc to go back changes the screen title but the sidebar remains stale, leaving users unable to select a different workspace/space/folder without restarting the app.

## What Changes

- Fix `navigate_back()` to properly restore the parent navigation context by repopulating the sidebar with the correct items for the parent screen
- Restore selection to the previously selected item at each navigation level using the existing context tracking IDs (`current_workspace_id`, `current_space_id`, `current_folder_id`, `current_list_id`)
- Fix async message handlers (`WorkspacesLoaded`, `SpacesLoaded`, `FoldersLoaded`, `ListsLoaded`) to restore selection based on context IDs after populating sidebar items
- Ensure sidebar displays the correct items matching the current screen after any navigation action

## Capabilities

### New Capabilities
- `navigation-state`: State management for tracking selected items at each navigation level, including the logic to restore selection when switching between navigation screens

### Modified Capabilities
- (none - this is a bug fix in the navigation implementation, not a change to user-facing capabilities or requirements)

## Impact

- `src/tui/app.rs`: 
  - `navigate_back()` method needs to repopulate sidebar and restore selection
  - Async message handlers need to restore selection based on context IDs
- `src/tui/widgets/sidebar.rs`: May need `select_by_id()` enhancement or verification
- `src/tui/helpers.rs`: `SelectableList` already has `select_by()` which should work correctly
- Session restore functionality may benefit from this fix but should not require changes
- No API changes, no breaking changes, no new dependencies
