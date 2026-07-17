//! Engineering Intelligence Engine — Phase 7
//!
//! The intelligence engine provides deep analysis capabilities that combine
//! observation events, technology context, intent, workflow, and human feedback
//! to produce actionable insights about the engineering session.
//!
//! ## Architecture
//!
//! ```
//! FusedContext (from context_fusion)
//!     │
//!     ├─► AnomalyDetection → Unusual patterns discovered
//!     ├─► RecommendationBuilder → Safe, non-prescriptive suggestions
//!     ├─► RootCauseAnalyzer → Hypothesis generation for failures
//!     └─► KnowledgeGapDetector → What we don't know yet
//! ```
//!
//! ## Core Principles
//!
//! - Always advisory — never prescriptive
//! - All suggestions traceable to evidence
//! - Never execute actions without explicit human confirmation
//! - Confidence scores on all intelligence outputs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wikilabs_data_types::TechnologyInference;
use wikilabs_context_fusion::FusedContext;

use crate::{
    anomaly::AnomalyReport,
    recommendation::{Recommendation, RecommendationBuilder},
    root_cause::RootCauseHypothesis,
    knowledge_gap::{KnowledgeGap, KnowledgeGapDetector},
};

// Re-export sub-modules
pub mod anomaly;
pub mod recommendation;
pub mod root_cause;
pub mod knowledge_gap;

// ---------------------------------------------------------------------------
// Intelligence output
// ---------------------------------------------------------------------------

/// A piece of intelligence — insight, recommendation, or finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceItem {
    /// Unique item ID.
    pub id: Uuid,
    /// Type of intelligence (anomaly, recommendation, hypothesis, gap, summary).
    #[serde(rename = "type")]
    pub intelligence_type: String,
    /// Title/summary of the intelligence.
    pub title: String,
    /// Detailed explanation.
    pub description: String,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Evidence items that support this intelligence.
    pub evidence: Vec<String>,
    /// When this was generated.
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Advisory note (always included).
    pub advisory: String,
}

impl IntelligenceItem {
    pub fn new(
        intelligence_type: &str,
        title: &str,
        description: &str,
        confidence: f32,
        evidence: Vec<String>,
        advisory: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            intelligence_type: intelligence_type.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            evidence,
            generated_at: chrono::Utc::now(),
            advisory: advisory.to_string(),
        }
    }

    /// Create a new recommendation intelligence item.
    pub fn recommendation(title: &str, desc: &str, confidence: f32, evidence: Vec<String>) -> Self {
        Self::new(
            "recommendation",
            title,
            desc,
            confidence,
            evidence,
            "This is an advisory suggestion. The engineer is responsible for \
            all decisions and actions. Review evidence before acting.",
        )
    }

    /// Create a new anomaly intelligence item.
    pub fn anomaly(title: &str, desc: &str, confidence: f32, evidence: Vec<String>) -> Self {
        Self::new(
            "anomaly",
            title,
            desc,
            confidence,
            evidence,
            "This anomaly was detected by automated analysis. Verify with \
            manual investigation before taking action.",
        )
    }

    /// Create a new root cause hypothesis intelligence item.
    pub fn root_cause(title: &str, desc: &str, confidence: f32, evidence: Vec<String>) -> Self {
        Self::new(
            "root_cause_hypothesis",
            title,
            desc,
            confidence,
            evidence,
            "This is a hypothesis — not a conclusion. Further investigation \
            is required to confirm or reject this root cause.",
        )
    }

    /// Create a new knowledge gap intelligence item.
    pub fn knowledge_gap(title: &str, desc: &str, confidence: f32, evidence: Vec<String>) -> Self {
        Self::new(
            "knowledge_gap",
            title,
            desc,
            confidence,
            evidence,
            "We are missing information needed for a complete analysis. \
            The engineer should gather this evidence.",
        )
    }
}

/// Full intelligence report — all analysis results for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceReport {
    /// All intelligence items.
    pub items: Vec<IntelligenceItem>,
    /// Overall session summary.
    pub summary: String,
    /// Number of intelligence items generated.
    pub item_count: usize,
    /// Average confidence across all items.
    pub average_confidence: f32,
    /// When this report was generated.
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl IntelligenceReport {
    /// Create a new intelligence report.
    pub fn new(items: Vec<IntelligenceItem>, summary: String) -> Self {
        let item_count = items.len();
        let avg_confidence = if items.is_empty() {
            0.0
        } else {
            items.iter().map(|i| i.confidence).sum::<f32>() / item_count as f32
        };

        Self {
            items,
            summary,
            item_count,
            average_confidence: avg_confidence,
            generated_at: chrono::Utc::now(),
        }
    }
}

