## ADDED Requirements

### Requirement: Text input handling
The system SHALL capture and process text input for forms (authentication, task creation, editing). Input SHALL be captured character-by-character in raw terminal mode.

#### Scenario: Character input captured
- **WHEN** user types a character in a text field
- **THEN** the character is appended to the input buffer
- **AND** cursor position advances
- **AND** character is displayed (or `*` for password fields)

#### Scenario: Backspace handling
- **WHEN** user presses `Backspace` in a text field
- **THEN** the last character is removed from input buffer
- **AND** cursor position moves back one character

#### Scenario: Escape cancels input
- **WHEN** user presses `Esc` while editing a text field
- **THEN** input is cancelled
- **AND** original value is restored (if editing)
- **AND** focus returns to previous view

#### Scenario: Enter submits form
- **WHEN** user presses `Enter` in the last field of a form
- **THEN** the form is submitted
- **AND** validation is performed

### Requirement: Multi-line text editing
The system SHALL support multi-line text input for task descriptions and document content. The editor SHALL support basic navigation within the text area.

#### Scenario: Newline insertion
- **WHEN** user presses `Enter` in a multi-line field
- **THEN** a new line is inserted
- **AND** cursor moves to the new line

#### Scenario: Navigate within text area
- **WHEN** user presses arrow keys in a multi-line field
- **THEN** cursor moves within the text area
- **AND** view scrolls to keep cursor visible

#### Scenario: Text area scrolling
- **WHEN** text exceeds visible area of multi-line field
- **THEN** only visible portion is rendered
- **AND** scroll indicator shows position in content

### Requirement: Form validation feedback
The system SHALL display validation errors inline within forms. Errors SHALL be clearly visible and indicate the problematic field.

#### Scenario: Required field empty
- **WHEN** user submits a form with a required field empty
- **THEN** error message is displayed near the field
- **AND** field is highlighted (e.g., red border)
- **AND** focus moves to the invalid field

#### Scenario: Invalid format
- **WHEN** user enters invalid data format
- **THEN** error message describes the expected format
- **AND** field is highlighted

#### Scenario: Error cleared on edit
- **WHEN** user edits a field with an error
- **THEN** error message is cleared
- **AND** field highlight is removed

### Requirement: Form submission states
The system SHALL provide visual feedback during form submission (loading, success, error).

#### Scenario: Submitting state
- **WHEN** form is being submitted
- **THEN** loading indicator is displayed
- **AND** form inputs are disabled
- **AND** submit button shows "Submitting..."

#### Scenario: Success feedback
- **WHEN** form submission succeeds
- **THEN** success message is displayed briefly
- **AND** form is closed or reset

#### Scenario: Error feedback
- **WHEN** form submission fails
- **THEN** error message is displayed
- **AND** form inputs are re-enabled
- **AND** user can retry

### Requirement: Password field masking
The system SHALL mask password input (API token) with asterisks or dots. The actual characters SHALL NOT be visible on screen.

#### Scenario: Password characters masked
- **WHEN** user types in password field
- **THEN** asterisks (`*`) are displayed instead of characters
- **AND** actual characters are stored in buffer

#### Scenario: Password toggle (optional)
- **WHEN** user presses `Ctrl+v` in password field
- **THEN** visibility toggles between masked and unmasked
- **AND** current content is shown/hidden
