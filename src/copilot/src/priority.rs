//! Priority module for the Copilot Engine.
//!
//! Manages recommendation priority levels (Critical, Warning, Suggestion, Information)
//! and provides utilities for sorting, filtering, and policy-aware priority decisions.
//!
//! Priority influences presentation order and which items pass policy filtering.
//!
//! ## Priority Levels
//!
//! - **Critical** — Immediate attention required, always shown
//! - **Warning** — Important issues that should be addressed
//! - **Suggestion** — Helpful recommendations for improvement
//! - **Information** — Low-priority context and awareness

use crate::Priority;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Priority filter for policy-based recommendation selection.
///
/// Each filter defines which priority levels it includes,
/// enabling the Policy Engine to control recommendation visibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorityFilter {
    /// Policy level that this filter corresponds to.
    pub policy_level: crate::policy::PolicyLevel,
    /// Priority levels included in this filter.
    pub included_priorities: Vec<Priority>,
}

impl PriorityFilter {
    /// Create a new priority filter for a given policy level.
    pub fn new(policy_level: crate::policy::PolicyLevel, priorities: Vec<Priority>) -> Self {
        PriorityFilter {
            policy_level,
            included_priorities: priorities,
        }
    }

    /// Check if a given priority passes this filter.
    pub fn passes(&self, priority: &Priority) -> bool {
        self.included_priorities.contains(priority)
    }

    /// Get the default filter for a given policy level.
    pub fn default_for_policy(policy_level: crate::policy::PolicyLevel) -> Self {
        match policy_level {
            crate::policy::PolicyLevel::Minimal => {
                PriorityFilter::new(policy_level, vec![Priority::Critical])
            }
            crate::policy::PolicyLevel::Balanced => {
                PriorityFilter::new(policy_level, vec![Priority::Critical, Priority::Warning])
            }
            crate::policy::PolicyLevel::Teaching | crate::policy::PolicyLevel::Expert => {
                PriorityFilter::new(
                    policy_level,
                    vec![Priority::Critical, Priority::Warning, Priority::Suggestion],
                )
            }
            crate::policy::PolicyLevel::Silent => PriorityFilter::new(policy_level, vec![]),
        }
    }

    /// Get the most urgent priority level present in the filter.
    pub fn most_urgent(&self) -> Option<Priority> {
        self.included_priorities
            .iter()
            .cloned()
            .max_by_key(|p| match p {
                Priority::Critical => 4,
                Priority::Warning => 3,
                Priority::Suggestion => 2,
                Priority::Information => 1,
            })
    }
}

/// Priority-weighted ranking for sorting recommendations.
///
/// Assigns a numeric score to each priority level so that
/// recommendations can be sorted by urgency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorityScore(pub u8);

impl PriorityScore {
    /// Convert a Priority to its numeric score.
    pub fn from_priority(priority: Priority) -> Self {
        match priority {
            Priority::Critical => PriorityScore(4),
            Priority::Warning => PriorityScore(3),
            Priority::Suggestion => PriorityScore(2),
            Priority::Information => PriorityScore(1),
        }
    }

    /// Convert a numeric score back to a Priority.
    pub fn to_priority(self) -> Option<Priority> {
        match self.0 {
            4 => Some(Priority::Critical),
            3 => Some(Priority::Warning),
            2 => Some(Priority::Suggestion),
            1 => Some(Priority::Information),
            _ => None,
        }
    }

    /// Check if this score represents an urgent priority.
    pub fn is_urgent(&self) -> bool {
        self.0 >= 3
    }

    /// Check if this score represents an informational priority.
    pub fn is_informational(&self) -> bool {
        self.0 <= 1
    }
}

impl fmt::Display for PriorityScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            4 => write!(f, "Critical"),
            3 => write!(f, "Warning"),
            2 => write!(f, "Suggestion"),
            1 => write!(f, "Information"),
            _ => write!(f, "Unknown"),
        }
    }
}

/// Priority filter context — holds the current filtering state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorityFilterContext {
    /// Current policy level.
    pub policy_level: crate::policy::PolicyLevel,
    /// Active filter.
    pub active_filter: PriorityFilter,
    /// Minimum score to show.
    pub min_score: u8,
}

impl PriorityFilterContext {
    /// Create a new priority filter context.
    pub fn new(policy_level: crate::policy::PolicyLevel) -> Self {
        let filter = PriorityFilter::default_for_policy(policy_level);
        let min_score = filter
            .most_urgent()
            .map(|p| PriorityScore::from_priority(p).0)
            .unwrap_or(0);
        PriorityFilterContext {
            policy_level,
            active_filter: filter,
            min_score,
        }
    }

    /// Update the context with a new policy level.
    pub fn with_policy(mut self, policy_level: crate::policy::PolicyLevel) -> Self {
        let filter = PriorityFilter::default_for_policy(policy_level);
        let min_score = filter
            .most_urgent()
            .map(|p| PriorityScore::from_priority(p).0)
            .unwrap_or(0);
        self.policy_level = policy_level;
        self.active_filter = filter;
        self.min_score = min_score;
        self
    }

    /// Check if a priority level passes the current filter.
    pub fn passes(&self, priority: &Priority) -> bool {
        self.active_filter.passes(priority)
    }

