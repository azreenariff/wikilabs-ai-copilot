//! Knowledge pack loader — load packs from a directory, validate structure,
//! check schema versions, and produce fully constructed KnowledgePack objects.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use super::manifest::{Manifest, ManifestError};
use super::metadata::{DocumentEncoding, PackDocumentEntry, PackMetadata};
use super::{validate_pack_directory, KnowledgePack};

/// Loader for knowledge packs from a directory.
pub struct PackLoader {
    /// Base directory where packs are stored.
    base_path: PathBuf,
}

impl PackLoader {
    /// Create a new pack loader rooted at the given directory.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Load a single pack from a known subdirectory path.
    pub fn load(&self, pack_name: &str) -> Result<KnowledgePack> {
        let pack_path = self.base_path.join(pack_name);

        debug!(pack_name, path = %pack_path.display(), "Loading pack");

        // Validate directory structure
        validate_pack_directory(&pack_path)
            .with_context(|| format!("Invalid pack directory: {}", pack_name))?;

        // Load manifest
        let manifest = self.load_manifest(&pack_path)?;

        // Load metadata
        let metadata = self.load_metadata(&pack_path)?;

        // Build KnowledgePack
        let pack =
            KnowledgePack::new(manifest, metadata, pack_path.clone(), true)
                .with_context(|| format!("Failed to construct pack: {}", pack_name))?;

        info!(name = %pack.name(), version = %pack.version(), "Pack loaded successfully");
        Ok(pack)
    }

    /// Load a manifest from a pack's root directory.
    fn load_manifest(&self, pack_path: &Path) -> Result<Manifest> {
        let manifest_path = pack_path.join("manifest.yaml");
        Manifest::from_file(&manifest_path)
            .map_err(|e| anyhow!("Failed to load manifest for {}: {}", pack_path.display(), e))
    }

    /// Load metadata from a pack's root directory.
    fn load_metadata(&self, pack_path: &Path) -> Result<PackMetadata> {
        let metadata_path = pack_path.join("metadata.yaml");
        PackMetadata::from_file(&metadata_path)
            .map_err(|e| anyhow!("Failed to load metadata for {}: {}", pack_path.display(), e))
    }

    /// List all pack directories found in the base path.
    pub fn list_available(&self) -> Result<Vec<PathBuf>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut available = Vec::new();
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let manifest = path.join("manifest.yaml");
                let metadata = path.join("metadata.yaml");
                if manifest.exists() && metadata.exists() {
                    available.push(path);
                }
            }
        }

        Ok(available)
    }

    /// Load all valid packs from the base directory.
    pub fn load_all(&self) -> Result<Vec<KnowledgePack>> {
        let paths = self.list_available()?;
        let mut packs = Vec::new();

        for path in &paths {
            let pack_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            match self.load(&pack_name) {
                Ok(pack) => packs.push(pack),
                Err(e) => {
                    warn!(pack = %pack_name, error = %e, "Failed to load pack, skipping");
                }
            }
        }

        // Sort by priority
        packs.sort_by_key(|p| p.manifest.priority.clone());

        info!(count = packs.len(), "Loaded {} packs", packs.len());
        Ok(packs)
    }
}