## ADDED Requirements

### Requirement: Inbox navigation entry point
The system SHALL display an inbox entry point in the navigation sidebar. The inbox entry point SHALL be positioned at the top level of the sidebar, separate from the workspace hierarchy.

#### Scenario: Inbox item visible in sidebar
- **WHEN** the sidebar is displayed
- **THEN** an "Inbox" navigation item is visible at the top level

#### Scenario: Inbox item has icon
- **WHEN** the inbox navigation item is displayed
- **THEN** a mail/inbox icon precedes the "Inbox" label

#### Scenario: Inbox item is selectable
- **WHEN** user navigates to the inbox item using j/k keys
- **THEN** the inbox item can be selected and highlighted

#### Scenario: Enter opens inbox view
- **WHEN** user presses Enter on the selected inbox item
- **THEN** the inbox view is displayed in the main content area

### Requirement: Inbox navigation state
The system SHALL maintain inbox navigation state separately from workspace hierarchy navigation state.

#### Scenario: Inbox view state tracked
- **WHEN** user enters the inbox view
- **THEN** the application state reflects inbox as the current view

#### Scenario: Back navigation from inbox
- **WHEN** user presses Esc while in inbox view
- **THEN** navigation returns to the previous view (workspace list or sidebar)

#### Scenario: Sidebar shows inbox context
- **WHEN** inbox view is active
- **THEN** the inbox navigation item remains highlighted in the sidebar
