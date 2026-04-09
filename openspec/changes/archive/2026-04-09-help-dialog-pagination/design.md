## Context

The help dialog (`src/tui/widgets/help.rs`) currently renders all 9 shortcut sections as fixed-height `Paragraph` widgets within a single `Layout` split. The layout uses hardcoded `Constraint::Length` values totaling ~43 content lines. The dialog is sized at 70%×70% of the terminal, yielding only ~15 content rows on a minimum 80×24 terminal — less than half of what's needed. Sections at the bottom (Comments, Forms, Session) are consistently truncated.

The current close interaction is "press any key to close," which prevents using j/k for navigation within the dialog. The `HelpState` struct is minimal: just a `visible` bool.

The app's `get_hints()` method returns an empty string when help is visible, leaving the status bar blank — this space should show pagination info.

## Goals / Non-Goals

**Goals:**
- All shortcuts accessible on minimum-size terminals (80×24)
- Page 1 shows contextually relevant shortcuts (current screen + focus state)
- Page 2 is a stable anchor (Global shortcuts — always the same content)
- Page 3 contains remaining sections not shown on page 1
- Pagination navigation via j/k or ←/→ arrow keys
- Explicit close via Esc or ? (not "any key")
- Pagination footer within the dialog area
- No new external dependencies

**Non-Goals:**
- Scrollable lists within pages (content should always fit per page)
- Dynamic page count based on terminal size (fixed 3 pages)
- Search/filter within shortcuts
- Changing the set of documented shortcuts

## Decisions

### 1. Paginated state in `HelpState`

Add `page: u8` and `total_pages: u8` fields to `HelpState`. The `total_pages` is always 3 for this change.

```rust
pub struct HelpState {
    pub visible: bool,
    pub page: u8,       // 0-indexed: 0, 1, 2
}
```

Add methods: `next_page()`, `prev_page()`, `go_to_first_page()`, `reset()` (sets page=0 and visible=false on hide).

### 2. Page content determined at render time based on app context

Rather than baking context-awareness into `HelpState` itself, the render function receives a `HelpContext` enum that tells it which page 1 content to show:

```rust
pub enum HelpContext {
    Auth,
    Navigation,     // Workspace/Space/Folder screens
    TaskList,
    TaskDetail,     // Description focus
    Comments,       // Comment focus
    Document,
}
```

The app passes the current context when calling `render_help()`. The render function maps context → page 1 content, with pages 2 and 3 derived accordingly.

**Alternative considered:** Store the context in `HelpState` when opening. Rejected because it duplicates app state and risks going stale. Computing at render time is simpler and always accurate.

### 3. Page 2 (Global) is always the same content

Page 2 contains: Navigation, Global, Actions, Forms. These are ~16 lines including section titles and spacers — fits comfortably within the dialog's inner area at minimum terminal size.

### 4. Page 3 (Reference) is the complement of page 1

Page 3 contains all sections NOT shown on page 1 or page 2. Since page 2 is fixed, page 3 varies based on what page 1 consumed:

```
If page 1 = TaskList     → page 3 = TaskDetail + Comments + Document + Session
If page 1 = TaskDetail   → page 3 = TaskList + Comments + Document + Session
If page 1 = Comments     → page 3 = TaskDetail + TaskList + Document + Session
If page 1 = Navigation   → page 3 = TaskList + TaskDetail + Comments + Document + Session
```

Session info (2 lines, not real shortcuts) goes on page 3 as the lowest-priority content.

### 5. Render each page as a single scrollable `Vec<Line>`

Instead of the current approach of rendering multiple `Paragraph` widgets at fixed layout positions, build a single `Vec<Line>` per page and render it as one `Paragraph`. This:
- Eliminates the complex Layout constraint math
- Makes it trivial to ensure content fits (just count lines)
- Allows a single consistent rendering path per page

Section headers use bold `Span`s, key bindings use a fixed-width format with padding.

### 6. Dialog title shows current page info

Format: `Keyboard Shortcuts — <Section Name>  (1/3)`

This tells users both what they're looking at and where they are in the sequence.

### 7. Footer replaces the "Press any key to close" hint

```
◄ ►  1/3  │  j/k: Pages  │  Esc: Close
```

Rendered at the bottom of the dialog inner area. Uses `Theme::SECONDARY` for the dimmed style.

### 8. Close behavior change: Esc/? instead of any key

The app's key handler in `app.rs` currently routes any keypress to hide the help dialog. This changes to only respond to `Esc` and `?` for closing, and `j/k`/`←/→` for pagination. Other keys are ignored (no passthrough to underlying UI while help is open).

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Page 3 content could still overflow on small terminals | Page 3 is reference material — users rarely need it. If it overflows, the `Constraint::Min(1)` ensures ratatui clips gracefully rather than crashing. Future: add intra-page scrolling. |
| Breaking "any key closes" muscle memory | `Esc` and `?` are both natural close keys for a modal. The footer explicitly shows "Esc: Close". |
| HelpContext enum drift as new screens are added | The enum is small and co-located with `HelpState`. New screens already update `get_hints()` — adding a `HelpContext` variant follows the same pattern. |
| Total page count hardcoded to 3 | Acceptable for now. If more shortcuts are added, the page count can be dynamic. For now, fixed 3 keeps the UX predictable. |

## Migration Plan

No migration needed — this is a pure UI change. The help dialog is stateless between opens (resets to page 1 each time).

## Open Questions

- Should the help dialog reset to page 1 each time it's opened? **Decision: Yes** — users expect to see the contextual page first every time.
- Should arrow keys (←/→) also paginate, or just j/k? **Decision: Yes** — both j/k and ←/→ for discoverability.
