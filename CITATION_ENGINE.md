# Citation Engine Specification

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

The Citation & Provenance Engine ensures every retrieval result retains full provenance — traceable from the retrieved chunk back to its original source. This enables engineers to verify results, navigate to original documents, and maintain audit trails.

## Core Principle

**Every retrieval result MUST include provenance.** There are no exceptions. The Knowledge Platform never returns content without complete source attribution.

## Citation Structure

```rust
pub struct Citation {
    /// Unique citation ID
    pub id: Uuid,

    /// Result this citation belongs to
    pub result_id: Uuid,

    /// Knowledge Pack that contains the source
    pub pack_name: String,

    /// Knowledge Pack display name
    pub pack_display_name: String,

    /// Original document title
    pub document_title: String,

    /// Document ID in the knowledge platform
    pub document_id: Uuid,

    /// Document source (file path, URL, etc.)
    pub source: String,

    /// Section heading where the chunk was found
    pub section: Option<String>,

    /// Heading context (nested heading hierarchy)
    pub heading_context: Option<Vec<String>>,

    /// Chunk index within the document
    pub chunk_index: Option<usize>,

    /// Total chunks in the document
    pub total_chunks: Option<usize>,

    /// Document version
    pub document_version: Option<String>,

    /// Document author
    pub author: Option<String>,

    /// Publication date
    pub publication_date: Option<DateTime<Utc>>,

    /// When the document was indexed
    pub index_timestamp: DateTime<Utc>,

    /// Confidence score of the retrieval
    pub confidence: f32,

    /// Retrieval score (vector similarity + metadata score)
    pub retrieval_score: f32,

    /// Original source (file path, URL, API endpoint)
    pub original_source: String,

    /// Provider that supplied the document
    pub provider_name: String,

    /// Embedding model version used
    pub embedding_version: String,

    /// Workspace that enabled this pack
    pub workspace_id: Uuid,

    /// Engineering context at time of retrieval
    pub engineering_context: Option<String>,

    /// Navigation URL/path to original document
    pub navigation_url: Option<String>,

    /// Navigation path within document (anchor, line number)
    pub navigation_path: Option<String>,
}
```

## Result with Citation

Every search result includes its citation:

```rust
pub struct CitationResult {
    /// The retrieved content
    pub content: String,

    /// Title of the result
    pub title: String,

    /// Similarity/search score
    pub score: f32,

    /// Ranking position
    pub rank: usize,

    /// Full provenance/citation
    pub citation: Citation,

    /// Section path within document
    pub section_path: String,

    /// Context metadata
    pub metadata: HashMap<String, String>,

    /// Related results (optional)
    pub related: Option<Vec<Uuid>>,
}
```

## Citation Generation

The citation is generated during retrieval, combining:

1. **Vector search result** — score, chunk ID
2. **Metadata lookup** — pack info, document info, author, version
3. **Provider info** — source, provider name
4. **Index timestamp** — when the document was indexed
5. **Engineering context** — workspace, technology, workflow

```rust
pub struct CitationEngine {
    pub metadata_store: MetadataStore,
    pub pack_repository: PackRepository,
    pub index_history: IndexHistoryStore,
    pub config: CitationConfig,
}

pub struct CitationConfig {
    pub include_source_path: bool,
    pub include_author: bool,
    pub include_version: bool,
    pub include_publication_date: bool,
    pub include_index_timestamp: bool,
    pub include_navigation: bool,
    pub include_engineering_context: bool,
    pub confidence_weight: f32,
}

impl CitationEngine {
    pub fn generate(&self, search_result: &SearchResult) -> Citation;
    pub fn generate_batch(&self, results: &[SearchResult]) -> Vec<Citation>;
    pub fn enrich(&self, citation: Citation) -> Citation;
    pub fn format_navigator(&self, citation: &Citation) -> String;
}
```

## Provenance Chain

The provenance chain tracks the complete lifecycle of each chunk:

```
Original Document (source)
  ↓
Provider (Filesystem, PDF, Markdown, etc.)
  ↓
Ingestion Pipeline (validation, parsing, chunking)
  ↓
Metadata Extraction (title, author, tags, technology)
  ↓
Embedding Generation (model, version, dimensions)
  ↓
Vector Storage (namespace, workspace, index)
  ↓
Retrieval (search, score, ranking)
  ↓
Citation (full provenance)
```

Each step is logged in the index history:

