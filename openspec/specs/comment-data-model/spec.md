## ADDED Requirements

### Requirement: Comment data model structure
The system SHALL define a Comment struct that represents a ClickUp comment with all necessary fields for display and manipulation. The struct SHALL support serialization and deserialization to/from JSON for API communication.

#### Scenario: Comment struct defined with all fields
- **WHEN** Comment struct is defined
- **THEN** it includes all fields returned by the ClickUp API:
  - `id`: String (comment identifier)
  - `text`: String (comment content, from `comment_text` in API)
  - `text_preview`: String (preview text, from `text_preview` in API)
  - `commenter`: Option<User> (author of the comment, from `user` in API)
  - `created_at`: Option<i64> (creation timestamp in milliseconds, from `date` in API)
  - `updated_at`: Option<i64> (last update timestamp, from `date_updated` in API)
  - `assigned_commenter`: Option<User> (assigned user, from `assignee` in API)
  - `assigned_by`: Option<User> (who assigned the comment, from `assigned_by` in API)
  - `assigned`: bool (resolved status, from `resolved` in API)
  - `reaction`: String (reaction emoji/text, from `reaction` in API)
  - `parent_id`: Option<String> (parent comment ID for replies)

#### Scenario: Comment struct derives Serialize and Deserialize
- **WHEN** Comment struct is defined
- **THEN** it derives `Serialize` and `Deserialize` traits from serde
- **AND** it derives `Debug` and `Clone` traits for debugging and copying

#### Scenario: Comment struct uses correct field renames
- **WHEN** Comment struct fields map to API field names
- **THEN** serde rename attributes are used where field names differ:
  - `text` maps from `comment_text`
  - `text_preview` maps from `text_preview`
  - `commenter` maps from `user`
  - `created_at` maps from `date`
  - `updated_at` maps from `date_updated`
  - `assigned_commenter` maps from `assignee`
  - `assigned` maps from `resolved`
  - `parent_id` maps from `parent_id`

### Requirement: User data model structure
The system SHALL define a User struct that represents a ClickUp user in comment context. The struct SHALL support optional fields to handle varying API responses.

#### Scenario: User struct defined with all fields
- **WHEN** User struct is defined
- **THEN** it includes all fields returned by the ClickUp API:
  - `id`: i64 (user identifier)
  - `username`: String (display name)
  - `color`: Option<String> (user's color preference)
  - `email`: Option<String> (email address)
  - `profile_picture`: Option<String> (profile picture URL, from `profilePicture` in API)
  - `initials`: Option<String> (user initials)

#### Scenario: User struct uses correct field renames
- **WHEN** User struct fields map to API field names
- **THEN** serde rename attributes are used where field names differ:
  - `profile_picture` maps from `profilePicture`

### Requirement: CreateCommentRequest structure
The system SHALL define a CreateCommentRequest struct for creating new comments via the API. The struct SHALL include only writable fields and support optional parameters.

#### Scenario: CreateCommentRequest defined with required fields
- **WHEN** CreateCommentRequest struct is defined
- **THEN** it includes:
  - `comment_text`: String (required, the comment content)
  - `assignee`: Option<i64> (optional, user ID to assign)
  - `assigned_commenter`: Option<i64> (optional, user ID who assigned)
  - `parent_id`: Option<String> (optional, for threaded replies)

#### Scenario: CreateCommentRequest skips null fields
- **WHEN** CreateCommentRequest is serialized to JSON
- **THEN** optional fields with None values are omitted from the JSON
- **AND** the `skip_serializing_if = "Option::is_none"` attribute is used

### Requirement: UpdateCommentRequest structure
The system SHALL define an UpdateCommentRequest struct for updating existing comments via the API. All fields SHALL be optional to support partial updates.

#### Scenario: UpdateCommentRequest defined with optional fields
- **WHEN** UpdateCommentRequest struct is defined
- **THEN** it includes:
  - `comment_text`: Option<String> (new comment text)
  - `assigned`: Option<bool> (resolved status)
  - `assignee`: Option<i64> (assigned user ID)
  - `assigned_commenter`: Option<i64> (who assigned)

#### Scenario: UpdateCommentRequest skips null fields
- **WHEN** UpdateCommentRequest is serialized to JSON
- **THEN** optional fields with None values are omitted from the JSON

### Requirement: CommentsResponse structure
The system SHALL define a CommentsResponse struct for parsing API responses containing multiple comments.

#### Scenario: CommentsResponse wraps comment array
- **WHEN** CommentsResponse struct is defined
- **THEN** it includes:
  - `comments`: Vec<Comment> (array of comments, from `comments` in API)
- **AND** the field uses serde rename to map from `comments`

### Requirement: Comment field defaults for null values
The system SHALL handle null values in comment fields gracefully using custom deserializers. Each field type SHALL have appropriate default behavior.

#### Scenario: String fields default to empty string
- **WHEN** a string field receives null from API
- **THEN** it deserializes as empty string ""
- **AND** the `null_to_empty_string` deserializer is used

#### Scenario: Boolean fields default to false
- **WHEN** a boolean field receives null from API
- **THEN** it deserializes as false
- **AND** the `null_to_false` deserializer is used

#### Scenario: Integer ID fields default to 0
- **WHEN** a user ID field receives null from API
- **THEN** it deserializes as 0
- **AND** the `null_to_default_id` deserializer is used

#### Scenario: Optional fields accept null
- **WHEN** an Option<T> field receives null from API
- **THEN** it deserializes as None
- **AND** no custom deserializer is needed (serde default behavior)

### Requirement: Comment timestamp flexibility
The system SHALL handle timestamp fields that may be integers or strings. The ClickUp API may return timestamps in either format.

#### Scenario: Timestamp deserializer accepts integer
- **WHEN** a timestamp field receives an integer (i64) from API
- **THEN** it deserializes as Some(i64)

#### Scenario: Timestamp deserializer accepts string
- **WHEN** a timestamp field receives a string from API
- **THEN** the string is parsed as i64
- **AND** it deserializes as Some(i64) if parsing succeeds

#### Scenario: Timestamp deserializer handles null
- **WHEN** a timestamp field receives null from API
- **THEN** it deserializes as None

#### Scenario: Timestamp deserializer handles missing field
- **WHEN** a timestamp field is omitted from API response
- **THEN** it defaults to None

### Requirement: Comment parent_id for threading
The system SHALL support the parent_id field for threaded comments. The field SHALL be optional to distinguish top-level comments from replies.

#### Scenario: Reply comment has parent_id
- **WHEN** a comment is a reply to another comment
- **THEN** parent_id contains the parent comment's ID
- **AND** the comment is identified as a reply in the UI

#### Scenario: Top-level comment has no parent_id
- **WHEN** a comment is not a reply
- **THEN** parent_id is None
- **AND** the comment is displayed in top-level view

#### Scenario: Parent_id serialization skipped when None
- **WHEN** CreateCommentRequest is serialized with parent_id = None
- **THEN** the parent_id field is omitted from the JSON
- **AND** the API receives no parent_id field
