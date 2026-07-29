//! Engineering Persona — default system behavior for Wiki Labs AI Copilot.
//!
//! The AI behaves as:
//! - Senior Infrastructure Engineer
//! - Technical Advisor
//! - Enterprise Consultant
//! - Troubleshooting Mentor
//!
//! The AI must:
//! - Explain reasoning clearly
//! - Prefer evidence-based recommendations
//! - Suggest verification steps
//! - Avoid assumptions when confidence is low
//! - Never claim to have observed something it has not

use serde::{Deserialize, Serialize};

/// Persona definition for the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringPersona {
    /// Role name.
    pub role: String,
    /// System prompt defining behavior.
    pub system_prompt: String,
    /// Whether the persona is active.
    pub active: bool,
    /// Confidence thresholds.
    pub confidence_thresholds: ConfidenceThresholds,
}

impl EngineeringPersona {
    /// Create the default engineering persona.
    pub fn default_persona() -> Self {
        Self {
            role: "Senior Infrastructure Engineer".to_string(),
            system_prompt: Self::default_system_prompt(),
            active: true,
            confidence_thresholds: ConfidenceThresholds::default(),
        }
    }

    /// The default system prompt text.
    /// Loaded from assets/system_prompt.md at runtime; falls back to inline if file not found.
    pub fn default_system_prompt() -> String {
        // Try loading from embedded file first
        if let Ok(content) = std::fs::read_to_string("assets/system_prompt.md") {
            return content.trim().to_string();
        }
        // Fallback: inline prompt (used during development when assets dir not available)
        r#"You are the Wiki Labs AI Copilot — an AI assistant built into the Wiki Labs AI Copilot desktop application.

## Your Identity
- You ARE the Wiki Labs AI Copilot app itself
- You observe the user's environment (active apps, browser URLs, terminal commands, file activity) and provide contextual guidance and recommendations
- You are a senior infrastructure engineer, technical advisor, enterprise consultant, and troubleshooting mentor
- Your role is to watch "what a technical engineer is doing" and proactively suggest helpful actions

## Your Behavior
- Be conversational and like a helpful teammate giving natural suggestions ("you should also check MySQL status")
- Provide actionable, specific recommendations — avoid vague statements or formal metadata cards
- Explain your reasoning clearly and step-by-step
- Prefer evidence-based recommendations over assumptions
- Suggest verification steps so the engineer can confirm your advice
- State your confidence level when making recommendations (HIGH/MEDIUM/LOW)
- When suggesting commands or configuration changes, explain why each step matters

## Knowledge Packs & Skills
- You have access to knowledge packs and skills that contain specific technical expertise
- When observations suggest the user is working with something related to a knowledge pack (e.g., Kubernetes, MySQL, Docker, AWS, networking, Linux sysadmin), proactively load and use that knowledge pack's content to provide targeted guidance
- Skills are reusable procedures for recurring task types — reference them when the user's work matches the skill's domain
- Use knowledge packs to provide specific, authoritative guidance rather than generic advice

## What You Know
- You are designed to observe and guide — you watch what users do in their work environment
- You can recommend commands, suggest checking system status, flag potential issues
- You understand infrastructure, systems engineering, databases, containers, networks, cloud platforms, Linux, Windows, networking
- You cannot execute commands or directly interact with the user's system — you only suggest and guide
- You receive observation context about what's happening in the user's environment with each message

## Important Constraints
- You are an AI assistant. The human engineer remains responsible for all actions.
- You cannot observe the user's screen, filesystem, or running processes unless explicitly provided that information through observation context.
- If asked about something you cannot see or know, clearly state your limitations.
- Always recommend that critical changes be verified in a non-production environment first."#.to_string()
    }

    /// Create a persona with a custom system prompt.
    pub fn custom(role: &str, system_prompt: &str) -> Self {
        Self {
            role: role.to_string(),
            system_prompt: system_prompt.to_string(),
            active: true,
            confidence_thresholds: ConfidenceThresholds::default(),
        }
    }

    /// Get the system prompt text.
    pub fn system_prompt_text(&self) -> &str {
        &self.system_prompt
    }

    /// Activate the persona.
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate the persona.
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Check if the persona is active.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for EngineeringPersona {
    fn default() -> Self {
        Self::default_persona()
    }
}

/// Confidence level for AI recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

use std::fmt::{Display, Formatter};

impl Display for ConfidenceLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfidenceLevel::High => write!(f, "HIGH"),
            ConfidenceLevel::Medium => write!(f, "MEDIUM"),
            ConfidenceLevel::Low => write!(f, "LOW"),
        }
    }
}

/// Confidence thresholds for different recommendation types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceThresholds {
    /// Minimum confidence to recommend without disclaimer (0.0 - 1.0).
    pub high_threshold: f32,
    /// Minimum confidence to provide a reasoned answer (0.0 - 1.0).
    pub medium_threshold: f32,
    /// Below this, the AI should clearly state low confidence.
    pub low_threshold: f32,
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self {
            high_threshold: 0.9,
            medium_threshold: 0.6,
            low_threshold: 0.3,
        }
    }
}

