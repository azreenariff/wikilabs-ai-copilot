//! Workspace history — chat session management.

pub struct HistoryManager;

impl HistoryManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn add_entry(&self, _workspace_id: uuid::Uuid, _content: &str) -> anyhow::Result<()> {
        // TODO: Append to chat history
        anyhow::bail!("Not yet implemented")
    }

    pub async fn get_entries(&self, _workspace_id: uuid::Uuid, _limit: usize) -> anyhow::Result<Vec<String>> {
        // TODO: Fetch chat history
        anyhow::bail!("Not yet implemented")
    }
}