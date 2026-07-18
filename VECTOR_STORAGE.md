# Vector Storage Specification

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

The Vector Storage layer provides the foundation for semantic search in the Enterprise Knowledge Platform. It uses SQLite VSS (Vector Similarity Search) with namespace isolation, incremental indexing, and future migration support.

## Architecture

```
Knowledge Platform
  ↓
Vector Storage Layer
  ├─ Namespace Management
  ├─ Vector Operations (store, search, delete)
  ├─ Index Management
  ├─ Incremental Indexing
  ├─ Deletion
  └─ Migration
```

## Database Schema

### Vector Table

```sql
CREATE TABLE IF NOT EXISTS vectors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id TEXT NOT NULL UNIQUE,
    doc_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    vector BLOB NOT NULL,       -- serialized f32[]
    chunk_index INTEGER NOT NULL,
    total_chunks INTEGER NOT NULL,
    heading_context TEXT,
    section_path TEXT,
    pack_name TEXT NOT NULL,
    namespace TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    embedding_version TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Metadata Index Table

```sql
CREATE TABLE IF NOT EXISTS vector_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id TEXT NOT NULL UNIQUE,
    doc_id TEXT NOT NULL,
    pack_name TEXT NOT NULL,
    vendor TEXT,
    product TEXT,
    version TEXT,
    technology TEXT,
    author TEXT,
    language TEXT,
    tags TEXT,                  -- JSON array
    security_classification TEXT,
    source TEXT,
    confidence REAL,
    citation_id TEXT,
    FOREIGN KEY (chunk_id) REFERENCES vectors(chunk_id)
);
```

### Namespace Registry

```sql
CREATE TABLE IF NOT EXISTS namespaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    namespace TEXT NOT NULL UNIQUE,
    pack_name TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    schema_version TEXT NOT NULL DEFAULT '1.0.0',
    total_chunks INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Index History

```sql
CREATE TABLE IF NOT EXISTS index_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    namespace TEXT NOT NULL,
    pack_name TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    operation TEXT NOT NULL,     -- 'index', 'delete', 'reindex', 'migration'
    document_count INTEGER NOT NULL,
    chunk_count INTEGER NOT NULL,
    embedding_version TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    duration_ms INTEGER
);
```

### Schema Version Table

```sql
CREATE TABLE IF NOT EXISTS schema_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schema_name TEXT NOT NULL UNIQUE,
    current_version TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    migration_history TEXT       -- JSON array of migrations
);
```

## Indexes

```sql
-- Vector search indexes
CREATE INDEX IF NOT EXISTS idx_vectors_namespace ON vectors(namespace);
CREATE INDEX IF NOT EXISTS idx_vectors_pack_name ON vectors(pack_name);
CREATE INDEX IF NOT EXISTS idx_vectors_workspace_id ON vectors(workspace_id);
CREATE INDEX IF NOT EXISTS idx_vectors_doc_id ON vectors(doc_id);
CREATE INDEX IF NOT EXISTS idx_vectors_embedding_version ON vectors(embedding_version);

-- Metadata search indexes
CREATE INDEX IF NOT EXISTS idx_metadata_pack_name ON vector_metadata(pack_name);
CREATE INDEX IF NOT EXISTS idx_metadata_technology ON vector_metadata(technology);
CREATE INDEX IF NOT EXISTS idx_metadata_vendor ON vector_metadata(vendor);
CREATE INDEX IF NOT EXISTS idx_metadata_tags ON vector_metadata(tags);
CREATE INDEX IF NOT EXISTS idx_metadata_workspace_id ON vector_metadata(workspace_id);

-- Namespace indexes
CREATE INDEX IF NOT EXISTS idx_namespaces_pack ON namespaces(pack_name);
CREATE INDEX IF NOT EXISTS idx_namespaces_workspace ON namespaces(workspace_id);
```

## Vector Storage Trait

