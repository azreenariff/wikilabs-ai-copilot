//! Embedding pipeline module.

pub mod provider;
pub mod local;
pub mod batch;
pub mod incremental;
pub mod version;

pub use provider::{EmbeddingProvider, EmbeddingPipelineError};
pub use local::LocalEmbeddingProvider;
pub use batch::BatchEmbedder;
pub use incremental::IncrementalEmbedder;
pub use version::{EmbeddingVersionManager, EmbeddingVersion};

pub(crate) type EmbeddingResult = super::embedding::provider::EmbeddingResult;
use super::pipeline::result::PipelineResult;
use crate::doc::{KnowledgeChunk, KnowledgeDocument};
use crate::storage::vector_store::VectorStore;
use anyhow::Context;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Configuration for the embedding pipeline.
#[derive(Debug, Clone)]
pub struct EmbeddingPipelineConfig {
    /// Provider to use for embedding generation
    pub provider: Arc<dyn EmbeddingProvider>,
    /// Maximum batch size for embedding generation
    pub batch_size: usize,
    /// Number of parallel workers
    pub parallel_workers: usize,
    /// Embedding dimensions (from provider)
    pub dimensions: usize,
}

impl Default for EmbeddingPipelineConfig {
    fn default() -> Self {
        let provider = Arc::new(LocalEmbeddingProvider::new());
        let dimensions = provider.dimensions();
        Self {
            provider,
            batch_size: 32,
            parallel_workers: 4,
            dimensions,
        }
    }
}

/// The embedding pipeline orchestrator.
pub struct EmbeddingPipeline {
    config: EmbeddingPipelineConfig,
    version_manager: EmbeddingVersionManager,
    vector_store: Option<Arc<Mutex<VectorStore>>>,
}

impl EmbeddingPipeline {
    pub fn new(config: Option<EmbeddingPipelineConfig>) -> Self {
        let config = config.unwrap_or_default();
        Self {
            config: config.clone(),
            version_manager: EmbeddingVersionManager::new(),
            vector_store: None,
        }
    }

    pub fn with_vector_store(mut self, store: Arc<Mutex<VectorStore>>) -> Self {
        self.vector_store = Some(store);
        self
    }

    pub fn provider(&self) -> &Arc<dyn EmbeddingProvider> {
        &self.config.provider
    }

    /// Embed a single chunk of text.
    pub async fn embed_chunk(&self, content: &str) -> anyhow::Result<Vec<f32>> {
        self.config.provider
            .embed(content)
            .await
            .with_context(|| format!("Failed to embed text ({} chars)", content.len()))
    }

    /// Embed chunks in parallel batches.
    pub async fn embed_chunks(&self, chunks: &[&KnowledgeChunk]) -> anyhow::Result<Vec<EmbeddingResult>> {
        let batch_size = self.config.batch_size;
        let mut results = Vec::new();

        for batch in chunks.chunks(batch_size) {
            let batch_vecs: Vec<&KnowledgeChunk> = batch.to_vec();
            let tasks: Vec<_> = batch_vecs.iter().map(|chunk| {
                let content = chunk.content.clone();
                tokio::spawn(async move {
                    match self.config.provider.embed(&content).await {
                        Ok(r) => Some(r),
                        Err(e) => {
                            warn!(chunk_id = %chunk.id, error = %e, "Embedding failed");
                            None
                        }
                    }
                })
            }).collect();

            for task in futures::future::join_all(tasks).await {
                match task {
                    Ok(Ok(Some(result))) => results.push(result),
                    Ok(Ok(None)) => {} // Failed embedding, already logged
                    Ok(Err(e)) => {
                        warn!(error = %e, "Task error in embed_chunks");
                    }
                    Err(e) => {
                        warn!(error = %e, "Spawn error in embed_chunks");
                    }
                }
            }
        }

        debug!(
            total = results.len(),
            "Embedding complete"
        );

        Ok(results)
    }

    /// Embed all chunks from a pipeline result and index them.
    pub async fn embed_and_index(
        &self,
        result: &PipelineResult,
    ) -> anyhow::Result<()> {
        let docs = result.documents();
        let chunks = result.all_chunks();

        if chunks.is_empty() {
            info!("No chunks to embed");
            return Ok(());
        }

        info!(doc_count = docs.len(), chunk_count = chunks.len(), "Starting embedding pipeline");

        // Track version
        let version = self.version_manager.create_version(
            self.config.provider.name(),
            self.config.dimensions,
            format!("Indexed {} chunks", chunks.len()),
        );

        // Prepare chunks for embedding
        let chunk_refs: Vec<&KnowledgeChunk> = chunks.iter().map(|c| c as &KnowledgeChunk).collect();
        let embeddings = self.embed_chunks(&chunk_refs).await?;

        // Index embeddings in vector store if available
        if let Some(ref store) = self.vector_store {
            let mut store_lock = store.lock().await;

            for (i, embedding) in embeddings.iter().enumerate() {
                if i < chunks.len() {
                    let chunk = &chunks[i];
                    store_lock
                        .insert_vector(
                            &chunk.vector_id,
                            &embedding.vector,
                            &chunk.content,
                            chunk.document_id.to_string(),
                        )
                        .await?;
                }
            }

            // Update version
            self.version_manager.finalize_version(
                &version,
                Some(embeddings.len()),
            );
        }

        debug!("Embedding and indexing complete");
        Ok(())
    }

    /// Re-index documents with new embeddings.
    pub async fn reindex(&self, doc_ids: &[uuid::Uuid]) -> anyhow::Result<()> {
        info!(count = doc_ids.len(), "Starting reindex");

        for doc_id in doc_ids {
            debug!(doc_id = %doc_id, "Re-indexing document");
            // Placeholder: in a full implementation, fetch chunks, re-embed, and update store
            // For now, just track the version
            self.version_manager.create_version(
                self.config.provider.name(),
                self.config.dimensions,
                format!("Re-indexed document {}", doc_id),
            );
        }

        Ok(())
    }
}

impl Default for EmbeddingPipeline {
    fn default() -> Self {
        Self::new(None)
    }
}