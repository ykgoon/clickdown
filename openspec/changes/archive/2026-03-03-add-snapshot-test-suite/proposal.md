## Why

ClickDown currently lacks snapshot testing for UI components and rendered views. This makes it difficult to detect unintended visual regressions when modifying the TUI layout, widgets, or rendering logic. Adding snapshot tests with mocked network calls enables fast, deterministic regression testing without requiring actual API connectivity.

## What Changes

- Add snapshot testing infrastructure using `insta` crate for Rust
- Create snapshot tests for TUI widgets (sidebar, task list, task detail, auth view, document view)
- Create snapshot tests for screen layouts and rendering output
- Mock all network calls using existing `MockClickUpClient` pattern
- Add snapshot test documentation to TESTING.md
- Configure CI-friendly snapshot review workflow

## Capabilities

### New Capabilities
- `snapshot-testing`: Snapshot testing infrastructure using insta crate for capturing and comparing rendered UI output
- `widget-snapshots`: Snapshot tests for individual TUI widgets (sidebar, task list, task detail, auth view, document view, help dialog)
- `layout-snapshots`: Snapshot tests for full screen layouts at various terminal sizes

### Modified Capabilities
- None (this is a new testing capability, no existing spec requirements change)

## Impact

- **Dependencies**: Adds `insta` crate (dev-dependency) for snapshot testing
- **Test Suite**: New test files in `tests/snapshot_test.rs` and potentially `src/tui/snapshot_tests.rs`
- **CI/CD**: Snapshot review workflow for pull requests (snapshots stored in `tests/snapshots/`)
- **Build**: No impact on release builds (snapshot testing is dev-only)
- **Existing Tests**: No changes to existing unit or integration tests
