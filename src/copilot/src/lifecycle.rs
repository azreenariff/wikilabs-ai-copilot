//! Recommendation lifecycle management.
//!
//! Every recommendation flows through a strict lifecycle:
//!
//! ```text
//! Candidate → Ready → Displayed → Accepted → Completed → Archived
//!                      │
//!                      └── Dismissed → (end)
//! ```
//!
//! The same recommendation should not repeatedly appear.

use crate::{Confidence, Priority, Recommendation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Lifecycle state for a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LifecycleState {
    /// Recommendation is being assembled but not yet ready to show.
    Candidate,
    /// Recommendation passed the Decision Engine — ready to display.
    Ready,
    /// Recommendation is currently displayed to the engineer.
    Displayed,
    /// Engineer accepted the recommendation.
    Accepted,
    /// Engineer dismissed the recommendation.
    Dismissed,
    /// Accepted recommendation was completed.
    Completed,
    /// Recommendation is archived for reference.
    Archived,
}

impl LifecycleState {
    /// Returns true if this state is terminal (no further transitions possible).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            LifecycleState::Completed | LifecycleState::Dismissed | LifecycleState::Archived
        )
    }

    /// Returns the display order for showing recommendations.
    /// Active states come before terminal states.
    pub fn display_priority(&self) -> u8 {
        match self {
            LifecycleState::Candidate => 0,
            LifecycleState::Ready => 1,
            LifecycleState::Displayed => 2,
            LifecycleState::Accepted => 3,
            LifecycleState::Completed => 4,
            LifecycleState::Dismissed => 5,
            LifecycleState::Archived => 6,
        }
    }
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecycleState::Candidate => write!(f, "Candidate"),
            LifecycleState::Ready => write!(f, "Ready"),
            LifecycleState::Displayed => write!(f, "Displayed"),
            LifecycleState::Accepted => write!(f, "Accepted"),
            LifecycleState::Dismissed => write!(f, "Dismissed"),
            LifecycleState::Completed => write!(f, "Completed"),
            LifecycleState::Archived => write!(f, "Archived"),
        }
    }
}

/// Tracks the lifecycle state for a recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRecord {
    pub recommendation_id: uuid::Uuid,
    pub state: LifecycleState,
    pub timestamp: DateTime<Utc>,
    pub previous_state: Option<LifecycleState>,
}

impl LifecycleRecord {
    pub fn new(recommendation_id: uuid::Uuid, state: LifecycleState) -> Self {
        LifecycleRecord {
            recommendation_id,
            state,
            timestamp: Utc::now(),
            previous_state: None,
        }
    }

    pub fn with_previous(mut self, previous: LifecycleState) -> Self {
        self.previous_state = Some(previous);
        self
    }
}

/// Manages the lifecycle of recommendations.
///
/// Ensures recommendations flow through the correct states
/// and the same recommendation doesn't repeatedly appear.
pub struct RecommendationLifecycle {
    states: HashMap<uuid::Uuid, LifecycleState>,
    history: Vec<LifecycleRecord>,
    /// Track dismissed recommendations to avoid re-display.
    dismissed_ids: Vec<uuid::Uuid>,
    /// Track shown recommendations to avoid repetition.
    shown_ids: Vec<uuid::Uuid>,
    /// Maximum history to keep.
    max_history: usize,
}

