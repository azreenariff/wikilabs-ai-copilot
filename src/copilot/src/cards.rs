//! Recommendation cards for desktop UI.
//!
//! Cards display recommendations in the desktop sidebar with
//! clear, concise information and actionable buttons.
//!
//! Display: Title, Technology, Confidence, Priority, Reason,
//! Supporting documents, and Actions (Explain, Open Docs, Dismiss, Complete).

use crate::{Confidence, Priority, Recommendation};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Actions available on a recommendation card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardAction {
    /// Show full explanation with evidence.
    Explain,
    /// Open supporting documentation.
    OpenDocumentation { doc_index: usize },
    /// Dismiss the recommendation.
    Dismiss,
    /// Mark as complete.
    MarkComplete,
}

impl fmt::Display for CardAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardAction::Explain => write!(f, "Explain"),
            CardAction::OpenDocumentation { doc_index } => {
                write!(f, "Open Documentation (#{})", doc_index + 1)
            }
            CardAction::Dismiss => write!(f, "Dismiss"),
            CardAction::MarkComplete => write!(f, "Mark Complete"),
        }
    }
}

/// A recommendation card for display in the desktop UI.
///
/// Contains a condensed view of a recommendation with
/// all information needed for the engineer to decide
/// whether to act on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationCard {
    pub recommendation_id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub confidence: Confidence,
    pub technology: String,
    pub reason: String,
    pub suggested_next_step: Option<String>,
    pub available_actions: Vec<CardAction>,
    /// Number of supporting documents.
    pub supporting_doc_count: usize,
    /// Whether this card has been interacted with.
    pub is_interactive: bool,
}

impl RecommendationCard {
    /// Create a card from a recommendation.
    pub fn from_recommendation(rec: &Recommendation) -> Self {
        // Extract primary technology from evidence sources
        let technology = rec
            .evidence
            .first()
            .map(|e| {
                if e.source.contains("SOP") {
                    "SOP".to_string()
                } else if e.source.contains("internal") {
                    "Internal".to_string()
                } else {
                    e.source.clone()
                }
            })
            .unwrap_or_else(|| "General".to_string());

        RecommendationCard {
            recommendation_id: rec.id,
            title: rec.title.clone(),
            description: rec.description.clone(),
            priority: rec.priority,
            confidence: rec.confidence,
            technology,
            reason: rec.reason.clone(),
            suggested_next_step: rec.suggested_next_step.clone(),
            available_actions: vec![
                CardAction::Explain,
                if !rec.supporting_documents.is_empty() {
                    CardAction::OpenDocumentation { doc_index: 0 }
                } else {
                    CardAction::Dismiss
                },
                CardAction::Dismiss,
                CardAction::MarkComplete,
            ],
            supporting_doc_count: rec.supporting_documents.len(),
            is_interactive: true,
        }
    }

    /// Get the color associated with this card's priority.
    pub fn priority_color(&self) -> &str {
        match self.priority {
            Priority::Critical => "#ef4444",    // red
            Priority::Warning => "#f59e0b",     // amber
            Priority::Suggestion => "#3b82f6",  // blue
            Priority::Information => "#6b7280", // gray
        }
    }

    /// Get the icon associated with this card's priority.
    pub fn priority_icon(&self) -> &str {
        match self.priority {
            Priority::Critical => "⚠️",
            Priority::Warning => "🔶",
            Priority::Suggestion => "💡",
            Priority::Information => "ℹ️",
        }
    }

    /// Create a simplified card with minimal information.
    /// Used when the engineer is active (less detail).
    pub fn from_recommendation_minimal(rec: &Recommendation) -> Self {
        let mut card = Self::from_recommendation(rec);
        card.is_interactive = false;
        card.description = Self::truncated_description(&card.description, 80);
        // Remove OpenDocumentation from actions for minimal view
        card.available_actions
            .retain(|a| !matches!(a, CardAction::OpenDocumentation { .. }));
        card
    }

