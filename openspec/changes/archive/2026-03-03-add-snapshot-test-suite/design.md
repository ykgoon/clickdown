## Context

ClickDown uses ratatui for terminal UI rendering with crossterm backend. The current test suite includes unit tests for models and integration tests using `MockClickUpClient`, but lacks visual regression testing for rendered output. UI changes can introduce visual bugs that aren't caught by existing tests.

**Constraints:**
- Must work in headless CI environments (no actual terminal required)
- Must use existing mock client pattern for API calls
- Should not impact release build size or performance
- Snapshots must be deterministic across platforms (Linux, macOS, Windows)

**Current Architecture:**
- TUI rendering via `TuiApp::render()` in `src/tui/app.rs`
- Widgets in `src/tui/widgets/` (sidebar, task_list, task_detail, auth_view, document_view)
- Layout in `src/tui/layout.rs`
- Test infrastructure in `tests/` using `MockClickUpClient`

## Goals / Non-Goals

**Goals:**
- Add snapshot testing infrastructure using `insta` crate
- Create snapshot tests for all major TUI widgets
- Create snapshot tests for full screen layouts at standard terminal sizes
- Mock all network calls using existing patterns
- Enable easy snapshot review in CI/CD workflows
- Document snapshot testing workflow in TESTING.md

**Non-Goals:**
- Testing actual terminal rendering (crossterm backend behavior)
- Pixel-perfect visual testing (terminal fonts/colors vary)
- Snapshot testing for document Markdown rendering (covered by existing tests)
- Modifying existing unit or integration tests

## Decisions

### 1. Use `insta` crate for snapshot testing

**Rationale:** `insta` is the de facto standard for Rust snapshot testing with:
- Mature ecosystem and active maintenance
- Built-in support for inline and file-based snapshots
- CI-friendly review workflow (`cargo insta review`)
- Deterministic output formatting
- Support for JSON, YAML, and string snapshots

**Alternatives Considered:**
- Custom snapshot solution: Would require building review tooling from scratch
- `expect-test`: More suited for small output snippets, lacks file snapshot workflow
- Manual file comparison: No review workflow, harder to maintain

### 2. Snapshot rendered buffer content, not terminal state

**Rationale:** ratatui renders to a `Buffer` which contains character and style information. Snapshotting the buffer content (via `buffer.content` as string) provides:
- Deterministic output independent of terminal emulator
- Fast execution (no actual terminal rendering)
- Easy comparison and review

**Implementation:** Use `ratatui::backend::TestBackend` to render to an off-screen buffer, then snapshot the buffer content as a string representation.

### 3. Organize snapshots by widget type

**Rationale:** Separate snapshot directories for different test categories:
- `snapshots/widget_snapshots/` - Individual widget rendering
- `snapshots/layout_snapshots/` - Full screen layouts
- `snapshots/state_snapshots/` - Application state transitions

This organization makes it easy to:
- Locate failing snapshots
- Review changes by category
- Add new tests without clutter

### 4. Use fixed test data for deterministic snapshots

**Rationale:** Snapshots must be identical across runs and platforms. Use:
- Fixed timestamps (not `SystemTime::now()`)
- Deterministic mock data from `fixtures.rs`
- Fixed terminal sizes (80x24, 120x30, 160x40)
- Consistent color schemes (dark theme only)

### 5. Mock all external dependencies

**Rationale:** Snapshot tests must be fully isolated:
- Use `MockClickUpClient` for API calls
- Mock clipboard operations (return success without actual clipboard access)
- Mock file system operations where applicable
- No network calls, no disk I/O beyond snapshot storage

## Risks / Trade-offs

**[Snapshot bloat]** → Mitigation: Regular snapshot cleanup, limit test matrix to essential terminal sizes (3 standard sizes)

**[Fragile tests]** → Mitigation: Focus on structural layout, not exact character positions; use descriptive test names

**[CI storage]** → Mitigation: Snapshots are text-based and compressed in git; estimate ~100KB total for initial suite

**[Platform differences]** → Mitigation: Use `insta`'s platform-agnostic formatting; avoid platform-specific rendering paths in tests

**[Maintenance overhead]** → Mitigation: Document snapshot update workflow; use `cargo insta auto-review` for local development

## Migration Plan

1. Add `insta` and `ratatui` test backend to `Cargo.toml` (dev-dependencies)
2. Create test module structure in `tests/snapshot_test.rs`
3. Implement widget snapshot tests (sidebar, task list, auth view)
4. Implement layout snapshot tests (full screen at various sizes)
5. Add snapshot review documentation to TESTING.md
6. Configure CI to run snapshot tests on pull requests

**Rollback:** Remove dev-dependencies and test files; no impact on production code.

## Open Questions

None - design is complete and ready for implementation.
