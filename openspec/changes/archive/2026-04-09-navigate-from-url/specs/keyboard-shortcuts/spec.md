## ADDED Requirements

### Requirement: g-u chord opens URL navigation dialog
The system SHALL allow users to open the URL navigation dialog by pressing the `g` key followed by the `u` key. This two-key chord SHALL follow vim-style conventions where `g` acts as a leader key for navigation commands.

#### Scenario: User presses g then u to open URL navigation
- **WHEN** the user is on any authenticated screen (Workspaces, Spaces, Folders, Lists, Tasks, TaskDetail, Document)
- **AND** presses `g` followed by `u`
- **THEN** the URL navigation dialog opens

#### Scenario: g-u is inactive on auth screen
- **WHEN** the user is on the auth screen (not authenticated)
- **AND** presses `g` followed by `u`
- **THEN** no action is taken
- **AND** the status bar displays "Please authenticate first"

#### Scenario: g followed by non-u passes through second key
- **WHEN** the user presses `g`
- **AND** then presses a key other than `u` (e.g., `j`, `k`, `Esc`)
- **THEN** the second key is processed as normal input for the current screen
- **AND** the `g` prefix is consumed without triggering navigation

#### Scenario: Esc cancels g leader key
- **WHEN** the user presses `g`
- **AND** then presses `Esc`
- **THEN** the leader state is cleared
- **AND** no navigation action occurs
