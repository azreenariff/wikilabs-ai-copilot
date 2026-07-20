//! Technology inference — structured detections about the engineering environment.

use serde::{Deserialize, Serialize};

/// A technology inference — what the AI has detected about the environment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnologyInference {
    /// Name of the technology (e.g., "Rust", "Kubernetes", "Linux").
    pub name: String,
    /// Confidence in this inference (0.0–1.0).
    pub confidence: f32,
    /// Source of the inference ("observation", "analysis", "user").
    pub source: String,
    /// Why this technology was detected.
    pub reason: String,
}

impl TechnologyInference {
    /// Create a new technology inference.
    pub fn new(
        name: impl Into<String>,
        confidence: f32,
        source: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            confidence: confidence.clamp(0.0, 1.0),
            source: source.into(),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technology_inference_creation() {
        let inf = TechnologyInference::new("Rust", 0.9, "observation", "Found Cargo.toml");
        assert_eq!(inf.name, "Rust");
        assert_eq!(inf.confidence, 0.9);
        assert_eq!(inf.source, "observation");
        assert_eq!(inf.reason, "Found Cargo.toml");
    }

    #[test]
    fn test_confidence_clamping() {
        let inf = TechnologyInference::new("Test", 1.5, "obs", "reason");
        assert_eq!(inf.confidence, 1.0);

        let inf2 = TechnologyInference::new("Test", -0.3, "obs", "reason");
        assert_eq!(inf2.confidence, 0.0);
    }

    #[test]
    fn test_technology_inference_serialization() {
        let inf =
            TechnologyInference::new("Kubernetes", 0.85, "analysis", "Found kubectl references");
        let json = serde_json::to_string(&inf).unwrap();
        let deserialized: TechnologyInference = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Kubernetes");
        assert_eq!(deserialized.confidence, 0.85);
    }
}
