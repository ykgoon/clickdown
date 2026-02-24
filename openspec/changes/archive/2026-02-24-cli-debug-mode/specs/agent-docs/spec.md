# agent-docs Specification

## Purpose
Define documentation requirements in AGENTS.md for using CLI debug mode during development and debugging sessions.

## ADDED Requirements

### Requirement: AGENTS.md Debug Mode Section

The system SHALL include a dedicated section in AGENTS.md documenting CLI debug mode usage for developers.

#### Scenario: Developer reads AGENTS.md for debugging guidance
- **WHEN** a developer or AI agent needs to debug the application
- **THEN** AGENTS.md SHALL contain a "Debugging with CLI Mode" section
- **AND** the section SHALL be located after the "Building and Running" section

#### Scenario: Quick start examples provided
- **WHEN** developer reads the debug section
- **THEN** the documentation SHALL provide copy-paste ready command examples
- **AND** examples SHALL cover common debugging scenarios

### Requirement: Authentication Debugging Documentation

The documentation SHALL explain how to debug authentication issues using CLI mode.

#### Scenario: Debug authentication status
- **WHEN** developer needs to verify authentication
- **THEN** documentation SHALL show `clickdown debug auth-status` command
- **AND** documentation SHALL explain exit codes (0=authenticated, 3=not authenticated)

#### Scenario: Test with different token
- **WHEN** developer needs to test with alternate credentials
- **THEN** documentation SHALL show `--token <token>` override option
- **AND** documentation SHALL warn that override token is not saved

### Requirement: Data Fetching Debugging Documentation

The documentation SHALL explain how to debug data fetching issues using CLI mode.

#### Scenario: Debug workspace loading
- **WHEN** developer needs to verify workspace API calls
- **THEN** documentation SHALL show `clickdown debug workspaces --json` command
- **AND** documentation SHALL explain how to interpret JSON output

#### Scenario: Debug task loading
- **WHEN** developer needs to verify task API calls for a specific list
- **THEN** documentation SHALL show `clickdown debug tasks <list_id> --json` command
- **AND** documentation SHALL explain how to find list IDs from workspace hierarchy

### Requirement: Verbose Logging Documentation

The documentation SHALL explain how to use verbose logging for detailed debugging.

#### Scenario: Enable verbose output
- **WHEN** developer needs to see HTTP requests and responses
- **THEN** documentation SHALL show `--verbose` flag usage
- **AND** documentation SHALL explain that logs go to stderr, data to stdout

#### Scenario: Combine verbose with JSON output
- **WHEN** developer needs both machine-readable output and debug logs
- **THEN** documentation SHALL show combining `--json` and `--verbose` flags
- **AND** documentation SHALL explain stderr/stdout separation for piping

### Requirement: Common Debugging Workflows

The documentation SHALL provide end-to-end debugging workflows for common scenarios.

#### Scenario: Reproduce bug report
- **WHEN** reproducing a bug reported by a user
- **THEN** documentation SHALL provide step-by-step workflow:
  1. Check auth status
  2. List workspaces to verify access
  3. Fetch specific data with `--json` for inspection
  4. Use `--verbose` to see API interaction details

#### Scenario: Test before TUI launch
- **WHEN** developer wants to verify API connectivity before launching TUI
- **THEN** documentation SHALL explain running `clickdown debug auth-status` first
- **AND** documentation SHALL explain this avoids TUI initialization overhead

### Requirement: Integration with Testing Documentation

The documentation SHALL reference TESTING.md for programmatic testing patterns.

#### Scenario: Cross-reference testing guide
- **WHEN** developer reads AGENTS.md debug section
- **THEN** documentation SHALL link to TESTING.md for mock client testing
- **AND** documentation SHALL explain when to use CLI mode vs. unit tests

#### Scenario: Explain CLI mode use cases
- **WHEN** developer decides between debugging approaches
- **THEN** documentation SHALL clarify:
  - CLI mode: Real API calls, headless, quick iteration
  - Unit tests: Mock data, automated, reproducible
  - TUI mode: Full application, visual debugging
