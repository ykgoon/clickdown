## ADDED Requirements

### Requirement: U key does not interfere with typing
The system SHALL NOT trigger the URL copy action when the `u` key is pressed during active text input. The letter `u` SHALL be entered into the active text field instead of copying a URL.

#### Scenario: U types into comment text
- **WHEN** user is actively editing a comment (`comment_editing_index` is set or `comment_new_text` is non-empty)
- **AND** user presses `u`
- **THEN** the letter `u` is appended to the comment text
- **AND** no URL is copied to clipboard

#### Scenario: U types into task name during creation
- **WHEN** user is creating a new task (`task_creating` is true)
- **AND** the task creation focus is on the name field
- **AND** user presses `u`
- **THEN** the letter `u` is appended to the task name
- **AND** no URL is copied to clipboard

#### Scenario: U types into task description during creation
- **WHEN** user is creating a new task (`task_creating` is true)
- **AND** the task creation focus is on the description field
- **AND** user presses `u`
- **THEN** the letter `u` is appended to the task description
- **AND** no URL is copied to clipboard

#### Scenario: U types into URL input dialog
- **WHEN** the URL input dialog is open (`url_input_open` is true)
- **AND** user presses `u`
- **THEN** the letter `u` is entered into the URL input field
- **AND** no URL is copied to clipboard

#### Scenario: U copies URL when not typing
- **WHEN** no text input is active
- **AND** user presses `u`
- **THEN** the URL for the current context is copied to clipboard
- **AND** a confirmation message is shown in the status bar
