//! Knowledge document and chunk types with metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A knowledge document (manual, KB article, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeDocument {
    /// Unique document ID.
    pub id: Uuid,
    /// Document title.
    pub title: String,
    /// Source of the document (file path, URL, etc.).
    pub source: String,
    /// Workspace this document belongs to.
    pub workspace_id: Uuid,
    /// Document author.
    pub author: String,
    /// When the document was first indexed.
    pub created_at: DateTime<Utc>,
    /// When the document was last updated.
    pub updated_at: DateTime<Utc>,
}

/// A chunk of text from a knowledge document, suitable for embedding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeChunk {
    /// Unique chunk ID.
    pub id: Uuid,
    /// Parent document ID.
    pub document_id: Uuid,
    /// The chunk's text content.
    pub content: String,
    /// Vector database ID for this chunk.
    pub vector_id: String,
}

impl KnowledgeDocument {
    /// Create a new document with current timestamps.
    pub fn new(title: impl Into<String>, source: impl Into<String>, workspace_id: Uuid, author: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            source: source.into(),
            workspace_id,
            author: author.into(),
            created_at: now,
            updated_at: now,
        }
    }
}