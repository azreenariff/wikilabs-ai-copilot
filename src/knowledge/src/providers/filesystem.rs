//! Filesystem provider — recursively scans directories for files.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

use super::{KnowledgeProvider, ProviderDocument};

/// A provider that scans the filesystem for files by extension.
pub struct FilesystemProvider {
    #[allow(dead_code)]
    enabled: bool,
    extensions: Vec<String>,
}

impl Default for FilesystemProvider {
    fn default() -> Self {
        Self {
            enabled: true,
            extensions: vec![],
        }
    }
}

impl FilesystemProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }
}

#[async_trait]
impl KnowledgeProvider for FilesystemProvider {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn supported_formats(&self) -> &[&str] {
        &[
            "txt", "md", "markdown", "html", "htm", "yaml", "yml", "json", "xml", "csv", "log",
            "conf", "cfg", "ini", "toml", "sh", "bash", "py", "rs", "go", "java", "js", "ts", "c",
            "cpp", "h", "hpp", "rb", "php", "pl", "sql", "r", "scala", "kt", "swift", "m",
        ]
    }

    fn get_enabled(&self, enabled: bool) -> bool {
        enabled
    }

    async fn discover(&self, path: &str) -> Result<Vec<ProviderDocument>> {
        let scan_dir = Path::new(path);
        if !scan_dir.exists() {
            return Err(anyhow!("Path does not exist: {}", path));
        }
        if !scan_dir.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", path));
        }

        let mut docs = Vec::new();
        let entries = fs::read_dir(scan_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if self.extensions.is_empty() || self.extensions.contains(&ext.to_string()) {
                        match self.parse(&path.to_string_lossy()).await {
                            Ok(doc) => docs.push(doc),
                            Err(e) => {
                                warn!(path = ?path, error = %e, "Failed to parse file");
                            }
                        }
                    }
                }
            }
        }

        debug!(count = docs.len(), path, "Filesystem discovery complete");
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

        let mime_type = Self::guess_mime_type(&extension);

        Ok(ProviderDocument {
            id: uuid::Uuid::new_v4(),
            title,
            source: path.to_string(),
            extension,
            content,
            size_bytes: metadata.len() as usize,
            mime_type,
            author: String::new(),
            modified_at,
        })
    }
}

impl FilesystemProvider {
    fn guess_mime_type(ext: &str) -> String {
        match ext {
            "txt" | "text" => "text/plain".to_string(),
            "md" | "markdown" => "text/markdown".to_string(),
            "html" | "htm" => "text/html".to_string(),
            "json" => "application/json".to_string(),
            "xml" => "application/xml".to_string(),
            "yaml" | "yml" => "text/yaml".to_string(),
            "pdf" => "application/pdf".to_string(),
            "csv" => "text/csv".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
}
