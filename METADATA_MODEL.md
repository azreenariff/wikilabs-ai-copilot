# Metadata Model Specification

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

The Metadata Model stores structured metadata about knowledge documents. It is designed from the ground up to support a future Knowledge Graph without requiring any redesign.

## Core Principles

1. **Graph-Ready** — metadata schema supports node and edge relationships from day one
2. **Extensible** — new metadata fields can be added without migration
3. **Queryable** — indexed for fast retrieval and filtering
4. **Versioned** — track metadata changes over time
5. **Rich** — capture all relevant document and pack metadata

## Metadata Entry

```rust
pub struct MetadataEntry {
    /// Unique metadata entry ID
    pub id: Uuid,

    /// Parent document ID
    pub document_id: Uuid,

    /// Parent chunk ID (for chunk-level metadata)
    pub chunk_id: Uuid,

    // ─── Basic Document Metadata ───

    /// Document title
    pub title: String,

    /// Document source (file path, URL, etc.)
    pub source: String,

    /// Knowledge Pack name
    pub pack_name: String,

    /// Knowledge Pack display name
    pub pack_display_name: String,

    // ─── Vendor & Product ───

    /// Vendor name
    pub vendor: Option<String>,

    /// Vendor URL
    pub vendor_url: Option<String>,

    /// Product name
    pub product: Option<String>,

    /// Product version
    pub version: Option<String>,

    // ─── Technology ───

    /// Primary technology
    pub technology: Option<String>,

    /// Technology domains (secondary)
    pub technologies: Vec<String>,

    /// Technology categories
    pub categories: Vec<String>,

    // ─── Author & Publication ───

    /// Document author
    pub author: Option<String>,

    /// Publication date
    pub publication_date: Option<DateTime<Utc>>,

    /// Last modified date
    pub last_modified: Option<DateTime<Utc>>,

    // ─── Classification ───

    /// Security classification level
    pub security_classification: Option<String>,

    /// Customer scope
    pub customer_scope: Option<String>,

    /// Internal notes
    pub internal_notes: Option<String>,

    // ─── Tags & Classification ───

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Document type
    pub document_type: Option<String>,

    /// Content language
    pub language: String,

    // ─── Embedding Metadata ───

    /// Embedding model version used
    pub embedding_version: String,

    /// Embedding provider
    pub embedding_provider: String,

    /// Embedding dimension
    pub embedding_dimension: usize,

    /// Embedding confidence score
    pub embedding_confidence: Option<f32>,

    // ─── Graph-Ready Fields ───

    /// Node type for future knowledge graph
    pub node_type: NodeType,

    /// Graph node ID (for future graph)
    pub node_id: Option<String>,

    /// Source node ID (for future graph edges)
    pub source_node_id: Option<String>,

    /// Target node ID (for future graph edges)
    pub target_node_id: Option<String>,

    /// Relationship type (for future graph edges)
    pub relationship_type: Option<String>,

    /// Edge properties as JSON
    pub edge_properties: Option<String>,

    // ─── Processing Metadata ───

    /// When the metadata was last indexed
    pub last_indexed: DateTime<Utc>,

    /// Indexing provider used
    pub indexing_provider: String,

    /// Index processing status
    pub processing_status: ProcessingStatus,

    /// Quality score of the document
    pub quality_score: Option<f32>,

    /// Validation status
    pub validation_status: ValidationStatus,

    // ─── Counts & Statistics ───

    /// Word count
    pub word_count: usize,

    /// Character count
    pub char_count: usize,

    /// Number of chunks
    pub chunk_count: usize,

    /// Number of sections
    pub section_count: usize,

    /// Number of headings
    pub heading_count: usize,

    /// Number of tables
    pub table_count: usize,

    /// Number of code blocks
    pub code_block_count: usize,

    /// Number of commands
    pub command_count: usize,

    /// Number of references
    pub reference_count: usize,
}
```

## Node Types (Graph-Ready)

```rust
pub enum NodeType {
    /// Technology node (e.g., OpenShift, Kubernetes, Linux)
    Technology,

    /// Workflow node (e.g., Deploy, Migrate, Troubleshoot)
    Workflow,

    /// Command node (e.g., oc apply, kubectl get)
    Command,

    /// Documentation node (manuals, guides, references)
    Documentation,

    /// Vendor knowledge base node
    VendorKB,

    /// Standard Operating Procedure node
    SOP,

    /// Best Practice node
    BestPractice,

    /// Skill node (automatable skill)
    Skill,

    /// Error node (error messages, patterns)
    Error,

    /// Troubleshooting Guide node
    TroubleshootingGuide,

    /// Runbook node
    Runbook,

    /// Playbook node
    Playbook,

    /// Custom node type
    Custom(String),
}
```

## Relationship Types (Graph-Ready)

