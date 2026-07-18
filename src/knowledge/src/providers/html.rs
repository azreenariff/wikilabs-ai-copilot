//! HTML provider — parses HTML files, extracting body content and meta tags.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::debug;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider that parses HTML files, extracting visible text content.
pub struct HtmlProvider {
    enabled: bool,
}

impl Default for HtmlProvider {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl HtmlProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KnowledgeProvider for HtmlProvider {
    fn name(&self) -> &str {
        "html"
    }

    fn supported_formats(&self) -> &[&str] {
        &["html", "htm"]
    }

    async fn discover(&self, path: &str) -> Result<Vec<ProviderDocument>> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path));
        }

        let mut docs = Vec::new();
        if p.is_file() {
            if p.extension().and_then(|e| e.to_str()) == Some("html") || 
               p.extension().and_then(|e| e.to_str()) == Some("htm") {
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
                    if ext == "html" || ext == "htm" {
                        docs.push(self.parse(&path.to_string_lossy()).await?);
                    }
                }
            }
        }

        debug!(count = docs.len(), "HTML discovery complete");
        Ok(docs)
    }

    async fn parse(&self, path: &str) -> Result<ProviderDocument> {
        let p = Path::new(path);
        let content = fs::read_to_string(p)
            .with_context(|| format!("Failed to read HTML file: {}", path))?;

        let metadata = fs::metadata(p)
            .with_context(|| format!("Failed to read metadata: {}", path))?;

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| Utc::timestamp_opt(d.as_secs() as i64, 0))
            .unwrap_or_else(Utc::now);

        // Extract title from HTML
        let title = self.extract_title(&content);

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
            mime_type: "text/html".to_string(),
            author: String::new(),
            modified_at,
        })
    }
}

impl HtmlProvider {
    fn extract_title(&self, html: &str) -> String {
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start..].find("</title>") {
                return html[start + 7..start + end].trim().to_string();
            }
        }
        "Unknown".to_string()
    }
}

