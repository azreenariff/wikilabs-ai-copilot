//! Workspace configuration — technology stack, metadata.

pub struct WorkspaceConfig;

impl WorkspaceConfig {
    pub fn new() -> Self {
        Self
    }

    pub fn add_technology(&mut self, _tech: &str) {
        // TODO: Add technology to stack
    }

    pub fn remove_technology(&mut self, _tech: &str) {
        // TODO: Remove technology from stack
    }

    pub fn technologies(&self) -> Vec<String> {
        // TODO: Return technology stack
        Vec::new()
    }
}