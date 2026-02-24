## REMOVED Requirements

### Requirement: User can authenticate with username and password
**Reason**: ClickUp does not support username/password authentication for obtaining API tokens programmatically. Only Personal API Tokens (manual generation from web UI) and OAuth 2.0 (app registration + browser flow) are supported by ClickUp's API.

**Migration**: Users must obtain Personal API Token from ClickUp web UI (Settings → Apps → Generate Token) and enter it in the authentication screen.

#### Scenario: Successful authentication (REMOVED)
- **WHEN** user enters valid username and password
- **AND** user clicks "Login" button
- **THEN** system exchanges credentials for an API token
- **AND** token is stored securely for future sessions
- **AND** user is navigated to the main application view

#### Scenario: Invalid credentials (REMOVED)
- **WHEN** user enters incorrect username or password
- **AND** user clicks "Login" button
- **THEN** system displays error message "Invalid username or password"
- **AND** user remains on the login screen
- **AND** password field is cleared for re-entry

#### Scenario: Network error during authentication (REMOVED)
- **WHEN** network is unavailable during login attempt
- **THEN** system displays error message "Unable to connect to ClickUp. Please check your internet connection."
- **AND** user remains on the login screen

#### Scenario: Account locked or disabled (REMOVED)
- **WHEN** user's ClickUp account is locked or disabled
- **THEN** system displays error message "Your account has been locked. Please contact ClickUp support."
- **AND** user remains on the login screen

### Requirement: Credentials are handled securely
**Reason**: Since username/password authentication is not supported by ClickUp, credential handling is no longer applicable.

**Migration**: No migration needed - Personal API Token is stored using existing AuthManager mechanism.

#### Scenario: Credentials in memory (REMOVED)
- **WHEN** user enters username and password
- **THEN** credentials are stored only in memory during the authentication request
- **AND** credentials are cleared from memory after token exchange completes

#### Scenario: Credentials not logged (REMOVED)
- **WHEN** authentication request is processed
- **THEN** username and password MUST NOT appear in application logs
- **AND** password MUST NOT appear in error messages

#### Scenario: Token storage unchanged (REMOVED)
- **WHEN** authentication succeeds
- **THEN** the obtained token is stored using existing AuthManager mechanism
- **AND** token storage format and location remain unchanged

## ADDED Requirements

### Requirement: User can authenticate with Personal API Token
The system SHALL allow users to authenticate by entering their Personal API Token obtained from ClickUp web UI.

#### Scenario: Successful token authentication
- **WHEN** user enters valid Personal API Token
- **AND** user clicks "Connect" button
- **THEN** system validates token with ClickUp API
- **AND** token is stored securely for future sessions
- **AND** user is navigated to the main application view

#### Scenario: Invalid token
- **WHEN** user enters invalid or expired token
- **AND** user clicks "Connect" button
- **THEN** system displays error message "Invalid API token. Please check your token and try again."
- **AND** user remains on the authentication screen
- **AND** token field is cleared for re-entry

#### Scenario: Network error during token validation
- **WHEN** network is unavailable during token validation
- **THEN** system displays error message "Unable to connect to ClickUp. Please check your internet connection."
- **AND** user remains on the authentication screen

#### Scenario: Token guidance displayed
- **WHEN** user views the authentication screen
- **THEN** system displays help text "Get your token from ClickUp Settings → Apps → ClickUp API"
- **AND** system provides example token format placeholder

### Requirement: User can log out
The system SHALL provide a logout mechanism that clears the stored token and returns to the authentication screen.

#### Scenario: User initiates logout
- **WHEN** user clicks "Logout" button
- **THEN** stored token is cleared from disk
- **AND** user is navigated back to the authentication screen
- **AND** token input field is cleared
