//! SQLite caching module

pub mod schema;

use crate::models::SessionState;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Cache manager for storing ClickUp data locally
pub struct CacheManager {
    conn: Connection,
}

impl CacheManager {
    /// Create a new CacheManager with a database at the given path
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create cache directory")?;
        }

        let conn = Connection::open(&db_path).context("Failed to open database")?;

        let manager = Self { conn };
        manager.init_schema()?;
        Ok(manager)
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(schema::INIT_SQL)?;
        Ok(())
    }

    // ==================== Session State ====================

    /// Save session state to the cache
    ///
    /// Persists the user's current navigation state as JSON in the session_state table.
    /// The state is stored with the key 'current_session'.
    pub fn save_session_state(&mut self, state: &SessionState) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Clear existing session state
        tx.execute("DELETE FROM session_state", [])?;

        // Serialize state to JSON
        let json = serde_json::to_string(state).context("Failed to serialize session state")?;

        // Insert the JSON value
        tx.execute(
            "INSERT INTO session_state (key, value) VALUES (?1, ?2)",
            params!["current_session", json],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Load session state from the cache
    ///
    /// Retrieves the user's last saved navigation state.
    /// Returns Ok(None) if no session state exists (first launch).
    pub fn load_session_state(&self) -> Result<Option<SessionState>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM session_state WHERE key = ?1")?;

        let result: Result<String, _> = stmt.query_row(["current_session"], |row| row.get(0));

        match result {
            Ok(json) => {
                let state: SessionState =
                    serde_json::from_str(&json).context("Failed to deserialize session state")?;
                Ok(Some(state))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to load session state"),
        }
    }

    /// Record that inbox entries have been seen (and optionally dismissed).
    ///
    /// The inbox API exposes no read state, so "inbox zero" lives here: an id
    /// recorded with `dismissed = false` still counts as read, while `true`
    /// hides it from the list entirely. Re-recording an id refreshes its
    /// timestamp, so a task that becomes active again can be marked unread by
    /// deleting the row.
    pub fn mark_inbox_read(
        &mut self,
        team_id: &str,
        task_ids: &[String],
        dismissed: bool,
    ) -> Result<()> {
        if task_ids.is_empty() {
            return Ok(());
        }
        let now = chrono::Utc::now().timestamp_millis();
        let tx = self.conn.transaction()?;
        for task_id in task_ids {
            tx.execute(
                "INSERT INTO inbox_read (team_id, task_id, read_at, dismissed)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(team_id, task_id) DO UPDATE SET
                     read_at = excluded.read_at,
                     dismissed = MAX(inbox_read.dismissed, excluded.dismissed)",
                params![team_id, task_id, now, dismissed as i64],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Load locally tracked inbox state for a workspace.
    ///
    /// Returns `(read_ids, dismissed_ids)` as separate sets because the two are
    /// used differently: read entries still appear (greyed) while dismissed
    /// ones are filtered out.
    pub fn load_inbox_state(
        &self,
        team_id: &str,
    ) -> Result<(
        std::collections::HashSet<String>,
        std::collections::HashSet<String>,
    )> {
        let mut stmt = self
            .conn
            .prepare("SELECT task_id, dismissed FROM inbox_read WHERE team_id = ?1")?;
        let rows = stmt.query_map([team_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? != 0))
        })?;

        let mut read = std::collections::HashSet::new();
        let mut dismissed = std::collections::HashSet::new();
        for row in rows {
            let (task_id, is_dismissed) = row?;
            if is_dismissed {
                dismissed.insert(task_id);
            } else {
                read.insert(task_id);
            }
        }
        Ok((read, dismissed))
    }

    /// Clear session state from the cache
    ///
    /// Removes all session state key-value pairs.
    /// Used when logging out or when restored state is invalid.
    pub fn clear_session_state(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM session_state", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a temporary in-memory database for testing
    fn create_test_cache() -> CacheManager {
        // Use :memory: for in-memory database
        let conn = Connection::open(":memory:").unwrap();
        let cache = CacheManager { conn };
        cache.init_schema().unwrap();
        cache
    }

    #[test]
    fn test_save_and_load_session_state_full() {
        let mut cache = create_test_cache();

        let state = SessionState {
            screen: String::from("Tasks"),
            workspace_id: Some("ws-123".to_string()),
            space_id: Some("space-456".to_string()),
            folder_id: Some("folder-789".to_string()),
            list_id: Some("list-abc".to_string()),
            task_id: Some("task-def".to_string()),
            document_id: None,
            user_id: None,
        };

        // Save session state
        cache.save_session_state(&state).unwrap();

        // Load session state
        let loaded = cache.load_session_state().unwrap().unwrap();

        assert_eq!(loaded.screen, state.screen);
        assert_eq!(loaded.workspace_id, state.workspace_id);
        assert_eq!(loaded.space_id, state.space_id);
        assert_eq!(loaded.folder_id, state.folder_id);
        assert_eq!(loaded.list_id, state.list_id);
        assert_eq!(loaded.task_id, state.task_id);
    }

    #[test]
    fn test_save_and_load_session_state_partial() {
        let mut cache = create_test_cache();

        // Only workspace ID (first launch after selecting workspace)
        let state = SessionState {
            screen: String::from("Spaces"),
            workspace_id: Some("ws-123".to_string()),
            ..Default::default()
        };

        cache.save_session_state(&state).unwrap();
        let loaded = cache.load_session_state().unwrap().unwrap();

        assert_eq!(loaded.workspace_id, Some("ws-123".to_string()));
        assert_eq!(loaded.space_id, None);
        assert_eq!(loaded.folder_id, None);
        assert_eq!(loaded.list_id, None);
        assert_eq!(loaded.task_id, None);
    }

    #[test]
    fn test_load_session_state_empty() {
        let cache = create_test_cache();

        // No session state saved
        let loaded = cache.load_session_state().unwrap();

        assert!(
            loaded.is_none(),
            "Should return None when no session state exists"
        );
    }

    #[test]
    fn test_save_session_state_overwrites() {
        let mut cache = create_test_cache();

        // Save initial state
        let state1 = SessionState {
            screen: String::from("Workspaces"),
            workspace_id: Some("ws-old".to_string()),
            ..Default::default()
        };
        cache.save_session_state(&state1).unwrap();

        // Save new state
        let state2 = SessionState {
            screen: String::from("Tasks"),
            workspace_id: Some("ws-new".to_string()),
            ..Default::default()
        };
        cache.save_session_state(&state2).unwrap();

        // Load should return new state
        let loaded = cache.load_session_state().unwrap().unwrap();
        assert_eq!(loaded.workspace_id, Some("ws-new".to_string()));
        assert_eq!(loaded.screen, String::from("Tasks"));
    }

    #[test]
    fn test_clear_session_state() {
        let mut cache = create_test_cache();

        // Save session state
        let state = SessionState {
            screen: String::from("Tasks"),
            workspace_id: Some("ws-123".to_string()),
            ..Default::default()
        };
        cache.save_session_state(&state).unwrap();

        // Verify it exists
        let loaded = cache.load_session_state().unwrap();
        assert!(loaded.is_some());

        // Clear session state
        cache.clear_session_state().unwrap();

        // Verify it's gone
        let loaded = cache.load_session_state().unwrap();
        assert!(loaded.is_none());
    }
}
