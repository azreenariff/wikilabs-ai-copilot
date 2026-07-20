//! TXT provider — reads plain text files.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::debug;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider for plain text files.
pub struct TxtProvider {
    enabled: bool,
}

impl Default for TxtProvider {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl TxtProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KnowledgeProvider for TxtProvider {
    fn name(&self) -> &str {
        "txt"
    }

    fn supported_formats(&self) -> &[&str] {
        &["txt", "text", "log", "conf", "cfg", "ini", "md", "rst"]
    }

    async fn discover(&self, path: &str) -> Result<Vec<ProviderDocument>> {
        let p = Path::new(path);
        let mut docs = Vec::new();

        if p.is_file() {
            docs.push(self.parse(path).await?);
        } else if p.is_dir() {
            for entry in std::fs::read_dir(p)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_file() {
                    docs.push(self.parse(&entry_path.to_string_lossy()).await?);
                }
            }
        }

        debug!(count = docs.len(), "TXT discovery complete");
        Ok(docs)
    }

    async fn parse(&self, path: &str) -> Result<ProviderDocument> {
        let p = Path::new(path);
        let content =
            fs::read_to_string(p).with_context(|| format!("Failed to read text file: {}", path))?;

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
