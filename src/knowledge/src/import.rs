//! Knowledge import pipeline.

pub struct ImportPipeline;

impl ImportPipeline {
    pub fn new() -> Self {
        Self
    }

    pub async fn import_file(&self, _path: &str) -> anyhow::Result<Vec<crate::doc::KnowledgeChunk>> {
        // TODO: Parse file, chunk, embed, index
        anyhow::bail!("Not yet implemented")
    }

    pub async fn import_text(&self, _title: &str, _content: &str) -> anyhow::Result<Vec<crate::doc::KnowledgeChunk>> {
        // TODO: Chunk text, embed, index
        anyhow::bail!("Not yet implemented")
    }
}