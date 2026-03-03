## MODIFIED Requirements

### Requirement: Go back in hierarchy
The system SHALL implement navigation back to the parent level in the workspace hierarchy. When navigating back, the system SHALL:

1. Change the current screen to the parent screen
2. Repopulate the sidebar with items from the parent level
3. Restore selection to the previously selected item at that level
4. Update the screen title to reflect the parent level

Selection restoration SHALL use the following context tracking:
- When returning from Spaces to Workspaces: select the workspace matching `current_workspace_id`
- When returning from Folders to Spaces: select the space matching `current_space_id`
- When returning from Lists to Folders: select the folder matching `current_folder_id`
- When returning from Tasks to Lists: select the list matching `current_list_id`

If the tracked ID does not match any item (e.g., item was deleted), the system SHALL select the first item as fallback.

#### Scenario: Go back from Spaces to Workspaces
- **WHEN** user is viewing Spaces screen and presses `Esc`
- **THEN** sidebar is repopulated with workspaces
- **AND** the previously selected workspace is highlighted
- **AND** screen title shows "Workspaces"

#### Scenario: Go back from Folders to Spaces
- **WHEN** user is viewing Folders screen and presses `Esc`
- **THEN** sidebar is repopulated with spaces from the current workspace
- **AND** the previously selected space is highlighted
- **AND** screen title shows the space name

#### Scenario: Go back from Lists to Folders
- **WHEN** user is viewing Lists screen and presses `Esc`
- **THEN** sidebar is repopulated with folders from the current space
- **AND** the previously selected folder is highlighted
- **AND** screen title shows the folder name

#### Scenario: Go back from Tasks to Lists
- **WHEN** user is viewing Tasks screen and presses `Esc`
- **THEN** sidebar is repopulated with lists from the current folder
- **AND** the previously selected list is highlighted
- **AND** screen title shows the list name

#### Scenario: Fallback when tracked item not found
- **WHEN** user navigates back and the tracked ID does not match any item
- **THEN** the first item in the list is selected
- **AND** a status message indicates the saved item was not found

### Requirement: Workspace hierarchy navigation
The system SHALL support navigation through the workspace hierarchy (Workspaces → Spaces → Folders → Lists) using keyboard. When navigating into a level, the system SHALL populate the sidebar with items at that level and maintain context for navigation back.

After populating the sidebar via async API responses, the system SHALL restore selection based on existing context IDs if present, otherwise select the first item.

#### Scenario: Navigate into workspace
- **WHEN** user selects a workspace and presses `Enter`
- **THEN** spaces within that workspace are fetched via API
- **AND** sidebar is populated with the spaces
- **AND** first space is selected by default

#### Scenario: Navigate into space with existing context
- **WHEN** user navigates to Spaces screen and `current_space_id` is already set (e.g., from session restore)
- **THEN** spaces are fetched via API
- **AND** sidebar is populated with the spaces
- **AND** the space matching `current_space_id` is selected

#### Scenario: Navigate into space
- **WHEN** user selects a space and presses `Enter`
- **THEN** folders within that space are fetched via API
- **AND** sidebar is populated with the folders
- **AND** first folder is selected by default

#### Scenario: Navigate into folder
- **WHEN** user selects a folder and presses `Enter`
- **THEN** lists within that folder are fetched via API
- **AND** sidebar is populated with the lists
- **AND** first list is selected by default

#### Scenario: Navigate into list
- **WHEN** user selects a list and presses `Enter`
- **THEN** tasks within that list are fetched via API
- **AND** task list view displays the tasks
- **AND** first task is selected by default

#### Scenario: Go back in hierarchy
- **WHEN** user presses `Esc` or `Backspace`
- **THEN** parent level in hierarchy is displayed
- **AND** selection is restored to the item that was previously selected at that level