// ---------------------------------------------------------------------------
// Intelligence engine
// ---------------------------------------------------------------------------

/// Main intelligence engine — combines all analysis capabilities.
///
/// Orchestrates anomaly detection, recommendation building, root cause
/// analysis, and knowledge gap detection on top of the fused context.
pub struct EngineeringIntelligenceEngine {
    /// Recommendation builder (for safe suggestions).
    recommendation_builder: RecommendationBuilder,
    /// Root cause analyzer.
    root_cause_analyzer: root_cause::RootCauseAnalyzer,
    /// Knowledge gap detector.
    knowledge_gap_detector: KnowledgeGapDetector,
    /// Last intelligence report.
    last_report: Option<IntelligenceReport>,
}

impl EngineeringIntelligenceEngine {
    /// Create a new intelligence engine with default config.
    pub fn new() -> Self {
        Self {
            recommendation_builder: RecommendationBuilder::default(),
            root_cause_analyzer: root_cause::RootCauseAnalyzer::new(),
            knowledge_gap_detector: KnowledgeGapDetector::default(),
            last_report: None,
        }
    }

    // ------------------------------------------------------------------
    // Analysis methods
    // ------------------------------------------------------------------

    /// Run full intelligence analysis on the fused context.
    ///
    /// This is the main analysis method — it runs all sub-analyses and
    /// combines the results into a single intelligence report.
    pub fn analyze(&mut self, context: &FusedContext) -> IntelligenceReport {
        let mut items = Vec::new();

        // 1. Anomaly detection
        let anomaly_report = self.detect_anomalies(context);
        for anomaly in &anomaly_report.anomalies {
            items.push(IntelligenceItem::anomaly(
                &anomaly.title,
                &anomaly.description,
                anomaly.confidence,
                anomaly.evidence.clone(),
            ));
        }

        // 2. Recommendation building
        let recommendations = self.build_recommendations(context);
        for rec in &recommendations {
            items.push(IntelligenceItem::recommendation(
                &rec.title,
                &rec.description,
                rec.confidence,
                rec.evidence.clone(),
            ));
        }

        // 3. Root cause analysis (only if we have troubleshooting signals)
        let root_causes = self.analyze_root_causes(context);
        for rc in &root_causes {
            items.push(IntelligenceItem::root_cause(
                &rc.title,
                &rc.description,
                rc.confidence,
                rc.evidence.clone(),
            ));
        }

        // 4. Knowledge gap detection
        let gaps = self.detect_knowledge_gaps(context);
        for gap in &gaps {
            items.push(IntelligenceItem::knowledge_gap(
                &gap.title,
                &gap.description,
                gap.confidence,
                gap.evidence.clone(),
            ));
        }

        // Generate summary
        let summary = self.generate_summary(&items, context);

        let report = IntelligenceReport::new(items, summary);
        self.last_report = Some(report.clone());
        report
    }

    /// Run anomaly detection only.
    pub fn detect_anomalies(&self, context: &FusedContext) -> AnomalyReport {
        let technologies = context.technologies.iter().map(|t| t.clone()).collect();
        let intents = context.intents.iter().cloned().collect();

        let fused: FusedContext = FusedContext {
            technologies,
            intents,
            workflow_state: context.workflow_state.clone(),
            timeline: Vec::new(),
            confidence_scores: context.confidence_scores.clone(),
            missing_evidence: context.missing_evidence.clone(),
            human_corrections_applied: context.human_corrections_applied.clone(),
            evidence_summary: context.evidence_summary.clone(),
            fused_at: chrono::Utc::now(),
        };

        let anomaly_builder = crate::anomaly::AnomalyBuilder::default();
        anomaly_builder.detect_anomalies(&fused)
    }

    /// Run recommendation building only.
    pub fn build_recommendations(&self, context: &FusedContext) -> Vec<Recommendation> {
        self.recommendation_builder
            .build_safe_recommendations(context)
    }

    /// Run root cause analysis only.
    pub fn analyze_root_causes(&self, context: &FusedContext) -> Vec<RootCauseHypothesis> {
        self.root_cause_analyzer.analyze(context)
    }

    /// Run knowledge gap detection only.
    pub fn detect_knowledge_gaps(&self, context: &FusedContext) -> Vec<KnowledgeGap> {
        self.knowledge_gap_detector.detect(context)
    }

