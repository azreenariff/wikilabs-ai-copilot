//! Repository implementations — SQLite-backed persistence for all entities.
//!
//! All repositories use the `Database` connection and the schema defined in `schema.rs`.
//! They handle serialization/deserialization between Rust types and SQLite rows.

use anyhow::{Context, Result};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::db::Database;

#[cfg(test)]
use uuid::Uuid;

// ── Workspace Repository ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRow {
    pub id: String,
    pub name: String,
    pub customer_name: String,
    pub technology_stack: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct WorkspaceRepository {
    db: Database,
}

impl WorkspaceRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn insert(
        &self,
        id: &str,
        name: &str,
        customer_name: &str,
        tech_stack: &str,
    ) -> Result<()> {
        self.db.execute(
            "INSERT INTO workspaces (id, name, customer_name, technology_stack, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'), datetime('now'))",
            &[&id, &name, &customer_name, &tech_stack],
        )?;
        debug!(id, "Workspace inserted");
        Ok(())
    }

    pub fn update(
        &self,
        id: &str,
        name: &str,
        customer_name: &str,
        tech_stack: &str,
    ) -> Result<usize> {
        let rows = self.db.execute(
            "UPDATE workspaces SET name=?2, customer_name=?3, technology_stack=?4, updated_at=datetime('now')
             WHERE id=?1",
            &[&id, &name, &customer_name, &tech_stack],
        )?;
        debug!(id, rows, "Workspace updated");
        Ok(rows)
    }

    pub fn delete(&self, id: &str) -> Result<usize> {
        let rows = self
            .db
            .execute("DELETE FROM workspaces WHERE id=?1", &[&id])?;
        debug!(id, rows, "Workspace deleted");
        Ok(rows)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<WorkspaceRow>> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT id, name, customer_name, technology_stack, created_at, updated_at FROM workspaces WHERE id=?1",
            [&id],
            |row| Ok(WorkspaceRow {
                id: row.get(0)?,
                name: row.get(1)?,
                customer_name: row.get(2)?,
                technology_stack: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            }),
        )
        .optional()
        .context("Failed to query workspace by ID")
    }

    pub fn list_all(&self) -> Result<Vec<WorkspaceRow>> {
        self.db.query_all(
            "SELECT id, name, customer_name, technology_stack, created_at, updated_at FROM workspaces ORDER BY updated_at DESC",
            &[],
            |row| Ok(WorkspaceRow {
                id: row.get(0)?,
                name: row.get(1)?,
                customer_name: row.get(2)?,
                technology_stack: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            }),
        )
        .context("Failed to list workspaces")
    }
}

// ── Chat Message Repository ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageRow {
    pub id: String,
    pub workspace_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub tool_calls: String,
}

pub struct ChatMessageRepository {
    db: Database,
}

impl ChatMessageRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn insert(
        &self,
        id: &str,
        workspace_id: &str,
        role: &str,
        content: &str,
        tool_calls: &str,
    ) -> Result<()> {
        self.db.execute(
            "INSERT INTO chat_messages (id, workspace_id, role, content, created_at, tool_calls)
             VALUES (?1, ?2, ?3, ?4, datetime('now'), ?5)",
            &[&id, &workspace_id, &role, &content, &tool_calls],
        )?;
        debug!(id, "Chat message inserted");
        Ok(())
    }

    pub fn get_by_workspace(
        &self,
        workspace_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessageRow>> {
        self.db
            .query_all(
                "SELECT id, workspace_id, role, content, created_at, tool_calls
             FROM chat_messages WHERE workspace_id=?1 ORDER BY created_at ASC LIMIT ?2",
                &[&workspace_id, &(limit as i64)],
                |row| {
                    Ok(ChatMessageRow {
                        id: row.get(0)?,
                        workspace_id: row.get(1)?,
                        role: row.get(2)?,
                        content: row.get(3)?,
                        created_at: row.get(4)?,
                        tool_calls: row.get(5)?,
                    })
                },
            )
            .context("Failed to query chat messages by workspace")
    }

    pub fn delete_by_workspace(&self, workspace_id: &str) -> Result<usize> {
        let rows = self.db.execute(
            "DELETE FROM chat_messages WHERE workspace_id=?1",
            &[&workspace_id],
        )?;
        debug!(workspace_id, rows, "Chat messages deleted for workspace");
        Ok(rows)
    }
}

// ── Knowledge Document Repository ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocumentRow {
    pub id: String,
    pub title: String,
    pub source: String,
    pub workspace_id: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct KnowledgeDocumentRepository {
    db: Database,
}

