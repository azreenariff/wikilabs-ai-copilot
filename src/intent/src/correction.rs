//! Human correction mechanisms for intent recognition.
//!
//! Records corrections so the ML model can be retrained later.

/// A single correction record.
#[derive(Debug, Clone)]
pub struct CorrectionRecord {
    pub expected: crate::engine::Intent,
    pub actual: crate::engine::Intent,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: Option<String>,
}

/// Correction engine that records human corrections.
pub struct CorrectionEngine {
    corrections: Vec<CorrectionRecord>,
}

impl CorrectionEngine {
    pub fn new() -> Self {
        Self {
            corrections: Vec::new(),
        }
    }

    pub fn record_correction(
        &mut self,
        expected: crate::engine::Intent,
        actual: crate::engine::Intent,
        context: Option<String>,
    ) {
        self.corrections.push(CorrectionRecord {
            expected,
            actual,
            timestamp: chrono::Utc::now(),
            context,
        });
    }

    pub fn record_simple(&mut self, expected: crate::engine::Intent, actual: crate::engine::Intent) {
        self.record_correction(expected, actual, None);
    }

    pub fn correction_count(&self) -> usize {
        self.corrections.len()
    }

    pub fn corrections_for_intent(&self, intent: &crate::engine::Intent) -> usize {
        self.corrections
            .iter()
            .filter(|c| c.expected == *intent || c.actual == *intent)
            .count()
    }

    pub fn most_corrected_intent(&self) -> Option<crate::engine::Intent> {
        let mut counts: std::collections::HashMap<crate::engine::Intent, usize> =
            std::collections::HashMap::new();
        for c in &self.corrections {
            *counts.entry(c.expected.clone()).or_insert(0) += 1;
            *counts.entry(c.actual.clone()).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(intent, _)| intent)
    }

    pub fn error_rate(&self) -> f32 {
        if self.corrections.is_empty() {
            return 0.0;
        }
        // Percentage of corrections where actual != expected
        let mismatches = self
            .corrections
            .iter()
            .filter(|c| c.actual != c.expected)
            .count();
        mismatches as f32 / self.corrections.len() as f32
    }

    pub fn clear(&mut self) {
        self.corrections.clear();
    }

    pub fn recent(&self, limit: usize) -> &[CorrectionRecord] {
        let start = if self.corrections.len() > limit {
            self.corrections.len() - limit
        } else {
            0
        };
        &self.corrections[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine_empty() {
        let engine = CorrectionEngine::new();
        assert_eq!(engine.correction_count(), 0);
        assert_eq!(engine.error_rate(), 0.0);
    }

    #[test]
    fn test_record_correction() {
        let mut engine = CorrectionEngine::new();
        engine.record_simple(
            crate::engine::Intent::Troubleshooting,
            crate::engine::Intent::Unknown,
        );
        assert_eq!(engine.correction_count(), 1);
    }

    #[test]
    fn test_record_with_context() {
        let mut engine = CorrectionEngine::new();
        engine.record_correction(
            crate::engine::Intent::Configuration,
            crate::engine::Intent::Troubleshooting,
            Some("pod config issue".to_string()),
        );
        assert_eq!(engine.correction_count(), 1);
        assert_eq!(engine.corrections[0].context, Some("pod config issue".to_string()));
    }

    #[test]
    fn test_corrections_for_intent() {
        let mut engine = CorrectionEngine::new();
        engine.record_simple(
            crate::engine::Intent::Troubleshooting,
            crate::engine::Intent::Unknown,
        );
        engine.record_simple(
            crate::engine::Intent::Configuration,
            crate::engine::Intent::Unknown,
        );

        assert_eq!(
            engine.corrections_for_intent(&crate::engine::Intent::Troubleshooting),
            1
        );
        assert_eq!(
            engine.corrections_for_intent(&crate::engine::Intent::Unknown),
            2
        );
    }

    #[test]
    fn test_most_corrected_intent() {
        let mut engine = CorrectionEngine::new();
        engine.record_simple(
            crate::engine::Intent::Unknown,
            crate::engine::Intent::Troubleshooting,
        );
        engine.record_simple(
            crate::engine::Intent::Unknown,
            crate::engine::Intent::Configuration,
        );

        let most = engine.most_corrected_intent();
        assert!(most.is_some());
        // Unknown should be most corrected (appears in both)
        assert_eq!(most.unwrap(), crate::engine::Intent::Unknown);
    }

    #[test]
    fn test_error_rate() {
        let mut engine = CorrectionEngine::new();
        // All correct
        engine.record_simple(
            crate::engine::Intent::Troubleshooting,
            crate::engine::Intent::Troubleshooting,
        );
        assert_eq!(engine.error_rate(), 0.0);

        // One mismatch
        engine.record_simple(
            crate::engine::Intent::Unknown,
            crate::engine::Intent::Configuration,
        );
        assert_eq!(engine.error_rate(), 0.5);
    }

    #[test]
    fn test_clear() {
        let mut engine = CorrectionEngine::new();
        engine.record_simple(
            crate::engine::Intent::Unknown,
            crate::engine::Intent::Troubleshooting,
        );
        engine.clear();
        assert_eq!(engine.correction_count(), 0);
        assert_eq!(engine.error_rate(), 0.0);
    }

    #[test]
    fn test_recent() {
        let mut engine = CorrectionEngine::new();
        for i in 0..10 {
            engine.record_simple(
                crate::engine::Intent::Unknown,
                match i % 3 {
                    0 => crate::engine::Intent::Troubleshooting,
                    1 => crate::engine::Intent::Configuration,
                    _ => crate::engine::Intent::Deployment,
                },
            );
        }
        let recent = engine.recent(3);
        assert_eq!(recent.len(), 3);
    }
}