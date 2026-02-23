## Context

The current authentication flow in ClickDown requires users to manually obtain their ClickUp API token from the ClickUp web interface and paste it into the application. This creates several friction points:

1. Users must navigate away from the app to find their token
2. The token format (`pk_xxx`) is not intuitive
3. No clear error messages when token is invalid
4. Feels more like a developer tool than a polished desktop application

The existing architecture has:
- `AuthManager` in `src/api/auth.rs` - handles token storage/retrieval from config directory
- `auth_view.rs` - renders the token input screen
- `app.rs` - manages authentication state via `Message::TokenEntered`, `Message::AuthSuccess`, `Message::AuthError`

ClickUp's API supports token exchange via OAuth or direct credential authentication, which this change will leverage.

## Goals / Non-Goals

**Goals:**
- Replace token input screen with username/password login form
- Implement credential-to-token exchange flow
- Maintain existing token storage mechanism (no changes to where/how tokens are stored)
- Provide clear error messages for authentication failures
- Keep credentials in memory only (never persist username/password)

**Non-Goals:**
- OAuth flow with browser redirect (out of scope for this change)
- "Remember me" checkbox or session management beyond current token persistence
- Password reset flow (handled by ClickUp web interface)
- Multi-factor authentication support (future enhancement)
- Changing the token storage format or location

## Decisions

### 1. Credential Exchange Approach

**Decision:** Use ClickUp's credential authentication endpoint to exchange username/password for a token, then store the token using existing `AuthManager`.

**Rationale:**
- Minimal changes to existing token storage infrastructure
- Credentials are transient (in memory only)
- Aligns with standard desktop app authentication patterns
- Simpler than implementing full OAuth flow

**Alternatives Considered:**
- **OAuth with browser redirect**: More secure but requires redirect URI setup and browser handling complexity
- **Continue requiring manual token entry**: Rejects the goal of improving UX

### 2. UI Component Structure

**Decision:** Create new `LoginView` state and component parallel to existing `auth_view.rs`, replacing token input with username/password fields.

**Rationale:**
- Clear separation of concerns
- Easy to revert if needed
- Follows existing Elm architecture pattern in `app.rs`

**Alternatives Considered:**
- **Modify existing auth_view.rs**: Would require conditional rendering logic based on auth mode
- **Separate screen with navigation**: Adds unnecessary steps; login should be the first screen

### 3. Error Handling Strategy

**Decision:** Map HTTP authentication errors to user-friendly messages in the UI via existing `Message::AuthError` flow.

**Rationale:**
- Leverages existing error handling infrastructure
- Consistent with current app architecture
- No new error types needed in `app.rs`

**Alternatives Considered:**
- **New `Message::CredentialError`**: Adds complexity without benefit
- **Popup dialogs**: Breaks flow; inline errors are clearer

## Risks / Trade-offs

**[Security - Credential Transmission]** → Mitigation: Use HTTPS only (ClickUp API requires TLS), credentials never logged or persisted

**[Password Changes]** → Users changing ClickUp password will need to re-authenticate (same as token revocation currently)

**[API Compatibility]** → ClickUp could deprecate credential auth endpoint (mitigation: monitor API changelog, fallback to token entry if needed)

**[No Token Entry Fallback]** → Power users who prefer token entry lose that option (mitigation: could add hidden/dev mode later)

**[Testing Complexity]** → Credential auth requires mock API responses (mitigation: `MockClickUpClient` already supports this pattern)

## Migration Plan

1. **Development**: Implement `LoginView` alongside existing `auth_view.rs`
2. **Testing**: Update mock client to support credential exchange responses
3. **Deployment**: Replace auth screen reference in `app.rs` view function
4. **Rollback**: Revert `app.rs` to use original `auth_view::view()` if issues arise

No database migrations or config changes required. Token storage remains unchanged.

## Open Questions

- Should we add a "Show password" toggle to the password input field?
- Do we need rate limiting on the login button to prevent accidental multiple submissions?
