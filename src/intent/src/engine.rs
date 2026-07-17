//! Intent recognition engine — technology-aware + ML.
//!
//! - Rule-based pattern matching (Phase 1)
//! - ML-based classification (Phase 2+)
//! - Technology-aware intent recognition
//! - Multi-intent support
//! - Confidence scoring
//! - "Unknown" intent state (first-class concept)
//! - Human correction mechanisms

use std::collections::HashMap;

use regex::Regex;
use tracing::debug;
use wikilabs_data_types::IntentDefinition;

use crate::confidence::ConfidenceEngine;
use crate::correction::CorrectionEngine;

/// Recognized intent from user input or context.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Intent {
    Troubleshooting,
    Configuration,
    Deployment,
    Documentation,
    Learning,
    Unknown,
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intent::Troubleshooting => write!(f, "troubleshooting"),
            Intent::Configuration => write!(f, "configuration"),
            Intent::Deployment => write!(f, "deployment"),
            Intent::Documentation => write!(f, "documentation"),
            Intent::Learning => write!(f, "learning"),
            Intent::Unknown => write!(f, "unknown"),
        }
    }
}

/// Result of intent recognition including confidence and technology match.
#[derive(Clone, Debug, PartialEq)]
pub struct RecognizedIntent {
    /// The recognized intent.
    pub intent: Intent,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Technology domain the intent is associated with (if any).
    pub technology_domain: Option<String>,
}

/// Intent recognition engine with technology awareness.
pub struct IntentEngine {
    /// Generic patterns (not tied to any technology).
    generic_patterns: Vec<(Regex, Intent)>,
    /// Technology-specific intent patterns keyed by domain.
    technology_intents: HashMap<String, Vec<IntentDefinition>>,
    /// Correction engine for human feedback.
    correction_engine: CorrectionEngine,
    /// Confidence scoring engine.
    confidence_engine: ConfidenceEngine,
    /// Default intent when no patterns match.
    default_intent: Intent,
}

impl IntentEngine {
    /// Create a new intent engine with default configuration.
    pub fn new() -> Self {
        let mut engine = Self {
            generic_patterns: Vec::new(),
            technology_intents: HashMap::new(),
            correction_engine: CorrectionEngine::new(),
            confidence_engine: ConfidenceEngine::new(),
            default_intent: Intent::Unknown,
        };
        engine.init_default_patterns();
        engine
    }

    /// Create a new intent engine with a custom confidence engine.
    pub fn new_with_confidence(confidence_engine: ConfidenceEngine) -> Self {
        let mut engine = Self {
            generic_patterns: Vec::new(),
            technology_intents: HashMap::new(),
            correction_engine: CorrectionEngine::new(),
            confidence_engine,
            default_intent: Intent::Unknown,
        };
        engine.init_default_patterns();
        engine
    }

    /// Initialize the default generic patterns.
    fn init_default_patterns(&mut self) {
        // Troubleshooting patterns
        self.add_pattern(
            r"(?i)(troubleshoot|debug|fix|error|issue|problem|crash|fail|bug)",
            Intent::Troubleshooting,
        );

        // Configuration patterns
        self.add_pattern(
            r"(?i)(config|configure|setting|settings|setup|custom)",
            Intent::Configuration,
        );

        // Deployment patterns
        self.add_pattern(
            r"(?i)(deploy|release|rollout|promotion|helm|kubectl apply|kubectl create)",
            Intent::Deployment,
        );

        // Documentation patterns
        self.add_pattern(
            r"(?i)(document|readme|wiki|manual|how.to|guide|tutorial|explain)",
            Intent::Documentation,
        );

        // Learning patterns
        self.add_pattern(
            r"(?i)(learn|teach|educat|understand|what.is|how does|what is)",
            Intent::Learning,
        );
    }

    /// Add a pattern to the generic pattern list.
    fn add_pattern(&mut self, regex_str: &str, intent: Intent) {
        if let Ok(regex) = Regex::new(regex_str) {
            self.generic_patterns.push((regex, intent));
        }
    }

