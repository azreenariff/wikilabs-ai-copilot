//! Embedding Pipeline — vector embedding generation and management.
//!
//! Provider abstraction for embedding models, batch generation,
//! incremental updates, and embedding versioning.
//!
//! Embedding generation is independent from the AI reasoning model.

pub mod provider;
pub mod batch;
pub mod incremental;

pub use provider::{EmbeddingProvider, EmbeddingProviderRegistry, LocalEmbeddingProvider};
pub use batch::BatchEmbedder;
pub use incremental::IncrementalEmbedder;

/// Embedding configuration for a pack or document batch.
#[derive(Debug, Clone)]
pub struct EmbeddingPipelineConfig {
    pub provider: String,
    pub model: String,
    pub dimensions: usize,
    pub batch_size: usize,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub normalize: bool,
    pub caching: bool,
}

impl Default for EmbeddingPipelineConfig {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            model: "all-MiniLM-L6-v2".to_string(),
            dimensions: 384,
            batch_size: 32,
            max_retries: 3,
            retry_delay_ms: 100,
            normalize: true,
            caching: true,
        }
    }
}

/// Status of embedding generation for a document or pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbeddingStatus {
    Pending,
    Generating,
    Completed,
    Failed(String),
    Skipped,
}

/// Result of embedding a single chunk.
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub chunk_id: String,
    pub vector: Vec<f32>,
    pub model: String,
    pub dimensions: usize,
    pub status: EmbeddingStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Overall embedding pipeline result.
#[derive(Debug, Clone)]
pub struct EmbeddingPipelineResult {
    pub total_chunks: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub results: Vec<EmbeddingResult>,
}

/// Normalize a vector to unit length.
pub fn normalize_vector(mut vec: Vec<f32>) -> Vec<f32> {
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