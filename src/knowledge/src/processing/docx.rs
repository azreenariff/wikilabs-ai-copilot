//! DOCX parser preserving structure.
//!
//! Parses DOCX files using the ZIP + XML extraction approach.
//! DOCX files are ZIP archives containing XML; we extract text and
//! preserve structural elements.

use super::Document;
use super::{DocumentElement, ParserProvider};
use tracing::debug;

/// DOCX parser.
pub struct DocxParser;

impl DocxParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse DOCX bytes into a Document.
    pub fn parse_docx_bytes(&self, bytes: &[u8], author: &str, source: &str) -> Document {
        // Check if this looks like a DOCX (ZIP magic bytes)
        if bytes.len() >= 2 && bytes[0] == 0x50 && bytes[1] == 0x4B {
            // It's a ZIP file (DOCX)
            if let Some(text) = self.extract_text_from_docx(bytes) {
                let elements = self.extract_structure_from_text(&text);
                let mut doc = Document::new(text, author, source);
                let filename = source.rsplit('/').next().unwrap_or("unknown");
                let extension = source.rsplit('.').next().unwrap_or("docx");
                doc.set_derived(filename, extension);
                doc.elements = elements;
                return doc;
            }
        }

        // Fallback: treat as plain text
        let text = String::from_utf8_lossy(bytes).to_string();
        let elements = self.extract_structure_from_text(&text);
        let mut doc = Document::new(text, author, source);
        let filename = source.rsplit('/').next().unwrap_or("unknown");
        let extension = source.rsplit('.').next().unwrap_or("docx");
        doc.set_derived(filename, extension);
        doc.elements = elements;
        doc
    }

    /// Extract text from DOCX bytes using simple ZIP text extraction.
    fn extract_text_from_docx(&self, bytes: &[u8]) -> Option<String> {
        let text = String::from_utf8_lossy(bytes);

        let t_re = regex::Regex::new(r"<w:t[^>]*>(.*?)</w:t>").unwrap();
        let mut extracted = Vec::new();
        for cap in t_re.captures_iter(&text) {
            let t_text = cap[1].to_string();
            if !t_text.trim().is_empty() {
                extracted.push(t_text);
            }
        }

        if !extracted.is_empty() {
            Some(extracted.join("\n"))
        } else {
            None
        }
    }

    fn extract_structure_from_text(&self, text: &str) -> Vec<DocumentElement> {
        let mut elements = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.len() < 100 && (trimmed == trimmed.to_uppercase() || trimmed.ends_with(':'))
            {
                elements.push(DocumentElement::Heading(1, trimmed.to_string()));
                continue;
            }

            if trimmed.starts_with("$ ") || trimmed.starts_with("# ") {
                let cmd = if let Some(stripped) = trimmed.strip_prefix("$ ") {
                    stripped
                } else {
                    &trimmed[1..]
                };
                elements.push(DocumentElement::Command(cmd.to_string()));
                continue;
            }

            elements.push(DocumentElement::Paragraph(trimmed.to_string()));
        }

        elements
    }
}

impl ParserProvider for DocxParser {
    fn parse(&self, content: &str, author: &str, source: &str) -> Document {
        let bytes = content.as_bytes();
        let doc = self.parse_docx_bytes(bytes, author, source);
        debug!(source, "DOCX parsing complete");
        doc
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["docx".to_string()]
    }
}

impl Default for DocxParser {
    fn default() -> Self {
        Self::new()
    }
}