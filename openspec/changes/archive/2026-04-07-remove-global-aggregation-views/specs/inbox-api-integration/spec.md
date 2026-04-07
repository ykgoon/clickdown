## REMOVED Requirements

### Requirement: Notifications API endpoint
**Reason**: ClickUp API v2 does not have a notifications endpoint. The smart inbox approach (polling multiple endpoints) is also being removed.

**Migration**: No replacement. The `get_inbox_activity` method and its multi-endpoint aggregation logic are removed entirely.

### Requirement: Notification data model
**Reason**: The Notification model was designed for a non-existent API endpoint. The `InboxActivity` model that replaced it is also being removed.

**Migration**: No replacement. Standard Task and Comment models continue to be used.

### Requirement: Flexible notification deserialization
**Reason**: No longer desizing notification or activity responses.

**Migration**: No replacement.

### Requirement: Manual refresh mechanism
**Reason**: The inbox refresh mechanism for multi-endpoint activity fetching is removed.

**Migration**: Use standard list-level refresh ('r' key in list view).