```rust
pub struct IndexHistory {
    pub id: Uuid,
    pub namespace: String,
    pub pack_name: String,
    pub workspace_id: Uuid,
    pub operation: IndexOperation,
    pub document_count: usize,
    pub chunk_count: usize,
    pub embedding_version: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: IndexStatus,
    pub error_message: Option<String>,
    pub duration_ms: Option<usize>,
    pub indexer_version: String,
    pub provider_used: String,
    pub config_snapshot: String,  // JSON
}

pub enum IndexOperation {
    InitialIndex,
    IncrementalUpdate,
    Reindex,
    Delete,
    Migration,
    Import,
    Export,
}

pub enum IndexStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}
```

## Navigation Support

The citation engine generates navigation paths for UI consumption:

```rust
pub struct NavigationInfo {
    pub type_: NavigationTarget,
    pub path: String,
    pub anchor: Option<String>,
    pub line_number: Option<usize>,
    pub display_text: String,
}

pub enum NavigationTarget {
    /// Navigate to local file
    File { path: String },

    /// Navigate to URL
    Url { url: String },

    /// Navigate within document
    Anchor { document_id: Uuid, anchor: String },

    /// Navigate to specific section
    Section { document_id: Uuid, section: String },

    /// Navigate to line number
    Line { document_id: Uuid, line_number: usize },

    /// Internal navigation (not external)
    Internal,
}

impl CitationEngine {
    pub fn generate_navigation(&self, citation: &Citation) -> Option<NavigationInfo>;
    pub fn format_display(&self, citation: &Citation) -> String;
    pub fn format_markdown(&self, citation: &Citation) -> String;
}
```

## Confidence Calculation

Confidence score combines multiple factors:

```rust
pub struct ConfidenceCalculator {
    pub vector_weight: f32,
    pub metadata_weight: f32,
    pub recency_weight: f32,
    pub pack_priority_weight: f32,
    pub quality_weight: f32,
}

impl ConfidenceCalculator {
    pub fn calculate(&self, result: &SearchResult, metadata: &MetadataEntry) -> f32 {
        let vector_score = self.vector_score(result);
        let metadata_score = self.metadata_score(metadata);
        let recency_score = self.recency_score(metadata);
        let priority_score = self.pack_priority_score(metadata);
        let quality_score = self.quality_score(metadata);

        let mut confidence = 0.0;
        confidence += vector_score * self.vector_weight;
        confidence += metadata_score * self.metadata_weight;
        confidence += recency_score * self.recency_weight;
        confidence += priority_score * self.pack_priority_weight;
        confidence += quality_score * self.quality_weight;

        confidence.min(1.0).max(0.0)
    }

    fn vector_score(&self, result: &SearchResult) -> f32 {
        result.score
    }

    fn metadata_score(&self, metadata: &MetadataEntry) -> f32 {
        match metadata.quality_score {
            Some(q) => q,
            None => 0.5,
        }
    }

    fn recency_score(&self, metadata: &MetadataEntry) -> f32 {
        let age = Utc::now() - metadata.last_indexed;
        if age < Duration::days(30) { 1.0 }
        else if age < Duration::days(90) { 0.8 }
        else if age < Duration::days(180) { 0.6 }
        else { 0.4 }
    }

    fn pack_priority_score(&self, metadata: &MetadataEntry) -> f32 {
        // Higher priority packs get slightly higher confidence
        let priority = metadata.pack_name.as_str();
        // Priority is a value from 0-1000
        0.5 + (priority as f32 / 2000.0)
    }

    fn quality_score(&self, metadata: &MetadataEntry) -> f32 {
        metadata.quality_score.unwrap_or(0.5)
    }
}
```

## Citation Formatting

Different formats for different UI needs:

```rust
impl CitationEngine {
    /// Human-readable plain text citation
    pub fn format_display(&self, citation: &Citation) -> String {
        format!(
            "{}\n{}\nPack: {} | Version: {} | Author: {}\nSource: {}\nLast Indexed: {}",
            citation.pack_display_name,
            citation.document_title,
            citation.pack_name,
            citation.document_version.as_deref().unwrap_or("N/A"),
            citation.author.as_deref().unwrap_or("N/A"),
            citation.source,
            citation.index_timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        )
    }

    /// Markdown formatted citation (for documentation)
    pub fn format_markdown(&self, citation: &Citation) -> String {
        let section = citation.section.as_deref().map(|s| format!(" - {}", s)).unwrap_or_default();
        let author = citation.author.as_deref().unwrap_or("Unknown");
        let version = citation.document_version.as_deref().unwrap_or("N/A");

        format!(
            "> **{}**{} ({})\n> — {} | v{} | [{}]({})",
            citation.pack_display_name,
            section,
            citation.document_title,
            author,
            version,
            citation.source,
            citation.navigation_url.as_deref().unwrap_or("#")
        )
    }

    /// JSON citation (for API response)
    pub fn format_json(&self, citation: &Citation) -> String {
        serde_json::to_string_pretty(citation).unwrap_or_default()
    }

    /// Navigation URL for UI (clickable link)
    pub fn format_navigator(&self, citation: &Citation) -> String {
        match self.generate_navigation(citation) {
            Some(nav) => format!("[{}]({})", nav.display_text, nav.path),
            None => citation.source.clone(),
        }
    }
}
```

