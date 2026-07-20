//! Pipeline module — Document ingestion pipeline with chainable, extensible steps.
//!
//! Steps: discover → validate → dedup → incremental → parse → clean → normalize →
//! chunk → metadata_extract → version_detect → index_prepare

pub mod result;
pub mod steps;

use anyhow::Context;
use steps::chunk::ChunkStep;
use steps::clean::CleanStep;
use steps::dedup::DedupStep;
use steps::discover::DiscoverStep;
use steps::incremental::IncrementalStep;
use steps::index_prepare::IndexPrepareStep;
use steps::metadata_extract::MetadataExtractStep;
use steps::normalize::NormalizeStep;
use steps::parse::ParseStep;
use steps::validate::ValidateStep;
use steps::version_detect::VersionDetectStep;
use steps::Language;

use crate::doc::{KnowledgeChunk, KnowledgeDocument};
use crate::pipeline::result::{DiscoveredDoc, PipelineResult};
use crate::processing::Document;
pub use result::ChunkInfo;

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Configuration for the ingestion pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Maximum document size in bytes (default: 10 MB)
    pub max_size: u64,
    /// Chunk size in characters (default: 1000)
    pub chunk_size: usize,
    /// Chunk overlap in characters (default: 100)
    pub chunk_overlap: usize,
    /// Enable deduplication (default: true)
    pub enable_dedup: bool,
    /// Enable incremental updates (default: true)
    pub enable_incremental: bool,
    /// Supported file extensions
    pub supported_extensions: Vec<String>,
    /// Workspace ID for all documents
    pub workspace_id: uuid::Uuid,
    /// Author for all documents
    pub author: String,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024, // 10 MB
            chunk_size: 1000,
            chunk_overlap: 100,
            enable_dedup: true,
            enable_incremental: true,
            supported_extensions: vec![
                ".md".to_string(),
                ".markdown".to_string(),
                ".html".to_string(),
                ".htm".to_string(),
                ".txt".to_string(),
                ".yml".to_string(),
                ".yaml".to_string(),
                ".json".to_string(),
                ".xml".to_string(),
                ".pdf".to_string(),
                ".docx".to_string(),
            ],
            workspace_id: uuid::Uuid::new_v4(),
            author: "system".to_string(),
        }
    }
}

/// Represents the status of a document through the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Skipped,
    Failed(String),
}

/// A single document's pipeline execution state.
#[derive(Debug, Clone)]
pub struct DocPipelineState {
    pub discovered: Option<DiscoveredDoc>,
    pub parsed: Option<crate::processing::Document>,
    pub chunks: Vec<KnowledgeChunk>,
    pub document: Option<KnowledgeDocument>,
    pub step_results: std::collections::HashMap<String, StepStatus>,
}

impl DocPipelineState {
    fn new() -> Self {
        Self {
            discovered: None,
            parsed: None,
            chunks: Vec::new(),
            document: None,
            step_results: std::collections::HashMap::new(),
        }
    }
}

/// The ingestion pipeline orchestrator.
pub struct IngestionPipeline {
    config: PipelineConfig,
    /// Track indexed content hashes for incremental processing
    indexed_hashes: Arc<Mutex<HashSet<String>>>,
    /// Track indexed file paths and their last-modified timestamps
    indexed_mtimes: Arc<Mutex<std::collections::HashMap<String, std::time::SystemTime>>>,
}

