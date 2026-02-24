## Context

ClickDown is currently built on the iced GUI framework (version 0.13), which requires a graphical display server (X11/Wayland). The application follows the Elm architecture pattern with a central `ClickDown` struct managing state and a `Message` enum handling all state transitions.

The current UI layer consists of:
- `src/ui/sidebar.rs` - Navigation sidebar for workspace hierarchy
- `src/ui/task_list.rs` - Task list view with status/priority indicators
- `src/ui/task_detail.rs` - Task create/edit panel
- `src/ui/auth_view.rs` - Authentication screen
- `src/ui/document_view.rs` - Document/Markdown viewer

The application uses screen IDs (4-character alphanumeric identifiers) for debugging, as defined in the `screen-identification` spec. These are displayed in the bottom-left corner of each screen.

**Constraints:**
- Must maintain all existing functionality (workspace navigation, task CRUD, document viewing)
- Must preserve the Elm architecture pattern
- Must work within terminal constraints (limited colors, fixed-width fonts, no mouse by default)
- Screen titles must be unique and visible on every screen for debugging

## Goals / Non-Goals

**Goals:**
- Replace iced with Ratatui + crossterm for terminal-based rendering
- Implement keyboard-first navigation throughout the application
- Display unique screen titles prominently on each screen (replacing screen IDs)
- Maintain the existing Message-based state management pattern
- Preserve all current features: auth, workspace nav, task CRUD, document viewing
- Support standard terminal sizes (minimum 80x24)

**Non-Goals:**
- Adding new features beyond the TUI conversion
- Mouse support as primary interaction (optional enhancement only)
- Supporting terminals without 256-color capability
- Real-time collaboration features
- Offline mode improvements

## Decisions

### Decision 1: Use Ratatui as the TUI framework

**Rationale:** Ratatui is the most mature and actively maintained TUI framework for Rust. It provides:
- Terminal-agnostic rendering via crossterm backend
- Widget-based composition similar to iced's mental model
- Strong community and extensive examples
- Good documentation and active development

**Alternatives considered:**
- `crossterm` alone: Too low-level; would require building widget system from scratch
- `termion`: Less actively maintained, fewer features
- `tui-rs` (predecessor): Now deprecated in favor of Ratatui

### Decision 2: Replace screen IDs with screen titles

**Rationale:** Screen titles are more descriptive and debuggable than 4-character IDs. Each screen will display a unique title at the top:
- Auth screen: "ClickDown - Authentication"
- Workspace view: "ClickDown - Workspaces"
- Space view: "ClickDown - {space_name}"
- Task list: "ClickDown - Tasks: {list_name}"
- Task detail: "ClickDown - Task: {task_name}"
- Document view: "ClickDown - Doc: {doc_title}"

This replaces the `screen-identification` spec entirely.

**Alternatives considered:**
- Keep screen IDs + add titles: Redundant; titles provide better context
- Use breadcrumb navigation: More complex, less at-a-glance clarity

### Decision 3: Keyboard-first navigation with vim-style bindings

**Rationale:** Terminal users expect keyboard-centric interaction. Primary navigation:
- `j/k` or `↑/↓` - Move selection up/down
- `Enter` - Select/open item
- `Esc` - Go back/close panel
- `q` - Quit (with confirmation)
- `Tab` - Switch focus between panels
- `n` - Create new item (context-aware)
- `e` - Edit selected item
- `d` - Delete selected item (with confirmation)

**Alternatives considered:**
- Number-based selection (press `1`, `2`, `3`): Doesn't scale for long lists
- Mouse-first: Defeats TUI purpose; keep as optional enhancement

### Decision 4: Layout structure with fixed sidebar

**Rationale:** Maintain the existing sidebar navigation pattern:
```
┌─────────────────────────────────────────────────────────────┐
│ ClickDown - {Screen Title}                                  │
├──────────────┬──────────────────────────────────────────────┤
│              │                                              │
│  Sidebar     │           Main Content Area                  │
│  (20-25%)    │           (75-80%)                           │
│              │                                              │
│  - Workspaces│    - Task list / Detail / Auth / Doc         │
│  - Spaces    │                                              │
│  - Folders   │                                              │
│  - Lists     │                                              │
│              │                                              │
├──────────────┴──────────────────────────────────────────────┤
│ Status bar: {loading状态} | {error messages} | {help hints} │
└─────────────────────────────────────────────────────────────┘
```

**Alternatives considered:**
- Tab-based navigation: Less at-a-glance hierarchy visibility
- Full-screen per-section: Loses context of workspace structure

### Decision 5: Maintain Elm architecture with crossterm events

**Rationale:** The existing Message pattern maps well to TUI:
- `crossterm::Event::Key` → `Message::KeyPressed(Key)`
- Application state machine remains unchanged
- Only the view layer and event subscription change

**Message additions:**
- `Message::KeyPressed(KeyEvent)` - Handle keyboard input
- `Message::TerminalResized(u16, u16)` - Handle resize events

**Alternatives considered:**
- Direct state mutation in event handlers: Breaks existing pattern
- Command pattern: Over-engineering for this scope

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| **Terminal size constraints** - Complex forms may not fit in 80x24 | Use scrolling regions, multi-page forms, and responsive layout that adapts to available space |
| **Loss of rich interactions** - No drag-drop, limited mouse support | Focus on keyboard efficiency; add mouse support as optional enhancement later |
| **Markdown rendering limitations** - Terminal has limited styling | Use pulldown-cmark with terminal-appropriate styling (bold, italic, lists); skip complex HTML |
| **Learning curve for GUI users** - Different interaction model | Provide clear help screen (`?` key) with keybindings; maintain familiar navigation structure |
| **Screen title length** - Long task/space names may truncate | Truncate with ellipsis, show full name in status bar on selection |
| **Color palette limitations** - Terminal may have limited colors | Use 256-color palette with graceful degradation; ensure readability in monochrome |
| **Testing complexity** - TUI testing differs from GUI | Use ratatui's buffer-based testing; snapshot testing for screen rendering |

## Migration Plan

1. **Add dependencies**: Add `ratatui` and `crossterm` to `Cargo.toml`; remove `iced`
2. **Create TUI module**: Create `src/tui/` directory parallel to existing `src/ui/`
3. **Implement terminal setup**: Terminal initialization, rendering loop, event subscription
4. **Recreate views**: Port each UI module to TUI widgets (auth, sidebar, task_list, task_detail, document_view)
5. **Implement screen title system**: Replace screen ID logic with title generation
6. **Update main.rs**: Replace iced application entry point with Ratatui terminal loop
7. **Remove screen-identification spec**: Delete or archive the spec
8. **Test and iterate**: Verify all flows work in terminal

**Rollback strategy:** Keep iced code until TUI is fully functional; use feature flag for switching.

## Open Questions

1. Should we support both GUI and TUI builds via feature flags, or fully replace?
2. What is the minimum terminal size to support? (Recommendation: 80x24)
3. Should mouse clicks be supported for basic navigation (click to select)?
4. How to handle forms that exceed terminal height? (Recommendation: scrolling input regions)
