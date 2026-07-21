//! Recommendation Readiness Engine — Phase 7
//!
//! Determines whether enough engineering context has been gathered to
//! produce actionable recommendations.
//!
//! ## Architecture
//!
//! - **EvidenceItem** — Structured pieces of evidence collected during workflows
//! - **RecommendationReadinessEngine** — Tracks readiness, evidence, and missing items
//! - **ReadinessReport** — Serializable report with readiness percentage and recommendation
//!
//! ## Core Principles
//!
//! - Readiness is a continuous metric (0.0–1.0)
//! - Evidence is tracked with type, source, confidence, and timestamp
//! - Human feedback overrides all AI inference
//! - Works in conjunction with the WorkflowEngine for state-aware readiness

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;
use wikilabs_workflow_engine::WorkflowState;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A piece of evidence collected during engineering workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceItem {
    /// Unique evidence ID.
    pub id: String,
    /// Type of evidence ("file_analysis", "code_structure", "dependency", etc.).
    pub type_: String,
    /// Source of the evidence ("observation", "analysis", "user", "skill").
    pub source: String,
    /// Human-readable description.
    pub description: String,
    /// Confidence in this evidence (0.0–1.0).
    pub confidence: f32,
    /// When it was collected.
    pub timestamp: DateTime<Utc>,
}

impl EvidenceItem {
    /// Create a new evidence item.
    pub fn new(
        type_: impl Into<String>,
        source: impl Into<String>,
        description: impl Into<String>,
        confidence: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            type_: type_.into(),
            source: source.into(),
            description: description.into(),
            confidence: confidence.clamp(0.0, 1.0),
            timestamp: Utc::now(),
        }
    }
}

/// Readiness report with all details for consumption by the UI or other systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessReport {
    /// Whether enough context exists for actionable recommendations.
    pub is_ready: bool,
    /// Readiness percentage (0.0–100.0).
    pub readiness_percentage: f32,
    /// Detected or inferred user intent.
    pub intent: Option<String>,
    /// Current workflow state.
    pub current_state: Option<String>,
    /// Evidence types still needed.
    pub missing_evidence: Vec<String>,
    /// Evidence types that have been collected.
    pub evidence_collected: Vec<String>,
    /// Human-readable recommendation.
    pub recommendation: String,
}

