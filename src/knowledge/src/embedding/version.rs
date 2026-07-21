//! Embedding versioning — track which model generated which embeddings.
//!
/// Supports embedding version tracking for rollback and migration.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Represents an embedding version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingVersion {
    /// Unique version identifier
    pub id: uuid::Uuid,
    /// Embedding model/provider name
    pub model_name: String,
    /// Embedding dimensions
    pub dimensions: usize,
    /// Description of changes
    pub description: String,
    /// Number of chunks embedded in this version
    pub chunk_count: usize,
    /// When this version was created
    pub created_at: DateTime<Utc>,
    /// When this version was finalized
    pub finalized_at: Option<DateTime<Utc>>,
    /// Status of the version
    pub status: EmbeddingVersionStatus,
}

/// Status of an embedding version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmbeddingVersionStatus {
    Creating,
    Active,
    Deprecated,
}

impl EmbeddingVersion {
    pub fn new(model_name: &str, dimensions: usize, description: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            model_name: model_name.to_string(),
            dimensions,
            description: description.to_string(),
            chunk_count: 0,
            created_at: Utc::now(),
            finalized_at: None,
            status: EmbeddingVersionStatus::Creating,
        }
    }

    pub fn finalize(mut self, chunk_count: usize) -> Self {
        self.chunk_count = chunk_count;
        self.finalized_at = Some(Utc::now());
        self.status = EmbeddingVersionStatus::Active;
        debug!(
            version_id = %self.id,
            chunk_count = chunk_count,
            model = self.model_name,
            "Embedding version finalized"
        );
        self
    }

    pub fn deprecated(mut self) -> Self {
        self.status = EmbeddingVersionStatus::Deprecated;
        self
    }
}

/// Manages embedding version tracking.
pub struct EmbeddingVersionManager {
    versions: Vec<EmbeddingVersion>,
    active_version: Option<uuid::Uuid>,
}

impl EmbeddingVersionManager {
    pub fn new() -> Self {
        Self {
            versions: Vec::new(),
            active_version: None,
        }
    }

    /// Create a new embedding version.
    pub fn create_version(
        &mut self,
        model_name: String,
        dimensions: usize,
        description: String,
    ) -> EmbeddingVersion {
        // Deprecate existing active version
        if let Some(active_id) = self.active_version {
            for v in &mut self.versions {
                if v.id == active_id && v.status == EmbeddingVersionStatus::Active {
                    v.status = EmbeddingVersionStatus::Deprecated;
                    debug!(version_id = %v.id, "Previous version deprecated");
                    break;
                }
            }
        }

        let version = EmbeddingVersion::new(&model_name, dimensions, &description);
        self.versions.push(version.clone());
        self.active_version = Some(version.id);
        debug!(version_id = %version.id, model = model_name, "New embedding version created");
        version
    }

    /// Finalize a version with chunk count.
    pub fn finalize_version(&mut self, version: &EmbeddingVersion, chunk_count: Option<usize>) {
        if let Some(count) = chunk_count {
            self.versions.iter_mut().for_each(|v| {
                if v.id == version.id {
                    *v = v.clone().finalize(count);
                }
            });
        } else {
            self.versions.iter_mut().for_each(|v| {
                if v.id == version.id {
                    v.status = EmbeddingVersionStatus::Active;
                }
            });
        }
        debug!(version_id = %version.id, "Version finalized");
    }

    /// Get the currently active version.
    pub fn get_active(&self) -> Option<&EmbeddingVersion> {
        if let Some(active_id) = self.active_version {
            self.versions.iter().find(|v| v.id == active_id)
        } else {
            None
        }
    }

    /// Get all versions.
    pub fn all_versions(&self) -> &[EmbeddingVersion] {
        &self.versions
    }

    /// Get a version by ID.
    pub fn get_by_id(&self, id: uuid::Uuid) -> Option<&EmbeddingVersion> {
        self.versions.iter().find(|v| v.id == id)
    }

    /// Migrate embeddings to a new version (placeholder for future implementation).
    pub fn migrate_to(
        &mut self,
        target_version_id: uuid::Uuid,
        source_docs: &[uuid::Uuid],
    ) -> anyhow::Result<()> {
        if !self.versions.iter().any(|v| v.id == target_version_id) {
            anyhow::bail!("Target version {} does not exist", target_version_id);
        }

        debug!(
            version_id = %target_version_id,
            doc_count = source_docs.len(),
            "Migration planned"
        );

        Ok(())
    }

    /// Rollback to a previous version (placeholder for future implementation).
    pub fn rollback_to(&mut self, version_id: uuid::Uuid) -> anyhow::Result<()> {
        let version = self
            .get_by_id(version_id)
            .ok_or_else(|| anyhow::anyhow!("Version {} not found", version_id))?;

        if version.status != EmbeddingVersionStatus::Active {
            anyhow::bail!("Cannot rollback to non-active version");
        }

        // Deactivate current active version
        if let Some(active_id) = self.active_version {
            for v in &mut self.versions {
                if v.id == active_id {
                    v.status = EmbeddingVersionStatus::Deprecated;
                    break;
                }
            }
        }

        self.active_version = Some(version_id);
        debug!(version_id = %version_id, "Rolled back to version");
        Ok(())
    }
}

impl Default for EmbeddingVersionManager {
    fn default() -> Self {
        Self::new()
    }
}
