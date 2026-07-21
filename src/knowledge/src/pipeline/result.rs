//! Pipeline result — tracks all intermediate data through the pipeline.

use crate::doc::KnowledgeDocument;
pub use crate::pipeline::steps::discover::DiscoveredDoc;
use crate::pipeline::DocPipelineState;
use crate::processing::DocumentElement;
use chrono::{DateTime, Utc};

/// Result from a single pipeline run.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// Total documents encountered
    pub total_count: usize,
    /// Documents that passed discovery
    pub discovery_count: usize,
    /// Documents that passed validation
    pub validated_count: usize,
    /// Documents that were unique (not duplicates)
    pub unique_count: usize,
    /// Documents successfully parsed
    pub parsed_count: usize,
    /// Documents successfully chunked
    pub chunked_count: usize,
    /// Duplicates skipped
    pub duplicates_skipped: usize,
    /// Skipped due to incremental check
    pub skipped_count: usize,
    /// Documents that failed processing
    pub failed_count: usize,
    /// List of discovered documents
    pub discovered_docs: Vec<DiscoveredDoc>,
    /// List of fully processed documents (state after all steps)
    pub processed_docs: Vec<DocPipelineState>,
    /// Documents that failed (path, error)
    pub failed_docs: Vec<(String, String)>,
    /// Processing timestamps
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

impl PipelineResult {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            total_count: 0,
            discovery_count: 0,
            validated_count: 0,
            unique_count: 0,
            parsed_count: 0,
            chunked_count: 0,
            duplicates_skipped: 0,
            skipped_count: 0,
            failed_count: 0,
            discovered_docs: Vec::new(),
            processed_docs: Vec::new(),
            failed_docs: Vec::new(),
            started_at: now,
            finished_at: now,
        }
    }

    /// Get all successfully processed documents.
    pub fn documents(&self) -> Vec<KnowledgeDocument> {
        self.processed_docs
            .iter()
            .filter_map(|s| s.document.clone())
            .collect()
    }

    /// Get all chunks from all processed documents.
    pub fn all_chunks(&self) -> Vec<super::result::ChunkInfo> {
        self.processed_docs
            .iter()
            .flat_map(|s| {
                s.chunks
                    .iter()
                    .map(|c| {
                        let elements: Vec<&DocumentElement> = if let Some(ref p) = s.parsed {
                            p.elements.iter().collect()
                        } else {
                            Vec::new()
                        };
                        super::result::ChunkInfo {
                            chunk_id: c.id,
                            document_id: c.document_id,
                            content: c.content.clone(),
                            vector_id: c.vector_id.clone(),
                            element_types: elements.iter().map(|e| format!("{:?}", e)).collect(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Get summary statistics as a human-readable string.
    pub fn summary(&self) -> String {
        format!(
            "Pipeline: discovered={} validated={} unique={} parsed={} chunked={} duplicates={} skipped={} failed={}",
            self.discovery_count, self.validated_count, self.unique_count,
            self.parsed_count, self.chunked_count, self.duplicates_skipped,
            self.skipped_count, self.failed_count
        )
    }
}

/// Information about a processed chunk.
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub chunk_id: uuid::Uuid,
    pub document_id: uuid::Uuid,
    pub content: String,
    pub vector_id: String,
    pub element_types: Vec<String>,
}
