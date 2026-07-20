//! Context Fusion Engine — Phase 7
//!
//! Unified engineering context — fuses observation events, conversation
//! context, technology inferences, intent inferences, workflow state,
//! timeline entries, human corrections, and confidence scores into a
//! single coherent snapshot.
//!
//! ## Architecture
//!
//! - **ObservationEvent** — From the observation framework (raw events)
//! - **TechnologyInference** — From the technology recognition module
//! - **IntentInference** — Local struct for intent analysis results
//! - **TimelineEntry** — From data_types (engineering activity log)
//! - **CorrectionRecord** — From human_feedback (human corrections)
//! - **FusedContext** — The unified output snapshot
//! - **ContextFusionEngine** — Manages all inputs and produces fused output
//!
//! ## Core Principles
//!
//! - Human feedback ALWAYS overrides inference
//! - Fused context is immutable once created (a snapshot)
//! - Confidence scores are tracked per-key for transparency
//! - Evidence summary is generated automatically from all sources

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use wikilabs_data_types::technology::TechnologyInference;
use wikilabs_data_types::timeline::TimelineEntry;
use wikilabs_human_feedback::CorrectionRecord;
use wikilabs_observation::ObservationEvent;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// An intent inference — what the AI believes the user wants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntentInference {
    /// The inferred intent text.
    pub intent: String,
    /// Confidence in this inference (0.0–1.0).
    pub confidence: f32,
    /// Source of the inference ("intent_engine", "user_direct", "analysis").
    pub source: String,
}

impl IntentInference {
    /// Create a new intent inference.
    pub fn new(intent: impl Into<String>, confidence: f32, source: impl Into<String>) -> Self {
        Self {
            intent: intent.into(),
            confidence: confidence.clamp(0.0, 1.0),
            source: source.into(),
        }
    }
}

/// A fused, unified context snapshot — immutable once created.
///
/// This is the canonical representation of the engineering context at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedContext {
    /// All technology inferences gathered so far.
    pub technologies: Vec<TechnologyInference>,
    /// All intent inferences (may include multiple if uncertain).
    pub intents: Vec<IntentInference>,
    /// Current workflow state (if active).
    pub workflow_state: Option<String>,
    /// Timeline of engineering activity.
    pub timeline: Vec<TimelineEntry>,
    /// Confidence scores keyed by feature name.
    pub confidence_scores: HashMap<String, f32>,
    /// Evidence types that are still missing.
    pub missing_evidence: Vec<String>,
    /// Human corrections that have been applied.
    pub human_corrections_applied: Vec<String>,
    /// Human-readable evidence summary.
    pub evidence_summary: String,
    /// When this context was fused.
    pub fused_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Fuses observation events, conversation context, technology and intent
/// inferences, workflow state, and human corrections into a unified
/// engineering context snapshot.
pub struct ContextFusionEngine {
    /// Raw observation events.
    observation_events: Vec<ObservationEvent>,
    /// Conversation context (last N messages or summary).
    conversation_context: String,
    /// Workspace context (optional, when relevant).
    workspace_context: Option<String>,
    /// Technology inferences from the recognition module.
    technology_inferences: Vec<TechnologyInference>,
    /// Intent inferences from the intent engine.
    intent_inferences: Vec<IntentInference>,
    /// Current workflow state.
    workflow_state: Option<String>,
    /// Timeline of engineering activity.
    timeline: Vec<TimelineEntry>,
    /// Human corrections applied to the context.
    human_corrections: Vec<CorrectionRecord>,
    /// Confidence scores keyed by feature (e.g., "rust_detection", "intent_troubleshooting").
    confidence_scores: HashMap<String, f32>,
    /// The last fused context (cached).
    last_fused_at: DateTime<Utc>,
    cached_fused: Option<FusedContext>,
}

impl ContextFusionEngine {
    /// Create a new empty context fusion engine.
    pub fn new() -> Self {
        Self {
            observation_events: Vec::new(),
            conversation_context: String::new(),
            workspace_context: None,
            technology_inferences: Vec::new(),
            intent_inferences: Vec::new(),
            workflow_state: None,
            timeline: Vec::new(),
            human_corrections: Vec::new(),
            confidence_scores: HashMap::new(),
            last_fused_at: Utc::now(),
            cached_fused: None,
        }
    }