/// Confidence assessment for a specific recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceAssessment {
    pub level: ConfidenceLevel,
    pub score: f32,
    pub reasoning: String,
    pub suggested_verification: Option<String>,
}

impl ConfidenceAssessment {
    pub fn high(reasoning: &str) -> Self {
        Self {
            level: ConfidenceLevel::High,
            score: 0.9,
            reasoning: reasoning.to_string(),
            suggested_verification: Some(
                "Standard practice — verify in non-production first".to_string(),
            ),
        }
    }

    pub fn medium(reasoning: &str, verification: &str) -> Self {
        Self {
            level: ConfidenceLevel::Medium,
            score: 0.6,
            reasoning: reasoning.to_string(),
            suggested_verification: Some(verification.to_string()),
        }
    }

    pub fn low(reasoning: &str, ask_clarifying: &str) -> Self {
        Self {
            level: ConfidenceLevel::Low,
            score: 0.3,
            reasoning: reasoning.to_string(),
            suggested_verification: Some(format!("Ask: {}", ask_clarifying)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_persona() {
        let persona = EngineeringPersona::default_persona();
        assert!(persona.is_active());
        assert_eq!(persona.role, "Senior Infrastructure Engineer");
        assert!(!persona.system_prompt.is_empty());
    }

    #[test]
    fn test_default_system_prompt_content() {
        let prompt = EngineeringPersona::default_system_prompt();
        assert!(prompt.contains("Senior Infrastructure Engineer"));
        assert!(prompt.contains("evidence-based"));
        assert!(prompt.contains("verification"));
        assert!(prompt.contains("confidence"));
        assert!(prompt.contains("cannot observe"));
    }

    #[test]
    fn test_custom_persona() {
        let persona = EngineeringPersona::custom("Custom Role", "Custom instructions here.");
        assert_eq!(persona.role, "Custom Role");
        assert_eq!(persona.system_prompt, "Custom instructions here.");
        assert!(persona.is_active());
    }

    #[test]
    fn test_activate_deactivate() {
        let mut persona = EngineeringPersona::default_persona();
        assert!(persona.is_active());

        persona.deactivate();
        assert!(!persona.is_active());

        persona.activate();
        assert!(persona.is_active());
    }

    #[test]
    fn test_system_prompt_text() {
        let persona = EngineeringPersona::default_persona();
        let text = persona.system_prompt_text();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_confidence_thresholds_defaults() {
        let thresholds = ConfidenceThresholds::default();
        assert_eq!(thresholds.high_threshold, 0.9);
        assert_eq!(thresholds.medium_threshold, 0.6);
        assert_eq!(thresholds.low_threshold, 0.3);
    }

    #[test]
    fn test_confidence_levels() {
        assert_eq!(ConfidenceLevel::High.to_string(), "HIGH");
        assert_eq!(ConfidenceLevel::Medium.to_string(), "MEDIUM");
        assert_eq!(ConfidenceLevel::Low.to_string(), "LOW");
    }

    #[test]
    fn test_confidence_assessment_high() {
        let assessment = ConfidenceAssessment::high("Standard deployment pattern");
        assert_eq!(assessment.level, ConfidenceLevel::High);
        assert_eq!(assessment.score, 0.9);
        assert!(assessment.suggested_verification.is_some());
    }

    #[test]
    fn test_confidence_assessment_medium() {
        let assessment = ConfidenceAssessment::medium(
            "Based on logs, likely config issue",
            "Check application logs for specific error codes",
        );
        assert_eq!(assessment.level, ConfidenceLevel::Medium);
        assert_eq!(assessment.score, 0.6);
        assert_eq!(
            assessment.suggested_verification,
            Some("Check application logs for specific error codes".to_string())
        );
    }

    #[test]
    fn test_confidence_assessment_low() {
        let assessment = ConfidenceAssessment::low(
            "Insufficient information",
            "What is the operating system version?",
        );
        assert_eq!(assessment.level, ConfidenceLevel::Low);
        assert_eq!(assessment.score, 0.3);
        assert!(assessment
            .suggested_verification
            .unwrap()
            .contains("What is the operating system version?"));
    }

    #[test]
    fn test_personas_are_cloneable() {
        let persona = EngineeringPersona::default_persona();
        let cloned = persona.clone();
        assert_eq!(persona.role, cloned.role);
        assert_eq!(persona.system_prompt, cloned.system_prompt);
        assert_eq!(persona.is_active(), cloned.is_active());
    }

    #[test]
    fn test_default_trait() {
        let persona: EngineeringPersona = Default::default();
        assert!(persona.is_active());
        assert_eq!(persona.role, "Senior Infrastructure Engineer");
    }
}
