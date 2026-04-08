## ADDED Requirements

### Requirement: Assignee picker widget
The system SHALL provide an `assignee_picker` widget module that renders a checkbox-style list for selecting multiple assignees. The widget SHALL be usable as an overlay on top of other views (task detail, comments).

#### Scenario: Assignee picker renders as overlay
- **WHEN** the assignee picker widget is rendered
- **THEN** it displays a bordered panel titled "Select Assignees"
- **AND** each member is shown with a checkbox indicator (`[x]` or `[ ]`)
- **AND** username and email (if available) are displayed
- **AND** a hint line shows available keyboard shortcuts at the bottom

#### Scenario: Picker keyboard hints
- **WHEN** picker is rendered
- **THEN** the hint line displays "Space: toggle | j/k: navigate | Ctrl+S: save | Esc: cancel"

### Requirement: Assignee picker state
The system SHALL maintain an `AssigneePickerState` struct containing the list of members, the set of currently selected assignee IDs, and the cursor position.

#### Scenario: Picker state initialization
- **WHEN** picker state is created with members list and current task assignees
- **THEN** the state contains all members
- **AND** the selected set matches current task assignee IDs
- **AND** cursor is at position 0

#### Scenario: Toggle selection in state
- **WHEN** `toggle_member(user_id)` is called on a member not in the selected set
- **THEN** the member's ID is added to the selected set
- **AND** calling it again removes the ID from the selected set

## MODIFIED Requirements

### Requirement: Task detail widget
The system SHALL render a task detail widget for viewing and editing task properties. The widget SHALL display all task fields in a form layout.

#### Scenario: Task detail displays fields
- **WHEN** task detail is opened
- **THEN** the following fields are displayed:
  - Task name (editable)
  - Description (editable, multi-line)
  - Status (dropdown/selectable)
  - Priority (dropdown/selectable)
  - Assignees (displayed as comma-separated usernames, "None" if empty)
  - Due date (editable)

#### Scenario: Task detail editing
- **WHEN** user presses `e` or clicks edit
- **THEN** fields become editable
- **AND** changes are saved with `Ctrl+s`

#### Scenario: Task detail actions
- **WHEN** task detail is open
- **THEN** actions are available:
  - Save (`Ctrl+s`)
  - Delete (`d` with confirmation)
  - Close (`Esc`)
  - Open assignee picker (`A`)