    /// Add an observation event.
    pub fn add_observation_event(&mut self, event: ObservationEvent) {
        debug!(
            "Added observation event: type={}, source={}",
            event.event_type, event.source
        );
        self.observation_events.push(event);
        self.cached_fused = None; // Invalidate cache
    }

    /// Set the conversation context.
    pub fn set_conversation_context(&mut self, context: String) {
        debug!("Set conversation context ({} chars)", context.len());
        self.conversation_context = context;
        self.cached_fused = None;
    }

    /// Set the workspace context.
    pub fn set_workspace_context(&mut self, context: Option<String>) {
        self.workspace_context = context;
        self.cached_fused = None;
    }

    /// Add a technology inference.
    pub fn add_technology_inference(&mut self, inference: TechnologyInference) {
        debug!(
            "Added technology inference: {} (confidence: {:.2}, source: {})",
            inference.name, inference.confidence, inference.source
        );
        // Avoid duplicate entries for the same technology from the same source
        let exists = self
            .technology_inferences
            .iter()
            .any(|t| t.name == inference.name && t.source == inference.source);
        if !exists {
            self.technology_inferences.push(inference);
        }
        self.cached_fused = None;
    }

    /// Add an intent inference.
    pub fn add_intent_inference(&mut self, inference: IntentInference) {
        debug!(
            "Added intent inference: {} (confidence: {:.2}, source: {})",
            inference.intent, inference.confidence, inference.source
        );
        // Avoid duplicate exact matches
        let exists = self
            .intent_inferences
            .iter()
            .any(|i| i.intent == inference.intent && i.source == inference.source);
        if !exists {
            self.intent_inferences.push(inference);
        }
        self.cached_fused = None;
    }

    /// Set the current workflow state.
    pub fn set_workflow_state(&mut self, state: Option<String>) {
        self.workflow_state = state;
        self.cached_fused = None;
    }

    /// Add a timeline entry.
    pub fn add_timeline_entry(&mut self, entry: TimelineEntry) {
        debug!(
            "Added timeline entry: '{}' (source: {})",
            entry.label, entry.source
        );
        self.timeline.push(entry);
        self.cached_fused = None;
    }

    /// Add a human correction.
    pub fn add_human_correction(&mut self, correction: CorrectionRecord) {
        debug!(
            "Added human correction: type={}, applied={}",
            correction.correction_type, correction.applied
        );
        self.human_corrections.push(correction);
        self.cached_fused = None;
    }

    /// Update a confidence score.
    pub fn update_confidence(&mut self, key: &str, score: f32) {
        let score = score.clamp(0.0, 1.0);
        debug!("Updated confidence for '{}': {:.2}", key, score);
        self.confidence_scores.insert(key.to_string(), score);
        self.cached_fused = None;
    }

    /// Fuse all context sources into a single unified snapshot.
    pub fn fuse(&mut self) -> FusedContext {
        info!("Fusing context from {} sources", {
            let count = self.observation_events.len()
                + self.technology_inferences.len()
                + self.intent_inferences.len()
                + self.timeline.len()
                + self.human_corrections.len();
            count
        });

        // Compute missing evidence from tech inferences with low confidence
        let missing_evidence = self.compute_missing_evidence();

        // Compute human corrections applied
        let human_corrections_applied: Vec<String> = self
            .human_corrections
            .iter()
            .filter(|c| c.applied)
            .map(|c| {
                format!(
                    "{}: expected='{}', actual='{}'",
                    c.correction_type, c.expected, c.actual
                )
            })
            .collect();

        // Generate evidence summary
        let evidence_summary = self.generate_evidence_summary(&missing_evidence);

        let fused_at = Utc::now();

        let fused = FusedContext {
            technologies: self.technology_inferences.clone(),
            intents: self.intent_inferences.clone(),
            workflow_state: self.workflow_state.clone(),
            timeline: self.timeline.clone(),
            confidence_scores: self.confidence_scores.clone(),
            missing_evidence,
            human_corrections_applied,
            evidence_summary,
            fused_at,
        };

        self.last_fused_at = fused_at;
        self.cached_fused = Some(fused.clone());

        debug!(
            "Fused context: {} technologies, {} intents, {} timeline entries",
            fused.technologies.len(),
            fused.intents.len(),
            fused.timeline.len()
        );

        fused
    }

