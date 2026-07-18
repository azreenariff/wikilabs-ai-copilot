//! TXT parser.
//!
//! Simple text parser that splits content into paragraphs.

use super::{DocumentElement, ParserProvider};
use super::Document;

/// Text file parser.
pub struct TxtParser;

impl TxtParser {
    pub fn new() -> Self {
        Self
    }

    fn detect_structure(&self, content: &str) -> Vec<DocumentElement> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Skip empty lines
            if line.trim().is_empty() {
                i += 1;
                continue;
            }

            // Detect heading-like lines (all caps, ends with colon, or short lines)
            let trimmed = line.trim();
            let is_heading_candidate = trimmed.len() < 200
                && (trimmed == trimmed.to_uppercase() || trimmed.ends_with(':'));

            if is_heading_candidate {
                elements.push(DocumentElement::Heading(
                    if line.starts_with("## ") || line.starts_with("### ") { 2 } else { 1 },
                    trimmed.to_string(),
                ));
                i += 1;
                continue;
            }

            // Detect code block lines
            if line.starts_with("    ") || line.starts_with("\t") || line.starts_with("$ ") || line.starts_with("# ") {
                let mut code_lines = Vec::new();
                while i < lines.len() {
                    if lines[i].trim().is_empty() || (lines[i].len() > 4 && !lines[i].starts_with("    ") && !lines[i].starts_with("\t")) {
                        break;
                    }
                    code_lines.push(lines[i]);
                    i += 1;
                }
                let code = code_lines.join("\n");
                elements.push(DocumentElement::CodeBlock(String::new(), code));
                continue;
            }

            // Detect warning lines
            if trimmed.to_lowercase().starts_with("warning")
                || trimmed.to_lowercase().starts_with("note:")
                || trimmed.to_lowercase().starts_with("caution:") {
                elements.push(DocumentElement::Warning(trimmed.to_string()));
                i += 1;
                continue;
            }

            // Detect command lines
            if trimmed.starts_with("$ ") || trimmed.starts_with("# ") {
                let cmd = if trimmed.starts_with("$ ") {
                    &trimmed[2..]
                } else {
                    &trimmed[1..]
                };
                elements.push(DocumentElement::Command(cmd.to_string()));
                i += 1;
                continue;
            }

            // Collect paragraph
            let mut para_lines = Vec::new();
            while i < lines.len() && !lines[i].trim().is_empty() {
                para_lines.push(lines[i].trim());
                i += 1;
            }
            if !para_lines.is_empty() {
                elements.push(DocumentElement::Paragraph(para_lines.join(" ")));
            }
        }

        elements
    }
}

impl ParserProvider for TxtParser {
    fn parse(&self, content: &str, author: &str, source: &str) -> Document {
        let elements = self.detect_structure(content);
        let full_text = content.to_string();
        let mut doc = Document::new(full_text, author, source);
        let filename = source.rsplit('/').next().unwrap_or("unknown");
        let extension = source.rsplit('.').next().unwrap_or("txt");
        doc.set_derived(filename, extension);
        doc.elements = elements;
        doc
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["txt".to_string(), "text".to_string()]
    }
}

impl Default for TxtParser {
    fn default() -> Self {
        Self::new()
    }
}