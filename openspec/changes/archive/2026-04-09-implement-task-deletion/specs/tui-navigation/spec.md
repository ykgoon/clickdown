## MODIFIED Requirements

### Requirement: Context-aware actions
The system SHALL implement context-aware action keys that change behavior based on the current screen. Inbox view SHALL have its own set of context-aware actions.

#### Scenario: Create new item
- **WHEN** user presses `n` in task list view
- **THEN** task creation form is opened

#### Scenario: Edit item
- **WHEN** user presses `e` on a selected task
- **THEN** task edit form is opened with current values

#### Scenario: Delete item
- **WHEN** user presses `d` on a selected task
- **THEN** confirmation dialog is shown with the task name
- **AND** task is deleted via API on confirmation
- **AND** task is removed from the local task list on success
- **AND** an error status message is shown on failure, task remains in list

#### Scenario: Delete item with no selection
- **WHEN** user presses `d` with no task selected
- **THEN** no action is taken
- **AND** no dialog is shown

#### Scenario: Delete item - cancel
- **WHEN** user presses `d` on a selected task
- **AND** confirmation dialog is shown
- **AND** user presses `Esc` or selects "No"
- **THEN** dialog is dismissed
- **AND** task is NOT deleted

#### Scenario: Clear notification
- **WHEN** user presses `c` on a selected notification in inbox view
- **THEN** the notification is marked as read and removed from the list

#### Scenario: Clear all notifications
- **WHEN** user presses `C` (shift+c) in inbox view
- **THEN** all unread notifications are marked as read
