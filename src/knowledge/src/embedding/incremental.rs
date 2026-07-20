//! Incremental embedding — only new/changed docs.
//!
use super::local::LocalEmbeddingProvider;
/// Tracks which documents have been embedded and skips unchanged ones.
use super::provider::{EmbeddingProvider, EmbeddingResult};
use super::BatchEmbedder;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// State for incremental embedding tracking.
#[derive(Debug, Clone)]
pub struct IncrementalState {
    /// Last embedded content hashes (doc_id -> hash)
    pub embedded_hashes: HashMap<String, String>,
    /// Last embedding generation time
    pub last_embedding_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl IncrementalState {
    pub fn new() -> Self {
        Self {
            embedded_hashes: HashMap::new(),
            last_embedding_time: None,
        }
    }
}

/// Incremental embedder that tracks which documents have been embedded.
pub struct IncrementalEmbedder {
    provider: Arc<dyn EmbeddingProvider>,
    batch_embedder: BatchEmbedder,
    state: Arc<Mutex<IncrementalState>>,
    content_hasher: Arc<dyn Fn(&str) -> String + Send + Sync>,
}

impl IncrementalEmbedder {
    pub fn new(
        provider: Arc<dyn EmbeddingProvider>,
        content_hasher: Option<Arc<dyn Fn(&str) -> String + Send + Sync>>,
    ) -> Self {
        let batch_embedder = BatchEmbedder::new(Arc::clone(&provider), None);
        let hasher = content_hasher.unwrap_or_else(|| {
            Arc::new(|text: &str| -> String {
                let mut hash: u32 = 5381;
                for byte in text.bytes() {
                    hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
                }
                format!("{:x}", hash)
            }) as Arc<dyn Fn(&str) -> String + Send + Sync>
        });

        Self {
            provider: Arc::clone(&provider),
            batch_embedder,
            state: Arc::new(Mutex::new(IncrementalState::new())),
            content_hasher: hasher,
        }
    }

    /// Get a new embedder for incremental embedding with the same provider type.
    pub fn with_provider(provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self::new(provider, None)
    }

    /// Compute content hash for a document.
    pub fn compute_content_hash(&self, text: &str) -> String {
        (self.content_hasher)(text)
    }

    /// Embed only new or changed documents.
    ///
    /// Takes a list of (doc_id, content) pairs and returns embeddings
    /// only for documents that have not been embedded before or have changed.
    pub async fn embed_incremental(
        &self,
        updates: Vec<(String, String)>,
    ) -> anyhow::Result<Vec<(String, EmbeddingResult)>> {
        let state = self.state.lock().await;
        let mut to_embed = Vec::new();
        let mut skipped = 0;

        for (doc_id, content) in &updates {
            let hash = (self.content_hasher)(content);
            if let Some(cached_hash) = state.embedded_hashes.get(doc_id) {
                if cached_hash == &hash {
                    debug!(doc_id, "Content unchanged, skipping");
                    skipped += 1;
                    continue;
                }
            }
            to_embed.push((doc_id.clone(), content.clone(), hash));
        }

        drop(state);

        info!(
            total = updates.len(),
            to_embed = to_embed.len(),
            skipped = skipped,
            "Incremental embedding"
        );

        if to_embed.is_empty() {
            return Ok(Vec::new());
        }

        // Batch embed new/changed documents
        let texts: Vec<&str> = to_embed
            .iter()
            .map(|(_, content, _)| content.as_str())
            .collect();
        let results = self.batch_embedder.embed(texts).await?;

        // Update state
        let mut state = self.state.lock().await;
        let mut output = Vec::new();

        for ((doc_id, _, hash), result) in to_embed.into_iter().zip(results.into_iter()) {
            state.embedded_hashes.insert(doc_id.clone(), hash);
            output.push((doc_id, result));
        }

        state.last_embedding_time = Some(chrono::Utc::now());

        debug!(embedded = output.len(), "Incremental embedding complete");

        Ok(output)
    }

    /// Mark a document as embedded (for external tracking).
    pub async fn mark_embedded(&self, doc_id: &str, content_hash: &str) {
        let mut state = self.state.lock().await;
        state
            .embedded_hashes
            .insert(doc_id.to_string(), content_hash.to_string());
    }

    /// Clear the incremental state (e.g., on full reindex).
    pub async fn clear_state(&self) {
        let mut state = self.state.lock().await;
        state.embedded_hashes.clear();
        state.last_embedding_time = None;
    }

    /// Get the number of embedded documents.
    pub async fn embedded_count(&self) -> usize {
        let state = self.state.lock().await;
        state.embedded_hashes.len()
    }
}

impl Default for IncrementalEmbedder {
    fn default() -> Self {
        Self::new(Arc::new(LocalEmbeddingProvider::new()), None)
    }
}
