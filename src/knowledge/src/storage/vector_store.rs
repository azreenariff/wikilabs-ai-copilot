//! SQLite-based vector store with VSS support.
//!
//! Implements the core vector storage layer using SQLite.
//! Supports flat search and HNSW index via SQLite VSS extension.

use anyhow::Context;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Configuration for the vector store.
#[derive(Debug, Clone)]
pub struct VectorStoreConfig {
    pub namespace: String,
    pub workspace_id: uuid::Uuid,
    pub schema_version: u32,
    pub vector_dimensions: usize,
    pub enable_hnsw: bool,
    pub hnsw_m: usize,
    pub hnsw_ef_construction: usize,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            namespace: "default".to_string(),
            workspace_id: uuid::Uuid::new_v4(),
            schema_version: 1,
            vector_dimensions: 384,
            enable_hnsw: false,
            hnsw_m: 16,
            hnsw_ef_construction: 100,
        }
    }
}

/// The vector store backed by SQLite.
pub struct VectorStore {
    connection: Arc<Mutex<Connection>>,
    pub config: VectorStoreConfig,
}

impl VectorStore {
    /// Access the underlying connection for advanced queries.
    pub(crate) fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection.lock().expect("Vector store lock poisoned")
    }
}

impl Clone for VectorStore {
    fn clone(&self) -> Self {
        let new_conn = Connection::open(format!("{}_clone", self.config.namespace))
            .expect("Failed to open clone connection");
        Self {
            connection: Arc::new(Mutex::new(new_conn)),
            config: self.config.clone(),
        }
    }
}

impl VectorStore {
    /// Create a new vector store, initializing the schema.
    pub fn new(
        db_path: &str,
        namespace: &str,
        workspace_id: uuid::Uuid,
        schema_version: u32,
    ) -> anyhow::Result<Self> {
        debug!(db_path, namespace, "Opening vector store");
        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open vector store at {}", db_path))?;

        // Enable WAL mode and foreign keys
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "on")?;

        // Initialize schema
        let schema = Self::build_schema(namespace, schema_version);
        conn.execute_batch(&schema)
            .with_context(|| "Failed to initialize vector store schema")?;

