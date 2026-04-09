## MODIFIED Requirements

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

### Requirement: Help dialog works from any screen
The system SHALL display the help dialog from any screen. Page 1 content SHALL vary based on the current screen as follows: Auth → Auth shortcuts; Workspace/Space/Folder → Navigation; Task List → Task List; Task Detail (description focus) → Task Detail; Task Detail (comments focus) → Comments; Document → Document shortcuts.

#### Scenario: Help opens from task list shows task list shortcuts
- **WHEN** user is on the Tasks screen AND presses `?`
- **THEN** the help dialog opens with Task List shortcuts on page 1

#### Scenario: Help opens from task detail shows contextual shortcuts
- **WHEN** user is in task detail with comments focused AND presses `?`
- **THEN** the help dialog opens with Comments shortcuts on page 1

## REMOVED Requirements

### Requirement: Help dialog toggle with question mark (partial — close behavior)
**Reason**: The "any key closes" behavior is replaced by explicit Esc/? close to enable pagination navigation within the dialog.
**Migration**: Users close the help dialog with `Esc` or `?` instead of any key. The `?` key still toggles the dialog as before.
