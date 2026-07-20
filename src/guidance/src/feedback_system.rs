/// Feature 10 — Guidance Feedback System
///
/// Allows engineers to provide feedback on AI recommendations.
///
/// Feedback types:
/// - Useful: The recommendation helped
/// - Not useful: The recommendation did not help
/// - Already completed: The step was already done
/// - Incorrect: The recommendation was wrong
/// - Different approach: The engineer used a different method
///
/// Feedback influences current-session behavior only.
/// It does NOT trigger autonomous learning.
///
/// The AI adapts its recommendations based on feedback patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of feedback the engineer can provide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// The recommendation was helpful.
    Useful,
    /// The recommendation did not help.
    NotUseful,
    /// The step was already completed by the engineer.
    AlreadyCompleted,
    /// The recommendation was incorrect or misleading.
    Incorrect,
    /// The engineer used a different approach.
    DifferentApproach,
}

impl std::fmt::Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Useful => write!(f, "Useful"),
            Self::NotUseful => write!(f, "Not useful"),
            Self::AlreadyCompleted => write!(f, "Already completed"),
            Self::Incorrect => write!(f, "Incorrect"),
            Self::DifferentApproach => write!(f, "Different approach"),
        }
    }
}

/// Feedback from the engineer on a specific recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineerFeedback {
    /// Unique feedback identifier.
    pub id: Uuid,
    /// The recommendation this feedback is for.
    pub recommendation_id: Uuid,
    /// Type of feedback.
    pub feedback_type: FeedbackType,
    /// Optional additional notes from the engineer.
    pub notes: Option<String>,
    /// When the feedback was given.
    pub timestamp: DateTime<Utc>,
}

impl EngineerFeedback {
    /// Create new feedback for a recommendation.
    pub fn new(recommendation_id: Uuid, feedback_type: FeedbackType, notes: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            recommendation_id,
            feedback_type,
            notes,
            timestamp: Utc::now(),
        }
    }

    /// Check if this feedback is positive (useful).
    pub fn is_positive(&self) -> bool {
        matches!(self.feedback_type, FeedbackType::Useful)
    }

    /// Check if this feedback indicates the recommendation should be avoided.
    pub fn is_negative(&self) -> bool {
        matches!(
            self.feedback_type,
            FeedbackType::NotUseful | FeedbackType::Incorrect
        )
    }

    /// Check if this feedback indicates the step was already done.
    pub fn is_redundant(&self) -> bool {
        matches!(self.feedback_type, FeedbackType::AlreadyCompleted)
    }
}

/// Statistics about feedback patterns in a session.
#[derive(Debug, Clone)]
pub struct FeedbackStats {
    /// Total feedback received.
    pub total: usize,
    /// Positive feedback count.
    pub useful_count: usize,
    /// Negative feedback count.
    pub not_useful_count: usize,
    /// Redundant (already completed) count.
    pub redundant_count: usize,
    /// Incorrect feedback count.
    pub incorrect_count: usize,
    /// Different approach count.
    pub different_approach_count: usize,
    /// Average helpfulness score (0.0 to 1.0).
    pub average_helpfulness: f64,
}

impl FeedbackStats {
    /// Calculate statistics from a list of feedback items.
    pub fn from_feedback(feedback: &[EngineerFeedback]) -> Self {
        let total = feedback.len();
        let mut useful_count = 0;
        let mut not_useful_count = 0;
        let mut redundant_count = 0;
        let mut incorrect_count = 0;
        let mut different_approach_count = 0;
        let mut helpfulness_sum: f64 = 0.0;

        for fb in feedback {
            match fb.feedback_type {
                FeedbackType::Useful => {
                    useful_count += 1;
                    helpfulness_sum += 1.0;
                }
                FeedbackType::NotUseful => {
                    not_useful_count += 1;
                    helpfulness_sum += 0.0;
                }
                FeedbackType::AlreadyCompleted => {
                    redundant_count += 1;
                    helpfulness_sum += 0.5; // Partially helpful, just not needed now
                }
                FeedbackType::Incorrect => {
                    incorrect_count += 1;
                    helpfulness_sum += 0.0;
                }
                FeedbackType::DifferentApproach => {
                    different_approach_count += 1;
                    helpfulness_sum += 0.5; // Engineer found an alternative
                }
            }
        }

        let average_helpfulness = if total > 0 {
            helpfulness_sum / total as f64
        } else {
            0.5 // Neutral default
        };

        Self {
            total,
            useful_count,
            not_useful_count,
            redundant_count,
            incorrect_count,
            different_approach_count,
            average_helpfulness,
        }
    }