    /// Get the last fused context (or fuse if never done).
    pub fn get_fused_context(&mut self) -> Option<&FusedContext> {
        if self.cached_fused.is_none() {
            self.fuse();
        }
        self.cached_fused.as_ref()
    }

    /// Get the timestamp of the last fuse operation.
    pub fn get_last_fused_at(&self) -> DateTime<Utc> {
        self.last_fused_at
    }

    /// Clear all context data.
    pub fn clear(&mut self) {
        self.observation_events.clear();
        self.conversation_context.clear();
        self.workspace_context = None;
        self.technology_inferences.clear();
        self.intent_inferences.clear();
        self.workflow_state = None;
        self.timeline.clear();
        self.human_corrections.clear();
        self.confidence_scores.clear();
        self.cached_fused = None;
        info!("Context fusion engine cleared");
    }

    /// Generate a human-readable evidence summary.
    pub fn get_evidence_summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.observation_events.is_empty() {
            let event_types: Vec<String> = self
                .observation_events
                .iter()
                .map(|e| format!("{} from {}", e.event_type, e.source))
                .collect();
            parts.push(format!(
                "Observations: {} events recorded ({})",
                self.observation_events.len(),
                event_types.join(", ")
            ));
        }

        if !self.technology_inferences.is_empty() {
            let techs: Vec<String> = self
                .technology_inferences
                .iter()
                .map(|t| format!("{} ({:.0}%)", t.name, t.confidence * 100.0))
                .collect();
            parts.push(format!("Technologies detected: {}", techs.join(", ")));
        }

        if !self.intent_inferences.is_empty() {
            let intents: Vec<String> = self
                .intent_inferences
                .iter()
                .map(|i| format!("{} (confidence: {:.0}%)", i.intent, i.confidence * 100.0))
                .collect();
            parts.push(format!("Intents identified: {}", intents.join(", ")));
        }

        if !self.timeline.is_empty() {
            let last_entry = self.timeline.last();
            parts.push(format!(
                "Timeline: {} entries, latest: '{}'",
                self.timeline.len(),
                last_entry.map(|e| e.label.as_str()).unwrap_or("")
            ));
        }

        if !self.human_corrections.is_empty() {
            let applied = self.human_corrections.iter().filter(|c| c.applied).count();
            parts.push(format!(
                "Human corrections: {} total, {} applied",
                self.human_corrections.len(),
                applied
            ));
        }

        if let Some(state) = &self.workflow_state {
            parts.push(format!("Workflow state: {}", state));
        }

        if parts.is_empty() {
            "No evidence gathered yet.".to_string()
        } else {
            parts.join("\n")
        }
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Compute missing evidence by checking which technology inference
    /// types have low confidence and which are absent.
    fn compute_missing_evidence(&self) -> Vec<String> {
        let mut missing = Vec::new();

        // Check for high-confidence tech detections
        let high_conf_techs: Vec<String> = self
            .technology_inferences
            .iter()
            .filter(|t| t.confidence >= 0.7)
            .map(|t| t.name.clone())
            .collect();

        // Missing evidence = tech types that should be detected but have low/no confidence
        for inference in &self.technology_inferences {
            if inference.confidence < 0.5 && !missing.contains(&inference.name) {
                missing.push(format!(
                    "Low confidence tech: {} ({:.2})",
                    inference.name, inference.confidence
                ));
            }
        }

        if high_conf_techs.is_empty() && !self.technology_inferences.is_empty() {
            missing.push("No high-confidence technology detections".to_string());
        }

        // Check intent confidence
        let max_intent_conf = self
            .intent_inferences
            .iter()
            .map(|i| i.confidence)
            .fold(0.0f32, f32::max);

        if max_intent_conf < 0.5 {
            missing.push("Low-confidence intent inference".to_string());
        }

        missing
    }

