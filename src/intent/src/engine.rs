//! Intent recognition engine — rule-based + ML.

use regex::Regex;

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

/// Intent recognition engine using rule-based pattern matching.
pub struct IntentEngine {
    patterns: Vec<(Regex, Intent)>,
}

impl IntentEngine {
    pub fn new() -> Self {
        let mut engine = Self { patterns: Vec::new() };
        engine.init_default_patterns();
        engine
    }

    fn init_default_patterns(&mut self) {
        // Troubleshooting patterns
        self.add_pattern(
            r"(?i)(troubleshoot|debug|fix|error|issue|problem|crash|fail|bug)",
            Intent::Troubleshooting,
        );

        // Configuration patterns
        self.add_pattern(
            r"(?i)(config|configure|setting|settings|setup|configure|custom)",
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

    fn add_pattern(&mut self, regex_str: &str, intent: Intent) {
        if let Ok(regex) = Regex::new(regex_str) {
            self.patterns.push((regex, intent));
        }
    }

    pub fn recognize(&self, context: &str) -> Intent {
        for (regex, intent) in &self.patterns {
            if regex.is_match(context) {
                return intent.clone();
            }
        }
        Intent::Unknown
    }

    pub fn recognize_with_confidence(
        &self,
        context: &str,
    ) -> (Intent, f32) {
        let mut match_count = 0;
        let mut last_intent = Intent::Unknown;

        for (regex, intent) in &self.patterns {
            if regex.is_match(context) {
                match_count += 1;
                last_intent = intent.clone();
            }
        }

        // More matches = higher confidence
        let confidence = if match_count == 0 {
            0.1
        } else {
            (0.7 + 0.25 * (match_count - 1) as f32 / match_count as f32).min(0.95)
        };

        (last_intent, confidence)
    }

    pub fn intent_count(&self) -> usize {
        self.patterns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine_has_patterns() {
        let engine = IntentEngine::new();
        assert!(engine.intent_count() > 0);
    }

    #[test]
    fn test_recognize_troubleshooting() {
        let engine = IntentEngine::new();
        assert_eq!(engine.recognize("fix this crash"), Intent::Troubleshooting);
        assert_eq!(engine.recognize("debug the error"), Intent::Troubleshooting);
        assert_eq!(engine.recognize("solve the bug"), Intent::Troubleshooting);
    }

    #[test]
    fn test_recognize_configuration() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("configure the server"),
            Intent::Configuration,
        );
        assert_eq!(
            engine.recognize("settings for pod"),
            Intent::Configuration,
        );
    }

    #[test]
    fn test_recognize_deployment() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("deploy to production"),
            Intent::Deployment,
        );
        assert_eq!(
            engine.recognize("kubectl create deployment"),
            Intent::Deployment,
        );
    }

    #[test]
    fn test_recognize_documentation() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("write a readme"),
            Intent::Documentation,
        );
        assert_eq!(
            engine.recognize("explain how this works"),
            Intent::Documentation,
        );
    }

    #[test]
    fn test_recognize_learning() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("what is OpenShift"),
            Intent::Learning,
        );
        assert_eq!(
            engine.recognize("teach me about pods"),
            Intent::Learning,
        );
    }

    #[test]
    fn test_recognize_unknown() {
        let engine = IntentEngine::new();
        assert_eq!(engine.recognize("blabla random words"), Intent::Unknown);
        assert_eq!(engine.recognize(""), Intent::Unknown);
    }

    #[test]
    fn test_recognize_case_insensitive() {
        let engine = IntentEngine::new();
        assert_eq!(
            engine.recognize("TROUBLESHOOT the pod"),
            Intent::Troubleshooting,
        );
        assert_eq!(
            engine.recognize("Deploy to staging"),
            Intent::Deployment,
        );
    }

    #[test]
    fn test_recognize_with_confidence_single_match() {
        let engine = IntentEngine::new();
        let (intent, confidence) = engine.recognize_with_confidence("fix this crash");
        assert_eq!(intent, Intent::Troubleshooting);
        assert!(confidence >= 0.6 && confidence <= 0.8);
    }

    #[test]
    fn test_recognize_with_confidence_no_match() {
        let engine = IntentEngine::new();
        let (intent, confidence) = engine.recognize_with_confidence("random text");
        assert_eq!(intent, Intent::Unknown);
        assert_eq!(confidence, 0.1);
    }

    #[test]
    fn test_recognize_with_confidence_multiple_match() {
        let engine = IntentEngine::new();
        let (intent, confidence) =
            engine.recognize_with_confidence("configure and deploy the system");
        // Matches both Configuration and Deployment
        assert_eq!(intent, Intent::Deployment); // last match wins
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
}