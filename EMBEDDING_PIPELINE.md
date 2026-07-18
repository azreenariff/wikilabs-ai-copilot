# Embedding Pipeline Specification

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

The Embedding Pipeline transforms document chunks into vector embeddings for semantic search. The embedding architecture is provider-independent, model-configurable, and designed for future extensibility.

Embedding generation is independent from the AI reasoning model — different embedding models can be used regardless of which LLM powers the copilot.

## Architecture

```
Chunked Document → [Provider Abstraction] → [Model Selection] → [Batch Processing] → [Embedding Generation] → [Version Tracking] → Vector Store
```

## Provider Trait

```rust
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Embedding dimensions
    fn dimension(&self) -> usize;

    /// Supported batch size
    fn batch_size(&self) -> usize;

    /// Generate embeddings for a batch of texts
    async fn embed(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>>;

    /// Generate embedding for a single text
    async fn embed_single(&self, text: String) -> anyhow::Result<Vec<f32>>;

    /// Supported languages
    fn supported_languages(&self) -> Vec<String>;

    /// Maximum token/text length
    fn max_length(&self) -> usize;

    /// Provider capabilities
    fn capabilities(&self) -> Vec<EmbeddingCapability>;
}
```

## Embedding Capabilities

```rust
pub enum EmbeddingCapability {
    BatchGeneration,
    IncrementalGeneration,
    Multilingual,
    CustomModel,
    Quantization,
    Asynchronous,
}
```

## Embedding Models

### Local Embedding Provider

Uses local ONNX models for embedding generation.

```rust
pub struct LocalEmbeddingProvider {
    model_name: String,
    model: ONNXModel,
    dimension: usize,
    tokenizer: Tokenizer,
    batch_size: usize,
    device: Device,
}
```

Supported local models:

- `all-MiniLM-L6-v2` — 384 dimensions (default)
- `all-MiniLM-L12-v2` — 384 dimensions
- `all-mpnet-base-v2` — 768 dimensions
- `sentence-transformers/all-MiniLM-L6-v2` — 384 dimensions

### Provider Factory

```rust
pub struct EmbeddingProviderFactory;

impl EmbeddingProviderFactory {
    pub fn create(provider_type: EmbeddingProviderType, config: EmbeddingConfig) -> anyhow::Result<Box<dyn EmbeddingProvider>>;
    pub fn available_providers() -> Vec<String>;
}

pub enum EmbeddingProviderType {
    Local,
    OpenAI,
    Anthropic,
    Custom(String),
}
```

## Embedding Configuration

```rust
pub struct EmbeddingConfig {
    pub provider: EmbeddingProviderType,
    pub model_name: String,
    pub dimension: usize,
    pub batch_size: usize,
    pub max_length: usize,
    pub truncate: bool,
    pub normalize: bool,
    pub precision: EmbeddingPrecision,
    pub cache_size: usize,
    pub timeout: Duration,
    pub retry_count: usize,
}

pub enum EmbeddingPrecision {
    FP32,
    FP16,
    INT8,
}
```

## Embedding Pipeline

```rust
pub struct EmbeddingPipeline {
    pub provider: Box<dyn EmbeddingProvider>,
    pub config: EmbeddingConfig,
    pub vector_store: Box<dyn VectorStorage>,
    pub cache: EmbeddingCache,
    pub version_tracker: EmbeddingVersionTracker,
}

impl EmbeddingPipeline {
    pub fn new(config: EmbeddingConfig, vector_store: Box<dyn VectorStorage>) -> Self;
    pub async fn embed_chunks(&self, chunks: Vec<DocumentChunk>) -> anyhow::Result<EmbeddingResult>;
    pub async fn embed_batch(&self, chunks: &[DocumentChunk], batch_size: usize) -> anyhow::Result<Vec<Vec<f32>>>;
    pub async fn reindex(&self, pack_name: &str) -> anyhow::Result<ReindexResult>;
}
```

## Batch Processing

Embedding is done in batches for efficiency:

```rust
pub struct BatchProcessor {
    pub batch_size: usize,
    pub parallel_workers: usize,
    pub max_memory: usize,
}

impl BatchProcessor {
    pub fn process(&self, chunks: &[DocumentChunk]) -> Vec<Vec<String>> {
        chunks
            .chunks(self.batch_size)
            .map(|batch| batch.iter().map(|c| c.content.clone()).collect())
            .collect()
    }
}
```

