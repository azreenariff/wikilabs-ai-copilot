//! PDF provider — parses PDF files using minimal tag extraction.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use std::path::Path;
use tracing::debug;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider that extracts text from PDF files.
#[derive(Default)]
pub struct PdfProvider {
    enabled: bool,
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
            .map(|d| {
                chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                    .unwrap_or_default()
            })
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
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => return format!("[PDF read error: {}]", e),
        };

        // Check for PDF magic number
        if bytes.len() < 5 || &bytes[0..5] != b"%PDF-" {
            return "[File does not appear to be a valid PDF]".to_string();
        }

        let mut result = String::new();

        // Extract PDF metadata from the header trailer
        if let Some(text) = Self::extract_pdf_metadata(&bytes) {
            result.push_str(&text);
        }

        // Extract text content from PDF streams
        let text_content = Self::extract_pdf_stream_text(&bytes);
        if !text_content.is_empty() {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(&text_content);
        }

        if result.is_empty() {
            result = format!("[PDF file: {} ({} bytes)]", path, bytes.len());
        }

        result
    }

    /// Extract metadata from the PDF Info dictionary or trailer.
    fn extract_pdf_metadata(bytes: &[u8]) -> Option<String> {
        let text = String::from_utf8_lossy(bytes);

        // Look for Info dictionary
        if let Some(start) = text.find("/Author") {
            let slice = &text[start..start.saturating_add(200)];
            let title = Self::extract_pdf_value(slice, "/Title");
            let author = Self::extract_pdf_value(slice, "/Author");
            let creator = Self::extract_pdf_value(slice, "/Creator");
            let producer = Self::extract_pdf_value(slice, "/Producer");
            let mut metadata = String::new();
            if !title.is_empty() {
                metadata.push_str(&format!("Title: {}\n", title));
            }
            if !author.is_empty() {
                metadata.push_str(&format!("Author: {}\n", author));
            }
            if !creator.is_empty() {
                metadata.push_str(&format!("Creator: {}\n", creator));
            }
            if !producer.is_empty() {
                metadata.push_str(&format!("Producer: {}\n", producer));
            }
            Some(metadata)
        } else {
            // Try trailer for document info
            let title = Self::extract_pdf_value(&text, "/Title");
            let author = Self::extract_pdf_value(&text, "/Author");
            if !title.is_empty() || !author.is_empty() {
                let mut metadata = String::new();
                if !title.is_empty() {
                    metadata.push_str(&format!("Title: {}\n", title));
                }
                if !author.is_empty() {
                    metadata.push_str(&format!("Author: {}\n", author));
                }
                Some(metadata)
            } else {
                None
            }
        }
    }

    /// Extract a value from PDF dictionary (handles /Key (value) and /Key <hex> forms).
    fn extract_pdf_value(text: &str, key: &str) -> String {
        if let Some(start) = text.find(key) {
            let after_key = &text[start + key.len()..];
            // Check for parenthesized string: (value)
            if let Some(paren_start) = after_key.find('(') {
                let after_paren = &after_key[paren_start + 1..];
                if let Some(paren_end) = after_paren.find(')') {
                    return after_paren[..paren_end].to_string();
                }
            }
            // Check for hex string: <48656C6C6F>
            if let Some(hex_start) = after_key.find('<') {
                let after_hex = &after_key[hex_start + 1..];
                if let Some(hex_end) = after_hex.find('>') {
                    let hex_str = &after_hex[..hex_end];
                    let decoded: String = hex_str
                        .as_bytes()
                        .chunks(2)
                        .map(|pair| {
                            if pair.len() == 2 {
                                if let Ok(byte_val) =
                                    u8::from_str_radix(&String::from_utf8_lossy(pair), 16)
                                {
                                    byte_val as char
                                } else {
                                    '?'
                                }
                            } else {
                                '?'
                            }
                        })
                        .collect();
                    return decoded;
                }
            }
        }
        String::new()
    }

    /// Extract text content from PDF stream objects.
    fn extract_pdf_stream_text(bytes: &[u8]) -> String {
        let mut texts = Vec::new();
        let text = String::from_utf8_lossy(bytes);

        // Find all stream...endstream blocks
        let mut stream_start = 0;
        while let Some(start) = text[stream_start..].find("stream\n") {
            let actual_start = stream_start + start + 7; // "stream\n" length
            stream_start = actual_start;

            if let Some(end) = text[actual_start..].find("\rendstream") {
                let stream_data = &text[actual_start..actual_start + end];

                // Try to decode the stream as text
                let decoded = Self::decode_pdf_stream(stream_data);
                if !decoded.is_empty() {
                    texts.push(decoded);
                }
            } else {
                break;
            }
        }

        texts.join("\n\n")
    }

    /// Decode a PDF stream, handling common encodings.
    fn decode_pdf_stream(stream: &str) -> String {
        // Try UTF-16BE with BOM (common for PDF text)
        if stream.starts_with('\u{feff}') || stream.starts_with('\u{ff}') {
            let s = &stream[2..];
            let bytes: Vec<u8> = s
                .chars()
                .filter(|c| !c.is_control())
                .collect::<String>()
                .into_bytes();
            if bytes.len() >= 2 && bytes.len().is_multiple_of(2) {
                let chars: Vec<char> = bytes
                    .chunks(2)
                    .filter_map(|c| {
                        if c.len() == 2 {
                            Some(u16::from_be_bytes([c[0], c[1]]))
                        } else {
                            None
                        }
                    })
                    .filter_map(|c| char::from_u32(c as u32))
                    .collect();
                if !chars.is_empty() && chars.iter().any(|c| c.is_alphanumeric()) {
                    return chars.into_iter().collect();
                }
            }
        }

        // Fallback: extract readable ASCII/UTF-8 text
        stream
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
