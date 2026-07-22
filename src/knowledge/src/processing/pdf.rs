//! PDF parser preserving structure.
//!
//! Uses text extraction, preserving headings, paragraphs, tables,
//! code blocks, and other structure from extracted text.

use super::{Document, DocumentElement, ParserProvider};
use regex::Regex;

/// PDF parser using text extraction from raw bytes.
pub struct PdfParser;

impl PdfParser {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_structure(&self, text: &str) -> Vec<DocumentElement> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Empty line
            if line.is_empty() {
                i += 1;
                continue;
            }

            // Heading detection: short, uppercase, or starts with section numbers
            let is_heading = self.is_heading(line, lines.clone(), i);
            if is_heading {
                elements.push(DocumentElement::Heading(
                    self.estimate_heading_level(line),
                    line.to_string(),
                ));
                i += 1;
                continue;
            }

            // Table detection: lines with multiple pipe-separated or tab-separated values
            if line.contains('|') && line.matches('|').count() >= 2 {
                if let Some(table) = self.parse_pipe_table(&lines, &mut i) {
                    elements.push(DocumentElement::Table(table));
                    continue;
                }
            }

            // Code block detection: indented or monospace patterns
            if self.is_code_block_line(line)
                || (i + 1 < lines.len() && self.is_code_block_line(lines[i + 1]))
            {
                let mut code_lines = Vec::new();
                while i < lines.len() && !lines[i].trim().is_empty() {
                    code_lines.push(lines[i]);
                    i += 1;
                }
                let code = code_lines.join("\n");
                elements.push(DocumentElement::CodeBlock(String::new(), code));
                continue;
            }

            // Warning / Alert detection
            let lower = line.to_lowercase();
            if lower.contains("warning")
                || lower.contains("caution")
                || lower.contains("important:")
            {
                elements.push(DocumentElement::Warning(line.to_string()));
                i += 1;
                continue;
            }

            // Example detection
            if line.starts_with("> ")
                || line.starts_with("Example:")
                || lower.starts_with("sample:")
            {
                let text = if line.starts_with("> ") {
                    &line[2..]
                } else {
                    line
                };
                elements.push(DocumentElement::Example(text.to_string()));
                i += 1;
                continue;
            }

            // Command detection
            if line.starts_with("$ ") || line.starts_with("# ") || line.starts_with("% ") {
                let cmd = if line.starts_with("$ ") || line.starts_with("# ") {
                    &line[2..]
                } else {
                    &line[1..]
                };
                elements.push(DocumentElement::Command(cmd.to_string()));
                i += 1;
                continue;
            }

            // Paragraph — collect consecutive lines
            let mut para_lines = Vec::new();
            while i < lines.len() && !lines[i].trim().is_empty() {
                let current = lines[i].trim();
                if self.is_heading(current, lines.clone(), i) {
                    break;
                }
                para_lines.push(current);
                i += 1;
            }
            if !para_lines.is_empty() {
                let para = para_lines.join(" ");
                elements.push(DocumentElement::Paragraph(para));
            }
        }

        elements
    }

    fn is_heading(&self, line: &str, lines: Vec<&str>, idx: usize) -> bool {
        let _ = lines;
        let _ = idx;
        let trimmed = line.trim();
        let len = trimmed.len();

        if len == 0 || len > 200 {
            return false;
        }

        // Check for section number patterns
        let section_re = regex::Regex::new(r"^(\d+\.){0,5}\d+\s+[A-Z]").unwrap();
        if section_re.is_match(trimmed) {
            return true;
        }

        // All caps short line (potential heading)
        if len < 100
            && trimmed
                .chars()
                .all(|c| c.is_uppercase() || c.is_whitespace() || c == '-' || c == '.' || c == '_')
            && trimmed.chars().filter(|c| c.is_alphabetic()).count() > 2
        {
            return true;
        }

        // Line with colon at end and short length
        if len < 80
            && trimmed.ends_with(':')
            && trimmed.chars().filter(|c| c.is_alphabetic()).count() < 50
        {
            return true;
        }

        false
    }

    fn estimate_heading_level(&self, line: &str) -> u32 {
        let trimmed = line.trim();
        let len = trimmed.len();

        // Check for level-1 indicators
        if len < 50 && self.is_heading(trimmed, vec![], 0) {
            // Very short heading → likely level 1
            return 1;
        }

        // Check for numbered sections
        let section_re = regex::Regex::new(r"^(\d+)\.([A-Z])").unwrap();
        if section_re.is_match(trimmed) {
            let caps: Vec<u32> = section_re
                .captures(trimmed)
                .map(|c| {
                    let num = c[1].len();
                    if num == 0 {
                        1
                    } else {
                        num as u32
                    }
                })
                .into_iter()
                .collect();
            return caps.first().copied().unwrap_or(2);
        }

        2
    }

    fn is_code_block_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("    ")
            || trimmed.starts_with("\t")
            || trimmed.starts_with("```")
            || trimmed.starts_with("$ ")
    }

    fn parse_pipe_table(&self, lines: &[&str], i: &mut usize) -> Option<Vec<Vec<String>>> {
        let mut table = Vec::new();

        while *i < lines.len() {
            let line = lines[*i].trim();

            // Skip separator lines
            if line
                .chars()
                .filter(|c| *c == '|' || *c == '-' || *c == ':')
                .count()
                > line.len() / 2
            {
                *i += 1;
                continue;
            }

            if !line.starts_with('|') {
                break;
            }

            let cells: Vec<String> = line
                .split('|')
                .filter(|c| !c.trim().is_empty())
                .map(|c| c.trim().to_string())
                .collect();

            if !cells.is_empty() {
                table.push(cells);
            }
            *i += 1;
        }

        if !table.is_empty() {
            Some(table)
        } else {
            None
        }
    }
}

impl ParserProvider for PdfParser {
    fn parse(&self, content: &str, author: &str, source: &str) -> Document {
        let elements = self.extract_structure(content);
        let full_text = content.to_string();
        let mut doc = Document::new(full_text, author, source);
        let filename = source.rsplit('/').next().unwrap_or("unknown");
        let extension = source.rsplit('.').next().unwrap_or("pdf");
        doc.set_derived(filename, extension);
        doc.elements = elements;
        doc
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["pdf".to_string()]
    }
}

impl Default for PdfParser {
    fn default() -> Self {
        Self::new()
    }
}
