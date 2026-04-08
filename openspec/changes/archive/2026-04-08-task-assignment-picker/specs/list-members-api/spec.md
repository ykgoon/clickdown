## ADDED Requirements

### Requirement: List members endpoint
The system SHALL expose a `get_list_members(list_id: &str)` method on the `ClickUpApi` trait that returns a list of `User` objects representing all members who can access a given list. The implementation SHALL call `GET /list/{list_id}/member` on the ClickUp API v2.

#### Scenario: Successful member fetch
- **WHEN** `get_list_members` is called with a valid list ID
- **THEN** the system sends a GET request to `/api/v2/list/{list_id}/member`
- **AND** returns a `Vec<User>` containing all members from the response

#### Scenario: API error handling
- **WHEN** the ClickUp API returns an error (e.g., invalid list ID, auth failure)
- **THEN** the method returns an `Err` with a descriptive error message

### Requirement: Members response model
The system SHALL define a `MembersResponse` struct that deserializes the JSON response from `GET /list/{list_id}/member`. The response format is `{"members": [...]}` where each member object contains `id`, `username`, `email`, `color`, `initials`, and `profilePicture` fields. The system SHALL reuse the existing `User` model for individual member objects, as the response fields are compatible with `User`'s deserialization.

#### Scenario: Deserialize members response
- **WHEN** the API returns `{"members": [{"id": 123, "username": "Test", "email": "test@example.com", "color": "#ff0000", "initials": "T", "profilePicture": null}]}`
- **THEN** `MembersResponse` deserializes into a struct with `members: Vec<User>`
- **AND** each `User` has correct id, username, email, color, initials, and profile_picture values

#### Scenario: Handle null fields in member objects
- **WHEN** a member object has null values for optional fields (e.g., `"color": null, "email": null`)
- **THEN** deserialization succeeds
- **AND** those fields are set to `None` in the `User` struct

### Requirement: Mock client support for list members
The `MockClickUpClient` SHALL support configuring a `get_list_members` response via a `get_list_members_response` field. When configured, the mock SHALL return the configured response; when not configured, it SHALL return an error.

#### Scenario: Mock returns configured members
- **WHEN** `MockClickUpClient` is configured with `with_list_members_response(vec![user1, user2])`
- **AND** `get_list_members` is called
- **THEN** it returns the configured list of users

#### Scenario: Mock returns error when not configured
- **WHEN** `get_list_members` is called on an unconfigured mock
- **THEN** it returns an error indicating the method was not configured

### Requirement: API endpoint helper
The `ApiEndpoints` module SHALL provide a `list_members(list_id: &str)` function that returns the URL string `https://api.clickup.com/api/v2/list/{list_id}/member`.

#### Scenario: Generate endpoint URL
- **WHEN** `list_members("901608512233")` is called
- **THEN** it returns `"https://api.clickup.com/api/v2/list/901608512233/member"`
