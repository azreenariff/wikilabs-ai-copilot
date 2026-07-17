//! Root Cause Analysis — Part of Engineering Intelligence Engine
//!
//! Generates hypotheses about the root cause of issues based on the
//! fused engineering context. All hypotheses are labeled as such —
//! never conclusions.
//!
//! ## Approach
//!
//! Root cause analysis works by:
//! 1. Looking for failure patterns in the context
//! 2. Identifying the most likely causal factor
//! 3. Generating testable hypotheses
//! 4. Ranking hypotheses by confidence

use serde::{Deserialize, Serialize};
use wikilabs_context_fusion::FusedContext;

/// A hypothesis about the root cause of an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseHypothesis {
    /// Human-readable title of the hypothesis.
    pub title: String,
    /// Detailed description explaining the hypothesis.
    pub description: String,
    /// Confidence in this hypothesis (0.0-1.0).
    pub confidence: f32,
    /// Evidence items that support this hypothesis.
    pub evidence: Vec<String>,
}

/// Root cause analyzer — generates hypotheses.
pub struct RootCauseAnalyzer;

impl RootCauseAnalyzer {
    /// Create a new root cause analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyze the fused context and generate root cause hypotheses.
    ///
    /// Returns a list of hypotheses ranked by confidence.
    /// All hypotheses are clearly labeled as such — never conclusions.
    pub fn analyze(&self, context: &FusedContext) -> Vec<RootCauseHypothesis> {
        let mut hypotheses = Vec::new();

        // Check 1: Low-confidence technology detection
        if let Some(tech) = context.technologies.first() {
            if tech.confidence < 0.5 {
                hypotheses.push(RootCauseHypothesis {
                    title: format!(
                        "Technology mismatch for {}",
                        tech.name
                    ),
                    description: format!(
                        "The detected technology '{}' has low confidence ({:.0}%). \
                        This may indicate the system is running a different version, \
                        a fork, or a similar technology that was misidentified.",
                        tech.name,
                        tech.confidence * 100.0
                    ),
                    confidence: 0.6,
                    evidence: vec![
                        format!(
                            "Technology '{}' has confidence {:.2}",
                            tech.name, tech.confidence
                        ),
                    ],
                });
            }
        }

        // Check 2: Conflicting intents
        if context.intents.len() > 1 {
            let top = &context.intents[0];
            let second = &context.intents[1];
            if top.confidence - second.confidence < 0.15 {
                hypotheses.push(RootCauseHypothesis {
                    title: "Conflicting intent signals".to_string(),
                    description: format!(
                        "Multiple intents with similar confidence scores make it \
                        unclear which is correct. This may indicate ambiguous input \
                        from the engineer or an incomplete observation window.",
                    ),
                    confidence: 0.5,
                    evidence: vec![
                        format!(
                            "Intent '{}' at {:.2}",
                            top.intent, top.confidence
                        ),
                        format!(
                            "Intent '{}' at {:.2}",
                            second.intent, second.confidence
                        ),
                        "Confidence gap < 0.15".to_string(),
                    ],
                });
            }
        }

        // Check 3: Missing critical evidence
        if !context.missing_evidence.is_empty() {
            let gap_count = context.missing_evidence.len();
            hypotheses.push(RootCauseHypothesis {
                title: format!(
                    "Incomplete evidence for root cause analysis ({})",
                    gap_count
                ),
                description: format!(
                    "{} pieces of evidence are missing, which may prevent accurate \
                    root cause analysis. The most critical missing item is: {}",
                    gap_count,
                    context
                        .missing_evidence
                        .first()
                        .map(|s| s.as_str())
                        .unwrap_or("unknown")
                ),
                confidence: 0.7,
                evidence: context.missing_evidence.clone(),
            });
        }

        // Sort by confidence descending
        hypotheses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        hypotheses
    }
}