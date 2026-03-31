//! Database schema definitions

/// SQL to initialize the database schema
pub const INIT_SQL: &str = "
-- Workspaces table
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT
);

-- Spaces table
CREATE TABLE IF NOT EXISTS spaces (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT,
    private INTEGER DEFAULT 0,
    FOREIGN KEY (team_id) REFERENCES workspaces(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_spaces_team ON spaces(team_id);

-- Folders table
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    space_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT,
    private INTEGER DEFAULT 0,
    FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_folders_space ON folders(space_id);

-- Lists table
CREATE TABLE IF NOT EXISTS lists (
    id TEXT PRIMARY KEY,
    folder_id TEXT,
    space_id TEXT,
    name TEXT NOT NULL,
    archived INTEGER DEFAULT 0,
    hidden INTEGER DEFAULT 0,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE,
    FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_lists_folder ON lists(folder_id);
CREATE INDEX IF NOT EXISTS idx_lists_space ON lists(space_id);

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    list_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT,
    priority TEXT,
    due_date INTEGER,
    created_at INTEGER,
    updated_at INTEGER,
    FOREIGN KEY (list_id) REFERENCES lists(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks(list_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);

-- Task comments table
CREATE TABLE IF NOT EXISTS task_comments (
    comment_id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    text TEXT NOT NULL,
    commenter_id INTEGER,
    commenter_name TEXT,
    created_at INTEGER,
    updated_at INTEGER,
    fetched_at INTEGER NOT NULL,
    parent_id TEXT
);
CREATE INDEX IF NOT EXISTS idx_task_comments_task ON task_comments(task_id);
CREATE INDEX IF NOT EXISTS idx_task_comments_fetched ON task_comments(fetched_at);
CREATE INDEX IF NOT EXISTS idx_task_comments_parent ON task_comments(parent_id);

-- Session state table for persisting navigation state across sessions
CREATE TABLE IF NOT EXISTS session_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Notifications table
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    created_at INTEGER,
    read_at INTEGER,
    fetched_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notifications_workspace ON notifications(workspace_id);
CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read_at);
CREATE INDEX IF NOT EXISTS idx_notifications_fetched ON notifications(fetched_at);

-- Assigned tasks table for caching tasks assigned to current user
CREATE TABLE IF NOT EXISTS assigned_tasks (
    id TEXT PRIMARY KEY,
    list_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT,
    priority TEXT,
    due_date INTEGER,
    created_at INTEGER,
    updated_at INTEGER,
    fetched_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_assigned_tasks_fetched ON assigned_tasks(fetched_at);
CREATE INDEX IF NOT EXISTS idx_assigned_tasks_status ON assigned_tasks(status);
CREATE INDEX IF NOT EXISTS idx_assigned_tasks_due_date ON assigned_tasks(due_date);

-- Assigned comments table for caching comments assigned to current user
CREATE TABLE IF NOT EXISTS assigned_comments (
    comment_id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    task_name TEXT,
    text TEXT NOT NULL,
    commenter_id INTEGER,
    commenter_name TEXT,
    assigned_commenter_id INTEGER,
    assigned_commenter_name TEXT,
    created_at INTEGER,
    updated_at INTEGER,
    assigned_at INTEGER,
    fetched_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_assigned_comments_fetched ON assigned_comments(fetched_at);
CREATE INDEX IF NOT EXISTS idx_assigned_comments_task ON assigned_comments(task_id);

-- Inbox activity table for smart inbox feature
CREATE TABLE IF NOT EXISTS inbox_activity (
    id TEXT PRIMARY KEY,
    activity_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    timestamp INTEGER NOT NULL,
    task_id TEXT,
    comment_id TEXT,
    workspace_id TEXT NOT NULL,
    task_name TEXT,
    previous_status TEXT,
    new_status TEXT,
    due_date INTEGER,
    fetched_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_inbox_activity_workspace ON inbox_activity(workspace_id);
CREATE INDEX IF NOT EXISTS idx_inbox_activity_timestamp ON inbox_activity(timestamp);
CREATE INDEX IF NOT EXISTS idx_inbox_activity_type ON inbox_activity(activity_type);
CREATE INDEX IF NOT EXISTS idx_inbox_activity_fetched ON inbox_activity(fetched_at);

-- Key-value store for metadata (e.g., last_inbox_check timestamp)
CREATE TABLE IF NOT EXISTS inbox_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_inbox_metadata_key ON inbox_metadata(key);
";