    /// Check if feedback is generally positive.
    pub fn is_positive(&self) -> bool {
        self.total > 0 && self.average_helpfulness > 0.6
    }

    /// Check if there are too many incorrect recommendations.
    pub fn needs_adjustment(&self) -> bool {
        self.total > 0 && (self.incorrect_count as f64 / self.total as f64) > 0.3
    }
}

/// The feedback system manager.
///
/// Collects feedback and provides session-level adaptation.
pub struct FeedbackSystem {
    feedback: Vec<EngineerFeedback>,
    /// Recommendations that should be suppressed based on feedback.
    suppressed_recommendations: Vec<Uuid>,
}

impl Default for FeedbackSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackSystem {
    /// Create a new feedback system.
    pub fn new() -> Self {
        Self {
            feedback: Vec::new(),
            suppressed_recommendations: Vec::new(),
        }
    }

    /// Record feedback for a recommendation.
    pub fn record(&mut self, feedback: EngineerFeedback) {
        self.feedback.push(feedback.clone());

        // Suppress this specific recommendation ID based on negative feedback
        if feedback.is_negative() || feedback.is_redundant() {
            if !self.suppressed_recommendations.contains(&feedback.recommendation_id) {
                self.suppressed_recommendations.push(feedback.recommendation_id);
            }
        }
    }

    /// Record feedback for a recommendation by ID.
    pub fn record_for(&mut self, recommendation_id: Uuid, feedback_type: FeedbackType, notes: Option<String>) {
        self.record(EngineerFeedback::new(recommendation_id, feedback_type, notes));
    }

    /// Check if a recommendation should be suppressed based on feedback.
    pub fn is_suppressed(&self, recommendation_id: &Uuid) -> bool {
        self.suppressed_recommendations.contains(recommendation_id)
    }

    /// Get all feedback recorded so far.
    pub fn all(&self) -> &[EngineerFeedback] {
        &self.feedback
    }

    /// Get feedback for a specific recommendation.
    pub fn for_recommendation(&self, recommendation_id: &Uuid) -> Vec<&EngineerFeedback> {
        self.feedback
            .iter()
            .filter(|f| f.recommendation_id == *recommendation_id)
            .collect()
    }

    /// Get statistics about current session feedback.
    pub fn stats(&self) -> FeedbackStats {
        FeedbackStats::from_feedback(&self.feedback)
    }

    /// Get the average helpfulness score.
    pub fn average_helpfulness(&self) -> f64 {
        self.stats().average_helpfulness
    }

