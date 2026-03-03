## ADDED Requirements

### Requirement: Sidebar widget snapshots
The system SHALL capture snapshot tests for the sidebar navigation widget in various states.

#### Scenario: Empty sidebar
- **WHEN** sidebar renders with no workspaces loaded
- **THEN** snapshot shows empty state with loading indicator or prompt

#### Scenario: Sidebar with workspaces
- **WHEN** sidebar renders with multiple workspaces
- **THEN** snapshot shows workspace list with proper indentation and icons

#### Scenario: Sidebar with selection
- **WHEN** sidebar renders with an item selected
- **THEN** snapshot shows selected item with highlight styling

#### Scenario: Sidebar with nested hierarchy
- **WHEN** sidebar renders workspaces, spaces, folders, and lists
- **THEN** snapshot shows proper indentation levels for each hierarchy depth

### Requirement: Task list widget snapshots
The system SHALL capture snapshot tests for the task list widget in various states.

#### Scenario: Empty task list
- **WHEN** task list renders with no tasks
- **THEN** snapshot shows empty state message

#### Scenario: Task list with multiple tasks
- **WHEN** task list renders with tasks of different statuses and priorities
- **THEN** snapshot shows task list with status indicators and priority colors

#### Scenario: Task list with selection
- **WHEN** task list renders with a task selected
- **THEN** snapshot shows selected task with highlight styling

#### Scenario: Task list sorted by status
- **WHEN** task list renders with tasks sorted by status group priority
- **THEN** snapshot shows tasks ordered: in progress → todo → done

### Requirement: Task detail widget snapshots
The system SHALL capture snapshot tests for the task detail/create/edit panel.

#### Scenario: Task detail view mode
- **WHEN** task detail renders in view mode
- **THEN** snapshot shows task name, description, status, assignees, and metadata

#### Scenario: Task create mode
- **WHEN** task detail renders in create mode with empty form
- **THEN** snapshot shows form fields with placeholder text and cursor position

#### Scenario: Task edit mode
- **WHEN** task detail renders in edit mode with populated form
- **THEN** snapshot shows form fields with task data and cursor position

### Requirement: Auth view widget snapshots
The system SHALL capture snapshot tests for the authentication view.

#### Scenario: Auth view with empty token
- **WHEN** auth view renders with no token entered
- **THEN** snapshot shows empty input field with cursor

#### Scenario: Auth view with partial token
- **WHEN** auth view renders with partially entered token
- **THEN** snapshot shows first 4 characters visible, rest masked with bullets

#### Scenario: Auth view with error state
- **WHEN** auth view renders after authentication failure
- **THEN** snapshot shows error message below input field

### Requirement: Document view widget snapshots
The system SHALL capture snapshot tests for the document/Markdown viewer.

#### Scenario: Document with markdown content
- **WHEN** document view renders with markdown content
- **THEN** snapshot shows rendered markdown with proper formatting

#### Scenario: Document with nested pages
- **WHEN** document view renders with page hierarchy
- **THEN** snapshot shows page tree with proper indentation

#### Scenario: Document empty state
- **WHEN** document view renders with no document selected
- **THEN** snapshot shows empty state prompt

### Requirement: Help dialog snapshots
The system SHALL capture snapshot tests for the help dialog overlay.

#### Scenario: Help dialog visible
- **WHEN** help dialog renders with keyboard shortcuts
- **THEN** snapshot shows all keyboard shortcuts in organized sections

#### Scenario: Help dialog with quit confirmation
- **WHEN** quit confirmation dialog renders
- **THEN** snapshot shows confirmation prompt with Yes/No selection
