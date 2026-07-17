//! Knowledge system — vector + keyword search.
//!
//! - SQLite VSS (vector search, dim=384)
//! - SQLite FTS5 (full-text search)
//! - Local embedding: all-MiniLM-L6-v2 (ONNX)
//! - Hybrid search: 70% vector + 30% FTS5
//! - Knowledge import pipeline (markdown, PDF, text)
//! - Deduplication & quality scoring

pub mod import;
pub mod search;
pub mod embedding;
pub mod dedup;
pub mod quality;
pub mod doc;