## Citation Validation

The citation engine validates citations before returning them:

```rust
pub struct CitationValidator;

impl CitationValidator {
    pub fn validate(&self, citation: &Citation) -> anyhow::Result<()> {
        // Required fields
        if citation.pack_name.is_empty() {
            return Err(anyhow::anyhow!("Citation missing pack_name"));
        }
        if citation.document_title.is_empty() {
            return Err(anyhow::anyhow!("Citation missing document_title"));
        }
        if citation.source.is_empty() {
            return Err(anyhow::anyhow!("Citation missing source"));
        }
        if citation.original_source.is_empty() {
            return Err(anyhow::anyhow!("Citation missing original_source"));
        }
        if citation.provider_name.is_empty() {
            return Err(anyhow::anyhow!("Citation missing provider_name"));
        }
        if citation.embedding_version.is_empty() {
            return Err(anyhow::anyhow!("Citation missing embedding_version"));
        }
        if citation.confidence < 0.0 || citation.confidence > 1.0 {
            return Err(anyhow::anyhow!("Citation confidence out of range: {}", citation.confidence));
        }
        if citation.index_timestamp > Utc::now() {
            return Err(anyhow::anyhow!("Citation index_timestamp in future"));
        }
        Ok(())
    }

    pub fn validate_batch(&self, citations: &[Citation]) -> anyhow::Result<()> {
        for citation in citations {
            self.validate(citation)?;
        }
        Ok(())
    }
}
```

## Provenance Report

Generate a full provenance report for audit purposes:

```rust
pub struct ProvenanceReport {
    pub report_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub total_results: usize,
    pub results: Vec<ProvenanceResult>,
    pub summary: ProvenanceSummary,
}

pub struct ProvenanceResult {
    pub result_id: Uuid,
    pub citation: Citation,
    pub index_history: Vec<IndexHistory>,
    pub processing_steps: Vec<ProcessingStep>,
    pub validation_status: ValidationStatus,
}

pub struct ProcessingStep {
    pub step_name: String,
    pub timestamp: DateTime<Utc>,
    pub status: StepStatus,
    pub duration_ms: Option<usize>,
    pub details: HashMap<String, String>,
}

pub struct ProvenanceSummary {
    pub total_documents: usize,
    pub total_packs: usize,
    pub total_providers: usize,
    pub avg_confidence: f32,
    pub min_confidence: f32,
    pub max_confidence: f32,
    pub oldest_document: Option<DateTime<Utc>>,
    pub newest_document: Option<DateTime<Utc>>,
    pub most_recent_index: Option<DateTime<Utc>>,
}
```

## Audit Trail

All citation generation is logged for audit:

```rust
pub struct CitationAuditLog {
    pub log_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub workspace_id: Uuid,
    pub user_id: Option<String>,
    pub query_text: String,
    pub result_count: usize,
    pub pack_names: Vec<String>,
    pub technologies: Vec<String>,
    pub avg_confidence: f32,
    pub duration_ms: usize,
    pub citation_ids: Vec<Uuid>,
}
```

## Constraints

### What Citations Include

- ✅ Knowledge Pack name and display name
- ✅ Document title and ID
- ✅ Document source and original source
- ✅ Section and heading context
- ✅ Document version
- ✅ Author (when available)
- ✅ Publication date
- ✅ Index timestamp
- ✅ Confidence score
- ✅ Retrieval score
- ✅ Provider name
- ✅ Embedding version
- ✅ Workspace ID
- ✅ Navigation URL/path (when available)

### What Citations Do NOT Include

- ❌ AI-generated recommendations
- ❌ AI-generated reasoning
- ❌ AI-generated advice
- ❌ MCP execution results
- ❌ Automation steps
- ❌ Future knowledge graph data (graph not implemented)

## Security

- Provenance data is never modified after generation
- Citation IDs are immutable
- Index timestamps cannot be falsified
- All provenance data is append-only
- Audit logs are tamper-evident

## Performance

- Citations are generated lazily (only when needed)
- Metadata lookups are batched
- Index history is cached
- Navigation URLs are pre-computed for common sources
- Provenance reports are paginated