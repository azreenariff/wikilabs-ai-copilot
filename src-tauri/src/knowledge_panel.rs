//! Knowledge Management UI panel — displays and manages knowledge packs in the Tauri app.
//!
//! Provides IPC commands for the frontend to:
//! - List installed packs
//! - Enable/disable packs
//! - Import/export packs
//! - Re-index packs
//! - View metadata
//! - View validation reports
//!
//! Does NOT implement editing of pack contents.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::runtime::Handle;
use tokio::sync::Mutex;

/// Pack information displayed in the knowledge management UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackInfo {
    /// Pack name (e.g., "openshift-knowledge").
    pub name: String,
    /// Pack version (e.g., "1.0.0").
    pub version: String,
    /// Description of the pack.
    pub description: String,
    /// Author or source of the pack.
    pub author: String,
    /// License.
    pub license: String,
    /// Number of documents in the pack.
    pub document_count: usize,
    /// Embedding model used.
    pub embedding_model: String,
    /// Embedding dimensions.
    pub embedding_dimensions: u32,
    /// Whether the pack is enabled.
    pub enabled: bool,
    /// Whether the pack has been indexed.
    pub indexed: bool,
    /// Timestamp of last indexing.
    pub last_indexed: Option<String>,
    /// Validation status (OK, WARNINGS, ERRORS, or ERROR).
    pub validation_status: String,
    /// Number of validation errors.
    pub validation_errors: usize,
    /// Number of validation warnings.
    pub validation_warnings: usize,
    /// File system path to the pack directory.
    pub path: String,
    /// Tags associated with the pack.
    pub tags: Vec<String>,
    /// Categories associated with the pack.
    pub categories: Vec<String>,
    /// Whether the pack was created by the SDK.
    pub sdk_created: bool,
}

/// Validation report displayed in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Overall status.
    pub status: String,
    /// List of errors.
    pub errors: Vec<String>,
    /// List of warnings.
    pub warnings: Vec<String>,
}

/// Represents the state of a knowledge pack in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePackState {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub indexed: bool,
    pub document_count: usize,
    pub last_indexed: Option<String>,
}

/// Knowledge management UI backend.
pub struct KnowledgePanel {
    /// Registered pack directories.
    packs: Mutex<Vec<PackInfo>>,
}

impl KnowledgePanel {
    /// Creates a new knowledge panel.
    pub fn new() -> Self {
        Self {
            packs: Mutex::new(Vec::new()),
        }
    }

