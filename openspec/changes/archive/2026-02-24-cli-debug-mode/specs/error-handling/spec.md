# error-handling Specification (Delta)

## Purpose
Add CLI-specific error handling patterns and exit codes for debug mode operations.

## ADDED Requirements

### Requirement: CLI Exit Code Mapping

The system SHALL map application errors to standard Unix exit codes for CLI operations.

#### Scenario: Map API errors to exit codes
- **WHEN** a ClickUp API error occurs during debug operation
- **THEN** authentication errors SHALL map to exit code 3
- **AND** network errors SHALL map to exit code 4
- **AND** not-found errors SHALL map to exit code 1

#### Scenario: Map argument parsing errors
- **WHEN** CLI argument parsing fails
- **THEN** the error SHALL map to exit code 2
- **AND** usage information SHALL be printed to stderr

#### Scenario: Map internal errors
- **WHEN** an unexpected internal error occurs (e.g., cache corruption)
- **THEN** the error SHALL map to exit code 1
- **AND** a descriptive message SHALL be printed to stderr

### Requirement: CLI Error Message Formatting

The system SHALL format error messages appropriately for CLI output.

#### Scenario: Single-line error messages
- **WHEN** an error is reported in CLI mode
- **THEN** the primary error message SHALL be a single line
- **AND** the message SHALL be printed to stderr
- **AND** the message SHALL NOT include stack traces or debug info unless verbose mode is enabled

#### Scenario: Verbose error details
- **WHEN** an error occurs with `--verbose` flag enabled
- **THEN** the system SHALL include additional context (e.g., request ID, endpoint URL)
- **AND** the system SHALL include the error chain (all `caused by:` messages)

#### Scenario: User-friendly API errors
- **WHEN** ClickUp API returns an error response
- **THEN** the system SHALL extract the user-friendly message from the API response
- **AND** the system SHALL print "ClickUp API error: <message>" to stderr
