//! Copilot modes — Teaching, Balanced, Expert, Silent.
//!
//! Modes influence Decision Engine behaviour and how much
//! detail is provided in recommendations.
//!
//! - **Teaching**: Explains everything, assumes no prior knowledge
//! - **Balanced**: Default mode, moderate explanations
//! - **Expert**: Concise, assumes deep knowledge
//! - **Silent**: No proactive suggestions, only responds to direct questions

use crate::policy::{PolicyEngine, PolicyLevel};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Copilot mode that influences behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum CopilotMode {
    /// Explains everything, shows all recommendations with detail.
    Teaching,
    /// Balanced suggestions with moderate detail. Default mode.
    #[default]
    Balanced,
    /// Concise suggestions assuming deep knowledge.
    Expert,
    /// No proactive suggestions.
    Silent,
}

impl fmt::Display for CopilotMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CopilotMode::Teaching => write!(f, "Teaching"),
            CopilotMode::Balanced => write!(f, "Balanced"),
            CopilotMode::Expert => write!(f, "Expert"),
            CopilotMode::Silent => write!(f, "Silent"),
        }
    }
}

impl CopilotMode {
    /// Returns the corresponding PolicyLevel for this mode.
    pub fn policy_level(&self) -> PolicyLevel {
        match self {
            CopilotMode::Teaching => PolicyLevel::Teaching,
            CopilotMode::Balanced => PolicyLevel::Balanced,
            CopilotMode::Expert => PolicyLevel::Expert,
            CopilotMode::Silent => PolicyLevel::Silent,
        }
    }

    /// Returns whether this mode provides detailed explanations.
    pub fn provides_explanations(&self) -> bool {
        !matches!(self, CopilotMode::Silent)
    }

    /// Returns whether this mode shows proactive suggestions.
    pub fn shows_proactive_suggestions(&self) -> bool {
        !matches!(self, CopilotMode::Silent)
    }
}


/// Configurable mode settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub mode: CopilotMode,
    /// Additional confidence boost for critical items.
    pub critical_boost: f64,
    /// Maximum explanation length in characters.
    pub max_explanation_chars: usize,
}

impl Default for ModeConfig {
    fn default() -> Self {
        ModeConfig {
            mode: CopilotMode::Balanced,
            critical_boost: 0.1,
            max_explanation_chars: 500,
        }
    }
}

impl ModeConfig {
    pub fn policy_engine(&self) -> PolicyEngine {
        PolicyEngine::new(self.mode.policy_level())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_default() {
        assert_eq!(CopilotMode::default(), CopilotMode::Balanced);
    }

    #[test]
    fn test_mode_policy_levels() {
        assert_eq!(CopilotMode::Teaching.policy_level(), PolicyLevel::Teaching);
        assert_eq!(CopilotMode::Balanced.policy_level(), PolicyLevel::Balanced);
        assert_eq!(CopilotMode::Expert.policy_level(), PolicyLevel::Expert);
        assert_eq!(CopilotMode::Silent.policy_level(), PolicyLevel::Silent);
    }

    #[test]
    fn test_mode_explanations() {
        assert!(CopilotMode::Teaching.provides_explanations());
        assert!(CopilotMode::Balanced.provides_explanations());
        assert!(CopilotMode::Expert.provides_explanations());
        assert!(!CopilotMode::Silent.provides_explanations());
    }

    #[test]
    fn test_mode_proactive_suggestions() {
        assert!(CopilotMode::Teaching.shows_proactive_suggestions());
        assert!(CopilotMode::Balanced.shows_proactive_suggestions());
        assert!(CopilotMode::Expert.shows_proactive_suggestions());
        assert!(!CopilotMode::Silent.shows_proactive_suggestions());
    }

    #[test]
    fn test_mode_display() {
        assert_eq!(format!("{}", CopilotMode::Teaching), "Teaching");
        assert_eq!(format!("{}", CopilotMode::Balanced), "Balanced");
        assert_eq!(format!("{}", CopilotMode::Expert), "Expert");
        assert_eq!(format!("{}", CopilotMode::Silent), "Silent");
    }

    #[test]
    fn test_mode_config_default() {
        let config = ModeConfig::default();
        assert_eq!(config.mode, CopilotMode::Balanced);
        assert_eq!(config.critical_boost, 0.1);
        assert_eq!(config.max_explanation_chars, 500);
    }

    #[test]
    fn test_mode_config_creates_policy_engine() {
        let config = ModeConfig::default();
        let engine = config.policy_engine();
        assert_eq!(engine.config().level, PolicyLevel::Balanced);
    }
}
