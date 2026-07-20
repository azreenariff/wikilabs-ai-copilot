//! Session memory — tracks accepted, dismissed, corrected, and ignored recommendations.
//!
//! The Session Memory prevents the Copilot from repeatedly
//! suggesting the same things and adapts to engineer preferences.
//!
//! It tracks:
//! - Accepted recommendations (with timestamps)
//! - Dismissed recommendations (and reasons, when provided)
//! - Corrections (what the engineer did instead)
//! - Ignored recommendations (silent dismissals)
//! - Time since last correction
//! - Correction frequency (to detect persistent mismatches)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How the engineer handled a recommendation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Handling {
    Accepted,
    Dismissed { reason: Option<String> },
    Ignored,
    Corrected { instead: String },
}

/// A record of how a recommendation was handled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlingRecord {
    pub recommendation_id: uuid::Uuid,
    pub handling: Handling,
    pub timestamp: DateTime<Utc>,
}

/// Session memory that adapts the Copilot to engineer preferences.
///
/// Tracks accepted/dismissed/corrected recommendations and uses
/// this history to inform future recommendation generation.
pub struct SessionMemory {
    handling_history: Vec<HandlingRecord>,
    /// Count of each handling type.
    acceptance_count: u32,
    dismissal_count: u32,
    ignored_count: u32,
    correction_count: u32,
    /// Track recently accepted titles to avoid repetitive suggestions.
    accepted_titles: Vec<String>,
    /// Recently corrected recommendations with the correction details.
    corrections: Vec<(uuid::Uuid, String)>,
    /// Maximum history to keep.
    max_history: usize,
}