impl RecommendationLifecycle {
    pub fn new() -> Self {
        RecommendationLifecycle {
            states: HashMap::new(),
            history: Vec::new(),
            dismissed_ids: Vec::new(),
            shown_ids: Vec::new(),
            max_history: 1000,
        }
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Create a new recommendation in Candidate state.
    pub fn create(&mut self, recommendation: &Recommendation) {
        let id = recommendation.id;
        if !self.states.contains_key(&id) {
            self.states.insert(id, LifecycleState::Candidate);
            self.record_transition(id, LifecycleState::Candidate, None);
        }
    }

    /// Mark a recommendation as ready to display.
    pub fn mark_ready(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
        let _current = self.transition(
            recommendation_id,
            LifecycleState::Candidate,
            LifecycleState::Ready,
        )?;
        Ok(())
    }

    /// Mark a recommendation as being displayed.
    pub fn mark_displayed(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
        self.transition(
            recommendation_id,
            LifecycleState::Ready,
            LifecycleState::Displayed,
        )?;
        if !self.shown_ids.contains(&recommendation_id) {
            self.shown_ids.push(recommendation_id);
        }
        Ok(())
    }

    /// Mark a recommendation as accepted by the engineer.
    pub fn mark_accepted(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
        self.transition(
            recommendation_id,
            LifecycleState::Displayed,
            LifecycleState::Accepted,
        )?;
        Ok(())
    }

    /// Mark an accepted recommendation as completed.
    pub fn mark_completed(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
        self.transition(
            recommendation_id,
            LifecycleState::Accepted,
            LifecycleState::Completed,
        )?;
        Ok(())
    }

    /// Dismiss a recommendation.
    pub fn dismiss(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
        let current = self.state(recommendation_id)?;
        if current == LifecycleState::Candidate {
            self.transition(
                recommendation_id,
                LifecycleState::Candidate,
                LifecycleState::Dismissed,
            )?;
        } else {
            self.transition(recommendation_id, current, LifecycleState::Dismissed)?;
        }
        if !self.dismissed_ids.contains(&recommendation_id) {
            self.dismissed_ids.push(recommendation_id);
        }
        Ok(())
    }

    /// Archive a recommendation (any terminal state).
    pub fn archive(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
        let current = self.state(recommendation_id)?;
        self.transition(recommendation_id, current, LifecycleState::Archived)?;
        Ok(())
    }

    /// Transition to a state — validates the transition is allowed.
    fn transition(
        &mut self,
        id: uuid::Uuid,
        from: LifecycleState,
        to: LifecycleState,
    ) -> Result<LifecycleState, String> {
        // Validate transition
        if !is_valid_transition(from, to) {
            return Err(format!(
                "Invalid transition: {from} -> {to} for recommendation {id}"
            ));
        }

        let prev = self.states.get(&id).cloned();
        self.states.insert(id, to);
        self.record_transition(id, to, prev);
        Ok(to)
    }

    /// Record a state transition in the history.
    fn record_transition(
        &mut self,
        id: uuid::Uuid,
        to: LifecycleState,
        prev: Option<LifecycleState>,
    ) {
        let record = LifecycleRecord::new(id, to).with_previous(prev.unwrap_or(to));
        self.history.push(record);

        // Trim history if needed
        if self.history.len() > self.max_history {
            self.history.drain(..self.history.len() - self.max_history);
        }
    }

    /// Get the current state for a recommendation.
    pub fn state(&self, recommendation_id: uuid::Uuid) -> Result<LifecycleState, String> {
        self.states
            .get(&recommendation_id)
            .copied()
            .ok_or_else(|| format!("Recommendation {recommendation_id} not found"))
    }

    /// Check if a recommendation is ready to show (Candidate or Ready state).
    pub fn is_ready(&self, recommendation_id: uuid::Uuid) -> bool {
        matches!(
            self.states.get(&recommendation_id),
            Some(&LifecycleState::Candidate | LifecycleState::Ready)
        )
    }

    /// Check if a recommendation has already been shown.
    pub fn has_been_shown(&self, recommendation_id: uuid::Uuid) -> bool {
        self.shown_ids.contains(&recommendation_id)
    }

    /// Check if a recommendation has been dismissed.
    pub fn is_dismissed(&self, recommendation_id: uuid::Uuid) -> bool {
        self.dismissed_ids.contains(&recommendation_id)
    }

    /// Check if a recommendation is in a terminal state.
    pub fn is_terminal(&self, recommendation_id: uuid::Uuid) -> bool {
        matches!(self.states.get(&recommendation_id), Some(s) if s.is_terminal())
    }

    /// Get all active (non-terminal) recommendations.
    pub fn active_recommendations(&self) -> Vec<(uuid::Uuid, LifecycleState)> {
        self.states
            .iter()
            .filter(|(_, s)| !s.is_terminal())
            .map(|(id, &state)| (*id, state))
            .collect()
    }

    /// Get the full history of state transitions.
    pub fn history(&self) -> &[LifecycleRecord] {
        &self.history
    }

    /// Get the number of dismissed recommendations.
    pub fn dismissed_count(&self) -> usize {
        self.dismissed_ids.len()
    }

    /// Get the number of shown recommendations.
    pub fn shown_count(&self) -> usize {
        self.shown_ids.len()
    }
}

/// Validates whether a state transition is allowed.
fn is_valid_transition(from: LifecycleState, to: LifecycleState) -> bool {
    match from {
        LifecycleState::Candidate => {
            matches!(to, LifecycleState::Ready | LifecycleState::Dismissed)
        }
        LifecycleState::Ready => {
            matches!(to, LifecycleState::Displayed | LifecycleState::Dismissed)
        }
        LifecycleState::Displayed => {
            matches!(to, LifecycleState::Accepted | LifecycleState::Dismissed)
        }
        LifecycleState::Accepted => {
            matches!(to, LifecycleState::Completed | LifecycleState::Dismissed)
        }
        LifecycleState::Completed => false, // Terminal
        LifecycleState::Dismissed => false, // Terminal
        LifecycleState::Archived => false,  // Terminal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Evidence;

    fn make_rec(id: &str) -> Recommendation {
        let mut u = uuid::Uuid::nil();
        u = uuid::Uuid::parse_str(id).unwrap_or(uuid::Uuid::new_v4());
        Recommendation {
            id: u,
            title: "Test".into(),
            description: "Test".into(),
            reason: "Test".into(),
            confidence: Confidence::new(0.8),
            evidence: vec![],
            priority: Priority::Suggestion,
            supporting_documents: vec![],
            suggested_next_step: None,
            created_at: Utc::now(),
            workflow_context: None,
        }
    }

    #[test]
    fn test_lifecycle_candidate_to_ready() {
        let mut ll = RecommendationLifecycle::new();
        let rec = make_rec("11111111-1111-1111-1111-111111111111");
        ll.create(&rec);
        assert!(ll.is_ready(rec.id));
        ll.mark_ready(rec.id).unwrap();
        assert_eq!(ll.state(rec.id).unwrap(), LifecycleState::Ready);
    }

    #[test]
    fn test_lifecycle_full_flow() {
        let mut ll = RecommendationLifecycle::new();
        let rec = make_rec("22222222-2222-2222-2222-222222222222");
        ll.create(&rec);
        ll.mark_ready(rec.id).unwrap();
        ll.mark_displayed(rec.id).unwrap();
        ll.mark_accepted(rec.id).unwrap();
        ll.mark_completed(rec.id).unwrap();
        assert_eq!(ll.state(rec.id).unwrap(), LifecycleState::Completed);
        assert!(ll.is_terminal(rec.id));
    }

    #[test]
    fn test_lifecycle_dismiss() {
        let mut ll = RecommendationLifecycle::new();
        let rec = make_rec("33333333-3333-3333-3333-333333333333");
        ll.create(&rec);
        ll.dismiss(rec.id).unwrap();
        assert!(ll.is_dismissed(rec.id));
        assert!(ll.is_terminal(rec.id));
    }

    #[test]
    fn test_lifecycle_invalid_transition() {
        let mut ll = RecommendationLifecycle::new();
        let rec = make_rec("44444444-4444-4444-4444-444444444444");
        ll.create(&rec);
        ll.mark_completed(rec.id).unwrap();
        // Cannot transition from Completed
        assert!(ll
            .transition(rec.id, LifecycleState::Completed, LifecycleState::Ready)
            .is_err());
    }

    #[test]
    fn test_not_found() {
        let ll = RecommendationLifecycle::new();
        let unknown = uuid::Uuid::new_v4();
        assert!(ll.state(unknown).is_err());
    }

    #[test]
    fn test_shown_tracking() {
        let mut ll = RecommendationLifecycle::new();
        let rec = make_rec("55555555-5555-5555-5555-555555555555");
        ll.create(&rec);
        ll.mark_ready(rec.id).unwrap();
        ll.mark_displayed(rec.id).unwrap();
        assert!(ll.has_been_shown(rec.id));
    }

    #[test]
    fn test_active_recommendations() {
        let mut ll = RecommendationLifecycle::new();
        let r1 = make_rec("66666666-6666-6666-6666-666666666666");
        let r2 = make_rec("77777777-7777-7777-7777-777777777777");
        ll.create(&r1);
        ll.create(&r2);
        ll.mark_ready(r1.id).unwrap();
        ll.dismiss(r2.id).unwrap();
        let active = ll.active_recommendations();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].0, r1.id);
    }

    #[test]
    fn test_display_priority_ordering() {
        let states = [
            LifecycleState::Candidate,
            LifecycleState::Ready,
            LifecycleState::Displayed,
            LifecycleState::Accepted,
            LifecycleState::Completed,
            LifecycleState::Dismissed,
            LifecycleState::Archived,
        ];
        for i in 0..states.len() {
            for j in 0..i {
                assert!(states[i].display_priority() > states[j].display_priority());
            }
        }
    }

    #[test]
    fn test_terminal_states() {
        assert!(LifecycleState::Completed.is_terminal());
        assert!(LifecycleState::Dismissed.is_terminal());
        assert!(LifecycleState::Archived.is_terminal());
        assert!(!LifecycleState::Candidate.is_terminal());
        assert!(!LifecycleState::Ready.is_terminal());
        assert!(!LifecycleState::Displayed.is_terminal());
        assert!(!LifecycleState::Accepted.is_terminal());
    }
}