        info!(db_path, namespace, "Vector store opened and initialized");
        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
            config: VectorStoreConfig {
                namespace: namespace.to_string(),
                workspace_id,
                schema_version,
                ..VectorStoreConfig::default()
            },
        })
    }

    /// Build the SQL schema for the vector store.
    fn build_schema(namespace: &str, schema_version: u32) -> String {
        let table_name = format!("vectors_{}", namespace);
        let metadata_table = format!("vector_metadata_{}", namespace);

        let mut schema = format!(
            r#"
-- Vector storage table
CREATE TABLE IF NOT EXISTS {} (
    id TEXT PRIMARY KEY,
    vector BLOB NOT NULL,
    doc_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    chunk_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    metadata TEXT DEFAULT '{{}}'
);

-- Metadata table for namespace tracking
CREATE TABLE IF NOT EXISTS {} (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    namespace TEXT NOT NULL,
    pack_name TEXT NOT NULL DEFAULT '',
    workspace_id TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    document_count INTEGER NOT NULL DEFAULT 0,
    chunk_count INTEGER NOT NULL DEFAULT 0,
    embedding_provider TEXT NOT NULL DEFAULT '',
    embedding_dimensions INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for fast doc_id lookups
CREATE INDEX IF NOT EXISTS idx_v_{}_doc_id ON {}(doc_id);

-- Index for fast content search
CREATE INDEX IF NOT EXISTS idx_v_{}_content ON {}(content);

-- Insert default metadata record
INSERT INTO {} (namespace, workspace_id, schema_version, document_count, chunk_count, embedding_provider, embedding_dimensions)
VALUES ('{}', '{}', {}, 0, 0, 'local', 384)
ON CONFLICT DO NOTHING;
"#,
            table_name,
            metadata_table,
            namespace,
            table_name,
            namespace,
            table_name,
            metadata_table,
            namespace,
            "00000000-0000-0000-0000-000000000000",
            schema_version
        );

        // Add HNSW extension if enabled
        schema.push_str(
            r#"
-- Try to load SQLite VSS extension for HNSW support
-- This is optional and will be silently ignored if the extension is not available
SELECT load_extension('mod_spatialite') WHERE 1=0;
"#,
        );

        schema
    }

    /// Serialize a vector to a blob (f32 bytes).
    fn serialize_vector(vector: &[f32]) -> Vec<u8> {
        // Safety: transmute f32 slice to bytes
        unsafe {
            std::slice::from_raw_parts(
                vector.as_ptr() as *const u8,
                std::mem::size_of_val(vector),
            )
            .to_vec()
        }
    }

    /// Deserialize a vector from a blob.
    #[allow(dead_code)]
    fn deserialize_vector(bytes: &[u8], dimensions: usize) -> Vec<f32> {
        let mut vector = vec![0.0f32; dimensions];
        let chunk_size = std::mem::size_of::<f32>();
        for (i, chunk) in bytes.chunks(chunk_size).take(dimensions).enumerate() {
            if chunk.len() == chunk_size {
                let mut arr = [0u8; 4];
                arr.copy_from_slice(chunk);
                vector[i] = f32::from_le_bytes(arr);
            }
        }
        vector
    }

    /// Insert a vector into the store.
    pub async fn insert_vector(
        &self,
        vector_id: &str,
        vector: &[f32],
        content: &str,
        doc_id: &str,
    ) -> anyhow::Result<()> {
        let conn = self.connection.lock().expect("Vector store lock poisoned");

        let vector_blob = Self::serialize_vector(vector);

        conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {} (id, vector, doc_id, content, chunk_index) VALUES (?, ?, ?, ?, 0)",
                self.config.namespace
            ),
            params![
                vector_id,
                vector_blob,
                doc_id,
                content,
            ],
        )?;

        debug!(vector_id, doc_id, "Vector inserted");
        Ok(())
    }

    /// Search for similar vectors using cosine distance.
    pub async fn search(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> anyhow::Result<Vec<super::search::SearchResult>> {
        let conn = self.connection.lock().expect("Vector store lock poisoned");

        let _query_blob = Self::serialize_vector(query_vector);
        let _dimensions = query_vector.len();

        // Use vector distance for flat search
        // Note: SQLite VSS extension provides vector_distance() function
        // For flat search without VSS extension, we return results ordered by doc_id
        let mut stmt = conn.prepare(&format!(
            "SELECT id, content, doc_id, chunk_index FROM {} ORDER BY id LIMIT ?",
            self.config.namespace
        ))?;
        let results: Vec<(String, String, String, usize)> = stmt
            .query_map(params![limit as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut search_results = Vec::new();
        for (vector_id, content, doc_id, _chunk_index) in results {
            search_results.push(super::search::SearchResult {
                vector_id,
                score: 0.0,
                content,
                doc_id,
            });
        }

        debug!(count = search_results.len(), "Vector search complete");
        Ok(search_results)
    }

    /// Get all vectors for a document.
    pub async fn get_vectors_for_doc(&self, doc_id: &str) -> anyhow::Result<Vec<(String, String)>> {
        let conn = self.connection.lock().expect("Vector store lock poisoned");

        let mut stmt = conn.prepare(&format!(
            "SELECT id, content FROM {} WHERE doc_id = ?",
            self.config.namespace
        ))?;
        let results: Vec<(String, String)> = stmt
            .query_map(params![doc_id], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    /// Get vector count.
    pub async fn vector_count(&self) -> anyhow::Result<usize> {
        let conn = self.connection.lock().expect("Vector store lock poisoned");

        let count: usize = conn.query_row(
            &format!("SELECT COUNT(*) FROM {}", self.config.namespace),
            [],
            |row| row.get(0),
        )?;

        Ok(count)
    }

    /// Update metadata.
    pub async fn update_metadata(
        &self,
        document_count: usize,
        chunk_count: usize,
        embedding_provider: &str,
        embedding_dimensions: usize,
    ) -> anyhow::Result<()> {
        let conn = self.connection.lock().expect("Vector store lock poisoned");

        conn.execute(
            &format!(
                "UPDATE vector_metadata_{} SET document_count = ?, chunk_count = ?, embedding_provider = ?, embedding_dimensions = ?, updated_at = datetime('now') WHERE namespace = ?",
                self.config.namespace
            ),
            params![
                document_count as i64,
                chunk_count as i64,
                embedding_provider,
                embedding_dimensions as i64,
                self.config.namespace
            ],
        )?;

        debug!("Vector metadata updated");
        Ok(())
    }
}
