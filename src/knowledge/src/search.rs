//! Knowledge search — hybrid VSS + FTS5.

pub struct SearchEngine;

#[derive(Debug)]
pub struct SearchQuery {
    pub text: String,
    pub workspace_id: uuid::Uuid,
}

#[derive(Debug)]
pub struct SearchResult {
    pub chunk_id: String,
    pub score: f32,
    pub content: String,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn search(&self, _query: &SearchQuery) -> anyhow::Result<Vec<SearchResult>> {
        // TODO: Hybrid search (70% vector + 30% FTS5)
        anyhow::bail!("Not yet implemented")
    }
}