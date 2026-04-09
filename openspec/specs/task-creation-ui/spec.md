## ADDED Requirements

### Requirement: Task creation form opens from task list
When the user presses `n` in the task list view, the system SHALL navigate to the TaskDetail screen in creation mode, displaying an empty task creation form with name and description input fields.

#### Scenario: Press 'n' in task list opens creation form
- **WHEN** user presses `n` while viewing a task list
- **THEN** the system navigates to the TaskDetail screen with `creating` mode enabled, showing the task creation form

#### Scenario: Task list hint text documents 'n' key
- **WHEN** the task list view is rendered
- **THEN** the hint bar displays `n: New` among the available keyboard shortcuts

### Requirement: Task creation form captures name and description
The task creation form SHALL accept text input for a required task name and an optional description. The name field SHALL be focused by default. Tab SHALL toggle focus between name and description fields.

#### Scenario: Typing enters text into focused field
- **WHEN** the user types characters while the creation form is active
- **THEN** the characters are appended to the currently focused input field (name or description)

#### Scenario: Backspace removes characters
- **WHEN** the user presses Backspace while the creation form is active
- **THEN** the last character is removed from the currently focused input field

#### Scenario: Tab toggles focus between name and description
- **WHEN** the user presses Tab while in the creation form
- **THEN** focus switches between the name and description fields, with visual indication of the focused field

### Requirement: Ctrl+S saves the task via API
When the user presses Ctrl+S in the creation form, the system SHALL validate the name field is non-empty, then call the `create_task()` API with the name and optional description. On success, the system SHALL reload the task list and return to the task list view. On failure, the system SHALL display an error message and keep the form open.

#### Scenario: Successful task creation
- **WHEN** user presses Ctrl+S with a non-empty name in the creation form
- **THEN** the system calls `create_task()` API, reloads the task list, returns to the task list view, and shows a success status message

#### Scenario: Creation fails with empty name
- **WHEN** user presses Ctrl+S with an empty or whitespace-only name
- **THEN** the system displays an error message ("Task name is required") and keeps the creation form open

#### Scenario: Creation fails with API error
- **WHEN** the `create_task()` API call returns an error
- **THEN** the system displays the error message and keeps the creation form open with the entered text preserved

### Requirement: Esc cancels task creation
When the user presses Esc in the creation form, the system SHALL cancel the creation, clear all input fields, and return to the task list view.

#### Scenario: Esc cancels and returns to task list
- **WHEN** user presses Esc while in the creation form
- **THEN** the system clears the name and description inputs, sets `creating` to false, and navigates back to the task list view

### Requirement: AppMessage::TaskCreated handles async result
The system SHALL define an `AppMessage::TaskCreated(Result<Task, String>)` variant. On receiving `Ok(task)`, the system SHALL reload the current task list and switch to `Screen::Tasks`. On receiving `Err(message)`, the system SHALL set the error display and keep the creation form open.

#### Scenario: TaskCreated with success reloads list
- **WHEN** `AppMessage::TaskCreated(Ok(task))` is received
- **THEN** the system reloads the current task list and switches to `Screen::Tasks`

#### Scenario: TaskCreated with error shows message
- **WHEN** `AppMessage::TaskCreated(Err(message))` is received
- **THEN** the system displays the error message and keeps the creation form open
