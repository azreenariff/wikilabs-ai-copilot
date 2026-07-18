# Document Pipeline Specification

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

The Document Pipeline is a chainable, extensible ingestion pipeline that processes documents from raw files into indexed chunks suitable for vector storage. It consists of 12 stages that transform, validate, and prepare documents for knowledge retrieval.

## Pipeline Architecture

```
Raw Document → [Discover] → [Validate] → [Dedup] → [Incremental] → [Parse]
    → [Clean] → [Normalize] → [Language] → [Chunk] → [Metadata] → [Version] → [Index] → Indexed Chunk
```

## Pipeline Trait

Each stage implements the `PipelineStep` trait:

```rust
#[async_trait]
pub trait PipelineStep: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, input: DocumentState) -> anyhow::Result<DocumentState>;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}
```

## Pipeline Configuration

```rust
pub struct PipelineConfig {
    pub max_file_size: usize,         // Maximum file size to process (bytes)
    pub chunk_size: usize,            // Target chunk size in characters
    pub chunk_overlap: usize,         // Overlap between chunks
    pub enable_dedup: bool,           // Enable duplicate detection
    pub enable_incremental: bool,     // Enable incremental updates
    pub enable_language_detection: bool,
    pub languages: Vec<String>,       // Accepted languages
    pub enable_version_detection: bool,
    pub providers: Vec<Box<dyn KnowledgeProvider>>,
    pub progress_callback: Option<Box<dyn ProgressReporter>>,
}
```

## Stage 1: Document Discovery

Finds documents in specified paths.

**Inputs:** Paths, glob patterns, file type filters  
**Outputs:** List of candidate documents with metadata (path, size, modified time, permissions)  
**Steps:**

1. Walk directory tree
2. Apply file extension filters
3. Skip hidden files and directories
4. Apply size filters
5. Collect file metadata
6. Return list of discoverable documents

```rust
pub struct DiscoverStep {
    pub paths: Vec<String>,
    pub extensions: Vec<String>,
    pub max_depth: usize,
    pub include_hidden: bool,
}
```

## Stage 2: Validation

Validates each document for processability.

**Inputs:** Discovered documents  
**Outputs:** Validated documents or rejection reasons  
**Steps:**

1. Check file exists and is readable
2. Validate file extension against supported formats
3. Check file size against max_file_size
4. Attempt to open file for reading
5. Detect file encoding
6. Verify file integrity (checksum)
7. Return validation result

```rust
pub struct ValidateStep {
    pub max_file_size: usize,
    pub supported_formats: Vec<String>,
    pub encoding: Encoding,
}

pub enum ValidationResult {
    Valid,
    Invalid { reason: String, code: ValidationCode },
    Skipped { reason: String },
}

pub enum ValidationCode {
    FileNotFound,
    UnsupportedFormat,
    FileTooLarge,
    EncodingError,
    ReadPermissionDenied,
    CorruptedFile,
}
```

## Stage 3: Duplicate Detection

Detects and filters duplicate documents.

**Inputs:** Validated documents  
**Outputs:** Unique documents + deduplication report  
**Steps:**

1. Compute content hash (SHA-256) for each document
2. Compare hashes against existing indexed documents
3. Mark exact duplicates for removal
4. Mark near-duplicates for human review (optional)
5. Generate deduplication report

```rust
pub struct DedupStep {
    pub existing_hashes: HashMap<String, Uuid>, // hash → doc_id
    pub near_dup_threshold: f64,    // Similarity threshold for near-duplicates
    pub enabled: bool,
}

pub struct DedupResult {
    pub exact_duplicates: Vec<DuplicateInfo>,
    pub near_duplicates: Vec<NearDuplicateInfo>,
    pub unique_count: usize,
    pub duplicate_count: usize,
}

pub struct DuplicateInfo {
    pub original_id: Uuid,
    pub duplicate_id: Uuid,
    pub hash: String,
}
```

## Stage 4: Incremental Updates

Checks for changes since last indexing.

**Inputs:** Documents + indexed document timestamps  
**Outputs:** Documents needing re-indexing  
**Steps:**

1. Compare file mtime vs indexed timestamp
2. Compare file size vs indexed size
3. Compare content hash vs stored hash
4. Flag changed documents for re-indexing
5. Flag new documents for initial indexing
6. Skip unchanged documents

```rust
pub struct IncrementalStep {
    pub indexed_metadata: HashMap<Uuid, IndexedMetadata>,
    pub enabled: bool,
}

pub struct IndexedMetadata {
    pub timestamp: DateTime<Utc>,
    pub size: usize,
    pub hash: String,
    pub embedding_version: String,
}

pub enum ChangeType {
    New,              // Not previously indexed
    Modified,         // Content changed
    Deleted,          // No longer exists
    Unchanged,        // Skip
}
```

