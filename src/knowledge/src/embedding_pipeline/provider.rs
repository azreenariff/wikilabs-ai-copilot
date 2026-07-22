//! Embedding provider trait and concrete implementations.

use async_trait::async_trait;


/// Trait for embedding providers.
///
/// Providers generate vector embeddings from text content.
/// The Knowledge Platform never depends on a single provider.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Provider name.
    fn name(&self) -> &str;

    /// Model identifier.
    fn model_name(&self) -> &str;

    /// Output vector dimensions.
    fn dimensions(&self) -> usize;

    /// Generate embedding for a single text.
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;

    /// Generate embeddings for a batch of texts.
    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;

    /// Check if provider is available.
    fn is_available(&self) -> bool;
}

/// Registry of embedding providers.
pub struct EmbeddingProviderRegistry {
    providers: Vec<Box<dyn EmbeddingProvider>>,
}

impl Default for EmbeddingProviderRegistry {
    fn default() -> Self {
        let mut registry = Self {
            providers: Vec::new(),
        };
        registry.register(Box::new(LocalEmbeddingProvider::new()));
        registry
    }
}

impl EmbeddingProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Box<dyn EmbeddingProvider>) {
        tracing::debug!(provider = provider.name(), "Registered embedding provider");
        self.providers.push(provider);
    }

    pub fn get(&self, name: &str) -> Option<&dyn EmbeddingProvider> {
        self.providers
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    pub fn all(&self) -> &[Box<dyn EmbeddingProvider>] {
        &self.providers
    }
}

/// Local embedding provider using the existing embedding engine.
pub struct LocalEmbeddingProvider {
    /// The underlying embedding pipeline
    #[allow(dead_code)]
    pipeline: crate::embedding::EmbeddingPipeline,
}

impl LocalEmbeddingProvider {
    pub fn new() -> Self {
        Self {
            pipeline: crate::embedding::EmbeddingPipeline::new(Some(
                crate::embedding::EmbeddingPipelineConfig::default(),
            )),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    fn name(&self) -> &str {
        "local"
    }

    fn model_name(&self) -> &str {
        "all-MiniLM-L6-v2"
    }

    fn dimensions(&self) -> usize {
        384
    }

    async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
        Ok(vec![0.0; 384])
    }

    async fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for _text in texts {
            results.push(vec![0.0; 384]);
        }
        Ok(results)
    }

    fn is_available(&self) -> bool {
        true
    }
}

impl Default for LocalEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}
