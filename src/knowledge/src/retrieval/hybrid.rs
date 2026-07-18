//! Hybrid retrieval — combines vector similarity with full-text search.

use super::chunker::KnowledgeChunk;
use super::{RetrievedChunk, RetrievalFilter, RetrievalResult, RelevanceLevel};
use crate::storage::vector_store::VectorStore;
use crate::storage::namespace::NamespaceManager;
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use tracing::info;

/// Combines vector and full-text search for best results.
pub struct HybridRetriever {
    vector_store: VectorStore,
    namespace_mgr: NamespaceManager,
    vector_weight: f32,
    fts_weight: f32,
    min_score: f32,
}

impl HybridRetriever {
    pub fn new(vector_store: VectorStore, namespace_mgr: NamespaceManager) -> Self {
        Self {
            vector_store,
            namespace_mgr,
            vector_weight: 0.7,
            fts_weight: 0.3,
            min_score: 0.1,
        }
    }

    pub fn with_weights(mut self, vector: f32, fts: f32) -> Self {
        self.vector_weight = vector;
        self.fts_weight = fts;
        self
    }

    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = score;
        self
    }

    /// Hybrid search combining vector and FTS.
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

        // Filter criteria
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

        // Vector search
        let vector_results = self
            .vector_store
            .search(namespace.id, query_vector, filter.top_k.unwrap_or(10), Some(filter_vec.clone()))
            .await?;

        // FTS search (fallback: simple text match)
        let fts_results = self.search_fts(query, namespace.id, filter.top_k.unwrap_or(10)).await?;

        // Merge and score
        let mut scored_results: std::collections::HashMap<String, (f32, RetrievedChunk)> = std::collections::HashMap::new();

        for (chunk_id, doc_id, text, section, heading_context) in &vector_results {
            let score = self.vector_weight * 0.7; // vector score placeholder
            let chunk = RetrievedChunk {
                chunk_id: chunk_id.clone(),
                document_id: doc_id.clone(),
                text: text.clone(),
                heading_context: heading_context.clone(),
                section: section.clone(),
                metadata: json!({
                    "chunk_id": chunk_id,
                    "document_id": doc_id,
                    "knowledge_pack": knowledge_pack,
                }),
                similarity_score: score,
                source_file: knowledge_pack.to_string(),
                relevance: RelevanceLevel::High,
            };
            scored_results.insert(chunk_id.clone(), (score, chunk));
        }

        for (chunk_id, doc_id, text, score, _) in &fts_results {
            let existing = scored_results.entry(chunk_id.clone()).or_insert((0.0, self.chunk_from_fts(chunk_id, doc_id, text)));
            let combined_score = (existing.0 * self.vector_weight + (*score * self.fts_weight)).min(1.0);
            existing.0 = combined_score;
        }

        // Convert to sorted vector
        let mut chunks: Vec<RetrievedChunk> = scored_results
            .into_values()
            .filter(|(score, _)| *score >= self.min_score)
            .map(|(_, chunk)| {
                let relevance = match chunk.similarity_score {
                    s if s > 0.8 => RelevanceLevel::Exact,
                    s if s > 0.6 => RelevanceLevel::High,
                    s if s > 0.4 => RelevanceLevel::Moderate,
                    _ => RelevanceLevel::Low,
                };
                let mut chunk = chunk;
                chunk.relevance = relevance;
                chunk
            })
            .collect();

        chunks.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));

        let duration_ms = Utc::now() - start;
        let duration_ms = duration_ms.num_milliseconds() as u64;

        info!(
            query_length = query.len(),
            results_count = chunks.len(),
            duration_ms,
            "Hybrid search complete"
        );

        Ok(RetrievalResult {
            query: query.to_string(),
            chunks,
            total_candidates: scored_results.len(),
            filter_applied: !filter_vec.is_empty(),
            duration_ms,
            retrieval_strategy: "hybrid_vector_fts".to_string(),
        })
    }

    /// FTS search (simple text match fallback).
    async fn search_fts(&self, query: &str, namespace_id: i64, top_k: usize) -> Result<Vec<(String, String, String, f32, Option<String>)>> {
        // Simple fallback: return empty results
        // In production, this would query the FTS5 index
        Ok(Vec::new())
    }

    fn chunk_from_fts(&self, chunk_id: &str, doc_id: &str, text: &str) -> RetrievedChunk {
        RetrievedChunk {
            chunk_id: chunk_id.to_string(),
            document_id: doc_id.to_string(),
            text: text.to_string(),
            heading_context: None,
            section: None,
            metadata: json!({
                "chunk_id": chunk_id,
                "document_id": doc_id,
                "knowledge_pack": "",
            }),
            similarity_score: 0.3,
            source_file: doc_id.to_string(),
            relevance: RelevanceLevel::FtsMatch,
        }
    }
}