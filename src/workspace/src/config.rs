//! Workspace configuration — technology stack, metadata.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub description: Option<String>,
    pub technology_stack: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl WorkspaceConfig {
    pub fn new(name: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.to_string(),
            description: None,
            technology_stack: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self.updated_at = chrono::Utc::now();
        self
    }

    pub fn add_technology(&mut self, tech: &str) {
        if !self.technology_stack.contains(&tech.to_string()) {
            self.technology_stack.push(tech.to_string());
        }
        self.updated_at = chrono::Utc::now();
    }

    pub fn remove_technology(&mut self, tech: &str) {
        self.technology_stack.retain(|t| t != tech);
        self.updated_at = chrono::Utc::now();
    }

    pub fn technologies(&self) -> &[String] {
        &self.technology_stack
    }

    pub fn has_technology(&self, tech: &str) -> bool {
        self.technology_stack.iter().any(|t| t == tech)
    }

    pub fn tech_count(&self) -> usize {
        self.technology_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        let cfg = WorkspaceConfig::new("test-workspace");
        assert_eq!(cfg.name, "test-workspace");
        assert!(cfg.technology_stack.is_empty());
        assert!(cfg.description.is_none());
    }

    #[test]
    fn test_with_description() {
        let cfg = WorkspaceConfig::new("ws").with_description("A test workspace");
        assert_eq!(cfg.description, Some("A test workspace".to_string()));
    }

    #[test]
    fn test_add_technology() {
        let mut cfg = WorkspaceConfig::new("ws");
        cfg.add_technology("rust");
        cfg.add_technology("kubernetes");
        assert_eq!(cfg.tech_count(), 2);
        assert!(cfg.has_technology("rust"));
    }

    #[test]
    fn test_add_technology_no_duplicates() {
        let mut cfg = WorkspaceConfig::new("ws");
        cfg.add_technology("rust");
        cfg.add_technology("rust");
        assert_eq!(cfg.tech_count(), 1);
    }

    #[test]
    fn test_remove_technology() {
        let mut cfg = WorkspaceConfig::new("ws");
        cfg.add_technology("rust");
        cfg.add_technology("go");
        cfg.remove_technology("rust");
        assert_eq!(cfg.tech_count(), 1);
        assert!(!cfg.has_technology("rust"));
        assert!(cfg.has_technology("go"));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut cfg = WorkspaceConfig::new("ws");
        cfg.remove_technology("nonexistent");
        assert_eq!(cfg.tech_count(), 0);
    }

    #[test]
    fn test_serialization() {
        let cfg = WorkspaceConfig::new("ws");
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: WorkspaceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "ws");
    }
}