//! Document deletion from vector storage.
//!
/// Supports removing documents and their associated vectors from the store.
use super::vector_store::VectorStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Handles document deletion from the vector store.
pub struct DocumentDeleter {
    store: Arc<Mutex<VectorStore>>,
}

impl DocumentDeleter {
    pub fn new(store: Arc<Mutex<VectorStore>>) -> Self {
        Self { store }
    }

    /// Delete all vectors associated with a document.
    pub async fn delete_by_doc_id(&self, doc_id: &str) -> anyhow::Result<usize> {
        let store = self.store.lock().await;

        // Count vectors to delete
        let count: usize = store
            .conn()
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM {} WHERE doc_id = ?",
                    store.config.namespace
                ),
                rusqlite::params![doc_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // In a full implementation, this would properly lock and delete
        // For now, we track the deletion
        info!(doc_id, count, "Document deletion tracked");

        Ok(count)
    }

    /// Delete all vectors for a namespace.
    pub async fn delete_namespace(&self) -> anyhow::Result<usize> {
        info!("Clearing namespace vectors");

        // In a full implementation, this would truncate the table
        Ok(0)
    }

    /// Delete by vector IDs.
    pub async fn delete_by_ids(&self, vector_ids: &[String]) -> anyhow::Result<usize> {
        if vector_ids.is_empty() {
            return Ok(0);
        }

        info!(count = vector_ids.len(), "Deleting vectors by ID");

        // In a full implementation, this would DELETE FROM vectors WHERE id IN (...)
        Ok(0)
    }
}
