//! Decision Engine — determines whether a recommendation should be shown.
//!
//! The Decision Engine is the gatekeeper for all recommendations.
//! It evaluates context, timing, confidence, and policy before
//! allowing a recommendation to be shown to the engineer.

use crate::{Confidence, Priority};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Context about the current engineer state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub is_idle: bool,
    pub is_typing: bool,
    pub current_workflow_state: Option<String>,
    pub recent_activity: Vec<String>,
    pub active_technologies: Vec<String>,
    pub session_recommendation_count: u32,
    pub last_recommendation_age_seconds: Option<f64>,
    pub confidence_history: Vec<f64>,
}

impl Default for DecisionContext {
    fn default() -> Self {
        DecisionContext {
            is_idle: true,
            is_typing: false,
            current_workflow_state: None,
            recent_activity: Vec::new(),
            active_technologies: Vec::new(),
            session_recommendation_count: 0,
            last_recommendation_age_seconds: None,
            confidence_history: Vec::new(),
        }
    }
}

/// Decision outcome from the Decision Engine.
#[derive(Debug, Clone)]
pub struct DecisionOutcome {
    pub should_show: bool,
    pub adjusted_priority: Priority,
    pub adjusted_confidence: Confidence,
    pub reasoning: String,
}

/// The Decision Engine — the gatekeeper for all recommendations.
///
/// Evaluates:
/// - Current workflow state
/// - Confidence scores
/// - Missing evidence
/// - Engineer activity (idle vs typing)
/// - Interruption timing
/// - Recommendation priority
/// - Recommendation history (duplicates)
/// - User preferences (via PolicyEngine)
/// - Session context
///
/// The Copilot never bypasses the Decision Engine.
pub struct DecisionEngine {
    /// Track shown recommendation IDs to avoid duplicates.
    shown_ids: HashSet<uuid::Uuid>,
    /// Maximum recommendations per session before throttling.
    max_per_session: u32,
    /// Minimum time between interruptions (seconds).
    min_interruption_seconds: f64,
    /// Minimum confidence below which recommendations are rejected.
    min_confidence: f64,
}

impl DecisionEngine {
    pub fn new() -> Self {
        DecisionEngine {
            shown_ids: HashSet::new(),
            max_per_session: 50,
            min_interruption_seconds: 15.0,
            min_confidence: 0.5,
        }
    }

    pub fn with_max_session(mut self, max: u32) -> Self {
        self.max_per_session = max;
        self
    }

    pub fn with_min_interruption(mut self, seconds: f64) -> Self {
        self.min_interruption_seconds = seconds;
        self
    }

    pub fn with_min_confidence(mut self, min: f64) -> Self {
        self.min_confidence = min;
        self
    }

    /// Make a decision about whether a recommendation should be shown.
    ///
    /// Returns a DecisionOutcome with the recommendation's adjusted priority,
    /// confidence, and reasoning.
    pub fn evaluate(
        &mut self,
        recommendation_id: uuid::Uuid,
        confidence: Confidence,
        priority: Priority,
        has_evidence: bool,
        ctx: &DecisionContext,
    ) -> DecisionOutcome {
        let mut reasoning = Vec::new();
        let mut adjusted_priority = priority;
        let mut adjusted_confidence = confidence;
        let mut should_show = true;

        // Rule 1: Never interrupt while engineer is typing
        if ctx.is_typing {
            should_show = false;
            reasoning.push("Engineer is actively typing — postponing".to_string());
        }

        // Rule 2: Respect minimum interruption time
        if let Some(age) = ctx.last_recommendation_age_seconds {
            if age < self.min_interruption_seconds {
                should_show = false;
                reasoning.push(format!(
                    "Only {:.0}s since last recommendation (min: {:.0}s)",
                    age, self.min_interruption_seconds
                ));
            }
        }

        // Rule 3: Respect session limits
        if self.shown_ids.len() as u32 >= self.max_per_session {
            should_show = false;
            reasoning.push("Session recommendation limit reached".to_string());
        }

        // Rule 4: Duplicate check
        if self.shown_ids.contains(&recommendation_id) {
            should_show = false;
            reasoning.push("Recommendation already shown this session".to_string());
        }

        // Rule 5: Missing evidence reduces confidence
        if !has_evidence {
            adjusted_confidence = Confidence::new(confidence.score * 0.7);
            if adjusted_confidence.is_low() {
                should_show = false;
                reasoning.push("Insufficient evidence — confidence too low".to_string());
            } else {
                reasoning.push("Low evidence — reduced confidence".to_string());
            }
        }

        // Rule 6: Critical items always pass through (even with low confidence)
        if priority == Priority::Critical {
            should_show = true;
            reasoning.push("Critical priority — always shown".to_string());
        }

        // Rule 6b: Reject if confidence is below minimum threshold
        if adjusted_confidence.score < self.min_confidence && priority != Priority::Critical {
            should_show = false;
            reasoning.push(format!(
                "Confidence {:.2} below minimum {:.2}",
                adjusted_confidence.score, self.min_confidence
            ));
        }

        // Rule 7: Adjust priority based on context
        if !ctx.is_idle && priority == Priority::Information {
            adjusted_priority = Priority::Information; // Keep as-is, won't show due to other rules
        }
        if ctx.is_idle && confidence.is_high() && priority == Priority::Suggestion {
            adjusted_priority = Priority::Warning;
            reasoning.push("High confidence + idle context — elevated priority".to_string());
        }

        // Rule 8: Check workflow relevance
        if let Some(workflow_state) = &ctx.current_workflow_state {
            if !workflow_state.is_empty() {
                reasoning.push(format!("Current workflow: {}", workflow_state));
            }
        }

        DecisionOutcome {
            should_show,
            adjusted_priority,
            adjusted_confidence,
            reasoning: reasoning.join("; "),
        }
    }

