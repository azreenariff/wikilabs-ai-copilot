//! Human Feedback Loop — Phase 7
//!
//! Enhanced version of the intent correction module. Human input ALWAYS
//! overrides AI inference — this is a core principle of the system.
//!
//! ## Architecture
//!
//! - **CorrectionRecord** — Records where AI inference was wrong
//! - **OverrideRecord** — Human explicitly setting a value
//! - **FeedbackRecord** — Pending feedback awaiting processing
//! - **HumanFeedbackEngine** — Manages all three, processes pending feedback
//!
//! ## Core Principles
//!
//! - Human input ALWAYS overrides AI inference
//! - Corrections are persistent — they inform future AI decisions
//! - Overrides are immediate — they take effect right away
//! - Pending feedback must be explicitly processed

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use wikilabs_intent::engine::Intent;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A correction record from human feedback.
/// Records when AI inference was wrong and what the human expected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorrectionRecord {
    /// Type of correction being made.
    pub correction_type: CorrectionType,
    /// What the human believes is correct.
    pub expected: String,
    /// What the AI inferred instead.
    pub actual: String,
    /// When the correction was recorded.
    pub timestamp: DateTime<Utc>,
    /// Optional context (conversation turn, user message, etc.).
    pub context: Option<String>,
    /// Whether this correction has been applied to future inferences.
    pub applied: bool,
}

/// Types of corrections a human can provide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CorrectionType {
    /// User corrected an AI intent inference.
    IntentCorrection,
    /// User corrected an AI technology inference.
    TechnologyCorrection,
    /// User corrected a workflow state.
    WorkflowCorrection,
    /// User corrected an evidence assessment.
    EvidenceCorrection,
    /// User adjusted an AI confidence score.
    ConfidenceOverride,
}

impl std::fmt::Display for CorrectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorrectionType::IntentCorrection => write!(f, "intent_correction"),
            CorrectionType::TechnologyCorrection => write!(f, "technology_correction"),
            CorrectionType::WorkflowCorrection => write!(f, "workflow_correction"),
            CorrectionType::EvidenceCorrection => write!(f, "evidence_correction"),
            CorrectionType::ConfidenceOverride => write!(f, "confidence_override"),
        }
    }
}

/// An override record — human explicitly setting a value.
/// Overrides take immediate effect and always beat AI inference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverrideRecord {
    /// What was overridden: "intent", "technology", "workflow_state", etc.
    pub target: String,
    /// The value the human set.
    pub value: String,
    /// Why the human made this change.
    pub reason: String,
    /// When the override was recorded.
    pub timestamp: DateTime<Utc>,
}

/// A pending feedback record that hasn't been processed yet.
/// These accumulate until the human explicitly processes them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedbackRecord {
    /// Type of feedback being provided.
    pub feedback_type: FeedbackType,
    /// The feedback content.
    pub content: String,
    /// Where the feedback came from (UI, CLI, etc.).
    pub source: String,
    /// When the feedback was recorded.
    pub timestamp: DateTime<Utc>,
}

/// Types of feedback a human can provide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// Human is clarifying their intent.
    Clarification,
    /// Human is correcting an AI inference.
    Correction,
    /// Human is confirming an AI inference.
    Confirmation,
    /// Human is refusing an AI suggestion.
    Refusal,
}

impl std::fmt::Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackType::Clarification => write!(f, "clarification"),
            FeedbackType::Correction => write!(f, "correction"),
            FeedbackType::Confirmation => write!(f, "confirmation"),
            FeedbackType::Refusal => write!(f, "refusal"),
        }
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Human feedback engine — manages corrections, overrides, and pending feedback.
///
/// Human input ALWAYS overrides AI inference. This is the core principle.
pub struct HumanFeedbackEngine {
    /// All correction records ever recorded.
    corrections: Vec<CorrectionRecord>,
    /// All override records (human-explicit value assignments).
    overrides: Vec<OverrideRecord>,
    /// Feedback that hasn't been processed yet.
    pending_feedback: Vec<FeedbackRecord>,
}

