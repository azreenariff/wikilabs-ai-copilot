//! Knowledge import pipeline.
//!
//! Provides two main import methods:
//! - `import_file`: Parse a file, chunk it, generate embeddings, and index.
//! - `import_text`: Chunk raw text, generate embeddings, and index.

use crate::doc::KnowledgeChunk as DataKnowledgeChunk;
use crate::doc::KnowledgeDocument;
use crate::embedding::EmbeddingPipeline;
use crate::embedding::EmbeddingPipelineConfig;
use crate::pipeline::PipelineConfig;
use crate::processing::{MarkdownParser, ParserProvider, TxtParser};
use crate::retrieval::chunker::{ChunkStrategy, Chunker};
use crate::storage::namespace::NamespaceManager;
use crate::storage::vector_store::VectorStore;
use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use uuid::Uuid;

/// Result of an import operation.
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub document_id: Uuid,
    pub chunks_imported: usize,
    pub chunks_skipped: usize,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// Knowledge import pipeline — orchestrates file/text ingestion,
/// chunking, embedding, and vector store indexing.
pub struct ImportPipeline {
    vector_store: Arc<Mutex<VectorStore>>,
    namespace_mgr: NamespaceManager,
    embedding_pipeline: EmbeddingPipeline,
    config: PipelineConfig,
}

impl ImportPipeline {
    pub fn new() -> Self {
        let config = PipelineConfig::default();
        let vector_store = Arc::new(Mutex::new(
            VectorStore::new(
                "knowledge_store.db",
                &config.workspace_id.to_string(),
                config.workspace_id,
                1,
            )
            .expect("Failed to initialize vector store"),
        ));
        // Open a separate connection for namespace management
        let namespace_conn = Connection::open("knowledge_store.db")
            .expect("Failed to open namespace manager connection");
        let namespace_mgr = NamespaceManager::new(namespace_conn);
        let embedding_config = EmbeddingPipelineConfig::default();
        let embedding_pipeline = EmbeddingPipeline::new(Some(embedding_config));

        Self {
            vector_store,
            namespace_mgr,
            embedding_pipeline,
            config,
        }
    }

