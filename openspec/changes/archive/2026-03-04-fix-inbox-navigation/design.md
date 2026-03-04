## Context

The ClickDown TUI application uses an Elm architecture pattern with a `navigate_into()` function in `src/tui/app.rs` that handles navigation when the user presses Enter on a sidebar item. The sidebar contains an "Inbox" item at the top, followed by workspace hierarchy items (Workspaces → Spaces → Folders → Lists).

**Current State:**
- The `navigate_into()` function uses a match statement on `self.screen` to determine navigation behavior
- Each screen type (Workspaces, Spaces, Folders, Lists) only handles its specific item type
- The `Screen::Inbox` case exists but only handles navigation when already in the Inbox screen
- When "Inbox" is selected in the sidebar from any other screen, pressing Enter falls through without action

**Root Cause:**
The match arms for `Screen::Workspaces`, `Screen::Spaces`, `Screen::Folders`, and `Screen::Lists` check for their specific sidebar item types but don't include a case for `SidebarItem::Inbox`.

## Goals / Non-Goals

**Goals:**
- Enable Inbox navigation from any workspace hierarchy screen
- Maintain consistent navigation behavior (Enter on sidebar item navigates to that destination)
- Minimal code change with no breaking changes to existing functionality

**Non-Goals:**
- No changes to Inbox functionality itself
- No changes to notification loading or caching logic
- No UI/visual changes to the sidebar or inbox view

## Decisions

### Decision: Add Inbox handling to each navigation screen match arm

**Approach:**
Each match arm in `navigate_into()` will check for `SidebarItem::Inbox` first, before checking for its specific item type.

**Rationale:**
- Consistent with existing navigation pattern
- Minimal code duplication
- Easy to understand and maintain
- No changes to data flow or state management

**Alternative Considered:**
Extract Inbox navigation logic to a separate method and call it from each screen. Rejected because:
- The Inbox navigation logic is simple (set screen, load notifications)
- Adding a method would add indirection without significant benefit
- Current pattern keeps all navigation logic in one function

## Risks / Trade-offs

**Risk:** Code duplication across match arms
- **Mitigation:** The Inbox handling code is identical in each arm, but the duplication is minimal (5-7 lines). Future refactoring could extract this if needed.

**Risk:** Breaking existing navigation flow
- **Mitigation:** The change only adds new behavior for Inbox selection. Existing Workspace/Space/Folder/List navigation remains unchanged.

**Risk:** Workspace ID not set when navigating to Inbox from Workspaces screen
- **Mitigation:** The existing Inbox loading code already handles the case where `current_workspace_id` is None, showing an appropriate message to the user.

## Open Questions

None - this is a straightforward bug fix with clear implementation path.