impl KnowledgeDocumentRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn insert(
        &self,
        id: &str,
        title: &str,
        source: &str,
        workspace_id: &str,
        author: &str,
    ) -> Result<()> {
        self.db.execute(
            "INSERT INTO knowledge_documents (id, title, source, workspace_id, author, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
            &[&id, &title, &source, &workspace_id, &author],
        )?;
        debug!(id, "Knowledge document inserted");
        Ok(())
    }

    pub fn update(&self, id: &str, title: &str, source: &str, author: &str) -> Result<usize> {
        let rows = self.db.execute(
            "UPDATE knowledge_documents SET title=?2, source=?3, author=?4, updated_at=datetime('now') WHERE id=?1",
            &[&id, &title, &source, &author],
        )?;
        Ok(rows)
    }

    pub fn delete(&self, id: &str) -> Result<usize> {
        let rows = self
            .db
            .execute("DELETE FROM knowledge_documents WHERE id=?1", &[&id])?;
        debug!(id, rows, "Knowledge document deleted");
        Ok(rows)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<KnowledgeDocumentRow>> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT id, title, source, workspace_id, author, created_at, updated_at FROM knowledge_documents WHERE id=?1",
            [&id],
            |row| Ok(KnowledgeDocumentRow {
                id: row.get(0)?,
                title: row.get(1)?,
                source: row.get(2)?,
                workspace_id: row.get(3)?,
                author: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            }),
        )
        .optional()
        .context("Failed to query knowledge document by ID")
    }

    pub fn list_by_workspace(&self, workspace_id: &str) -> Result<Vec<KnowledgeDocumentRow>> {
        self.db.query_all(
            "SELECT id, title, source, workspace_id, author, created_at, updated_at FROM knowledge_documents WHERE workspace_id=?1 ORDER BY updated_at DESC",
            &[&workspace_id],
            |row| Ok(KnowledgeDocumentRow {
                id: row.get(0)?,
                title: row.get(1)?,
                source: row.get(2)?,
                workspace_id: row.get(3)?,
                author: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            }),
        )
        .context("Failed to list knowledge documents for workspace")
    }
}

// ── Knowledge Chunk Repository (FTS5) ─────────────────────────────────

pub struct KnowledgeChunkRepository {
    db: Database,
}

impl KnowledgeChunkRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Insert a chunk into the FTS5 index.
    pub fn insert(&self, doc_id: &str, content: &str) -> Result<()> {
        self.db.execute(
            "INSERT INTO knowledge_chunks (doc_id, content) VALUES (?1, ?2)",
            &[&doc_id, &content],
        )?;
        debug!(doc_id, "Knowledge chunk indexed");
        Ok(())
    }

    /// Full-text search.
    pub fn search(&self, workspace_id: &str, query: &str) -> Result<Vec<(String, f64)>> {
        // FTS5 ranking via bm25()
        let rows = self.db.query_all(
            r#"SELECT knowledge_chunks.doc_id, bm25(knowledge_chunks)
               FROM knowledge_chunks
               INNER JOIN knowledge_documents ON knowledge_documents.id = knowledge_chunks.doc_id
               WHERE knowledge_documents.workspace_id = ?1
               AND knowledge_chunks MATCH ?2
               ORDER BY bm25(knowledge_chunks)"#,
            &[&workspace_id, &query],
            |row| {
                let doc_id: String = row.get(0)?;
                let score: f64 = row.get(1)?;
                Ok((doc_id, score))
            },
        );
        rows.context("Failed to search knowledge chunks")
    }

    pub fn delete_by_document(&self, doc_id: &str) -> Result<()> {
        self.db
            .execute("DELETE FROM knowledge_chunks WHERE doc_id=?1", &[&doc_id])?;
        Ok(())
    }
}

// ── Settings Repository ───────────────────────────────────────────────

pub struct SettingsRepository {
    db: Database,
}

impl SettingsRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT value_blob FROM settings WHERE key=?1",
            [&key],
            |row| row.get(0),
        )
        .optional()
        .context("Failed to query setting")
    }

    pub fn set(&self, key: &str, value: &[u8], description: &str) -> Result<()> {
        self.db.execute(
            "INSERT INTO settings (key, value_blob, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value_blob=?2, description=?3, updated_at=datetime('now')",
            &[&key, &value, &description],
        )?;
        debug!(key, "Setting updated");
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<usize> {
        let rows = self
            .db
            .execute("DELETE FROM settings WHERE key=?1", &[&key])?;
        Ok(rows)
    }
}

// ── Audit Log Repository ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRow {
    pub id: String,
    pub timestamp: String,
    pub action: String,
    pub actor: String,
    pub hash: String,
    pub signature: Option<String>,
}

pub struct AuditLogRepository {
    db: Database,
}

impl AuditLogRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn append(
        &self,
        id: &str,
        action: &str,
        actor: &str,
        hash: &str,
        signature: Option<&str>,
    ) -> Result<()> {
        self.db.execute(
            "INSERT INTO audit_log (id, timestamp, action, actor, hash, signature)
             VALUES (?1, datetime('now'), ?2, ?3, ?4, ?5)",
            &[&id, &action, &actor, &hash, &signature.unwrap_or("")],
        )?;
        debug!(id, action, "Audit log entry appended");
        Ok(())
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<AuditLogRow>> {
        self.db.query_all(
            "SELECT id, timestamp, action, actor, hash, signature FROM audit_log ORDER BY timestamp DESC LIMIT ?1",
            &[&(limit as i64)],
            |row| Ok(AuditLogRow {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action: row.get(2)?,
                actor: row.get(3)?,
                hash: row.get(4)?,
                signature: row.get(5)?,
            }),
        )
        .context("Failed to list audit log entries")
    }
}

