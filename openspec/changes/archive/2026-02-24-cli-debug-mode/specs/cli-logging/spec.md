# cli-logging Specification

## Purpose
Define logging configuration and output behavior for CLI debug mode operations.

## ADDED Requirements

### Requirement: Logging Configuration for CLI

The system SHALL configure logging appropriately for CLI debug mode execution.

#### Scenario: Default logging level
- **WHEN** user runs debug operation without `--verbose` flag
- **THEN** the system SHALL use the existing logging configuration from TUI mode
- **AND** the system SHALL NOT output debug logs to stderr

#### Scenario: Verbose logging enabled
- **WHEN** user runs debug operation with `--verbose` flag
- **THEN** the system SHALL set minimum log level to DEBUG
- **AND** the system SHALL output logs to stderr in human-readable format

#### Scenario: Trace logging for deep debugging
- **WHEN** user sets RUST_LOG=trace environment variable
- **THEN** the system SHALL output trace-level logs including HTTP request/response bodies
- **AND** the system SHALL exclude sensitive data (tokens, credentials) from logs

### Requirement: Log Output Format

The system SHALL format log output appropriately for CLI consumption.

#### Scenario: Standard log format
- **WHEN** verbose mode is enabled
- **THEN** logs SHALL include timestamp, level, and module path
- **AND** the format SHALL be: `[timestamp] [LEVEL] module: message`

#### Scenario: Error messages to stderr
- **WHEN** an error occurs during debug operation
- **THEN** the error message SHALL be printed to stderr
- **AND** the error SHALL also be logged at ERROR level

#### Scenario: Data output to stdout
- **WHEN** debug operation produces data output (workspaces, tasks, docs)
- **THEN** the data SHALL be printed to stdout
- **AND** logs SHALL remain separate on stderr

### Requirement: Sensitive Data Redaction

The system SHALL prevent sensitive data from appearing in logs.

#### Scenario: Token redaction in logs
- **WHEN** logging HTTP headers or auth-related operations
- **THEN** the system SHALL redact token values as `[REDACTED]`
- **AND** the token SHALL NOT appear in any log output even in trace mode

#### Scenario: Credential redaction
- **WHEN** user provides `--token` override flag
- **THEN** the system SHALL NOT log the token value
- **AND** the system SHALL log "Using override token" without the value

### Requirement: Logging Integration with Existing Infrastructure

The system SHALL integrate with ClickDown's existing tracing infrastructure.

#### Scenario: Reuse existing tracing subscriber
- **WHEN** CLI debug mode initializes
- **THEN** the system SHALL use the same tracing-subscriber configuration as TUI mode
- **AND** the system SHALL adjust log level based on `--verbose` flag

#### Scenario: Environment variable precedence
- **WHEN** both RUST_LOG environment variable and `--verbose` flag are present
- **THEN** the explicit RUST_LOG value SHALL take precedence
- **AND** the `--verbose` flag SHALL only set default if RUST_LOG is not set
