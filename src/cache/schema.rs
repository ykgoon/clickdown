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

-- Migration: Drop tables removed in favor of per-list filtering
DROP TABLE IF EXISTS assigned_tasks;
DROP TABLE IF EXISTS assigned_comments;
DROP TABLE IF EXISTS inbox_activity;
DROP TABLE IF EXISTS inbox_metadata;
DROP TABLE IF EXISTS notifications;
";
