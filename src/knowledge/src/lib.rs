//! Enterprise Knowledge Platform
//!
//! - Document pipeline (discover, parse, chunk, clean, dedup, normalize)
//! - Embedding (local ONNX, batch, incremental, versioning)
//! - Vector + FTS5 hybrid search
//! - Knowledge pack management
//! - Association graphs (proximity, topic, workspace)
//! - Citation management and cross-references
//! - Performance (caching, background indexing, cancellation)
//! - Validation and SDK

// Core types
pub mod doc;

// Data pipeline
pub mod import;
pub mod pipeline;
pub mod processing;

// Embedding
pub mod embedding;
pub mod embedding_pipeline;

// Storage (vector store + FTS5)
pub mod storage;

// Retrieval (search, hybrid, chunker)
pub mod retrieval;

// Knowledge pack management
pub mod pack;

// Knowledge providers (sources)
pub mod providers;

// Associations (graphs, proximity, topic, workspace)
pub mod association;

// Citation management
pub mod citation;

// Metadata models
pub mod metadata;

// Performance
pub mod performance;

// Validation
pub mod validate;

// SDK
pub mod sdk;
