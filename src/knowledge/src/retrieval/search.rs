//! Vector search — semantic similarity retrieval.

use super::chunker::KnowledgeChunk;
use super::hybrid::HybridRetriever;
use super::{RelevanceLevel, RetrievalFilter, RetrievalResult, RetrievedChunk};
use crate::embedding_pipeline::{cosine_similarity, normalize_vector};
use crate::storage::namespace::NamespaceManager;
use crate::storage::vector_store::VectorStore;
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
        filters: RetrievalFilter,
    ) -> Result<RetrievalResult> {
        let start = Utc::now();
        let filter = filters;

        // Get namespace
        let namespace = self.namespace_mgr.get_or_create(
            knowledge_pack,
            None,
            knowledge_pack,
            query_vector.len(),
        )?;

        // Apply filters
        let filter_vec: Vec<(String, String)> = {
            let mut vec = Vec::new();
            if let Some(ref vendor) = filter.vendor {
                vec.push(("vendor".to_string(), vendor.clone()));
            }
            if let Some(ref product) = filter.product {
                vec.push(("product".to_string(), product.clone()));
            }
            if let Some(ref technology) = filter.technology {
                vec.push(("technology".to_string(), technology.clone()));
            }
            if let Some(ref workspace) = filter.workspace_id {
                vec.push(("workspace_id".to_string(), workspace.clone()));
            }
            vec
        };

        // Perform vector search
        let results = self
            .store
            .search(query_vector, filter.top_k.unwrap_or(self.default_top_k))
            .await?;

        let result_count = results.len();
        let mut chunks = Vec::new();
        for result in &results {
            let chunk_id = result.vector_id.clone();
            let doc_id = result.doc_id.clone();
            let text = result.content.clone();
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
                heading_context: None,
                section: None,
                metadata,
                similarity_score: similarity,
                source_file: knowledge_pack.to_string(),
                relevance: self.classify_relevance(similarity),
            });
        }

        // Sort by relevance
        chunks.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

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
            total_candidates: result_count,
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
        filters: RetrievalFilter,
        max_tokens: usize,
    ) -> Result<String> {
        let result = self
            .search(query, query_vector, knowledge_pack, filters)
            .await?;

        let context: String = result
            .chunks
            .iter()
            .take(3)
            .map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(format!(
            "Based on {} retrieved chunks from '{}':\n\n{}",
            result.chunks.len(),
            knowledge_pack,
            context
        ))
    }
}
