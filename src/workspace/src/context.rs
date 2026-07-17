//! Workspace context — knowledge association, skill selection.

pub struct ContextManager;

impl ContextManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_context(&self, _workspace_id: uuid::Uuid) -> anyhow::Result<serde_json::Value> {
        // TODO: Build context from workspace config + active knowledge
        anyhow::bail!("Not yet implemented")
    }
}