//! Database schema definitions for the single SQLite database.
//!
//! Tables: workspaces, chat_messages, knowledge_documents,
//!         knowledge_chunks, settings, audit_log

pub const INIT_SQL: &str = r#"
-- Workspaces
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    customer_name TEXT NOT NULL,
    technology_stack TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Chat messages
CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    tool_calls TEXT DEFAULT '[]'
);

-- Knowledge documents
CREATE TABLE IF NOT EXISTS knowledge_documents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    source TEXT NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    author TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Knowledge chunks (VSS indexed - VSS extension optional)
CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_chunks USING fts5(
    doc_id,
    content
);

-- Application settings (encrypted values stored as blobs)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_blob BLOB NOT NULL,
    description TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Audit log with hash chain
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    action TEXT NOT NULL,
    actor TEXT NOT NULL DEFAULT 'system',
    hash TEXT NOT NULL,
    signature TEXT
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_chat_messages_workspace ON chat_messages(workspace_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_created ON chat_messages(created_at);
CREATE INDEX IF NOT EXISTS idx_knowledge_docs_workspace ON knowledge_documents(workspace_id);
"#;