//! Intent confidence scoring.

pub struct ConfidenceScore {
    pub intent: crate::engine::Intent,
    pub confidence: f32,   // 0.0 - 1.0
}

pub struct ConfidenceEngine;

impl ConfidenceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn score(&self, input: f32) -> ConfidenceScore {
        ConfidenceScore {
            intent: crate::engine::Intent::Unknown,
            confidence: input,
        }
    }
}