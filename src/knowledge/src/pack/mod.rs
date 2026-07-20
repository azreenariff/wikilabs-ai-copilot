//! Knowledge pack — represents a single loaded pack directory with its
//! manifest, metadata, and filesystem layout.

mod manifest;
mod metadata;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::debug;
use uuid::Uuid;

use manifest::{Manifest, PackPriority};
use metadata::{DocumentEncoding, PackDocumentEntry, PackMetadata};

/// Required subdirectories that every pack must contain.
const REQUIRED_DIRS: &[&str] = &[
    "documents",
    "embeddings",
    "indexes",
    "relationships",
    "tests",
    "documentation",
];

/// A fully loaded knowledge pack.
///
/// Combines the manifest, metadata, and the filesystem path to all
/// pack resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePack {
    /// Unique pack identifier.
    pub id: Uuid,

    /// Pack manifest (name, version, dependencies, etc.).
    pub manifest: Manifest,

    /// Detailed runtime metadata.
    pub metadata: PackMetadata,

    /// Absolute path to the pack's root directory.
    pub path: PathBuf,

    /// Whether the pack is currently enabled.
    pub enabled: bool,

    /// When the pack was loaded.
    pub loaded_at: chrono::DateTime<Utc>,

    /// Manifest schema version.
    pub schema_version: u32,
}

impl KnowledgePack {
    /// Create a new KnowledgePack from a loaded manifest and metadata.
    pub fn new(
        manifest: Manifest,
        metadata: PackMetadata,
        path: PathBuf,
        enabled: bool,
    ) -> Result<Self> {
        let now = Utc::now();
        let total_size = metadata.compute_total_size();
        let mut metadata = metadata;
        metadata.total_size_bytes = Some(total_size);
        metadata.last_indexed = Some(now);
        metadata.pack_name.clone_from(&manifest.name);

        let manifest = manifest.clone();
        let schema_version = manifest.schema_version.clone();
        Ok(Self {
            id: Uuid::new_v4(),
            manifest,
            metadata,
            path,
            enabled,
            loaded_at: now,
            schema_version,
        })
    }

    /// Return the pack's name.
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// Return the pack's version.
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// Return the pack's priority.
    pub fn priority(&self) -> &PackPriority {
        &self.manifest.priority
    }

    /// Return the pack's technology tags.
    pub fn technologies(&self) -> &[String] {
        &self.manifest.technologies
    }

    /// Return the pack's vendor.
    pub fn vendor(&self) -> &str {
        &self.manifest.vendor
    }

    /// List all document paths inside the pack's documents/ directory.
    pub fn document_paths(&self) -> Result<Vec<PathBuf>> {
        let docs_dir = self.path.join("documents");
        if !docs_dir.exists() {
            return Ok(Vec::new());
        }
        let mut files = Vec::new();
        Self::collect_files(&docs_dir, &mut files)?;
        Ok(files)
    }

    fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                Self::collect_files(&path, files)?;
            }
        }
        Ok(())
    }

    /// Get the path to a specific subdirectory within the pack.
    pub fn sub_dir(&self, subdir: &str) -> PathBuf {
        self.path.join(subdir)
    }

    /// Check if the pack is deprecated.
    pub fn is_deprecated(&self) -> bool {
        self.manifest.deprecated.unwrap_or(false)
    }
}

/// Validate that a directory looks like a valid knowledge pack root.
pub fn validate_pack_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Pack path does not exist: {}", path.display());
    }
    if !path.is_dir() {
        anyhow::bail!("Pack path is not a directory: {}", path.display());
    }

    // Check manifest.yaml exists
    let manifest_path = path.join("manifest.yaml");
    if !manifest_path.exists() {
        anyhow::bail!("Missing required file: manifest.yaml");
    }

    // Check metadata.yaml exists
    let metadata_path = path.join("metadata.yaml");
    if !metadata_path.exists() {
        anyhow::bail!("Missing required file: metadata.yaml");
    }

    // Check required subdirectories
    for dir in REQUIRED_DIRS {
        let dir_path = path.join(dir);
        if !dir_path.exists() {
            anyhow::bail!("Missing required directory: {}", dir);
        }
        if !dir_path.is_dir() {
            anyhow::bail!("Expected directory but found file: {}", dir);
        }
    }

    debug!(path = %path.display(), "Pack directory validated");
    Ok(())
}

/// Discover all valid knowledge pack directories inside a parent directory.
pub fn discover_packs(parent: &Path) -> Result<Vec<PathBuf>> {
    let mut packs = Vec::new();

    if !parent.exists() {
        return Ok(packs);
    }

    for entry in std::fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_path = path.join("manifest.yaml");
            let metadata_path = path.join("metadata.yaml");

            if manifest_path.exists() && metadata_path.exists() {
                packs.push(path);
            }
        }
    }

    debug!(count = packs.len(), "Discovered knowledge packs");
    Ok(packs)
}

/// Sort packs by priority order (Critical > High > Normal > Low).
pub fn sort_packs_by_priority(packs: &mut Vec<KnowledgePack>) {
    packs.sort_by_key(|p| p.manifest.priority.clone());
}

/// Return the set of all technology tags across a list of packs.
pub fn collect_technologies(packs: &[KnowledgePack]) -> HashSet<String> {
    let mut techs = HashSet::new();
    for pack in packs {
        for t in &pack.manifest.technologies {
            techs.insert(t.clone());
        }
    }
    techs
}

/// Return the set of all vendors across a list of packs.
pub fn collect_vendors(packs: &[KnowledgePack]) -> HashSet<String> {
    let mut vendors = HashSet::new();
    for pack in packs {
        vendors.insert(pack.manifest.vendor.clone());
    }
    vendors
}
