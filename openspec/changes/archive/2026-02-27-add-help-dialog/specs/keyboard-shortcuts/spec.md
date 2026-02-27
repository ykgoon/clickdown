## ADDED Requirements

### Requirement: Question mark toggles help dialog
The system SHALL allow users to toggle the help dialog by pressing the `?` key. This shortcut SHALL work globally across all application states and screens.

#### Scenario: User presses question mark to open help
- **WHEN** user presses `?` key
- **THEN** the help dialog opens showing all keyboard shortcuts

#### Scenario: User presses question mark to close help
- **WHEN** the help dialog is visible AND user presses `?` key
- **THEN** the help dialog closes

#### Scenario: Question mark works in all states
- **WHEN** user presses `?` during any application state (loading, authenticated, unauthenticated)
- **THEN** the help dialog toggles appropriately

#### Scenario: Question mark does not interfere with other input
- **WHEN** user is typing in a form field AND presses `?`
- **THEN** the `?` character is entered in the field instead of toggling help

### Requirement: Any key closes help dialog
The system SHALL close the help dialog when any key is pressed while the dialog is visible.

#### Scenario: Pressing navigation key closes help
- **WHEN** the help dialog is visible AND user presses `j`, `k`, `Enter`, or `Esc`
- **THEN** the help dialog closes without processing the key

#### Scenario: Pressing action key closes help
- **WHEN** the help dialog is visible AND user presses `n`, `e`, or `d`
- **THEN** the help dialog closes without processing the key

#### Scenario: Pressing global key closes help
- **WHEN** the help dialog is visible AND user presses `q`, `Tab`, or `u`
- **THEN** the help dialog closes without processing the key

#### Scenario: Pressing question mark closes help
- **WHEN** the help dialog is visible AND user presses `?`
- **THEN** the help dialog closes (toggles off)