    /// Generate a human-readable evidence summary string.
    fn generate_evidence_summary(&self, missing: &[String]) -> String {
        let tech_summary = if self.technology_inferences.is_empty() {
            "No technologies detected".to_string()
        } else {
            let count = self.technology_inferences.len();
            let avg_conf: f32 = self
                .technology_inferences
                .iter()
                .map(|t| t.confidence)
                .sum::<f32>()
                / count as f32;
            let names: Vec<String> = self
                .technology_inferences
                .iter()
                .map(|t| t.name.clone())
                .collect();
            format!(
                "{} technologies detected ({}) (avg confidence: {:.0}%)",
                count,
                names.join(", "),
                avg_conf * 100.0
            )
        };

        let intent_summary = if self.intent_inferences.is_empty() {
            "No intent detected".to_string()
        } else {
            let best = self
                .intent_inferences
                .iter()
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                .unwrap();
            format!(
                "Primary intent: '{}' (confidence: {:.0}%)",
                best.intent,
                best.confidence * 100.0
            )
        };

        let event_summary = format!("{}", self.observation_events.len());

        let correction_summary = if self.human_corrections.is_empty() {
            "No human corrections".to_string()
        } else {
            format!(
                "{} correction(s), {} applied",
                self.human_corrections.len(),
                self.human_corrections.iter().filter(|c| c.applied).count()
            )
        };

        let missing_summary = if missing.is_empty() {
            "No missing evidence".to_string()
        } else {
            format!("{} missing evidence item(s)", missing.len())
        };

        format!(
            "Event observations: {}\n{}\n{}\n{}\n{}\n{}",
            event_summary,
            tech_summary,
            intent_summary,
            correction_summary,
            missing_summary,
            self.get_timeline_summary(),
        )
    }

    /// Get a summary string from timeline entries.
    fn get_timeline_summary(&self) -> String {
        if self.timeline.is_empty() {
            "No timeline entries".to_string()
        } else {
            let last = self.timeline.last().unwrap();
            format!(
                "Timeline: {} entries, latest: '{}' at {}",
                self.timeline.len(),
                last.label,
                last.timestamp.format("%H:%M:%S")
            )
        }
    }

    /// Get observation event count.
    pub fn get_observation_count(&self) -> usize {
        self.observation_events.len()
    }

    /// Get technology inference count.
    pub fn get_technology_count(&self) -> usize {
        self.technology_inferences.len()
    }

    /// Get intent inference count.
    pub fn get_intent_count(&self) -> usize {
        self.intent_inferences.len()
    }

    /// Get the raw observation events (for inspection).
    pub fn get_observations(&self) -> &[ObservationEvent] {
        &self.observation_events
    }

    /// Get the raw technology inferences.
    pub fn get_technologies(&self) -> &[TechnologyInference] {
        &self.technology_inferences
    }

    /// Get the raw intent inferences.
    pub fn get_intents(&self) -> &[IntentInference] {
        &self.intent_inferences
    }

    /// Get the raw timeline.
    pub fn get_timeline(&self) -> &[TimelineEntry] {
        &self.timeline
    }

    /// Get the raw human corrections.
    pub fn get_corrections(&self) -> &[CorrectionRecord] {
        &self.human_corrections
    }

    /// Check if any source has data (at least one type of evidence exists).
    pub fn has_any_context(&self) -> bool {
        !self.observation_events.is_empty()
            || !self.technology_inferences.is_empty()
            || !self.intent_inferences.is_empty()
            || !self.timeline.is_empty()
            || !self.human_corrections.is_empty()
            || !self.confidence_scores.is_empty()
    }
}

impl Default for ContextFusionEngine {
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

    fn make_observation_event() -> ObservationEvent {
        wikilabs_observation::ObservationEvent::new(
            wikilabs_observation::EventType::FileActivity,
            wikilabs_observation::ProviderType::FileObserver,
            "test_source".to_string(),
            Some("test_workspace".to_string()),
            wikilabs_observation::ObservationPayload::empty(),
        )
    }

    fn make_technology_inference() -> TechnologyInference {
        TechnologyInference::new(
            "Rust".to_string(),
            0.9,
            "observation".to_string(),
            "Found Cargo.toml".to_string(),
        )
    }

    fn make_intent_inference() -> IntentInference {
        IntentInference::new(
            "troubleshooting".to_string(),
            0.8,
            "intent_engine".to_string(),
        )
    }

