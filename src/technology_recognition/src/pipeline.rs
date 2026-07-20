//! Recognition Pipeline — multi-pass evidence combination.
//!
//! This module is intentionally minimal: it does not implement any
//! knowledge retrieval, RAG, MCP execution, command execution,
//! automation, screen AI analysis, OCR reasoning, or autonomous actions.

use crate::engine::DetectionEngine;
use wikilabs_data_types::{DetectionRule, TechnologyInference};
use wikilabs_observation::ObservationEvent;

/// Run a recognition pass using the detection engine.
pub fn run_pass(rules: &[DetectionRule], event: &ObservationEvent) -> Vec<TechnologyInference> {
    let mut engine = DetectionEngine::new();
    for rule in rules {
        engine.add_rule(rule.clone());
    }
    engine.recognize(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wikilabs_data_types::DetectionType;
    use wikilabs_observation::{EventType, ObservationEvent, ObservationPayload, ProviderType};

    #[test]
    fn test_run_pass_empty_rules() {
        let rules = Vec::<DetectionRule>::new();
        let payload = ObservationPayload::empty();
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "test".to_string(),
            None,
            payload,
        );
        let inferences = run_pass(&rules, &event);
        assert!(inferences.is_empty());
    }

    #[test]
    fn test_run_pass_with_rules() {
        let rules = vec![DetectionRule {
            id: "rule-1".to_string(),
            name: "alacritty-detect".to_string(),
            detection_type: DetectionType::Environment,
            pattern: "alacritty".to_string(),
            confidence: 0.9,
            technology_domain: "linux".to_string(),
            priority: 1,
            flags: String::new(),
            extract: None,
        }];

        let payload = ObservationPayload::empty();
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "alacritty".to_string(),
            None,
            payload,
        );
        let inferences = run_pass(&rules, &event);
        assert_eq!(inferences.len(), 1);
        assert_eq!(inferences[0].name, "linux");
        assert_eq!(inferences[0].confidence, 0.9);
    }
}
