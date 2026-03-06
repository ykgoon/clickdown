## ADDED Requirements

### Requirement: Assigned tasks item in navigation
The system SHALL display an "Assigned to Me" navigation item at the top of the sidebar, above the workspace hierarchy. The item SHALL have a distinct icon and show a count badge.

#### Scenario: Assigned tasks item at top of sidebar
- **WHEN** the sidebar is rendered
- **THEN** the "Assigned to Me" item appears at the top of the navigation list
- **AND** workspace hierarchy items appear below it

#### Scenario: Assigned tasks item has user icon
- **WHEN** the assigned tasks item is displayed
- **THEN** a user icon (👤) or inbox icon (📬) precedes the "Assigned to Me" label

#### Scenario: Assigned tasks count badge visible
- **WHEN** assigned tasks are loaded
- **THEN** a count badge displaying the number of assigned tasks is shown next to the item label
- **AND** the badge is styled distinctly (e.g., rounded background, contrasting color)

#### Scenario: Assigned tasks item separated from workspace
- **WHEN** both assigned tasks item and workspace items are displayed
- **THEN** a visual divider or spacing separates the assigned tasks item from the workspace hierarchy
