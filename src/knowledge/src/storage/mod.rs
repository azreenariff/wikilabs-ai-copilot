//! Vector Storage module — SQLite-based vector storage with VSS support.
//!
//! Provides namespace isolation, workspace isolation, incremental indexing,
//! deletion support, versioning, and migration capabilities.

pub mod deletion;
pub mod incremental;
pub mod index;
pub mod migration;
pub mod namespace;
pub mod vector_store;

pub use deletion::DocumentDeleter;
pub use incremental::IncrementalIndexer;
pub use index::{IndexManager, IndexStats};
pub use migration::{MigrationResult, SchemaMigration};
pub use vector_store::{VectorStore, VectorStoreConfig};

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Configuration for the vector storage system.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Database path
    pub db_path: String,
    /// Default namespace
    pub namespace: String,
    /// Workspace ID
    pub workspace_id: uuid::Uuid,
    /// Schema version
    pub schema_version: u32,
}

impl StorageConfig {
    pub fn new(db_path: &str, namespace: &str, workspace_id: uuid::Uuid) -> Self {
        Self {
            db_path: db_path.to_string(),
            namespace: namespace.to_string(),
            workspace_id,
            schema_version: 1,
        }
    }
}

/// The vector storage orchestrator — provides a unified interface
/// to all storage sub-components.
pub struct VectorStorage {
    store: Arc<Mutex<vector_store::VectorStore>>,
    index_manager: IndexManager,
    #[allow(dead_code)]
    incremental_indexer: IncrementalIndexer,
    deleter: DocumentDeleter,
    config: StorageConfig,
}

impl VectorStorage {
    pub fn new(config: StorageConfig) -> anyhow::Result<Self> {
        info!(
            db_path = config.db_path,
            namespace = config.namespace,
            "Creating VectorStorage"
        );

        let store = Arc::new(Mutex::new(vector_store::VectorStore::new(
            &config.db_path,
            &config.namespace,
            config.workspace_id,
            config.schema_version,
        )?));

        let index_manager = IndexManager::new(Arc::clone(&store));
        let incremental_indexer = IncrementalIndexer::new(Arc::clone(&store));
        let deleter = DocumentDeleter::new(Arc::clone(&store));

        Ok(Self {
            store,
            index_manager,
            incremental_indexer,
            deleter,
            config,
        })
    }

    pub async fn insert_vector(
        &self,
        vector_id: &str,
        vector: &[f32],
        content: &str,
        doc_id: &str,
    ) -> anyhow::Result<()> {
        let store = self.store.lock().await;
        store
            .insert_vector(vector_id, vector, content, doc_id)
            .await
    }

    pub async fn search(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> anyhow::Result<Vec<search::SearchResult>> {
        let store = self.store.lock().await;
        store.search(query_vector, limit).await
    }

    pub async fn delete_document(&self, doc_id: &str) -> anyhow::Result<usize> {
        self.deleter.delete_by_doc_id(doc_id).await
    }

    pub async fn get_index_stats(&self) -> anyhow::Result<IndexStats> {
        let store = self.store.lock().await;
        Ok(self.index_manager.get_stats(&store).await)
    }

    pub fn namespace(&self) -> &str {
        &self.config.namespace
    }

    pub fn workspace_id(&self) -> uuid::Uuid {
        self.config.workspace_id
    }
}

// Re-export types for convenience
pub mod search {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResult {
        pub vector_id: String,
        pub score: f32,
        pub content: String,
        pub doc_id: String,
    }
}