impl SessionMemory {
    pub fn new() -> Self {
        SessionMemory {
            handling_history: Vec::new(),
            acceptance_count: 0,
            dismissal_count: 0,
            ignored_count: 0,
            correction_count: 0,
            accepted_titles: Vec::new(),
            corrections: Vec::new(),
            max_history: 500,
        }
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Record that the engineer accepted a recommendation.
    pub fn record_accepted(&mut self, recommendation_id: uuid::Uuid, title: String) {
        self.handling_history.push(HandlingRecord {
            recommendation_id,
            handling: Handling::Accepted,
            timestamp: Utc::now(),
        });
        self.acceptance_count += 1;
        self.accepted_titles.push(title);
        self.trim_history();
    }

    /// Record that the engineer dismissed a recommendation with a reason.
    pub fn record_dismissed(&mut self, recommendation_id: uuid::Uuid, reason: Option<String>) {
        self.handling_history.push(HandlingRecord {
            recommendation_id,
            handling: Handling::Dismissed { reason },
            timestamp: Utc::now(),
        });
        self.dismissal_count += 1;
        self.trim_history();
    }

    /// Record that the engineer ignored a recommendation (silent dismissal).
    pub fn record_ignored(&mut self, recommendation_id: uuid::Uuid) {
        self.handling_history.push(HandlingRecord {
            recommendation_id,
            handling: Handling::Ignored,
            timestamp: Utc::now(),
        });
        self.ignored_count += 1;
        self.trim_history();
    }

    /// Record a correction — what the engineer did instead.
    pub fn record_correction(&mut self, recommendation_id: uuid::Uuid, instead: String) {
        self.handling_history.push(HandlingRecord {
            recommendation_id,
            handling: Handling::Corrected {
                instead: instead.clone(),
            },
            timestamp: Utc::now(),
        });
        self.correction_count += 1;
        self.corrections.push((recommendation_id, instead));
        self.trim_history();
    }

    /// Get the acceptance rate (acceptances / total interactions).
    pub fn acceptance_rate(&self) -> f64 {
        let total = self.acceptance_count
            + self.dismissal_count
            + self.ignored_count
            + self.correction_count;
        if total == 0 {
            return 1.0; // No history yet — assume all good
        }
        self.acceptance_count as f64 / total as f64
    }

    /// Get the correction rate (corrections / total interactions).
    pub fn correction_rate(&self) -> f64 {
        let total = self.acceptance_count
            + self.dismissal_count
            + self.ignored_count
            + self.correction_count;
        if total == 0 {
            return 0.0;
        }
        self.correction_count as f64 / total as f64
    }

    /// Check if a specific title was recently accepted.
    /// Used to avoid suggesting the same thing repeatedly.
    pub fn was_recently_accepted(&self, title: &str) -> bool {
        self.accepted_titles.iter().any(|t| t == title)
    }

    /// Get recently accepted titles for context.
    pub fn recent_accepted_titles(&self) -> &[String] {
        if self.accepted_titles.len() > 20 {
            &self.accepted_titles[self.accepted_titles.len() - 20..]
        } else {
            &self.accepted_titles
        }
    }

    /// Get recent corrections for context.
    pub fn recent_corrections(&self) -> &[(uuid::Uuid, String)] {
        if self.corrections.len() > 10 {
            &self.corrections[self.corrections.len() - 10..]
        } else {
            &self.corrections
        }
    }

    /// Get the confidence adjustment based on correction rate.
    /// High correction rate = reduce confidence on similar topics.
    pub fn confidence_adjustment_for_topic(&self, topic: &str) -> f64 {
        let recent = self.recent_corrections();
        let matching = recent
            .iter()
            .filter(|(_, instead)| instead.to_lowercase().contains(&topic.to_lowercase()))
            .count();

        if matching > 3 {
            0.7 // Significant penalty — engineer frequently corrects
        } else if matching > 1 {
            0.85 // Moderate penalty
        } else {
            1.0 // No penalty
        }
    }

    /// Get all handling records.
    pub fn handling_history(&self) -> &[HandlingRecord] {
        &self.handling_history
    }

    /// Get counts of each handling type.
    pub fn counts(&self) -> (u32, u32, u32, u32) {
        (
            self.acceptance_count,
            self.dismissal_count,
            self.ignored_count,
            self.correction_count,
        )
    }

    /// Clear the session memory (e.g., at session boundary).
    pub fn clear(&mut self) {
        self.handling_history.clear();
        self.acceptance_count = 0;
        self.dismissal_count = 0;
        self.ignored_count = 0;
        self.correction_count = 0;
        self.accepted_titles.clear();
        self.corrections.clear();
    }

    /// Check if there's any history.
    pub fn has_history(&self) -> bool {
        !self.handling_history.is_empty()
    }

    /// Get total interactions.
    pub fn total_interactions(&self) -> u32 {
        self.acceptance_count + self.dismissal_count + self.ignored_count + self.correction_count
    }

    fn trim_history(&mut self) {
        if self.handling_history.len() > self.max_history {
            let excess = self.handling_history.len() - self.max_history;
            self.handling_history.drain(..excess);
            // Trim accepted_titles
            if self.accepted_titles.len() > 100 {
                self.accepted_titles
                    .drain(..self.accepted_titles.len() - 100);
            }
            // Trim corrections
            if self.corrections.len() > 50 {
                self.corrections.drain(..self.corrections.len() - 50);
            }
        }
    }
}

impl Default for SessionMemory {
    fn default() -> Self {
        SessionMemory::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_memory_default() {
        let mem = SessionMemory::new();
        assert_eq!(mem.acceptance_rate(), 1.0);
        assert_eq!(mem.correction_rate(), 0.0);
        assert!(!mem.has_history());
        assert_eq!(mem.total_interactions(), 0);
    }

    #[test]
    fn test_record_accepted() {
        let mut mem = SessionMemory::new();
        let id = uuid::Uuid::new_v4();
        mem.record_accepted(id, "Check Memory".to_string());
        assert_eq!(mem.total_interactions(), 1);
        assert_eq!(mem.acceptance_rate(), 1.0);
        assert!(mem.was_recently_accepted("Check Memory"));
    }

    #[test]
    fn test_record_dismissed() {
        let mut mem = SessionMemory::new();
        let id = uuid::Uuid::new_v4();
        mem.record_dismissed(id, Some("Not relevant".into()));
        assert_eq!(mem.dismissal_count, 1);
        assert_eq!(mem.total_interactions(), 1);
        assert_eq!(mem.acceptance_rate(), 0.0);
    }

    #[test]
    fn test_record_correction() {
        let mut mem = SessionMemory::new();
        let id = uuid::Uuid::new_v4();
        mem.record_correction(id, "I restarted the pod instead".into());
        assert_eq!(mem.correction_count, 1);
        let corrections = mem.recent_corrections();
        assert_eq!(corrections.len(), 1);
        assert!(corrections[0].1.contains("restarted"));
    }

    #[test]
    fn test_acceptance_rate_mixed() {
        let mut mem = SessionMemory::new();
        let id = uuid::Uuid::new_v4();
        mem.record_accepted(id, "Rec 1".into());
        let id2 = uuid::Uuid::new_v4();
        mem.record_dismissed(id2, None);
        let id3 = uuid::Uuid::new_v4();
        mem.record_accepted(id3, "Rec 2".into());
        assert_eq!(mem.acceptance_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_topic_correction_adjustment() {
        let mut mem = SessionMemory::new();
        // Simulate 4 corrections about "memory"
        for i in 0..4 {
            let id = uuid::Uuid::new_v4();
            mem.record_correction(id, format!("Fixed memory issue {i} instead"));
        }
        let adjustment = mem.confidence_adjustment_for_topic("memory");
        assert!(adjustment <= 0.71);
    }

    #[test]
    fn test_no_topic_correction_adjustment() {
        let mut mem = SessionMemory::new();
        let id = uuid::Uuid::new_v4();
        mem.record_correction(id, "Fixed networking issue instead".into());
        let adjustment = mem.confidence_adjustment_for_topic("memory");
        assert_eq!(adjustment, 1.0);
    }

    #[test]
    fn test_session_memory_clear() {
        let mut mem = SessionMemory::new();
        let id = uuid::Uuid::new_v4();
        mem.record_accepted(id, "Rec".into());
        assert!(mem.has_history());
        mem.clear();
        assert!(!mem.has_history());
        assert_eq!(mem.total_interactions(), 0);
    }

    #[test]
    fn test_handling_record() {
        let record = HandlingRecord {
            recommendation_id: uuid::Uuid::new_v4(),
            handling: Handling::Accepted,
            timestamp: Utc::now(),
        };
        assert_eq!(format!("{:?}", record.handling), "Accepted");

        let disc = HandlingRecord {
            recommendation_id: uuid::Uuid::new_v4(),
            handling: Handling::Dismissed {
                reason: Some("bad".into()),
            },
            timestamp: Utc::now(),
        };
        assert!(matches!(disc.handling, Handling::Dismissed { .. }));
    }
}
