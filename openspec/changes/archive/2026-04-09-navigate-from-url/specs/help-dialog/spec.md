## MODIFIED Requirements

### Requirement: Help dialog content organization
The system SHALL display keyboard shortcuts organized by category with clear labels and formatting. The help dialog SHALL include a section for navigation shortcuts that includes the `g-u` URL navigation chord.

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

#### Scenario: Close hint is displayed
- **WHEN** the help dialog is open
- **THEN** it displays "Press any key to close" at the bottom
