/// Feature 1 — Guidance Decision Engine
///
/// Responsible for deciding:
/// - When to provide guidance.
/// - What guidance is relevant.
/// - How detailed guidance should be.
/// - Whether interruption is appropriate.
///
/// The AI does not interrupt unnecessarily.

use serde::{Deserialize, Serialize};

/// How detailed the guidance should be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailLevel {
    /// Minimal — one-liner, no explanation.
    Minimal,
    /// Standard — explanation included.
    Standard,
    /// Comprehensive — full context, background, and alternatives.
    Comprehensive,
}

/// How detailed the guidance should be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum GuidanceMode {
    /// Balanced — default mode.
    #[default]
    Balanced,
    /// Teaching — provide thorough explanations.
    Teaching,
    /// Expert — brief, high-signal guidance only.
    Expert,
    /// Silent — guidance only when explicitly requested.
    Silent,
}


/// Result of the guidance decision evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidanceDecision {
    /// Whether guidance should be provided.
    pub should_guidance: bool,
    /// How detailed the guidance should be.
    pub detail_level: DetailLevel,
    /// Whether the engineer should be interrupted.
    pub should_interrupt: bool,
    /// Reasoning for this decision.
    pub reasoning: Vec<String>,
}

/// Decision criteria evaluated by the Guidance Decision Engine.
pub struct DecisionCriteria {
    /// Current screen context (active application, URL, window title).
    pub screen_context: Option<ScreenContext>,
    /// Currently detected technology.
    pub current_technology: Option<String>,
    /// Current workflow stage (e.g., evidence collection, diagnosis).
    pub workflow_stage: Option<String>,
    /// Whether the engineer is actively typing or interacting.
    pub is_engineer_active: bool,
    /// Available knowledge sources relevant to current context.
    pub has_knowledge_sources: bool,
    /// Confidence level of the current detection.
    pub confidence: f64,
    /// Previous recommendations already shown in this session.
    pub previous_recommendations: Vec<String>,
    /// Engineer preferences (mode, detail level).
    pub guidance_mode: GuidanceMode,
}

/// Screen context information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenContext {
    /// Active application name.
    pub application: String,
    /// Window title.
    pub window_title: String,
    /// Browser URL if applicable.
    pub url: Option<String>,
}

impl Default for DecisionCriteria {
    fn default() -> Self {
        Self {
            screen_context: None,
            current_technology: None,
            workflow_stage: None,
            is_engineer_active: false,
            has_knowledge_sources: false,
            confidence: 0.5,
            previous_recommendations: Vec::new(),
            guidance_mode: GuidanceMode::default(),
        }
    }
}

/// The Guidance Decision Engine.
pub struct GuidanceDecisionEngine {
    /// Configurable minimum confidence threshold for guidance.
    min_confidence: f64,
    /// Minimum detail level for guidance.
    min_detail_level: DetailLevel,
}

impl Default for GuidanceDecisionEngine {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            min_detail_level: DetailLevel::Minimal,
        }
    }
}

