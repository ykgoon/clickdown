## Why

The current authentication flow requires users to manually obtain and enter their ClickUp API token, which creates friction during onboarding. This change introduces a username/password login flow that automatically exchanges credentials for an API token, providing a smoother user experience similar to standard web applications.

## What Changes

- Replace the API token input screen with a username/password login form on the first screen
- Add a new authentication endpoint call to exchange username/password for an API token
- Automatically store the obtained token for future sessions (existing token storage remains unchanged)
- Add error handling for invalid credentials with user-friendly messages
- **BREAKING**: Direct API token entry is no longer available in the UI (users must authenticate via username/password)

## Capabilities

### New Capabilities

- `credential-auth`: Handles username/password authentication flow, token exchange, and credential validation

### Modified Capabilities

- `error-handling`: Add authentication-specific error cases (invalid credentials, account locked, etc.)

## Impact

- **UI**: Authentication view changes from token input to username/password form
- **API**: New credential exchange endpoint integration
- **Security**: Credentials handled in memory only (not stored), token storage remains unchanged
- **Dependencies**: None - uses existing ClickUp API authentication endpoints