impl HumanFeedbackEngine {
    /// Create a new empty feedback engine.
    pub fn new() -> Self {
        Self {
            corrections: Vec::new(),
            overrides: Vec::new(),
            pending_feedback: Vec::new(),
        }
    }

    /// Record a human correction.
    pub fn record_correction(&mut self, record: CorrectionRecord) {
        info!(
            "Recorded {} correction: expected='{}', actual='{}'",
            record.correction_type, record.expected, record.actual
        );
        self.corrections.push(record);
    }

    /// Record a human override (takes immediate effect).
    pub fn record_override(&mut self, target: &str, value: &str, reason: &str) {
        info!(
            "Recorded override: target='{}', value='{}', reason='{}'",
            target, value, reason
        );
        self.overrides.push(OverrideRecord {
            target: target.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
            timestamp: Utc::now(),
        });
    }

    /// Record pending feedback (not yet processed).
    pub fn record_feedback(&mut self, feedback: FeedbackRecord) {
        debug!(
            "Recorded {} feedback from '{}': '{}'",
            feedback.feedback_type, feedback.source, feedback.content
        );
        self.pending_feedback.push(feedback);
    }

    /// Get all pending (unprocessed) feedback.
    pub fn get_pending(&self) -> &[FeedbackRecord] {
        &self.pending_feedback
    }

    /// Process all pending feedback, converting Corrections and Refusals
    /// into CorrectionRecord + OverrideRecord pairs.
    ///
    /// Clarifications become just overrides (no correction needed).
    /// Confirmations are discarded (AI was right).
    ///
    /// Returns a list of (CorrectionRecord, Option<OverrideRecord>) pairs.
    pub fn process_pending(&mut self) -> Vec<(CorrectionRecord, Option<OverrideRecord>)> {
        let pending = self.pending_feedback.drain(..).collect::<Vec<_>>();
        let mut results = Vec::new();

        for fb in &pending {
            match fb.feedback_type {
                FeedbackType::Correction => {
                    // Parse the feedback to extract expected vs actual
                    // Format: "expected=X, actual=Y" or just a correction statement
                    let (expected, actual) = Self::parse_correction(&fb.content);
                    let correction = CorrectionRecord {
                        correction_type: Self::feedback_to_correction_type(
                            &fb.feedback_type,
                            &fb.content,
                        ),
                        expected,
                        actual,
                        timestamp: fb.timestamp,
                        context: Some(format!(
                            "Feedback from {} ({}): {}",
                            fb.source, fb.feedback_type, fb.content
                        )),
                        applied: false,
                    };
                    let override_rec = Some(OverrideRecord {
                        target: "inferred_value".to_string(),
                        value: correction.expected.clone(),
                        reason: fb.content.clone(),
                        timestamp: fb.timestamp,
                    });
                    results.push((correction, override_rec));
                }
                FeedbackType::Clarification => {
                    // Clarification doesn't create a correction, just an override
                    let correction = CorrectionRecord {
                        correction_type: CorrectionType::IntentCorrection,
                        expected: fb.content.clone(),
                        actual: "unknown".to_string(),
                        timestamp: fb.timestamp,
                        context: Some(format!("Clarification from {}: {}", fb.source, fb.content)),
                        applied: false,
                    };
                    let override_rec = Some(OverrideRecord {
                        target: "clarification".to_string(),
                        value: fb.content.clone(),
                        reason: "User clarified their intent".to_string(),
                        timestamp: fb.timestamp,
                    });
                    results.push((correction, override_rec));
                }
                FeedbackType::Confirmation => {
                    // AI was right — just log it, no correction needed
                    debug!("Feedback confirmed AI inference: '{}'", fb.content);
                }
                FeedbackType::Refusal => {
                    // User refused AI suggestion — record as correction
                    let correction = CorrectionRecord {
                        correction_type: CorrectionType::EvidenceCorrection,
                        expected: "no_action".to_string(),
                        actual: fb.content.clone(),
                        timestamp: fb.timestamp,
                        context: Some(format!("Refusal from {}: {}", fb.source, fb.content)),
                        applied: false,
                    };
                    let override_rec = Some(OverrideRecord {
                        target: "suggested_action".to_string(),
                        value: "rejected".to_string(),
                        reason: "User refused AI suggestion".to_string(),
                        timestamp: fb.timestamp,
                    });
                    results.push((correction, override_rec));
                }
            }
        }

        // Apply the corrections and overrides
        for (correction, override_rec) in &results {
            if let Some(override_rec) = override_rec {
                self.overrides.push(OverrideRecord {
                    target: override_rec.target.clone(),
                    value: override_rec.value.clone(),
                    reason: override_rec.reason.clone(),
                    timestamp: override_rec.timestamp,
                });
            }
            self.corrections.push(correction.clone());
        }

        info!(
            "Processed {} pending feedback items, generated {} results",
            pending.len(),
            results.len()
        );

        results
    }

