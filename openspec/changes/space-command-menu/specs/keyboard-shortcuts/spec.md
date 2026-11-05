## ADDED Requirements

### Requirement: SPACE-prefixed commands
In NORMAL mode, task and window commands SHALL be invoked only via `SPACE <group> [<key>]` chords; single-key command bindings SHALL NOT trigger commands.

#### Scenario: Direct command key ignored
- **WHEN** the user presses `d` in NORMAL mode on the task list
- **THEN** no delete confirmation opens

#### Scenario: SPACE chord fires command
- **WHEN** the user presses `SPACE`, then `t`, then `d` on the task list
- **THEN** the delete confirmation dialog opens

### Requirement: Transient menu shows available commands
- **WHEN** `SPACE` is pressed in NORMAL mode
- **THEN** a transient menu overlay lists command groups and closes on `Esc`
- **AND** unknown keys close the menu with a status message
