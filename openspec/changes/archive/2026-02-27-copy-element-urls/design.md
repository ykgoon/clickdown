## Context

ClickDown provides terminal-based access to ClickUp elements (workspaces, spaces, folders, lists, tasks, comments, documents). Users sometimes need to view these elements in the ClickUp web app when they need details not available in ClickDown (e.g., rich attachments, custom field views, activity logs, or collaboration features).

Currently, there's no quick way to access the web URL for these elements. Users must manually navigate to ClickUp in a browser and search for the element, which is time-consuming and breaks their workflow.

ClickUp uses consistent URL patterns for all elements, and the application already has the necessary IDs (workspace ID, task ID, comment ID, etc.) to construct these URLs.

## Goals / Non-Goals

**Goals:**
- Generate ClickUp web app URLs for all element types (workspace, space, folder, list, task, comment, document)
- Add keyboard shortcut to copy URL to clipboard from relevant views
- Provide visual feedback when URL is copied
- Cross-platform clipboard support (Linux, macOS, Windows)
- Minimal dependencies - use lightweight clipboard crate
- Preserve existing functionality - no breaking changes

**Non-Goals:**
- Opening URLs automatically in browser (user may prefer to paste URL manually)
- URL shortening or customization
- QR code generation
- Social sharing features
- Document editing or rich text support

## Decisions

### 1. URL Pattern Strategy

**Decision:** Use ClickUp's standard URL patterns based on element type.

ClickUp URL patterns:
- Workspace: `https://app.clickup.com/{workspace_id}`
- Space: `https://app.clickup.com/{workspace_id}/s/{space_id}`
- Folder: `https://app.clickup.com/{workspace_id}/f/{folder_id}`
- List: `https://app.clickup.com/{workspace_id}/l/{list_id}`
- Task: `https://app.clickup.com/{workspace_id}/l/{list_id}/t/{task_id}`
- Comment: `https://app.clickup.com/{workspace_id}/l/{list_id}/t/{task_id}/comment/{comment_id}`
- Document: `https://app.clickup.com/{workspace_id}/d/{doc_id}`

**Rationale:** These are the canonical URLs ClickUp uses. Users can navigate directly to any element.

**Alternatives considered:**
- Use ClickUp's share URLs: More complex, requires API calls
- Store URLs in models: Wastes memory, URLs are derivable from IDs

### 2. Clipboard Library Selection

**Decision:** Use `arboard` crate for cross-platform clipboard access.

**Rationale:**
- Cross-platform (Linux, macOS, Windows)
- No external dependencies (uses native clipboard APIs)
- Well-maintained, simple API
- Lightweight (~10 dependencies)
- Supports both text and image (we only need text)

**Alternatives considered:**
- `copypasta`: Older, less maintained
- `x11-clipboard` + `macos-clipboard` + `windows-clipboard`: Platform-specific, more complex
- Shell commands (`xclip`, `pbcopy`): Platform-dependent, requires external tools

### 3. Keyboard Shortcut

**Decision:** Use `Ctrl+Shift+C` for copying URL (or `Cmd+Shift+C` on macOS).

**Rationale:**
- `C` for "Copy" or "URL Copy"
- Shift modifier distinguishes from regular `C` key
- Ctrl/Cmd is standard for copy operations
- Doesn't conflict with existing shortcuts (j/k navigation, Enter select, Esc back)

**Alternatives considered:**
- `y` then `u`: Vim-style copy URL (too complex)
- `Ctrl+U`: Common browser shortcut for "view source"
- `Ctrl+Shift+U`: Unicode input on Linux
- Context menu: Not idiomatic for TUI

### 4. URL Generation Architecture

**Decision:** Implement `UrlGenerator` trait with methods for each element type.

```rust
trait UrlGenerator {
    fn workspace_url(&self) -> String;
    fn space_url(&self) -> Option<String>;
    fn folder_url(&self) -> Option<String>;
    fn list_url(&self) -> Option<String>;
    fn task_url(&self) -> Option<String>;
    fn comment_url(&self, comment_id: &str) -> Option<String>;
    fn document_url(&self) -> Option<String>;
}
```