    /// Get the count of recorded corrections.
    pub fn get_correction_count(&self) -> usize {
        self.corrections.len()
    }

    /// Get the count of recorded overrides.
    pub fn get_override_count(&self) -> usize {
        self.overrides.len()
    }

    /// Find the most frequently occurring correction type.
    pub fn most_frequent_correction(&self) -> Option<CorrectionType> {
        if self.corrections.is_empty() {
            return None;
        }

        let mut counts: HashMap<CorrectionType, usize> = HashMap::new();
        for c in &self.corrections {
            *counts.entry(c.correction_type.clone()).or_insert(0) += 1;
        }

        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(typ, _)| typ)
    }

    /// Get all corrections for a specific type.
    pub fn get_corrections_for_target(&self, target: &str) -> Vec<&CorrectionRecord> {
        let typ = self.string_to_correction_type(target);
        self.corrections
            .iter()
            .filter(|c| c.correction_type == typ)
            .collect()
    }

    /// Get the latest override for a specific target.
    pub fn get_latest_override(&self, target: &str) -> Option<&OverrideRecord> {
        self.overrides.iter().rev().find(|o| o.target == target)
    }

    /// Check if there is a human override for a given target.
    /// Human overrides always take priority over AI inference.
    pub fn has_override(&self, target: &str) -> bool {
        self.overrides.iter().any(|o| o.target == target)
    }

    /// Get the latest override value for a target (if any).
    /// Returns None if no override exists — AI inference should be used.
    pub fn get_override_value(&self, target: &str) -> Option<String> {
        self.get_latest_override(target).map(|o| o.value.clone())
    }

    /// Apply overrides to an AI inference.
    ///
    /// If a human override exists for the target, return the override value.
    /// Otherwise return the AI-inferred value.
    pub fn apply_overrides(&self, ai_value: &str, target: &str) -> String {
        if let Some(override_val) = self.get_override_value(target) {
            info!(
                "Override applied: target='{}', AI_value='{}', override='{}'",
                target, ai_value, override_val
            );
            override_val
        } else {
            ai_value.to_string()
        }
    }

    /// Clear all feedback data.
    pub fn clear(&mut self) {
        self.corrections.clear();
        self.overrides.clear();
        self.pending_feedback.clear();
        info!("Human feedback engine cleared");
    }

    /// Get a summary of all recorded corrections (for display/logging).
    pub fn get_correction_summary(&self) -> String {
        if self.corrections.is_empty() {
            return "No corrections recorded.".to_string();
        }

        let mut summary = String::new();
        summary.push_str(&format!("Total corrections: {}\n", self.corrections.len()));

        // Group by type
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for c in &self.corrections {
            let key = c.correction_type.to_string();
            *by_type.entry(key).or_insert(0) += 1;
        }

        for (typ, count) in &by_type {
            summary.push_str(&format!("  {}: {}\n", typ, count));
        }

        summary
    }

    /// Get all corrections (for persistence or inspection).
    pub fn get_all_corrections(&self) -> &[CorrectionRecord] {
        &self.corrections
    }

    /// Get all overrides (for persistence or inspection).
    pub fn get_all_overrides(&self) -> &[OverrideRecord] {
        &self.overrides
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    /// Parse a correction content string to extract expected vs actual.
    fn parse_correction(content: &str) -> (String, String) {
        // Try "expected=X, actual=Y" format
        if content.contains("expected=") && content.contains("actual=") {
            let parts: Vec<&str> = content.splitn(2, "actual=").collect();
            let expected = parts[0].trim_start_matches("expected=").trim();
            let actual = parts[1].trim();
            return (expected.to_string(), actual.to_string());
        }

        // Default: expected = content, actual = unknown
        (content.to_string(), "unknown".to_string())
    }

    /// Convert a FeedbackType to a CorrectionType based on content.
    fn feedback_to_correction_type(_feedback_type: &FeedbackType, content: &str) -> CorrectionType {
        let lower = content.to_lowercase();
        if lower.contains("intent") || lower.contains("goal") || lower.contains("purpose") {
            CorrectionType::IntentCorrection
        } else if lower.contains("tech")
            || lower.contains("language")
            || lower.contains("framework")
        {
            CorrectionType::TechnologyCorrection
        } else if lower.contains("workflow") || lower.contains("state") || lower.contains("phase") {
            CorrectionType::WorkflowCorrection
        } else if lower.contains("confidence") || lower.contains("score") {
            CorrectionType::ConfidenceOverride
        } else {
            CorrectionType::EvidenceCorrection
        }
    }

    /// Convert a string to CorrectionType.
    fn string_to_correction_type(&self, target: &str) -> CorrectionType {
        match target.to_lowercase().as_str() {
            "intent" => CorrectionType::IntentCorrection,
            "technology" | "tech" => CorrectionType::TechnologyCorrection,
            "workflow" | "state" => CorrectionType::WorkflowCorrection,
            "evidence" => CorrectionType::EvidenceCorrection,
            "confidence" => CorrectionType::ConfidenceOverride,
            _ => CorrectionType::EvidenceCorrection,
        }
    }

    /// Get all corrections sorted by timestamp (newest first).
    pub fn recent_corrections(&self, limit: usize) -> Vec<&CorrectionRecord> {
        let mut sorted: Vec<&CorrectionRecord> = self.corrections.iter().collect();
        sorted.sort_by_key(|a| a.timestamp);
        sorted.reverse();
        sorted.into_iter().take(limit).collect()
    }

    /// Apply corrections to update future inference behaviour.
    ///
    /// This marks all pending corrections as applied. In a real system,
    /// these would update the intent recognition patterns or technology
    /// detection rules.
    pub fn apply_corrections(&mut self) -> usize {
        let count = self.corrections.iter().filter(|c| !c.applied).count();

        for c in &mut self.corrections {
            c.applied = true;
        }

        if count > 0 {
            info!("Applied {} corrections", count);
        }

        count
    }

    /// Get the most corrected intent value (from existing intent correction records).
    /// This uses the wikilabs-intent types.
    pub fn most_corrected_intent(&self) -> Option<Intent> {
        let intent_corrections: Vec<&CorrectionRecord> = self
            .corrections
            .iter()
            .filter(|c| c.correction_type == CorrectionType::IntentCorrection)
            .collect();

        if intent_corrections.is_empty() {
            return None;
        }

        // Count frequency of "expected" values (what the human thinks the intent should be)
        let mut counts: HashMap<String, usize> = HashMap::new();
        for c in &intent_corrections {
            *counts.entry(c.expected.clone()).or_insert(0) += 1;
        }

        let most_frequent = counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(intent_str, _)| intent_str);

        // Map string back to Intent enum
        match most_frequent.as_deref() {
            Some("troubleshooting") | Some("debug") => Some(Intent::Troubleshooting),
            Some("configuration") | Some("config") => Some(Intent::Configuration),
            Some("deployment") | Some("deploy") => Some(Intent::Deployment),
            Some("documentation") | Some("doc") => Some(Intent::Documentation),
            Some("learning") => Some(Intent::Learning),
            _ => None,
        }
    }
}

