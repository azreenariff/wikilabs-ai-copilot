//! Human approval framework — engineer must approve before action.
//!
//! The Copilot never performs work autonomously. Every recommendation
//! requires explicit human approval before any action is taken.
//!
//! This module defines the approval states and approval flow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Approval states for a recommendation action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalState {
    /// Awaiting engineer approval.
    Pending,
    /// Engineer approved.
    Approved,
    /// Engineer denied.
    Denied,
    /// Auto-approved based on policy (e.g., non-destructive).
    AutoApproved,
}

impl fmt::Display for ApprovalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApprovalState::Pending => write!(f, "Pending"),
            ApprovalState::Approved => write!(f, "Approved"),
            ApprovalState::Denied => write!(f, "Denied"),
            ApprovalState::AutoApproved => write!(f, "Auto-approved"),
        }
    }
}

/// A request for engineer approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub recommendation_id: uuid::Uuid,
    pub action_description: String,
    pub state: ApprovalState,
    pub requested_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub denial_reason: Option<String>,
    /// Whether this action is non-destructive and can be auto-approved.
    pub is_non_destructive: bool,
}

impl ApprovalRequest {
    pub fn new(recommendation_id: uuid::Uuid, action_description: impl Into<String>) -> Self {
        ApprovalRequest {
            recommendation_id,
            action_description: action_description.into(),
            state: ApprovalState::Pending,
            requested_at: Utc::now(),
            approved_at: None,
            denial_reason: None,
            is_non_destructive: false,
        }
    }

    pub fn with_non_destructive(mut self) -> Self {
        self.is_non_destructive = true;
        self
    }

    /// Approve this request.
    pub fn approve(mut self) -> Self {
        self.state = ApprovalState::Approved;
        self.approved_at = Some(Utc::now());
        self
    }

    /// Deny this request.
    pub fn deny(mut self, reason: impl Into<String>) -> Self {
        self.state = ApprovalState::Denied;
        self.denial_reason = Some(reason.into());
        self
    }

    /// Auto-approve if non-destructive.
    pub fn auto_approve_if_safe(mut self) -> Self {
        if self.is_non_destructive {
            self.state = ApprovalState::AutoApproved;
            self.approved_at = Some(Utc::now());
        }
        self
    }

    /// Check if this request can proceed (approved or auto-approved).
    pub fn can_proceed(&self) -> bool {
        matches!(
            self.state,
            ApprovalState::Approved | ApprovalState::AutoApproved
        )
    }

    /// Check if this request is still pending.
    pub fn is_pending(&self) -> bool {
        self.state == ApprovalState::Pending
    }

    /// Get a summary for the engineer.
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} — {}",
            self.state, self.recommendation_id, self.action_description
        )
    }
}

/// The Human Approval Framework — manages the approval flow.
///
/// Ensures every action requires explicit human approval
/// before being executed. Non-destructive actions may be
/// auto-approved based on policy.
#[derive(Debug, Clone)]
pub struct HumanApproval {
    requests: Vec<ApprovalRequest>,
}

impl HumanApproval {
    pub fn new() -> Self {
        HumanApproval {
            requests: Vec::new(),
        }
    }

    /// Create a new approval request.
    pub fn create_request(
        &mut self,
        recommendation_id: uuid::Uuid,
        action: impl Into<String>,
    ) -> ApprovalRequest {
        let request = ApprovalRequest::new(recommendation_id, action);
        self.requests.push(request);
        // Return the request for builder chain, then re-find it
        self.requests.last().unwrap().clone()
    }

    /// Approve a request by ID.
    pub fn approve(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
        match self
            .requests
            .iter_mut()
            .find(|r| r.recommendation_id == recommendation_id)
        {
            Some(r) if r.is_pending() => {
                r.state = ApprovalState::Approved;
                r.approved_at = Some(Utc::now());
                Ok(())
            }
            Some(_) => Err("Request is no longer pending".to_string()),
            None => Err(format!("Request for {recommendation_id} not found")),
        }
    }

