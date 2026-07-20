//! Recommendation Builder — Part of Engineering Intelligence Engine
//!
//! Builds safe, advisory recommendations that are always non-prescriptive.
//! The engineer is responsible for all decisions and actions.
//!
//! Recommendations are built from the fused context and are always accompanied
//! by a confidence score and evidence chain.

use serde::{Deserialize, Serialize};
use wikilabs_context_fusion::FusedContext;

/// A safe, advisory recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Human-readable title of the recommendation.
    pub title: String,
    /// Detailed description with context.
    pub description: String,
    /// Confidence in this recommendation (0.0-1.0).
    pub confidence: f32,
    /// Evidence items that support this recommendation.
    pub evidence: Vec<String>,
}

/// Builder for safe, advisory recommendations.
pub struct RecommendationBuilder {
    /// Maximum recommendations to generate.
    pub max_recommendations: usize,
}

impl Default for RecommendationBuilder {
    fn default() -> Self {
        Self {
            max_recommendations: 5,
        }
    }
}

impl RecommendationBuilder {
    /// Build safe, advisory recommendations from the fused context.
    pub fn build_safe_recommendations(&self, context: &FusedContext) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Recommendation 1: If we have technologies but missing evidence
        if !context.technologies.is_empty() && !context.missing_evidence.is_empty() {
            let tech_names: Vec<String> = context
                .technologies
                .iter()
                .map(|t| t.name.clone())
                .collect();
            recommendations.push(Recommendation {
                title: format!("Gather evidence for {} technologies", tech_names.join(", ")),
                description: format!(
                    "We have detected {} technology(s) but are missing {} pieces of \
                    evidence. Consider gathering: {}",
                    tech_names.len(),
                    context.missing_evidence.len(),
                    context.missing_evidence.join(", ")
                ),
                confidence: 0.6,
                evidence: context.missing_evidence.clone(),
            });
        }

        // Recommendation 2: If workflow state is set
        if let Some(ref state) = context.workflow_state {
            recommendations.push(Recommendation {
                title: format!("Review current workflow state: {}", state),
                description: format!(
                    "The current workflow state is '{}'. Verify this matches the \
                    engineer's actual workflow and update if needed.",
                    state
                ),
                confidence: 0.5,
                evidence: vec![format!("workflow_state = '{}'", state)],
            });
        }

        // Recommendation 3: If human corrections exist
        if !context.human_corrections_applied.is_empty() {
            recommendations.push(Recommendation {
                title: "Review applied human corrections".to_string(),
                description: format!(
                    "There are {} applied human corrections. Review these to ensure \
                    the AI understanding is aligned with the engineer's actual goals.",
                    context.human_corrections_applied.len()
                ),
                confidence: 0.7,
                evidence: context.human_corrections_applied.clone(),
            });
        }

        // Limit results
        recommendations.truncate(self.max_recommendations);

        recommendations
    }
}
