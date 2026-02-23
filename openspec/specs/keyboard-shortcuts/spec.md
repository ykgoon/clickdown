## Purpose

The keyboard-shortcuts capability defines global keyboard shortcuts that provide quick access to common application actions. These shortcuts work across all application states and follow desktop application conventions.

## Requirements

### Requirement: Ctrl-Q quits the application
The system SHALL allow users to quit the application by pressing `ctrl-q` keyboard shortcut. The shortcut MUST work reliably across different keyboard layouts and operating systems.

#### Scenario: User presses ctrl-q
- **WHEN** user presses `ctrl` and `q` keys simultaneously
- **THEN** the application exits immediately without confirmation dialog

#### Scenario: Ctrl-Q works with non-US keyboard layouts
- **WHEN** user presses `ctrl` and `q` keys on a non-US keyboard layout
- **THEN** the application exits regardless of keyboard layout mapping

#### Scenario: Ctrl-Q works with both character and named key events
- **WHEN** the system generates either `Key::Character("q")` or `Key::Named(Named::KeyQ)` event
- **THEN** both event types trigger application exit when ctrl modifier is pressed

#### Scenario: Ctrl-Q does not interfere with normal typing
- **WHEN** user types the letter 'q' without holding ctrl
- **THEN** the application continues running normally

#### Scenario: Ctrl-Q works in all application states
- **WHEN** user presses `ctrl-q` during any application state (loading, authenticated, unauthenticated)
- **THEN** the application exits in all states
