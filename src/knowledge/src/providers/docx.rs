//! DOCX provider — parses Word documents.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::debug;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider for DOCX files.
pub struct DocxProvider {
    enabled: bool,
}

impl Default for DocxProvider {
    fn default() -> Self {
        Self { enabled: false }
    }
}

impl DocxProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KnowledgeProvider for DocxProvider {
    fn name(&self) -> &str {
        "docx"
    }

    fn supported_formats(&self) -> &[&str] {
        &["docx"]
    }

    async fn discover(&self, path: &str) -> Result<Vec<ProviderDocument>> {
        let mut docs = Vec::new();
        let p = Path::new(path);
        if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("docx") {
            docs.push(self.parse(path).await?);
        } else if p.is_dir() {
            for entry in std::fs::read_dir(p)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("docx") {
                    docs.push(self.parse(&path.to_string_lossy()).await?);
                }
            }
        }
        debug!(count = docs.len(), "DOCX discovery complete");
        Ok(docs)
    }

    async fn parse(&self, path: &str) -> Result<ProviderDocument> {
        let p = Path::new(path);
        if !p.exists() {
            anyhow::bail!("DOCX file not found: {}", path);
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
            .map(|d| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                    .unwrap_or_default()
            })
            .unwrap_or_else(Utc::now);

        let title = p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let content = Self::extract_docx_text(path);

        debug!(path, title, "DOCX parsed");
        Ok(ProviderDocument {
            id: uuid::Uuid::new_v4(),
            title,
            source: path.to_string(),
            extension,
            content,
            size_bytes: metadata.len() as usize,
            mime_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                .to_string(),
            author: String::new(),
            modified_at,
        })
    }
}

impl DocxProvider {
    fn extract_docx_text(path: &str) -> String {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => return format!("[DOCX read error: {}]", e),
        };

        // DOCX files are ZIP archives starting with PK magic bytes
        if bytes.len() < 4 || &bytes[0..2] != b"PK" {
            return "[File does not appear to be a valid DOCX]".to_string();
        }

        // DOCX is a ZIP archive containing document.xml with the text content.
        // We'll extract text by finding <w:t> tags (Word processing text elements)
        let text = Self::extract_text_from_docx_xml(&bytes);

        // Also try to extract metadata from [Content_Types].xml and docProps/app.xml
        let title = Self::extract_docx_title(&bytes);
        let author = Self::extract_docx_author(&bytes);

        let mut result = String::new();

        if !title.is_empty() {
            result.push_str(&format!("Title: {}\n", title));
        }
        if !author.is_empty() {
            result.push_str(&format!("Author: {}\n", author));
        }

        if !text.is_empty() {
            if !result.is_empty() {
                result.push_str("\n");
            }
            result.push_str(&text);
        }

        if result.is_empty() {
            result = format!("[DOCX file: {} ({} bytes)]", path, bytes.len());
        }

        result
    }

    /// Extract body text from DOCX document.xml by parsing <w:t> tags.
    fn extract_text_from_docx_xml(bytes: &[u8]) -> String {
        let text = String::from_utf8_lossy(bytes);

        // Find document.xml path and extract content
        // DOCX structure: [Content_Types].xml, word/document.xml
        let xml_start = if let Some(pos) = text.find("word/document.xml") {
            // Look for the actual XML content after the path reference
            // In a ZIP, the actual content follows the filename entries
            pos
        } else {
            return String::new();
        };

        // Extract text between <w:t> and </w:t> tags
        let mut result = String::new();
        let t_re = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
        let mut last_end = 0;

        for cap in t_re.captures_iter(&text[xml_start..]) {
            let start_in_slice = cap.get(0).unwrap().start();
            // Find newline between last result and this match to preserve structure
            if last_end > 0 {
                let between = &text[xml_start + last_end..xml_start + start_in_slice];
                if between.contains('\n') || between.contains('\r') {
                    result.push('\n');
                }
            }
            let text_content = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !text_content.trim().is_empty() {
                result.push_str(text_content.trim());
            }
            last_end = cap.get(0).unwrap().end();
        }

        result
    }

    /// Extract document title from DOCX properties.
    fn extract_docx_title(bytes: &[u8]) -> String {
        let text = String::from_utf8_lossy(bytes);

        // Try docProps/app.xml first
        if let Some(app_start) = text.find("docProps/app.xml") {
            if let Some(title_re) = regex::Regex::new(r"<Title>([^<]*)</Title>").ok() {
                if let Some(cap) =
                    title_re.captures(&text[app_start..app_start.saturating_add(500)])
                {
                    let title = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                    if !title.is_empty() {
                        return title;
                    }
                }
            }
        }

        // Try docProps/core.xml (dc:title)
        if let Some(core_start) = text.find("docProps/core.xml") {
            if let Some(title_re) = regex::Regex::new(r"<dc:title[^>]*>([^<]*)</dc:title>").ok() {
                if let Some(cap) =
                    title_re.captures(&text[core_start..core_start.saturating_add(1000)])
                {
                    let title = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                    if !title.is_empty() {
                        return title;
                    }
                }
            }
        }

        String::new()
    }

    /// Extract document author from DOCX properties.
    fn extract_docx_author(bytes: &[u8]) -> String {
        let text = String::from_utf8_lossy(bytes);

        // Try docProps/app.xml (LastAuthor)
        if let Some(app_start) = text.find("docProps/app.xml") {
            if let Some(re) = regex::Regex::new(r"<LastAuthor>([^<]*)</LastAuthor>").ok() {
                if let Some(cap) = re.captures(&text[app_start..app_start.saturating_add(500)]) {
                    let author = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                    if !author.is_empty() {
                        return author;
                    }
                }
            }
        }

        // Try docProps/core.xml (dc:creator)
        if let Some(core_start) = text.find("docProps/core.xml") {
            if let Some(re) = regex::Regex::new(r"<dc:creator[^>]*>([^<]*)</dc:creator>").ok() {
                if let Some(cap) = re.captures(&text[core_start..core_start.saturating_add(1000)]) {
                    let author = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                    if !author.is_empty() {
                        return author;
                    }
                }
            }
        }

        String::new()
    }
}
