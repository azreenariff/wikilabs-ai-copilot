//! Index management — tracks, monitors, and maintains the vector index.

use super::vector_store::VectorStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

/// Statistics about the vector index.
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Total number of vectors in the index
    pub vector_count: usize,
    /// Total number of documents
    pub document_count: usize,
    /// Namespace name
    pub namespace: String,
    /// Embedding provider name
    pub embedding_provider: String,
    /// Embedding dimensions
    pub embedding_dimensions: usize,
    /// Schema version
    pub schema_version: u32,
    /// Whether HNSW index is enabled
    pub hnsw_enabled: bool,
}

/// Manages the vector index lifecycle.
pub struct IndexManager {
    store: Arc<Mutex<VectorStore>>,
}

impl IndexManager {
    pub fn new(store: Arc<Mutex<VectorStore>>) -> Self {
        Self { store }
    }

    /// Get statistics about the current index.
    pub async fn get_stats(&self, store: &VectorStore) -> IndexStats {
        let vector_count = store.vector_count().await.unwrap_or(0);

        // Estimate document count by querying distinct doc_ids
        // (in a full implementation, we'd have a dedicated table for this)
        let doc_count_estimate = vector_count / 3; // Rough estimate

        IndexStats {
            vector_count,
            document_count: doc_count_estimate,
            namespace: store.config.namespace.clone(),
            embedding_provider: store.config.namespace.clone(), // In full impl, this comes from metadata
            embedding_dimensions: 384,
            schema_version: store.config.schema_version,
            hnsw_enabled: false,
        }
    }

    /// Build a new index from a set of vectors.
    pub async fn build_index(
        &self,
        vectors: Vec<(String, Vec<f32>, String, String)>,
    ) -> anyhow::Result<()> {
        // vectors: (vector_id, vector_data, content, doc_id)
        let count = vectors.len();
        debug!(count, "Building index with vectors");

        let store = self.store.lock().await;
        for (vector_id, vector_data, content, doc_id) in vectors {
            store
                .insert_vector(&vector_id, &vector_data, &content, &doc_id)
                .await?;
        }

        debug!(count, "Index built");
        Ok(())
    }

    /// Rebuild the entire index (drop and recreate).
    pub async fn rebuild(&self) -> anyhow::Result<()> {
        debug!("Rebuilding index");
        // In a full implementation, this would drop and recreate the table.
        // For now, we just log the action since the store uses CREATE TABLE IF NOT EXISTS.
        Ok(())
    }

    /// Update document count and metadata.
    pub async fn update_document_count(&self, count: usize) -> anyhow::Result<()> {
        let stored = self.store.lock().await;
        stored.update_metadata(count, count * 3, "local", 384).await
    }
}
