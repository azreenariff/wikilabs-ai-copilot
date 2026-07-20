//! Conversation context — tracks what's been discussed to avoid repetition.
//!
//! The Copilot never repeatedly suggests the same thing.
//! It tracks conversation history, recommended topics, dismissed
//! topics, and pending follow-ups.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A single conversation turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub id: uuid::Uuid,
    pub turn_type: TurnType,
    pub content: String,
    pub recommendation_id: Option<uuid::Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnType {
    /// A recommendation was shown.
    RecommendationShown,
    /// Engineer accepted a recommendation.
    RecommendationAccepted,
    /// Engineer dismissed a recommendation.
    RecommendationDismissed,
    /// Engineer corrected a recommendation.
    RecommendationCorrected,
    /// Engineer asked a direct question.
    QuestionAsked,
    /// Copilot answered a question.
    QuestionAnswered,
    /// A new topic was introduced.
    NewTopic,
}

/// Tracks what's been discussed to avoid repetition and
/// provide context-aware follow-ups.
pub struct ConversationContext {
    turns: Vec<ConversationTurn>,
    shown_titles: HashSet<String>,
    dismissed_titles: HashSet<String>,
    accepted_titles: HashSet<String>,
    pending_topics: Vec<String>,
    max_history: usize,
}

impl ConversationContext {
    pub fn new() -> Self {
        ConversationContext {
            turns: Vec::new(),
            shown_titles: HashSet::new(),
            dismissed_titles: HashSet::new(),
            accepted_titles: HashSet::new(),
            pending_topics: Vec::new(),
            max_history: 200,
        }
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Record that a recommendation was shown.
    pub fn record_shown(&mut self, title: String, recommendation_id: uuid::Uuid) {
        self.shown_titles.insert(title.clone());
        self.turns.push(ConversationTurn {
            id: uuid::Uuid::new_v4(),
            turn_type: TurnType::RecommendationShown,
            content: title,
            recommendation_id: Some(recommendation_id),
            timestamp: Utc::now(),
        });
        self.trim();
    }

    /// Record that a recommendation was accepted.
    pub fn record_accepted(&mut self, title: String, recommendation_id: uuid::Uuid) {
        self.accepted_titles.insert(title);
        self.turns.push(ConversationTurn {
            id: uuid::Uuid::new_v4(),
            turn_type: TurnType::RecommendationAccepted,
            content: String::new(),
            recommendation_id: Some(recommendation_id),
            timestamp: Utc::now(),
        });
        self.trim();
    }

    /// Record that a recommendation was dismissed.
    pub fn record_dismissed(&mut self, title: String, recommendation_id: uuid::Uuid) {
        self.dismissed_titles.insert(title);
        self.turns.push(ConversationTurn {
            id: uuid::Uuid::new_v4(),
            turn_type: TurnType::RecommendationDismissed,
            content: String::new(),
            recommendation_id: Some(recommendation_id),
            timestamp: Utc::now(),
        });
        self.trim();
    }

    /// Record that a recommendation was corrected.
    pub fn record_corrected(
        &mut self,
        title: String,
        instead: String,
        recommendation_id: uuid::Uuid,
    ) {
        self.turns.push(ConversationTurn {
            id: uuid::Uuid::new_v4(),
            turn_type: TurnType::RecommendationCorrected,
            content: instead,
            recommendation_id: Some(recommendation_id),
            timestamp: Utc::now(),
        });
        self.trim();
    }

    /// Record a direct question from the engineer.
    pub fn record_question(&mut self, content: String) {
        self.turns.push(ConversationTurn {
            id: uuid::Uuid::new_v4(),
            turn_type: TurnType::QuestionAsked,
            content,
            recommendation_id: None,
            timestamp: Utc::now(),
        });
        self.trim();
    }

    /// Record a copilot answer to a question.
    pub fn record_answer(&mut self, content: String) {
        self.turns.push(ConversationTurn {
            id: uuid::Uuid::new_v4(),
            turn_type: TurnType::QuestionAnswered,
            content,
            recommendation_id: None,
            timestamp: Utc::now(),
        });
        self.trim();
    }

    /// Add a pending follow-up topic.
    pub fn add_pending_topic(&mut self, topic: String) {
        if !self.pending_topics.contains(&topic) {
            self.pending_topics.push(topic);
        }
    }

    /// Remove a pending follow-up topic.
    pub fn remove_pending_topic(&mut self, topic: &str) {
        self.pending_topics.retain(|t| t != topic);
    }

    /// Get all pending follow-up topics.
    pub fn pending_topics(&self) -> &[String] {
        &self.pending_topics
    }

    /// Check if a topic is acceptable to show (not recently dismissed).
    pub fn is_topic_acceptable(&self, title: &str) -> bool {
        !self.dismissed_titles.contains(title)
    }

    /// Check if a title was shown recently.
    pub fn was_shown(&self, title: &str) -> bool {
        self.shown_titles.contains(title)
    }

    /// Check if a title was accepted.
    pub fn was_accepted(&self, title: &str) -> bool {
        self.accepted_titles.contains(title)
    }

    /// Get recent conversation history (last N turns).
    pub fn recent_history(&self, max_turns: usize) -> &[ConversationTurn] {
        if self.turns.len() > max_turns {
            &self.turns[self.turns.len() - max_turns..]
        } else {
            &self.turns
        }
    }

    /// Get the total number of turns.
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    /// Clear all conversation context.
    pub fn clear(&mut self) {
        self.turns.clear();
        self.shown_titles.clear();
        self.dismissed_titles.clear();
        self.accepted_titles.clear();
        self.pending_topics.clear();
    }

    fn trim(&mut self) {
        if self.turns.len() > self.max_history {
            self.turns.drain(..self.turns.len() - self.max_history);
        }
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        ConversationContext::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_shown() {
        let mut ctx = ConversationContext::new();
        ctx.record_shown("Check Memory".into(), uuid::Uuid::new_v4());
        assert!(ctx.was_shown("Check Memory"));
        assert_eq!(ctx.turn_count(), 1);
    }

    #[test]
    fn test_record_accepted() {
        let mut ctx = ConversationContext::new();
        ctx.record_accepted("Check Memory".into(), uuid::Uuid::new_v4());
        assert!(ctx.was_accepted("Check Memory"));
    }

    #[test]
    fn test_record_dismissed() {
        let mut ctx = ConversationContext::new();
        ctx.record_dismissed("Check Memory".into(), uuid::Uuid::new_v4());
        assert!(!ctx.is_topic_acceptable("Check Memory"));
    }

    #[test]
    fn test_pending_topics() {
        let mut ctx = ConversationContext::new();
        ctx.add_pending_topic("Memory analysis".into());
        assert_eq!(ctx.pending_topics().len(), 1);
        ctx.remove_pending_topic("Memory analysis");
        assert_eq!(ctx.pending_topics().len(), 0);
    }

    #[test]
    fn test_duplicate_topic_not_added() {
        let mut ctx = ConversationContext::new();
        ctx.add_pending_topic("Memory".into());
        ctx.add_pending_topic("Memory".into());
        assert_eq!(ctx.pending_topics().len(), 1);
    }

    #[test]
    fn test_conversation_history() {
        let mut ctx = ConversationContext::new();
        let id = uuid::Uuid::new_v4();
        ctx.record_shown("Shown".into(), id);
        ctx.record_question("What is this?".into());
        ctx.record_answer("Explanation".into());
        assert_eq!(ctx.turn_count(), 3);
        let history = ctx.recent_history(2);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_turn_types() {
        for ty in [
            TurnType::RecommendationShown,
            TurnType::RecommendationAccepted,
            TurnType::RecommendationDismissed,
            TurnType::RecommendationCorrected,
            TurnType::QuestionAsked,
            TurnType::QuestionAnswered,
            TurnType::NewTopic,
        ] {
            let turn = ConversationTurn {
                id: uuid::Uuid::new_v4(),
                turn_type: ty.clone(),
                content: "test".into(),
                recommendation_id: None,
                timestamp: Utc::now(),
            };
            assert_eq!(format!("{:?}", ty), format!("{:?}", turn.turn_type));
        }
    }

    #[test]
    fn test_clear() {
        let mut ctx = ConversationContext::new();
        ctx.record_shown("R".into(), uuid::Uuid::new_v4());
        ctx.record_accepted("R".into(), uuid::Uuid::new_v4());
        ctx.add_pending_topic("T".into());
        ctx.clear();
        assert_eq!(ctx.turn_count(), 0);
        assert_eq!(ctx.pending_topics().len(), 0);
    }
}