```rust
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Store embeddings for chunks
    async fn store(&self, embeddings: Vec<IndexedEmbedding>) -> anyhow::Result<StoreResult>;

    /// Search by vector similarity
    async fn search(&self, query: SearchQuery) -> anyhow::Result<Vec<SearchResult>>;

    /// Search with metadata filters
    async fn search_filtered(&self, query: SearchQuery, filters: SearchFilters) -> anyhow::Result<Vec<SearchResult>>;

    /// Delete document chunks
    async fn delete(&self, doc_id: Uuid, pack_name: &str) -> anyhow::Result<DeleteResult>;

    /// Delete by namespace
    async fn delete_namespace(&self, namespace: &str) -> anyhow::Result<DeleteResult>;

    /// Get vector by chunk ID
    async fn get(&self, chunk_id: Uuid) -> anyhow::Result<Option<IndexedEmbedding>>;

    /// Get statistics
    async fn stats(&self, namespace: Option<&str>) -> anyhow::Result<StorageStats>;

    /// Check namespace exists
    async fn namespace_exists(&self, namespace: &str) -> bool;

    /// Create namespace
    async fn create_namespace(&self, namespace: &str) -> anyhow::Result<()>;

    /// Migrate schema
    async fn migrate(&self, from_version: &str, to_version: &str) -> anyhow::Result<MigrationResult>;

    /// Compact database
    async fn compact(&self) -> anyhow::Result<()>;

    /// Backup database
    async fn backup(&self, path: &str) -> anyhow::Result<()>;
}
```

## Search Query

```rust
pub struct SearchQuery {
    pub vector: Vec<f32>,
    pub text: String,
    pub limit: usize,
    pub offset: usize,
    pub min_score: f32,
    pub filters: Option<SearchFilters>,
    pub namespace: Option<String>,
    pub workspace_id: Option<Uuid>,
}

pub struct SearchFilters {
    pub pack_name: Option<String>,
    pub technology: Option<String>,
    pub vendor: Option<String>,
    pub tags: Option<Vec<String>>,
    pub language: Option<String>,
    pub security_classification: Option<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub embedding_version: Option<String>,
    pub exclude_pack_names: Option<Vec<String>>,
}

pub struct SearchResult {
    pub chunk_id: String,
    pub doc_id: String,
    pub title: String,
    pub content: String,
    pub heading_context: Option<String>,
    pub section_path: String,
    pub score: f32,
    pub rank: usize,
    pub pack_name: String,
    pub namespace: String,
    pub metadata: HashMap<String, String>,
}

pub struct StorageStats {
    pub total_vectors: usize,
    pub total_namespaces: usize,
    pub namespace_stats: Vec<NamespaceStats>,
    pub database_size: usize,
    pub chunk_count: usize,
    pub doc_count: usize,
}
```

## Namespace Management

Namespaces provide isolation between knowledge packs and workspaces:

```rust
pub struct NamespaceManager {
    pub db: SqliteConnection,
}

impl NamespaceManager {
    pub fn create(&self, namespace: &str, pack_name: &str, workspace_id: Uuid) -> anyhow::Result<()>;
    pub fn delete(&self, namespace: &str) -> anyhow::Result<()>;
    pub fn list(&self) -> anyhow::Result<Vec<NamespaceInfo>>;
    pub fn get_stats(&self, namespace: &str) -> anyhow::Result<NamespaceStats>;
    pub fn is_enabled(&self, pack_name: &str, workspace_id: Uuid) -> bool;
}

pub struct NamespaceInfo {
    pub namespace: String,
    pub pack_name: String,
    pub workspace_id: Uuid,
    pub schema_version: String,
    pub total_chunks: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Namespace naming convention:

- Format: `{pack_name}::{workspace_id}`
- Default: `{pack_name}::default`
- Example: `openshift::abc123-def456`

## Incremental Indexing

Only indexes new or changed documents:

```rust
pub struct IncrementalIndexer {
    pub db: SqliteConnection,
    pub change_tracker: ChangeTracker,
}

impl IncrementalIndexer {
    pub fn new(db: SqliteConnection) -> Self;
    pub async fn index_new(&self, embeddings: Vec<IndexedEmbedding>) -> anyhow::Result<IndexResult>;
    pub async fn update_changed(&self, embeddings: Vec<IndexedEmbedding>) -> anyhow::Result<IndexResult>;
    pub async fn upsert(&self, embeddings: Vec<IndexedEmbedding>) -> anyhow::Result<IndexResult>;
    pub async fn delete_removed(&self, removed_doc_ids: Vec<Uuid>) -> anyhow::Result<DeleteResult>;
}

