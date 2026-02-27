## MODIFIED Requirements

### Requirement: Context-aware actions
The system SHALL implement context-aware action keys that change behavior based on the current screen.

#### Scenario: Create new item
- **WHEN** user presses `n` in task list view
- **THEN** task creation form is opened

#### Scenario: Edit item
- **WHEN** user presses `e` on a selected task
- **THEN** task edit form is opened with current values

#### Scenario: Delete item
- **WHEN** user presses `d` on a selected task
- **THEN** confirmation dialog is shown
- **AND** task is deleted on confirmation

#### Scenario: Copy element URL
- **WHEN** user presses `Ctrl+Shift+C` (or `Cmd+Shift+C` on macOS) in any view
- **THEN** the system determines the most relevant URL based on current context:
  - Workspace list: copies selected workspace URL
  - Space list: copies selected space URL
  - Folder list: copies selected folder URL
  - List view: copies selected list URL
  - Task list: copies selected task URL
  - Task detail: copies current task URL
  - Comment thread: copies selected comment URL (with task context)
  - Document view: copies current document URL
- **AND** the URL is copied to the system clipboard
- **AND** visual feedback is displayed showing the copied URL (truncated if long)
- **AND** if no item is selected, a message "No item selected" is shown
