# help-dialog Specification

## Purpose
TBD - created by archiving change add-help-dialog. Update Purpose after archive.
## Requirements
### Requirement: Help dialog toggle with question mark
The system SHALL allow users to toggle the help dialog by pressing the `?` key. The help dialog SHALL display as a modal overlay showing all available keyboard shortcuts.

#### Scenario: User presses question mark to open help
- **WHEN** user presses `?` key
- **THEN** the help dialog appears as a modal overlay displaying keyboard shortcuts

#### Scenario: User presses question mark to close help
- **WHEN** the help dialog is visible AND user presses `?` key
- **THEN** the help dialog closes (toggles off)

#### Scenario: Help dialog blocks underlying interactions
- **WHEN** the help dialog is visible AND user presses a shortcut key (e.g., `j`, `k`, `n`, `e`)
- **THEN** the shortcut is NOT processed by the underlying UI

#### Scenario: Help dialog works from any screen
- **WHEN** user is on any screen (Auth, Workspaces, Tasks, Task Detail, Document)
- **THEN** pressing `?` opens the help dialog. Page 1 content SHALL vary based on the current screen as follows: Auth screen → Auth shortcuts; Workspace/Space/Folder → Navigation; Task List → Task List shortcuts; Task Detail (description focus) → Task Detail shortcuts; Task Detail (comments focus) → Comments shortcuts; Document → Document shortcuts.

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
The system SHALL display keyboard shortcuts organized by category across 3 paginated pages. Page 1 SHALL show contextually relevant shortcuts based on the current screen. Page 2 SHALL show Navigation, Global, Actions, and Forms shortcuts. Page 3 SHALL show remaining shortcut sections. The dialog SHALL display a page indicator in the title and a pagination footer with navigation hints. The dialog SHALL NOT include a "Press any key to close" hint — closing is done via `Esc` or `?`.

#### Scenario: Contextual shortcuts shown on page 1
- **WHEN** the help dialog is opened from the task list screen
- **THEN** page 1 shows Task List shortcuts with the title `Keyboard Shortcuts — Task List  (1/3)`

#### Scenario: Global shortcuts shown on page 2
- **WHEN** the help dialog is visible AND user navigates to page 2
- **THEN** page 2 shows Navigation, Global, Actions, and Forms shortcuts with the title `Keyboard Shortcuts — Global  (2/3)`

#### Scenario: Pagination footer displayed
- **WHEN** the help dialog is open
- **THEN** a footer displays `◄ ►  N/3  │  j/k: Pages  │  Esc: Close`

#### Scenario: Navigation shortcuts are displayed
- **WHEN** the help dialog is open
- **THEN** it shows a "Navigation" section with j/k, Enter, and Esc shortcuts
- **AND** it shows a "Go to" subsection with `g` `u` for URL navigation

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

### Requirement: Help dialog displays 3 paginated pages
The system SHALL display keyboard shortcuts across 3 pages within the help dialog. Page 1 SHALL show contextually relevant shortcuts based on the current screen and focus state. Page 2 SHALL always show Navigation, Global, Actions, and Forms shortcuts. Page 3 SHALL show all remaining shortcut sections not displayed on page 1.

#### Scenario: Help dialog opens on page 1
- **WHEN** user opens the help dialog
- **THEN** the dialog displays page 1 with shortcuts relevant to the current screen

#### Scenario: Page 2 shows global shortcuts
- **WHEN** user navigates to page 2
- **THEN** the dialog shows Navigation, Global, Actions, and Forms shortcuts

#### Scenario: Page 3 shows remaining shortcuts
- **WHEN** user navigates to page 3
- **THEN** the dialog shows shortcut sections not displayed on page 1 or page 2

### Requirement: Page 1 content is contextual
The system SHALL determine page 1 content based on the current application screen and focus state. The mapping SHALL be: Auth screen → Auth shortcuts; Workspace/Space/Folder → Navigation; Task List → Task List shortcuts; Task Detail (description focus) → Task Detail shortcuts; Task Detail (comments focus) → Comments shortcuts; Document → Document shortcuts.

#### Scenario: Task list screen shows task list shortcuts on page 1
- **WHEN** user is on the task list screen AND opens help
- **THEN** page 1 shows Task List shortcuts (j/k, Enter, a, n, s, d)

#### Scenario: Task detail with description focus shows task detail shortcuts on page 1
- **WHEN** user is in task detail with description focus AND opens help
- **THEN** page 1 shows Task Detail shortcuts (s, A, e, Tab, Esc)

#### Scenario: Task detail with comments focus shows comments shortcuts on page 1
- **WHEN** user is in task detail with comments focus AND opens help
- **THEN** page 1 shows Comments shortcuts (Tab, j/k, n, e, Enter, Ctrl+S, Esc)

### Requirement: Pagination navigation via j/k and arrow keys
The system SHALL allow users to navigate between pages by pressing `j`, `k`, `↓`, or `→` (next page) and `k`, `↑`, or `←` (previous page). Navigation SHALL wrap around: next from page 3 goes to page 1, previous from page 1 goes to page 3.

#### Scenario: j/↓/→ advances to next page
- **WHEN** help dialog is visible AND user presses `j`, `↓`, or `→`
- **THEN** the dialog advances to the next page (wrapping from page 3 to page 1)

#### Scenario: k/↑/← goes to previous page
- **WHEN** help dialog is visible AND user presses `k`, `↑`, or `←`
- **THEN** the dialog goes to the previous page (wrapping from page 1 to page 3)

### Requirement: Page indicator in dialog title
The system SHALL display the current page number and total pages in the dialog title. The format SHALL be `Keyboard Shortcuts — <Section Name>  (N/3)` where `<Section Name>` is the name of the primary section shown on page 1.

#### Scenario: Title shows page number on page 1
- **WHEN** help dialog is visible on page 1 showing Task List shortcuts
- **THEN** the title reads `Keyboard Shortcuts — Task List  (1/3)`

#### Scenario: Title shows page number on page 2
- **WHEN** help dialog is visible on page 2
- **THEN** the title reads `Keyboard Shortcuts — Global  (2/3)`

#### Scenario: Title shows page number on page 3
- **WHEN** help dialog is visible on page 3
- **THEN** the title reads `Keyboard Shortcuts — Reference  (3/3)`

### Requirement: Pagination footer with navigation hints
The system SHALL display a pagination footer at the bottom of the help dialog. The footer SHALL show the current page position, navigation hints, and close hint in the format: `◄ ►  N/3  │  j/k: Pages  │  Esc: Close`.

#### Scenario: Footer shows pagination info
- **WHEN** help dialog is visible
- **THEN** the footer displays the current page position (e.g., `1/3`) and navigation hints

### Requirement: Help dialog resets to page 1 on open
The system SHALL reset the help dialog to page 1 each time it is opened.

#### Scenario: Help dialog resets page on re-open
- **WHEN** user navigates to page 3, closes help, then opens help again
- **THEN** the dialog opens on page 1