pub struct IndexResult {
    pub inserted: usize,
    pub updated: usize,
    pub deleted: usize,
    pub errors: Vec<String>,
}
```

Change tracking strategy:

1. Store content hash with each chunk
2. On re-index, compare new hash vs stored hash
3. Insert new chunks, update changed chunks, skip unchanged
4. Delete chunks for removed documents

## Deletion

```rust
pub struct DeletionHandler {
    pub db: SqliteConnection,
}

impl DeletionHandler {
    pub fn delete_document(&self, doc_id: Uuid, pack_name: &str) -> anyhow::Result<DeleteResult>;
    pub fn delete_namespace(&self, namespace: &str) -> anyhow::Result<DeleteResult>;
    pub fn delete_by_filter(&self, filters: SearchFilters) -> anyhow::Result<DeleteResult>;
    pub fn delete_all(&self) -> anyhow::Result<DeleteResult>;
}

pub struct DeleteResult {
    pub chunks_deleted: usize,
    pub documents_deleted: usize,
    pub namespaces_affected: Vec<String>,
    pub errors: Vec<String>,
}
```

Atomic deletion:

1. Begin transaction
2. Delete vectors matching criteria
3. Delete metadata matching criteria
4. Update namespace stats
5. Log deletion in index_history
6. Commit transaction or rollback on error

## Migration Support

```rust
pub struct MigrationManager {
    pub db: SqliteConnection,
    pub migrations: Vec<Migration>,
}

pub struct Migration {
    pub from_version: String,
    pub to_version: String,
    pub steps: Vec<MigrationStep>,
    pub rollback_steps: Vec<MigrationStep>,
}

pub enum MigrationStep {
    AddColumn { table: String, column: String, schema: String },
    DropColumn { table: String, column: String },
    RenameColumn { table: String, old_name: String, new_name: String },
    CreateTable { table: String, schema: String },
    DropTable { table: String },
    AddIndex { table: String, index: String, columns: Vec<String> },
    DropIndex { table: String, index: String },
    InsertData { table: String, query: String },
    UpdateData { table: String, query: String },
}
```

Migration process:

1. Check current schema version
2. Load migration path (e.g., 1.0 → 1.1 → 2.0)
3. Apply migrations sequentially
4. Validate data integrity
5. Update schema version table
6. Log migration in index_history

## Background Indexing

Indexing runs in background tasks:

```rust
pub struct BackgroundIndexer {
    pub db: SqliteConnection,
    pub task_queue: Arc<Mutex<Vec<IndexTask>>>,
    pub running: AtomicBool,
}

impl BackgroundIndexer {
    pub fn enqueue(&self, task: IndexTask) -> TaskId;
    pub fn start(&self);
    pub fn stop(&self);
    pub fn is_running(&self) -> bool;
    pub fn progress(&self) -> BackgroundProgress;
}

pub struct IndexTask {
    pub id: TaskId,
    pub pack_name: String,
    pub workspace_id: Uuid,
    pub embeddings: Vec<IndexedEmbedding>,
    pub operation: IndexOperation,  // Insert, Update, Delete
    pub priority: TaskPriority,     // Low, Normal, High
    pub cancel_token: CancellationToken,
}

pub struct BackgroundProgress {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub running_tasks: usize,
    pub failed_tasks: usize,
    pub current_task: Option<String>,
}
```

## Performance

### Batch Operations

- Bulk insert with transactions
- Batch delete with WHERE IN
- Batch update with CASE WHEN

### Memory Management

- Streaming reads for large queries
- Cursor-based pagination
- Result size limits

### Concurrency

- SQLite WAL mode for concurrent reads
- Write lock per transaction
- Read connections don't block write connection

### Optimization

- Use EXPLAIN ANALYZE for slow queries
- Index maintenance (REINDEX periodically)
- Vacuum when table sizes shrink significantly
- Prune old index_history entries

## Data Integrity

- UNIQUE constraints on chunk_id
- NOT NULL on required fields
- CHECK constraints on validation
- Foreign key enforcement
- Transaction-based operations
- Checksum verification for vector data