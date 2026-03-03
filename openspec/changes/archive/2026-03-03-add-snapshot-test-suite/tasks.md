## 1. Setup and Configuration

- [x] 1.1 Add `insta` crate to `Cargo.toml` as dev-dependency (version 1.39+)
- [x] 1.2 Add `ratatui` test backend dependency if not already present
- [x] 1.3 Create `tests/snapshots/` directory structure for snapshot storage
- [x] 1.4 Create `tests/snapshot_test.rs` test module file

## 2. Infrastructure Implementation

- [x] 2.1 Create test helper functions for snapshot rendering in `tests/snapshot_test.rs`
- [x] 2.2 Implement `render_to_snapshot()` utility using `ratatui::backend::TestBackend`
- [x] 2.3 Create mock clipboard helper for tests
- [x] 2.4 Create fixture data generators for deterministic test data

## 3. Widget Snapshot Tests

- [x] 3.1 Implement sidebar widget snapshot tests (empty, with items, with selection)
- [x] 3.2 Implement task list widget snapshot tests (empty, with tasks, sorted)
- [x] 3.3 Implement task detail widget snapshot tests (view, create, edit modes)
- [x] 3.4 Implement auth view widget snapshot tests (empty, partial token, error)
- [x] 3.5 Implement document view widget snapshot tests (with content, empty)
- [x] 3.6 Implement help dialog snapshot tests (visible, quit confirmation)

## 4. Layout Snapshot Tests

- [x] 4.1 Implement full screen layout snapshots at 80x24, 120x30, 160x40
- [x] 4.2 Implement authentication screen layout snapshots
- [x] 4.3 Implement main application layout snapshots (sidebar collapsed/expanded)
- [x] 4.4 Implement screen title snapshots for all major views
- [x] 4.5 Implement status bar snapshots (help, error, loading states)

## 5. Documentation and CI Integration

- [x] 5.1 Add snapshot testing section to `TESTING.md` with usage instructions
- [x] 5.2 Document `cargo insta review` and `cargo insta accept` workflow
- [x] 5.3 Document how to add new snapshot tests
- [x] 5.4 Verify all snapshot tests pass with `cargo test --test snapshot_test`
- [x] 5.5 Run `cargo insta review` to accept initial snapshots