    /// Deny a request by ID.
    pub fn deny(&mut self, recommendation_id: uuid::Uuid, reason: String) -> Result<(), String> {
        match self
            .requests
            .iter_mut()
            .find(|r| r.recommendation_id == recommendation_id)
        {
            Some(r) if r.is_pending() => {
                r.state = ApprovalState::Denied;
                r.denial_reason = Some(reason);
                Ok(())
            }
            Some(_) => Err("Request is no longer pending".to_string()),
            None => Err(format!("Request for {recommendation_id} not found")),
        }
    }

    /// Get pending requests.
    pub fn pending_requests(&self) -> Vec<&ApprovalRequest> {
        self.requests.iter().filter(|r| r.is_pending()).collect()
    }

    /// Check if a request has been resolved (approved or denied).
    pub fn is_resolved(&self, recommendation_id: uuid::Uuid) -> bool {
        self.requests.iter().any(|r| {
            r.recommendation_id == recommendation_id && r.can_proceed()
                || matches!(r.state, ApprovalState::Denied)
        })
    }

    /// Count pending requests.
    pub fn pending_count(&self) -> usize {
        self.requests.iter().filter(|r| r.is_pending()).count()
    }

    /// Get all requests.
    pub fn all_requests(&self) -> &[ApprovalRequest] {
        &self.requests
    }
}

impl Default for HumanApproval {
    fn default() -> Self {
        HumanApproval::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_pending() {
        let mut approval = HumanApproval::new();
        let id = uuid::Uuid::new_v4();
        let req = approval.create_request(id, "Check memory usage");
        assert!(req.is_pending());
        assert_eq!(req.state, ApprovalState::Pending);
    }

    #[test]
    fn test_approval_approve() {
        let mut approval = HumanApproval::new();
        let id = uuid::Uuid::new_v4();
        let req = approval.create_request(id, "Check memory");
        assert!(!req.can_proceed()); // Created as Pending, not yet approved
        approval.approve(id);
        let req2 = approval
            .requests
            .iter()
            .find(|r| r.recommendation_id == id)
            .unwrap();
        assert!(req2.can_proceed()); // Now approved
    }

    #[test]
    fn test_approval_deny() {
        let mut approval = HumanApproval::new();
        let id = uuid::Uuid::new_v4();
        let req = approval
            .create_request(id, "Check memory")
            .deny("Not relevant");
        assert!(!req.can_proceed());
        assert_eq!(req.state, ApprovalState::Denied);
    }

    #[test]
    fn test_approval_auto_approve_non_destructive() {
        let req = ApprovalRequest::new(uuid::Uuid::new_v4(), "View log")
            .with_non_destructive()
            .auto_approve_if_safe();
        assert_eq!(req.state, ApprovalState::AutoApproved);
        assert!(req.can_proceed());
    }

    #[test]
    fn test_approval_approve_method() {
        let mut approval = HumanApproval::new();
        let id = uuid::Uuid::new_v4();
        approval.create_request(id, "Check memory");
        assert!(approval.approve(id).is_ok());
        assert!(approval.is_resolved(id));
    }

    #[test]
    fn test_approval_deny_method() {
        let mut approval = HumanApproval::new();
        let id = uuid::Uuid::new_v4();
        approval.create_request(id, "Check memory");
        assert!(approval.deny(id, "Wrong".into()).is_ok());
    }

    #[test]
    fn test_approval_not_found() {
        let mut approval = HumanApproval::new();
        let id = uuid::Uuid::new_v4();
        assert!(approval.approve(id).is_err());
    }

    #[test]
    fn test_approval_count() {
        let mut approval = HumanApproval::new();
        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();
        approval.create_request(id1, "A");
        approval.create_request(id2, "B");
        assert_eq!(approval.pending_count(), 2);
    }

    #[test]
    fn test_approval_summary() {
        let req = ApprovalRequest::new(uuid::Uuid::new_v4(), "Check memory");
        let summary = req.summary();
        assert!(summary.contains("Check memory"));
        assert!(summary.contains("Pending"));
    }

    #[test]
    fn test_approval_state_display() {
        assert_eq!(format!("{}", ApprovalState::Pending), "Pending");
        assert_eq!(format!("{}", ApprovalState::Approved), "Approved");
        assert_eq!(format!("{}", ApprovalState::Denied), "Denied");
        assert_eq!(format!("{}", ApprovalState::AutoApproved), "Auto-approved");
    }
}