    /// Clear all feedback (reset session).
    pub fn clear(&mut self) {
        self.feedback.clear();
        self.suppressed_recommendations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_types_display() {
        assert_eq!(format!("{}", FeedbackType::Useful), "Useful");
        assert_eq!(format!("{}", FeedbackType::NotUseful), "Not useful");
        assert_eq!(format!("{}", FeedbackType::AlreadyCompleted), "Already completed");
        assert_eq!(format!("{}", FeedbackType::Incorrect), "Incorrect");
        assert_eq!(format!("{}", FeedbackType::DifferentApproach), "Different approach");
    }

    #[test]
    fn test_feedback_positive() {
        let feedback = EngineerFeedback::new(
            Uuid::nil(),
            FeedbackType::Useful,
            None,
        );
        assert!(feedback.is_positive());
        assert!(!feedback.is_negative());
        assert!(!feedback.is_redundant());
    }

    #[test]
    fn test_feedback_negative() {
        let feedback = EngineerFeedback::new(
            Uuid::nil(),
            FeedbackType::NotUseful,
            None,
        );
        assert!(!feedback.is_positive());
        assert!(feedback.is_negative());
    }

    #[test]
    fn test_feedback_incorrect() {
        let feedback = EngineerFeedback::new(
            Uuid::nil(),
            FeedbackType::Incorrect,
            Some("Wrong command".to_string()),
        );
        assert!(feedback.is_negative());
        assert!(feedback.notes.as_ref().unwrap() == "Wrong command");
    }

    #[test]
    fn test_feedback_already_completed() {
        let feedback = EngineerFeedback::new(
            Uuid::nil(),
            FeedbackType::AlreadyCompleted,
            None,
        );
        assert!(feedback.is_redundant());
    }

    #[test]
    fn test_feedback_stats_useful_only() {
        let mut system = FeedbackSystem::new();
        system.record(EngineerFeedback::new(
            Uuid::new_v4(),
            FeedbackType::Useful,
            None,
        ));
        system.record(EngineerFeedback::new(
            Uuid::new_v4(),
            FeedbackType::Useful,
            None,
        ));

        let stats = system.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.useful_count, 2);
        assert_eq!(stats.average_helpfulness, 1.0);
        assert!(stats.is_positive());
    }

    #[test]
    fn test_feedback_stats_mixed() {
        let mut system = FeedbackSystem::new();
        system.record_for(Uuid::new_v4(), FeedbackType::Useful, None);
        system.record_for(Uuid::new_v4(), FeedbackType::NotUseful, None);
        system.record_for(Uuid::new_v4(), FeedbackType::AlreadyCompleted, None);

        let stats = system.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.useful_count, 1);
        assert_eq!(stats.not_useful_count, 1);
        assert_eq!(stats.redundant_count, 1);
        assert_eq!(stats.average_helpfulness, (1.0 + 0.0 + 0.5) / 3.0);
    }

    #[test]
    fn test_feedback_suppression() {
        let mut system = FeedbackSystem::new();
        let rec_id = Uuid::new_v4();

        system.record(EngineerFeedback::new(
            rec_id,
            FeedbackType::NotUseful,
            None,
        ));

        assert!(system.is_suppressed(&rec_id));
    }

    #[test]
    fn test_feedback_no_suppression_for_useful() {
        let mut system = FeedbackSystem::new();
        let rec_id = Uuid::new_v4();

        system.record(EngineerFeedback::new(
            rec_id,
            FeedbackType::Useful,
            None,
        ));

        assert!(!system.is_suppressed(&rec_id));
    }

    #[test]
    fn test_feedback_needs_adjustment() {
        let mut system = FeedbackSystem::new();

        // 3 incorrect out of 4 total = 75% > 30%
        for _ in 0..3 {
            system.record_for(Uuid::new_v4(), FeedbackType::Incorrect, None);
        }
        system.record_for(Uuid::new_v4(), FeedbackType::Useful, None);

        assert!(system.stats().needs_adjustment());
    }

    #[test]
    fn test_feedback_needs_adjustment_low_error_rate() {
        let mut system = FeedbackSystem::new();

        // 1 incorrect out of 5 total = 20% < 30%
        for _ in 0..4 {
            system.record_for(Uuid::new_v4(), FeedbackType::Useful, None);
        }
        system.record_for(Uuid::new_v4(), FeedbackType::Incorrect, None);

        assert!(!system.stats().needs_adjustment());
    }

    #[test]
    fn test_feedback_clear() {
        let mut system = FeedbackSystem::new();
        let rec_id = Uuid::new_v4();

        system.record(EngineerFeedback::new(
            rec_id,
            FeedbackType::NotUseful,
            None,
        ));

        assert_eq!(system.all().len(), 1);
        assert!(system.is_suppressed(&rec_id));

        system.clear();

        assert_eq!(system.all().len(), 0);
        assert!(!system.is_suppressed(&rec_id));
    }
}