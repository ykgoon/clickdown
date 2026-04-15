## MODIFIED Requirements

### Requirement: Keyboard shortcut for copying URL
The system SHALL provide keyboard shortcuts to trigger the copy URL action. The primary shortcuts SHALL be `Ctrl+Shift+C` on Linux/Windows and `Cmd+Shift+C` on macOS. Additionally, the single key `u` SHALL copy the URL when no text input is active. The `u` shortcut SHALL NOT fire when any text input field or modal overlay is active.

#### Scenario: User presses Ctrl+Shift+C on Linux/Windows
- **WHEN** the user presses `Ctrl`, `Shift`, and `C` simultaneously
- **THEN** the URL for the current context is copied to clipboard

#### Scenario: User presses Cmd+Shift+C on macOS
- **WHEN** the user presses `Cmd`, `Shift`, and `C` simultaneously on macOS
- **THEN** the URL for the current context is copied to clipboard

#### Scenario: Copy URL with lowercase 'c'
- **WHEN** the user presses `Ctrl+Shift+c` (lowercase)
- **THEN** the URL is copied (case-insensitive for the letter key)

#### Scenario: Copy URL does not interfere with typing 'C'
- **WHEN** the user presses `C` without modifiers
- **THEN** no copy action is triggered
- **AND** normal input continues

#### Scenario: Copy URL does not interfere with Ctrl+C
- **WHEN** the user presses `Ctrl+C` without Shift
- **THEN** no copy URL action is triggered
- **AND** Ctrl+C retains its existing behavior (if any)

#### Scenario: Single key u copies URL when not typing
- **WHEN** no text input is active (`is_text_input_active()` returns `false`)
- **AND** user presses `u`
- **THEN** the URL for the current context is copied to clipboard

#### Scenario: Single key u types when text input is active
- **WHEN** any text input is active (`is_text_input_active()` returns `true`)
- **AND** user presses `u`
- **THEN** the letter `u` is entered into the active text field
- **AND** no URL is copied to clipboard