    fn truncated_description(desc: &str, max_len: usize) -> String {
        if desc.len() <= max_len {
            desc.to_string()
        } else {
            format!("{}...", &desc[..max_len.saturating_sub(3)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Confidence, Evidence, Priority};

    fn make_rec() -> Recommendation {
        Recommendation::new(
            "Check Pod Memory",
            "Pod memory usage is approaching limits",
            "Memory monitoring indicates elevated usage",
            Confidence::new(0.85),
            vec![
                Evidence {
                    source: "Internal Monitoring".into(),
                    description: "Memory at 85% of limit".into(),
                    confidence: Confidence::new(0.9),
                },
                Evidence {
                    source: "SOP Memory Management".into(),
                    description: "Step 3: Check memory".into(),
                    confidence: Confidence::new(0.8),
                },
            ],
            Priority::Warning,
            vec!["memory-sop.md".into()],
        )
    }

    #[test]
    fn test_card_from_recommendation() {
        let rec = make_rec();
        let card = RecommendationCard::from_recommendation(&rec);
        assert_eq!(card.title, "Check Pod Memory");
        assert_eq!(card.priority, Priority::Warning);
        assert_eq!(card.confidence.score, 0.85);
        assert_eq!(card.supporting_doc_count, 1);
        assert!(card.is_interactive);
    }

    #[test]
    fn test_card_priority_color() {
        let mut rec = make_rec();
        rec.priority = Priority::Critical;
        let card = RecommendationCard::from_recommendation(&rec);
        assert_eq!(card.priority_color(), "#ef4444");

        rec.priority = Priority::Suggestion;
        let card = RecommendationCard::from_recommendation(&rec);
        assert_eq!(card.priority_color(), "#3b82f6");
    }

    #[test]
    fn test_card_priority_icon() {
        let mut rec = make_rec();
        rec.priority = Priority::Critical;
        let card = RecommendationCard::from_recommendation(&rec);
        assert_eq!(card.priority_icon(), "⚠️");

        rec.priority = Priority::Information;
        let card = RecommendationCard::from_recommendation(&rec);
        assert_eq!(card.priority_icon(), "ℹ️");
    }

    #[test]
    fn test_card_minimal_view() {
        let rec = make_rec();
        let card = RecommendationCard::from_recommendation_minimal(&rec);
        assert!(!card.is_interactive);
        assert!(card.description.len() <= 83); // 80 + "..."
    }

    #[test]
    fn test_card_actions() {
        let rec = make_rec();
        let card = RecommendationCard::from_recommendation(&rec);
        assert!(!card.available_actions.is_empty());
        assert!(card
            .available_actions
            .iter()
            .any(|a| matches!(a, CardAction::Explain)));
        assert!(card
            .available_actions
            .iter()
            .any(|a| matches!(a, CardAction::Dismiss)));
    }

    #[test]
    fn test_card_truncated_description() {
        let desc = "This is a very long description that should be truncated because it exceeds the maximum length allowed for display in the minimal card view";
        let truncated = RecommendationCard::truncated_description(desc, 80);
        assert!(truncated.len() < desc.len());
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_no_evidence_technology() {
        let mut rec = make_rec();
        rec.evidence.clear();
        rec.supporting_documents.clear();
        let card = RecommendationCard::from_recommendation(&rec);
        assert_eq!(card.technology, "General");
        // Without docs, OpenDocumentation replaced with Dismiss
        assert!(card
            .available_actions
            .iter()
            .all(|a| !matches!(a, CardAction::OpenDocumentation { .. })));
    }

    #[test]
    fn test_card_action_display() {
        assert_eq!(format!("{}", CardAction::Explain), "Explain");
        assert_eq!(format!("{}", CardAction::Dismiss), "Dismiss");
        assert_eq!(format!("{}", CardAction::MarkComplete), "Mark Complete");
        assert_eq!(
            format!("{}", CardAction::OpenDocumentation { doc_index: 2 }),
            "Open Documentation (#3)"
        );
    }
}