**Rationale:**
- Clean separation of concerns
- Easy to test
- Extensible for future element types
- Models don't need to store URLs (computed on demand)

**Alternatives considered:**
- Add `url()` method to each model: Couples models to URL logic
- Store URLs in API responses: Wastes bandwidth, URLs are derivable

### 5. Context Awareness for URL Copy

**Decision:** URL copy action is context-aware - copies the most relevant URL for the current view.

| View | URL Copied |
|------|------------|
| Workspace list | Selected workspace URL |
| Space list | Selected space URL |
| Folder list | Selected folder URL |
| List view | Selected list URL |
| Task list | Selected task URL |
| Task detail | Selected task URL |
| Comment thread | Selected comment URL (with task context) |
| Document view | Current document URL |

**Rationale:** User always copies the most relevant URL for their current context.

### 6. Feedback Mechanism

**Decision:** Show brief confirmation in status bar: "Copied: <URL>" (truncated if long).

**Rationale:**
- Consistent with existing status bar usage
- Non-intrusive
- Confirms action succeeded
- Shows what was copied (user can verify)

**Alternatives considered:**
- Toast notification: More complex UI changes
- Modal dialog: Interrupts workflow
- No feedback: User unsure if copy succeeded
- Flash screen effect: Too distracting in TUI

### 7. Error Handling

**Decision:** If clipboard access fails, show error in status bar but don't crash.

**Rationale:**
- Clipboard may be unavailable (headless SSH, Wayland issues, etc.)
- Application should remain usable
- User should know why copy failed

**Error message:** "Failed to copy URL: <reason>"

## Risks / Trade-offs

### [Risk] Clipboard library compatibility issues on Linux/Wayland

**Mitigation:** 
- `arboard` uses X11/Wayland/Windows/macOS native APIs
- Test on common Linux distributions
- Graceful error handling if clipboard unavailable
- Document known issues in README

### [Risk] URL patterns may change if ClickUp updates their routing

**Mitigation:**
- URL generation logic is isolated in one module
- Easy to update patterns
- Add tests for URL patterns
- Monitor ClickUp UI changes

### [Risk] Comment URLs require task context (list ID, task ID)

**Mitigation:**
- Comment views always have parent task context
- Store task/list ID in comment view state
- If context missing, disable URL copy for that comment

### [Trade-off] No automatic browser opening

**Rationale:** Some users work in terminal-only environments or prefer to paste URLs elsewhere. Automatic opening would require additional dependency (`open` crate) and may be unwanted behavior. Can be added later if requested.

### [Trade-off] URL truncation in feedback

**Rationale:** Long URLs may not fit in status bar. Truncate to 60 chars with "..." indicator. User can paste to see full URL.

## Migration Plan

Not applicable - this is a new feature with no data migration or breaking changes.

**Implementation steps:**
1. Add `arboard` dependency to `Cargo.toml`
2. Create `src/utils/url_generator.rs` with URL generation logic
3. Add `Message::CopyUrl` variant to app message enum
4. Add keyboard handler for `Ctrl+Shift+C` in input handling
5. Add clipboard integration in TUI app update loop
6. Add status bar feedback for copy success/failure
7. Add context-aware URL generation for each view
8. Write unit tests for URL generation
9. Update keyboard shortcuts help text
10. Test on Linux, macOS, Windows

**Rollback strategy:**
- Feature is isolated - can be removed by reverting commits
- No database changes
- No API changes
- Keyboard shortcut is additive (no conflicts)

## Open Questions

1. **Should we support custom ClickUp subdomain URLs?** (e.g. `https://custom.clickup.com`)
   - Current assumption: Use `app.clickup.com` for all
   - If needed: Add `base_url` config option

2. **Should we add a "Share" menu with multiple URL options?**
   - Current: Single URL per context
   - Future: Could add share URLs, permalinks, etc.

3. **Should we support copying multiple URLs at once?**
   - Current: One URL at a time
   - Future: Could add multi-select + batch copy
