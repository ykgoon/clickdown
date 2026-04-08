## ADDED Requirements

### Requirement: Sidebar widget
The system SHALL render a sidebar widget displaying the workspace hierarchy (Workspaces, Spaces, Folders, Lists). The sidebar SHALL support keyboard navigation and selection.

#### Scenario: Sidebar displays hierarchy
- **WHEN** sidebar is visible
- **THEN** workspaces are listed at top level
- **AND** spaces are indented under selected workspace
- **AND** folders are indented under selected space
- **AND** lists are indented under selected folder

#### Scenario: Sidebar selection
- **WHEN** user navigates sidebar with `j/k` keys
- **THEN** current selection is highlighted
- **AND** selection can be expanded/collapsed with `Enter`

#### Scenario: Sidebar scroll
- **WHEN** hierarchy exceeds sidebar height
- **THEN** sidebar scrolls to show selected item
- **AND** scroll indicator is shown

### Requirement: Task list widget
The system SHALL render a task list widget displaying tasks with status and priority indicators. The list SHALL support selection, sorting indicators, and keyboard navigation.

#### Scenario: Task displayed with status
- **WHEN** task list is rendered
- **THEN** each task shows:
  - Task name
  - Status indicator (color-coded: todo, in progress, complete)
  - Priority indicator (urgent, high, normal, low)

#### Scenario: Task selection
- **WHEN** user navigates task list with `j/k` keys
- **THEN** selected task is highlighted
- **AND** task detail can be opened with `Enter`

#### Scenario: Task list scroll
- **WHEN** task list exceeds visible area
- **THEN** list scrolls to keep selection visible
- **AND** scroll position indicator is shown

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

### Requirement: Authentication widget
The system SHALL render an authentication widget for API token entry. The widget SHALL include guidance text and secure password input.

#### Scenario: Auth form displayed
- **WHEN** authentication screen is shown
- **THEN** the following are displayed:
  - Title: "ClickDown - Authentication"
  - Help text: "Get your token from ClickUp Settings → Apps → ClickUp API"
  - Token input field (password type)
  - Connect button

#### Scenario: Token input
- **WHEN** user types in token field
- **THEN** characters are masked with `*`
- **AND** Connect button becomes enabled

#### Scenario: Auth feedback
- **WHEN** Connect is pressed
- **THEN** loading indicator is shown
- **AND** success navigates to main view
- **AND** error displays message and clears field

### Requirement: Document view widget
The system SHALL render a document view widget displaying Markdown content. The widget SHALL support scrolling and basic Markdown formatting.

#### Scenario: Document content rendered
- **WHEN** document is opened
- **THEN** Markdown is parsed and rendered with:
  - Headers (bold, larger)
  - Lists (bullets/numbers)
  - Code blocks (monospace)
  - Links (underlined or colored)

#### Scenario: Document scroll
- **WHEN** document exceeds visible area
- **THEN** content scrolls with `j/k` or arrow keys
- **AND** scroll position is indicated

### Requirement: Confirmation dialog widget
The system SHALL render confirmation dialogs for destructive actions (delete, quit). The dialog SHALL require explicit confirmation.

#### Scenario: Confirmation dialog displayed
- **WHEN** user initiates destructive action
- **THEN** dialog is displayed with:
  - Confirmation message
  - "Yes" and "No" options
  - Focus on "No" by default

#### Scenario: Confirmation accepted
- **WHEN** user selects "Yes" and presses `Enter`
- **THEN** action is executed
- **AND** dialog is closed

#### Scenario: Confirmation cancelled
- **WHEN** user selects "No" or presses `Esc`
- **THEN** action is cancelled
- **AND** dialog is closed

### Requirement: Help overlay widget
The system SHALL render a help overlay displaying all keyboard shortcuts. The overlay SHALL be dismissible.

#### Scenario: Help overlay displayed
- **WHEN** user presses `?`
- **THEN** overlay is displayed with:
  - All keyboard shortcuts grouped by context
  - Dismissal hint ("Press any key to close")

#### Scenario: Help overlay dismissed
- **WHEN** user presses any key
- **THEN** overlay is closed
- **AND** previous view is restored
