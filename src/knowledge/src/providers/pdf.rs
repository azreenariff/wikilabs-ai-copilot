//! PDF provider — parses PDF files using minimal tag extraction.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::debug;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider that extracts text from PDF files.
pub struct PdfProvider {
    enabled: bool,
}

impl Default for PdfProvider {
    fn default() -> Self {
        Self { enabled: false }
    }
}

impl PdfProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KnowledgeProvider for PdfProvider {
    fn name(&self) -> &str {
        "pdf"
    }

    fn supported_formats(&self) -> &[&str] {
        &["pdf"]
    }

    async fn discover(&self, path: &str) -> Result<Vec<ProviderDocument>> {
        let mut docs = Vec::new();
        let p = Path::new(path);
        if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("pdf") {
            docs.push(self.parse(path).await?);
        } else if p.is_dir() {
            for entry in std::fs::read_dir(p)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                    docs.push(self.parse(&path.to_string_lossy()).await?);
                }
            }
        }
        debug!(count = docs.len(), "PDF discovery complete");
        Ok(docs)
    }

    async fn parse(&self, path: &str) -> Result<ProviderDocument> {
        let p = Path::new(path);
        if !p.exists() {
            anyhow::bail!("PDF file not found: {}", path);
        }

        let metadata = fs::metadata(p)?;
        let extension = p
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

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

        let content = Self::extract_pdf_text(path);

        debug!(path, title, "PDF parsed");
        Ok(ProviderDocument {
            id: uuid::Uuid::new_v4(),
            title,
            source: path.to_string(),
            extension,
            content,
            size_bytes: metadata.len() as usize,
            mime_type: "application/pdf".to_string(),
            author: String::new(),
            modified_at,
        })
    }
}

impl PdfProvider {
    fn extract_pdf_text(path: &str) -> String {
        // TODO: Implement actual PDF text extraction
        // For now, return a placeholder
        format!("[PDF content not extracted: {}]", path)
    }
}

