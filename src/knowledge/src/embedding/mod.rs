//! Embedding pipeline module.

pub mod batch;
pub mod incremental;
pub mod local;
pub mod provider;
pub mod version;

pub use batch::BatchEmbedder;
pub use incremental::IncrementalEmbedder;
pub use local::LocalEmbeddingProvider;
pub use provider::{EmbeddingPipelineError, EmbeddingProvider};
pub use version::{EmbeddingVersion, EmbeddingVersionManager};

pub(crate) type EmbeddingResult = super::embedding::provider::EmbeddingResult;
use super::pipeline::result::PipelineResult;
use crate::doc::KnowledgeChunk;
use crate::storage::vector_store::VectorStore;
use anyhow::Context;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Configuration for the embedding pipeline.
#[derive(Clone)]
pub struct EmbeddingPipelineConfig {
    /// Provider to use for embedding generation
    pub provider: Arc<dyn EmbeddingProvider + Send + Sync>,
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
    version_manager: std::sync::Mutex<EmbeddingVersionManager>,
    vector_store: Option<Arc<Mutex<VectorStore>>>,
}

impl EmbeddingPipeline {
    pub fn new(config: Option<EmbeddingPipelineConfig>) -> Self {
        let config = config.unwrap_or_default();
        Self {
            config: config.clone(),
            version_manager: std::sync::Mutex::new(EmbeddingVersionManager::new()),
            vector_store: None,
        }
    }

    pub fn with_vector_store(mut self, store: Arc<Mutex<VectorStore>>) -> Self {
        self.vector_store = Some(store);
        self
    }

    pub fn provider(&self) -> &Arc<dyn EmbeddingProvider + Send + Sync> {
        &self.config.provider
    }

    /// Embed a single chunk of text.
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let result = self
            .config
            .provider
            .embed(text)
            .await
            .with_context(|| format!("Failed to embed text ({} chars)", text.len()))?;
        Ok(result.vector)
    }

    /// Embed chunks in parallel batches.
    pub async fn embed_chunks(
        &self,
        chunks: &[&KnowledgeChunk],
    ) -> anyhow::Result<Vec<EmbeddingResult>> {
        let batch_size = self.config.batch_size;
        let mut results = Vec::new();

        for batch in chunks.chunks(batch_size) {
            let batch_data: Vec<(String, String)> = batch
                .iter()
                .map(|c| (c.content.clone(), c.id.to_string()))
                .collect();
            let tasks: Vec<_> = batch_data
                .iter()
                .map(|(content, chunk_id)| {
                    let provider = self.config.provider.clone();
                    let content = content.clone();
                    let chunk_id = chunk_id.clone();
                    tokio::spawn(async move {
                        match provider.embed(&content).await {
                            Ok(r) => Ok::<_, anyhow::Error>(Some(r)),
                            Err(e) => {
                                warn!(chunk_id = %chunk_id, error = %e, "Embedding failed");
                                Ok(None)
                            }
                        }
                    })
                })
                .collect();

            for task in futures::future::join_all(tasks).await {
                match task {
                    Ok(Ok(Some(result))) => results.push(result),
                    Ok(Ok(None)) => {} // Failed embedding, already logged
                    Ok(Err(e)) => {
                        warn!(error = %e, "Task returned error in embed_chunks");
                    }
                    Err(e) => {
                        warn!(error = %e, "Task join error in embed_chunks");
                    }
                }
            }
        }

        debug!(total = results.len(), "Embedding complete");

        Ok(results)
    }

    /// Embed all chunks from a pipeline result and index them.
    pub async fn embed_and_index(&mut self, result: &PipelineResult) -> anyhow::Result<()> {
        let docs = result.documents();
        let chunks = result.all_chunks();

        if chunks.is_empty() {
            info!("No chunks to embed");
            return Ok(());
        }

        info!(
            doc_count = docs.len(),
            chunk_count = chunks.len(),
            "Starting embedding pipeline"
        );

        // Track version
        let version = self.version_manager.lock().unwrap().create_version(
            self.config.provider.name(),
            self.config.dimensions,
            format!("Indexed {} chunks", chunks.len()),
        );

        // Prepare chunks for embedding
        let knowledge_chunks: Vec<KnowledgeChunk> = chunks
            .iter()
            .map(|c| KnowledgeChunk {
                id: c.chunk_id,
                document_id: c.document_id,
                content: c.content.clone(),
                vector_id: c.vector_id.clone(),
            })
            .collect();
        let chunk_refs: Vec<&KnowledgeChunk> = knowledge_chunks.iter().collect();
        // Embed each chunk individually
        let embeddings: Vec<EmbeddingResult> = self.embed_chunks(&chunk_refs).await?;

        // Index embeddings in vector store if available
        if let Some(ref store) = self.vector_store {
            let store_lock = store.lock().await;

            for (i, embedding) in embeddings.iter().enumerate() {
                if i < chunks.len() {
                    let chunk = &chunks[i];
                    store_lock
                        .insert_vector(
                            &chunk.vector_id,
                            &embedding.vector,
                            &chunk.content,
                            &chunk.document_id.to_string(),
                        )
                        .await?;
                }
            }

            // Update version
            self.version_manager
                .lock()
                .unwrap()
                .finalize_version(&version, Some(embeddings.len()));
        }

        debug!("Embedding and indexing complete");
        Ok(())
    }

    /// Re-index documents with new embeddings.
    pub async fn reindex(&mut self, doc_ids: &[uuid::Uuid]) -> anyhow::Result<()> {
        info!(count = doc_ids.len(), "Starting reindex");

        for doc_id in doc_ids {
            debug!(doc_id = %doc_id, "Re-indexing document");
            // Placeholder: in a full implementation, fetch chunks, re-embed, and update store
            // For now, just track the version
            // Track the version
            self.version_manager.lock().unwrap().create_version(
                self.config.provider.name().clone(),
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
