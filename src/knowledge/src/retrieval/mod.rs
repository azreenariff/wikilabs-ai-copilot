//! Retrieval — vector search, full-text search, chunking, and answer generation.
//!
//! Combines semantic similarity with full-text search to find the most relevant
//! Retrieval — chunking, vector search, hybrid search, and answer generation.

pub mod chunker;
pub mod search;
pub mod hybrid;

pub use chunker::{Chunker, ChunkStrategy};
pub use search::VectorSearcher;
pub use hybrid::HybridRetriever;

/// Filter criteria for retrieval.
#[derive(Debug, Clone)]
pub struct RetrievalFilter {
    pub knowledge_pack: Option<String>,
    pub vendor: Option<String>,
    pub product: Option<String>,
    pub technology: Option<String>,
    pub workspace_id: Option<String>,
    pub min_similarity: Option<f32>,
    pub top_k: Option<usize>,
}

impl Default for RetrievalFilter {
    fn default() -> Self {
        Self {
            knowledge_pack: None,
            vendor: None,
            product: None,
            technology: None,
            workspace_id: None,
            min_similarity: Some(0.3),
            top_k: Some(10),
        }
    }
}

/// Result of a retrieval operation.
#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    pub chunk_id: String,
    pub document_id: String,
    pub text: String,
    pub heading_context: Option<String>,
    pub section: Option<String>,
    pub metadata: serde_json::Value,
    pub similarity_score: f32,
    pub source_file: String,
    pub relevance: RelevanceLevel,
}

/// Relevance level for retrieved chunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelevanceLevel {
    /// Exact match (heading/title).
    Exact,
    /// High semantic similarity.
    High,
    /// Moderate similarity.
    Moderate,
    /// Low similarity.
    Low,
    /// Full-text match only.
    FtsMatch,
}

/// Aggregated retrieval result with metadata.
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub query: String,
    pub chunks: Vec<RetrievedChunk>,
    pub total_candidates: usize,
    pub filter_applied: bool,
    pub duration_ms: u64,
    pub retrieval_strategy: String,
}