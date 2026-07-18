//! Validation step — check format, size, readability.

use super::discover::DiscoveredDoc;
use crate::pipeline::PipelineConfig;
use std::fs;
use std::io::Read;
use anyhow::Context;
use tracing::debug;

/// Validation result.
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub format: String,
    pub readable: bool,
}

/// The validation pipeline step.
pub struct ValidateStep {
    max_size: u64,
    supported_extensions: Vec<String>,
}

impl ValidateStep {
    pub fn new(max_size: u64, supported_extensions: &[String]) -> Self {
        Self {
            max_size,
            supported_extensions: supported_extensions.to_vec(),
        }
    }

    /// Run the validation step on a discovered document.
    pub fn run(&self, doc: &DiscoveredDoc) -> anyhow::Result<ValidationResult> {
        let mut result = ValidationResult {
            valid: false,
            format: doc.extension.clone(),
            readable: false,
        };

        // Check file size
        if doc.file_size > self.max_size {
            anyhow::bail!(
                "Document exceeds maximum size: {} > {} bytes",
                doc.file_size,
                self.max_size
            );
        }
        debug!(path = ?doc.path, size = doc.file_size, "Size check passed");

        // Check extension is supported
        let ext = format!(".{}", doc.extension);
        if !self.supported_extensions.iter().any(|s| s == &ext) {
            anyhow::bail!("Unsupported file extension: {}", doc.extension);
        }
        debug!(ext = doc.extension, "Extension check passed");

        // Check readability (try to read first bytes)
        match fs::read(&doc.path) {
            Ok(contents) => {
                // For text-based formats, verify we can read them
                if !Self::is_text_format(&doc.extension) {
                    // Binary formats (PDF, DOCX) — just check size > 0
                    if contents.is_empty() {
                        anyhow::bail!("File is empty");
                    }
                    result.readable = true;
                } else {
                    // Text-based formats — verify UTF-8 readability
                    let text = String::from_utf8(contents.clone())
                        .map_err(|e| anyhow::anyhow!("File is not valid UTF-8: {}", e))?;
                    if text.trim().is_empty() {
                        anyhow::bail!("File contains no content");
                    }
                    result.readable = true;
                }
            }
            Err(e) => {
                anyhow::bail!("Failed to read file: {}", e);
            }
        }

        result.valid = true;
        debug!(path = ?doc.path, "Validation passed");
        Ok(result)
    }

    fn is_text_format(ext: &str) -> bool {
        matches!(
            ext,
            "md" | "markdown" | "txt" | "text" | "html" | "htm" | "yaml" | "yml"
                | "json" | "xml"
        )
    }
}