## Stage 5: Parsing

Parses document content using the appropriate provider.

**Inputs:** Validated documents  
**Outputs:** Parsed documents with DocumentElement tree  
**Steps:**

1. Determine file format from extension
2. Select appropriate provider
3. Parse document content
4. Build DocumentElement tree
5. Extract basic metadata from parsed content
6. Return parsed document

```rust
pub struct ParseStep {
    pub providers: ProviderRegistry,
}

pub struct ParsedDocument {
    pub id: Uuid,
    pub title: String,
    pub source: String,
    pub provider: String,
    pub format: String,
    pub elements: Vec<DocumentElement>,
    pub raw_text: String,
    pub metadata: HashMap<String, String>,
}
```

## Stage 6: Cleaning

Cleans parsed content.

**Inputs:** Parsed documents  
**Outputs:** Cleaned documents  
**Steps:**

1. Strip excess whitespace
2. Normalize line endings
3. Remove control characters
4. Clean formatting artifacts
5. Normalize spacing in code blocks
6. Clean table formatting

```rust
pub struct CleanStep {
    pub strip_whitespace: bool,
    pub normalize_line_endings: bool,
    pub remove_control_chars: bool,
    pub clean_formatting_artifacts: bool,
}
```

## Stage 7: Normalization

Normalizes document content.

**Inputs:** Cleaned documents  
**Outputs:** Normalized documents  
**Steps:**

1. Ensure UTF-8 encoding
2. Normalize Unicode (NFC form)
3. Standardize date formats
4. Standardize number formats
5. Normalize heading markers
6. Consistent list formatting

```rust
pub struct NormalizeStep {
    pub encoding: Encoding,
    pub unicode_normalization: UnicodeForm,
}

pub enum UnicodeForm {
    NFC,
    NFD,
    NFKC,
    NFKD,
}
```

## Stage 8: Language Detection

Detects the language of document content.

**Inputs:** Normalized documents  
**Outputs:** Documents with language metadata  
**Steps:**

1. Extract first N characters of content
2. Run language detection algorithm
3. Assign language code (ISO 639-1)
4. Filter by accepted languages
5. Skip unsupported documents

```rust
pub struct LanguageDetectionStep {
    pub sample_size: usize,
    pub supported_languages: Vec<String>,
    pub confidence_threshold: f64,
}

pub struct LanguageResult {
    pub language: String,
    pub confidence: f64,
    pub supported: bool,
}
```

## Stage 9: Chunk Generation

Splits documents into chunks for embedding.

**Inputs:** Parsed documents  
**Outputs:** Document chunks with content and metadata  
**Steps:**

1. Traverse DocumentElement tree
2. Split content into chunks based on headings
3. Apply chunk size and overlap parameters
4. Preserve structural context in each chunk
5. Assign chunk IDs and parent document IDs

```rust
pub struct ChunkStep {
    pub chunk_size: usize,        // Characters per chunk
    pub chunk_overlap: usize,     // Overlap between chunks
    pub min_chunk_size: usize,    // Minimum chunk size
    pub preserve_headings: bool,  // Include heading context in chunk
    pub preserve_structure: bool, // Preserve DocumentElement structure
}

pub struct DocumentChunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub heading_context: Option<String>,
    pub section_path: String,
    pub word_count: usize,
}
```

## Stage 10: Metadata Extraction

Extracts structured metadata from parsed documents.

**Inputs:** Chunked documents  
**Outputs:** Documents with extracted metadata  
**Steps:**

1. Extract title from first heading or frontmatter
2. Extract author from metadata or file info
3. Extract publication date
4. Detect technology keywords
5. Extract tags from content
6. Identify security classification
7. Map to vendor/product
8. Store all in metadata map

```rust
pub struct MetadataExtractStep {
    pub extract_headings: bool,
    pub extract_tables: bool,
    pub extract_commands: bool,
    pub extract_code_blocks: bool,
    pub extract_references: bool,
    pub extract_tags: bool,
    pub extract_security_class: bool,
    pub tech_keywords: Vec<String>,
}

pub struct ExtractedMetadata {
    pub title: String,
    pub author: String,
    pub publication_date: Option<DateTime<Utc>>,
    pub technologies: Vec<String>,
    pub tags: Vec<String>,
    pub security_classification: String,
    pub vendor: Option<String>,
    pub product: Option<String>,
    pub language: String,
    pub heading_count: usize,
    pub table_count: usize,
    pub command_count: usize,
    pub code_block_count: usize,
}
```

## Stage 11: Version Detection

Detects document version information.

