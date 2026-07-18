//! Workspace config extension for knowledge pack management.

use uuid::Uuid;

/// Extension for workspace config to manage knowledge packs.
pub struct WorkspaceKnowledgeConfig {
    workspace_id: Uuid,
}

impl WorkspaceKnowledgeConfig {
    /// Creates a new workspace knowledge config.
    pub fn new(workspace_id: Uuid) -> Self {
        Self { workspace_id }
    }

    /// Returns the workspace ID.
    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    /// Returns the workspace ID as a string.
    pub fn workspace_id_str(&self) -> String {
        self.workspace_id.to_string()
    }
}