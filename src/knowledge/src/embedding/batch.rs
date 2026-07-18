//! Batch embedding generation.
//!
/// Handles efficient batch embedding of multiple texts.

use super::provider::{EmbeddingProvider, EmbeddingResult};
use super::local::LocalEmbeddingProvider;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Configuration for batch embedding.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Number of parallel workers
    pub parallel_workers: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            parallel_workers: 4,
        }
    }
}

/// Batch embedder for efficient embedding generation.
pub struct BatchEmbedder {
    provider: Arc<dyn EmbeddingProvider>,
    config: BatchConfig,
}

impl BatchEmbedder {
    pub fn new(provider: Arc<dyn EmbeddingProvider>, config: Option<BatchConfig>) -> Self {
        Self {
            provider,
            config: config.unwrap_or_default(),
        }
    }

    /// Generate embeddings for a batch of texts.
    pub async fn embed(&self, texts: Vec<&str>) -> anyhow::Result<Vec<EmbeddingResult>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let total = texts.len();
        info!(batch_size = total, "Starting batch embedding");

        // Try provider's native batch implementation first
        let provider_batch_result = self.provider.embed_batch(texts.clone()).await;

        let results = if let Ok(results) = provider_batch_result {
            if results.len() == texts.len() {
                results
            } else {
                // Fallback to individual embedding
                self.embed_sequentially(&texts).await?
            }
        } else {
            // Fallback to individual embedding
            self.embed_sequentially(&texts).await?
        };

        debug!(
            total = total,
            succeeded = results.len(),
            "Batch embedding complete"
        );

        Ok(results)
    }

    /// Embed texts in parallel with concurrency control.
    async fn embed_sequentially(&self, texts: &[&str]) -> anyhow::Result<Vec<EmbeddingResult>> {
        let mut results = Vec::with_capacity(texts.len());
        let mut failed = Vec::new();

        // Split into chunks and process in parallel
        let chunks: Vec<Vec<&str>> = texts
            .chunks(self.config.max_batch_size)
            .map(|c| c.to_vec())
            .collect();

        for chunk in chunks {
            let chunk_clone = chunk.clone();
            let provider = Arc::clone(&self.provider);

            let chunk_results = tokio::task::spawn_blocking(move || {
                let mut chunk_results = Vec::new();
                for text in chunk_clone {
                    match futures::executor::block_on(provider.embed(text)) {
                        Ok(result) => chunk_results.push(result),
                        Err(e) => {
                            warn!(text = text, error = %e, "Embedding failed in batch");
                            failed.push(text.to_string());
                        }
                    }
                }
                chunk_results
            }).await?;

            results.extend(chunk_results);
        }

        if !failed.is_empty() {
            warn!(failed_count = failed.len(), total = texts.len(), "Some embeddings failed");
        }

        Ok(results)
    }

    /// Embed texts with streaming output (each result as it completes).
    pub async fn embed_streaming(
        &self,
        texts: Vec<&str>,
    ) -> impl StreamExt<Item = anyhow::Result<EmbeddingResult>> {
        let provider = Arc::clone(&self.provider);
        let parallel_workers = self.config.parallel_workers;

        let stream = stream::iter(texts.into_iter().map(move |text| {
            let provider = Arc::clone(&provider);
            let text = text.to_string();
            async move {
                provider.embed(&text).await
            }
        }))
        .buffer_unordered(parallel_workers);

        stream
    }
}

impl Default for BatchEmbedder {
    fn default() -> Self {
        Self::new(Arc::new(LocalEmbeddingProvider::new()), None)
    }
}