//! Incremental indexing — only index new/changed vectors.
//!
/// Tracks which documents have been indexed and skips unchanged ones.
use super::vector_store::VectorStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// State tracking for incremental indexing.
#[derive(Debug, Clone)]
pub struct IncrementalState {
    /// Last indexed content hashes (doc_id -> hash)
    pub indexed_hashes: std::collections::HashMap<String, String>,
    /// Last indexing timestamp
    pub last_index_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for IncrementalState {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalState {
    pub fn new() -> Self {
        Self {
            indexed_hashes: std::collections::HashMap::new(),
            last_index_time: None,
        }
    }
}

/// Incremental indexer that tracks which documents have been indexed.
pub struct IncrementalIndexer {
    store: Arc<Mutex<VectorStore>>,
    state: std::sync::Arc<Mutex<IncrementalState>>,
}

impl IncrementalIndexer {
    pub fn new(store: Arc<Mutex<VectorStore>>) -> Self {
        Self {
            store,
            state: std::sync::Arc::new(Mutex::new(IncrementalState::new())),
        }
    }

    /// Compute content hash for a document.
    pub fn compute_content_hash(&self, text: &str) -> String {
        let mut hash: u32 = 5381;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        format!("{:x}", hash)
    }

    /// Index only new or changed documents.
    ///
    /// Takes a list of (doc_id, content, vector) triples and indexes
    /// only documents that have not been indexed before or have changed.
    pub async fn index_incremental(
        &self,
        updates: Vec<(String, String, Vec<f32>)>,
    ) -> anyhow::Result<usize> {
        let state = self.state.lock().await;
        let mut to_index = Vec::new();
        let mut skipped = 0;

        for (doc_id, content, vector) in &updates {
            let hash = self.compute_content_hash(content);
            if let Some(cached_hash) = state.indexed_hashes.get(doc_id) {
                if cached_hash == &hash {
                    debug!(doc_id, "Content unchanged, skipping indexing");
                    skipped += 1;
                    continue;
                }
            }
            to_index.push((doc_id.clone(), content.clone(), vector.clone()));
        }

        drop(state);

        info!(
            total = updates.len(),
            to_index = to_index.len(),
            skipped = skipped,
            "Incremental indexing"
        );

        if to_index.is_empty() {
            return Ok(0);
        }

        let store = self.store.lock().await;
        let mut indexed = 0;

        for (doc_id, content, vector) in to_index {
            let vector_id = format!("{}_{}", doc_id, indexed);
            store
                .insert_vector(&vector_id, &vector, &content, &doc_id)
                .await?;
            indexed += 1;

            // Update state
            let mut state = self.state.lock().await;
            state
                .indexed_hashes
                .insert(doc_id, self.compute_content_hash(&content));
        }

        let mut state = self.state.lock().await;
        state.last_index_time = Some(chrono::Utc::now());

        debug!(indexed = indexed, "Incremental indexing complete");
        Ok(indexed)
    }

    /// Mark a document as indexed (for external tracking).
    pub async fn mark_indexed(&self, doc_id: &str, content_hash: &str) {
        let mut state = self.state.lock().await;
        state
            .indexed_hashes
            .insert(doc_id.to_string(), content_hash.to_string());
    }

    /// Clear the incremental state (e.g., on full reindex).
    pub async fn clear_state(&self) {
        let mut state = self.state.lock().await;
        state.indexed_hashes.clear();
        state.last_index_time = None;
    }

    /// Get the number of indexed documents.
    pub async fn indexed_count(&self) -> usize {
        let state = self.state.lock().await;
        state.indexed_hashes.len()
    }
}
