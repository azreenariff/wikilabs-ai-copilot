//! Workspace data types with lifecycle tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A workspace represents a customer's environment and context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    /// Unique workspace identifier.
    pub id: Uuid,
    /// Display name of the workspace.
    pub name: String,
    /// Name of the customer/organization.
    pub customer_name: String,
    /// Technology stack in use (e.g., ["OpenShift", "Linux"]).
    pub technology_stack: Vec<String>,
    /// When the workspace was created.
    pub created_at: DateTime<Utc>,
    /// When the workspace was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Workspace {
    /// Create a new workspace with current timestamps.
    pub fn new(name: impl Into<String>, customer_name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            customer_name: customer_name.into(),
            technology_stack: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a technology to the stack (if not already present).
    pub fn add_technology(&mut self, tech: &str) {
        if !self.technology_stack.iter().any(|t| t == tech) {
            self.technology_stack.push(tech.to_string());
            self.updated_at = Utc::now();
        }
    }

    /// Update the workspace's updated timestamp.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let ws = Workspace::new("Test", "Test Customer");
        assert_eq!(ws.name, "Test");
        assert_eq!(ws.customer_name, "Test Customer");
        assert!(ws.technology_stack.is_empty());
    }

    #[test]
    fn test_add_technology() {
        let mut ws = Workspace::new("Test", "Test Customer");
        ws.add_technology("OpenShift");
        assert_eq!(ws.technology_stack, vec!["OpenShift"]);
        ws.add_technology("OpenShift"); // duplicate, should not add
        assert_eq!(ws.technology_stack, vec!["OpenShift"]);
    }
}
