//! Index preparation step — format for vector indexing.

use crate::pipeline::DocPipelineState;
use crate::processing::DocumentElement;
use crate::doc::{KnowledgeDocument, KnowledgeChunk};
use crate::pipeline::PipelineConfig;
use chrono::{DateTime, Utc};
use tracing::debug;

/// The index preparation pipeline step.
pub struct IndexPrepareStep;

impl IndexPrepareStep {
    pub fn new() -> Self {
        Self
    }

    /// Run the index preparation step to create a KnowledgeDocument from pipeline state.
    pub fn run(&self, state: &DocPipelineState, doc: &super::discover::DiscoveredDoc) -> Option<KnowledgeDocument> {
        let discovered = state.discovered.as_ref()?;
        let parsed = state.parsed.as_ref()?;

        let now = Utc::now();

        let document = KnowledgeDocument {
            id: uuid::Uuid::new_v4(),
            title: parsed.title.clone(),
            source: doc.path.to_string_lossy().to_string(),
            workspace_id: discovered.workspace_id,
            author: parsed.author.clone(),
            created_at: now,
            updated_at: now,
        };

        // Update chunk document_ids to reference the parent document
        for chunk in &mut state.chunks {
            chunk.document_id = document.id;
        }

        debug!(
            doc_id = %document.id,
            title = %document.title,
            chunk_count = state.chunks.len(),
            "Index preparation complete"
        );

        Some(document)
    }
}

impl Default for IndexPrepareStep {
    fn default() -> Self {
        Self::new()
    }
}