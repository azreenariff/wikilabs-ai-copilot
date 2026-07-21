//! Context-aware retrieval — multi-factor ranking for relevance scoring.
//!
//! Ranks retrieved chunks using multiple context signals:
//! - **Technology match**: Boost for chunks mentioning relevant tech stacks
//! - **Intent match**: Boost based on query intent classification
//! - **Workspace relevance**: Boost for same-workspace documents
//! - **Conversation context**: Boost for chunks related to recent conversation history
//!
//! Combines vector similarity with contextual scoring for smarter results.

use crate::retrieval::{RelevanceLevel, RetrievedChunk};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Context signals available for ranking.
#[derive(Debug, Clone, Default)]
pub struct RetrievalContext {
    /// Technologies relevant to the current task (e.g., ["rust", "docker", "aws"]).
    pub technologies: Vec<String>,
    /// Detected user intent from the query.
    pub intent: Intent,
    /// Workspace ID being queried.
    pub workspace_id: Option<String>,
    /// Recent conversation history for contextual boosting.
    pub conversation_history: Vec<String>,
    /// User's role/team (e.g., "frontend", "devops", "backend").
    pub user_role: Option<String>,
    /// Priority keywords extracted from conversation.
    pub priority_keywords: Vec<String>,
}

/// Detected intent from a query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Intent {
    /// Looking for documentation/instructions.
    Documentation,
    /// Searching for code/example snippets.
    CodeExample,
    /// Debugging an error/problem.
    Debugging,
    /// Understanding a concept/architecture.
    ConceptExplainer,
    /// Comparing options/approaches.
    Comparison,
    /// General knowledge query.
    #[default]
    General,
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intent::Documentation => write!(f, "documentation"),
            Intent::CodeExample => write!(f, "code_example"),
            Intent::Debugging => write!(f, "debugging"),
            Intent::ConceptExplainer => write!(f, "concept_explainer"),
            Intent::Comparison => write!(f, "comparison"),
            Intent::General => write!(f, "general"),
        }
    }
}

impl Intent {
    /// Detect intent from query text (heuristic).
    pub fn detect_from_query(query: &str) -> Self {
        let lower = query.to_lowercase();

        if lower.contains("error")
            || lower.contains("fix")
            || lower.contains("bug")
            || lower.contains("issue")
            || lower.contains("troubleshoot")
            || lower.contains("why")
        {
            return Intent::Debugging;
        }
        if lower.contains("how to")
            || lower.contains("tutorial")
            || lower.contains("guide")
            || lower.contains("learn")
            || lower.contains("explain")
        {
            return Intent::Documentation;
        }
        if lower.contains("code")
            || lower.contains("example")
            || lower.contains("snippet")
            || lower.contains("implementation")
        {
            return Intent::CodeExample;
        }
        if lower.contains("vs")
            || lower.contains("versus")
            || lower.contains("compare")
            || lower.contains("difference")
            || lower.contains("better")
            || lower.contains("should i use")
        {
            return Intent::Comparison;
        }
        if lower.contains("what is")
            || lower.contains("concept")
            || lower.contains("architecture")
            || lower.contains("design pattern")
        {
            return Intent::ConceptExplainer;
        }

        Intent::General
    }
}

/// Context-aware retriever — applies multi-factor ranking to retrieved chunks.
pub struct ContextAwareRetriever {
    /// Weights for each ranking factor.
    weights: RankingWeights,
    /// Minimum combined score threshold.
    min_score: f32,
}

/// Configurable weights for ranking factors.
#[derive(Debug, Clone)]
pub struct RankingWeights {
    /// Weight for technology match (default: 0.25).
    pub tech_match: f32,
    /// Weight for intent match (default: 0.20).
    pub intent_match: f32,
    /// Weight for workspace relevance (default: 0.20).
    pub workspace_relevance: f32,
    /// Weight for conversation context (default: 0.15).
    pub conversation_context: f32,
    /// Weight for keyword overlap (default: 0.20).
    pub keyword_overlap: f32,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            tech_match: 0.25,
            intent_match: 0.20,
            workspace_relevance: 0.20,
            conversation_context: 0.15,
            keyword_overlap: 0.20,
        }
    }
}

impl ContextAwareRetriever {
    pub fn new() -> Self {
        Self {
            weights: RankingWeights::default(),
            min_score: 0.1,
        }
    }

