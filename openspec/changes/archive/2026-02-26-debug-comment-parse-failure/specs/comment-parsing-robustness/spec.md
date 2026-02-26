## ADDED Requirements

### Requirement: Comment parsing error diagnostics
The system SHALL provide detailed diagnostic information when comment deserialization fails. This enables developers to identify the root cause of parsing failures without requiring network packet captures. Diagnostics SHALL include the raw API response, the specific field that failed, and the error details.

#### Scenario: Parse error logged with response preview
- **WHEN** a comment API response fails to deserialize
- **THEN** the error message includes a preview of the response body (first 200 characters)
- **AND** the error is logged via tracing at debug level with the full response
- **AND** the user sees a user-friendly error message in the status bar

#### Scenario: Verbose mode shows full response
- **WHEN** CLI debug mode is run with `--verbose` flag
- **AND** a comment operation fails to parse
- **THEN** the full API response body is logged to stderr
- **AND** the JSON parsing error location (line/column) is included if available

#### Scenario: Field-level error identification
- **WHEN** a specific field fails to deserialize (e.g., type mismatch)
- **THEN** the error message identifies the field name
- **AND** the expected type is mentioned
- **AND** the actual value type is mentioned if detectable

### Requirement: Graceful handling of unknown comment fields
The system SHALL gracefully handle unknown or unexpected fields in comment API responses. The deserializer SHALL ignore unknown fields rather than failing, following the principle of robustness (be liberal in what you accept).

#### Scenario: Unknown fields ignored
- **WHEN** the ClickUp API returns a comment with fields not in the Comment struct
- **THEN** deserialization succeeds
- **AND** unknown fields are silently ignored
- **AND** known fields are populated correctly

#### Scenario: New API version compatibility
- **WHEN** ClickUp adds new fields to comment responses in a future API version
- **THEN** existing code continues to work without modification
- **AND** new fields are ignored until the model is updated

### Requirement: Flexible timestamp deserialization
The system SHALL handle multiple timestamp formats in comment API responses. ClickUp API may return timestamps as integers (milliseconds since epoch) or as strings. The deserializer SHALL accept both formats.

#### Scenario: Timestamp as integer
- **WHEN** API returns `date: 1234567890000` (integer milliseconds)
- **THEN** the value is deserialized to `Option<i64>` correctly

#### Scenario: Timestamp as string
- **WHEN** API returns `date: "1234567890000"` (string)
- **THEN** the value is parsed as integer and deserialized to `Option<i64>` correctly

#### Scenario: Timestamp as null
- **WHEN** API returns `date: null`
- **THEN** the value is deserialized as `None`

#### Scenario: Missing timestamp field
- **WHEN** API response omits the `date` field entirely
- **THEN** the value defaults to `None` without error

### Requirement: Null-safe string deserialization
The system SHALL handle null values in string fields gracefully. String fields SHALL default to empty string when the API returns null or missing values.

#### Scenario: Null string field
- **WHEN** API returns `comment_text: null`
- **THEN** the field is deserialized as empty string `""`
- **AND** no error is raised

#### Scenario: Missing string field
- **WHEN** API response omits a string field
- **THEN** the field defaults to empty string `""`

#### Scenario: Empty string preserved
- **WHEN** API returns `comment_text: ""`
- **THEN** the empty string is preserved (not converted to null)

### Requirement: Null-safe boolean deserialization
The system SHALL handle null values in boolean fields. Boolean fields SHALL default to `false` when the API returns null or missing values.

#### Scenario: Null boolean field
- **WHEN** API returns `resolved: null`
- **THEN** the field is deserialized as `false`
- **AND** no error is raised

#### Scenario: Missing boolean field
- **WHEN** API response omits a boolean field
- **THEN** the field defaults to `false`

### Requirement: Comment reply parsing with parent_id
The system SHALL correctly parse comment replies that include a `parent_id` field. The `parent_id` SHALL be optional to support both top-level comments and replies.

#### Scenario: Reply with parent_id
- **WHEN** API returns a comment with `parent_id: "parent123"`
- **THEN** the `parent_id` field is populated as `Some("parent123")`
- **AND** the comment is identified as a reply

#### Scenario: Top-level comment without parent_id
- **WHEN** API returns a comment without `parent_id` field
- **THEN** the `parent_id` field is `None`
- **AND** the comment is identified as top-level

#### Scenario: Top-level comment with null parent_id
- **WHEN** API returns `parent_id: null`
- **THEN** the `parent_id` field is `None`
- **AND** the comment is identified as top-level

### Requirement: User object deserialization in comments
The system SHALL handle various forms of user object data in comment responses. User objects may appear as full objects, null, or be omitted entirely.

#### Scenario: Full user object
- **WHEN** API returns `user: {id: 123, username: "test", ...}`
- **THEN** all user fields are deserialized correctly
- **AND** null fields within the user object use appropriate defaults

#### Scenario: Null user object
- **WHEN** API returns `user: null`
- **THEN** the commenter field is `None`
- **AND** no error is raised

#### Scenario: Missing user object
- **WHEN** API response omits the `user` field
- **THEN** the commenter field is `None`

#### Scenario: User with null ID
- **WHEN** API returns `user: {id: null, username: "test"}`
- **THEN** the user ID defaults to `0`
- **AND** other fields are deserialized normally

### Requirement: Reactions array handling
The system SHALL handle the `reactions` field in comment API responses. The field may be an array, null, or omitted.

#### Scenario: Reactions as empty array
- **WHEN** API returns `reactions: []`
- **THEN** the field is handled without error
- **AND** the reaction field (if present) is set appropriately

#### Scenario: Reactions as null
- **WHEN** API returns `reactions: null`
- **THEN** the field is handled without error

#### Scenario: Reactions omitted
- **WHEN** API response omits the `reactions` field
- **THEN** no error is raised
