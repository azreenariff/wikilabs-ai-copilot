//! Incremental embedding — update only changed/missing embeddings.

use super::provider::EmbeddingProvider;
use super::{EmbeddingPipelineConfig, EmbeddingPipelineResult, EmbeddingResult, EmbeddingStatus};
use crate::doc::KnowledgeChunk;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, info};

/// Incremental embedder — only processes changed documents.
pub struct IncrementalEmbedder {
    config: EmbeddingPipelineConfig,
}

impl IncrementalEmbedder {
    pub fn new(config: EmbeddingPipelineConfig) -> Self {
        Self { config }
    }

    /// Generate embeddings only for chunks that are missing or changed.
    pub async fn embed_incremental(
        &self,
        all_chunks: &[KnowledgeChunk],
        provider: &dyn EmbeddingProvider,
        last_embeddings: &[(&str, &Vec<f32>, &str)], // (chunk_id, vector, embedding_version)
    ) -> anyhow::Result<EmbeddingPipelineResult> {
        let start = Utc::now();
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Build lookup of existing embeddings
        let existing: HashMap<String, (&Vec<f32>, &str)> = last_embeddings
            .iter()
            .map(|(id, vec, ver)| (id.to_string(), (*vec, *ver)))
            .collect();

        // Find chunks that need updating
        let mut pending_ids = Vec::new();
        let mut pending_texts = Vec::new();

        for chunk in all_chunks {
            // A chunk needs embedding if it has no vector_id yet
            let has_embedding = !chunk.vector_id.is_empty();
            let version_changed = existing
                .get(&chunk.id.to_string())
                .map(|(_, ver)| ver != &self.config.model)
                .unwrap_or(false);

            if has_embedding && !version_changed {
                skipped += 1;
                continue;
            }

            if has_embedding {
                // Embedding exists but version changed — regenerate
                debug!(chunk_id = %chunk.id, "Re-embedding due to version change");
            }

            pending_ids.push(chunk.id.to_string());
            pending_texts.push(chunk.content.clone());
        }

        let total_pending = pending_texts.len();
        let total = all_chunks.len();

        if total_pending == 0 {
            info!("No chunks need re-embedding");
            return Ok(EmbeddingPipelineResult {
                total_chunks: total,
                successful: skipped,
                failed: 0,
                skipped,
                duration_ms: 0,
                results: Vec::new(),
            });
        }

        info!(
            pending = total_pending,
            skipped,
            provider = self.config.provider,
            "Starting incremental embedding"
        );

        // Process in batches
        for (i, text) in pending_texts.iter().enumerate() {
            match provider.embed(text).await {
                Ok(vector) => {
                    let result = EmbeddingResult {
                        chunk_id: pending_ids[i].clone(),
                        vector: if self.config.normalize {
                            normalize_vector(vector)
                        } else {
                            vector
                        },
                        model: self.config.model.clone(),
                        dimensions: self.config.dimensions,
                        status: EmbeddingStatus::Completed,
                        timestamp: Utc::now(),
                    };
                    results.push(result);
                    successful += 1;
                }
                Err(e) => {
                    failed += 1;
                    results.push(EmbeddingResult {
                        chunk_id: pending_ids[i].clone(),
                        vector: Vec::new(),
                        model: self.config.model.clone(),
                        dimensions: self.config.dimensions,
                        status: EmbeddingStatus::Failed(e.to_string()),
                        timestamp: Utc::now(),
                    });
                    debug!(chunk_id = pending_ids[i], error = %e, "Incremental embedding failed");
                }
            }
        }

        let duration_ms = Utc::now() - start;
        let duration_ms = duration_ms.num_milliseconds() as u64;

        info!(
            pending = total_pending,
            successful, failed, duration_ms, "Incremental embedding complete"
        );

        Ok(EmbeddingPipelineResult {
            total_chunks: total,
            successful,
            failed,
            skipped,
            duration_ms,
            results,
        })
    }
}

fn normalize_vector(mut vec: Vec<f32>) -> Vec<f32> {
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
    vec
}
