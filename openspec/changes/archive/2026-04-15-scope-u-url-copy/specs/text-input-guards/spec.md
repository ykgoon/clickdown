## ADDED Requirements

### Requirement: Centralized text input state check
The system SHALL provide a single method `is_text_input_active()` that returns true when any text input field or modal overlay is active. This method SHALL be called before processing global single-key shortcuts to prevent them from intercepting typed characters.

#### Scenario: Text input active during comment editing
- **WHEN** `comment_editing_index` is `Some` OR `comment_new_text` is non-empty
- **THEN** `is_text_input_active()` returns `true`

#### Scenario: Text input active during task creation
- **WHEN** `task_creating` is `true`
- **THEN** `is_text_input_active()` returns `true`

#### Scenario: Text input active during URL input dialog
- **WHEN** `url_input_open` is `true`
- **THEN** `is_text_input_active()` returns `true`

#### Scenario: Text input active during status picker
- **WHEN** `status_picker_open` is `true`
- **THEN** `is_text_input_active()` returns `true`

#### Scenario: Text input active during assignee picker
- **WHEN** `assignee_picker_open` is `true`
- **THEN** `is_text_input_active()` returns `true`

#### Scenario: No text input active
- **WHEN** none of the above conditions are true
- **THEN** `is_text_input_active()` returns `false`

### Requirement: Text input receives keys before global shortcuts
The system SHALL route keystrokes to the active text input handler before evaluating global single-key shortcuts. When `is_text_input_active()` returns `true`, the key SHALL be processed by the appropriate text input handler and SHALL NOT trigger global shortcuts.

#### Scenario: Key routed to active text input
- **WHEN** `is_text_input_active()` returns `true`
- **AND** a key is pressed
- **THEN** the key is routed to the active text input handler
- **AND** global shortcuts are NOT evaluated

#### Scenario: Key routed to global shortcuts when not typing
- **WHEN** `is_text_input_active()` returns `false`
- **AND** a key is pressed
- **THEN** global shortcuts are evaluated normally
