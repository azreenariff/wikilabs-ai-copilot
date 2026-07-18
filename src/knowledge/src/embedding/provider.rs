//! Embedding provider abstraction.

use async_trait::async_trait;
use thiserror::Error;
use tracing::debug;

/// Custom error types for embedding operations.
#[derive(Debug, Error)]
pub enum EmbeddingPipelineError {
    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Invalid dimensions: expected {expected}, got {actual}")]
    InvalidDimensions { expected: usize, actual: usize },

    #[error("Embedding generation failed: {0}")]
    GenerationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result of an embedding generation.
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub vector: Vec<f32>,
    pub dimensions: usize,
}

/// Trait for embedding providers.
///
/// Implement this trait to support different embedding models:
/// - Local (e.g., ONNX models, random for dev)
/// - Remote (e.g., OpenAI, HuggingFace)
/// - Custom (e.g., fine-tuned local models)
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for the given text.
    async fn embed(&self, text: &str) -> anyhow::Result<EmbeddingResult>;

    /// Get the number of dimensions for the embedding model.
    fn dimensions(&self) -> usize;

    /// Get the name/identifier of the embedding model.
    fn name(&self) -> String;

    /// Get the provider type for configuration.
    fn provider_type(&self) -> &str;

    /// Embed a batch of texts (optional optimization).
    async fn embed_batch(&self, texts: Vec<&str>) -> anyhow::Result<Vec<EmbeddingResult>> {
        // Default: embed individually
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }
}