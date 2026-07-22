//! Git repository provider — scans Git repositories for tracked files.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

use super::{KnowledgeProvider, ProviderDocument};

/// A provider that discovers and parses files from Git repositories.
pub struct GitProvider {
    #[allow(dead_code)]
    enabled: bool,
    extensions: Vec<String>,
}

impl Default for GitProvider {
    fn default() -> Self {
        Self {
            enabled: true,
            extensions: vec![],
        }
    }
}

impl GitProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }
}

#[async_trait]
impl KnowledgeProvider for GitProvider {
    fn name(&self) -> &str {
        "git"
    }

    fn supported_formats(&self) -> &[&str] {
        &[
            "md",
            "txt",
            "html",
            "yaml",
            "yml",
            "json",
            "xml",
            "sh",
            "bash",
            "py",
            "rs",
            "go",
            "java",
            "js",
            "ts",
            "toml",
            "conf",
            "cfg",
            "ini",
            "dockerfile",
            "gitignore",
            "gitattributes",
        ]
    }

    fn get_enabled(&self, enabled: bool) -> bool {
        enabled
    }

    async fn discover(&self, path: &str) -> Result<Vec<ProviderDocument>> {
        let repo_path = Path::new(path);
        if !repo_path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path));
        }

        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            return Err(anyhow::anyhow!(
                "Not a git repository (no .git directory found): {}",
                path
            ));
        }

        let mut docs = Vec::new();

        let output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["ls-files", "-z"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let files_str = String::from_utf8_lossy(&output.stdout);
                for file in files_str.split('\0') {
                    if file.is_empty() {
                        continue;
                    }
                    let full_path = repo_path.join(file);
                    if full_path.is_file()
                        && (self.extensions.is_empty()
                            || self.extensions.iter().any(|e| {
                                full_path.extension().and_then(|ext| ext.to_str())
                                    == Some(e.as_str())
                            }))
                    {
                        match self.parse(&full_path.to_string_lossy()).await {
                            Ok(doc) => docs.push(doc),
                            Err(e) => warn!(file, error = %e, "Failed to parse git file"),
                        }
                    }
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Failed to list git files"));
            }
        }

        debug!(count = docs.len(), path, "Git discovery complete");
        Ok(docs)
    }

    async fn parse(&self, path: &str) -> Result<ProviderDocument> {
        let p = Path::new(path);
        let content =
            fs::read_to_string(p).with_context(|| format!("Failed to read file: {}", path))?;

        let metadata =
            fs::metadata(p).with_context(|| format!("Failed to read metadata: {}", path))?;

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                    .unwrap_or_default()
            })
            .unwrap_or_else(Utc::now);

        let title = p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = p
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(ProviderDocument {
            id: uuid::Uuid::new_v4(),
            title,
            source: path.to_string(),
            extension,
            content,
            size_bytes: metadata.len() as usize,
            mime_type: "text/plain".to_string(),
            author: String::new(),
            modified_at,
        })
    }
}