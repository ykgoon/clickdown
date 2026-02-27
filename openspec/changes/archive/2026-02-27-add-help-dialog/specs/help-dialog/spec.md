## ADDED Requirements

### Requirement: Help dialog toggle with question mark
The system SHALL allow users to toggle the help dialog by pressing the `?` key. The help dialog SHALL display as a modal overlay showing all available keyboard shortcuts.

#### Scenario: User presses question mark to open help
- **WHEN** user presses `?` key
- **THEN** the help dialog appears as a modal overlay displaying keyboard shortcuts

#### Scenario: User presses any key to close help
- **WHEN** the help dialog is visible AND user presses any key
- **THEN** the help dialog closes and returns to the previous screen

#### Scenario: Help dialog blocks underlying interactions
- **WHEN** the help dialog is visible AND user presses a shortcut key (e.g., `j`, `k`, `n`, `e`)
- **THEN** the shortcut is NOT processed by the underlying UI

#### Scenario: Help dialog works from any screen
- **WHEN** user is on any screen (Auth, Workspaces, Tasks, Task Detail, Document)
- **THEN** pressing `?` opens the help dialog

### Requirement: Help shortcut hint in status bar
The system SHALL display `?` in the status bar as a global hint across all application screens.

#### Scenario: Status bar shows help hint on auth screen
- **WHEN** user is on the auth screen
- **THEN** the status bar includes `?` in the hints

#### Scenario: Status bar shows help hint on navigation screens
- **WHEN** user is on Workspaces, Spaces, Folders, or Lists screen
- **THEN** the status bar includes `?` in the hints

#### Scenario: Status bar shows help hint on tasks screen
- **WHEN** user is on the Tasks screen
- **THEN** the status bar includes `?` in the hints

#### Scenario: Status bar shows help hint on task detail screen
- **WHEN** user is on the Task Detail screen
- **THEN** the status bar includes `?` in the hints

#### Scenario: Status bar shows help hint on document screen
- **WHEN** user is on the Document screen
- **THEN** the status bar includes `?` in the hints

### Requirement: Help dialog content organization
The help dialog SHALL display keyboard shortcuts organized by category with clear labels and formatting.

#### Scenario: Navigation shortcuts are displayed
- **WHEN** the help dialog is open
- **THEN** it shows a "Navigation" section with j/k, Enter, and Esc shortcuts

#### Scenario: Global shortcuts are displayed
- **WHEN** the help dialog is open
- **THEN** it shows a "Global" section with q, Tab, ?, and u shortcuts

#### Scenario: Action shortcuts are displayed
- **WHEN** the help dialog is open
- **THEN** it shows an "Actions" section with n, e, and d shortcuts

#### Scenario: Comment shortcuts are displayed
- **WHEN** the help dialog is open
- **THEN** it shows a "Comments" section with comment-related shortcuts

#### Scenario: Form shortcuts are displayed
- **WHEN** the help dialog is open
- **THEN** it shows a "Forms" section with Ctrl+S and Esc shortcuts

#### Scenario: Close hint is displayed
- **WHEN** the help dialog is open
- **THEN** it displays "Press any key to close" at the bottom
