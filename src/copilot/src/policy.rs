//! Recommendation policy engine.
//!
//! Policies control how aggressively the copilot interrupts
//! and how much detail it provides.
//!
//! - **Minimal** — Only critical interruptions
//! - **Balanced** — Default, moderate suggestions
//! - **Teaching** — More detail, explains reasoning
//! - **Expert** — Less explanation, assumes knowledge
//! - **Silent** — No suggestions, only responds to direct questions

use crate::Priority;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Recommendation policy that controls copilot behavior.
///
/// Each policy sets different thresholds for:
/// - Minimum confidence to show a recommendation
/// - Maximum recommendations per minute
/// - Minimum time between interruptions (seconds)
/// - Level of detail in explanations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub level: PolicyLevel,
    pub min_confidence: f64,
    pub max_recommendations_per_minute: u8,
    pub min_interruption_seconds: u64,
    pub detail_level: DetailLevel,
}

/// Policy level for recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyLevel {
    /// Only show critical recommendations, very few.
    Minimal,
    /// Moderate suggestions, default mode.
    Balanced,
    /// More suggestions with detailed explanations.
    Teaching,
    /// Concise suggestions assuming expertise.
    Expert,
    /// No proactive suggestions.
    Silent,
}

impl fmt::Display for PolicyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyLevel::Minimal => write!(f, "Minimal"),
            PolicyLevel::Balanced => write!(f, "Balanced"),
            PolicyLevel::Teaching => write!(f, "Teaching"),
            PolicyLevel::Expert => write!(f, "Expert"),
            PolicyLevel::Silent => write!(f, "Silent"),
        }
    }
}

impl PolicyLevel {
    /// Returns the default configuration for this policy level.
    pub fn config(&self) -> PolicyConfig {
        match self {
            PolicyLevel::Minimal => PolicyConfig {
                level: PolicyLevel::Minimal,
                min_confidence: 0.9,
                max_recommendations_per_minute: 1,
                min_interruption_seconds: 120,
                detail_level: DetailLevel::High,
            },
            PolicyLevel::Balanced => PolicyConfig {
                level: PolicyLevel::Balanced,
                min_confidence: 0.6,
                max_recommendations_per_minute: 3,
                min_interruption_seconds: 30,
                detail_level: DetailLevel::Medium,
            },
            PolicyLevel::Teaching => PolicyConfig {
                level: PolicyLevel::Teaching,
                min_confidence: 0.5,
                max_recommendations_per_minute: 5,
                min_interruption_seconds: 15,
                detail_level: DetailLevel::High,
            },
            PolicyLevel::Expert => PolicyConfig {
                level: PolicyLevel::Expert,
                min_confidence: 0.7,
                max_recommendations_per_minute: 4,
                min_interruption_seconds: 20,
                detail_level: DetailLevel::Low,
            },
            PolicyLevel::Silent => PolicyConfig {
                level: PolicyLevel::Silent,
                min_confidence: 1.0,
                max_recommendations_per_minute: 0,
                min_interruption_seconds: 999999,
                detail_level: DetailLevel::Low,
            },
        }
    }
}

/// Level of detail in explanations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailLevel {
    /// Minimal explanation, assume expertise.
    Low,
    /// Balanced explanation with key reasoning.
    Medium,
    /// Detailed explanation with full reasoning chain.
    High,
}

impl fmt::Display for DetailLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetailLevel::Low => write!(f, "Low"),
            DetailLevel::Medium => write!(f, "Medium"),
            DetailLevel::High => write!(f, "High"),
        }
    }
}

impl DetailLevel {
    /// Returns the number of evidence items to include.
    pub fn evidence_count(&self) -> usize {
        match self {
            DetailLevel::Low => 1,
            DetailLevel::Medium => 2,
            DetailLevel::High => 3,
        }
    }

    /// Returns true if detailed reasoning should be included.
    pub fn include_reasoning(&self) -> bool {
        matches!(self, DetailLevel::High | DetailLevel::Medium)
    }
}