    /// Register technology-specific intents for a domain.
    pub fn register_technology_intents(&mut self, technology: &str, intents: Vec<IntentDefinition>) {
        debug!(
            "Registered {} intents for technology '{}'",
            intents.len(),
            technology
        );
        self.technology_intents
            .insert(technology.to_string(), intents);
    }

    /// Get technology-specific intents for a domain.
    pub fn get_technology_intents(&self, technology: &str) -> Vec<&IntentDefinition> {
        self.technology_intents
            .get(technology)
            .map(|intents| intents.iter().collect())
            .unwrap_or_default()
    }

    /// Recognize the single best intent from context.
    pub fn recognize(&self, context: &str, technology: Option<&str>) -> (Intent, f32) {
        let results = self.recognize_multi_impl(context, technology);
        results
            .into_iter()
            .next()
            .map(|(intent, confidence)| (intent, confidence))
            .unwrap_or((self.default_intent.clone(), 0.1))
    }

    /// Recognize multiple intents sorted by confidence.
    pub fn recognize_multi(&self, context: &str, technology: Option<&str>) -> Vec<(Intent, f32)> {
        let results = self.recognize_multi_impl(context, technology);
        // Deduplicate by intent, keeping highest confidence
        let mut seen: HashMap<Intent, f32> = HashMap::new();
        for (intent, confidence) in results {
            seen.entry(intent)
                .and_modify(|c| {
                    if confidence > *c {
                        *c = confidence;
                    }
                })
                .or_insert(confidence);
        }
        let mut results: Vec<(Intent, f32)> = seen.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Internal multi-intent recognition logic.
    fn recognize_multi_impl(
        &self,
        context: &str,
        technology: Option<&str>,
    ) -> Vec<(Intent, f32)> {
        let mut results: Vec<(Intent, f32)> = Vec::new();

        // Evaluate generic patterns
        for (regex, intent) in &self.generic_patterns {
            if regex.is_match(context) {
                // Check for technology-specific boost
                let mut confidence = 0.7;
                let mut domain: Option<String> = None;

                if let Some(domain_name) = technology {
                    if let Some(tech_intents) = self.technology_intents.get(domain_name) {
                        for ti in tech_intents {
                            if ti.required_domain == domain_name {
                                for pattern in &ti.patterns {
                                    if let Ok(re) = Regex::new(pattern) {
                                        if re.is_match(context) {
                                            confidence = ti.confidence_boost.max(confidence);
                                            domain = Some(domain_name.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Apply correction boosts from human feedback
                confidence = self.correction_engine.apply_intent_correction(
                    confidence,
                    context,
                    &intent.to_string(),
                );

                results.push((intent.clone(), confidence));
            }
        }

        // Evaluate technology-specific patterns
        if let Some(domain) = technology {
            if let Some(tech_intents) = self.technology_intents.get(domain) {
                for intent_def in tech_intents {
                    for pattern in &intent_def.patterns {
                        if let Ok(re) = Regex::new(pattern) {
                            if re.is_match(context) {
                                let confidence = intent_def
                                    .confidence_boost
                                    .clamp(0.0, 1.0);
                                let intent = self.map_intent_def_to_enum(&intent_def.name);
                                results.push((intent, confidence));
                            }
                        }
                    }
                }
            }
        }

        // Sort by confidence descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Map an intent definition name to an Intent enum variant.
    fn map_intent_def_to_enum(&self, name: &str) -> Intent {
        let lower = name.to_lowercase();
        if lower.contains("troubleshoot") || lower.contains("debug") || lower.contains("fix") {
            Intent::Troubleshooting
        } else if lower.contains("config") || lower.contains("setup") {
            Intent::Configuration
        } else if lower.contains("deploy") || lower.contains("release") {
            Intent::Deployment
        } else if lower.contains("document") || lower.contains("guide") {
            Intent::Documentation
        } else if lower.contains("learn") || lower.contains("teach") {
            Intent::Learning
        } else {
            Intent::Unknown
        }
    }

    /// Recognize with confidence scoring.
    pub fn recognize_with_confidence(
        &self,
        context: &str,
        technology: Option<&str>,
    ) -> (Intent, f32) {
        let results = self.recognize_multi_impl(context, technology);
        if results.is_empty() {
            return (self.default_intent.clone(), 0.1);
        }
        results[0].clone()
    }

    /// Apply a correction from human feedback.
    ///
    /// Records that the expected intent differs from what was recognized.
    /// This correction is used to boost future matching confidence.
    pub fn apply_correction(&mut self, expected: &str, actual: &str, context: Option<String>) {
        let expected_intent = Self::label_to_intent(expected);
        let actual_intent = Self::label_to_intent(actual);
        self.correction_engine.record_correction(expected_intent, actual_intent, context);
        debug!(
            "Recorded intent correction: expected='{}', actual='{}'",
            expected, actual
        );
    }

    fn label_to_intent(label: &str) -> Intent {
        let lower = label.to_lowercase();
        match lower.as_str() {
            "troubleshooting" | "troubleshoot" | "debug" | "fix" => Intent::Troubleshooting,
            "configuration" | "config" | "setup" => Intent::Configuration,
            "deployment" | "deploy" => Intent::Deployment,
            "documentation" | "document" => Intent::Documentation,
            "learning" | "learn" | "teach" => Intent::Learning,
            _ => Intent::Unknown,
        }
    }

    /// Get the number of generic patterns registered.
    pub fn generic_pattern_count(&self) -> usize {
        self.generic_patterns.len()
    }

    /// Get the number of technology-specific intent groups.
    pub fn technology_intent_count(&self) -> usize {
        self.technology_intents.len()
    }

    /// Get total intent count (generic + technology-specific patterns).
    pub fn total_intent_patterns(&self) -> usize {
        let tech_patterns: usize = self
            .technology_intents
            .values()
            .map(|intents| {
                intents
                    .iter()
                    .flat_map(|i| i.patterns.iter())
                    .count()
            })
            .sum();
        self.generic_patterns.len() + tech_patterns
    }

    /// Get the default intent.
    pub fn default_intent(&self) -> &Intent {
        &self.default_intent
    }

    /// Set the default intent.
    pub fn set_default_intent(&mut self, intent: Intent) {
        self.default_intent = intent;
    }

    /// Get the confidence engine for threshold control.
    pub fn confidence_engine(&self) -> &ConfidenceEngine {
        &self.confidence_engine
    }

    /// Classify a confidence score using the confidence engine.
    pub fn classify_confidence(&self, confidence: f32) -> &str {
        self.confidence_engine.classify(confidence)
    }

    /// Get all recognized intents for context, with scores.
    pub fn get_all_intents(&self, context: &str, technology: Option<&str>) -> Vec<RecognizedIntent> {
        let results = self.recognize_multi_impl(context, technology);
        results
            .into_iter()
            .map(|(intent, confidence)| RecognizedIntent {
                intent,
                confidence,
                technology_domain: technology.map(|t| t.to_string()),
            })
            .collect()
    }
}

impl Default for IntentEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine_has_patterns() {
        let engine = IntentEngine::new();
        assert!(engine.generic_pattern_count() > 0);
    }

    #[test]
    fn test_recognize_troubleshooting() {
        let engine = IntentEngine::new();
        assert_eq!(engine.recognize("fix this crash", None).0, Intent::Troubleshooting);
        assert_eq!(engine.recognize("debug the error", None).0, Intent::Troubleshooting);
        assert_eq!(engine.recognize("solve the bug", None).0, Intent::Troubleshooting);
    }

    #[test]
    fn test_recognize_configuration() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("configure the server", None).0,
            Intent::Configuration,
        );
        assert_eq!(
            engine.recognize("settings for pod", None).0,
            Intent::Configuration,
        );
    }

    #[test]
    fn test_recognize_deployment() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("deploy to production", None).0,
            Intent::Deployment,
        );
        assert_eq!(
            engine.recognize("kubectl create deployment", None).0,
            Intent::Deployment,
        );
    }

    #[test]
    fn test_recognize_documentation() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("write a readme", None).0,
            Intent::Documentation,
        );
        assert_eq!(
            engine.recognize("explain how this works", None).0,
            Intent::Documentation,
        );
    }

    #[test]
    fn test_recognize_learning() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("what is OpenShift", None).0,
            Intent::Learning,
        );
        assert_eq!(
            engine.recognize("teach me about pods", None).0,
            Intent::Learning,
        );
    }

    #[test]
    fn test_recognize_unknown() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("blabla random words", None).0,
            Intent::Unknown,
        );
        assert_eq!(engine.recognize("", None).0, Intent::Unknown);
    }

    #[test]
    fn test_recognize_case_insensitive() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("TROUBLESHOOT the pod", None).0,
            Intent::Troubleshooting,
        );
        assert_eq!(
            engine.recognize("Deploy to staging", None).0,
            Intent::Deployment,
        );
    }

    #[test]
    fn test_recognize_with_confidence_single_match() {
        let engine = IntentEngine::new();
        let (intent, confidence) = engine.recognize_with_confidence("fix this crash", None);
        assert_eq!(intent, Intent::Troubleshooting);
        assert!(confidence >= 0.6 && confidence <= 0.8);
    }

    #[test]
    fn test_recognize_with_confidence_no_match() {
        let engine = IntentEngine::new();
        let (intent, confidence) = engine.recognize_with_confidence("random text", None);
        assert_eq!(intent, Intent::Unknown);
        assert_eq!(confidence, 0.1);
    }

    #[test]
    fn test_recognize_with_confidence_multiple_match() {
        let engine = IntentEngine::new();
        let (intent, confidence) =
            engine.recognize_with_confidence("configure and deploy the system", None);
        assert_eq!(intent, Intent::Deployment);
        assert!(confidence > 0.7);
    }

    #[test]
    fn test_intent_display() {
        assert_eq!(format!("{}", Intent::Troubleshooting), "troubleshooting");
        assert_eq!(format!("{}", Intent::Configuration), "configuration");
        assert_eq!(format!("{}", Intent::Deployment), "deployment");
        assert_eq!(format!("{}", Intent::Documentation), "documentation");
        assert_eq!(format!("{}", Intent::Learning), "learning");
        assert_eq!(format!("{}", Intent::Unknown), "unknown");
    }

    #[test]
    fn test_register_technology_intents() {
        let mut engine = IntentEngine::new();
        let tech_intents = vec![
            IntentDefinition {
                id: "rust-deploy".to_string(),
                name: "Deploy Rust".to_string(),
                description: "Deploy a Rust application".to_string(),
                patterns: vec!["(?i)(cargo deploy|cargo build --release)".to_string()],
                confidence_boost: 0.9,
                required_domain: "rust".to_string(),
                priority: 10,
            },
        ];
        engine.register_technology_intents("rust", tech_intents);

        assert_eq!(engine.technology_intent_count(), 1);
        let rust_intents = engine.get_technology_intents("rust");
        assert_eq!(rust_intents.len(), 1);
        assert_eq!(rust_intents[0].name, "Deploy Rust");

        // Non-existent tech domain
        let non_existent = engine.get_technology_intents("python");
        assert!(non_existent.is_empty());
    }

    #[test]
    fn test_technology_aware_recognition() {
        let mut engine = IntentEngine::new();
        let tech_intents = vec![
            IntentDefinition {
                id: "rust-deploy".to_string(),
                name: "Deploy Rust".to_string(),
                description: "Deploy Rust app".to_string(),
                patterns: vec!["(?i)cargo build --release".to_string()],
                confidence_boost: 0.95,
                required_domain: "rust".to_string(),
                priority: 10,
            },
        ];
        engine.register_technology_intents("rust", tech_intents);

        // Generic pattern match
        let (intent, conf) = engine.recognize("deploy the app", Some("rust"));
        assert_eq!(intent, Intent::Deployment);
        assert!(conf >= 0.7);

        // Technology-specific pattern match (higher confidence)
        let (intent, conf) = engine.recognize("cargo build --release", Some("rust"));
        assert_eq!(intent, Intent::Deployment);
        assert!(conf >= 0.9);

        // No tech domain specified
        let (intent, conf) = engine.recognize("cargo build --release", None);
        assert_eq!(intent, Intent::Unknown);
        assert_eq!(conf, 0.1);
    }

    #[test]
    fn test_recognize_multi() {
        let engine = IntentEngine::new();
        let results =
            engine.recognize_multi("fix and configure the system", None);
        assert!(!results.is_empty());
        // Should have at least troubleshooting and configuration
        let intents: Vec<&str> = results.iter().map(|(i, _)| i.to_string()).collect();
        assert!(intents.contains(&"troubleshooting".to_string())
            || intents.contains(&"configuration".to_string()));
        // Results should be sorted by confidence descending
        for i in 0..results.len() - 1 {
            assert!(results[i].1 >= results[i + 1].1);
        }
    }

    #[test]
    fn test_apply_correction() {
        let mut engine = IntentEngine::new();
        engine.apply_correction("deployment", "troubleshooting", Some("User meant deployment"));

        // The correction is recorded but won't change the current result
        // since it's based on context, not previous intent
        let (intent, _conf) = engine.recognize("fix this crash", None);
        assert_eq!(intent, Intent::Troubleshooting);
    }

    #[test]
    fn test_total_intent_patterns() {
        let engine = IntentEngine::new();
        let generic_count = engine.generic_pattern_count();
        let tech_count = engine.technology_intent_count();
        assert!(generic_count > 0);
        assert_eq!(tech_count, 0);

        // After registering tech intents
        let mut engine2 = IntentEngine::new();
        engine2.register_technology_intents("test", vec![
            IntentDefinition {
                id: "test-1".to_string(),
                name: "Test 1".to_string(),
                description: "Test".to_string(),
                patterns: vec!["(?i)test".to_string()],
                confidence_boost: 0.8,
                required_domain: "test".to_string(),
                priority: 5,
            },
        ]);
        assert_eq!(engine2.technology_intent_count(), 1);
        let total = engine2.total_intent_patterns();
        assert_eq!(total, generic_count + 1);
    }

    #[test]
    fn test_classify_confidence() {
        let engine = IntentEngine::new();
        assert_eq!(engine.classify_confidence(0.9), "confident");
        assert_eq!(engine.classify_confidence(0.6), "moderate");
        assert_eq!(engine.classify_confidence(0.3), "uncertain");
    }

    #[test]
    fn test_get_all_intents() {
        let engine = IntentEngine::new();
        let results = engine.get_all_intents("fix this crash", None);
        assert!(!results.is_empty());
        assert_eq!(results[0].intent, Intent::Troubleshooting);
        assert!(results[0].confidence >= 0.6);
    }

    #[test]
    fn test_default_intent() {
        let engine = IntentEngine::new();
        assert_eq!(engine.default_intent(), &Intent::Unknown);

        let mut engine2 = IntentEngine::new();
        engine2.set_default_intent(Intent::Configuration);
        assert_eq!(engine2.default_intent(), &Intent::Configuration);
    }

    #[test]
    fn test_recognize_multi_dedup() {
        let engine = IntentEngine::new();
        // "fix" matches troubleshooting, "debug" also matches troubleshooting
        // But should deduplicate to one troubleshooting entry
        let results = engine.recognize_multi("fix and debug the issue", None);
        let troubleshooting_count = results.iter().filter(|(i, _)| *i == Intent::Troubleshooting).count();
        assert_eq!(troubleshooting_count, 1);
    }
}