    /// Record that a recommendation was shown.
    pub fn record_shown(&mut self, recommendation_id: uuid::Uuid) {
        self.shown_ids.insert(recommendation_id);
    }

    /// Clear the shown tracking (e.g., at session boundary).
    pub fn clear_shown(&mut self) {
        self.shown_ids.clear();
    }

    /// Get the count of recommendations shown this session.
    pub fn shown_count(&self) -> usize {
        self.shown_ids.len()
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        DecisionEngine::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(typing: bool, idle: bool) -> DecisionContext {
        DecisionContext {
            is_typing: typing,
            is_idle: idle,
            ..Default::default()
        }
    }

    #[test]
    fn test_never_interruption_while_typing() {
        let mut engine = DecisionEngine::new();
        let rec_id = uuid::Uuid::new_v4();
        let ctx = make_ctx(true, false);
        let outcome = engine.evaluate(
            rec_id,
            Confidence::new(0.95),
            Priority::Suggestion,
            true,
            &ctx,
        );
        assert!(!outcome.should_show, "Should not show while typing");
        assert!(outcome.reasoning.contains("typing"));
    }

    #[test]
    fn test_critical_always_shows() {
        let mut engine = DecisionEngine::new();
        let rec_id = uuid::Uuid::new_v4();
        let ctx = make_ctx(true, false);
        let outcome = engine.evaluate(
            rec_id,
            Confidence::new(0.3),
            Priority::Critical,
            false,
            &ctx,
        );
        assert!(
            outcome.should_show,
            "Critical always shows even with low confidence"
        );
    }

    #[test]
    fn test_no_evidence_reduces_confidence() {
        let mut engine = DecisionEngine::new();
        let rec_id = uuid::Uuid::new_v4();
        let ctx = make_ctx(false, true);
        let high = Confidence::new(0.5);
        let outcome = engine.evaluate(rec_id, high, Priority::Suggestion, false, &ctx);
        assert!(
            !outcome.should_show,
            "Should not show with no evidence and reduced confidence"
        );
    }

    #[test]
    fn test_high_confidence_elevates_suggestion() {
        let mut engine = DecisionEngine::new();
        let rec_id = uuid::Uuid::new_v4();
        let ctx = make_ctx(false, true);
        let outcome = engine.evaluate(
            rec_id,
            Confidence::new(0.9),
            Priority::Suggestion,
            true,
            &ctx,
        );
        assert_eq!(outcome.adjusted_priority, Priority::Warning);
    }

    #[test]
    fn test_duplicate_detection() {
        let mut engine = DecisionEngine::new();
        let rec_id = uuid::Uuid::new_v4();
        let ctx = make_ctx(false, true);
        engine.evaluate(
            rec_id,
            Confidence::new(0.9),
            Priority::Suggestion,
            true,
            &ctx,
        );
        engine.record_shown(rec_id);
        let outcome = engine.evaluate(
            rec_id,
            Confidence::new(0.9),
            Priority::Suggestion,
            true,
            &ctx,
        );
        assert!(!outcome.should_show, "Should not show duplicate");
    }

    #[test]
    fn test_session_limit() {
        let mut engine = DecisionEngine::new().with_max_session(2);
        let ctx = make_ctx(false, true);
        engine.evaluate(
            uuid::Uuid::new_v4(),
            Confidence::new(0.9),
            Priority::Suggestion,
            true,
            &ctx,
        );
        engine.record_shown(uuid::Uuid::new_v4());
        engine.evaluate(
            uuid::Uuid::new_v4(),
            Confidence::new(0.9),
            Priority::Suggestion,
            true,
            &ctx,
        );
        engine.record_shown(uuid::Uuid::new_v4());
        let outcome = engine.evaluate(
            uuid::Uuid::new_v4(),
            Confidence::new(0.9),
            Priority::Suggestion,
            true,
            &ctx,
        );
        assert!(!outcome.should_show, "Should respect session limit");
    }

    #[test]
    fn test_low_confidence_rejected() {
        let mut engine = DecisionEngine::new();
        let rec_id = uuid::Uuid::new_v4();
        let ctx = make_ctx(false, true);
        let outcome = engine.evaluate(
            rec_id,
            Confidence::new(0.2),
            Priority::Suggestion,
            true,
            &ctx,
        );
        assert!(
            !outcome.should_show,
            "Should not show low confidence recommendation"
        );
    }
}
