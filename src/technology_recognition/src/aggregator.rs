//! Evidence Aggregator — combines confidence scores from multiple detections.
//!
//! This module is intentionally minimal: no knowledge retrieval, RAG, MCP,
//! command execution, automation, screen AI, OCR, or autonomous actions.

use wikilabs_data_types::TechnologyInference;

/// Combine multiple inferences for the same technology by averaging confidence.
pub fn aggregate_by_technology(inferences: Vec<TechnologyInference>) -> Vec<TechnologyInference> {
    let mut by_tech: std::collections::HashMap<String, Vec<TechnologyInference>> =
        std::collections::HashMap::new();
    for inference in inferences {
        by_tech
            .entry(inference.name.clone())
            .or_default()
            .push(inference);
    }
    by_tech
        .into_values()
        .map(|group| {
            let avg_confidence: f32 =
                group.iter().map(|i| i.confidence).sum::<f32>() / group.len() as f32;
            let mut evidence = Vec::new();
            for item in &group {
                evidence.push(format!("{}:{}", item.source, item.reason));
            }
            TechnologyInference::new(
                group[0].name.clone(),
                avg_confidence,
                format!("aggregated({})", group.len()),
                evidence.join(", "),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_single() {
        let inferences = vec![TechnologyInference::new(
            "linux",
            0.8,
            "rule-1",
            "systemctl",
        )];
        let result = aggregate_by_technology(inferences);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].confidence, 0.8);
    }

    #[test]
    fn test_aggregate_multiple_same_technology() {
        let inferences = vec![
            TechnologyInference::new("linux", 0.8, "rule-1", "systemctl"),
            TechnologyInference::new("linux", 0.9, "rule-2", "journalctl"),
            TechnologyInference::new("linux", 0.7, "rule-3", "dmesg"),
        ];
        let result = aggregate_by_technology(inferences);
        assert_eq!(result.len(), 1);
        // Average: (0.8 + 0.9 + 0.7) / 3 = 0.8
        assert!((result[0].confidence - 0.8).abs() < 0.001);
        // Should have 3 evidence items
        assert_eq!(result[0].reason.split(", ").count(), 3);
    }

    #[test]
    fn test_aggregate_different_technologies() {
        let inferences = vec![
            TechnologyInference::new("linux", 0.8, "rule-1", "systemctl"),
            TechnologyInference::new("docker", 0.9, "rule-2", "docker ps"),
        ];
        let result = aggregate_by_technology(inferences);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_aggregate_empty() {
        let inferences: Vec<TechnologyInference> = vec![];
        let result = aggregate_by_technology(inferences);
        assert!(result.is_empty());
    }
}