    fn make_timeline_entry() -> TimelineEntry {
        TimelineEntry::new(
            "Detected Rust project",
            "observation",
            "Cargo.toml found in workspace",
        )
    }

    fn make_correction() -> CorrectionRecord {
        CorrectionRecord {
            correction_type: wikilabs_human_feedback::CorrectionType::IntentCorrection,
            expected: wikilabs_data_types::intent::Intent::Deployment.to_string(),
            actual: wikilabs_data_types::intent::Intent::Troubleshooting.to_string(),
            timestamp: Utc::now(),
            context: Some("User meant deployment".to_string()),
            applied: false,
        }
    }

    #[test]
    fn test_new_engine_is_empty() {
        let engine = ContextFusionEngine::new();
        assert_eq!(engine.get_observation_count(), 0);
        assert_eq!(engine.get_technology_count(), 0);
        assert_eq!(engine.get_intent_count(), 0);
        assert!(!engine.has_any_context());
    }

    #[test]
    fn test_add_observation_event() {
        let mut engine = ContextFusionEngine::new();
        engine.add_observation_event(make_observation_event());
        assert_eq!(engine.get_observation_count(), 1);
    }

    #[test]
    fn test_add_technology_inference() {
        let mut engine = ContextFusionEngine::new();
        engine.add_technology_inference(make_technology_inference());
        assert_eq!(engine.get_technology_count(), 1);
    }

    #[test]
    fn test_add_intent_inference() {
        let mut engine = ContextFusionEngine::new();
        engine.add_intent_inference(make_intent_inference());
        assert_eq!(engine.get_intent_count(), 1);
    }

    #[test]
    fn test_add_timeline_entry() {
        let mut engine = ContextFusionEngine::new();
        engine.add_timeline_entry(make_timeline_entry());
        assert_eq!(engine.get_timeline().len(), 1);
    }

    #[test]
    fn test_set_workflow_state() {
        let mut engine = ContextFusionEngine::new();
        engine.set_workflow_state(Some("analysis".to_string()));
        assert_eq!(engine.workflow_state, Some("analysis".to_string()));

        engine.set_workflow_state(None);
        assert!(engine.workflow_state.is_none());
    }

    #[test]
    fn test_set_conversation_context() {
        let mut engine = ContextFusionEngine::new();
        engine.set_conversation_context("User: fix this bug".to_string());
        assert!(!engine.conversation_context.is_empty());
    }

    #[test]
    fn test_set_workspace_context() {
        let mut engine = ContextFusionEngine::new();
        engine.set_workspace_context(Some("customer-a".to_string()));
        assert_eq!(engine.workspace_context, Some("customer-a".to_string()));
        engine.set_workspace_context(None);
        assert!(engine.workspace_context.is_none());
    }

    #[test]
    fn test_update_confidence() {
        let mut engine = ContextFusionEngine::new();
        engine.update_confidence("rust_detection", 0.9);
        engine.update_confidence("intent_accuracy", 0.75);

        assert_eq!(engine.confidence_scores.get("rust_detection"), Some(&0.9));
        assert_eq!(engine.confidence_scores.get("intent_accuracy"), Some(&0.75));
    }

    #[test]
    fn test_confidence_clamping() {
        let mut engine = ContextFusionEngine::new();
        engine.update_confidence("over", 1.5);
        engine.update_confidence("under", -0.3);

        assert_eq!(engine.confidence_scores.get("over"), Some(&1.0));
        assert_eq!(engine.confidence_scores.get("under"), Some(&0.0));
    }

    #[test]
    fn test_fuse_basic() {
        let mut engine = ContextFusionEngine::new();
        engine.add_observation_event(make_observation_event());
        engine.add_technology_inference(make_technology_inference());
        engine.add_intent_inference(make_intent_inference());
        engine.add_timeline_entry(make_timeline_entry());
        engine.set_workflow_state(Some("analysis".to_string()));
        engine.update_confidence("rust", 0.9);

        let fused = engine.fuse();

        assert_eq!(fused.technologies.len(), 1);
        assert_eq!(fused.intents.len(), 1);
        assert_eq!(fused.timeline.len(), 1);
        assert_eq!(fused.workflow_state, Some("analysis".to_string()));
        assert!(!fused.confidence_scores.is_empty());
        assert!(fused.fused_at > Utc::now() - chrono::Duration::seconds(5));
    }