impl IngestionPipeline {
    pub fn new(config: Option<PipelineConfig>) -> Self {
        let config = config.unwrap_or_default();
        info!(
            "IngestionPipeline created with config: max_size={}, chunk_size={}, chunk_overlap={}",
            config.max_size, config.chunk_size, config.chunk_overlap
        );
        Self {
            config,
            indexed_hashes: Arc::new(Mutex::new(HashSet::new())),
            indexed_mtimes: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Run the full pipeline on a set of file paths (glob or explicit).
    pub async fn run(&self, paths: Vec<&str>) -> anyhow::Result<PipelineResult> {
        info!(paths = ?paths, "Starting ingestion pipeline");

        let mut result = PipelineResult::new();

        // Step 1: Discover
        let discover = DiscoverStep::new(&self.config);
        let discovered = discover
            .run(&paths)
            .await
            .context("Document discovery failed")?;

        debug!(
            count = discovered.len(),
            "Discovered {} documents",
            discovered.len()
        );
        result.discovery_count = discovered.len();
        result.discovered_docs.clone_from(&discovered);

        if discovered.is_empty() {
            warn!("No documents discovered, pipeline complete");
            return Ok(result);
        }

        // Step 2: Validate
        let validate = ValidateStep::new(self.config.max_size, &self.config.supported_extensions);
        let validated: Vec<DiscoveredDoc> = {
            let mut valid_docs = Vec::new();
            for doc in &discovered {
                match validate.run(doc) {
                    Ok(_) => valid_docs.push(doc.clone()),
                    Err(e) => {
                        warn!(path = ?doc.path, error = %e, "Document validation failed, skipping");
                        result
                            .failed_docs
                            .push((doc.path.to_string_lossy().to_string(), e.to_string()));
                    }
                }
            }
            valid_docs
        };
        result.validated_count = validated.len();
        debug!(
            count = validated.len(),
            "Validated {} documents",
            validated.len()
        );

        // Step 3: Deduplication
        let dedup = DedupStep::new();
        let deduped: Vec<DiscoveredDoc> = {
            let mut hashes = Vec::new();
            for doc in &validated {
                match dedup.run(doc) {
                    Ok(hash) => {
                        hashes.push((doc.clone(), hash));
                    }
                    Err(e) => {
                        warn!(path = ?doc.path, error = %e, "Dedup check failed, skipping");
                        result
                            .failed_docs
                            .push((doc.path.to_string_lossy().to_string(), e.to_string()));
                    }
                }
            }

            let mut unique = Vec::new();
            let mut seen_hashes = HashSet::new();
            for (doc, hash) in hashes {
                if self.config.enable_dedup {
                    if seen_hashes.contains(&hash) {
                        info!(path = ?doc.path, "Duplicate detected, skipping");
                        result.duplicates_skipped += 1;
                        continue;
                    }
                    seen_hashes.insert(hash.clone());
                    // Update global index
                    let mut idx = self.indexed_hashes.lock().await;
                    idx.insert(hash);
                }
                unique.push(doc);
            }
            unique
        };
        result.unique_count = deduped.len();
        debug!(
            count = deduped.len(),
            "After dedup: {} documents",
            deduped.len()
        );

        // Step 4: Incremental check
        let incremental = IncrementalStep::new();
        let incremental_docs: Vec<DiscoveredDoc> = {
            let mut keep = Vec::new();
            let mtimes = self.indexed_mtimes.lock().await;

            for doc in &deduped {
                if !self.config.enable_incremental {
                    keep.push(doc.clone());
                    continue;
                }

                let needs_update = match incremental.check_incremental(doc, &mtimes) {
                    Ok(should_process) => should_process,
                    Err(_) => true, // default: process if check fails
                };

                if needs_update {
                    keep.push(doc.clone());
                } else {
                    info!(path = ?doc.path, "Document up to date, skipping");
                    result.skipped_count += 1;
                }
            }
            keep
        };
        result.skipped_count += deduped.len() - incremental_docs.len();
        debug!(
            count = incremental_docs.len(),
            "After incremental check: {} documents",
            incremental_docs.len()
        );

        // Steps 5-12: Process each document
        for doc in incremental_docs {
            let mut state = DocPipelineState::new();
            state.discovered = Some(doc.clone());

            // Step 5: Parse
            let parse_step = ParseStep::new();
            match parse_step.run(&doc, &self.config.author) {
                Ok(parsed) => {
                    state.parsed = Some(parsed.clone());
                    state
                        .step_results
                        .insert("parse".to_string(), StepStatus::Completed);
                    result.parsed_count += 1;

                    // Step 6: Clean
                    let clean_step = CleanStep::new();
                    let cleaned = match clean_step.run(parsed.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            error!(error = %e, "Cleaning failed");
                            state
                                .step_results
                                .insert("clean".to_string(), StepStatus::Failed(e.to_string()));
                            result.failed_docs.push((
                                doc.path.to_string_lossy().to_string(),
                                "cleaning failed".to_string(),
                            ));
                            continue;
                        }
                    };
                    state.parsed = Some(cleaned.clone());
                    state
                        .step_results
                        .insert("clean".to_string(), StepStatus::Completed);

                    // Step 7: Normalize
                    let normalize_step = NormalizeStep::new();
                    let normalized = match normalize_step.run(cleaned) {
                        Ok(n) => n,
                        Err(e) => {
                            error!(error = %e, "Normalization failed");
                            state
                                .step_results
                                .insert("normalize".to_string(), StepStatus::Failed(e.to_string()));
                            result.failed_docs.push((
                                doc.path.to_string_lossy().to_string(),
                                "normalization failed".to_string(),
                            ));
                            continue;
                        }
                    };
                    state.parsed = Some(normalized.clone());
                    state
                        .step_results
                        .insert("normalize".to_string(), StepStatus::Completed);

                    // Step 8: Language detection
                    let lang = Language::detect(&normalized.full_text);
                    debug!(path = ?doc.path, language = ?lang, "Language detected");
                    if let Some(ref mut p) = state.parsed {
                        // Convert steps::Language to processing::Language
                        p.detected_language = match lang {
                            Language::English => crate::processing::Language::En,
                            Language::Unknown => crate::processing::Language::None,
                        };
                    }
                    state
                        .step_results
                        .insert("language_detect".to_string(), StepStatus::Completed);

                    // Step 9: Chunk
                    let chunk_step =
                        ChunkStep::new(self.config.chunk_size, self.config.chunk_overlap);
                    match chunk_step.run(&normalized, doc.workspace_id) {
                        Ok(chunks) => {
                            state.chunks = chunks;
                            state
                                .step_results
                                .insert("chunk".to_string(), StepStatus::Completed);
                            result.chunked_count += 1;
                        }
                        Err(e) => {
                            error!(error = %e, "Chunk generation failed");
                            state
                                .step_results
                                .insert("chunk".to_string(), StepStatus::Failed(e.to_string()));
                            result.failed_docs.push((
                                doc.path.to_string_lossy().to_string(),
                                "chunking failed".to_string(),
                            ));
                            continue;
                        }
                    }

                    // Step 10: Metadata extraction
                    let meta_step = MetadataExtractStep::new();
                    if let Some(ref mut parsed) = state.parsed {
                        meta_step.run(parsed);
                    }
                    state
                        .step_results
                        .insert("metadata_extract".to_string(), StepStatus::Completed);

                    // Step 11: Version detection
                    let version_step = VersionDetectStep::new();
                    if let Some(ref mut parsed) = state.parsed {
                        version_step.run(parsed);
                    }
                    state
                        .step_results
                        .insert("version_detect".to_string(), StepStatus::Completed);

                    // Step 12: Index preparation
                    let idx_step = IndexPrepareStep::new();
                    let index_doc = match idx_step.run(&mut state, &doc) {
                        Some(d) => d,
                        None => {
                            state.step_results.insert(
                                "index_prepare".to_string(),
                                StepStatus::Failed("returned None".to_string()),
                            );
                            result.failed_docs.push((
                                "unknown".to_string(),
                                "index preparation failed".to_string(),
                            ));
                            continue;
                        }
                    };
                    state.document = Some(index_doc.clone());
                    state
                        .step_results
                        .insert("index_prepare".to_string(), StepStatus::Completed);

                    // Record in index
                    if let Some(ref doc_el) = state.document {
                        let mut mtimes = self.indexed_mtimes.lock().await;
                        mtimes.insert(
                            state
                                .discovered
                                .as_ref()
                                .unwrap()
                                .path
                                .to_string_lossy()
                                .to_string(),
                            doc_el.updated_at.into(),
                        );
                    }

                    result.processed_docs.push(state);
                }
                Err(e) => {
                    error!(path = ?doc.path, error = %e, "Parsing failed");
                    result
                        .failed_docs
                        .push((doc.path.to_string_lossy().to_string(), e.to_string()));
                    result.failed_count += 1;
                }
            }
        }

        info!(
            total = result.total_count,
            parsed = result.parsed_count,
            chunked = result.chunked_count,
            duplicates = result.duplicates_skipped,
            skipped = result.skipped_count,
            failed = result.failed_count,
            "Pipeline complete"
        );

        Ok(result)
    }

    /// Add a hash to the indexed set (for external tracking).
    pub async fn add_indexed_hash(&self, hash: String) {
        let mut idx = self.indexed_hashes.lock().await;
        idx.insert(hash);
    }

    /// Get the set of indexed hashes.
    pub async fn get_indexed_hashes(&self) -> HashSet<String> {
        let idx = self.indexed_hashes.lock().await;
        idx.clone()
    }
}
