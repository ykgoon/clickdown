## 1. Update Sidebar Rendering Logic

- [x] 1.1 Remove hardcoded indentation prefixes from `render_sidebar` function in `src/tui/widgets/sidebar.rs`
- [x] 1.2 Update the match expression to render all items flush-left without leading spaces
- [x] 1.3 Preserve type-specific styling (bold for workspaces, cyan for lists) without indentation

## 2. Add Type Labels for Hierarchy

- [x] 2.1 Add type label prefixes (e.g., "WS", "SP", "FL", "LI") to each item type
- [x] 2.2 Ensure type labels are visually distinct (e.g., dimmed or different color)
- [x] 2.3 Test that type labels align consistently across all item types

## 3. Visual Testing

- [x] 3.1 Build the application with `cargo build`
- [x] 3.2 Launch the TUI and navigate through deep hierarchy (Workspace → Space → Folder → List)
- [x] 3.3 Verify all items render flush-left with no indentation
- [x] 3.4 Verify type labels are visible and consistent
- [x] 3.5 Check that horizontal space is preserved for longer item names

## 4. Code Quality

- [x] 4.1 Run `cargo test` to ensure no existing tests break
- [x] 4.2 Update or add unit tests for sidebar rendering if needed
- [x] 4.3 Run any linters or formatters (`cargo fmt`, `cargo clippy`)
- [x] 4.4 Remove unused `indent` field from SidebarItem enum if no longer needed (optional cleanup)

## 5. Documentation

- [x] 5.1 Update help dialog if type labels are introduced
- [x] 5.2 Note the change in changelog or release notes