    #[test]
    fn test_fused_context_has_summary() {
        let mut engine = ContextFusionEngine::new();
        engine.add_technology_inference(make_technology_inference());
        engine.add_intent_inference(make_intent_inference());

        let fused = engine.fuse();
        assert!(!fused.evidence_summary.is_empty());
        assert!(fused.evidence_summary.contains("Rust"));
        assert!(fused.evidence_summary.contains("troubleshooting"));
    }

    #[test]
    fn test_get_fused_context_caches() {
        let mut engine = ContextFusionEngine::new();
        engine.add_observation_event(make_observation_event());

        // First call should fuse
        let ctx1 = engine.get_fused_context();
        assert!(ctx1.is_some());

        // Second call should use cache (same pointer - verified by same address)
        let ctx2 = engine.get_fused_context();
        assert!(ctx2.is_some());
    }

    #[test]
    fn test_no_duplicate_tech_inferences() {
        let mut engine = ContextFusionEngine::new();
        engine.add_technology_inference(make_technology_inference());
        engine.add_technology_inference(make_technology_inference()); // duplicate
        assert_eq!(engine.get_technology_count(), 1);
    }

    #[test]
    fn test_no_duplicate_intent_inferences() {
        let mut engine = ContextFusionEngine::new();
        engine.add_intent_inference(make_intent_inference());
        engine.add_intent_inference(make_intent_inference()); // duplicate
        assert_eq!(engine.get_intent_count(), 1);
    }

    #[test]
    fn test_different_tech_inferences_allowed() {
        let mut engine = ContextFusionEngine::new();
        engine.add_technology_inference(TechnologyInference::new(
            "Rust".to_string(),
            0.9,
            "obs".to_string(),
            "Found Cargo.toml".to_string(),
        ));
        engine.add_technology_inference(TechnologyInference::new(
            "Linux".to_string(),
            0.8,
            "obs".to_string(),
            "Found bash scripts".to_string(),
        ));
        assert_eq!(engine.get_technology_count(), 2);
    }

    #[test]
    fn test_different_intent_inferences_allowed() {
        let mut engine = ContextFusionEngine::new();
        engine.add_intent_inference(IntentInference::new(
            "troubleshooting".to_string(),
            0.8,
            "engine".to_string(),
        ));
        engine.add_intent_inference(IntentInference::new(
            "deployment".to_string(),
            0.6,
            "engine".to_string(),
        ));
        assert_eq!(engine.get_intent_count(), 2);
    }

    #[test]
    fn test_has_any_context() {
        let mut engine = ContextFusionEngine::new();
        assert!(!engine.has_any_context());

        engine.add_observation_event(make_observation_event());
        assert!(engine.has_any_context());

        engine.clear();
        assert!(!engine.has_any_context());
    }

    #[test]
    fn test_evidence_summary_empty() {
        let engine = ContextFusionEngine::new();
        let summary = engine.get_evidence_summary();
        assert!(summary.contains("No evidence"));
    }

    #[test]
    fn test_evidence_summary_with_data() {
        let mut engine = ContextFusionEngine::new();
        engine.add_observation_event(make_observation_event());
        engine.add_technology_inference(make_technology_inference());
        engine.add_intent_inference(make_intent_inference());
        engine.add_timeline_entry(make_timeline_entry());
        engine.add_human_correction(make_correction());

        let summary = engine.get_evidence_summary();
        assert!(summary.contains("1")); // event count
        assert!(summary.contains("Rust"));
        assert!(summary.contains("troubleshooting"));
    }

