## ADDED Requirements

### Requirement: Snapshot testing infrastructure
The system SHALL provide snapshot testing capabilities using the `insta` crate for capturing and comparing rendered UI output.

#### Scenario: Install insta dependency
- **WHEN** developer adds `insta` to `Cargo.toml` as a dev-dependency
- **THEN** snapshot testing is available for all test modules

#### Scenario: Configure test backend
- **WHEN** tests run using `ratatui::backend::TestBackend`
- **THEN** UI renders to off-screen buffer without terminal display

#### Scenario: Snapshot storage
- **WHEN** snapshot tests execute
- **THEN** snapshots are stored in `tests/snapshots/` directory organized by test category

#### Scenario: Deterministic output
- **WHEN** tests run on different platforms (Linux, macOS, Windows)
- **THEN** snapshot content is identical across platforms

### Requirement: Mock all external dependencies
The system SHALL mock all external dependencies in snapshot tests to ensure isolation and determinism.

#### Scenario: Mock API client
- **WHEN** snapshot test requires ClickUp API data
- **THEN** `MockClickUpClient` provides predefined responses without network calls

#### Scenario: Mock clipboard operations
- **WHEN** snapshot test triggers clipboard action
- **THEN** clipboard returns success without actual system clipboard access

#### Scenario: Mock file system operations
- **WHEN** snapshot test triggers file I/O
- **THEN** file operations use temporary directories or mocked responses

### Requirement: CI/CD integration
The system SHALL support snapshot review in continuous integration workflows.

#### Scenario: Run snapshot tests in CI
- **WHEN** CI pipeline executes `cargo test`
- **THEN** snapshot tests run and fail if snapshots don't match

#### Scenario: Review snapshots locally
- **WHEN** developer runs `cargo insta review`
- **THEN** interactive review shows differences between old and new snapshots

#### Scenario: Accept snapshot changes
- **WHEN** developer accepts snapshot changes via `cargo insta accept`
- **THEN** new snapshots replace old snapshots in version control
