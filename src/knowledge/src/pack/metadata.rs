//! Detailed metadata for a knowledge pack, stored in `metadata.yaml`.
//!
//! This complements the manifest with richer per-document and runtime info.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

/// Encoding used for documents in the pack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum DocumentEncoding {
    #[default]
    Utf8,
    Ascii,
    Iso8859,
}


/// Information about a single document within a pack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackDocumentEntry {
    /// Relative path within the pack's documents/ directory.
    pub path: String,

    /// Document title.
    pub title: String,

    /// File format / extension.
    pub format: String,

    /// Approximate size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,

    /// Last indexed timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_indexed: Option<DateTime<Utc>>,
}

/// Detailed metadata stored in `metadata.yaml` for a knowledge pack.
///
/// This file contains richer, runtime-generated information about the pack
/// beyond the static manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackMetadata {
    /// Pack name (must match manifest name).
    pub pack_name: String,

    /// List of documents included in the pack.
    #[serde(default)]
    pub documents: Vec<PackDocumentEntry>,

    /// When the pack was last indexed / scanned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_indexed: Option<DateTime<Utc>>,

    /// Provider that supplied the pack's documents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Default document encoding.
    #[serde(default)]
    pub encoding: DocumentEncoding,

    /// Total size of all documents in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size_bytes: Option<usize>,

    /// Custom key-value metadata.
    #[serde(default)]
    pub tags: std::collections::BTreeMap<String, String>,
}

impl PackMetadata {
    /// Load metadata from a YAML file at the given path.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read metadata file: {}", path.display()))?;
        Self::from_str(&content)
    }

    /// Parse metadata from a YAML string.
    pub fn from_str(content: &str) -> Result<Self> {
        let metadata: Self =
            serde_yaml::from_str(content).context("Failed to parse pack metadata YAML")?;
        debug!(pack_name = %metadata.pack_name, "Pack metadata loaded");
        Ok(metadata)
    }

    /// Compute the total document size from entries.
    pub fn compute_total_size(&self) -> usize {
        self.documents.iter().filter_map(|d| d.size).sum()
    }

    /// Serialize the metadata to a YAML string.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| e.into())
    }
}
