## Context

ClickDown currently initializes to a fresh state on every launch, requiring users to re-navigate through the workspace hierarchy each time. The application already has a SQLite cache database (`cache.db`) for storing ClickUp data, and the schema includes a `session_state` table (key-value storage) that is not yet used.

The TUI application uses the Elm architecture pattern with `TuiApp` maintaining:
- Current screen type (Auth, Workspaces, Spaces, Folders, Lists, Tasks, TaskDetail, Document)
- Navigation context (current_workspace_id, current_space_id, current_folder_id, current_list_id)
- Selected items in sidebar and task list
- UI states (dialog, help, comment editing)

The application lifecycle is defined in `src/tui/app.rs` with:
- `TuiApp::new()` - initialization
- `TuiApp::run()` - main loop
- Graceful shutdown via ctrl-q or quit confirmation

## Goals / Non-Goals

**Goals:**
- Persist navigation state (screen type + hierarchy IDs) on graceful shutdown
- Restore user to last viewed location on startup
- Handle gracefully when saved resources no longer exist (fallback to parent)
- Use existing SQLite cache infrastructure
- Minimal user-facing complexity (silent restore with status message)

**Non-Goals:**
- Persisting scroll positions or fine-grained UI state
- Restoring form input state (e.g., partially filled task edits)
- Session persistence across different machines or profiles
- Real-time session sync (single-user, local-only)
- Restoring comment panel state or help/dialog visibility

## Decisions

### 1. Session State Storage Format

**Decision:** Use JSON serialization stored in the existing `session_state` table with key `'current_session'`.

**Rationale:**
- Schema already exists, no migration needed
- JSON is flexible for schema evolution
- Single-row lookup is efficient
- Easy to debug/inspect manually if needed

**Alternatives Considered:**
- Separate columns for each field (screen, workspace_id, space_id, etc.): More rigid, requires schema changes
- Separate config file: Duplicates storage mechanisms, cache.db is already the single source of truth

### 2. When to Save State

**Decision:** Save state on graceful shutdown only (ctrl-q or confirmed quit). Do NOT save on crash, panic, or forced kill.

**Rationale:**
- Simplest implementation (single save point in shutdown flow)
- Avoids I/O overhead during navigation
- Crash recovery naturally falls back to fresh start

**Alternatives Considered:**
- Save on every navigation change: More resilient to crashes, but adds I/O latency during navigation
- Periodic background save: Complex, potential race conditions with shutdown

### 3. When to Restore State

**Decision:** Attempt restoration on every startup if session state exists. Validate that saved IDs still exist before navigating.

**Rationale:**
- Consistent user experience (always restores if possible)
- Validation prevents navigation to deleted resources

**Alternatives Considered:**
- Only restore if explicitly requested (e.g., command-line flag): Adds friction, most users want this
- Time-based expiration (e.g., don't restore if >24h old): Unnecessary complexity, users can clear manually if needed

### 4. Handling Invalid Saved State

**Decision:** Implement cascading fallback - if saved resource doesn't exist, navigate to nearest valid parent and show status message.

**Fallback Hierarchy:**
```
Document deleted → Tasks screen (or Lists if no tasks)
Task deleted     → Tasks screen
List deleted     → Lists screen (in saved folder/space)
Folder deleted   → Folders screen (in saved space)
Space deleted    → Spaces screen (in saved workspace)
Workspace deleted → Workspaces screen (root)
```

**Rationale:**
- User lands somewhere useful, not at auth or blank screen
- Status message provides clarity on what happened
- Matches the hierarchical nature of ClickUp data model

**Alternatives Considered:**
- Always fall back to root (Workspaces): Loses more context than necessary
- Prompt user to choose: Adds friction, most users just want to continue working

### 5. State to Persist

**Decision:** Persist only navigation state, not UI state:
- Screen type (enum variant)
- current_workspace_id
- current_space_id
- current_folder_id
- current_list_id
- current_task_id (when in TaskDetail)
- current_document_id (when in Document)

**NOT persisted:**
- Sidebar scroll position
- Task list scroll position
- Selected index within lists (user can re-select)
- Help/dialog visibility
- Comment panel state
- Form input values

**Rationale:**
- Navigation context is the high-value state (gets user to right place)
- UI state is transient and low-value
- Keeps implementation simple and focused

### 6. Integration Points

**Decision:** Add session methods to `CacheManager`:
- `save_session_state(&mut self, state: &SessionState) -> Result<()>`
- `load_session_state(&self) -> Result<Option<SessionState>>`
- `clear_session_state(&mut self) -> Result<()>`

Add session handling to `TuiApp`:
- `fn save_session_state(&self) -> Result<()>` - called before shutdown
- `fn restore_session_state(&mut self) -> Result<Option<SessionState>>` - called during initialization

**Rationale:**
- CacheManager already handles all database operations
- TuiApp owns application state and lifecycle
- Clean separation of concerns

## Risks / Trade-offs

**[State becomes stale]** → User navigates away before exit, old state is restored.
- *Mitigation:* Status message shows what was restored; user can navigate again immediately

**[Performance impact on shutdown]** → JSON serialization + DB write adds latency to exit.
- *Mitigation:* State is small (<1KB); SQLite writes are fast; user won't notice ~10ms

**[Database corruption]** → If cache.db is corrupted, session state is lost.
- *Mitigation:* Graceful fallback to fresh start; cache is rebuildable from API

**[Privacy concern]** → Session state reveals user's last activity.
- *Mitigation:* Local-only storage; no sync; user can delete cache.db if needed

**[Complexity in validation]** → Checking if saved IDs exist may require API calls.
- *Mitigation:* Validate against cached data first; if cache is stale, API calls happen anyway during normal initialization

**[Race condition with async loading]** → Restoration happens before data is loaded.
- *Mitigation:* Restore navigation IDs first; let normal data loading populate lists; selection will find the right item once data arrives

## Migration Plan

This is a client-side feature with no external dependencies or API changes.

**Implementation Steps:**
1. Add `SessionState` struct to `src/models/` or `src/tui/`
2. Add session methods to `CacheManager` in `src/cache/mod.rs`
3. Add `save_session_state()` and `restore_session_state()` to `TuiApp`
4. Integrate `restore_session_state()` into `TuiApp::new()` or early initialization
5. Integrate `save_session_state()` into shutdown flow (before `std::process::exit(0)`)
6. Add status messages for restore feedback
7. Add tests for fallback scenarios

**Rollback Strategy:**
- Feature is self-contained; can be reverted by removing session-related code
- Existing cache.db schema is unchanged (session_state table already exists)
- No data migration required

## Open Questions

1. **Should we persist the `AppState` enum value (Initializing, Main, etc.) or just the `Screen`?**
   - Leaning toward just `Screen` - AppState is more about initialization flow than user context

2. **Should logout clear session state?**
   - Leaning toward yes - different user should start fresh
   - But what about same user on different profiles? (Not currently supported)

3. **Should there be a way to disable session restore?**
   - Could add a config option (`restore_session = false`)
   - Out of scope for initial implementation; can add if users request

4. **What about multi-window or multi-instance scenarios?**
   - Out of scope - TUI is single-window; multiple instances would overwrite each other's state
   - Acceptable limitation for now
