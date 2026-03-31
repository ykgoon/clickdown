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

    // ==================== Notifications ====================

    /// Cache notifications for a workspace
    ///
    /// Stores notifications in the notifications table with the current timestamp.
    /// Existing notifications for the workspace are deleted before inserting new ones.
    #[allow(dead_code)]
    pub fn cache_notifications(
        &mut self,
        workspace_id: &str,
        notifications: &[Notification],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Delete existing notifications for this workspace
        tx.execute(
            "DELETE FROM notifications WHERE workspace_id = ?1",
            [workspace_id],
        )?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        for notif in notifications {
            tx.execute(
                "INSERT INTO notifications (id, workspace_id, title, description, created_at, read_at, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    notif.id,
                    workspace_id,
                    notif.title,
                    notif.description,
                    notif.created_at,
                    notif.read_at,
                    now,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get unread notifications for a workspace, ordered oldest-first
    pub fn get_unread_notifications(
        &self,
        workspace_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<Notification>> {
        if let Some(l) = limit {
            let query = "SELECT id, title, description, created_at, read_at FROM notifications
             WHERE workspace_id = ?1 AND read_at IS NULL
             ORDER BY created_at ASC
             LIMIT ?2";
            let mut stmt = self.conn.prepare(query)?;
            let notifications = stmt.query_map((workspace_id, l), |row| {
                Ok(Notification {
                    id: row.get(0)?,
                    workspace_id: workspace_id.to_string(),
                    title: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    read_at: row.get(4)?,
                })
            })?;
            notifications
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to read notifications")
        } else {
            let query = "SELECT id, title, description, created_at, read_at FROM notifications
             WHERE workspace_id = ?1 AND read_at IS NULL
             ORDER BY created_at ASC";
            let mut stmt = self.conn.prepare(query)?;
            let notifications = stmt.query_map((workspace_id,), |row| {
                Ok(Notification {
                    id: row.get(0)?,
                    workspace_id: workspace_id.to_string(),
                    title: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    read_at: row.get(4)?,
                })
            })?;
            notifications
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to read notifications")
        }
    }

    /// Check if cached notifications are still valid (not older than TTL)
    pub fn is_notifications_cache_valid(&self, workspace_id: &str, ttl_secs: i64) -> Result<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = self
            .conn
            .prepare("SELECT MAX(fetched_at) FROM notifications WHERE workspace_id = ?1")?;
        let max_fetched: Option<i64> = stmt.query_row([workspace_id], |row| row.get(0))?;

        match max_fetched {
            Some(fetched_at) => Ok((now - fetched_at) < ttl_secs),
            None => Ok(false),
        }
    }

    /// Mark a notification as read
    #[allow(dead_code)]
    pub fn mark_notification_read(&mut self, notification_id: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "UPDATE notifications SET read_at = ?1 WHERE id = ?2",
            params![now, notification_id],
        )?;
        Ok(())
    }

    /// Mark all notifications as read for a workspace
    #[allow(dead_code)]
    pub fn mark_all_notifications_read(&mut self, workspace_id: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "UPDATE notifications SET read_at = ?1 WHERE workspace_id = ?2 AND read_at IS NULL",
            params![now, workspace_id],
        )?;
        Ok(())
    }

    // ==================== Inbox Activity (Smart Inbox) ====================

    /// Cache inbox activity for a workspace
    ///
    /// Stores activity items in the inbox_activity table with the current timestamp.
    /// Existing activity for the workspace is deleted before inserting new ones.
    pub fn cache_inbox_activity(
        &mut self,
        workspace_id: &str,
        activities: &[crate::models::InboxActivity],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Delete existing activity for this workspace
        tx.execute(
            "DELETE FROM inbox_activity WHERE workspace_id = ?1",
            [workspace_id],
        )?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        for activity in activities {
            let activity_type_str = match activity.activity_type {
                crate::models::ActivityType::Assignment => "assignment",
                crate::models::ActivityType::Comment => "comment",
                crate::models::ActivityType::StatusChange => "status_change",
                crate::models::ActivityType::DueDate => "due_date",
            };

            tx.execute(
                "INSERT INTO inbox_activity (id, activity_type, title, description, timestamp, task_id, comment_id, workspace_id, task_name, previous_status, new_status, due_date, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    activity.id,
                    activity_type_str,
                    activity.title,
                    activity.description,
                    activity.timestamp,
                    activity.task_id,
                    activity.comment_id,
                    workspace_id,
                    activity.task_name,
                    activity.previous_status,
                    activity.new_status,
                    activity.due_date,
                    now,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get cached inbox activity for a workspace, ordered by timestamp DESC (newest first)
    pub fn get_cached_inbox_activity(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<crate::models::InboxActivity>> {
        let query = "SELECT id, activity_type, title, description, timestamp, task_id, comment_id, workspace_id, task_name, previous_status, new_status, due_date
                     FROM inbox_activity
                     WHERE workspace_id = ?1
                     ORDER BY timestamp DESC";
        
        let mut stmt = self.conn.prepare(query)?;
        let activities = stmt.query_map((workspace_id,), |row| {
            let activity_type_str: String = row.get(1)?;
            let activity_type = match activity_type_str.as_str() {
                "assignment" => crate::models::ActivityType::Assignment,
                "comment" => crate::models::ActivityType::Comment,
                "status_change" => crate::models::ActivityType::StatusChange,
                "due_date" => crate::models::ActivityType::DueDate,
                _ => crate::models::ActivityType::Assignment, // Fallback
            };

            Ok(crate::models::InboxActivity {
                id: row.get(0)?,
                activity_type,
                title: row.get(2)?,
                description: row.get(3)?,
                timestamp: row.get(4)?,
                task_id: row.get(5)?,
                comment_id: row.get(6)?,
                workspace_id: row.get(7)?,
                task_name: row.get(8)?,
                previous_status: row.get(9)?,
                new_status: row.get(10)?,
                due_date: row.get(11)?,
            })
        })?;
        
        activities
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read inbox activity")
    }

    /// Store the last inbox check timestamp
    pub fn store_last_inbox_check(&mut self, workspace_id: &str, timestamp: i64) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO inbox_metadata (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![
                format!("last_inbox_check_{}", workspace_id),
                timestamp.to_string(),
                now
            ],
        )?;
        Ok(())
    }

    /// Get the last inbox check timestamp
    #[allow(dead_code)]
    pub fn get_last_inbox_check(&self, workspace_id: &str) -> Result<Option<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM inbox_metadata WHERE key = ?1")?;

        let result: Result<String, _> =
            stmt.query_row([format!("last_inbox_check_{}", workspace_id)], |row| row.get(0));

        match result {
            Ok(value) => Ok(value.parse::<i64>().ok()),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to read last inbox check"),
        }
    }

    /// Cleanup old inbox activity (older than retention_days)
    #[allow(dead_code)]
    pub fn cleanup_old_inbox_activity(&mut self, workspace_id: &str, retention_days: i64) -> Result<()> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - (retention_days * 24 * 60 * 60 * 1000);

        self.conn.execute(
            "DELETE FROM inbox_activity WHERE workspace_id = ?1 AND timestamp < ?2",
            params![workspace_id, cutoff],
        )?;
        Ok(())
    }

    /// Check if cached inbox activity is still valid (not older than TTL)
    pub fn is_inbox_activity_cache_valid(&self, workspace_id: &str, ttl_secs: i64) -> Result<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = self
            .conn
            .prepare("SELECT MAX(fetched_at) FROM inbox_activity WHERE workspace_id = ?1")?;
        let max_fetched: Option<i64> = stmt.query_row([workspace_id], |row| row.get(0))?;

        match max_fetched {
            Some(fetched_at) => Ok((now - fetched_at) < ttl_secs),
            None => Ok(false),
        }
    }

    // ==================== Assigned Tasks ====================

    /// Cache assigned tasks
    pub fn cache_assigned_tasks(&mut self, tasks: &[Task]) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Delete existing assigned tasks
        tx.execute("DELETE FROM assigned_tasks", [])?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        for task in tasks {
            let status = task.status.as_ref().map(|s| s.status.as_str());
            let priority = task.priority.as_ref().map(|p| p.priority.as_str());
            
            tx.execute(
                "INSERT INTO assigned_tasks (id, list_id, name, status, priority, due_date, created_at, updated_at, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    task.id,
                    task.list.as_ref().map(|l| l.id.to_string()).unwrap_or_default(),
                    task.name,
                    status,
                    priority,
                    task.due_date,
                    task.created_at,
                    task.updated_at,
                    now
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get cached assigned tasks
    pub fn get_assigned_tasks(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, list_id, name, status, priority, due_date, created_at, updated_at FROM assigned_tasks ORDER BY due_date ASC, name ASC",
        )?;

        let tasks = stmt.query_map([], |row| {
            let status: Option<String> = row.get(3)?;
            let priority: Option<String> = row.get(4)?;

            Ok(Task {
                id: row.get(0)?,
                custom_id: None,
                custom_item_id: None,
                name: row.get(2)?,
                text_content: None,
                description: None,
                markdown_description: None,
                status: status.map(|s| TaskStatus {
                    id: None,
                    status: s,
                    color: None,
                    type_field: None,
                    orderindex: None,
                    status_group: None,
                }),
                orderindex: None,
                content: None,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                closed_at: None,
                done_at: None,
                archived: None,
                creator: None,
                assignees: vec![],
                group_assignees: vec![],
                watchers: vec![],
                checklists: vec![],
                tags: vec![],
                parent: None,
                top_level_parent: None,
                priority: priority.map(|p| crate::models::Priority {
                    priority: p,
                    color: None,
                }),
                due_date: row.get(5)?,
                start_date: None,
                points: None,
                custom_fields: vec![],
                attachments: vec![],
                dependencies: vec![],
                linked_tasks: vec![],
                locations: vec![],
                list: Some(crate::models::ListReference {
                    id: row.get(1)?,
                    name: None,
                    access: None,
                }),
                folder: None,
                space: None,
                project: None,
                url: None,
                team_id: None,
                sharing: None,
                permission_level: None,
                time_estimate: None,
                time_spent: None,
            })
        })?;

        tasks
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read assigned tasks")
    }

    /// Check if cached assigned tasks are still valid (not older than TTL)
    pub fn is_assigned_tasks_cache_valid(&self, ttl_secs: i64) -> Result<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = self
            .conn
            .prepare("SELECT MAX(fetched_at) FROM assigned_tasks")?;
        let max_fetched: Option<i64> = stmt.query_row([], |row| row.get(0))?;

        match max_fetched {
            Some(fetched_at) => Ok((now - fetched_at) < ttl_secs),
            None => Ok(false),
        }
    }

    /// Clear cached assigned tasks
    pub fn clear_assigned_tasks(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM assigned_tasks", [])?;
        Ok(())
    }

    // ==================== Assigned Comments ====================

    /// Cache assigned comments
    ///
    /// Stores assigned comments in the assigned_comments table with the current timestamp.
    /// Existing assigned comments are deleted before inserting new ones.
    pub fn cache_assigned_comments(
        &mut self,
        comments: &[crate::models::AssignedComment],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // Delete existing assigned comments
        tx.execute("DELETE FROM assigned_comments", [])?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        for ac in comments {
            let commenter_id = ac.comment.commenter.as_ref().map(|c| c.id);
            let commenter_name = ac.comment.commenter.as_ref().map(|c| c.username.clone());
            let assigned_commenter_id = ac.comment.assigned_commenter.as_ref().map(|c| c.id);
            let assigned_commenter_name =
                ac.comment.assigned_commenter.as_ref().map(|c| c.username.clone());

            tx.execute(
                "INSERT INTO assigned_comments (comment_id, task_id, task_name, text, commenter_id, commenter_name, assigned_commenter_id, assigned_commenter_name, created_at, updated_at, assigned_at, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    ac.comment.id,
                    ac.task.id,
                    ac.task.name,
                    ac.comment.text,
                    commenter_id,
                    commenter_name,
                    assigned_commenter_id,
                    assigned_commenter_name,
                    ac.comment.created_at,
                    ac.comment.updated_at,
                    ac.assigned_at,
                    now
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get cached assigned comments
    ///
    /// Retrieves assigned comments from the assigned_comments table, ordered by updated_at DESC.
    /// Returns an empty vector if no comments are cached.
    pub fn get_assigned_comments(&self) -> Result<Vec<crate::models::AssignedComment>> {
        use crate::models::{AssignedComment, Comment, TaskReference, User};

        let mut stmt = self.conn.prepare(
            "SELECT comment_id, task_id, task_name, text, commenter_id, commenter_name, assigned_commenter_id, assigned_commenter_name, created_at, updated_at, assigned_at FROM assigned_comments ORDER BY updated_at DESC",
        )?;

        let comments = stmt.query_map([], |row| {
            let task_id: String = row.get(1)?;
            let task_name: Option<String> = row.get(2)?;
            let text: String = row.get(3)?;
            let commenter_id: Option<i64> = row.get(4)?;
            let commenter_name: Option<String> = row.get(5)?;
            let assigned_commenter_id: Option<i64> = row.get(6)?;
            let assigned_commenter_name: Option<String> = row.get(7)?;
            let created_at: Option<i64> = row.get(8)?;
            let updated_at: Option<i64> = row.get(9)?;
            let assigned_at: Option<i64> = row.get(10)?;

            let commenter = commenter_id.map(|id| User {
                id,
                username: commenter_name.unwrap_or_default(),
                color: None,
                email: None,
                profile_picture: None,
                initials: None,
            });

            let assigned_commenter = assigned_commenter_id.map(|id| User {
                id,
                username: assigned_commenter_name.unwrap_or_default(),
                color: None,
                email: None,
                profile_picture: None,
                initials: None,
            });

            Ok(AssignedComment {
                comment: Comment {
                    id: row.get(0)?,
                    text,
                    text_preview: String::new(),
                    commenter,
                    created_at,
                    updated_at,
                    assigned_commenter,
                    assigned_by: None,
                    assigned: false,
                    reaction: String::new(),
                    parent_id: None,
                },
                task: TaskReference {
                    id: task_id,
                    name: task_name,
                },
                assigned_at,
            })
        })?;

        let mut result = Vec::new();
        for comment_result in comments {
            if let Ok(comment) = comment_result {
                result.push(comment);
            }
        }

        Ok(result)
    }

    /// Check if the assigned comments cache is valid (not older than TTL)
    pub fn is_assigned_comments_cache_valid(&self, ttl_secs: i64) -> Result<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = self
            .conn
            .prepare("SELECT MAX(fetched_at) FROM assigned_comments")?;
        let max_fetched: Option<i64> = stmt.query_row([], |row| row.get(0))?;

        match max_fetched {
            Some(fetched_at) => Ok((now - fetched_at) <= ttl_secs),
            None => Ok(false), // No cached data
        }
    }

    /// Clear cached assigned comments
    pub fn clear_assigned_comments(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM assigned_comments", [])?;
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

    // ==================== Assigned Tasks Cache Tests ====================

    #[test]
    fn test_cache_and_get_assigned_tasks() {
        let mut cache = create_test_cache();

        let tasks = vec![
            Task {
                id: "assigned-1".to_string(),
                name: "Assigned Task 1".to_string(),
                status: Some(TaskStatus {
                    id: None,
                    status: "active".to_string(),
                    color: None,
                    type_field: None,
                    orderindex: None,
                    status_group: None,
                }),
                priority: None,
                due_date: None,
                created_at: Some(1704067200000),
                updated_at: Some(1704067200000),
                ..Default::default()
            },
            Task {
                id: "assigned-2".to_string(),
                name: "Assigned Task 2".to_string(),
                status: Some(TaskStatus {
                    id: None,
                    status: "complete".to_string(),
                    color: None,
                    type_field: None,
                    orderindex: None,
                    status_group: None,
                }),
                priority: None,
                due_date: None,
                created_at: Some(1704153600000),
                updated_at: Some(1704153600000),
                ..Default::default()
            },
        ];

        // Cache assigned tasks
        cache.cache_assigned_tasks(&tasks).unwrap();

        // Get assigned tasks
        let loaded = cache.get_assigned_tasks().unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "assigned-1");
        assert_eq!(loaded[1].id, "assigned-2");
    }

    #[test]
    fn test_assigned_tasks_cache_ttl() {
        let mut cache = create_test_cache();

        let tasks = vec![Task {
            id: "ttl-test-1".to_string(),
            name: "TTL Test Task".to_string(),
            ..Default::default()
        }];

        // Cache assigned tasks
        cache.cache_assigned_tasks(&tasks).unwrap();

        // Should be valid with 5 minute TTL
        assert!(cache.is_assigned_tasks_cache_valid(300).unwrap());

        // Should not be valid with 1 nanosecond TTL (already expired)
        assert!(!cache.is_assigned_tasks_cache_valid(0).unwrap());
    }

    #[test]
    fn test_clear_assigned_tasks() {
        let mut cache = create_test_cache();

        let tasks = vec![Task {
            id: "clear-test-1".to_string(),
            name: "Clear Test Task".to_string(),
            ..Default::default()
        }];

        // Cache assigned tasks
        cache.cache_assigned_tasks(&tasks).unwrap();

        // Verify cached
        let loaded = cache.get_assigned_tasks().unwrap();
        assert_eq!(loaded.len(), 1);

        // Clear assigned tasks
        cache.clear_assigned_tasks().unwrap();

        // Verify cleared
        let loaded = cache.get_assigned_tasks().unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_cache_assigned_tasks_overwrites() {
        let mut cache = create_test_cache();

        // Cache initial tasks
        let initial_tasks = vec![Task {
            id: "initial-1".to_string(),
            name: "Initial Task".to_string(),
            ..Default::default()
        }];
        cache.cache_assigned_tasks(&initial_tasks).unwrap();

        // Cache new tasks (should overwrite)
        let new_tasks = vec![
            Task {
                id: "new-1".to_string(),
                name: "New Task 1".to_string(),
                ..Default::default()
            },
            Task {
                id: "new-2".to_string(),
                name: "New Task 2".to_string(),
                ..Default::default()
            },
        ];
        cache.cache_assigned_tasks(&new_tasks).unwrap();

        // Verify only new tasks remain
        let loaded = cache.get_assigned_tasks().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "new-1");
        assert_eq!(loaded[1].id, "new-2");
    }
}