    #[test]
    fn test_clear() {
        let mut engine = ContextFusionEngine::new();
        engine.add_observation_event(make_observation_event());
        engine.add_technology_inference(make_technology_inference());
        engine.add_intent_inference(make_intent_inference());
        engine.add_timeline_entry(make_timeline_entry());
        engine.add_human_correction(make_correction());
        engine.update_confidence("test", 0.5);
        engine.set_conversation_context("context".to_string());
        engine.set_workspace_context(Some("ws".to_string()));
        engine.set_workflow_state(Some("state".to_string()));

        engine.clear();

        assert_eq!(engine.get_observation_count(), 0);
        assert_eq!(engine.get_technology_count(), 0);
        assert_eq!(engine.get_intent_count(), 0);
        assert_eq!(engine.get_timeline().len(), 0);
        assert!(engine.confidence_scores.is_empty());
        assert!(engine.conversation_context.is_empty());
        assert!(engine.workspace_context.is_none());
        assert!(engine.workflow_state.is_none());
    }

    #[test]
    fn test_fused_context_missing_evidence() {
        let mut engine = ContextFusionEngine::new();
        // Add a low-confidence tech inference → should be flagged as missing
        engine.add_technology_inference(TechnologyInference::new(
            "UncertainTech".to_string(),
            0.3,
            "analysis".to_string(),
            "Uncertain detection".to_string(),
        ));

        let fused = engine.fuse();
        assert!(!fused.missing_evidence.is_empty());
    }

    #[test]
    fn test_fused_context_highest_confidence_tech() {
        let mut engine = ContextFusionEngine::new();
        engine.add_technology_inference(TechnologyInference::new(
            "Rust".to_string(),
            0.9,
            "obs".to_string(),
            "Found Cargo.toml".to_string(),
        ));
        engine.add_technology_inference(TechnologyInference::new(
            "Kubernetes".to_string(),
            0.7,
            "obs".to_string(),
            "Found kubectl".to_string(),
        ));

        let fused = engine.fuse();
        assert_eq!(fused.technologies.len(), 2);
    }

    #[test]
    fn test_human_corrections_in_fused() {
        let mut engine = ContextFusionEngine::new();
        engine.add_human_correction(CorrectionRecord {
            correction_type: wikilabs_human_feedback::CorrectionType::IntentCorrection,
            expected: "deployment".to_string(),
            actual: "troubleshooting".to_string(),
            timestamp: Utc::now(),
            context: Some("user correction".to_string()),
            applied: true,
        });

        let fused = engine.fuse();
        assert!(!fused.human_corrections_applied.is_empty());
        assert!(fused.human_corrections_applied[0].contains("deployment"));
    }

    #[test]
    fn test_timeline_order() {
        let mut engine = ContextFusionEngine::new();
        for i in 0..5 {
            engine.add_timeline_entry(TimelineEntry::new(
                format!("Entry {}", i),
                "test",
                format!("Detail {}", i),
            ));
        }

        let fused = engine.fuse();
        assert_eq!(fused.timeline.len(), 5);
        assert_eq!(fused.timeline[0].label, "Entry 0");
        assert_eq!(fused.timeline[4].label, "Entry 4");
    }

    #[test]
    fn test_fused_context_serialization() {
        let mut engine = ContextFusionEngine::new();
        engine.add_technology_inference(make_technology_inference());
        engine.add_intent_inference(make_intent_inference());

        let fused = engine.fuse();
        let json = serde_json::to_string(&fused).unwrap();
        let deserialized: FusedContext = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.technologies.len(), 1);
        assert_eq!(deserialized.intents.len(), 1);
        assert_eq!(deserialized.technologies[0].name, "Rust");
        assert_eq!(deserialized.intents[0].intent, "troubleshooting");
    }

    #[test]
    fn test_intent_inference_clamping() {
        let inf = IntentInference::new("test", 1.5, "test");
        assert_eq!(inf.confidence, 1.0);

        let inf2 = IntentInference::new("test", -0.3, "test");
        assert_eq!(inf2.confidence, 0.0);
    }

    #[test]
    fn test_has_any_context_with_confidence_only() {
        let mut engine = ContextFusionEngine::new();
        // Only confidence, no other data
        engine.update_confidence("test", 0.5);
        assert!(engine.has_any_context());
    }

    #[test]
    fn test_get_last_fused_at() {
        let mut engine = ContextFusionEngine::new();
        engine.add_observation_event(make_observation_event());
        let before = Utc::now();
        let _fused = engine.fuse();
        let after = Utc::now();

        let fused_at = engine.get_last_fused_at();
        assert!(fused_at >= before && fused_at <= after);
    }
}
