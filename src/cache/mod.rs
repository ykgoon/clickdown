//! SQLite caching module

pub mod schema;

#[allow(dead_code)]
use crate::models::{Comment, Notification, SessionState, Task, TaskStatus};
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

        // Migration: Add parent_id column to task_comments if it doesn't exist
        // SQLite doesn't support ADD COLUMN IF NOT EXISTS, so we catch the error
        let _ = self
            .conn
            .execute("ALTER TABLE task_comments ADD COLUMN parent_id TEXT", []);

        Ok(())
    }

    // ==================== Workspaces ====================

    // ==================== Spaces ====================

    // ==================== Folders ====================

    // ==================== Lists ====================

    // ==================== Tasks ====================

    // ==================== Comments ====================

    /// Cache comments for a task
    ///
    /// Stores comments in the task_comments table with the current timestamp.
    /// Existing comments for the task are deleted before inserting new ones.
    #[allow(dead_code)]
    pub fn cache_comments(&mut self, task_id: &str, comments: &[Comment]) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Delete existing comments for this task
        tx.execute("DELETE FROM task_comments WHERE task_id = ?1", [task_id])?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        for comment in comments {
            let commenter_id = comment.commenter.as_ref().map(|c| c.id);
            let commenter_name = comment.commenter.as_ref().map(|c| c.username.clone());

            tx.execute(
                "INSERT INTO task_comments (comment_id, task_id, text, commenter_id, commenter_name, created_at, updated_at, fetched_at, parent_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    comment.id,
                    task_id,
                    comment.text,
                    commenter_id,
                    commenter_name,
                    comment.created_at,
                    comment.updated_at,
                    now,
                    comment.parent_id,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get cached comments for a task
    ///
    /// Retrieves comments from the task_comments table, ordered by created_at DESC.
    /// Returns an empty vector if no comments are cached.
    #[allow(dead_code)]
    pub fn get_comments(&self, task_id: &str) -> Result<Vec<Comment>> {
        let mut stmt = self.conn.prepare(
            "SELECT comment_id, text, commenter_id, commenter_name, created_at, updated_at, parent_id FROM task_comments WHERE task_id = ?1 ORDER BY created_at DESC",
        )?;

        let comments = stmt.query_map((task_id,), |row| {
            let commenter_id: Option<i64> = row.get(2)?;
            let commenter_name: Option<String> = row.get(3)?;

            let commenter = match (commenter_id, commenter_name) {
                (Some(id), Some(name)) => Some(crate::models::User {
                    id,
                    username: name,
                    email: None,
                    color: None,
                    profile_picture: None,
                    initials: None,
                }),
                _ => None,
            };

            Ok(Comment {
                id: row.get(0)?,
                text: row.get(1)?,
                text_preview: String::new(),
                commenter,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                assigned_commenter: None,
                assigned_by: None,
                assigned: false,
                reaction: String::new(),
                parent_id: row.get(6)?,
            })
        })?;

        comments
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read comments")
    }

    /// Check if cached comments are still valid (not older than TTL)
    ///
    /// Returns true if the most recently fetched comment is within the TTL window.
    /// Returns false if no comments are cached or if the cache has expired.
    #[allow(dead_code)]
    pub fn is_cache_valid(&self, task_id: &str, ttl_secs: i64) -> Result<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = self
            .conn
            .prepare("SELECT MAX(fetched_at) FROM task_comments WHERE task_id = ?1")?;
        let max_fetched: Option<i64> = stmt.query_row([task_id], |row| row.get(0))?;

        match max_fetched {
            Some(fetched_at) => Ok((now - fetched_at) < ttl_secs),
            None => Ok(false),
        }
    }

    /// Clear cached comments for a task
    ///
    /// Removes all comments associated with the given task ID.
    #[allow(dead_code)]
    pub fn clear_comments(&mut self, task_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM task_comments WHERE task_id = ?1", [task_id])?;
        Ok(())
    }

    /// Clear all cached comments
    ///
    /// Removes all comments from the cache.
    #[allow(dead_code)]
    pub fn clear_all_comments(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM task_comments", [])?;
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

    /// Clear session state from the cache
    ///
    /// Removes all session state key-value pairs.
    /// Used when logging out or when restored state is invalid.
    #[allow(dead_code)]
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
