//! Local embedding provider.
//!
/// For development: generates deterministic pseudo-random vectors.
/// For production: swap in ONNX Runtime models (e.g., all-MiniLM-L6-v2).

use super::provider::{EmbeddingProvider, EmbeddingResult};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::debug;

/// Local embedding provider using deterministic pseudo-random vectors.
pub struct LocalEmbeddingProvider {
    dimensions: usize,
    seed_cache: HashMap<String, Vec<f32>>,
}

impl LocalEmbeddingProvider {
    pub fn new() -> Self {
        Self {
            dimensions: 384,
            seed_cache: HashMap::new(),
        }
    }

    /// Create a local provider with custom dimensions.
    pub fn with_dimensions(dimensions: usize) -> Self {
        Self {
            dimensions,
            seed_cache: HashMap::new(),
        }
    }

    /// Generate a deterministic pseudo-random vector from a seed string.
    fn generate_vector(&self, text: &str) -> Vec<f32> {
        // Simple hash-based approach for deterministic, reproducible vectors
        let seed = self.hash_seed(text);
        let mut vector = Vec::with_capacity(self.dimensions);

        for i in 0..self.dimensions {
            // Use hash of (text + index) for deterministic but distinct values
            let hash = self.hash_seed(&format!("{}:{}", text, i));
            // Normalize to [-1, 1] range using sine
            let val = ((hash as f64) / (u32::MAX as f64) * 2.0 - 1.0) as f32;
            vector.push(val);
        }

        vector
    }

    fn hash_seed(&self, input: &str) -> u32 {
        let mut hash: u32 = 5381;
        for byte in input.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        hash
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn embed(&self, text: &str) -> anyhow::Result<EmbeddingResult> {
        if text.trim().is_empty() {
            anyhow::bail!("Cannot embed empty text");
        }

        let vector = if let Some(cached) = self.seed_cache.get(text) {
            cached.clone()
        } else {
            let vector = self.generate_vector(text);
            self.seed_cache.insert(text.to_string(), vector.clone());
            vector
        };

        let result = EmbeddingResult {
            vector,
            dimensions: self.dimensions,
        };

        debug!(text_len = text.len(), dimensions = self.dimensions, "Local embedding generated");
        Ok(result)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn name(&self) -> String {
        format!("local-random-dim{}", self.dimensions)
    }

    fn provider_type(&self) -> &str {
        "local"
    }

    async fn embed_batch(&self, texts: Vec<&str>) -> anyhow::Result<Vec<EmbeddingResult>> {
        let mut results = Vec::new();
        for text in &texts {
            let vector = if let Some(cached) = self.seed_cache.get(*text) {
                cached.clone()
            } else {
                let vector = self.generate_vector(*text);
                self.seed_cache.insert((*text).to_string(), vector.clone());
                vector
            };
            results.push(EmbeddingResult {
                vector,
                dimensions: self.dimensions,
            });
        }
        Ok(results)
    }
}

impl Default for LocalEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}