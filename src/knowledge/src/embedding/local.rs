//! Local embedding provider.
//!
/// For development: generates deterministic pseudo-random vectors.
/// For production: swap in ONNX Runtime models (e.g., all-MiniLM-L6-v2).
use super::provider::{EmbeddingProvider, EmbeddingResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Local embedding provider using deterministic pseudo-random vectors.
pub struct LocalEmbeddingProvider {
    dimensions: usize,
    seed_cache: Arc<Mutex<HashMap<String, Vec<f32>>>>,
}

impl LocalEmbeddingProvider {
    pub fn new() -> Self {
        Self {
            dimensions: 384,
            seed_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a local provider with custom dimensions.
    pub fn with_dimensions(dimensions: usize) -> Self {
        Self {
            dimensions,
            seed_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate a deterministic pseudo-random vector from a seed string.
    fn generate_vector(&self, text: &str) -> Vec<f32> {
        let mut vector = Vec::with_capacity(self.dimensions);

        for i in 0..self.dimensions {
            let hash = self.hash_seed(&format!("{}:{}", text, i));
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

        let cache = self.seed_cache.clone();
        let dimensions = self.dimensions;
        let vector = {
            let mut map = cache.lock().unwrap();
            if let Some(cached) = map.get(text) {
                cached.clone()
            } else {
                let vector = self.generate_vector(text);
                let v_clone = vector.clone();
                map.insert(text.to_string(), vector);
                v_clone
            }
        };

        let result = EmbeddingResult { vector, dimensions };

        debug!(
            text_len = text.len(),
            dimensions = self.dimensions,
            "Local embedding generated"
        );
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
        let cache = self.seed_cache.clone();
        let dimensions = self.dimensions;
        for text in &texts {
            let text_ref = *text;
            let vector = {
                let mut map = cache.lock().unwrap();
                if let Some(cached) = map.get(text_ref) {
                    cached.clone()
                } else {
                    let vector = self.generate_vector(text_ref);
                    let v_clone = vector.clone();
                    map.insert(text_ref.to_string(), vector);
                    v_clone
                }
            };
            results.push(EmbeddingResult { vector, dimensions });
        }
        Ok(results)
    }
}

impl Default for LocalEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}