    /// Import a file: parse, chunk, embed, and index.
    pub async fn import_file(&self, path: &str) -> Result<ImportResult> {
        let start = Utc::now();
        info!(path, "Starting file import");

        let mut errors = Vec::new();
        let mut chunks_imported = 0;
        let mut chunks_skipped = 0;

        // Step 1: Read file content
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Failed to read file: {}", e));
                let duration = Utc::now() - start;
                return Ok(ImportResult {
                    document_id: Uuid::new_v4(),
                    chunks_imported: 0,
                    chunks_skipped: 0,
                    errors,
                    duration_ms: duration.num_milliseconds() as u64,
                });
            }
        };

        // Step 2: Select parser based on extension
        let extension = path.rsplit('.').next().unwrap_or("").to_string();
        let parser = self.select_parser(&extension);
        let author = "importer".to_string();
        let document = parser.parse(&content, &author, path);

        info!(
            path,
            extension = %extension,
            elements = document.elements.len(),
            "File parsed successfully"
        );

        // Step 3: Create document record
        let document_id = Uuid::new_v4();
        let workspace_id = self.config.workspace_id;

        let _doc_record = KnowledgeDocument::new(
            document.title.clone(),
            path.to_string(),
            workspace_id,
            author.clone(),
        );

        // Step 4: Chunk the document
        let chunk_strategy = match extension.as_str() {
            "md" | "markdown" => ChunkStrategy::ByStructure {
                max_chunk_size: 512,
                min_chunk_size: 64,
                overlap: 32,
            },
            "txt" | "text" => ChunkStrategy::ByParagraph {
                max_chunk_size: 1000,
                overlap: 100,
            },
            _ => ChunkStrategy::BySize {
                max_chunk_size: 500,
                overlap: 50,
            },
        };

        let chunker = Chunker::new(chunk_strategy);
        let metadata = json!({
            "document_id": document_id.to_string(),
            "source": path,
            "extension": extension,
            "title": document.title,
            "author": author,
        });

        let chunks = match chunker.chunk(&document.full_text, metadata.clone()) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Chunking failed: {}", e));
                let duration = Utc::now() - start;
                return Ok(ImportResult {
                    document_id,
                    chunks_imported: 0,
                    chunks_skipped: 0,
                    errors,
                    duration_ms: duration.num_milliseconds() as u64,
                });
            }
        };

        info!(chunk_count = chunks.len(), "Document chunked successfully");

        // Step 5: Generate embeddings for each chunk
        // Create KnowledgeChunk instances for the embedding pipeline
        let knowledge_chunks: Vec<DataKnowledgeChunk> = chunks
            .iter()
            .map(|c| {
                let chunk_id = Uuid::parse_str(&c.id).unwrap_or_else(|_| Uuid::new_v4());
                let doc_id_str = c
                    .metadata
                    .get("document_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let doc_id = Uuid::parse_str(doc_id_str).unwrap_or_else(|_| Uuid::new_v4());
                DataKnowledgeChunk {
                    id: chunk_id,
                    document_id: doc_id,
                    content: c.text.clone(),
                    vector_id: String::new(),
                }
            })
            .collect();

        // Create refs for the embedding pipeline (expects &[&KnowledgeChunk])
        let chunk_refs: Vec<&DataKnowledgeChunk> = knowledge_chunks.iter().collect();

        // Embed chunks in batches
        match self.embedding_pipeline.embed_chunks(&chunk_refs).await {
            Ok(embeddings) => {
                debug!(embedding_count = embeddings.len(), "Embeddings generated");

                // Step 6: Index embeddings in vector store
                let store = self.vector_store.clone();
                let dimensions = embeddings[0].dimensions;
                let ns_result = self.namespace_mgr.get_or_create(
                    &document.title,
                    None,
                    &document.title,
                    dimensions,
                );

                let store_lock = store.lock().await;
                for (i, embedding) in embeddings.iter().enumerate() {
                    if i < chunks.len() {
                        let chunk = &chunks[i];
                        let vector_id = format!("{}_{}", document_id, i);

                        match store_lock
                            .insert_vector(
                                &vector_id,
                                &embedding.vector,
                                &chunk.text,
                                &document_id.to_string(),
                            )
                            .await
                        {
                            Ok(_) => {
                                chunks_imported += 1;
                                debug!(vector_id, "Chunk indexed");
                            }
                            Err(e) => {
                                chunks_skipped += 1;
                                errors.push(format!("Indexing failed for chunk {}: {}", i, e));
                            }
                        }
                    }
                }

                // Update namespace metadata
                if ns_result.is_ok() {
                    let _ = store_lock.update_metadata(1, chunks_imported, "local", dimensions).await;
                }
            }
            Err(e) => {
                chunks_skipped += chunks.len();
                errors.push(format!("Embedding generation failed: {}", e));
            }
        }

        let duration = Utc::now() - start;
        info!(
            document_id = %document_id,
            chunks_imported,
            chunks_skipped,
            duration_ms = duration.num_milliseconds(),
            "Import complete"
        );

        Ok(ImportResult {
            document_id,
            chunks_imported,
            chunks_skipped,
            errors,
            duration_ms: duration.num_milliseconds() as u64,
        })
    }

    /// Import raw text: chunk, embed, and index.
    pub async fn import_text(&self, title: &str, content: &str) -> Result<ImportResult> {
        let start = Utc::now();
        info!(title = %title, content_len = content.len(), "Starting text import");

        let mut errors = Vec::new();
        let mut chunks_imported = 0;
        let mut chunks_skipped = 0;

        // Step 1: Create a mock document for text
        let document = crate::processing::Document::new(
            content.to_string(),
            "text-import",
            format!("text://{}", title),
        );

        // Step 2: Create document record
        let document_id = Uuid::new_v4();
        let workspace_id = self.config.workspace_id;

        let _doc_record = KnowledgeDocument::new(
            title.to_string(),
            format!("text://{}", title),
            workspace_id,
            "text-import".to_string(),
        );

        // Step 3: Chunk the text
        let chunker = Chunker::by_paragraph(1000, 100);
        let metadata = json!({
            "document_id": document_id.to_string(),
            "source": format!("text://{}", title),
            "title": title,
            "author": "text-import",
        });

        let chunks = match chunker.chunk(&document.full_text, metadata.clone()) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Chunking failed: {}", e));
                let duration = Utc::now() - start;
                return Ok(ImportResult {
                    document_id,
                    chunks_imported: 0,
                    chunks_skipped: 0,
                    errors,
                    duration_ms: duration.num_milliseconds() as u64,
                });
            }
        };

        info!(chunk_count = chunks.len(), "Text chunked successfully");

        // Step 4: Generate embeddings
        let knowledge_chunks: Vec<DataKnowledgeChunk> = chunks
            .iter()
            .map(|c| {
                let chunk_id = Uuid::parse_str(&c.id).unwrap_or_else(|_| Uuid::new_v4());
                let doc_id_str = c
                    .metadata
                    .get("document_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let doc_id = Uuid::parse_str(doc_id_str).unwrap_or_else(|_| Uuid::new_v4());
                DataKnowledgeChunk {
                    id: chunk_id,
                    document_id: doc_id,
                    content: c.text.clone(),
                    vector_id: String::new(),
                }
            })
            .collect();

        let chunk_refs: Vec<&DataKnowledgeChunk> = knowledge_chunks.iter().collect();

        match self.embedding_pipeline.embed_chunks(&chunk_refs).await {
            Ok(embeddings) => {
                debug!(embedding_count = embeddings.len(), "Embeddings generated");

                let store = self.vector_store.clone();
                let dimensions = embeddings[0].dimensions;

                // Try to get or create namespace
                let ns_result = self
                    .namespace_mgr
                    .get_or_create(title, None, title, dimensions);

                let store_lock = store.lock().await;
                for (i, embedding) in embeddings.iter().enumerate() {
                    if i < chunks.len() {
                        let chunk = &chunks[i];
                        let vector_id = format!("{}_{}", document_id, i);

                        match store_lock
                            .insert_vector(
                                &vector_id,
                                &embedding.vector,
                                &chunk.text,
                                &document_id.to_string(),
                            )
                            .await
                        {
                            Ok(_) => {
                                chunks_imported += 1;
                            }
                            Err(e) => {
                                chunks_skipped += 1;
                                errors.push(format!("Indexing failed for chunk {}: {}", i, e));
                            }
                        }
                    }
                }

                // Update namespace metadata
                if ns_result.is_ok() {
                    let _ = store_lock.update_metadata(1, chunks_imported, "local", dimensions).await;
                }
            }
            Err(e) => {
                chunks_skipped += chunks.len();
                errors.push(format!("Embedding generation failed: {}", e));
            }
        }

        let duration = Utc::now() - start;
        info!(
            document_id = %document_id,
            chunks_imported,
            chunks_skipped,
            duration_ms = duration.num_milliseconds(),
            "Text import complete"
        );

        Ok(ImportResult {
            document_id,
            chunks_imported,
            chunks_skipped,
            errors,
            duration_ms: duration.num_milliseconds() as u64,
        })
    }

    /// Select a parser based on file extension.
    fn select_parser(&self, extension: &str) -> Box<dyn ParserProvider> {
        match extension {
            "md" | "markdown" => Box::new(MarkdownParser::new()),
            "html" | "htm" => Box::new(crate::processing::HtmlParser::new()),
            "txt" | "text" => Box::new(TxtParser::new()),
            "yaml" | "yml" => Box::new(crate::processing::YamlParser::new()),
            "json" => Box::new(crate::processing::JsonParser::new()),
            "xml" => Box::new(crate::processing::XmlParser::new()),
            _ => Box::new(TxtParser::new()), // Fallback to plain text
        }
    }
}

impl Default for ImportPipeline {
    fn default() -> Self {
        Self::new()
    }
}
