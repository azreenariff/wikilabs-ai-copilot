//! Document discovery step — find documents in paths.

use crate::pipeline::PipelineConfig;
use anyhow::Context;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// A discovered document with its metadata.
#[derive(Debug, Clone)]
pub struct DiscoveredDoc {
    pub path: PathBuf,
    pub filename: String,
    pub title: String,
    pub extension: String,
    pub file_size: u64,
    pub modified_time: std::time::SystemTime,
    pub workspace_id: uuid::Uuid,
}

/// The discovery pipeline step.
pub struct DiscoverStep {
    config: PipelineConfig,
    globset: GlobSet,
}

impl DiscoverStep {
    pub fn new(config: &PipelineConfig) -> Self {
        let mut builder = GlobSetBuilder::new();

        // Build glob patterns from supported extensions
        for ext in &config.supported_extensions {
            let pattern = format!("**/*{}", ext);
            if let Ok(glob) = Glob::new(&pattern) {
                builder.add(glob);
            }
        }

        let globset = builder.build().expect("Failed to build globset");

        Self {
            config: config.clone(),
            globset,
        }
    }

    /// Run the discovery step on given paths.
    pub async fn run(&self, paths: &[&str]) -> anyhow::Result<Vec<DiscoveredDoc>> {
        let mut results = Vec::new();

        for path_str in paths {
            let path = Path::new(path_str);
            self.discover_from_path(path, &mut results)?;
        }

        debug!(count = results.len(), "Discovery step complete");
        Ok(results)
    }

    fn discover_from_path(&self, path: &Path, results: &mut Vec<DiscoveredDoc>) -> anyhow::Result<()> {
        if path.is_file() {
            if self.should_discover(path) {
                let meta = fs::metadata(path).with_context(|| format!("Failed to read metadata for {}", path.to_string_lossy()))?;
                let doc = DiscoveredDoc {
                    path: path.to_path_buf(),
                    filename: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string(),
                    title: super::super::filename_to_title(path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown")),
                    extension: path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_string(),
                    file_size: meta.len(),
                    modified_time: meta.modified().unwrap_or_else(|_| std::time::SystemTime::now()),
                    workspace_id: self.config.workspace_id,
                };
                debug!(path = ?doc.path, size = doc.file_size, "Discovered document");
                results.push(doc);
            }
        } else if path.is_dir() {
            // Recursively discover files in directory
            let entries = fs::read_dir(path)
                .with_context(|| format!("Failed to read directory: {}", path))?;

            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let entry_path = entry.path();
                        if entry_path.is_dir() {
                            // Skip hidden directories and common non-source dirs
                            if let Some(name) = entry_path.file_name() {
                                let name_str = name.to_str().unwrap_or("");
                                if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
                                    continue;
                                }
                            }
                            self.discover_from_path(&entry_path, results)?;
                        } else if self.should_discover(&entry_path) {
                            let meta = fs::metadata(path).with_context(|| format!("Failed to read metadata for {}", path.to_string_lossy()))?;
                            let doc = DiscoveredDoc {
                                path: entry_path.clone(),
                                filename: entry_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("")
                                    .to_string(),
                                title: super::super::filename_to_title(entry_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown")),
                                extension: entry_path
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("")
                                    .to_string(),
                                file_size: meta.len(),
                                modified_time: meta.modified().unwrap_or_else(|_| std::time::SystemTime::now()),
                                workspace_id: self.config.workspace_id,
                            };
                            results.push(doc);
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to read directory entry, skipping");
                    }
                }
            }
        }

        Ok(())
    }

    fn should_discover(&self, path: &Path) -> bool {
        // Check if file extension is supported
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !self.config.supported_extensions.iter().any(|s| s == &format!(".{}", ext)) {
                return false;
            }
        } else {
            return false;
        }

        // Check if path matches glob pattern
        self.globset.is_match(path)
    }
}