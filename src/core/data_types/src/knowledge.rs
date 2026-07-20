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

/// Category of a graph node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeCategory {
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "documentation")]
    Documentation,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "skill")]
    Skill,
    #[serde(rename = "sop")]
    Sop,
    #[serde(rename = "technology")]
    Technology,
    #[serde(rename = "workflow")]
    Workflow,

    Topic,
    Document,
    Entity,
    Concept,
    Relationship,
}

/// A node in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    /// Unique node ID.
    pub id: Uuid,
    /// Title or name.
    pub title: String,
    /// Label or name.
    pub label: String,
    /// Description.
    pub description: String,
    /// Category/type of node.
    pub category: NodeCategory,
    /// Technologies associated.
    pub technologies: Vec<String>,
    /// Vendor/organization.
    pub vendor: String,
    /// Properties/metadata.
    pub properties: serde_json::Value,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last updated.
    pub updated_at: DateTime<Utc>,
    /// Pack name if applicable.
    pub pack_name: String,
    /// Optional vendor metadata.
    pub vendor_metadata: Option<serde_json::Value>,
}

/// A relationship between two graph nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphRelationship {
    /// Unique relationship ID.
    pub id: Uuid,
    /// Source node ID.
    pub from_node: Uuid,
    /// Target node ID.
    pub to_node: Uuid,
    /// Relationship type (e.g., "references", "related_to").
    pub relation_type: String,
    /// Weight/score of the relationship.
    pub weight: f32,
}

/// Document from a knowledge provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderDocument {
    /// Unique document ID.
    pub id: Uuid,
    /// Title.
    pub title: String,
    /// Source/URL.
    pub source: String,
    /// File extension.
    pub extension: String,
    /// Raw content.
    pub content: String,
    /// File size in bytes.
    pub size_bytes: usize,
    /// MIME type.
    pub mime_type: String,
    /// When modified.
    pub modified_at: DateTime<Utc>,
    /// Author/source name.
    pub author: String,
}

impl KnowledgeDocument {
    /// Create a new document with current timestamps.
    pub fn new(
        title: impl Into<String>,
        source: impl Into<String>,
        workspace_id: Uuid,
        author: impl Into<String>,
    ) -> Self {
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
