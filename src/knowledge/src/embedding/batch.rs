//! Batch embedding generation.
//!
use super::local::LocalEmbeddingProvider;
/// Handles efficient batch embedding of multiple texts.
use super::provider::{EmbeddingProvider, EmbeddingResult};
use futures::stream::StreamExt;
use std::sync::Arc;
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
        let texts_clone: Vec<&str> = texts.clone();
        let provider_batch_result = self.provider.embed_batch(texts_clone).await;

        let results = if let Ok(results) = provider_batch_result {
            if results.len() == texts.len() {
                results
            } else {
                self.embed_sequentially(texts).await?
            }
        } else {
            self.embed_sequentially(texts).await?
        };

        debug!(
            total = total,
            succeeded = results.len(),
            "Batch embedding complete"
        );

        Ok(results)
    }

    /// Embed texts sequentially.
    async fn embed_sequentially(&self, texts: Vec<&str>) -> anyhow::Result<Vec<EmbeddingResult>> {
        let provider = Arc::clone(&self.provider);
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            match provider.embed(text).await {
                Ok(r) => results.push(r),
                Err(e) => warn!(text = text, error = %e, "Embedding failed"),
            }
        }

        debug!(total = results.len(), "Sequential embedding complete");
        Ok(results)
    }

    /// Embed texts with streaming output.
    pub async fn embed_streaming(
        &self,
        texts: Vec<String>,
    ) -> impl StreamExt<Item = anyhow::Result<EmbeddingResult>> {
        let provider = Arc::clone(&self.provider);
        let parallel_workers = self.config.parallel_workers;

        futures::stream::iter(texts.into_iter().map(move |text| {
            let provider = Arc::clone(&provider);
            async move { provider.embed(&text).await }
        }))
        .buffer_unordered(parallel_workers)
    }
}

impl Default for BatchEmbedder {
    fn default() -> Self {
        Self::new(Arc::new(LocalEmbeddingProvider::new()), None)
    }
}
