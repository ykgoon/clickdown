# credential-auth Specification (Delta)

This delta spec modifies the existing `credential-auth` specification to fix the authentication parsing error by improving error handling and response parsing.

## MODIFIED Requirements

### Requirement: User can authenticate with username and password
The system SHALL allow users to log in using their username (email) and password, exchanging credentials for an API token with proper error handling for parsing failures.

#### Scenario: Successful authentication
- **WHEN** user enters valid username and password
- **AND** user clicks "Login" button
- **THEN** system exchanges credentials for an API token
- **AND** token response is parsed correctly regardless of response format variations
- **AND** token is stored securely for future sessions
- **AND** user is navigated to the main application view

#### Scenario: Invalid credentials
- **WHEN** user enters incorrect username or password
- **AND** user clicks "Login" button
- **THEN** system displays error message "Invalid username or password"
- **AND** user remains on the login screen
- **AND** password field is cleared for re-entry

#### Scenario: Network error during authentication
- **WHEN** network is unavailable during login attempt
- **THEN** system displays error message "Unable to connect to ClickUp. Please check your internet connection."
- **AND** user remains on the login screen

#### Scenario: Account locked or disabled
- **WHEN** user's ClickUp account is locked or disabled
- **THEN** system displays error message "Your account has been locked. Please contact ClickUp support."
- **AND** user remains on the login screen

#### Scenario: Response parsing error
- **WHEN** API returns an unexpected response format that cannot be parsed
- **THEN** system displays error message "Failed to process authentication response. Please try again."
- **AND** the raw HTTP response status and body are logged for debugging (not shown to user)
- **AND** user remains on the login screen

#### Scenario: Token field missing in response
- **WHEN** API response does not contain the expected "token" field
- **THEN** system displays error message "Authentication response missing token. Please contact support."
- **AND** the full response is logged for debugging
- **AND** user remains on the login screen

### Requirement: Credentials are handled securely
The system SHALL handle user credentials with appropriate security measures.

#### Scenario: Credentials in memory
- **WHEN** user enters username and password
- **THEN** credentials are stored only in memory during the authentication request
- **AND** credentials are cleared from memory after token exchange completes

#### Scenario: Credentials not logged
- **WHEN** authentication request is processed
- **THEN** username and password MUST NOT appear in application logs
- **AND** password MUST NOT appear in error messages

#### Scenario: Token storage unchanged
- **WHEN** authentication succeeds
- **THEN** the obtained token is stored using existing AuthManager mechanism
- **AND** token storage format and location remain unchanged

### Requirement: User can log out
The system SHALL provide a logout mechanism that clears the stored token and returns to the login screen.

#### Scenario: User initiates logout
- **WHEN** user clicks "Logout" button
- **THEN** stored token is cleared from disk
- **AND** user is navigated back to the login screen
- **AND** username and password fields are cleared