```rust
pub enum RelationshipType {
    /// Technology is related to a workflow
    TechnologyToWorkflow,

    /// Technology is related to a command
    TechnologyToCommand,

    /// Technology is related to documentation
    TechnologyToDocumentation,

    /// Technology is related to vendor KB
    TechnologyToVendorKB,

    /// Workflow is related to SOP
    WorkflowToSOP,

    /// Workflow is related to a skill
    WorkflowToSkill,

    /// Command is related to a troubleshooting guide
    CommandToTroubleshootingGuide,

    /// Error is related to vendor KB
    ErrorToVendorKB,

    /// Error is related to a workflow
    ErrorToWorkflow,

    /// Error is related to a best practice
    ErrorToBestPractice,

    /// Custom relationship type
    Custom(String),
}
```

## Processing Status

```rust
pub enum ProcessingStatus {
    /// Waiting to be indexed
    Pending,

    /// Currently being indexed
    Indexing,

    /// Indexing complete
    Indexed,

    /// Indexing failed
    Failed(String),

    /// Indexing skipped
    Skipped(String),

    /// Re-indexing needed
    ReindexNeeded,
}
```

## Validation Status

```rust
pub enum ValidationStatus {
    /// Not yet validated
    NotValidated,

    /// Validation passed
    Valid,

    /// Validation failed
    Invalid(String),

    /// Validation warning
    Warning(String),
}
```

## SQLite Schema

```sql
CREATE TABLE IF NOT EXISTS metadata_entries (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    chunk_id TEXT,
    title TEXT NOT NULL,
    source TEXT NOT NULL,
    pack_name TEXT NOT NULL,
    pack_display_name TEXT,
    vendor TEXT,
    vendor_url TEXT,
    product TEXT,
    version TEXT,
    technology TEXT,
    technologies TEXT,            -- JSON array
    categories TEXT,              -- JSON array
    author TEXT,
    publication_date TEXT,
    last_modified TEXT,
    security_classification TEXT,
    customer_scope TEXT,
    internal_notes TEXT,
    tags TEXT,                    -- JSON array
    document_type TEXT,
    language TEXT NOT NULL DEFAULT 'en',
    embedding_version TEXT NOT NULL,
    embedding_provider TEXT NOT NULL,
    embedding_dimension INTEGER NOT NULL DEFAULT 384,
    embedding_confidence REAL,
    node_type TEXT NOT NULL DEFAULT 'documentation',
    node_id TEXT,
    source_node_id TEXT,
    target_node_id TEXT,
    relationship_type TEXT,
    edge_properties TEXT,         -- JSON object
    last_indexed TEXT NOT NULL,
    indexing_provider TEXT NOT NULL,
    processing_status TEXT NOT NULL DEFAULT 'pending',
    quality_score REAL,
    validation_status TEXT NOT NULL DEFAULT 'not_validated',
    word_count INTEGER DEFAULT 0,
    char_count INTEGER DEFAULT 0,
    chunk_count INTEGER DEFAULT 0,
    section_count INTEGER DEFAULT 0,
    heading_count INTEGER DEFAULT 0,
    table_count INTEGER DEFAULT 0,
    code_block_count INTEGER DEFAULT 0,
    command_count INTEGER DEFAULT 0,
    reference_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for fast querying
CREATE INDEX IF NOT EXISTS idx_metadata_pack_name ON metadata_entries(pack_name);
CREATE INDEX IF NOT EXISTS idx_metadata_technology ON metadata_entries(technology);
CREATE INDEX IF NOT EXISTS idx_metadata_vendor ON metadata_entries(vendor);
CREATE INDEX IF NOT EXISTS idx_metadata_tags ON metadata_entries(tags);
CREATE INDEX IF NOT EXISTS idx_metadata_node_type ON metadata_entries(node_type);
CREATE INDEX IF NOT EXISTS idx_metadata_relationship_type ON metadata_entries(relationship_type);
CREATE INDEX IF NOT EXISTS idx_metadata_source_node ON metadata_entries(source_node_id);
CREATE INDEX IF NOT EXISTS idx_metadata_target_node ON metadata_entries(target_node_id);
CREATE INDEX IF NOT EXISTS idx_metadata_processing_status ON metadata_entries(processing_status);
CREATE INDEX IF NOT EXISTS idx_metadata_validation_status ON metadata_entries(validation_status);
CREATE INDEX IF NOT EXISTS idx_metadata_document_id ON metadata_entries(document_id);
CREATE INDEX IF NOT EXISTS idx_metadata_chunk_id ON metadata_entries(chunk_id);
CREATE INDEX IF NOT EXISTS idx_metadata_language ON metadata_entries(language);
CREATE INDEX IF NOT EXISTS idx_metadata_document_type ON metadata_entries(document_type);
CREATE INDEX IF NOT EXISTS idx_metadata_embedding_version ON metadata_entries(embedding_version);

-- Full-text search index
CREATE VIRTUAL TABLE IF NOT EXISTS metadata_fts USING fts5(
    title,
    content=metadata_entries,
    tokenize='unicode61'
);
```

## Metadata Store API

