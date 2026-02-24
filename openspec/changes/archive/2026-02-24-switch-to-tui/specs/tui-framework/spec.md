## ADDED Requirements

### Requirement: Terminal initialization
The system SHALL initialize the Ratatui terminal backend on application startup. The terminal SHALL be configured with crossterm backend for cross-platform compatibility.

#### Scenario: Terminal backend initialized on startup
- **WHEN** the application starts
- **THEN** crossterm backend is initialized
- **AND** terminal enters raw mode for direct input capture
- **AND** terminal is ready for rendering

#### Scenario: Terminal cleanup on shutdown
- **WHEN** the application exits
- **THEN** terminal raw mode is disabled
- **AND** terminal state is restored to normal
- **AND** no orphaned terminal state remains

### Requirement: Rendering loop
The system SHALL implement a continuous rendering loop that processes events and redraws the terminal buffer. The loop SHALL run at a sufficient frame rate for responsive UI (target: 30 FPS).

#### Scenario: Event-driven rendering
- **WHEN** a user input event occurs
- **THEN** the event is processed
- **AND** the terminal buffer is redrawn
- **AND** changes are flushed to the terminal

#### Scenario: Periodic refresh
- **WHEN** no user events occur
- **THEN** the rendering loop continues running
- **AND** the display remains stable

### Requirement: Event subscription
The system SHALL subscribe to crossterm events including keyboard input, terminal resize, and optional mouse events. Events SHALL be mapped to the application's Message enum.

#### Scenario: Keyboard event captured
- **WHEN** user presses a key
- **THEN** crossterm captures the key event
- **AND** event is converted to `Message::KeyPressed(KeyEvent)`

#### Scenario: Terminal resize captured
- **WHEN** terminal window is resized
- **THEN** crossterm captures the resize event
- **AND** event is converted to `Message::TerminalResized(width, height)`

### Requirement: Color palette
The system SHALL use a 256-color palette for terminal rendering. Colors SHALL be defined as ANSI color codes or RGB values compatible with Ratatui's Color enum.

#### Scenario: Dark theme colors applied
- **WHEN** the application renders with dark theme
- **THEN** background uses dark ANSI color (e.g., Black or DarkGrey)
- **AND** text uses light ANSI color (e.g., White or LightGrey)
- **AND** accents use colored ANSI codes (Blue, Green, Yellow, Red)

#### Scenario: Graceful color degradation
- **WHEN** terminal supports fewer than 256 colors
- **THEN** colors degrade to nearest available ANSI color
- **AND** content remains readable

### Requirement: Main loop integration
The system SHALL integrate the Ratatui rendering loop with the existing Elm architecture. The loop SHALL process Messages, update state, and render views.

#### Scenario: Message processed in loop
- **WHEN** a Message is received
- **THEN** the update function processes the message
- **AND** state is updated
- **AND** view is re-rendered with new state

#### Scenario: Application runs until quit
- **WHEN** application starts
- **THEN** main loop continues running
- **AND** loop exits only on quit message or error
