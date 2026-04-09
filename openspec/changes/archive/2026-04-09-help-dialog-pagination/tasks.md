## 1. HelpState refactoring

- [x] 1.1 Add `page: u8` field to `HelpState` struct in `src/tui/widgets/help.rs`
- [x] 1.2 Add `next_page()`, `prev_page()`, and `reset()` methods to `HelpState` (wrapping at boundaries)
- [x] 1.3 Update `HelpState::new()` to initialize `page: 0`
- [x] 1.4 Define `HelpContext` enum with variants: Auth, Navigation, TaskList, TaskDetail, Comments, Document

## 2. Paginated render function

- [x] 2.1 Replace single-page `render_help()` with page-aware version that accepts `HelpContext`
- [x] 2.2 Implement page 1 content builder (contextual shortcuts based on `HelpContext`)
- [x] 2.3 Implement page 2 content builder (always Navigation + Global + Actions + Forms)
- [x] 2.4 Implement page 3 content builder (remaining sections not on page 1)
- [x] 2.5 Add page indicator to dialog title: `Keyboard Shortcuts — <Name>  (N/3)`
- [x] 2.6 Add pagination footer: `◄ ►  N/3  │  j/k: Pages  │  Esc: Close`
- [x] 2.7 Verify all pages fit within the dialog area at minimum terminal size (80×24)

## 3. App integration

- [x] 3.1 Update `render_help()` call in `src/tui/app.rs` to pass current `HelpContext`
- [x] 3.2 Add method to determine current `HelpContext` from app state (screen + comment_focus)
- [x] 3.3 Update key handler: `j`/`↓`/`→` calls `help.next_page()` when help is visible
- [x] 3.4 Update key handler: `k`/`↑`/`←` calls `help.prev_page()` when help is visible
- [x] 3.5 Update key handler: only `Esc` and `?` close the help dialog (remove "any key" behavior)
- [x] 3.6 Update `help.toggle()` to reset page to 0 when opening
- [x] 3.7 Update `get_hints()` to show pagination info when help is visible (replace empty string)

## 4. Tests

- [x] 4.1 Write unit tests for `HelpState::next_page()` wrapping behavior
- [x] 4.2 Write unit tests for `HelpState::prev_page()` wrapping behavior
- [x] 4.3 Write test: help dialog opens on page 1
- [x] 4.4 Write test: j key advances page, k key goes back
- [x] 4.5 Write test: Esc closes help dialog
- [x] 4.6 Write test: non-navigation keys do not close help dialog
- [x] 4.7 Write test: contextual page 1 content matches current screen

## 5. Spec archive

- [x] 5.1 Run `openspec verify` to confirm all specs are satisfied
- [x] 5.2 Archive the change with `openspec archive`