    /// Returns a static instance.
    pub fn instance() -> &'static Self {
        use once_cell::sync::Lazy;
        static INSTANCE: Lazy<KnowledgePanel> = Lazy::new(KnowledgePanel::new);
        &INSTANCE
    }

    /// Initializes the panel by discovering installed knowledge packs.
    ///
    /// Searches the configured knowledge packs directory for valid packs.
    pub async fn initialize(&self, packs_dir: &str) -> Result<()> {
        let mut packs = self.packs.lock().await;

        let path = PathBuf::from(packs_dir);
        if !path.exists() {
            tracing::debug!("Knowledge packs directory does not exist: {}", packs_dir);
            return Ok(());
        }

        let mut entries = std::fs::read_dir(&path)?;
        for entry in entries.by_ref() {
            let entry = entry?;
            let entry_path = entry.path();

            if !entry_path.is_dir() {
                continue;
            }

            // Check for manifest.yaml
            let manifest_path = entry_path.join("manifest.yaml");
            let metadata_path = entry_path.join("metadata.yaml");

            if !manifest_path.exists() || !metadata_path.exists() {
                continue;
            }

            // Load manifest
            let manifest_content = std::fs::read_to_string(&manifest_path)?;
            let manifest: wikilabs_knowledge::sdk::schema::Manifest =
                serde_yaml::from_str(&manifest_content).ok().unwrap_or(
                    wikilabs_knowledge::sdk::schema::Manifest {
                        schema_version: "1.0".to_string(),
                        name: "unknown".to_string(),
                        version: "0.0.0".to_string(),
                        description: "No manifest loaded".to_string(),
                        author: "unknown".to_string(),
                        license: "unknown".to_string(),
                        format_version: "1.0".to_string(),
                        documents: vec![],
                        dependencies: vec![],
                    },
                );

            // Load metadata
            let metadata_content = std::fs::read_to_string(&metadata_path)?;
            let metadata: wikilabs_knowledge::sdk::schema::Metadata =
                serde_yaml::from_str(&metadata_content).ok().unwrap_or(
                    wikilabs_knowledge::sdk::schema::Metadata::new(
                        "unknown",
                        "0.0.0",
                        "No metadata loaded",
                        "all-MiniLM-L6-v2",
                    ),
                );

            // Run validation
            let validation_report =
                wikilabs_knowledge::validate::validate_pack_comprehensive(entry_path.to_str().unwrap())
                    .unwrap_or_else(|_| wikilabs_knowledge::validate::ValidationReport {
                        pack_path: entry_path.display().to_string(),
                        overall_status: wikilabs_knowledge::validate::ValidationResult::Error {
                            message: "Validation failed".to_string(),
                        },
                        manifest_ok: false,
                        metadata_ok: false,
                        documents_ok: false,
                        embedding_ok: false,
                        schema_version_ok: false,
                        duplicate_ids_ok: false,
                        dependencies_ok: false,
                        broken_refs_ok: false,
                        version_compat_ok: false,
                        error_count: 1,
                        warning_count: 0,
                        errors: vec!["Validation failed".to_string()],
                        warnings: vec![],
                    });

            let (validation_status, error_count, warning_count) = match &validation_report
                .overall_status
            {
                wikilabs_knowledge::validate::ValidationResult::Valid => {
                    ("OK".to_string(), 0usize, 0usize)
                }
                wikilabs_knowledge::validate::ValidationResult::Invalid {
                    error_count: ec,
                    warning_count: wc,
                } => {
                    if *ec > 0 {
                        ("ERRORS".to_string(), *ec, *wc)
                    } else {
                        ("WARNINGS".to_string(), 0usize, *wc)
                    }
                }
                wikilabs_knowledge::validate::ValidationResult::Error { message: _ } => {
                    ("ERROR".to_string(), 1usize, 0usize)
                }
            };

            let pack = PackInfo {
                name: manifest.name.clone(),
                version: manifest.version,
                description: manifest.description.clone(),
                author: manifest.author.clone(),
                license: manifest.license.clone(),
                document_count: manifest.documents.len(),
                embedding_model: metadata.embedding_model.clone(),
                embedding_dimensions: metadata.embedding_dimensions,
                enabled: true, // Default to enabled
                indexed: false,
                last_indexed: None,
                validation_status,
                validation_errors: error_count,
                validation_warnings: warning_count,
                path: entry_path.display().to_string(),
                tags: metadata.tags.clone(),
                categories: metadata.categories.clone(),
                sdk_created: true,
            };

            packs.push(pack);
        }

        tracing::info!(
            found = packs.len(),
            "Knowledge packs discovered and initialized"
        );

        Ok(())
    }

    /// Returns all installed pack information.
    pub async fn list_packs(&self) -> Vec<PackInfo> {
        let packs = self.packs.lock().await;
        packs.clone()
    }

    /// Gets information about a specific pack.
    pub async fn get_pack(&self, name: &str) -> Option<PackInfo> {
        let packs = self.packs.lock().await;
        packs.iter().find(|p| p.name == name).cloned()
    }

    /// Enables a pack by name.
    pub async fn enable_pack(&self, name: &str) -> Result<()> {
        let mut packs = self.packs.lock().await;
        if let Some(pack) = packs.iter_mut().find(|p| p.name == name) {
            pack.enabled = true;
            tracing::info!(pack = %name, "Pack enabled");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Pack '{}' not found", name))
        }
    }

    /// Disables a pack by name.
    pub async fn disable_pack(&self, name: &str) -> Result<()> {
        let mut packs = self.packs.lock().await;
        if let Some(pack) = packs.iter_mut().find(|p| p.name == name) {
            pack.enabled = false;
            tracing::info!(pack = %name, "Pack disabled");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Pack '{}' not found", name))
        }
    }

    /// Toggles a pack's enabled state.
    pub async fn toggle_pack(&self, name: &str) -> Result<()> {
        let packs = self.packs.lock().await;
        if let Some(pack) = packs.iter().find(|p| p.name == name) {
            let new_state = !pack.enabled;
            drop(packs);
            if new_state {
                self.enable_pack(name).await
            } else {
                self.disable_pack(name).await
            }
        } else {
            Err(anyhow::anyhow!("Pack '{}' not found", name))
        }
    }

    /// Sets the indexed status for a pack.
    pub async fn set_indexed(&self, name: &str, indexed: bool, last_indexed: Option<String>) -> Result<()> {
        let mut packs = self.packs.lock().await;
        if let Some(pack) = packs.iter_mut().find(|p| p.name == name) {
            pack.indexed = indexed;
            pack.last_indexed = last_indexed;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Pack '{}' not found", name))
        }
    }

    /// Gets a validation report for a pack.
    pub async fn get_validation_report(&self, name: &str) -> Result<ValidationReport> {
        let packs = self.packs.lock().await;
        let pack = packs
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Pack '{}' not found", name))?;

        let path = PathBuf::from(&pack.path);

        // Run validation
        let report = wikilabs_knowledge::validate::validate_pack_comprehensive(path.to_str().unwrap())?;

        let status = match &report.overall_status {
            wikilabs_knowledge::validate::ValidationResult::Valid => "VALID".to_string(),
            wikilabs_knowledge::validate::ValidationResult::Invalid {
                error_count: 0,
                warning_count,
            } => {
                if *warning_count > 0 {
                    "WARNINGS".to_string()
                } else {
                    "VALID".to_string()
                }
            }
            wikilabs_knowledge::validate::ValidationResult::Invalid {
                error_count,
                warning_count: _,
            } => {
                if *error_count > 0 {
                    "ERRORS".to_string()
                } else {
                    "WARNINGS".to_string()
                }
            }
            wikilabs_knowledge::validate::ValidationResult::Error { message } => {
                format!("ERROR: {}", message)
            }
        };

        Ok(ValidationReport {
            status,
            errors: report.errors,
            warnings: report.warnings,
        })
    }

    /// Exports a pack to a .wkl file.
    ///
    /// Uses the SDK packager to create a .wkl archive.
    pub async fn export_pack(&self, name: &str, output_path: &str) -> Result<()> {
        let packs = self.packs.lock().await;
        let pack = packs
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Pack '{}' not found", name))?;

        let pack_path = PathBuf::from(&pack.path);

        wikilabs_knowledge::sdk::packager::package_pack(
            pack_path.to_str().unwrap(),
            output_path,
        )?;

        Ok(())
    }

    /// Imports a .wkl pack file.
    ///
    /// Extracts the archive to the knowledge packs directory.
    pub async fn import_pack(&self, archive_path: &str, destination: &str) -> Result<String> {
        wikilabs_knowledge::sdk::packager::extract_pack(archive_path, destination)?;

        // Discover the imported pack name from manifest
        let dest_path = PathBuf::from(destination);
        let extracted_path = dest_path.clone();
        let manifest_path = extracted_path.join("manifest.yaml");

        if manifest_path.exists() {
            let content = std::fs::read_to_string(&manifest_path)?;
            let manifest: wikilabs_knowledge::sdk::schema::Manifest =
                serde_yaml::from_str(&content)?;

            // Re-register the pack
            let mut packs = self.packs.lock().await;
            packs.push(PackInfo {
                name: manifest.name.clone(),
                version: manifest.version,
                description: manifest.description.clone(),
                author: manifest.author.clone(),
                license: manifest.license.clone(),
                document_count: manifest.documents.len(),
                embedding_model: "all-MiniLM-L6-v2".to_string(),
                embedding_dimensions: 384,
                enabled: true,
                indexed: false,
                last_indexed: None,
                validation_status: "UNKNOWN".to_string(),
                validation_errors: 0,
                validation_warnings: 0,
                path: extracted_path.display().to_string(),
                tags: vec![],
                categories: vec![],
                sdk_created: false,
            });
            Ok(extracted_path.display().to_string())
        } else {
            Err(anyhow::anyhow!("No manifest.yaml found in extracted pack"))
        }
    }

    /// Re-indexes a pack by marking it as un-indexed and clearing its cache.
    pub async fn reindex_pack(&self, name: &str) -> Result<()> {
        let mut packs = self.packs.lock().await;
        if let Some(pack) = packs.iter_mut().find(|p| p.name == name) {
            pack.indexed = false;
            pack.last_indexed = None;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Pack '{}' not found", name))
        }
    }

    /// Gets metadata for a pack (minimal info).
    pub async fn get_metadata(&self, name: &str) -> Option<PackInfo> {
        self.get_pack(name).await
    }
}

