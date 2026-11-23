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

-- Locally tracked inbox read state.
--
-- The inbox endpoint is GET-only and exposes no read/unread or dismiss state
-- (every dismiss route 404s), so inbox zero is maintained client-side: an
-- entry is remembered here once the user has seen or dismissed it. Rows are
-- keyed by task id and scoped by workspace.
CREATE TABLE IF NOT EXISTS inbox_read (
    team_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    read_at INTEGER NOT NULL,
    dismissed INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (team_id, task_id)
);

-- Session state table for persisting navigation state across sessions
CREATE TABLE IF NOT EXISTS session_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Migration: Drop tables removed in favor of per-list filtering.
-- Comment rows were never read back (comments are always refetched), so the
-- cache table and its indexes were dead weight.
DROP TABLE IF EXISTS task_comments;
DROP TABLE IF EXISTS assigned_tasks;
DROP TABLE IF EXISTS assigned_comments;
DROP TABLE IF EXISTS inbox_activity;
DROP TABLE IF EXISTS inbox_metadata;
-- Replaced by `inbox_read`: the old notifications endpoint 404s and no longer
-- feeds anything.
DROP TABLE IF EXISTS notifications;
";