    /// Get the minimum score threshold.
    pub fn min_score(&self) -> u8 {
        self.min_score
    }
}

impl Default for PriorityFilterContext {
    fn default() -> Self {
        PriorityFilterContext::new(crate::policy::PolicyLevel::Balanced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_score_from_priority() {
        assert_eq!(PriorityScore::from_priority(Priority::Critical).0, 4);
        assert_eq!(PriorityScore::from_priority(Priority::Warning).0, 3);
        assert_eq!(PriorityScore::from_priority(Priority::Suggestion).0, 2);
        assert_eq!(PriorityScore::from_priority(Priority::Information).0, 1);
    }

    #[test]
    fn test_priority_score_to_priority() {
        assert_eq!(PriorityScore(4).to_priority(), Some(Priority::Critical));
        assert_eq!(PriorityScore(3).to_priority(), Some(Priority::Warning));
        assert_eq!(PriorityScore(2).to_priority(), Some(Priority::Suggestion));
        assert_eq!(PriorityScore(1).to_priority(), Some(Priority::Information));
        assert_eq!(PriorityScore(5).to_priority(), None);
    }

    #[test]
    fn test_priority_score_urgent() {
        assert!(PriorityScore(4).is_urgent());
        assert!(PriorityScore(3).is_urgent());
        assert!(!PriorityScore(2).is_urgent());
        assert!(!PriorityScore(1).is_urgent());
    }

    #[test]
    fn test_priority_score_informational() {
        assert!(!PriorityScore(4).is_informational());
        assert!(!PriorityScore(3).is_informational());
        assert!(!PriorityScore(2).is_informational());
        assert!(PriorityScore(1).is_informational());
    }

    #[test]
    fn test_priority_filter_default_for_policy() {
        let minimal = PriorityFilter::default_for_policy(crate::policy::PolicyLevel::Minimal);
        assert_eq!(minimal.included_priorities, vec![Priority::Critical]);

        let balanced = PriorityFilter::default_for_policy(crate::policy::PolicyLevel::Balanced);
        assert_eq!(
            balanced.included_priorities,
            vec![Priority::Critical, Priority::Warning]
        );

        let teaching = PriorityFilter::default_for_policy(crate::policy::PolicyLevel::Teaching);
        assert_eq!(
            teaching.included_priorities,
            vec![Priority::Critical, Priority::Warning, Priority::Suggestion]
        );

        let silent = PriorityFilter::default_for_policy(crate::policy::PolicyLevel::Silent);
        assert!(silent.included_priorities.is_empty());
    }

    #[test]
    fn test_priority_filter_passes() {
        let filter = PriorityFilter::new(
            crate::policy::PolicyLevel::Balanced,
            vec![Priority::Critical, Priority::Warning],
        );
        assert!(filter.passes(&Priority::Critical));
        assert!(filter.passes(&Priority::Warning));
        assert!(!filter.passes(&Priority::Suggestion));
        assert!(!filter.passes(&Priority::Information));
    }

    #[test]
    fn test_priority_filter_most_urgent() {
        let filter = PriorityFilter::new(
            crate::policy::PolicyLevel::Balanced,
            vec![
                Priority::Warning,
                Priority::Suggestion,
                Priority::Information,
            ],
        );
        assert_eq!(filter.most_urgent(), Some(Priority::Warning));
    }

    #[test]
    fn test_priority_filter_empty_most_urgent() {
        let filter = PriorityFilter::new(crate::policy::PolicyLevel::Silent, vec![]);
        assert_eq!(filter.most_urgent(), None);
    }

    #[test]
    fn test_priority_filter_context_default() {
        let ctx = PriorityFilterContext::default();
        assert_eq!(ctx.policy_level, crate::policy::PolicyLevel::Balanced);
        assert!(ctx.passes(&Priority::Critical));
        assert!(ctx.passes(&Priority::Warning));
        assert!(!ctx.passes(&Priority::Suggestion));
    }

    #[test]
    fn test_priority_filter_context_policy_change() {
        let ctx =
            PriorityFilterContext::default().with_policy(crate::policy::PolicyLevel::Teaching);
        assert_eq!(ctx.policy_level, crate::policy::PolicyLevel::Teaching);
        assert!(ctx.passes(&Priority::Critical));
        assert!(ctx.passes(&Priority::Warning));
        assert!(ctx.passes(&Priority::Suggestion));
        assert!(!ctx.passes(&Priority::Information));
    }

    #[test]
    fn test_priority_filter_context_min_score() {
        let ctx = PriorityFilterContext::default();
        assert_eq!(ctx.min_score(), 4); // Critical = 4
    }

    #[test]
    fn test_priority_filter_context_silent_min_score() {
        let ctx = PriorityFilterContext::default().with_policy(crate::policy::PolicyLevel::Silent);
        assert_eq!(ctx.min_score(), 0);
    }

    #[test]
    fn test_priority_score_display() {
        assert_eq!(format!("{}", PriorityScore(4)), "Critical");
        assert_eq!(format!("{}", PriorityScore(3)), "Warning");
        assert_eq!(format!("{}", PriorityScore(2)), "Suggestion");
        assert_eq!(format!("{}", PriorityScore(1)), "Information");
    }
}
