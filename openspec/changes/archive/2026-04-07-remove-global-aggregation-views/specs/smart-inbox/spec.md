## REMOVED Requirements

### Requirement: Smart inbox activity feed
**Reason**: The smart inbox activity aggregation feature is being removed entirely. The multi-endpoint polling approach (assignments, comments, status changes, due dates) was inefficient and produced incomplete results.

**Migration**: No direct replacement. Users view tasks and comments within individual list contexts using the per-list "Assigned to Me" filter.

### Requirement: Activity type identification
**Reason**: Activity type icons and categorization were specific to the inbox activity feed, which is being removed.

**Migration**: No replacement. Task status and priority indicators remain in the standard list view.

### Requirement: Activity data model
**Reason**: The `InboxActivity` model was designed for the smart inbox feature. It is no longer needed.

**Migration**: No replacement. The standard Task and Comment models continue to be used for list display.

### Requirement: Incremental polling
**Reason**: The `last_inbox_check` timestamp tracking and incremental activity fetching was part of the smart inbox mechanism, which is being removed.

**Migration**: No replacement. Per-list data fetching uses cache-first strategy with existing TTL-based invalidation.

### Requirement: Manual refresh mechanism
**Reason**: The inbox-specific refresh mechanism is removed along with the inbox view.

**Migration**: Use the standard list refresh mechanism ('r' key in list view).

### Requirement: Handle API errors gracefully
**Reason**: The inbox error handling for partial endpoint failures was specific to the multi-endpoint aggregation approach.

**Migration**: Standard list-level error handling applies. If a single API call fails, the error is displayed in the status bar.