    pub fn with_weights(mut self, weights: RankingWeights) -> Self {
        self.weights = weights;
        self
    }

    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = score;
        self
    }

    /// Rank retrieved chunks using context-aware scoring.
    pub fn rank_chunks(
        &self,
        chunks: Vec<RetrievedChunk>,
        context: &RetrievalContext,
        target_workspace: Option<&str>,
    ) -> Vec<RetrievedChunk> {
        let mut ranked = Vec::new();

        for mut chunk in chunks {
            // Compute multi-factor score
            let tech_score = self.compute_tech_match_score(&chunk, context);
            let intent_score = self.compute_intent_match_score(&chunk, context);
            let workspace_score = self.compute_workspace_score(&chunk, target_workspace);
            let conversation_score = self.compute_conversation_score(&chunk, context);
            let keyword_score = self.compute_keyword_overlap(&chunk, context);

            // Weighted combination
            let combined = chunk.similarity_score * 0.4
                + tech_score * self.weights.tech_match
                + intent_score * self.weights.intent_match
                + workspace_score * self.weights.workspace_relevance
                + conversation_score * self.weights.conversation_context
                + keyword_score * self.weights.keyword_overlap;

            chunk.similarity_score = combined;
            chunk.relevance = self.classify_relevance(combined);

            ranked.push(chunk);
        }

        // Sort by combined score descending
        ranked.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Filter below minimum score
        ranked.retain(|c| c.similarity_score >= self.min_score);

        debug!(total = ranked.len(), "Context-aware ranking complete");

        ranked
    }

    /// Score based on technology tag overlap.
    fn compute_tech_match_score(&self, chunk: &RetrievedChunk, context: &RetrievalContext) -> f32 {
        if context.technologies.is_empty() {
            return 0.0;
        }

        let chunk_text_lower = chunk.text.to_lowercase();
        let mut matches = 0;

        for tech in &context.technologies {
            let tech_lower = tech.to_lowercase();
            // Check for exact word match or common prefixes
            if chunk_text_lower.contains(&tech_lower) {
                // Higher score for exact tech names in headings/metadata
                if chunk
                    .heading_context
                    .as_ref()
                    .is_some_and(|h| h.to_lowercase().contains(&tech_lower))
                {
                    matches += 2;
                } else {
                    matches += 1;
                }
            }
        }

        if matches == 0 {
            return 0.0;
        }

        // Normalize: max possible matches = num_techs * 2 (if all in headings)
        let max_possible = context.technologies.len() as f32 * 2.0;
        (matches as f32 / max_possible).min(1.0)
    }

    /// Score based on intent matching.
    fn compute_intent_match_score(
        &self,
        chunk: &RetrievedChunk,
        context: &RetrievalContext,
    ) -> f32 {
        let chunk_text_lower = chunk.text.to_lowercase();
        let chunk_metadata = &chunk.metadata;

        // Check metadata for intent hints
        let metadata_intent = chunk_metadata.get("intent").and_then(|v| v.as_str());
        if let Some(mi) = metadata_intent {
            if mi == context.intent.to_string() {
                return 1.0;
            }
        }

        // Heuristic: check chunk content for intent-matching patterns
        let mut score = 0.0;
        let lower = chunk_text_lower.as_str();

        match &context.intent {
            Intent::Documentation
                if lower.contains("how to")
                    || lower.contains("tutorial")
                    || lower.contains("guide") =>
            {
                score = 0.8;
            }
            Intent::Debugging
                if lower.contains("error")
                    || lower.contains("fix")
                    || lower.contains("stack trace")
                    || lower.contains("exception") =>
            {
                score = 0.9;
            }
            Intent::CodeExample
                if lower.contains("example")
                    || lower.contains("code")
                    || lower.contains("```")
                    || lower.contains("fn ")
                    || lower.contains("def ") =>
            {
                score = 0.85;
            }
            Intent::ConceptExplainer
                if lower.contains("concept")
                    || lower.contains("architecture")
                    || lower.contains("design") =>
            {
                score = 0.8;
            }
            Intent::Comparison
                if lower.contains("vs")
                    || lower.contains("versus")
                    || lower.contains("compare")
                    || lower.contains("difference") =>
            {
                score = 0.85;
            }
            _ => {}
        }

        score
    }

    /// Score based on workspace match.
    fn compute_workspace_score(
        &self,
        chunk: &RetrievedChunk,
        target_workspace: Option<&str>,
    ) -> f32 {
        // If no workspace context, neutral score
        let target = match target_workspace {
            Some(t) => t,
            None => return 0.5,
        };

        // Check metadata for workspace ID
        let chunk_workspace = chunk
            .metadata
            .get("workspace_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if chunk_workspace == target {
            return 1.0;
        }

        // If target workspace is in the chunk's source or title, partial match
        let chunk_lower = chunk.text.to_lowercase();
        if chunk_lower.contains(target) || chunk.source_file.contains(target) {
            return 0.5;
        }

        0.0
    }

    /// Score based on conversation history overlap.
    fn compute_conversation_score(
        &self,
        chunk: &RetrievedChunk,
        context: &RetrievalContext,
    ) -> f32 {
        if context.conversation_history.is_empty() {
            return 0.0;
        }

        let chunk_text_lower = chunk.text.to_lowercase();
        let mut overlap_count = 0;
        let mut total_terms = 0;

        for history in &context.conversation_history {
            let lower = history.to_lowercase();
            // Extract key terms (words > 3 chars)
            let terms: Vec<&str> = lower.split_whitespace().filter(|w| w.len() > 3).collect();

            total_terms += terms.len();

            for term in terms {
                if chunk_text_lower.contains(term) {
                    overlap_count += 1;
                }
            }
        }

        if total_terms == 0 {
            return 0.0;
        }

        (overlap_count as f32 / total_terms as f32).min(1.0)
    }

    /// Score based on keyword overlap with query.
    fn compute_keyword_overlap(&self, chunk: &RetrievedChunk, context: &RetrievalContext) -> f32 {
        if context.priority_keywords.is_empty() {
            return 0.5; // Neutral if no priority keywords
        }

        let chunk_text_lower = chunk.text.to_lowercase();
        let mut matched = 0;

        for keyword in &context.priority_keywords {
            let kw_lower = keyword.to_lowercase();
            if chunk_text_lower.contains(&kw_lower) {
                matched += 1;
            }
        }

        if context.priority_keywords.is_empty() {
            return 0.5;
        }

        matched as f32 / context.priority_keywords.len() as f32
    }

    /// Classify relevance level based on combined score.
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
}

