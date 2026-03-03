## Technical Approach

### Overview

The fix implements **progressive session restore** by replaying the navigation chain asynchronously. Instead of just setting the screen type and IDs, the system now:

1. Sets a `restoring_session` flag when restore begins
2. Stores the target IDs for each navigation level
3. As data loads at each level, finds and selects the matching item
4. Automatically triggers the next level load after selection
5. Clears the flag when the chain completes or fallback occurs

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    SESSION RESTORE FLOW                         │
└─────────────────────────────────────────────────────────────────┘

TuiApp::new()
    │
    ├─► restore_session_state()
    │   ├─► Load SessionState from cache
    │   ├─► Set restoring_session = true
    │   ├─► Store target IDs (workspace_id, space_id, etc.)
    │   └─► Set screen to deepest valid level
    │
    └─► load_workspaces()
        │
        ▼
WorkspacesLoaded Handler
    ├─► Check restoring_session flag
    │
    ├─► If restoring:
    │   ├─► Find workspace with matching ID
    │   ├─► If found: select it, load_spaces()
    │   └─► If not found: fallback to Workspaces screen
    │
    └─► If not restoring:
        └─► select_first() (original behavior)
        │
        ▼
SpacesLoaded Handler
    ├─► Check restoring_session flag
    │
    ├─► If restoring:
    │   ├─► Find space with matching ID
    │   ├─► If found: select it, load_folders()
    │   └─► If not found: fallback to Spaces screen
    │
    └─► If not restoring:
        └─► select_first() (original behavior)
        │
        ▼
... (repeat for Folders, Lists, Tasks)
        │
        ▼
Restore Complete
    └─► Clear restoring_session flag
        Show status message
```

### Data Structures

#### New Fields in `TuiApp`

```rust
/// Session restore state
restoring_session: bool,
restored_workspace_id: Option<String>,
restored_space_id: Option<String>,
restored_folder_id: Option<String>,
restored_list_id: Option<String>,
restored_task_id: Option<String>,
```

#### New Methods in `SidebarState`

```rust
/// Select item by ID, returns true if found
pub fn select_by_id(&mut self, id: &str) -> bool
```

### Implementation Details

#### 1. Session Restore Initialization

In `TuiApp::new()`:
- Call `restore_session_state()` before `load_workspaces()`
- The method loads the SessionState and sets the restoring flags
- Returns true if restore is in progress, false if no saved state

#### 2. Async Handler Modifications

Each `*Loaded` handler gets modified to:

```rust
if self.restoring_session {
    // Try to select the restored item
    let found = self.sidebar.select_by_id(&self.restored_workspace_id);
    if found {
        // Continue the chain
        self.load_spaces(...);
    } else {
        // Fallback
        self.restoring_session = false;
        self.screen = Screen::Workspaces;
        self.status = "Saved workspace not found".to_string();
    }
} else {
    // Original behavior
    self.sidebar.select_first();
}
```

#### 3. Selection Logic

The `select_by_id()` method iterates through sidebar items and selects the matching one:

```rust
pub fn select_by_id(&mut self, id: &str) -> bool {
    for (i, item) in self.items.iter().enumerate() {
        if item.id() == id {
            self.selected.select(Some(i));
            return true;
        }
    }
    false
}
```

This requires adding an `id()` method to `SidebarItem`:

```rust
impl SidebarItem {
    pub fn id(&self) -> &str {
        match self {
            SidebarItem::Workspace { id, .. } => id,
            SidebarItem::Space { id, .. } => id,
            SidebarItem::Folder { id, .. } => id,
            SidebarItem::List { id, .. } => id,
        }
    }
}
```

#### 4. Fallback Logic

Fallback occurs when:
- The saved ID doesn't match any loaded item
- The saved screen type is invalid

The fallback cascades to the nearest valid parent:
- Invalid task → show Tasks
- Invalid list → show Lists
- Invalid folder → show Folders
- Invalid space → show Spaces
- Invalid workspace → show Workspaces

#### 5. Completion Detection

The restore is complete when:
- All levels have been processed successfully, OR
- A fallback occurred

At completion:
- Clear `restoring_session` flag
- Show appropriate status message
- Normal navigation resumes

### Edge Cases

#### Partial Save
If the user quit while in the middle of navigation (e.g., spaces loading), only some IDs may be set. The restore handles this by:
- Only restoring levels that have IDs
- Stopping the chain at the deepest saved level

#### Deleted Resources
If a saved resource was deleted:
- The ID won't match any loaded item
- Fallback to the parent level occurs
- User sees a status message explaining the fallback

#### First Launch
If no session exists:
- `restore_session_state()` returns false
- `restoring_session` remains false
- Normal `select_first()` behavior occurs

### Testing Strategy

#### Unit Tests
- `SidebarState::select_by_id()` - finds matching item
- `SidebarState::select_by_id()` - returns false for non-existent ID
- SessionState serialization/deserialization

#### Integration Tests
- Session saved on quit, restored on startup
- Full navigation chain replay
- Fallback when workspace deleted
- Fallback when space deleted
- Fallback when folder deleted
- Fallback when list deleted
- Partial restore (missing intermediate levels)
- First launch (no session)

#### Manual Testing
1. Launch app, navigate to a task
2. Quit with Ctrl+Q
3. Relaunch, verify task detail is restored
4. Delete a list in ClickUp web UI
5. Relaunch app, verify fallback to Lists screen

### Performance Considerations

- Restore adds minimal overhead (one ID comparison per level)
- No additional API calls - uses existing load methods
- Async chain doesn't block UI
- Status messages provide feedback during restore

### Backwards Compatibility

- Existing behavior preserved when no session exists
- First-time users unaffected
- Token-based auth flow unchanged
- Cache schema unchanged (uses existing session_state table)