impl GuidanceDecisionEngine {
    /// Create a new Guidance Decision Engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Guidance Decision Engine with custom thresholds.
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.min_confidence = threshold;
        self
    }

    /// Evaluate whether guidance should be provided based on criteria.
    pub fn evaluate(&self, criteria: &DecisionCriteria) -> GuidanceDecision {
        let mut reasoning = Vec::new();
        let mut should_guidance = true;
        let mut should_interrupt = false;
        let mut detail_level = DetailLevel::Standard;

        // Check minimum confidence threshold
        if criteria.confidence < self.min_confidence {
            reasoning.push(format!(
                "Confidence {:.2} below threshold {:.2} — guidance limited",
                criteria.confidence, self.min_confidence
            ));
            detail_level = DetailLevel::Minimal;
        }

        // Check engineer activity — do not interrupt while typing
        if criteria.is_engineer_active {
            should_interrupt = false;
            reasoning.push("Engineer is active — deferring guidance".to_string());
        } else {
            // Only interrupt for high-confidence guidance when idle
            if criteria.confidence >= 0.8 {
                should_interrupt = true;
            }
        }

        // Check if previous recommendation matches — avoid repetition
        let has_knowledge = criteria.has_knowledge_sources || criteria.current_technology.is_some();
        if !has_knowledge {
            reasoning.push("No knowledge sources or technology detected — minimal guidance".to_string());
            detail_level = DetailLevel::Minimal;
        }

        // Set detail level based on guidance mode
        match criteria.guidance_mode {
            GuidanceMode::Teaching => {
                detail_level = DetailLevel::Comprehensive;
                reasoning.push("Teaching mode enabled — comprehensive guidance".to_string());
            }
            GuidanceMode::Expert => {
                detail_level = DetailLevel::Minimal;
                reasoning.push("Expert mode enabled — minimal guidance".to_string());
            }
            GuidanceMode::Silent => {
                should_guidance = false;
                should_interrupt = false;
                detail_level = DetailLevel::Minimal;
                reasoning.push("Silent mode — guidance only on request".to_string());
            }
            _ => {} // Balanced defaults
        }

        // Set detail level based on minimum requirement
        match (detail_level, &self.min_detail_level) {
            (DetailLevel::Minimal, DetailLevel::Standard | DetailLevel::Comprehensive) => {
                detail_level = DetailLevel::Standard;
            }
            (DetailLevel::Comprehensive, DetailLevel::Minimal | DetailLevel::Standard) => {
                detail_level = DetailLevel::Comprehensive;
            }
            _ => {}
        }

        if should_guidance && detail_level == DetailLevel::Standard {
            reasoning.push("Guidance provided at standard detail level".to_string());
        }

        GuidanceDecision {
            should_guidance,
            detail_level,
            should_interrupt,
            reasoning,
        }
    }

    /// Check whether guidance should interrupt the engineer.
    pub fn should_interrupt(&self, confidence: f64, is_active: bool, mode: &GuidanceMode) -> bool {
        // Never interrupt while engineer is active
        if is_active {
            return false;
        }

        // Silent mode never interrupts
        if matches!(mode, GuidanceMode::Silent) {
            return false;
        }

        // High confidence only when idle
        confidence >= 0.8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guidance_when_idle_and_high_confidence() {
        let engine = GuidanceDecisionEngine::new();
        let criteria = DecisionCriteria {
            confidence: 0.9,
            is_engineer_active: false,
            has_knowledge_sources: true,
            guidance_mode: GuidanceMode::Balanced,
            ..Default::default()
        };

        let decision = engine.evaluate(&criteria);
        assert!(decision.should_guidance);
        assert!(decision.should_interrupt);
        assert_eq!(decision.detail_level, DetailLevel::Standard);
    }

    #[test]
    fn test_guidance_when_engineer_active() {
        let engine = GuidanceDecisionEngine::new();
        let criteria = DecisionCriteria {
            confidence: 0.9,
            is_engineer_active: true,
            has_knowledge_sources: true,
            guidance_mode: GuidanceMode::Balanced,
            ..Default::default()
        };

        let decision = engine.evaluate(&criteria);
        assert!(decision.should_guidance);
        assert!(!decision.should_interrupt);
    }

    #[test]
    fn test_silent_mode_no_guidance() {
        let engine = GuidanceDecisionEngine::new();
        let criteria = DecisionCriteria {
            confidence: 0.95,
            is_engineer_active: false,
            has_knowledge_sources: true,
            guidance_mode: GuidanceMode::Silent,
            ..Default::default()
        };

        let decision = engine.evaluate(&criteria);
        assert!(!decision.should_guidance);
        assert!(!decision.should_interrupt);
    }

    #[test]
    fn test_low_confidence_minimal_detail() {
        let engine = GuidanceDecisionEngine::new();
        let criteria = DecisionCriteria {
            confidence: 0.3,
            is_engineer_active: false,
            has_knowledge_sources: false,
            guidance_mode: GuidanceMode::Balanced,
            ..Default::default()
        };

        let decision = engine.evaluate(&criteria);
        assert!(decision.should_guidance);
        assert_eq!(decision.detail_level, DetailLevel::Minimal);
        assert!(!decision.should_interrupt);
    }

    #[test]
    fn test_teaching_mode_comprehensive() {
        let engine = GuidanceDecisionEngine::new();
        let criteria = DecisionCriteria {
            confidence: 0.8,
            is_engineer_active: false,
            has_knowledge_sources: true,
            guidance_mode: GuidanceMode::Teaching,
            ..Default::default()
        };

        let decision = engine.evaluate(&criteria);
        assert!(decision.should_guidance);
        assert_eq!(decision.detail_level, DetailLevel::Comprehensive);
    }

    #[test]
    fn test_expert_mode_minimal() {
        let engine = GuidanceDecisionEngine::new();
        let criteria = DecisionCriteria {
            confidence: 0.8,
            is_engineer_active: false,
            has_knowledge_sources: true,
            guidance_mode: GuidanceMode::Expert,
            ..Default::default()
        };

        let decision = engine.evaluate(&criteria);
        assert!(decision.should_guidance);
        assert_eq!(decision.detail_level, DetailLevel::Minimal);
    }

    #[test]
    fn test_should_interrupt_never_when_active() {
        let engine = GuidanceDecisionEngine::new();
        assert!(!engine.should_interrupt(0.99, true, &GuidanceMode::Balanced));
        assert!(!engine.should_interrupt(0.99, true, &GuidanceMode::Teaching));
    }

    #[test]
    fn test_should_interrupt_idle_high_confidence() {
        let engine = GuidanceDecisionEngine::new();
        assert!(engine.should_interrupt(0.9, false, &GuidanceMode::Balanced));
        assert!(engine.should_interrupt(0.85, false, &GuidanceMode::Teaching));
    }

    #[test]
    fn test_should_interrupt_no_when_low_confidence() {
        let engine = GuidanceDecisionEngine::new();
        assert!(!engine.should_interrupt(0.5, false, &GuidanceMode::Balanced));
        assert!(!engine.should_interrupt(0.7, false, &GuidanceMode::Teaching));
    }

    #[test]
    fn test_should_interrupt_silent_mode() {
        let engine = GuidanceDecisionEngine::new();
        assert!(!engine.should_interrupt(0.99, false, &GuidanceMode::Silent));
    }
}