impl Default for ContextAwareRetriever {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract priority keywords from a query (simple heuristic).
pub fn extract_keywords(query: &str, count: usize) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can",
        "need", "dare", "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by", "from",
        "as", "into", "through", "during", "before", "after", "above", "below", "between", "out",
        "off", "over", "under", "again", "further", "then", "once", "and", "but", "or", "nor",
        "not", "so", "if", "than", "that", "this", "these", "those", "which", "what", "where",
        "when", "how", "why", "who", "whom",
    ];

    query
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty() && !stop_words.contains(&w.as_str()) && w.len() > 2)
        .collect::<Vec<_>>()
        .into_iter()
        .take(count)
        .collect()
}

/// Detect technologies mentioned in a query.
pub fn detect_technologies(query: &str, known_tech: &[&str]) -> Vec<String> {
    let query_lower = query.to_lowercase();
    let mut detected = Vec::new();

    for tech in known_tech {
        if query_lower.contains(tech.to_lowercase().as_str()) {
            detected.push(tech.to_string());
        }
    }

    detected.sort_by(|a, b| {
        query_lower
            .find(a.to_lowercase().as_str())
            .unwrap_or(usize::MAX)
            .cmp(
                &query_lower
                    .find(b.to_lowercase().as_str())
                    .unwrap_or(usize::MAX),
            )
    });

    detected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retrieval::RetrievedChunk;
    use serde_json;
    use serde_json::json;

    fn make_chunk(text: &str, score: f32) -> RetrievedChunk {
        RetrievedChunk {
            chunk_id: "test".to_string(),
            document_id: "doc".to_string(),
            text: text.to_string(),
            heading_context: None,
            section: None,
            metadata: json!({}),
            similarity_score: score,
            source_file: "test".to_string(),
            relevance: RelevanceLevel::Moderate,
        }
    }

    #[test]
    fn test_intent_detection() {
        assert_eq!(
            Intent::detect_from_query("how to install docker"),
            Intent::Documentation
        );
        assert_eq!(
            Intent::detect_from_query("fix error: segfault in main.rs"),
            Intent::Debugging
        );
        assert_eq!(
            Intent::detect_from_query("rust vs go code example"),
            Intent::CodeExample
        );
        assert_eq!(
            Intent::detect_from_query("kubernetes vs docker compare"),
            Intent::Comparison
        );
        assert_eq!(
            Intent::detect_from_query("what is microservices architecture"),
            Intent::ConceptExplainer
        );
    }

    #[test]
    fn test_tech_match_scoring() {
        let retriever = ContextAwareRetriever::new();
        let context = RetrievalContext {
            technologies: vec!["rust".to_string(), "docker".to_string()],
            ..Default::default()
        };

        // Chunk with high tech relevance
        let chunk_with_tech =
            make_chunk("This guide covers Docker setup for Rust applications", 0.6);
        let score = retriever.compute_tech_match_score(&chunk_with_tech, &context);
        assert!(
            score >= 0.5,
            "High tech overlap should score high, got {}",
            score
        );

        // Chunk with low tech relevance
        let chunk_no_tech = make_chunk("This is a general overview of project management", 0.6);
        let score_no_tech = retriever.compute_tech_match_score(&chunk_no_tech, &context);
        assert_eq!(score_no_tech, 0.0, "No tech overlap should score zero");
    }

    #[test]
    fn test_workspace_scoring() {
        let retriever = ContextAwareRetriever::new();

        let chunk_same_workspace = RetrievedChunk {
            chunk_id: "test".to_string(),
            document_id: "doc".to_string(),
            text: "workspace content".to_string(),
            heading_context: None,
            section: None,
            metadata: json!({"workspace_id": "ws-123"}),
            similarity_score: 0.5,
            source_file: "test".to_string(),
            relevance: RelevanceLevel::Moderate,
        };

        let score = retriever.compute_workspace_score(&chunk_same_workspace, Some("ws-123"));
        assert_eq!(score, 1.0, "Same workspace should score 1.0");

        let chunk_diff_workspace = RetrievedChunk {
            chunk_id: "test".to_string(),
            document_id: "doc".to_string(),
            text: "different workspace".to_string(),
            heading_context: None,
            section: None,
            metadata: json!({"workspace_id": "ws-456"}),
            similarity_score: 0.5,
            source_file: "test".to_string(),
            relevance: RelevanceLevel::Moderate,
        };

        let score_diff = retriever.compute_workspace_score(&chunk_diff_workspace, Some("ws-123"));
        assert_eq!(score_diff, 0.0, "Different workspace should score 0.0");

        // No target workspace context
        let score_no_target = retriever.compute_workspace_score(&chunk_same_workspace, None);
        assert_eq!(score_no_target, 0.5, "No target should give neutral score");
    }

    #[test]
    fn test_context_aware_ranking() {
        let retriever = ContextAwareRetriever::new();
        let context = RetrievalContext {
            technologies: vec!["rust".to_string()],
            intent: Intent::Debugging,
            workspace_id: Some("ws-123".to_string()),
            conversation_history: vec!["segmentation fault".to_string()],
            priority_keywords: vec!["rust".to_string(), "debug".to_string()],
            ..Default::default()
        };

        let chunks = vec![
            make_chunk("Rust segfault debugging guide for vector stores", 0.4),
            make_chunk("General documentation about web servers", 0.7),
            make_chunk("Docker container setup for production", 0.5),
        ];

        let ranked = retriever.rank_chunks(chunks, &context, Some("ws-123"));

        // The Rust debugging chunk should be ranked highest due to tech, intent, and keyword matches
        assert!(!ranked.is_empty(), "Should have at least one result");
        assert_eq!(
            ranked[0].text, "Rust segfault debugging guide for vector stores",
            "Debugging Rust chunk should be ranked first"
        );
    }

    #[test]
    fn test_keyword_extraction() {
        let keywords = extract_keywords("How to deploy Docker containers to AWS ECS", 5);
        assert!(keywords.contains(&"docker".to_string()));
        assert!(keywords.contains(&"containers".to_string()));
        assert!(keywords.contains(&"aws".to_string()));
        assert!(keywords.contains(&"ecs".to_string()));
        assert!(!keywords.contains(&"how".to_string()));
        assert!(!keywords.contains(&"to".to_string()));
    }

    #[test]
    fn test_technology_detection() {
        let known = &["rust", "docker", "kubernetes", "aws"];
        let detected = detect_technologies("deploy to kubernetes using docker", known);
        assert!(detected.contains(&"docker".to_string()));
        assert!(detected.contains(&"kubernetes".to_string()));
    }
}
