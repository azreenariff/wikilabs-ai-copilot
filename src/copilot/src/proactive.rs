//! Proactive assistance — when and how the Copilot interrupts.
//!
//! The Copilot proactively suggests improvements when the engineer
//! is idle or working on related tasks. It avoids interrupting
//! during critical work.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Signals that trigger proactive suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProactiveSignal {
    /// Engineer is idle (no keystrokes for X seconds).
    Idle { seconds_idle: u64 },
    /// Engineer is working on a related task.
    RelatedWork { technology: String },
    /// Error condition detected.
    ErrorDetected { error_type: String },
    /// Resource threshold approached.
    ResourceThreshold { metric: String, value: String },
    /// New information available.
    NewInformation { source: String },
}

/// Proactive assistance engine.
///
/// Decides when to interrupt with suggestions based on signals.
pub struct ProactiveAssistance {
    /// Minimum idle time before suggesting (seconds).
    idle_threshold_seconds: u64,
    /// Track recent signals to avoid flooding.
    recent_signals: Vec<(ProactiveSignal, chrono::DateTime<chrono::Utc>)>,
    max_recent_signals: usize,
}

impl ProactiveAssistance {
    pub fn new() -> Self {
        ProactiveAssistance {
            idle_threshold_seconds: 30,
            recent_signals: Vec::new(),
            max_recent_signals: 50,
        }
    }

    pub fn with_idle_threshold(mut self, seconds: u64) -> Self {
        self.idle_threshold_seconds = seconds;
        self
    }

    /// Evaluate a signal and decide if a proactive suggestion should be made.
    pub fn should_interrupt(&self, signal: &ProactiveSignal, confidence: f64) -> bool {
        // Always allow critical-level confidence
        if confidence >= 0.9 {
            return true;
        }

        // Filter signals by type
        match signal {
            ProactiveSignal::Idle { seconds_idle } => {
                // Only interrupt if idle long enough
                *seconds_idle >= self.idle_threshold_seconds
            }
            ProactiveSignal::RelatedWork { .. } => {
                // Moderate confidence for related work
                confidence >= 0.6
            }
            ProactiveSignal::ErrorDetected { .. } => {
                // Errors always warrant attention
                confidence >= 0.7
            }
            ProactiveSignal::ResourceThreshold { .. } => {
                // Thresholds are important but not critical
                confidence >= 0.75
            }
            ProactiveSignal::NewInformation { .. } => {
                // New info needs some confidence
                confidence >= 0.65
            }
        }
    }

    /// Record a signal to track flooding.
    pub fn record_signal(&mut self, signal: ProactiveSignal) {
        self.recent_signals.push((signal, Utc::now()));
        if self.recent_signals.len() > self.max_recent_signals {
            self.recent_signals
                .drain(..self.recent_signals.len() - self.max_recent_signals);
        }
    }

    /// Check if too many signals in recent history (flooding prevention).
    pub fn is_flooding(&self, window_seconds: u64) -> bool {
        let now = Utc::now();
        let recent = self
            .recent_signals
            .iter()
            .filter(|(_, ts)| {
                let delta = now.signed_duration_since(*ts);
                delta.num_seconds() < window_seconds as i64
            })
            .count();
        recent > 10
    }

    /// Get recent signals for context.
    pub fn recent_signals(&self) -> &[(ProactiveSignal, chrono::DateTime<chrono::Utc>)] {
        &self.recent_signals
    }
}

impl Default for ProactiveAssistance {
    fn default() -> Self {
        ProactiveAssistance::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_above_threshold() {
        let pa = ProactiveAssistance::new();
        let signal = ProactiveSignal::Idle { seconds_idle: 60 };
        assert!(pa.should_interrupt(&signal, 0.5));
    }

    #[test]
    fn test_idle_below_threshold() {
        let pa = ProactiveAssistance::new();
        let signal = ProactiveSignal::Idle { seconds_idle: 10 };
        assert!(!pa.should_interrupt(&signal, 0.5));
    }

    #[test]
    fn test_error_always_warrants_attention() {
        let pa = ProactiveAssistance::new();
        let signal = ProactiveSignal::ErrorDetected {
            error_type: "OOM".into(),
        };
        assert!(pa.should_interrupt(&signal, 0.7));
    }

    #[test]
    fn test_high_confidence_always_interruption() {
        let pa = ProactiveAssistance::new();
        let signal = ProactiveSignal::NewInformation {
            source: "git diff".into(),
        };
        assert!(pa.should_interrupt(&signal, 0.95));
    }

    #[test]
    fn test_related_work_threshold() {
        let pa = ProactiveAssistance::new();
        let signal = ProactiveSignal::RelatedWork {
            technology: "Rust".into(),
        };
        assert!(pa.should_interrupt(&signal, 0.65));
        assert!(!pa.should_interrupt(&signal, 0.5));
    }

    #[test]
    fn test_resource_threshold() {
        let pa = ProactiveAssistance::new();
        let signal = ProactiveSignal::ResourceThreshold {
            metric: "memory".into(),
            value: "85%".into(),
        };
        assert!(pa.should_interrupt(&signal, 0.8));
        assert!(!pa.should_interrupt(&signal, 0.7));
    }

    #[test]
    fn test_flooding_detection() {
        let mut pa = ProactiveAssistance::new();
        // Add many signals
        for i in 0..15 {
            pa.record_signal(ProactiveSignal::NewInformation {
                source: format!("source {i}"),
            });
        }
        assert!(pa.is_flooding(60));
    }

    #[test]
    fn test_flooding_resets() {
        let mut pa = ProactiveAssistance::new();
        for _ in 0..5 {
            pa.record_signal(ProactiveSignal::NewInformation {
                source: "test".into(),
            });
        }
        assert!(!pa.is_flooding(60));
    }

    #[test]
    fn test_idle_threshold_configurable() {
        let pa = ProactiveAssistance::new().with_idle_threshold(120);
        assert!(!pa.should_interrupt(&ProactiveSignal::Idle { seconds_idle: 60 }, 0.5));
    }
}
