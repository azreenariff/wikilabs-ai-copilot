//! Anomaly Detection — Part of Engineering Intelligence Engine
//!
//! Detects unusual patterns in the engineering context:
//! - Missing confidence scores
//! - Contradictory inferences
//! - Evidence gaps
//! - Unusual workflow state transitions

use serde::{Deserialize, Serialize};
use wikilabs_context_fusion::FusedContext;

/// A detected anomaly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Human-readable title of the anomaly.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Confidence in the anomaly detection (0.0-1.0).
    pub confidence: f32,
    /// Evidence items that support the anomaly finding.
    pub evidence: Vec<String>,
}

/// Anomaly report from the analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    /// All detected anomalies.
    pub anomalies: Vec<Anomaly>,
}

/// Builder for safe, advisory anomaly detection.
pub struct AnomalyBuilder {
    /// Maximum anomalies to return.
    pub max_anomalies: usize,
}

impl Default for AnomalyBuilder {
    fn default() -> Self {
        Self { max_anomalies: 10 }
    }
}

impl AnomalyBuilder {
    /// Detect anomalies in the fused context.
    pub fn detect_anomalies(&self, context: &FusedContext) -> AnomalyReport {
        let mut anomalies = Vec::new();

        // Check 1: Missing confidence data
        if context.confidence_scores.is_empty() {
            anomalies.push(Anomaly {
                title: "No confidence scores available".to_string(),
                description:
                    "The context has no confidence scores for any inference. This may indicate \
                    that the analysis pipeline has not run or all inferences were rejected."
                        .to_string(),
                confidence: 0.9,
                evidence: vec![
                    "context has 0 confidence scores".to_string(),
                    "fused context has empty confidence_scores map".to_string(),
                ],
            });
        }

        // Check 2: Missing evidence requirements
        if !context.missing_evidence.is_empty() {
            let gap_count = context.missing_evidence.len();
            anomalies.push(Anomaly {
                title: format!("{} missing evidence items", gap_count),
                description: format!(
                    "The workflow is missing {} pieces of evidence. \
                    Without this evidence, recommendations may be incomplete.",
                    gap_count
                ),
                confidence: 0.7,
                evidence: context.missing_evidence.clone(),
            });
        }

        // Check 3: Low-confidence dominant technology
        if let Some(tech) = context.technologies.first() {
            if tech.confidence < 0.5 {
                anomalies.push(Anomaly {
                    title: format!(
                        "Low confidence technology detection: {}",
                        tech.name
                    ),
                    description: format!(
                        "The dominant technology inference ({}) has only {:.0}% confidence. \
                        Human confirmation is recommended before proceeding.",
                        tech.name,
                        tech.confidence * 100.0
                    ),
                    confidence: 0.8,
                    evidence: vec![
                        format!(
                            "technology '{}' has confidence {:.2}",
                            tech.name, tech.confidence
                        ),
                        "threshold for confident detection is 0.5".to_string(),
                    ],
                });
            }
        }

        // Check 4: Conflicting intent signals
        if context.intents.len() > 1 {
            let top_intent = &context.intents[0];
            let second_intent = &context.intents[1];
            if top_intent.confidence - second_intent.confidence < 0.15 {
                anomalies.push(Anomaly {
                    title: "Conflicting intent signals".to_string(),
                    description: format!(
                        "Two intents have similar confidence ({:.0}% and {:.0}%), \
                        making it unclear which is correct.",
                        top_intent.confidence * 100.0,
                        second_intent.confidence * 100.0
                    ),
                    confidence: 0.6,
                    evidence: vec![
                        format!(
                            "top intent '{}' at {:.2}",
                            top_intent.intent, top_intent.confidence
                        ),
                        format!(
                            "second intent '{}' at {:.2}",
                            second_intent.intent, second_intent.confidence
                        ),
                        "confidence gap < 0.15".to_string(),
                    ],
                });
            }
        }

        // Limit results
        anomalies.truncate(self.max_anomalies);

        AnomalyReport { anomalies }
    }
}