```rust
pub struct MetadataStore {
    db: SqliteConnection,
    fts_db: SqliteConnection,
}

impl MetadataStore {
    /// Store or update a metadata entry
    pub async fn upsert(&self, entry: MetadataEntry) -> anyhow::Result<()>;

    /// Get metadata by document ID
    pub async fn get_by_document(&self, document_id: Uuid) -> anyhow::Result<Vec<MetadataEntry>>;

    /// Get metadata by chunk ID
    pub async fn get_by_chunk(&self, chunk_id: Uuid) -> anyhow::Result<Option<MetadataEntry>>;

    /// Get metadata by pack name
    pub async fn get_by_pack(&self, pack_name: &str) -> anyhow::Result<Vec<MetadataEntry>>;

    /// Get metadata by technology
    pub async fn get_by_technology(&self, technology: &str) -> anyhow::Result<Vec<MetadataEntry>>;

    /// Get metadata by vendor
    pub async fn get_by_vendor(&self, vendor: &str) -> anyhow::Result<Vec<MetadataEntry>>;

    /// Search metadata by text
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<MetadataEntry>>;

    /// Filter metadata by criteria
    pub async fn filter(&self, filters: MetadataFilters) -> anyhow::Result<Vec<MetadataEntry>>;

    /// Get all metadata for a workspace
    pub async fn get_by_workspace(&self, workspace_id: Uuid) -> anyhow::Result<Vec<MetadataEntry>>;

    /// Update processing status
    pub async fn update_status(&self, chunk_id: Uuid, status: ProcessingStatus) -> anyhow::Result<()>;

    /// Update embedding metadata
    pub async fn update_embedding(&self, chunk_id: Uuid, version: &str, provider: &str, dimension: usize) -> anyhow::Result<()>;

    /// Get statistics
    pub async fn stats(&self) -> anyhow::Result<MetadataStats>;

    /// Delete metadata
    pub async fn delete(&self, chunk_id: Uuid) -> anyhow::Result<()>;

    /// Bulk upsert
    pub async fn bulk_upsert(&self, entries: Vec<MetadataEntry>) -> anyhow::Result<()>;

    /// Get graph nodes
    pub async fn get_graph_nodes(&self, node_type: Option<NodeType>) -> anyhow::Result<Vec<GraphNode>>;

    /// Get graph edges
    pub async fn get_graph_edges(&self) -> anyhow::Result<Vec<GraphEdge>>;
}
```

## Metadata Filters

```rust
pub struct MetadataFilters {
    pub pack_name: Option<String>,
    pub technologies: Option<Vec<String>>,
    pub vendors: Option<Vec<String>>,
    pub document_types: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub node_types: Option<Vec<NodeType>>,
    pub processing_status: Option<ProcessingStatus>,
    pub quality_min: Option<f32>,
    pub quality_max: Option<f32>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub search_text: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
}

pub enum SortField {
    Title,
    LastModified,
    QualityScore,
    WordCount,
    CreatedAt,
}

pub enum SortOrder {
    Ascending,
    Descending,
}
```

## Metadata Stats

```rust
pub struct MetadataStats {
    pub total_entries: usize,
    pub by_pack: HashMap<String, usize>,
    pub by_technology: HashMap<String, usize>,
    pub by_vendor: HashMap<String, usize>,
    pub by_document_type: HashMap<String, usize>,
    pub by_language: HashMap<String, usize>,
    pub by_processing_status: HashMap<String, usize>,
    pub by_node_type: HashMap<String, usize>,
    pub avg_quality_score: f32,
    pub total_word_count: usize,
    pub total_chunk_count: usize,
    pub graph_nodes: usize,
    pub graph_edges: usize,
}
```

## Graph Node

```rust
pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub title: String,
    pub source: String,
    pub metadata: HashMap<String, String>,
    pub relationship_count: usize,
}

pub struct GraphEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: RelationshipType,
    pub properties: HashMap<String, String>,
}
```

## Design Notes

### Why Graph-Ready

The metadata model is designed for a future knowledge graph, but the graph is NOT implemented in this phase. The graph-ready fields exist to:

1. **Prevent redesign** — When the graph is implemented later, no data migration is needed
2. **Enable gradual rollout** — Graph features can be enabled incrementally
3. **Support planning** — Schema is ready for graph implementation
4. **Enable testing** — Graph relationships can be tested without a full graph engine

### JSON Fields

Fields like `technologies`, `categories`, `tags`, and `edge_properties` use JSON storage because:

1. **Variable cardinality** — Different documents have different numbers of tags/categories
2. **No migration needed** — Adding a new tag doesn't require schema changes
3. **Query flexibility** — SQLite can query JSON fields with json_each() and json_extract()
4. **Future compatibility** — JSON maps naturally to graph properties

### FTS Index

Full-text search on metadata enables:

1. **Fast text search** — Search across titles and other text fields
2. **Combined search** — Combine with vector search for hybrid results
3. **Metadata filtering** — Filter FTS results by metadata fields

### Future Graph Implementation

When the knowledge graph is implemented:

1. `node_id` becomes the primary key in the graph store
2. `source_node_id` + `target_node_id` define edges
3. `relationship_type` defines edge semantics
4. `edge_properties` holds edge metadata
5. A graph engine (Neo4j, etc.) can use this data as-is

The current implementation stores this data in SQLite but does NOT build a graph traversal engine.