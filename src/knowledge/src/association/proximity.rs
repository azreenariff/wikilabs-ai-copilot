//! Semantic proximity — compute and rank proximity between knowledge chunks.

use super::{DiscoveryMethod, EdgeType, Relationship, Weight};
use crate::embedding_pipeline::cosine_similarity;
use crate::retrieval::chunker::KnowledgeChunk;
use tracing::debug;

/// Proximity between two knowledge chunks.
#[derive(Debug, Clone)]
pub struct ProximityScore {
    pub source_id: String,
    pub target_id: String,
    pub semantic_score: f32,
    pub lexical_score: f32,
    pub structural_score: f32,
    pub combined_score: f32,
    pub relationship: Option<EdgeType>,
}

/// Semantic proximity engine.
pub struct SemanticProximity {
    pub threshold: f32,
}

impl SemanticProximity {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    /// Compute proximity between two chunks.
    pub fn compute_proximity(&self, a: &KnowledgeChunk, b: &KnowledgeChunk) -> ProximityScore {
        let semantic = if let (Some(ref vec_a), Some(ref vec_b)) = (&a.embedding, &b.embedding) {
            cosine_similarity(vec_a, vec_b)
        } else {
            0.0
        };

        let lexical = self.lexical_similarity(&a.text, &b.text);
        let structural = self.structural_similarity(&a.metadata, &b.metadata);
        let combined = semantic * 0.6 + lexical * 0.25 + structural * 0.15;

        let relationship = self.infer_relationship(&a.metadata, &b.metadata);

        ProximityScore {
            source_id: a.id.clone(),
            target_id: b.id.clone(),
            semantic_score: semantic,
            lexical_score: lexical,
            structural_score: structural,
            combined_score: combined,
            relationship,
        }
    }

    /// Compute proximity for all pairs in a list (O(n²) — use for small sets).
    pub fn compute_all_proximities(&self, chunks: &[KnowledgeChunk]) -> Vec<ProximityScore> {
        let mut scores = Vec::new();
        for i in 0..chunks.len() {
            for j in (i + 1)..chunks.len() {
                let score = self.compute_proximity(&chunks[i], &chunks[j]);
                if score.combined_score >= self.threshold {
                    scores.push(score);
                }
            }
        }
        scores.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scores
    }

    /// Convert proximity scores to relationships.
    pub fn to_relationships(&self, scores: &[ProximityScore]) -> Vec<Relationship> {
        scores
            .iter()
            .filter(|s| s.combined_score >= self.threshold)
            .map(|s| {
                let edge_type = match s.relationship {
                    Some(ref etype) => etype.clone(),
                    None => EdgeType::Related,
                };
                let weight = if s.combined_score > 0.8 {
                    Weight::Strong
                } else if s.combined_score > 0.5 {
                    Weight::Moderate
                } else {
                    Weight::Weak
                };
                Relationship {
                    source_id: s.source_id.clone(),
                    target_id: s.target_id.clone(),
                    edge_type,
                    weight,
                    confidence: s.combined_score,
                    discovery_method: DiscoveryMethod::Automatic,
                    metadata: serde_json::json!({
                        "semantic_score": s.semantic_score,
                        "lexical_score": s.lexical_score,
                        "structural_score": s.structural_score,
                    }),
                    created_at: chrono::Utc::now(),
                }
            })
            .collect()
    }

    /// Simple lexical similarity (Jaccard-like on word sets).
    fn lexical_similarity(&self, text_a: &str, text_b: &str) -> f32 {
        let words_a: std::collections::HashSet<String> = text_a
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let words_b: std::collections::HashSet<String> = text_b
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if words_a.is_empty() && words_b.is_empty() {
            return 1.0;
        }
        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        intersection as f32 / union as f32
    }

    /// Structural similarity based on metadata overlap.
    fn structural_similarity(&self, meta_a: &serde_json::Value, meta_b: &serde_json::Value) -> f32 {
        let a_pack = meta_a.get("knowledge_pack").and_then(|v| v.as_str());
        let b_pack = meta_b.get("knowledge_pack").and_then(|v| v.as_str());

        if a_pack.is_some() && a_pack == b_pack {
            0.3
        } else {
            0.0
        }
    }

    /// Infer relationship type from metadata.
    fn infer_relationship(
        &self,
        meta_a: &serde_json::Value,
        meta_b: &serde_json::Value,
    ) -> Option<EdgeType> {
        let a_type = meta_a
            .get("node_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let b_type = meta_b
            .get("node_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if a_type.contains("spec") && b_type.contains("impl") {
            Some(EdgeType::Specification)
        } else if a_type.contains("impl") && b_type.contains("spec") {
            Some(EdgeType::Implementation)
        } else {
            Some(EdgeType::Related)
        }
    }
}