impl ReadinessReport {
    /// Create a "still gathering" report.
    pub fn gathering(missing: Vec<String>) -> Self {
        let pct = (1.0 - missing.len() as f32 / missing.len().max(1) as f32) * 100.0;
        Self {
            is_ready: false,
            readiness_percentage: pct,
            intent: None,
            current_state: None,
            missing_evidence: missing,
            evidence_collected: Vec::new(),
            recommendation: "Continue gathering engineering context before making recommendations."
                .to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Determines whether enough information exists to produce actionable
/// engineering recommendations.
pub struct RecommendationReadinessEngine {
    /// Overall readiness score (0.0–1.0).
    current_readiness: f32,
    /// Detected user intent.
    intent: Option<String>,
    /// Current workflow state.
    current_state: Option<String>,
    /// Evidence types still needed.
    missing_evidence: Vec<String>,
    /// All evidence items collected so far.
    evidence_collected: Vec<EvidenceItem>,
}

impl RecommendationReadinessEngine {
    /// Create a new readiness engine with no evidence.
    pub fn new() -> Self {
        Self {
            current_readiness: 0.0,
            intent: None,
            current_state: None,
            missing_evidence: Vec::new(),
            evidence_collected: Vec::new(),
        }
    }

    /// Set the detected user intent.
    pub fn set_intent(&mut self, intent: &str) {
        info!("Set intent to '{}'", intent);
        self.intent = Some(intent.to_string());
        self.recalculate_readiness();
    }

    /// Add an evidence item to the collected set.
    pub fn add_evidence(&mut self, item: EvidenceItem) {
        debug!(
            "Added evidence '{}' of type '{}' with confidence {:.2}",
            item.id, item.type_, item.confidence
        );
        self.evidence_collected.push(item.clone());
        // Remove matched type from missing_evidence
        self.missing_evidence.retain(|m| m != &item.type_);
        self.recalculate_readiness();
    }

    /// Remove an evidence item by ID.
    pub fn remove_evidence(&mut self, evidence_id: &str) {
        let before = self.evidence_collected.len();
        self.evidence_collected.retain(|e| e.id != evidence_id);
        let removed = before - self.evidence_collected.len();
        if removed > 0 {
            debug!(
                "Removed {} evidence item(s) matching '{}'",
                removed, evidence_id
            );
            self.recalculate_readiness();
        }
    }

    /// Set the list of missing evidence types.
    pub fn set_missing(&mut self, missing: Vec<String>) {
        self.missing_evidence = missing;
        self.recalculate_readiness();
    }

    /// Get a full readiness report.
    pub fn get_report(&self) -> ReadinessReport {
        let missing_count = self.missing_evidence.len();
        let collected_count = self.evidence_collected.len();
        let total_required = missing_count + collected_count;

        let pct = if total_required == 0 {
            if self.intent.is_some() || !self.evidence_collected.is_empty() {
                100.0
            } else {
                0.0
            }
        } else {
            (collected_count as f32 / total_required as f32) * 100.0
        };

        // Weighted by intent and confidence
        let weighted = if self.intent.is_some() {
            10.0 // Bonus for having an intent
        } else {
            0.0
        };

        let avg_confidence = if self.evidence_collected.is_empty() {
            0.0
        } else {
            self.evidence_collected
                .iter()
                .map(|e| e.confidence)
                .sum::<f32>()
                / self.evidence_collected.len() as f32
        };
        let confidence_bonus = avg_confidence * 15.0;

        let final_pct = (pct + weighted + confidence_bonus).min(100.0);
        let is_ready = final_pct >= 70.0;

        let recommendation = if is_ready {
            "Sufficient engineering context gathered. Ready to produce recommendations.".to_string()
        } else if self.intent.is_none() {
            "No intent detected. Clarify what the user wants to accomplish.".to_string()
        } else {
            format!(
                "Need more evidence of type(s): {}. Currently at {:.0}% readiness.",
                self.missing_evidence.join(", "),
                final_pct
            )
        };

        let collected_types: Vec<String> = self
            .evidence_collected
            .iter()
            .map(|e| format!("{} ({:.0}%)", e.type_, e.confidence * 100.0))
            .collect();

        ReadinessReport {
            is_ready,
            readiness_percentage: final_pct,
            intent: self.intent.clone(),
            current_state: self.current_state.clone(),
            missing_evidence: self.missing_evidence.clone(),
            evidence_collected: collected_types,
            recommendation,
        }
    }

    /// Quick check: is the engine ready to recommend?
    pub fn is_ready_to_recommend(&self) -> bool {
        self.get_report().is_ready
    }

    /// Update readiness from a workflow state.
    ///
    /// Extracts current state, missing evidence, and completed states
    /// from the workflow state and feeds them into the readiness engine.
    pub fn update_from_workflow(&mut self, workflow_state: &WorkflowState) {
        info!(
            "Updating readiness from workflow state: current={}, completed={}, missing={}",
            workflow_state.current_state,
            workflow_state.completed_states.len(),
            workflow_state.missing_evidence.len()
        );

        self.current_state = Some(workflow_state.current_state.clone());
        self.missing_evidence = workflow_state.missing_evidence.clone();

        // Mark completed states as having evidence
        for state in &workflow_state.completed_states {
            if !self
                .evidence_collected
                .iter()
                .any(|e| e.type_ == format!("state_{}", state))
            {
                self.evidence_collected.push(EvidenceItem {
                    id: format!("state_{}", state),
                    type_: format!("state_{}", state),
                    source: "workflow".to_string(),
                    description: format!("Reached state: {}", state),
                    confidence: 1.0,
                    timestamp: Utc::now(),
                });
            }
        }

        self.recalculate_readiness();
    }

    /// Reset the engine to its initial empty state.
    pub fn clear(&mut self) {
        self.current_readiness = 0.0;
        self.intent = None;
        self.current_state = None;
        self.missing_evidence.clear();
        self.evidence_collected.clear();
        info!("Recommendation readiness engine cleared");
    }

    /// Get the count of collected evidence items.
    pub fn get_evidence_count(&self) -> usize {
        self.evidence_collected.len()
    }

    /// Recalculate the readiness score based on current state.
    fn recalculate_readiness(&mut self) {
        let missing_count = self.missing_evidence.len();
        let collected_count = self.evidence_collected.len();
        let total_required = missing_count + collected_count;

        let base_pct = if total_required == 0 {
            if self.intent.is_some() || collected_count > 0 {
                100.0
            } else {
                0.0
            }
        } else {
            (collected_count as f32 / total_required as f32) * 100.0
        };

        let intent_bonus = if self.intent.is_some() { 10.0 } else { 0.0 };
        let confidence_bonus = if collected_count == 0 {
            0.0
        } else {
            let avg = self
                .evidence_collected
                .iter()
                .map(|e| e.confidence)
                .sum::<f32>()
                / collected_count as f32;
            avg * 15.0
        };

        self.current_readiness = (base_pct + intent_bonus + confidence_bonus).min(100.0);
        debug!("Recalculated readiness: {:.1}%", self.current_readiness);
    }

    /// Get evidence items filtered by type.
    pub fn get_evidence_by_type(&self, type_: &str) -> Vec<&EvidenceItem> {
        self.evidence_collected
            .iter()
            .filter(|e| e.type_ == type_)
            .collect()
    }

    /// Get evidence items with confidence above threshold.
    pub fn get_high_confidence_evidence(&self, threshold: f32) -> Vec<&EvidenceItem> {
        self.evidence_collected
            .iter()
            .filter(|e| e.confidence >= threshold)
            .collect()
    }

    /// Get all collected evidence (for serialization or inspection).
    pub fn get_all_evidence(&self) -> &[EvidenceItem] {
        &self.evidence_collected
    }
}

impl Default for RecommendationReadinessEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine_is_empty() {
        let engine = RecommendationReadinessEngine::new();
        assert_eq!(engine.get_evidence_count(), 0);
        assert!(!engine.is_ready_to_recommend());
        let report = engine.get_report();
        assert!(!report.is_ready);
        assert_eq!(report.readiness_percentage, 0.0);
    }

    #[test]
    fn test_set_intent() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.set_intent("debug production");
        let report = engine.get_report();
        assert_eq!(report.intent, Some("debug production".to_string()));
    }

    #[test]
    fn test_add_evidence() {
        let mut engine = RecommendationReadinessEngine::new();
        let evidence = EvidenceItem::new(
            "code_structure",
            "observation",
            "Found Rust project structure",
            0.9,
        );
        engine.add_evidence(evidence);

        assert_eq!(engine.get_evidence_count(), 1);
    }

    #[test]
    fn test_remove_evidence() {
        let mut engine = RecommendationReadinessEngine::new();
        let evidence =
            EvidenceItem::new("code_structure", "observation", "Found Rust project", 0.9);
        let id = evidence.id.clone();
        engine.add_evidence(evidence);

        assert_eq!(engine.get_evidence_count(), 1);
        engine.remove_evidence(&id);
        assert_eq!(engine.get_evidence_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_evidence() {
        let mut engine = RecommendationReadinessEngine::new();
        let evidence =
            EvidenceItem::new("code_structure", "observation", "Found Rust project", 0.9);
        engine.add_evidence(evidence);
        engine.remove_evidence("nonexistent-id");
        assert_eq!(engine.get_evidence_count(), 1);
    }

    #[test]
    fn test_set_missing_evidence() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.set_missing(vec!["tech_stack".to_string(), "intent".to_string()]);

        let report = engine.get_report();
        assert_eq!(report.missing_evidence.len(), 2);
    }

    #[test]
    fn test_readiness_report_gathering() {
        let report =
            ReadinessReport::gathering(vec!["tech_stack".to_string(), "intent".to_string()]);
        assert!(!report.is_ready);
        assert!(report.recommendation.contains("gathering"));
    }

    #[test]
    fn test_readiness_calculation() {
        let mut engine = RecommendationReadinessEngine::new();
        // Set missing evidence requirements
        engine.set_missing(vec![
            "code_structure".to_string(),
            "tech_stack".to_string(),
            "intent".to_string(),
        ]);

        // Initially 0 collected, 3 missing → ~0%
        let report = engine.get_report();
        assert_eq!(report.readiness_percentage, 0.0);

        // Add 1 of 3
        engine.add_evidence(EvidenceItem::new(
            "code_structure",
            "observation",
            "Found Cargo.toml",
            0.9,
        ));
        let report = engine.get_report();
        // 1 collected, 2 missing → 33% base + bonuses
        assert!(report.readiness_percentage > 0.0);

        // Add 2 more
        engine.add_evidence(EvidenceItem::new(
            "tech_stack",
            "analysis",
            "Detected Rust workspace",
            0.85,
        ));
        engine.add_evidence(EvidenceItem::new(
            "intent",
            "user",
            "User wants to debug",
            1.0,
        ));

        let report = engine.get_report();
        assert!(report.readiness_percentage >= 100.0);
        assert!(report.is_ready);
    }

    #[test]
    fn test_is_ready_to_recommend() {
        let mut engine = RecommendationReadinessEngine::new();
        assert!(!engine.is_ready_to_recommend());

        engine.set_missing(vec!["intent".to_string()]);
        engine.add_evidence(EvidenceItem::new(
            "intent",
            "user",
            "User wants deployment help",
            1.0,
        ));

        // Should be ready now (100% evidence + intent bonus)
        assert!(engine.is_ready_to_recommend());
    }

    #[test]
    fn test_update_from_workflow() {
        let mut engine = RecommendationReadinessEngine::new();
        let workflow_state = WorkflowState {
            current_state: "analysis".to_string(),
            completed_states: vec!["discovery".to_string()],
            evidence_collected: vec![],
            missing_evidence: vec!["tech_stack".to_string()],
        };

        engine.update_from_workflow(&workflow_state);

        assert_eq!(engine.current_state, Some("analysis".to_string()));
        assert_eq!(engine.get_evidence_count(), 1); // discovery state counted
        assert_eq!(engine.missing_evidence, vec!["tech_stack".to_string()]);
    }

    #[test]
    fn test_clear() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.set_intent("debug");
        engine.add_evidence(EvidenceItem::new(
            "code_structure",
            "observation",
            "Found project",
            0.9,
        ));

        engine.clear();
        assert!(engine.intent.is_none());
        assert!(engine.current_state.is_none());
        assert!(engine.missing_evidence.is_empty());
        assert_eq!(engine.get_evidence_count(), 0);
    }

    #[test]
    fn test_evidence_by_type() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.add_evidence(EvidenceItem::new(
            "code_structure",
            "observation",
            "Found Cargo.toml",
            0.9,
        ));
        engine.add_evidence(EvidenceItem::new(
            "code_structure",
            "analysis",
            "Multi-crate workspace",
            0.8,
        ));
        engine.add_evidence(EvidenceItem::new(
            "dependency",
            "analysis",
            "Found tokio dependency",
            0.95,
        ));

        let by_type = engine.get_evidence_by_type("code_structure");
        assert_eq!(by_type.len(), 2);

        let by_type = engine.get_evidence_by_type("nonexistent");
        assert!(by_type.is_empty());
    }

