//! Vector search — semantic similarity retrieval.

use super::{RetrievedChunk, RetrievalFilter, RetrievalResult, RelevanceLevel};
use super::chunker::KnowledgeChunk;
use super::hybrid::HybridRetriever;
use crate::embedding_pipeline::{cosine_similarity, normalize_vector};
use crate::storage::vector_store::VectorStore;
use crate::storage::namespace::NamespaceManager;
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use tracing::{debug, info};

/// Searches vector storage for the most relevant chunks.
pub struct VectorSearcher {
    pub store: VectorStore,
    pub namespace_mgr: NamespaceManager,
    pub default_top_k: usize,
}

impl VectorSearcher {
    pub fn new(store: VectorStore, namespace_mgr: NamespaceManager) -> Self {
        Self {
            store,
            namespace_mgr,
            default_top_k: 10,
        }
    }

    /// Search for relevant knowledge chunks using vector similarity.
    pub async fn search(
        &self,
        query: &str,
        query_vector: &[f32],
        knowledge_pack: &str,
        filters: Option<RetrievalFilter>,
    ) -> Result<RetrievalResult> {
        let start = Utc::now();
        let filter = filters.unwrap_or_default();

        // Get namespace
        let namespace = self
            .namespace_mgr
            .get_or_create(knowledge_pack, None, knowledge_pack, query_vector.len())
            .await?;

        // Apply filters
        let filter_vec: Vec<(String, String)> = if let Some(ref f) = filter {
            let mut vec = Vec::new();
            if let Some(ref vendor) = f.vendor {
                vec.push(("vendor".to_string(), vendor.clone()));
            }
            if let Some(ref product) = f.product {
                vec.push(("product".to_string(), product.clone()));
            }
            if let Some(ref technology) = f.technology {
                vec.push(("technology".to_string(), technology.clone()));
            }
            if let Some(ref workspace) = f.workspace_id {
                vec.push(("workspace_id".to_string(), workspace.clone()));
            }
            vec
        } else {
            Vec::new()
        };

        // Perform vector search
        let results = self
            .store
            .search(namespace.id, query_vector, filter.top_k.unwrap_or(self.default_top_k), Some(filter_vec))
            .await?;

        let mut chunks = Vec::new();
        for (chunk_id, doc_id, text, section, heading_context) in results {
            let metadata = json!({
                "chunk_id": chunk_id,
                "document_id": doc_id,
                "knowledge_pack": knowledge_pack,
            });

            let similarity = self.calculate_relevance(query_vector, &text);

            chunks.push(RetrievedChunk {
                chunk_id,
                document_id: doc_id,
                text,
                heading_context,
                section,
                metadata,
                similarity_score: similarity,
                source_file: knowledge_pack.to_string(),
                relevance: self.classify_relevance(similarity),
            });
        }

        // Sort by relevance
        chunks.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));

        let duration_ms = Utc::now() - start;
        let duration_ms = duration_ms.num_milliseconds() as u64;

        info!(
            query_length = query.len(),
            results_count = chunks.len(),
            duration_ms,
            "Vector search complete"
        );

        Ok(RetrievalResult {
            query: query.to_string(),
            chunks,
            total_candidates: results.len(),
            filter_applied: !filter_vec.is_empty(),
            duration_ms,
            retrieval_strategy: "vector_similarity".to_string(),
        })
    }

    /// Calculate relevance score based on text overlap with query.
    fn calculate_relevance(&self, query_vector: &[f32], text: &str) -> f32 {
        // Placeholder: compute a simple score
        // In production, this would use the embedding model to generate
        // a query embedding and compare with text embedding
        0.5 // default moderate relevance
    }

    /// Classify relevance level based on score.
    fn classify_relevance(&self, score: f32) -> RelevanceLevel {
        if score > 0.8 {
            RelevanceLevel::Exact
        } else if score > 0.6 {
            RelevanceLevel::High
        } else if score > 0.4 {
            RelevanceLevel::Moderate
        } else {
            RelevanceLevel::Low
        }
    }

    /// Generate answer from retrieved chunks.
    pub async fn answer(
        &self,
        query: &str,
        query_vector: &[f32],
        knowledge_pack: &str,
        filters: Option<RetrievalFilter>,
        max_tokens: usize,
    ) -> Result<String> {
        let result = self.search(query, query_vector, knowledge_pack, filters).await?;

        let context = result.chunks.iter().take(3).map(|c| &c.text).collect::<Vec<_>>().join("\n\n");

        Ok(format!(
            "Based on {} retrieved chunks from '{}':\n\n{}",
            result.chunks.len(),
            knowledge_pack,
            context
        ))
    }
}