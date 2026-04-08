## Purpose

This capability defines the user interface for viewing and managing task assignees, including the assignee display in task detail, the picker overlay, keyboard navigation, and the save/cancel flow.

## Requirements

### Requirement: Assignees displayed in task detail
The task detail view SHALL display current assignees as a readable line showing the usernames of all assigned users, separated by commas. If no assignees are set, the line SHALL display "None".

#### Scenario: Task has multiple assignees
- **WHEN** task detail is opened for a task with assignees `[User { username: "Alice" }, User { username: "Bob" }]`
- **THEN** the assignees line displays "Assignees: Alice, Bob"

#### Scenario: Task has no assignees
- **WHEN** task detail is opened for a task with an empty assignees list
- **THEN** the assignees line displays "Assignees: None"

#### Scenario: Task has single assignee
- **WHEN** task detail is opened for a task with one assignee `[User { username: "Alice" }]`
- **THEN** the assignees line displays "Assignees: Alice"

### Requirement: Assignee picker overlay
The system SHALL render an assignee picker overlay dialog when triggered from the task detail screen. The overlay SHALL display a selectable list of all members who can access the task's parent list, with checkbox indicators showing current assignment state.

#### Scenario: Picker displays list members
- **WHEN** assignee picker is opened
- **THEN** it shows all members of the task's parent list
- **AND** each member entry displays their username and email (if available)
- **AND** currently assigned members are shown with a checked indicator (e.g., `[x]`)
- **AND** unassigned members are shown with an unchecked indicator (e.g., `[ ]`)

#### Scenario: Picker highlights current selection
- **WHEN** picker is opened for a task already assigned to Alice and Bob
- **THEN** Alice and Bob entries show `[x]` (checked)
- **AND** all other members show `[ ]` (unchecked)
- **AND** cursor starts on the first member

### Requirement: Picker keyboard navigation
The assignee picker SHALL support keyboard navigation: `j` moves cursor down, `k` moves cursor up, `Space` toggles the checkbox on the current member, `Ctrl+S` saves and closes, `Esc` cancels without saving.

#### Scenario: Navigate down
- **WHEN** cursor is on member at index 0
- **AND** user presses `j`
- **THEN** cursor moves to member at index 1
- **AND** the selection highlight updates

#### Scenario: Navigate up
- **WHEN** cursor is on member at index 1
- **AND** user presses `k`
- **THEN** cursor moves to member at index 0

#### Scenario: Toggle assignment
- **WHEN** cursor is on a member with `[ ]` (unchecked)
- **AND** user presses `Space`
- **THEN** the member's checkbox changes to `[x]` (checked)
- **AND** cursor does not move

#### Scenario: Toggle off assignment
- **WHEN** cursor is on a member with `[x]` (checked)
- **AND** user presses `Space`
- **THEN** the member's checkbox changes to `[ ]` (unchecked)

#### Scenario: Cancel picker
- **WHEN** user presses `Esc`
- **THEN** picker closes without making API calls
- **AND** task assignees remain unchanged
- **AND** status bar shows "Assignment cancelled"

### Requirement: Save assignee changes
When the user presses `Ctrl+S` in the picker, the system SHALL send an `UpdateTaskRequest` with the new list of assignee user IDs to the ClickUp API. On success, the task detail SHALL refresh with updated assignees. On failure, an error message SHALL be displayed and the picker SHALL remain open.

#### Scenario: Successful assignment update
- **WHEN** user toggles members and presses `Ctrl+S`
- **THEN** system sends `PUT /task/{task_id}` with `assignees: [user_ids...]`
- **AND** on 200 response, picker closes
- **AND** task detail refreshes to show updated assignees
- **AND** status bar shows "Assignees updated"

#### Scenario: Failed assignment update
- **WHEN** user toggles members and presses `Ctrl+S`
- **AND** API returns an error
- **THEN** picker remains open
- **AND** status bar shows "Failed to update assignees: {error}"
- **AND** task assignees remain unchanged

#### Scenario: Empty assignee list
- **WHEN** user deselects all members and presses `Ctrl+S`
- **THEN** system sends `PUT /task/{task_id}` with `assignees: []`
- **AND** task assignees are cleared
- **AND** task detail shows "Assignees: None"

### Requirement: Assignee picker trigger
The picker SHALL be triggered by pressing `A` while viewing the task detail screen. The picker SHALL only be available when a task is selected and a list context is known (i.e., the task's parent list ID is available in application state).

#### Scenario: Open picker from task detail
- **WHEN** user is on task detail screen
- **AND** presses `A`
- **THEN** assignee picker overlay opens

#### Scenario: Picker requires list context
- **WHEN** user presses `A` but no list context is available
- **THEN** status bar shows "Cannot assign: list context not available"
- **AND** picker does not open

### Requirement: Multi-select assignment
The picker SHALL support selecting and deselecting multiple members simultaneously. The save operation SHALL replace the entire assignees list with the newly selected set.

#### Scenario: Add and remove in same session
- **WHEN** task has assignees [Alice]
- **AND** user checks Bob and unchecks Alice
- **AND** presses `Ctrl+S`
- **THEN** task assignees become [Bob] (Alice removed, Bob added)

#### Scenario: Add multiple new assignees
- **WHEN** task has no assignees
- **AND** user checks Alice, Bob, and Charlie
- **AND** presses `Ctrl+S`
- **THEN** task assignees become [Alice, Bob, Charlie]
