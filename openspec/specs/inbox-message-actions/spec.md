## ADDED Requirements

### Requirement: Clear notification action
The system SHALL allow users to clear individual notifications using a keyboard shortcut. The clear action SHALL mark the notification as read and remove it from the unread list.

#### Scenario: Clear selected notification
- **WHEN** user presses c on a selected notification
- **THEN** the notification is marked as read and removed from the list

#### Scenario: Selection moves after clear
- **WHEN** a notification is cleared
- **THEN** selection moves to the next notification (or previous if at end)

#### Scenario: Clear last notification
- **WHEN** user clears the last remaining notification
- **THEN** empty inbox state is displayed

### Requirement: Clear all notifications action
The system SHALL allow users to clear all unread notifications at once using a keyboard shortcut.

#### Scenario: Clear all notifications
- **WHEN** user presses C (shift+c)
- **THEN** all unread notifications are marked as read

#### Scenario: Clear all confirmation not required
- **WHEN** user presses C
- **THEN** all notifications are cleared immediately without confirmation

#### Scenario: Clear all updates UI
- **WHEN** all notifications are cleared
- **THEN** empty inbox state is displayed

### Requirement: Keyboard shortcut help
The system SHALL display inbox-specific keyboard shortcuts in the help dialog and status bar.

#### Scenario: Help dialog shows inbox shortcuts
- **WHEN** user presses ? in inbox view
- **THEN** help dialog shows inbox-specific shortcuts (c, C, etc.)

#### Scenario: Status bar shows context help
- **WHEN** inbox view is active
- **THEN** status bar shows relevant shortcuts (e.g., "c: clear, C: clear all")

### Requirement: Notification detail view
The system SHALL allow users to view full notification details in a detail panel or overlay.

#### Scenario: Open notification detail
- **WHEN** user presses Enter on a selected notification
- **THEN** full notification details are displayed

#### Scenario: Detail shows full description
- **WHEN** detail view is displayed
- **THEN** the complete notification description is shown (not truncated)

#### Scenario: Detail shows metadata
- **WHEN** detail view is displayed
- **THEN** creation time, source workspace, and related context are shown

#### Scenario: Close detail view
- **WHEN** user presses Esc in detail view
- **THEN** detail view closes and returns to notification list
