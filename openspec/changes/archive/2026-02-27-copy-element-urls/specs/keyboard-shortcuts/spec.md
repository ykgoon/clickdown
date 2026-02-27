## MODIFIED Requirements

### Requirement: Global navigation keys
The system SHALL implement global navigation keys that work from any screen. These keys SHALL provide quick access to common actions.

#### Scenario: Quit application
- **WHEN** user presses `q`
- **THEN** confirmation dialog is shown
- **AND** application exits on confirmation

#### Scenario: Toggle sidebar
- **WHEN** user presses `Tab` or `Ctrl+b`
- **THEN** sidebar visibility is toggled
- **AND** main content area is resized accordingly

#### Scenario: Show help
- **WHEN** user presses `?`
- **THEN** help overlay is displayed
- **AND** all keyboard shortcuts are listed

#### Scenario: Copy element URL
- **WHEN** user presses `Ctrl+Shift+C` (or `Cmd+Shift+C` on macOS)
- **THEN** the URL for the currently selected element is copied to the system clipboard
- **AND** visual feedback is displayed in the status bar
- **AND** the shortcut works from any view context (workspace list, task detail, comment thread, document view, etc.)
