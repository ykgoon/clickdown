## Context

ClickDown already supports **copying** resource URLs to the clipboard via the `u` key shortcut, using `ClickUpUrlGenerator` (`src/utils/url_generator.rs`) and `ClipboardService` (`src/utils/clipboard.rs`). The URL generation covers all element types: workspace, space, folder, list, task, comment, and document.

The application's navigation is hierarchical: `Auth → Workspaces → Spaces → Folders → Lists → Tasks → TaskDetail`. Each level requires loading data from the ClickUp API (async via `mpsc` channel) before the next level can be entered. Navigation state is tracked via `current_workspace_id`, `current_space_id`, `current_folder_id`, `current_list_id` fields on `TuiApp`.

There is currently no mechanism to accept a URL as input and navigate to the corresponding resource.

## Goals / Non-Goals

**Goals:**
- Accept a ClickUp URL from the user via a text input modal
- Parse the URL to extract resource identifiers (workspace, space, folder, list, task, comment, document)
- Navigate the TUI to the target resource, loading intermediate hierarchy levels as needed
- Provide clear error feedback for invalid URLs, unreachable resources, or missing authentication
- Reuse existing URL patterns from `ClickUpUrlGenerator` for parsing symmetry

**Non-Goals:**
- Support for arbitrary/non-ClickUp URLs (only `app.clickup.com` patterns)
- URL shortener expansion (e.g., bit.ly → ClickUp)
- Auto-detecting clipboard content on paste (user must explicitly invoke the shortcut)
- Opening external URLs in a browser (that's the inverse feature, already supported)
- Document editing or task creation from URL navigation

## Decisions

### 1. URL input triggered by `g` then `u` chord (Go → URL)

**Decision:** Use a two-key chord `g` → `u` (similar to vim's `g`-prefixed navigation commands like `gg`) rather than a modifier-based shortcut.

**Rationale:**
- The existing `u` key is already bound to URL copying. Adding a modifier variant (e.g., `Ctrl+U`) is fragile across terminal emulators — many terminals intercept `Ctrl+U` for line deletion.
- A two-key chord is discoverable (follows vim convention), terminal-safe, and leaves modifier keys free.
- The chord is implemented as a transient state: after `g` is pressed, a brief "pending" state waits for the next key. If the next key is `u`, open URL input; otherwise, pass through the second key as normal input.

**Alternatives considered:**
- `:` command-line mode (vim-style): More flexible long-term but out of scope for this change.
- `Ctrl+Shift+U`: Terminal emulators frequently intercept this; unreliable.
- `/` search-style input: Conflicts with existing search/filter shortcuts.

### 2. URL parsing via reverse of `ClickUpUrlGenerator` patterns

**Decision:** Create a `UrlParser` struct in `src/utils/url_parser.rs` that mirrors the URL generation patterns from `ClickUpUrlGenerator`. The parser extracts a `ParsedUrl` enum variant identifying the resource type and its IDs.

**Rationale:**
- The URL patterns are already documented and tested in `url_generator.rs`. Reversing them ensures symmetry — any URL the app copies can be parsed back.
- A dedicated parser keeps URL logic isolated and testable.
- The `ParsedUrl` enum makes downstream navigation type-safe:

```rust
pub enum ParsedUrl {
    Workspace { workspace_id: String },
    Space { workspace_id: String, space_id: String },
    Folder { workspace_id: String, folder_id: String },
    List { workspace_id: String, list_id: String },
    Task { task_id: String },
    Comment { task_id: String, comment_id: String },
    Document { doc_id: String },
}
```

**Alternatives considered:**
- Regex-based extraction on the URL string: Simpler but harder to maintain as URL patterns evolve.
- URL routing library: Overkill for a fixed set of patterns.

### 3. Navigation strategy: sequential hierarchy loading

**Decision:** When navigating to a resource, load each level of the hierarchy sequentially until the target is reached. For example, navigating to a task URL:
1. Find the workspace (from the URL's workspace_id or from task metadata)
2. Load spaces within that workspace → find the space containing the list
3. Load folders within that space → find the folder containing the list
4. Load lists within that folder → find the target list
5. Load tasks within that list → find and select the target task

For **short-form URLs** (task: `app.clickup.com/t/{task_id}`, comment, document), the URL doesn't contain workspace/list context. The strategy is:
1. Call the ClickUp API to fetch the resource directly (e.g., `GET /task/{task_id}`)
2. Use the response's workspace/list metadata to determine the hierarchy path
3. Navigate through the hierarchy as above

**Rationale:**
- The TUI requires intermediate data to be loaded at each level (sidebar items, task lists). Skipping levels would result in empty/broken views.
- Sequential loading matches the existing `load_spaces → load_folders → load_lists → load_tasks` async pattern.
- For short-form URLs, the ClickUp API returns sufficient context in the resource response to determine the full hierarchy.

**Alternatives considered:**
- Direct navigation to the target view without loading intermediates: Would show empty sidebars and break the UX.
- Batch-loading the entire workspace tree: Too expensive, wastes API quota, slow on large workspaces.

### 4. URL input UI: modal dialog overlay

**Decision:** Display a centered modal dialog with a single-line text input field, similar to existing dialog patterns (`DialogState`, `DialogType`). The dialog shows:
- A prompt: "Enter ClickUp URL:"
- A text input field (supports typing and `Ctrl+V` paste)
- `Enter` to submit, `Esc` to cancel

**Rationale:**
- Consistent with existing dialog patterns in the codebase (`render_dialog`)
- Modal approach blocks other interactions, preventing accidental input during URL entry
- Supports both manual typing and clipboard paste

**Alternatives considered:**
- Status-bar inline input: Less discoverable, harder to implement with existing input routing.
- Dedicated screen: Too heavy for a simple text input.

### 5. Error handling: in-dialog and status bar feedback

**Decision:** 
- Invalid URL format (unparseable): Show error inline in the dialog, keep dialog open for correction
- Valid URL but resource not found (API error): Close dialog, show error in status bar
- Unauthenticated: Close dialog, redirect to auth screen

**Rationale:**
- Format errors are user-correctable (typo in URL), so keep dialog open
- API errors require no user correction, so dismiss and inform

### 6. New dependency: no external crates

**Decision:** Implement URL parsing using only Rust stdlib (`std::str`, pattern matching). No new external dependencies.

**Rationale:**
- The URL patterns are simple and fixed — regex or a routing library would add complexity without benefit.
- Keeps `Cargo.toml` unchanged.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| **API rate limiting during sequential navigation** — Loading multiple hierarchy levels may trigger rate limits on large workspaces | Add a small delay between requests; cache intermediate results; show loading indicator |
| **Short-form task URLs lack workspace context** — The API `GET /task/{id}` response may not include sufficient workspace/list metadata to navigate the hierarchy | Inspect the actual API response; if workspace/list IDs are present, use them. If not, fall back to searching the user's workspaces |
| **URL format changes by ClickUp** — ClickUp may change URL patterns, breaking the parser | Parser is tested against known patterns; add version-aware parsing if patterns diverge |
| **Dialog input routing conflicts** — The modal dialog needs to capture all keyboard input without interfering with existing `update()` dispatch | Route dialog input at the top of `update()` before screen-specific handlers (existing pattern for help/status dialogs) |
| **Performance on deep navigation** — Navigating from a task URL to the task view may require 4+ sequential API calls | Show a loading indicator with current progress (e.g., "Loading... 2/4 levels"); consider parallel requests where possible |
| **Comment URLs from short-form task URLs** — Comment URLs use query params (`?comment=`), which may vary in format | Support both `?comment=` and `/comment/` path-style parsing; test against real examples |

## Migration Plan

This is a pure client-side feature with no server-side changes. Deployment is straightforward:
1. Merge the change — no migration or deployment steps needed
2. Users get the feature on next `cargo build && cargo run`

Rollback: Revert the commit. No data migration required.

## Open Questions

- **Workspace auto-discovery for short-form URLs**: When parsing a short-form task URL (no workspace ID), should we search all accessible workspaces to find the task, or rely on the task API response to include workspace/list context? This depends on the ClickUp API's `GET /task/{id}` response structure and should be verified during implementation.
- **Document navigation**: Documents may not be scoped to a specific list/folder/space hierarchy. The navigation path for documents may differ from tasks. Verify the document API response includes sufficient context for workspace-level navigation.
