//! Knowledge document types.

#[derive(Debug)]
pub struct KnowledgeDocument {
    pub id: uuid::Uuid,
    pub title: String,
    pub source: String,
    pub workspace_id: uuid::Uuid,
    pub author: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct KnowledgeChunk {
    pub id: uuid::Uuid,
    pub document_id: uuid::Uuid,
    pub content: String,
    pub vector_id: String,
}