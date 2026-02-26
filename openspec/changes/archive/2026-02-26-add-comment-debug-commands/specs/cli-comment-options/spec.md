# cli-comment-options Specification

## Purpose
Define CLI debug mode support for comment creation options including parent_id, assignee, and assigned_commenter.

## Requirements

### Requirement: Parent ID Option

The system SHALL support specifying a parent comment ID for threaded comments.

#### Scenario: Create comment with parent ID
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --parent-id <parent_comment_id>`
- **THEN** the system SHALL create a reply to the parent comment
- **AND** the request SHALL include `parent_id` field set to the specified value
- **AND** the system SHALL print "Reply created: <comment_id>" to stdout

#### Scenario: Parent ID with create-reply command
- **WHEN** user executes `clickdown debug create-reply <comment_id> --text "text"`
- **THEN** the system SHALL use the comment_id as the parent_id implicitly
- **AND** the `--parent-id` option SHALL be ignored for create-reply command

### Requirement: Assignee Option

The system SHALL support assigning a user to a comment.

#### Scenario: Create comment with assignee
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --assignee <user_id>`
- **THEN** the request SHALL include `assignee` field set to the specified user ID
- **AND** the created comment SHALL have the assignee set

#### Scenario: Assignee option is optional
- **WHEN** user executes without `--assignee` option
- **THEN** the request SHALL NOT include the assignee field
- **AND** the comment SHALL be created without an assignee

### Requirement: Assigned Commenter Option

The system SHALL support specifying who assigned the comment.

#### Scenario: Create comment with assigned commenter
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --assigned-commenter <user_id>`
- **THEN** the request SHALL include `assigned_commenter` field set to the specified user ID

#### Scenario: Assigned commenter option is optional
- **WHEN** user executes without `--assigned-commenter` option
- **THEN** the request SHALL NOT include the assigned_commenter field

### Requirement: Combined Options

The system SHALL support combining multiple options in a single command.

#### Scenario: Create comment with all options
- **WHEN** user executes `clickdown debug create-comment <task_id> --text "text" --parent-id <parent_id> --assignee <user_id> --assigned-commenter <user_id>`
- **THEN** the request SHALL include all specified fields
- **AND** the comment SHALL be created with all options applied

#### Scenario: Options can be in any order
- **WHEN** user executes with options in different orders
- **THEN** the system SHALL parse all options correctly regardless of order
- **AND** the resulting request SHALL be identical

### Requirement: Option Validation

The system SHALL validate option values and provide clear error messages.

#### Scenario: Invalid user ID format
- **WHEN** user provides non-numeric value for `--assignee` or `--assigned-commenter`
- **THEN** the system SHALL print "Invalid user ID: must be a number" to stderr
- **AND** the system SHALL exit with code 2

#### Scenario: Option requires value
- **WHEN** user provides `--assignee` without a value
- **THEN** the system SHALL print "--assignee requires a value" to stderr
- **AND** the system SHALL exit with code 2
