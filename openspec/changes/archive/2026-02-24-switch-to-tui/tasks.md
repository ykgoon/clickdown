## 1. Dependencies and Project Setup

- [x] 1.1 Add `ratatui` and `crossterm` dependencies to `Cargo.toml`
- [x] 1.2 Remove `iced` dependency from `Cargo.toml`
- [x] 1.3 Create `src/tui/` module directory structure
- [x] 1.4 Create `src/tui/mod.rs` with module exports
- [x] 1.5 Update `src/main.rs` to initialize TUI instead of iced

## 2. Terminal Framework Implementation

- [x] 2.1 Implement terminal initialization in `src/tui/terminal.rs`
- [x] 2.2 Implement terminal cleanup and shutdown handling
- [x] 2.3 Create event loop with crossterm event subscription
- [x] 2.4 Implement rendering loop with 30 FPS target
- [x] 2.5 Map crossterm events to application Messages
- [x] 2.6 Handle terminal resize events

## 3. Layout System and Screen Titles

- [x] 3.1 Create layout component with title bar, content area, and status bar
- [x] 3.2 Implement screen title generation for each screen type
- [x] 3.3 Implement status bar with loading, error, and help hint states
- [x] 3.4 Implement responsive layout that adapts to terminal size
- [x] 3.5 Add minimum terminal size check (80x24) with warning

## 4. Navigation System

- [x] 4.1 Implement vim-style keyboard bindings (`j/k`, `Enter`, `Esc`)
- [x] 4.2 Implement global navigation keys (`q`, `Tab`, `?`)
- [x] 4.3 Implement context-aware action keys (`n`, `e`, `d`)
- [x] 4.4 Implement workspace hierarchy navigation
- [x] 4.5 Implement focus management between panels
- [x] 4.6 Implement help overlay with keyboard shortcut reference

## 5. Widget Implementation

- [x] 5.1 Create sidebar widget with workspace hierarchy display
- [x] 5.2 Create task list widget with status/priority indicators
- [x] 5.3 Create task detail widget for viewing/editing tasks
- [x] 5.4 Create authentication widget with token input
- [x] 5.5 Create document view widget with Markdown rendering
- [x] 5.6 Create confirmation dialog widget
- [x] 5.7 Implement scrolling for lists that exceed visible area

## 6. Form Handling

- [x] 6.1 Implement text input capture and buffer management
- [x] 6.2 Implement password field masking for token input
- [x] 6.3 Implement multi-line text editing for descriptions
- [x] 6.4 Implement form validation with inline error display
- [x] 6.5 Implement form submission states (loading, success, error)

## 7. Integration and Wiring

- [x] 7.1 Connect TUI event loop to existing Message enum
- [x] 7.2 Wire up authentication flow with TUI widgets
- [x] 7.3 Wire up workspace navigation with sidebar widget
- [x] 7.4 Wire up task CRUD operations with task widgets
- [x] 7.5 Wire up document viewing with document widget
- [x] 7.6 Integrate status bar with application error/loading states

## 8. Remove Screen ID System

- [x] 8.1 Remove screen ID generation code from UI modules
- [x] 8.2 Remove screen ID display from all screens
- [x] 8.3 Archive or delete `openspec/specs/screen-identification/` spec
- [x] 8.4 Update tests to verify screen titles instead of IDs

## 9. Testing and Polish

- [x] 9.1 Test all navigation flows in terminal
- [x] 9.2 Test form inputs and validation
- [x] 9.3 Test terminal resize handling
- [x] 9.4 Test with different terminal sizes and color capabilities
- [x] 9.5 Verify all screen titles are unique and descriptive
- [x] 9.6 Add keyboard shortcut hints to status bar on each screen
- [x] 9.7 Run `cargo test` and fix any broken tests
- [x] 9.8 Build release version and test performance
