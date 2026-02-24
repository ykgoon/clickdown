## Context

The codebase currently contains credential-based authentication (username/password) implementation that was added but ClickUp's API does not support password grant OAuth flow. The existing code includes:

- `Message` enum variants for credential auth (`UsernameEntered`, `PasswordEntered`, `LoginRequested`, etc.)
- `login_view.rs` UI component with username/password fields
- `authenticate_with_credentials()` method in `ClickUpClient` that attempts OAuth password grant (returns error with guidance to use token)
- `login` state in `ClickDown` app struct

However, the current active UI (`auth_view.rs`) already uses token-based authentication, and the `AuthManager` is designed for token storage. The credential auth code is dead code that needs to be removed to restore the clean token-only design.

**Constraints:**
- ClickUp API only supports Personal API Tokens (manual) and OAuth 2.0 (app registration + browser flow)
- Personal API Token is the simpler, appropriate choice for a single-user desktop client
- Must maintain backward compatibility for users who have existing tokens stored

## Goals / Non-Goals

**Goals:**
- Remove all username/password authentication code from the application
- Restore Personal API Token as the sole authentication mechanism
- Clean up dead code and unused message variants
- Update spec to reflect token-only authentication
- Ensure tests pass with token-based authentication

**Non-Goals:**
- Implement OAuth 2.0 flow (requires app registration, browser redirect - overkill for single-user client)
- Change token storage mechanism (file-based storage in `~/.config/clickdown/token` remains)
- Modify existing UI flow (auth_view.rs is already token-based and correct)

## Decisions

### Decision 1: Remove login_view.rs entirely
**Rationale:** The login view implements username/password authentication which ClickUp doesn't support. The auth_view.rs already provides the correct token-based authentication UI.

**Alternatives considered:**
- Keep login_view as fallback → Rejected: adds confusion and dead code
- Convert login_view to token input → Rejected: auth_view.rs already does this correctly

### Decision 2: Remove credential-related Message variants
**Rationale:** Messages like `UsernameEntered`, `PasswordEntered`, `LoginRequested`, `LoginSuccess`, `LoginError` are unused and will cause compiler warnings.

**Alternatives considered:**
- Keep for future OAuth 2.0 → Rejected: OAuth 2.0 requires different flow (browser redirect), not these messages

### Decision 3: Remove authenticate_with_credentials() from ClickUpApi trait
**Rationale:** This method attempts OAuth password grant which ClickUp doesn't support. Keeping it would mislead future developers.

**Alternatives considered:**
- Keep as deprecated → Rejected: no valid use case, removes ambiguity

### Decision 4: Update credential-auth spec to document token-only approach
**Rationale:** The spec currently describes username/password authentication. It should be converted to a delta spec that documents the revert to token-only auth.

## Risks / Trade-offs

**[Risk]** Users who expect username/password login will be confused
→ **Mitigation:** Clear UI help text directs users to "Settings → Apps → Generate Token"

**[Risk]** Breaking change for any documentation/tutorials mentioning username/password
→ **Mitigation:** Update README and AGENTS.md with correct authentication flow

**[Risk]** Future developers might re-add credential auth
→ **Mitigation:** Add clear comments in code explaining ClickUp doesn't support password grant

**[Trade-off]** Removing OAuth 2.0 support entirely
→ **Acceptable:** OAuth 2.0 is overkill for single-user desktop client; Personal Token is simpler and appropriate

## Migration Plan

1. Remove `login_view.rs` file
2. Remove `login: login_view::State` field from `ClickDown` struct
3. Remove credential-related `Message` variants from enum
4. Remove `authenticate_with_credentials()` from `ClickUpApi` trait and implementations
5. Remove credential auth message handlers from `app.rs` update() method
6. Update `Cargo.toml` if any dependencies were added for credential auth
7. Update tests to remove credential-based test cases
8. Update spec with delta document

**Rollback:** Revert commit - all changes are code removal, no data migration needed

## Open Questions

None - this is a straightforward code cleanup to remove unsupported authentication flow.
