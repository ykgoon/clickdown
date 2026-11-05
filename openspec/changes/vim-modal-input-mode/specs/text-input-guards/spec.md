## ADDED Requirements

### Requirement: Modal input modes
The TUI SHALL behave as a two-mode editor: while the user types free text (description, comment, reply, task form) the app is in INPUT mode where keys are captured into the buffer; otherwise it is in NORMAL mode where `SPACE` opens the transient command menu and keys are interpreted as commands.

#### Scenario: Typing is never swallowed as commands
- **GIVEN** the user is in INPUT mode editing a comment
- **WHEN** the user presses letters, digits, punctuation, `Enter`, or `Backspace`
- **THEN** every key edits the buffer and no normal-mode command fires

#### Scenario: SPACE in NORMAL mode opens menu
- **GIVEN** no text input is active
- **WHEN** the user presses `SPACE`
- **THEN** the transient command menu opens

#### Scenario: Exit INPUT mode
- **WHEN** the user presses `Esc` in INPUT mode
- **THEN** the buffer is discarded and NORMAL mode resumes
- **AND** `Ctrl+S` commits the buffer and returns to NORMAL mode
