## ADDED Requirements

### Requirement: Screen title display
The system SHALL display a unique, descriptive title at the top of every screen. The title SHALL follow the format "ClickDown - {Context}" and be visually distinct.

#### Scenario: Title on authentication screen
- **WHEN** authentication screen is displayed
- **THEN** title reads "ClickDown - Authentication"
- **AND** title is displayed at top of screen
- **AND** title uses distinct styling (bold or different color)

#### Scenario: Title on workspace screen
- **WHEN** workspace list is displayed
- **THEN** title reads "ClickDown - Workspaces"

#### Scenario: Title on space screen
- **WHEN** space view is displayed
- **THEN** title reads "ClickDown - {space_name}"
- **AND** space name is truncated if too long

#### Scenario: Title on folder screen
- **WHEN** folder view is displayed
- **THEN** title reads "ClickDown - {folder_name}"

#### Scenario: Title on list screen
- **WHEN** list view is displayed
- **THEN** title reads "ClickDown - {list_name}"

#### Scenario: Title on task list
- **WHEN** task list is displayed
- **THEN** title reads "ClickDown - Tasks: {list_name}"

#### Scenario: Title on task detail
- **WHEN** task detail view is displayed
- **THEN** title reads "ClickDown - Task: {task_name}"
- **AND** task name is truncated with ellipsis if too long

#### Scenario: Title on document view
- **WHEN** document view is displayed
- **THEN** title reads "ClickDown - Doc: {doc_title}"

### Requirement: Title uniqueness
Each screen title SHALL be unique to enable debugging and accurate references. No two screens SHALL have the same title at the same time.

#### Scenario: Unique titles across navigation
- **WHEN** user navigates through different screens
- **THEN** each screen displays a different title
- **AND** title accurately reflects current context

### Requirement: Layout structure
The system SHALL implement a consistent layout structure across all screens. The layout SHALL include title bar, content area, and status bar.

#### Scenario: Standard layout rendered
- **WHEN** any screen is displayed
- **THEN** layout includes:
  - Title bar at top (1 row)
  - Content area (flexible height)
  - Status bar at bottom (1-3 rows)

#### Scenario: Sidebar layout
- **WHEN** sidebar is visible
- **THEN** layout includes:
  - Sidebar on left (20-25% width)
  - Main content on right (75-80% width)
  - Both share same title bar and status bar

#### Scenario: Full-width layout
- **WHEN** sidebar is hidden
- **THEN** main content uses full terminal width
- **AND** content is re-centered

### Requirement: Responsive layout
The layout SHALL adapt to terminal size changes. Content SHALL be reflowed or scrolled to fit available space.

#### Scenario: Terminal resized larger
- **WHEN** terminal is resized to larger dimensions
- **THEN** content area expands to use available space
- **AND** lists show more items

#### Scenario: Terminal resized smaller
- **WHEN** terminal is resized to smaller dimensions
- **THEN** content area contracts
- **AND** scrolling is enabled if content exceeds visible area
- **AND** minimum usable area is maintained

#### Scenario: Minimum terminal size
- **WHEN** terminal is smaller than 80x24
- **THEN** warning message is displayed
- **AND** content is still rendered but may require scrolling

### Requirement: Status bar
The system SHALL display a status bar at the bottom of every screen. The status bar SHALL show application state, errors, and contextual help.

#### Scenario: Status bar shows loading
- **WHEN** application is loading data
- **THEN** status bar displays "Loading..." or spinner

#### Scenario: Status bar shows errors
- **WHEN** an error occurs
- **THEN** status bar displays error message in red
- **AND** error is visible for at least 5 seconds

#### Scenario: Status bar shows help hints
- **WHEN** user is on a screen with available actions
- **THEN** status bar displays contextual help (e.g., "Press n to create, e to edit, d to delete")

#### Scenario: Status bar shows navigation path
- **WHEN** user is in nested navigation
- **THEN** status bar shows current path (e.g., "Workspace > Space > Folder > List")
