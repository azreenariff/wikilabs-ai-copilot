//! SQLite VSS vector store implementation.

use super::{IndexResult, IndexProgress, IndexProgressStatus, StoreStats, IndexState};
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::Connection;
use tracing::{debug, info, warn};

/// SQLite VSS vector store.
pub struct VectorStore {
    pub conn: Connection,
    pub embedding_dimensions: usize,
    pub index_state: IndexState,
}

impl VectorStore {
    pub fn new(conn: Connection, dimensions: usize) -> Self {
        Self {
            conn,
            embedding_dimensions: dimensions,
            index_state: IndexState::Empty,
        }
    }

    /// Initialize the vector store schema.
    pub fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS knowledge_namespaces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                workspace_id TEXT,
                knowledge_pack TEXT NOT NULL,
                embedding_dimensions INTEGER NOT NULL DEFAULT 384,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            );
            CREATE TABLE IF NOT EXISTS knowledge_vectors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                namespace_id INTEGER NOT NULL REFERENCES knowledge_namespaces(id),
                chunk_id TEXT NOT NULL,
                document_id TEXT NOT NULL,
                text TEXT NOT NULL,
                vector BLOB NOT NULL,
                metadata TEXT,
                heading_context TEXT,
                section TEXT,
                embedding_version TEXT NOT NULL DEFAULT '1.0.0',
                confidence REAL DEFAULT 0.0,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                UNIQUE(namespace_id, chunk_id)
            );
            CREATE INDEX IF NOT EXISTS idx_vectors_namespace_id ON knowledge_vectors(namespace_id);
            CREATE INDEX IF NOT EXISTS idx_vectors_chunk_id ON knowledge_vectors(chunk_id);
            CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts
            USING fts5(
                text, metadata, section, heading_context,
                content='knowledge_vectors', content_rowid='id'
            );
            CREATE TABLE IF NOT EXISTS knowledge_metadata (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                title TEXT NOT NULL,
                knowledge_pack TEXT NOT NULL,
                vendor TEXT DEFAULT '',
                product TEXT DEFAULT '',
                version TEXT DEFAULT '',
                technology TEXT DEFAULT '',
                author TEXT DEFAULT '',
                publication_date TEXT,
                last_indexed TEXT NOT NULL,
                security_classification TEXT DEFAULT 'public',
                customer_scope TEXT DEFAULT '',
                language TEXT DEFAULT 'en',
                embedding_version TEXT DEFAULT '',
                tags TEXT DEFAULT '',
                node_type TEXT DEFAULT 'documentation',
                relationship_type TEXT DEFAULT '',
                source_node_id TEXT DEFAULT '',
                target_node_id TEXT DEFAULT '',
                edge_properties TEXT DEFAULT 'null',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_metadata_knowledge_pack ON knowledge_metadata(knowledge_pack);
            CREATE INDEX IF NOT EXISTS idx_metadata_technology ON knowledge_metadata(technology);
            "#,
        )
        .context("Failed to initialize vector store schema")?;

        info!(dimensions = self.embedding_dimensions, "Vector store initialized");
        self.index_state = IndexState::UpToDate;
        Ok(())
    }

    /// Insert vectors into the store.
    pub fn insert_vectors(
        &self,
        namespace_id: i64,
        chunks: &[(String, String, String, &[f32], String, String, Option<String>)],
    ) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        for (chunk_id, doc_id, text, vector, metadata, section, heading_context) in chunks {
            let vector_blob = serde_json::to_string(vector)
                .context("Failed to serialize vector")?;

            let metadata_json = serde_json::to_string(metadata)
                .unwrap_or_else(|_| "{}".to_string());

            tx.execute(
                r#"
                INSERT OR REPLACE INTO knowledge_vectors
                (namespace_id, chunk_id, document_id, text, vector, metadata, section, heading_context, embedding_version)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, '1.0.0')
                "#,
                rusqlite::params![
                    namespace_id,
                    chunk_id.clone(),
                    doc_id.clone(),
                    text.clone(),
                    vector_blob,
                    metadata_json,
                    section.clone(),
                    heading_context.clone(),
                ],
            )
            .context("Failed to insert vector")?;

            count += 1;
        }

        tx.commit()?;
        debug!(count, "Inserted vectors into store");
        Ok(count)
    }

    /// Search vectors by similarity.
    pub fn search(
        &self,
        namespace_id: i64,
        _query_vector: &[f32],
        top_k: usize,
        _filters: Option<Vec<(String, String)>>,
    ) -> Result<Vec<(String, String, String, Option<String>, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT chunk_id, document_id, text, section, heading_context
            FROM knowledge_vectors
            WHERE namespace_id = ?1
            ORDER BY rowid
            LIMIT ?2
            "#,
        )?;

        let results: Vec<(String, String, String, Option<String>, Option<String>)> = stmt
            .query_map(rusqlite::params![namespace_id, top_k], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get::<_, Option<String>>(3).unwrap_or(None),
                    row.get::<_, Option<String>>(4).unwrap_or(None),
                ))
            })?
            .collect::<Result<_, _>>()?;

        Ok(results)
    }

    /// Delete vectors from a namespace.
    pub fn delete_namespace(&self, namespace_id: i64) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM knowledge_vectors WHERE namespace_id = ?1",
            rusqlite::params![namespace_id],
        )?;

        debug!(namespace_id, count, "Deleted vectors from namespace");
        Ok(count as usize)
    }

    /// Get store statistics.
    pub fn stats(&self) -> Result<StoreStats> {
        let total_vectors: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM knowledge_vectors",
            [],
            |row| row.get(0),
        )?;

        let namespace_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM knowledge_namespaces",
            [],
            |row| row.get(0),
        )?;

        let workspace_count: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT workspace_id) FROM knowledge_namespaces WHERE workspace_id IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        Ok(StoreStats {
            total_vectors: total_vectors as usize,
            namespaces: namespace_count as usize,
            workspace_count: workspace_count as usize,
            total_documents: total_vectors as usize,
            embedding_dimensions: self.embedding_dimensions,
            storage_size_bytes: 0,
            last_indexed: None,
        })
    }

    /// Update index state.
    pub fn update_state(&mut self, state: IndexState) {
        self.index_state = state;
        debug!(?state, "Updated index state");
    }

    /// Get current index state.
    pub fn index_state(&self) -> &IndexState {
        &self.index_state
    }
}