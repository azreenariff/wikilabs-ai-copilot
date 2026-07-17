//! Intent confidence scoring.

/// Confidence score for a recognized intent.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceScore {
    pub intent: crate::engine::Intent,
    pub confidence: f32, // 0.0 - 1.0
}

impl ConfidenceScore {
    pub fn is_confident(&self, threshold: f32) -> bool {
        self.confidence >= threshold
    }

    pub fn is_uncertain(&self) -> bool {
        self.confidence < 0.5
    }
}

/// Confidence scoring engine with configurable thresholds.
pub struct ConfidenceEngine {
    confident_threshold: f32,
    uncertain_threshold: f32,
}

impl ConfidenceEngine {
    pub fn new() -> Self {
        Self {
            confident_threshold: 0.7,
            uncertain_threshold: 0.5,
        }
    }

    pub fn new_with_thresholds(confident: f32, uncertain: f32) -> Self {
        Self {
            confident_threshold: confident,
            uncertain_threshold: uncertain,
        }
    }

    pub fn score(&self, input: f32, default_intent: crate::engine::Intent) -> ConfidenceScore {
        let confidence = input.clamp(0.0, 1.0);
        ConfidenceScore {
            intent: default_intent,
            confidence,
        }
    }

    pub fn classify(&self, score: f32) -> &str {
        let confidence = score.clamp(0.0, 1.0);
        if confidence >= self.confident_threshold {
            "confident"
        } else if confidence >= self.uncertain_threshold {
            "moderate"
        } else {
            "uncertain"
        }
    }

    pub fn set_confident_threshold(&mut self, threshold: f32) {
        self.confident_threshold = threshold;
    }

    pub fn set_uncertain_threshold(&mut self, threshold: f32) {
        self.uncertain_threshold = threshold;
    }

    pub fn thresholds(&self) -> (f32, f32) {
        (self.confident_threshold, self.uncertain_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine_default_thresholds() {
        let engine = ConfidenceEngine::new();
        assert_eq!(engine.thresholds(), (0.7, 0.5));
    }

    #[test]
    fn test_score_basic() {
        let engine = ConfidenceEngine::new();
        let score = engine.score(0.8, crate::engine::Intent::Troubleshooting);
        assert_eq!(score.confidence, 0.8);
        assert_eq!(score.intent, crate::engine::Intent::Troubleshooting);
    }

    #[test]
    fn test_score_clamp_0() {
        let engine = ConfidenceEngine::new();
        let score = engine.score(-0.5, crate::engine::Intent::Unknown);
        assert_eq!(score.confidence, 0.0);
    }

    #[test]
    fn test_score_clamp_1() {
        let engine = ConfidenceEngine::new();
        let score = engine.score(1.5, crate::engine::Intent::Unknown);
        assert_eq!(score.confidence, 1.0);
    }

    #[test]
    fn test_is_confident() {
        let engine = ConfidenceEngine::new();
        let score = engine.score(0.8, crate::engine::Intent::Unknown);
        assert!(score.is_confident(0.7));
        assert!(!score.is_confident(0.9));
    }

    #[test]
    fn test_is_uncertain() {
        let engine = ConfidenceEngine::new();
        let low = engine.score(0.3, crate::engine::Intent::Unknown);
        assert!(low.is_uncertain());

        let high = engine.score(0.6, crate::engine::Intent::Unknown);
        assert!(!high.is_uncertain());
    }

    #[test]
    fn test_classify_confident() {
        let engine = ConfidenceEngine::new();
        assert_eq!(engine.classify(0.9), "confident");
    }

    #[test]
    fn test_classify_moderate() {
        let engine = ConfidenceEngine::new();
        assert_eq!(engine.classify(0.6), "moderate");
    }

    #[test]
    fn test_classify_uncertain() {
        let engine = ConfidenceEngine::new();
        assert_eq!(engine.classify(0.3), "uncertain");
    }

    #[test]
    fn test_custom_thresholds() {
        let mut engine = ConfidenceEngine::new_with_thresholds(0.9, 0.8);
        assert_eq!(engine.classify(0.85), "moderate");
        assert_eq!(engine.classify(0.95), "confident");

        engine.set_confident_threshold(0.8);
        assert_eq!(engine.classify(0.85), "confident");
    }
}