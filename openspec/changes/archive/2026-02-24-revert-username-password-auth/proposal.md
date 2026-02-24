## Why

The current implementation supports username/password authentication which deviates from ClickUp's intended authentication flow. ClickUp does not support username/password authentication for obtaining API tokens programmatically - only Personal API Tokens (manual generation) and OAuth 2.0 (app registration required) are supported. This change reverts to the original design using Personal API Token authentication.

## What Changes

- **Remove** username/password authentication flow from the application
- **Remove** credential exchange logic that attempted to convert credentials to tokens
- **Restore** Personal API Token as the sole authentication mechanism
- **Update** authentication UI to prompt for Personal API Token instead of username/password
- **Revert** `AuthManager` to token-only storage/retrieval
- **Update** error messages and user guidance for token-based authentication

## Capabilities

### New Capabilities
<!-- None - this is a revert to original behavior -->

### Modified Capabilities
<!-- Existing capabilities whose REQUIREMENTS are changing -->
- `credential-auth`: Revert from username/password credential exchange back to Personal API Token input. Remove all credential handling, exchange logic, and related error scenarios. Users must manually generate Personal API Token from ClickUp web UI (Settings → Apps → Generate Token).

## Impact

- **Code**: `src/api/auth.rs` - Remove credential exchange logic, restore token-only flow
- **Code**: `src/ui/auth_view.rs` - Update UI from username/password fields to single token input field
- **Code**: `src/api/client.rs` - Remove any credential-based authentication methods
- **Specs**: `openspec/specs/credential-auth/spec.md` - Needs delta spec to revert requirements
- **Tests**: Update test fixtures and integration tests to use token-based authentication
- **Breaking**: Users currently using username/password will need to generate Personal API Token from ClickUp web UI
