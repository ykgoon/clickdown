# auth-input Specification

## Purpose
Define the authentication input behavior for the ClickDown TUI application, including token visibility, paste support, keyboard shortcuts, and input handling.

## Requirements

### Requirement: Token input displays partial visibility
The system SHALL display the first 4 characters of the API token unmasked, with remaining characters shown as bullets (•), allowing users to verify input while maintaining security.

#### Scenario: User types token characters
- **WHEN** user types characters into the token input field
- **THEN** the first 4 characters display as plain text
- **AND** characters beyond the 4th display as bullets (•)

#### Scenario: Empty token shows placeholder
- **WHEN** token input field is empty
- **THEN** the field displays a placeholder or empty state
- **AND** no bullets are shown

#### Scenario: Token shorter than 4 characters shows all
- **WHEN** user has typed fewer than 4 characters
- **THEN** all typed characters display as plain text
- **AND** no masking is applied

#### Scenario: Cursor position visible during input
- **WHEN** user is typing in the token field
- **THEN** the cursor position is visually indicated
- **AND** user can see where new characters will be inserted

### Requirement: Ctrl+V pastes clipboard content into token field
The system SHALL allow users to paste clipboard content into the token input field using Ctrl+V or Ctrl+Shift+V keyboard shortcuts.

#### Scenario: User pastes with Ctrl+V
- **WHEN** user presses Ctrl+V while token input has focus
- **THEN** clipboard content is inserted at cursor position
- **AND** pasted text follows the partial visibility masking rule

#### Scenario: User pastes with Ctrl+Shift+V
- **WHEN** user presses Ctrl+Shift+V while token input has focus
- **THEN** clipboard content is inserted at cursor position
- **AND** pasted text follows the partial visibility masking rule

#### Scenario: Paste fails gracefully
- **WHEN** clipboard is empty or inaccessible
- **THEN** a brief error message is displayed
- **AND** the application does not crash or show quit dialog

#### Scenario: Paste does not trigger quit dialog
- **WHEN** user presses Ctrl+V or Ctrl+Shift+V
- **THEN** no quit confirmation dialog appears
- **AND** the paste operation completes normally

### Requirement: Only Ctrl+Q triggers quit confirmation
The system SHALL display the quit confirmation dialog ONLY when the user presses exactly Ctrl+Q (no additional modifiers like Shift).

#### Scenario: User presses Ctrl+Q
- **WHEN** user presses Ctrl and Q simultaneously (no other modifiers)
- **THEN** the quit confirmation dialog is displayed

#### Scenario: User presses Ctrl+Shift+V
- **WHEN** user presses Ctrl+Shift+V
- **THEN** no quit dialog appears
- **AND** paste operation is attempted

#### Scenario: User presses Ctrl+Shift+Q
- **WHEN** user presses Ctrl+Shift+Q
- **THEN** no quit dialog appears
- **AND** application continues normally

#### Scenario: User presses Q without Ctrl
- **WHEN** user presses Q key without Ctrl modifier
- **THEN** no quit dialog appears
- **AND** 'q' may be handled by current screen context

### Requirement: Token input handles backspace correctly
The system SHALL allow users to delete characters from the token input using the Backspace key.

#### Scenario: User deletes last character
- **WHEN** cursor is at end of token input
- **AND** user presses Backspace
- **THEN** the last character is removed
- **AND** display updates to reflect new token length

#### Scenario: User deletes character in middle
- **WHEN** cursor is positioned in middle of token
- **AND** user presses Backspace
- **THEN** the character before cursor is removed
- **AND** cursor position adjusts accordingly