impl Default for HumanFeedbackEngine {
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
        let engine = HumanFeedbackEngine::new();
        assert_eq!(engine.get_correction_count(), 0);
        assert_eq!(engine.get_override_count(), 0);
        assert!(engine.get_pending().is_empty());
    }

    #[test]
    fn test_record_correction() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "deployment".to_string(),
            actual: "troubleshooting".to_string(),
            timestamp: Utc::now(),
            context: Some("User meant deployment, not troubleshooting".to_string()),
            applied: false,
        });

        assert_eq!(engine.get_correction_count(), 1);
    }

    #[test]
    fn test_record_override() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_override("intent", "deployment", "User explicitly stated deployment");

        assert_eq!(engine.get_override_count(), 1);
        assert!(engine.has_override("intent"));
    }

    #[test]
    fn test_override_applied_over_ai() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_override("intent", "deployment", "User wants deployment");

        // AI infers "troubleshooting" but override says "deployment"
        let result = engine.apply_overrides("troubleshooting", "intent");
        assert_eq!(result, "deployment");

        // No override target → AI value passes through
        let result2 = engine.apply_overrides("troubleshooting", "nonexistent");
        assert_eq!(result2, "troubleshooting");
    }

    #[test]
    fn test_record_pending_feedback() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Correction,
            content: "expected=deployment, actual=troubleshooting".to_string(),
            source: "cli".to_string(),
            timestamp: Utc::now(),
        });

        assert_eq!(engine.get_pending().len(), 1);
    }

    #[test]
    fn test_process_pending_correction() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Correction,
            content: "expected=deployment, actual=troubleshooting".to_string(),
            source: "cli".to_string(),
            timestamp: Utc::now(),
        });

        let results = engine.process_pending();
        assert!(!results.is_empty());
        assert_eq!(engine.get_pending().len(), 0);

        // Should have added the correction and override
        assert_eq!(engine.get_correction_count(), 1);
        assert_eq!(engine.get_override_count(), 1);
    }

    #[test]
    fn test_process_pending_clarification() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Clarification,
            content: "I want to configure the server".to_string(),
            source: "ui".to_string(),
            timestamp: Utc::now(),
        });

        let results = engine.process_pending();
        assert!(!results.is_empty());
        assert_eq!(engine.get_override_count(), 1);
    }

    #[test]
    fn test_process_pending_confirmation() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Confirmation,
            content: "Yes, troubleshooting is correct".to_string(),
            source: "ui".to_string(),
            timestamp: Utc::now(),
        });

        let results = engine.process_pending();
        assert!(results.is_empty()); // Confirmations don't generate results
        assert_eq!(engine.get_correction_count(), 0);
    }

    #[test]
    fn test_process_pending_refusal() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Refusal,
            content: "Don't suggest that approach".to_string(),
            source: "ui".to_string(),
            timestamp: Utc::now(),
        });

        let results = engine.process_pending();
        assert!(!results.is_empty());
        assert_eq!(engine.get_correction_count(), 1);
    }

    #[test]
    fn test_most_frequent_correction() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "deployment".to_string(),
            actual: "troubleshooting".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "deployment".to_string(),
            actual: "configuration".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::TechnologyCorrection,
            expected: "Kubernetes".to_string(),
            actual: "Docker".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });

        let most = engine.most_frequent_correction();
        assert_eq!(most, Some(CorrectionType::IntentCorrection));
    }

    #[test]
    fn test_get_corrections_for_target() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "A".to_string(),
            actual: "B".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::TechnologyCorrection,
            expected: "Rust".to_string(),
            actual: "Go".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });

        let intent_corrections = engine.get_corrections_for_target("intent");
        assert_eq!(intent_corrections.len(), 1);

        let tech_corrections = engine.get_corrections_for_target("technology");
        assert_eq!(tech_corrections.len(), 1);
    }

    #[test]
    fn test_get_latest_override() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_override("intent", "deployment", "First");
        engine.record_override("intent", "troubleshooting", "Second");

        let latest = engine.get_latest_override("intent");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().value, "troubleshooting");
    }

    #[test]
    fn test_clear() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "A".to_string(),
            actual: "B".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });
        engine.record_override("intent", "A", "test");
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Correction,
            content: "test".to_string(),
            source: "test".to_string(),
            timestamp: Utc::now(),
        });

        engine.clear();
        assert_eq!(engine.get_correction_count(), 0);
        assert_eq!(engine.get_override_count(), 0);
        assert!(engine.get_pending().is_empty());
    }

    #[test]
    fn test_correction_summary() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "A".to_string(),
            actual: "B".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "A".to_string(),
            actual: "C".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::TechnologyCorrection,
            expected: "Rust".to_string(),
            actual: "Go".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });

        let summary = engine.get_correction_summary();
        assert!(summary.contains("intent_correction"));
        assert!(summary.contains("technology_correction"));
        assert!(summary.contains("3")); // Total
    }

    #[test]
    fn test_recent_corrections() {
        let mut engine = HumanFeedbackEngine::new();
        for i in 0..5 {
            engine.record_correction(CorrectionRecord {
                correction_type: CorrectionType::IntentCorrection,
                expected: format!("expected_{}", i),
                actual: format!("actual_{}", i),
                timestamp: Utc::now(),
                context: None,
                applied: false,
            });
        }

        let recent = engine.recent_corrections(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_apply_corrections() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "A".to_string(),
            actual: "B".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "C".to_string(),
            actual: "D".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });

        let applied = engine.apply_corrections();
        assert_eq!(applied, 2);

        // All corrections should now be marked applied
        for c in engine.get_all_corrections() {
            assert!(c.applied);
        }
    }

    #[test]
    fn test_no_override_for_nonexistent_target() {
        let engine = HumanFeedbackEngine::new();
        assert!(!engine.has_override("nonexistent"));
        assert!(engine.get_override_value("nonexistent").is_none());
        assert_eq!(
            engine.apply_overrides("ai_value", "nonexistent"),
            "ai_value"
        );
    }

    #[test]
    fn test_process_multiple_pending() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Correction,
            content: "expected=A, actual=B".to_string(),
            source: "cli".to_string(),
            timestamp: Utc::now(),
        });
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Clarification,
            content: "clarified".to_string(),
            source: "ui".to_string(),
            timestamp: Utc::now(),
        });
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Confirmation,
            content: "confirmed".to_string(),
            source: "cli".to_string(),
            timestamp: Utc::now(),
        });

        let results = engine.process_pending();
        // Correction + Clarification = 2 results; Confirmation = 0
        assert_eq!(results.len(), 2);
        assert_eq!(engine.get_pending().len(), 0);
    }

    #[test]
    fn test_most_corrected_intent() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "deployment".to_string(),
            actual: "troubleshooting".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });
        engine.record_correction(CorrectionRecord {
            correction_type: CorrectionType::IntentCorrection,
            expected: "deployment".to_string(),
            actual: "configuration".to_string(),
            timestamp: Utc::now(),
            context: None,
            applied: false,
        });

        let intent = engine.most_corrected_intent();
        assert_eq!(intent, Some(Intent::Deployment));
    }

    #[test]
    fn test_process_pending_without_correction_data() {
        let mut engine = HumanFeedbackEngine::new();
        engine.record_feedback(FeedbackRecord {
            feedback_type: FeedbackType::Correction,
            content: "just a simple correction".to_string(),
            source: "cli".to_string(),
            timestamp: Utc::now(),
        });

        let results = engine.process_pending();
        assert!(!results.is_empty());
    }
}