**Inputs:** Documents with metadata  
**Outputs:** Documents with version information  
**Steps:**

1. Check for version in metadata
2. Check for version in filename
3. Check for version in content (changelog, release notes)
4. Check git history for version info
5. Extract semantic version
6. Store version metadata

```rust
pub struct VersionDetectionStep {
    pub parse_semver: bool,
    pub check_git: bool,
    pub extract_from_content: bool,
}

pub struct VersionInfo {
    pub version: Option<String>,
    pub semantic_version: Option<SemVer>,
    pub is_prerelease: bool,
    pub source: VersionSource,
}

pub enum VersionSource {
    Metadata,
    Filename,
    Content,
    Git,
    Unknown,
}
```

## Stage 12: Index Preparation

Prepares chunks for vector indexing.

**Inputs:** Processed chunks with metadata  
**Outputs:** Index-ready data structures  
**Steps:**

1. Format metadata for SQLite storage
2. Prepare vector data for embedding pipeline
3. Build namespace mappings
4. Assign chunk IDs for vector store
5. Create index preparation report
6. Output ready for embedding pipeline

```rust
pub struct IndexPrepStep {
    pub namespace: String,
    pub workspace_id: Uuid,
    pub embedding_model: String,
}

pub struct IndexPreparationResult {
    pub chunks: Vec<IndexableChunk>,
    pub metadata: Vec<MetadataRecord>,
    pub namespace: String,
    pub total_chunks: usize,
    pub total_size: usize,
    pub errors: Vec<String>,
}

pub struct IndexableChunk {
    pub id: Uuid,
    pub content: String,
    pub metadata: MetadataRecord,
    pub heading_context: String,
    pub section_path: String,
}
```

## Pipeline Orchestration

The `Pipeline` struct orchestrates all stages:

```rust
pub struct Pipeline {
    pub steps: Vec<Box<dyn PipelineStep>>,
    pub config: PipelineConfig,
    pub state: DocumentState,
}

impl Pipeline {
    pub fn new(config: PipelineConfig) -> Self;
    pub fn add_step(&mut self, step: Box<dyn PipelineStep>);
    pub async fn execute(&self, documents: Vec<DocumentPath>) -> anyhow::Result<PipelineResult>;
    pub async fn execute_incremental(&self, documents: Vec<DocumentPath>, existing: Vec<Uuid>) -> anyhow::Result<PipelineResult>;
    pub fn cancel(&self);
}
```

## PipelineResult

```rust
pub struct PipelineResult {
    pub total_found: usize,
    pub validated: usize,
    pub duplicates_removed: usize,
    pub unchanged_skipped: usize,
    pub parsed: usize,
    pub chunks_generated: usize,
    pub errors: Vec<PipelineError>,
    pub warnings: Vec<String>,
    pub duration: Duration,
    pub progress: Vec<ProgressUpdate>,
}
```

## PipelineError

```rust
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Step {step} failed: {reason}")]
    StepFailed { step: String, reason: String },

    #[error("Provider not found for format: {format}")]
    ProviderNotFound { format: String },

    #[error("Chunk generation failed for document {doc_id}")]
    ChunkFailed { doc_id: String },

    #[error("Encoding conversion failed: {reason}")]
    EncodingFailed { reason: String },

    #[error("Memory limit exceeded: {limit} bytes")]
    MemoryLimitExceeded { limit: usize },

    #[error("Pipeline cancelled")]
    Cancelled,
}
```

## Extensibility

New stages can be added by implementing `PipelineStep`:

```rust
pub struct CustomStep { ... }

#[async_trait]
impl PipelineStep for CustomStep {
    fn name(&self) -> &str { "custom_step" }

    async fn execute(&self, input: DocumentState) -> anyhow::Result<DocumentState> {
        // Custom logic
        Ok(output)
    }

    fn is_enabled(&self) -> bool { true }
    fn set_enabled(&mut self, _enabled: bool) {}
}
```

## Progress Reporting

The pipeline reports progress through `ProgressReporter`:

```rust
pub trait ProgressReporter: Send + Sync {
    fn on_start(&self, total: usize);
    fn on_progress(&self, current: usize, stage: &str, doc_id: Uuid);
    fn on_complete(&self, result: PipelineResult);
    fn on_error(&self, error: PipelineError);
}
```

## Error Handling

- Individual stage failures are caught and logged
- Pipeline continues processing remaining documents
- Errors are collected in PipelineResult
- Failed documents do not stop the entire pipeline
- Partial results can be indexed (with error report)

## Performance

- Stages are async to avoid blocking
- Large files are processed in streams
- Memory usage is bounded by config
- Progress is reported at each stage
- Cancellation is supported at any stage