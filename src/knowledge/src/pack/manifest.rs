//! Knowledge pack manifest schema.
//!
//! Each pack's `manifest.yaml` defines its identity, versioning, and dependencies.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tracing::debug;

/// Current manifest schema version.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Priority level for pack loading order.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum PackPriority {
    #[default]
    Critical,
    High,
    Normal,
    Low,
}

/// A single dependency on another pack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackDependency {
    pub name: String,
    pub version: String,
}

/// Minimal metadata for a knowledge pack, stored in `manifest.yaml`.
///
/// This schema is versioned; `schema_version` tracks the structure format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    /// Manifest schema version (e.g. 1).
    pub schema_version: u32,

    /// Human-readable pack name (e.g. "openshift", "rhel").
    pub name: String,

    /// Semantic version string (e.g. "1.2.3").
    pub version: String,

    /// One-line description of what the pack covers.
    pub description: String,

    /// Vendor / publisher (e.g. "Red Hat", "IBM").
    pub vendor: String,

    /// Technology tags the pack covers (e.g. ["openshift", "kubernetes"]).
    pub technologies: Vec<String>,

    /// Optional dependency packs.
    #[serde(default)]
    pub dependencies: Vec<PackDependency>,

    /// Loading priority.
    #[serde(default)]
    pub priority: PackPriority,

    /// Optional URL to the pack's source repository.
    #[serde(default)]
    pub repository: Option<String>,

    /// Optional URL to pack documentation.
    #[serde(default)]
    pub documentation_url: Option<String>,

    /// License identifier.
    #[serde(default)]
    pub license: Option<String>,

    /// Pack author(s).
    #[serde(default)]
    pub authors: Vec<String>,

    /// Optional deprecated flag.
    #[serde(default)]
    pub deprecated: Option<bool>,
}

/// Errors returned when loading or validating a manifest.
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("invalid schema version: expected {expected}, got {got}")]
    InvalidSchemaVersion { expected: u32, got: u32 },

    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("dependency resolution error: {0}")]
    DependencyError(String),

    #[error("manifest parse error: {0}")]
    ParseError(#[source] anyhow::Error),

    #[error("file not found: {0}")]
    NotFound(String),
}

impl Manifest {
    /// Load a manifest from a YAML file at the given path.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest file: {}", path.display()))?;
        Self::from_str(&content)
    }

    /// Parse a manifest from a YAML string.
    pub fn from_str(content: &str) -> Result<Self> {
        let manifest: Self =
            serde_yaml::from_str(content).map_err(|e| ManifestError::ParseError(e.into()))?;
        manifest.validate()?;
        debug!(name = %manifest.name, version = %manifest.version, "Manifest loaded");
        Ok(manifest)
    }

    /// Validate the manifest contents.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::MissingField("name".to_string()));
        }
        if self.version.is_empty() {
            return Err(ManifestError::MissingField("version".to_string()));
        }
        if self.description.is_empty() {
            return Err(ManifestError::MissingField("description".to_string()));
        }
        if self.schema_version == 0 {
            return Err(ManifestError::MissingField("schema_version".to_string()));
        }
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(ManifestError::InvalidSchemaVersion {
                expected: CURRENT_SCHEMA_VERSION,
                got: self.schema_version,
            });
        }
        Ok(())
    }

    /// Check if all declared dependencies are satisfied.
    ///
    /// The `satisfied` closure returns true if a dependency of the given
    /// name is present and enabled.
    pub fn is_dependency_satisfied<F>(&self, is_satisfied: F) -> bool
    where
        F: Fn(&str) -> bool,
    {
        self.dependencies.iter().all(|dep| is_satisfied(&dep.name))
    }

    /// Serialize the manifest to a YAML string.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| e.into())
    }
}
