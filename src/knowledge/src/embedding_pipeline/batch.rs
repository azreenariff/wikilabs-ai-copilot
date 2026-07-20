//! Batch embedding — generate embeddings for multiple chunks efficiently.

use super::provider::EmbeddingProvider;
use super::{EmbeddingPipelineConfig, EmbeddingPipelineResult, EmbeddingResult, EmbeddingStatus};
use crate::doc::KnowledgeChunk;
use chrono::Utc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Batch embedder — processes chunks in configurable batch sizes.
pub struct BatchEmbedder {
    config: EmbeddingPipelineConfig,
}

impl BatchEmbedder {
    pub fn new(config: EmbeddingPipelineConfig) -> Self {
        Self { config }
    }

    pub fn with_provider(mut self, provider: &dyn EmbeddingProvider) -> Self {
        self.config.provider = provider.name().to_string();
        self.config.model = provider.model_name().to_string();
        self.config.dimensions = provider.dimensions();
        self
    }

    /// Generate embeddings for a batch of chunks.
    pub async fn embed_batch(
        &self,
        chunks: &[KnowledgeChunk],
        provider: &dyn EmbeddingProvider,
    ) -> anyhow::Result<EmbeddingPipelineResult> {
        let start = Utc::now();
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Filter out chunks that already have a vector_id (not empty).
        let pending: Vec<&KnowledgeChunk> =
            chunks.iter().filter(|c| c.vector_id.is_empty()).collect();
        let texts: Vec<String> = pending.iter().map(|c| c.content.clone()).collect();

        let total_pending = texts.len();
        skipped = chunks.len().saturating_sub(total_pending);

        if total_pending == 0 {
            info!("All chunks already have embeddings, skipping");
            return Ok(EmbeddingPipelineResult {
                total_chunks: chunks.len(),
                successful: chunks.len(),
                failed: 0,
                skipped,
                duration_ms: 0,
                results: Vec::new(),
            });
        }

        info!(
            total = total_pending,
            batch_size = self.config.batch_size,
            provider = self.config.provider,
            model = self.config.model,
            "Starting batch embedding"
        );

        for batch in texts.chunks(self.config.batch_size) {
            let batch_size = batch.len();
            let batch_chunks = &pending[successful..successful + batch_size];

            match provider.embed_batch(batch).await {
                Ok(vectors) => {
                    for (i, vector) in vectors.iter().enumerate() {
                        let chunk_id = batch_chunks
                            .get(i)
                            .map(|c| c.id.to_string())
                            .unwrap_or_else(|| format!("batch_{}_{}", i, batch_chunks.len()));

                        let mut result = EmbeddingResult {
                            chunk_id,
                            vector: vector.clone(),
                            model: self.config.model.clone(),
                            dimensions: self.config.dimensions,
                            status: EmbeddingStatus::Completed,
                            timestamp: Utc::now(),
                        };

                        if self.config.normalize {
                            result.vector = normalize_vector(result.vector);
                        }

                        results.push(result);
                        successful += 1;
                    }
                    debug!(batch_size = batch.len(), "Completed batch embedding");
                }
                Err(e) => {
                    warn!(error = %e, batch_size = batch.len(), "Batch embedding failed");
                    for _ in batch {
                        failed += 1;
                        results.push(EmbeddingResult {
                            chunk_id: Uuid::new_v4().to_string(),
                            vector: Vec::new(),
                            model: self.config.model.clone(),
                            dimensions: self.config.dimensions,
                            status: EmbeddingStatus::Failed(e.to_string()),
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
        }

        let duration = Utc::now() - start;
        let duration_ms = duration.num_milliseconds() as u64;

        info!(
            total = chunks.len(),
            successful, failed, skipped, duration_ms, "Batch embedding complete"
        );

        Ok(EmbeddingPipelineResult {
            total_chunks: chunks.len(),
            successful,
            failed,
            skipped,
            duration_ms,
            results,
        })
    }
}

/// Normalize a vector to unit length.
fn normalize_vector(mut vec: Vec<f32>) -> Vec<f32> {
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
    vec
}

/// Compute cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-10 {
        return 0.0;
    }
    dot / denom
}
