## MODIFIED Requirements

### Requirement: No command-key hint bar
The application SHALL NOT render a command-key hint/menu bar at the bottom of the screen.

#### Scenario: Bottom of screen shows only status
- **WHEN** any screen is rendered
- **THEN** the bottom row shows only the transient status message
- **AND** no key hints (`j/k: Navigate | ...`) are rendered anywhere
