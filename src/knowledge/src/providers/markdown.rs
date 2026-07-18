//! Markdown provider — parses Markdown files with front-matter support.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::debug;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider that parses Markdown files, extracting front-matter metadata.
pub struct MarkdownProvider {
    enabled: bool,
}

impl Default for MarkdownProvider {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl MarkdownProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KnowledgeProvider for MarkdownProvider {
    fn name(&self) -> &str {
        "markdown"
    }

    fn supported_formats(&self) -> &[&str] {
        &["md", "markdown"]
    }

    async fn discover(&self, path: &str) -> Result<Vec<ProviderDocument>> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path));
        }

        let mut docs = Vec::new();
        if p.is_file() {
            if p.extension().and_then(|e| e.to_str()) == Some("md") || 
               p.extension().and_then(|e| e.to_str()) == Some("markdown") {
                docs.push(self.parse(&p.to_string_lossy()).await?);
            }
        } else if p.is_dir() {
            for entry in std::fs::read_dir(p)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let ext = path
                        .extension()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if ext == "md" || ext == "markdown" {
                        docs.push(self.parse(&path.to_string_lossy()).await?);
                    }
                }
            }
        }

        debug!(count = docs.len(), "Markdown discovery complete");
        Ok(docs)
    }

    async fn parse(&self, path: &str) -> Result<ProviderDocument> {
        let p = Path::new(path);
        let content = fs::read_to_string(p)
            .with_context(|| format!("Failed to read Markdown file: {}", path))?;

        let metadata = fs::metadata(p)
            .with_context(|| format!("Failed to read metadata: {}", path))?;

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| Utc::timestamp_opt(d.as_secs() as i64, 0))
            .unwrap_or_else(Utc::now);

        let title = p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = p
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        // Try to extract front-matter
        let mut author = String::new();
        let mut title_from_fm = title.clone();
        if let Some(fm) = content.strip_prefix("---\n") {
            if let Some(end) = fm.find("---\n") {
                let fm_content = &fm[..end];
                for line in fm_content.lines() {
                    if let Some(("author", val)) = line.split_once(':') {
                        author = val.trim().to_string();
                    } else if let Some(("title", val)) = line.split_once(':') {
                        title_from_fm = val.trim().to_string();
                    }
                }
            }
        }

        Ok(ProviderDocument {
            id: uuid::Uuid::new_v4(),
            title: title_from_fm,
            source: path.to_string(),
            extension,
            content,
            size_bytes: metadata.len() as usize,
            mime_type: "text/markdown".to_string(),
            author,
            modified_at,
        })
    }
}