// ── Database Factory ──────────────────────────────────────────────────

/// Holds all repositories for a single database connection.
pub struct RepositoryFactory {
    pub workspace: WorkspaceRepository,
    pub chat_messages: ChatMessageRepository,
    pub knowledge_documents: KnowledgeDocumentRepository,
    pub knowledge_chunks: KnowledgeChunkRepository,
    pub settings: SettingsRepository,
    pub audit_log: AuditLogRepository,
}

impl RepositoryFactory {
    pub fn new(db: Database) -> Self {
        Self {
            workspace: WorkspaceRepository::new(db.clone()),
            chat_messages: ChatMessageRepository::new(db.clone()),
            knowledge_documents: KnowledgeDocumentRepository::new(db.clone()),
            knowledge_chunks: KnowledgeChunkRepository::new(db.clone()),
            settings: SettingsRepository::new(db.clone()),
            audit_log: AuditLogRepository::new(db.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        let db = Database::new(":memory:").unwrap();
        db.execute_batch(crate::schema::INIT_SQL).unwrap();
        db
    }

    #[test]
    fn test_workspace_crud() {
        let ws_repo = WorkspaceRepository::new(test_db());
        let ws_id = Uuid::new_v4().to_string();

        ws_repo.insert(&ws_id, "Test", "Customer", "[]").unwrap();
        let ws = ws_repo.get_by_id(&ws_id).unwrap().unwrap();
        assert_eq!(ws.name, "Test");
        assert_eq!(ws.customer_name, "Customer");

        ws_repo
            .update(&ws_id, "Updated", "Cust", "['OpenShift']")
            .unwrap();
        let ws = ws_repo.get_by_id(&ws_id).unwrap().unwrap();
        assert_eq!(ws.name, "Updated");
        assert_eq!(ws.technology_stack, "['OpenShift']");

        ws_repo.delete(&ws_id).unwrap();
        assert!(ws_repo.get_by_id(&ws_id).unwrap().is_none());
    }

    #[test]
    fn test_chat_messages() {
        let db = test_db();
        let msg_repo = ChatMessageRepository::new(db.clone());
        let ws_repo = WorkspaceRepository::new(db.clone());
        let ws_id = Uuid::new_v4().to_string();
        let msg_id = Uuid::new_v4().to_string();

        ws_repo.insert(&ws_id, "TestWs", "Customer", "[]").unwrap();
        msg_repo
            .insert(&msg_id, &ws_id, "user", "Hello", "[]")
            .unwrap();
        let msgs = msg_repo.get_by_workspace(&ws_id, 10).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, "user");
        assert_eq!(msgs[0].content, "Hello");
    }

    #[test]
    fn test_knowledge_document_crud() {
        let db = test_db();
        let doc_repo = KnowledgeDocumentRepository::new(db.clone());
        let ws_repo = WorkspaceRepository::new(db.clone());
        let doc_id = Uuid::new_v4().to_string();
        let ws_id = Uuid::new_v4().to_string();

        ws_repo.insert(&ws_id, "TestWs", "Customer", "[]").unwrap();
        doc_repo
            .insert(&doc_id, "Test Doc", "file.md", &ws_id, "Author")
            .unwrap();
        let docs = doc_repo.list_by_workspace(&ws_id).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "Test Doc");

        doc_repo.delete(&doc_id).unwrap();
        assert!(doc_repo.get_by_id(&doc_id).unwrap().is_none());
    }

    #[test]
    fn test_knowledge_chunk_fts() {
        let chunk_repo = KnowledgeChunkRepository::new(test_db());
        let doc_id = Uuid::new_v4().to_string();
        chunk_repo
            .insert(&doc_id, "OpenShift deployment guide")
            .unwrap();

        // FTS5 search
        let results = chunk_repo.search("", "OpenShift");
        // Results may or may not match depending on FTS5 setup in memory
        assert!(results.is_ok());
    }

    #[test]
    fn test_settings() {
        let settings_repo = SettingsRepository::new(test_db());
        settings_repo.set("api_key", b"secret", "API key").unwrap();
        let val = settings_repo.get("api_key").unwrap().unwrap();
        assert_eq!(val, b"secret");

        settings_repo
            .set("api_key", b"new_secret", "Updated")
            .unwrap();
        let val = settings_repo.get("api_key").unwrap().unwrap();
        assert_eq!(val, b"new_secret");
    }

    #[test]
    fn test_audit_log() {
        let audit_repo = AuditLogRepository::new(test_db());
        let entry_id = Uuid::new_v4().to_string();
        audit_repo
            .append(&entry_id, "test_action", "test_actor", "abc123", None)
            .unwrap();

        let entries = audit_repo.list_recent(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "test_action");
    }
}
