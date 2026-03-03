## ADDED Requirements

### Requirement: Full screen layout snapshots
The system SHALL capture snapshot tests for complete screen layouts at standard terminal sizes.

#### Scenario: 80x24 terminal (minimum size)
- **WHEN** TUI renders at 80 columns x 24 rows
- **THEN** snapshot shows complete layout without truncation or overflow

#### Scenario: 120x30 terminal (standard size)
- **WHEN** TUI renders at 120 columns x 30 rows
- **THEN** snapshot shows layout with appropriate spacing and panel distribution

#### Scenario: 160x40 terminal (large size)
- **WHEN** TUI renders at 160 columns x 40 rows
- **THEN** snapshot shows layout utilizing available screen real estate

### Requirement: Authentication screen layout
The system SHALL capture snapshot tests for the authentication screen layout.

#### Scenario: Auth screen at minimum size
- **WHEN** auth screen renders at 80x24
- **THEN** snapshot shows centered auth form with title and status bar

#### Scenario: Auth screen at standard size
- **WHEN** auth screen renders at 120x30
- **THEN** snapshot shows centered auth form with appropriate margins

### Requirement: Main application layout snapshots
The system SHALL capture snapshot tests for the main application layout with sidebar and content area.

#### Scenario: Main layout with sidebar collapsed
- **WHEN** main layout renders with sidebar hidden
- **THEN** snapshot shows full-width content area with status bar

#### Scenario: Main layout with sidebar expanded
- **WHEN** main layout renders with sidebar visible
- **THEN** snapshot shows sidebar on left, content area on right, status bar at bottom

#### Scenario: Main layout with all panels
- **WHEN** main layout renders with sidebar, task list, and task detail
- **THEN** snapshot shows three-panel layout with proper dividers

### Requirement: Screen title snapshots
The system SHALL capture snapshot tests for screen titles at top of terminal.

#### Scenario: Title for authentication screen
- **WHEN** auth screen renders
- **THEN** snapshot shows "ClickDown - Authentication" in title area

#### Scenario: Title for workspace list
- **WHEN** workspace list renders
- **THEN** snapshot shows "ClickDown - Workspaces" in title area

#### Scenario: Title for task list
- **WHEN** task list renders for a specific list
- **THEN** snapshot shows "ClickDown - Tasks: <list name>" in title area

### Requirement: Status bar snapshots
The system SHALL capture snapshot tests for the status bar at bottom of terminal.

#### Scenario: Status bar with contextual help
- **WHEN** main screen renders
- **THEN** snapshot shows status bar with keyboard shortcuts for current view

#### Scenario: Status bar with error message
- **WHEN** error state is active
- **THEN** snapshot shows error message in status bar

#### Scenario: Status bar with loading indicator
- **WHEN** data is being fetched
- **THEN** snapshot shows loading indicator in status bar
