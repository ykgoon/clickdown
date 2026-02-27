## 1. Update Help Widget

- [x] 1.1 Add `get_help_hints()` function to return `"? - Help"` string
- [x] 1.2 Export `get_help_hints()` from `src/tui/widgets/help.rs`
- [x] 1.3 Export `get_help_hints()` from `src/tui/widgets/mod.rs`

## 2. Integrate Help Shortcut in Status Bar

- [x] 2.1 Update `get_hints()` in `src/tui/app.rs` to append ` | ? - Help` to all hint strings
- [x] 2.2 Verify `?` appears in status bar on all screens (Auth, Workspaces, Tasks, Task Detail, Document)

## 3. Fix Help Dialog Close Behavior

- [x] 3.1 Modify `update()` in `src/tui/app.rs` to check `self.help.visible` first
- [x] 3.2 When help is visible, any key press calls `self.help.hide()` and returns early
- [x] 3.3 Ensure `?` key still toggles help (closes when already open)

## 4. Update Help Dialog Content

- [x] 4.1 Review shortcuts in `src/tui/widgets/help.rs` match actual key bindings in `src/tui/app.rs`
- [x] 4.2 Add any missing shortcuts to help dialog (fixed Ctrl+Q vs q)
- [x] 4.3 Verify close message says "Press any key to close"

## 5. Testing

- [x] 5.1 Test `?` opens help dialog from all screens
- [x] 5.2 Test any key closes help dialog
- [x] 5.3 Test shortcuts are blocked while help is open
- [x] 5.4 Test `?` hint appears in status bar on all screens
- [x] 5.5 Run `cargo test` to ensure no regressions

## 6. Documentation

- [x] 6.1 Update README.md if new shortcuts are added
- [x] 6.2 Verify AGENTS.md reflects the help dialog feature