    #[test]
    fn test_high_confidence_evidence() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.add_evidence(EvidenceItem::new(
            "code_structure",
            "observation",
            "Found Cargo.toml",
            0.9,
        ));
        engine.add_evidence(EvidenceItem::new(
            "code_structure",
            "analysis",
            "Uncertain detection",
            0.3,
        ));

        let high = engine.get_high_confidence_evidence(0.8);
        assert_eq!(high.len(), 1);
        assert_eq!(high[0].description, "Found Cargo.toml");
    }

    #[test]
    fn test_report_evidence_display() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.add_evidence(EvidenceItem::new(
            "code_structure",
            "observation",
            "Found Cargo.toml",
            0.9,
        ));

        let report = engine.get_report();
        assert!(!report.evidence_collected.is_empty());
        assert!(report.evidence_collected[0].contains("code_structure"));
    }

    #[test]
    fn test_confidence_clamping() {
        let evidence = EvidenceItem::new(
            "test", "obs", "test", 1.5, // Should be clamped to 1.0
        );
        assert_eq!(evidence.confidence, 1.0);

        let evidence2 = EvidenceItem::new(
            "test", "obs", "test", -0.3, // Should be clamped to 0.0
        );
        assert_eq!(evidence2.confidence, 0.0);
    }

    #[test]
    fn test_get_all_evidence() {
        let mut engine = RecommendationReadinessEngine::new();
        let e1 = EvidenceItem::new("a", "obs", "a", 0.5);
        let e2 = EvidenceItem::new("b", "obs", "b", 0.8);
        engine.add_evidence(e1);
        engine.add_evidence(e2);

        let all = engine.get_all_evidence();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].type_, "a");
        assert_eq!(all[1].type_, "b");
    }

    #[test]
    fn test_recommendation_report_ready_message() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.set_intent("deployment");
        engine.set_missing(vec![]);
        engine.add_evidence(EvidenceItem::new(
            "context",
            "observation",
            "Sufficient context",
            0.9,
        ));

        let report = engine.get_report();
        assert!(report.is_ready);
        assert!(report
            .recommendation
            .contains("Ready to produce recommendations"));
    }

    #[test]
    fn test_recommendation_report_not_ready_message() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.set_intent("debug");
        engine.set_missing(vec!["tech_stack".to_string(), "logs".to_string()]);

        let report = engine.get_report();
        assert!(!report.is_ready);
        assert!(report.recommendation.contains("tech_stack"));
        assert!(report.recommendation.contains("logs"));
    }

    #[test]
    fn test_no_intent_message() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.set_missing(vec!["intent".to_string()]);
        // No intent set

        let report = engine.get_report();
        assert!(report.recommendation.contains("No intent detected"));
    }

    #[test]
    fn test_readiness_with_only_intent() {
        let mut engine = RecommendationReadinessEngine::new();
        engine.set_intent("deployment");
        engine.set_missing(vec![]);

        // Intent set but no collected evidence → report returns 100% if intent is set
        // and missing is empty
        let report = engine.get_report();
        assert_eq!(report.readiness_percentage, 100.0);
        assert!(report.is_ready);
    }

    #[test]
    fn test_evidence_count_accuracy() {
        let mut engine = RecommendationReadinessEngine::new();
        assert_eq!(engine.get_evidence_count(), 0);

        engine.add_evidence(EvidenceItem::new("a", "obs", "a", 0.5));
        assert_eq!(engine.get_evidence_count(), 1);

        engine.add_evidence(EvidenceItem::new("b", "obs", "b", 0.7));
        assert_eq!(engine.get_evidence_count(), 2);

        let first_id = engine.get_all_evidence()[0].id.clone();
        engine.remove_evidence(&first_id);
        assert_eq!(engine.get_evidence_count(), 1);
    }
}
