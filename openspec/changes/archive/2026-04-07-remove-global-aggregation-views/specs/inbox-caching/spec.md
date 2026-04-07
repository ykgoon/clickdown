## REMOVED Requirements

### Requirement: Notification storage schema
**Reason**: The `inbox_activity` and `inbox_metadata` cache tables were designed for the smart inbox feature, which is being removed.

**Migration**: These tables are deleted from the database schema. No data migration is needed — inbox activity was transient and did not contain unique data.

### Requirement: Cache notifications on fetch
**Reason**: Inbox activity caching is no longer performed.

**Migration**: No replacement. The `cache_inbox_activity`, `get_cached_inbox_activity`, `store_last_inbox_check`, `get_last_inbox_check`, and `cleanup_old_inbox_activity` methods are removed from the cache module.

### Requirement: Mark notification as read in cache
**Reason**: Notification read-state tracking was specific to the inbox feature.

**Migration**: No replacement needed.

### Requirement: Query unread notifications
**Reason**: Unread notification queries targeted the `inbox_activity` table, which is being removed.

**Migration**: No replacement needed.

### Requirement: Cache cleanup
**Reason**: Inbox activity cleanup was specific to the `inbox_activity` table.

**Migration**: No replacement needed.
