# cli-debug-mode Specification

## Purpose
Define CLI debug mode capabilities for headless debugging and bug reproduction in ClickDown.

## ADDED Requirements

### Requirement: CLI Subcommand Parsing

The system SHALL parse CLI subcommands to route between TUI mode and debug mode.

#### Scenario: User runs without arguments
- **WHEN** user executes `clickdown` with no arguments
- **THEN** the application SHALL launch in TUI mode (existing behavior)

#### Scenario: User runs debug subcommand
- **WHEN** user executes `clickdown debug`
- **THEN** the application SHALL enter debug mode and display available operations

#### Scenario: User runs debug with operation
- **WHEN** user executes `clickdown debug <operation>`
- **THEN** the application SHALL execute the specified debug operation

#### Scenario: User provides invalid subcommand
- **WHEN** user executes `clickdown <invalid-command>`
- **THEN** the application SHALL print usage information to stderr
- **AND** the application SHALL exit with code 2

### Requirement: Workspace Listing Operation

The system SHALL provide a debug operation to list all authorized workspaces.

#### Scenario: List workspaces successfully
- **WHEN** user executes `clickdown debug workspaces`
- **THEN** the system SHALL fetch workspaces from the API
- **AND** the system SHALL print workspace names and IDs to stdout

#### Scenario: List workspaces with JSON output
- **WHEN** user executes `clickdown debug workspaces --json`
- **THEN** the system SHALL output workspaces as a JSON array
- **AND** each workspace object SHALL include `id`, `name`, and `color` fields

#### Scenario: Workspace listing fails due to auth
- **WHEN** user executes `clickdown debug workspaces` without valid token
- **THEN** the system SHALL print authentication error to stderr
- **AND** the system SHALL exit with code 3

### Requirement: Task Listing Operation

The system SHALL provide a debug operation to fetch tasks from a specific list.

#### Scenario: List tasks from a list
- **WHEN** user executes `clickdown debug tasks <list_id>`
- **THEN** the system SHALL fetch tasks from the specified list
- **AND** the system SHALL print task names, statuses, and priorities to stdout

#### Scenario: List tasks with JSON output
- **WHEN** user executes `clickdown debug tasks <list_id> --json`
- **THEN** the system SHALL output tasks as a JSON array
- **AND** each task object SHALL include `id`, `name`, `status`, `priority`, and `assignees` fields

#### Scenario: Task listing fails due to invalid list ID
- **WHEN** user executes `clickdown debug tasks <invalid_id>`
- **THEN** the system SHALL print "List not found" error to stderr
- **AND** the system SHALL exit with code 1

### Requirement: Document Search Operation

The system SHALL provide a debug operation to search documents.

#### Scenario: Search documents
- **WHEN** user executes `clickdown debug docs <query>`
- **THEN** the system SHALL search documents matching the query
- **AND** the system SHALL print document titles and paths to stdout

#### Scenario: Search documents with JSON output
- **WHEN** user executes `clickdown debug docs <query> --json`
- **THEN** the system SHALL output documents as a JSON array
- **AND** each document object SHALL include `id`, `title`, `path`, and `created_at` fields

### Requirement: Authentication Status Check

The system SHALL provide a debug operation to check authentication status.

#### Scenario: Auth status shows authenticated
- **WHEN** user executes `clickdown debug auth-status` with valid token
- **THEN** the system SHALL print "Authenticated: yes" to stdout
- **AND** the system SHALL display the workspace count
- **AND** the system SHALL exit with code 0

#### Scenario: Auth status shows not authenticated
- **WHEN** user executes `clickdown debug auth-status` without token
- **THEN** the system SHALL print "Authenticated: no" to stdout
- **AND** the system SHALL exit with code 3

### Requirement: Verbose Logging Mode

The system SHALL provide a verbose logging option for all debug operations.

#### Scenario: Enable verbose logging
- **WHEN** user executes `clickdown debug <operation> --verbose`
- **THEN** the system SHALL set RUST_LOG environment variable to "debug"
- **AND** the system SHALL output trace-level logs to stderr

#### Scenario: Verbose mode shows HTTP requests
- **WHEN** user executes `clickdown debug workspaces --verbose`
- **THEN** the system SHALL log HTTP request URLs and methods
- **AND** the system SHALL log HTTP response status codes

### Requirement: Token Override for Testing

The system SHALL allow overriding the stored token for testing purposes.

#### Scenario: Override token via CLI flag
- **WHEN** user executes `clickdown debug <operation> --token <token_value>`
- **THEN** the system SHALL use the provided token instead of the stored one
- **AND** the system SHALL NOT save the override token to disk

#### Scenario: Token override with verbose mode
- **WHEN** user executes with `--token` and `--verbose`
- **THEN** the system SHALL log "Using override token" WITHOUT printing the token value
- **AND** the token value SHALL NOT appear in logs

### Requirement: Exit Code Standards

The system SHALL use standard Unix exit codes for CLI operations.

#### Scenario: Successful operation
- **WHEN** a debug operation completes successfully
- **THEN** the system SHALL exit with code 0

#### Scenario: General error occurs
- **WHEN** an unexpected error occurs during operation
- **THEN** the system SHALL exit with code 1

#### Scenario: Invalid arguments provided
- **WHEN** user provides invalid CLI arguments
- **THEN** the system SHALL exit with code 2

#### Scenario: Authentication fails
- **WHEN** authentication or authorization fails
- **THEN** the system SHALL exit with code 3

#### Scenario: Network error occurs
- **WHEN** HTTP request fails due to network issues
- **THEN** the system SHALL exit with code 4