## Incremental Embedding

Only generates embeddings for new or changed documents:

```rust
pub struct IncrementalEmbedder {
    pub existing_embeddings: EmbeddingRegistry,
    pub change_tracker: ChangeTracker,
}

pub struct IncrementalEmbeddingResult {
    pub new_embeddings: usize,
    pub updated_embeddings: usize,
    pub unchanged_skipped: usize,
    pub errors: Vec<String>,
}
```

Change detection:

1. Compare content hash vs stored hash
2. Compare embedding version vs current model version
3. Flag documents needing re-embedding
4. Only re-embed changed documents

## Embedding Versioning

Tracks which embedding model generated which embeddings:

```rust
pub struct EmbeddingVersionTracker {
    pub version_history: Vec<EmbeddingVersion>,
    pub current_version: String,
}

pub struct EmbeddingVersion {
    pub version: String,
    pub model_name: String,
    pub dimension: usize,
    pub provider: String,
    pub created_at: DateTime<Utc>,
    pub document_count: usize,
    pub checksum: String,
}
```

Version management:

- New model = new version
- Re-embedding with same model = same version
- Old versions kept for rollback support
- Version migration path for model upgrades

## Embedding Cache

Caches embeddings to avoid regeneration:

```rust
pub struct EmbeddingCache {
    pub cache: LRUCache<String, Vec<f32>>,
    pub hash_store: HashMap<String, String>, // content_hash → cache_key
    pub max_size: usize,
}

impl EmbeddingCache {
    pub fn get(&self, text: &str) -> Option<Vec<f32>>;
    pub fn put(&self, text: &str, embedding: Vec<f32>);
    pub fn invalidate(&mut self, text: &str);
}
```

## Embedding Result

```rust
pub struct EmbeddingResult {
    pub embeddings: Vec<IndexedEmbedding>,
    pub total_processed: usize,
    pub total_success: usize,
    pub total_failed: usize,
    pub errors: Vec<EmbeddingError>,
    pub duration: Duration,
    pub version: String,
    pub model_name: String,
}

pub struct IndexedEmbedding {
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub embedding: Vec<f32>,
    pub title: String,
    pub content: String,
    pub namespace: String,
    pub workspace_id: Uuid,
    pub pack_name: String,
}
```

## Embedding Errors

```rust
#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Provider not available: {provider}")]
    ProviderUnavailable { provider: String },

    #[error("Model loading failed: {reason}")]
    ModelLoadFailed { reason: String },

    #[error("Embedding generation failed for {n} texts")]
    GenerationFailed { count: usize, reason: String },

    #[error("Text too long: {length} > {max_length}")]
    TextTooLong { length: usize, max_length: usize },

    #[error("Batch size exceeded: {batch_size} > {max_batch}")]
    BatchSizeExceeded { batch_size: usize, max_batch: usize },

    #[error("Memory limit exceeded: {limit} bytes")]
    MemoryLimitExceeded { limit: usize },

    #[error("Provider error: {reason}")]
    ProviderError { provider: String, reason: String },
}
```

## Reindexing

Full reindex regenerates all embeddings:

```rust
pub struct ReindexResult {
    pub pack_name: String,
    pub total_documents: usize,
    pub total_chunks: usize,
    pub embeddings_generated: usize,
    pub embeddings_failed: usize,
    pub duration: Duration,
    pub previous_version: String,
    pub new_version: String,
}
```

Reindexing steps:

1. Load all chunks from pack
2. Check out existing embeddings
3. Generate new embeddings
4. Update vector store atomically
5. Update version tracking
6. Report results

## Performance

### Batch Processing

- Batches of configurable size (default 100)
- Parallel processing where supported
- Memory usage bounded by batch size

### Caching

- Content-based hashing
- LRU eviction
- Configurable cache size

### Incremental Processing

- Hash-based change detection
- Only processes new/changed documents
- Preserves existing embeddings

### Asynchronous

- Non-blocking generation
- Progress reporting
- Cancellation support

## Model Selection

Model selection considers:

1. Dimension compatibility with vector store
2. Language support
3. Quality vs performance tradeoff
4. Hardware constraints
5. Deployment environment

Default model: `all-MiniLM-L6-v2` (384 dimensions)

## Future Providers

- OpenAI embeddings (text-embedding-3-small, text-embedding-3-large)
- Cohere embeddings
- Hugging Face inference API
- Custom ONNX models
- Local GGML embeddings