/// The policy engine that evaluates and filters recommendations.
pub struct PolicyEngine {
    config: PolicyConfig,
    recommendations_shown: Vec<uuid::Uuid>,
    last_recommendation_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl PolicyEngine {
    pub fn new(level: PolicyLevel) -> Self {
        PolicyEngine {
            config: level.config(),
            recommendations_shown: Vec::new(),
            last_recommendation_time: None,
        }
    }

    pub fn with_config(mut self, config: PolicyConfig) -> Self {
        self.config = config;
        self
    }

    /// Check if a recommendation should be shown given the current policy.
    pub fn should_show(
        &self,
        confidence: f64,
        priority: Priority,
        _is_likely_idle: bool,
        _time_since_last: Option<u64>,
    ) -> bool {
        // Silent mode never shows proactive recommendations
        if self.config.level == PolicyLevel::Silent {
            return false;
        }

        // Check confidence threshold
        if confidence < self.config.min_confidence {
            return false;
        }

        // Silent mode: only critical items with minimal filtering
        // But Silent sets min_confidence=1.0 above, so this effectively
        // only shows critical items if somehow confidence is 1.0

        // Check frequency limit
        if self.config.max_recommendations_per_minute == 0 {
            return false;
        }

        if self.recommendations_shown.len() >= self.config.max_recommendations_per_minute as usize {
            return false;
        }

        // Check time since last recommendation
        if let Some(last) = self.last_recommendation_time {
            let now = chrono::Utc::now();
            let elapsed = (now - last).num_seconds() as u64;
            if elapsed < self.config.min_interruption_seconds {
                return false;
            }
        }

        // Silent mode: only allow critical
        if self.config.level == PolicyLevel::Silent && !priority.is_urgent() {
            return false;
        }

        // Minimal mode: only critical and high-confidence
        if self.config.level == PolicyLevel::Minimal {
            if !priority.is_urgent() {
                return false;
            }
            if confidence < 0.9 {
                return false;
            }
        }

        true
    }

    /// Record that a recommendation was shown.
    pub fn record_shown(&mut self, recommendation_id: uuid::Uuid) {
        self.recommendations_shown.push(recommendation_id);
        self.last_recommendation_time = Some(chrono::Utc::now());

        // Trim to keep only last N shown
        if self.recommendations_shown.len()
            > self.config.max_recommendations_per_minute as usize * 10
        {
            let start = self.recommendations_shown.len()
                - self.config.max_recommendations_per_minute as usize * 10;
            self.recommendations_shown.drain(..start);
        }
    }

    /// Get the current policy configuration.
    pub fn config(&self) -> &PolicyConfig {
        &self.config
    }

    /// Set a new policy level.
    pub fn set_level(&mut self, level: PolicyLevel) {
        self.config = level.config();
    }

    /// Get the recommended detail level for explanations.
    pub fn detail_level(&self) -> DetailLevel {
        self.config.detail_level
    }

    /// Get the number of recommendations shown in the current window.
    pub fn recommendations_shown(&self) -> usize {
        self.recommendations_shown.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_config_defaults() {
        let minimal = PolicyLevel::Minimal.config();
        assert_eq!(minimal.min_confidence, 0.9);
        assert_eq!(minimal.max_recommendations_per_minute, 1);

        let balanced = PolicyLevel::Balanced.config();
        assert_eq!(balanced.min_confidence, 0.6);
        assert_eq!(balanced.max_recommendations_per_minute, 3);

        let teaching = PolicyLevel::Teaching.config();
        assert_eq!(teaching.min_confidence, 0.5);
        assert_eq!(teaching.max_recommendations_per_minute, 5);

        let expert = PolicyLevel::Expert.config();
        assert_eq!(expert.min_confidence, 0.7);
        assert_eq!(expert.max_recommendations_per_minute, 4);

        let silent = PolicyLevel::Silent.config();
        assert_eq!(silent.min_confidence, 1.0);
        assert_eq!(silent.max_recommendations_per_minute, 0);
    }

    #[test]
    fn test_policy_minimal_shows_critical_high() {
        let mut engine = PolicyEngine::new(PolicyLevel::Minimal);
        assert!(engine.should_show(0.95, Priority::Critical, true, None));
        assert!(!engine.should_show(0.95, Priority::Warning, true, None));
    }

    #[test]
    fn test_policy_balanced_shows_warnings() {
        let mut engine = PolicyEngine::new(PolicyLevel::Balanced);
        assert!(engine.should_show(0.7, Priority::Warning, true, None));
        assert!(engine.should_show(0.65, Priority::Suggestion, true, None));
    }

    #[test]
    fn test_policy_silent_never_shows() {
        let engine = PolicyEngine::new(PolicyLevel::Silent);
        assert!(!engine.should_show(0.99, Priority::Critical, true, None));
    }

    #[test]
    fn test_policy_low_confidence_filtered() {
        let engine = PolicyEngine::new(PolicyLevel::Balanced);
        assert!(!engine.should_show(0.3, Priority::Warning, true, None));
        assert!(!engine.should_show(0.4, Priority::Critical, true, None));
    }

    #[test]
    fn test_policy_frequency_limit() {
        let mut engine = PolicyEngine::new(PolicyLevel::Minimal);
        // Minimal allows 1 per minute
        engine.recommendations_shown.push(uuid::Uuid::new_v4());
        assert!(!engine.should_show(0.99, Priority::Critical, true, None));
    }

    #[test]
    fn test_policy_interruption_cooldown() {
        let mut engine = PolicyEngine::new(PolicyLevel::Balanced);
        engine.record_shown(uuid::Uuid::new_v4());
        // Should respect cooldown (30 seconds for balanced)
        assert!(!engine.should_show(0.9, Priority::Warning, true, Some(5)));
    }

    #[test]
    fn test_policy_level_display() {
        assert_eq!(format!("{}", PolicyLevel::Minimal), "Minimal");
        assert_eq!(format!("{}", PolicyLevel::Balanced), "Balanced");
        assert_eq!(format!("{}", PolicyLevel::Teaching), "Teaching");
        assert_eq!(format!("{}", PolicyLevel::Expert), "Expert");
        assert_eq!(format!("{}", PolicyLevel::Silent), "Silent");
    }

    #[test]
    fn test_detail_level_evidence_count() {
        assert_eq!(DetailLevel::Low.evidence_count(), 1);
        assert_eq!(DetailLevel::Medium.evidence_count(), 2);
        assert_eq!(DetailLevel::High.evidence_count(), 3);
    }

    #[test]
    fn test_detail_level_reasoning() {
        assert!(!DetailLevel::Low.include_reasoning());
        assert!(DetailLevel::Medium.include_reasoning());
        assert!(DetailLevel::High.include_reasoning());
    }

    #[test]
    fn test_policy_change_level() {
        let mut engine = PolicyEngine::new(PolicyLevel::Balanced);
        assert_eq!(engine.config.max_recommendations_per_minute, 3);
        engine.set_level(PolicyLevel::Minimal);
        assert_eq!(engine.config.max_recommendations_per_minute, 1);
    }
}