/// Tauri IPC command to list all packs.
#[tauri::command]
pub fn knowledge_list_packs() -> Vec<PackInfo> {
    let panel = KnowledgePanel::instance();
    // This runs in the Tauri command context
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.list_packs())
}

/// Tauri IPC command to enable a pack.
#[tauri::command]
pub fn knowledge_enable_pack(name: String) -> Result<(), String> {
    let panel = KnowledgePanel::instance();
    Handle::current()
        .block_on(panel.enable_pack(&name))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to disable a pack.
#[tauri::command]
pub fn knowledge_disable_pack(name: String) -> Result<(), String> {
    let panel = KnowledgePanel::instance();
    Handle::current()
        .block_on(panel.disable_pack(&name))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to get pack metadata.
#[tauri::command]
pub fn knowledge_get_pack_metadata(name: String) -> Option<PackInfo> {
    let panel = KnowledgePanel::instance();
    Handle::current().block_on(panel.get_pack(&name))
}

/// Tauri IPC command to get validation report for a pack.
#[tauri::command]
pub fn knowledge_get_validation_report(name: String) -> Result<ValidationReport, String> {
    let panel = KnowledgePanel::instance();
    Handle::current()
        .block_on(panel.get_validation_report(&name))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to export a pack.
#[tauri::command]
pub fn knowledge_export_pack(name: String, output_path: String) -> Result<(), String> {
    let panel = KnowledgePanel::instance();
    Handle::current()
        .block_on(panel.export_pack(&name, &output_path))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to import a pack.
#[tauri::command]
pub fn knowledge_import_pack(
    archive_path: String,
    destination: String,
) -> Result<String, String> {
    let panel = KnowledgePanel::instance();
    Handle::current()
        .block_on(panel.import_pack(&archive_path, &destination))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to re-index a pack.
#[tauri::command]
pub fn knowledge_reindex_pack(name: String) -> Result<(), String> {
    let panel = KnowledgePanel::instance();
    Handle::current()
        .block_on(panel.reindex_pack(&name))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_info_default() {
        let pack = PackInfo {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test pack".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            document_count: 5,
            embedding_model: "all-MiniLM-L6-v2".to_string(),
            embedding_dimensions: 384,
            enabled: true,
            indexed: false,
            last_indexed: None,
            validation_status: "OK".to_string(),
            validation_errors: 0,
            validation_warnings: 0,
            path: "/tmp/test".to_string(),
            tags: vec![],
            categories: vec![],
            sdk_created: true,
        };
        assert_eq!(pack.name, "test");
        assert!(pack.enabled);
        assert!(!pack.indexed);
    }

    #[test]
    fn test_validation_report() {
        let report = ValidationReport {
            status: "VALID".to_string(),
            errors: vec![],
            warnings: vec![],
        };
        assert_eq!(report.status, "VALID");
        assert!(report.errors.is_empty());
    }
}