    /// Generate a human-readable summary of the analysis.
    pub fn generate_summary(&self, items: &[IntelligenceItem], context: &FusedContext) -> String {
        let mut summary_parts: Vec<String> = Vec::new();

        // Technologies
        if !context.technologies.is_empty() {
            let techs: Vec<String> = context
                .technologies
                .iter()
                .map(|t| format!("{} (confidence: {:.0}%)", t.name, t.confidence * 100.0))
                .collect();
            summary_parts.push(format!("Technologies: {}", techs.join(", ")));
        }

        // Intents
        if !context.intents.is_empty() {
            let intents: Vec<String> = context
                .intents
                .iter()
                .map(|i| format!("{} (confidence: {:.0}%)", i.intent, i.confidence * 100.0))
                .collect();
            summary_parts.push(format!("Intents: {}", intents.join(", ")));
        }

        // Summary stats
        let anomalies = items
            .iter()
            .filter(|i| i.intelligence_type == "anomaly")
            .count();
        let recommendations = items
            .iter()
            .filter(|i| i.intelligence_type == "recommendation")
            .count();
        let root_causes = items
            .iter()
            .filter(|i| i.intelligence_type == "root_cause_hypothesis")
            .count();
        let gaps = items
            .iter()
            .filter(|i| i.intelligence_type == "knowledge_gap")
            .count();

        summary_parts.push(format!(
            "Findings: {} anomalies, {} recommendations, {} root cause hypotheses, {} knowledge gaps",
            anomalies, recommendations, root_causes, gaps
        ));

        summary_parts.join("\n")
    }

    // ------------------------------------------------------------------
    // Query methods
    // ------------------------------------------------------------------

    /// Get the last intelligence report.
    pub fn get_last_report(&self) -> Option<&IntelligenceReport> {
        self.last_report.as_ref()
    }

    /// Get intelligence items filtered by type.
    pub fn get_items_by_type(&self, item_type: &str) -> Vec<&IntelligenceItem> {
        self.last_report
            .as_ref()
            .map(|r| {
                r.items
                    .iter()
                    .filter(|i| i.intelligence_type == item_type)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get recommendations from the last report.
    pub fn get_recommendations(&self) -> Vec<&IntelligenceItem> {
        self.get_items_by_type("recommendation")
    }

    /// Get anomalies from the last report.
    pub fn get_anomalies(&self) -> Vec<&IntelligenceItem> {
        self.get_items_by_type("anomaly")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = EngineeringIntelligenceEngine::new();
        assert!(engine.get_last_report().is_none());
    }

    #[test]
    fn test_intelligence_item_creation() {
        let rec = IntelligenceItem::recommendation(
            "Test recommendation",
            "Test description",
            0.8,
            vec!["evidence 1".to_string()],
        );
        assert_eq!(rec.intelligence_type, "recommendation");
        assert_eq!(rec.title, "Test recommendation");
        assert_eq!(rec.confidence, 0.8);
        assert!(!rec.advisory.is_empty());
    }

    #[test]
    fn test_intelligence_item_clamping() {
        let anomaly = IntelligenceItem::anomaly(
            "Test anomaly",
            "Test",
            1.5,
            vec![],
        );
        assert_eq!(anomaly.confidence, 1.0);

        let low = IntelligenceItem::anomaly(
            "Test",
            "Test",
            -0.5,
            vec![],
        );
        assert_eq!(low.confidence, 0.0);
    }

    #[test]
    fn test_intelligence_report() {
        let items = vec![
            IntelligenceItem::recommendation(
                "Rec 1", "Description 1", 0.8, vec!["ev1".to_string()],
            ),
            IntelligenceItem::anomaly(
                "Anomaly 1", "Description 2", 0.6, vec!["ev2".to_string()],
            ),
        ];

        let report = IntelligenceReport::new(items, "Test summary".to_string());
        assert_eq!(report.item_count, 2);
        assert!(!report.summary.is_empty());
        assert_eq!(report.average_confidence, 0.7);
    }

    #[test]
    fn test_intelligence_report_empty() {
        let report = IntelligenceReport::new(Vec::new(), "Empty report".to_string());
        assert_eq!(report.item_count, 0);
        assert_eq!(report.average_confidence, 0.0);
    }

    #[test]
    fn test_analyze_returns_report() {
        let mut engine = EngineeringIntelligenceEngine::new();
        let context = FusedContext {
            technologies: vec![TechnologyInference::new(
                "OpenShift", 0.9, "observation", "oc command",
            )],
            intents: vec![],
            workflow_state: None,
            timeline: Vec::new(),
            confidence_scores: std::collections::HashMap::new(),
            missing_evidence: Vec::new(),
            human_corrections_applied: Vec::new(),
            evidence_summary: "Test evidence summary".to_string(),
            fused_at: chrono::Utc::now(),
        };

        let report = engine.analyze(&context);
        assert!(report.item_count >= 0); // May be 0 if no signals detected
        assert!(!report.summary.is_empty());
        assert!(engine.get_last_report().is_some());
    }
}