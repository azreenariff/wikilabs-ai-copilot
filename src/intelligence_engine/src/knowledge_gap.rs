//! Knowledge Gap Detection — Part of Engineering Intelligence Engine
//!
//! Identifies what information is missing from the engineering context.
//! Knowledge gaps prevent accurate analysis and recommendations.

use serde::{Deserialize, Serialize};
use wikilabs_context_fusion::FusedContext;

/// A gap in our knowledge about the current situation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    /// Human-readable title of the gap.
    pub title: String,
    /// Detailed description of what's missing.
    pub description: String,
    /// Confidence that this is a real gap (0.0-1.0).
    pub confidence: f32,
    /// Evidence that suggests the gap exists.
    pub evidence: Vec<String>,
}

/// Detector for knowledge gaps.
pub struct KnowledgeGapDetector {
    /// Maximum gaps to return.
    pub max_gaps: usize,
}

impl Default for KnowledgeGapDetector {
    fn default() -> Self {
        Self { max_gaps: 10 }
    }
}

impl KnowledgeGapDetector {
    /// Detect knowledge gaps in the fused context.
    pub fn detect(&self, context: &FusedContext) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();

        // Gap 1: No technology detected
        if context.technologies.is_empty() {
            gaps.push(KnowledgeGap {
                title: "No technology detected".to_string(),
                description: "No technologies have been detected in the current context. \
                    Without knowing which technologies are involved, the AI cannot \
                    provide relevant advice."
                    .to_string(),
                confidence: 0.9,
                evidence: vec![
                    "technology list is empty".to_string(),
                    "observation events may not have been collected".to_string(),
                ],
            });
        }

        // Gap 2: No intent detected
        if context.intents.is_empty() {
            gaps.push(KnowledgeGap {
                title: "No intent detected".to_string(),
                description: "No engineering intent has been recognized. Without knowing \
                    what the engineer is trying to accomplish, the AI cannot \
                    provide targeted advice."
                    .to_string(),
                confidence: 0.8,
                evidence: vec![
                    "intent list is empty".to_string(),
                    "conversation may lack clear directives".to_string(),
                ],
            });
        }

        // Gap 3: Missing workflow state
        if context.workflow_state.is_none() {
            gaps.push(KnowledgeGap {
                title: "Workflow state not established".to_string(),
                description: "No workflow state has been identified. The workflow engine \
                    needs to know the current stage to provide appropriate guidance."
                    .to_string(),
                confidence: 0.6,
                evidence: vec!["workflow_state is None".to_string()],
            });
        }

        // Gap 4: Missing confidence scores
        if context.confidence_scores.is_empty() && !context.technologies.is_empty() {
            gaps.push(KnowledgeGap {
                title: "No confidence scores for technologies".to_string(),
                description: "Technologies have been detected but no confidence scores \
                    exist for them. Without confidence scores, we cannot \
                    assess the reliability of technology identification."
                    .to_string(),
                confidence: 0.7,
                evidence: vec![format!(
                    "{} technologies detected with no confidence scores",
                    context.technologies.len()
                )],
            });
        }

        // Gap 5: Missing evidence for known technology
        if !context.technologies.is_empty() && context.missing_evidence.is_empty() {
            // If we have technologies but no missing evidence, that's actually fine
            // Don't report a gap in this case
        }

        // Limit results
        gaps.truncate(self.max_gaps);

        gaps
    }
}
