//! XML parser preserving structure.
//!
//! Parses XML into structured Document elements representing
//! elements, attributes, text content, and code blocks.

use super::Document;
use super::{DocumentElement, ParserProvider};
use tracing::debug;

/// XML parser.
pub struct XmlParser;

impl XmlParser {
    pub fn new() -> Self {
        Self
    }

    fn parse_xml_content(&self, content: &str) -> Vec<DocumentElement> {
        let mut elements = Vec::new();

        // Use regex-based parsing for structure preservation
        // Extract elements with text content (no backreference support in Rust regex)
        let tag_re = regex::Regex::new(r"<([a-zA-Z_][\w.-]*)(\s+[^>]*)?>(.*?)(?=<|$)").unwrap();

        for cap in tag_re.captures_iter(content) {
            let tag_name = cap[1].to_string();
            let text = cap[3].trim();
            // Strip trailing closing tag if present
            let closing = format!("</{}", tag_name);
            let text = if text.ends_with(&closing) {
                &text[..text.len() - closing.len()]
            } else {
                text
            };

            if text.is_empty() {
                continue;
            }

            // Determine heading level based on tag name patterns
            let is_structural = matches!(
                tag_name.as_str(),
                "title"
                    | "header"
                    | "heading"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "section"
                    | "chapter"
                    | "part"
            );

            if is_structural {
                let level = if tag_name.starts_with('h') && tag_name.len() == 2 {
                    tag_name.chars().last().unwrap().to_digit(10).unwrap_or(1)
                } else {
                    1
                };
                elements.push(DocumentElement::Heading(level, text.to_string()));
            } else if tag_name == "code" || tag_name == "pre" {
                elements.push(DocumentElement::CodeBlock(String::new(), text.to_string()));
            } else if tag_name == "item" || tag_name == "entry" || tag_name == "li" {
                elements.push(DocumentElement::Paragraph(text.to_string()));
            } else if tag_name == "note"
                || tag_name == "warning"
                || tag_name == "caution"
                || tag_name == "important"
                || tag_name == "alert"
            {
                elements.push(DocumentElement::Warning(text.to_string()));
            } else if tag_name == "example" || tag_name == "sample" || tag_name == "demo" {
                elements.push(DocumentElement::Example(text.to_string()));
            } else if tag_name == "link" || tag_name == "a" {
                // Try to extract URL from attributes
                let href_re = regex::Regex::new(r#"href=["']([^"']+)["']"#).unwrap();
                let url = if let Some(href_cap) = href_re.captures(&cap[2]) {
                    href_cap[1].to_string()
                } else {
                    String::new()
                };
                elements.push(DocumentElement::Reference(text.to_string(), url));
            } else if tag_name == "table" {
                elements.push(DocumentElement::Paragraph(format!(
                    "<table> {} </table>",
                    text
                )));
            } else {
                elements.push(DocumentElement::Paragraph(format!(
                    "<{}> {} </{}>",
                    tag_name, text, tag_name
                )));
            }
        }

        // Also extract text nodes that are not inside tags
        let text_re = regex::Regex::new(r"[^\s<>\n]+").unwrap();
        for cap in text_re.captures_iter(content) {
            let text = cap[0].trim();
            // Skip if this looks like a tag name or value
            if text.starts_with('<') || text.starts_with('/') || text.starts_with('"') {
                continue;
            }
            if text.is_empty() || text.len() < 2 {
                continue;
            }
            // Only add as paragraph if not already covered by tag parsing
            if !elements.iter().any(|e| {
                if let DocumentElement::Paragraph(p) = e {
                    p.contains(text)
                } else {
                    false
                }
            }) {
                elements.push(DocumentElement::Paragraph(text.to_string()));
            }
        }

        elements
    }
}

impl ParserProvider for XmlParser {
    fn parse(&self, content: &str, author: &str, source: &str) -> Document {
        let mut elements = self.parse_xml_content(content);

        // Also try to extract XML structure for code blocks
        let code_re = regex::Regex::new(r"<code[^>]*>(.*?)</code>").unwrap();
        for cap in code_re.captures_iter(content) {
            if let Some(text) = cap.get(1) {
                elements.push(DocumentElement::CodeBlock(
                    String::new(),
                    text.as_str().to_string(),
                ));
            }
        }

        let full_text = content.to_string();
        let mut doc = Document::new(full_text, author, source);
        let filename = source.rsplit('/').next().unwrap_or("unknown");
        let extension = source.rsplit('.').next().unwrap_or("xml");
        doc.set_derived(filename, extension);
        doc.elements = elements;
        doc
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["xml".to_string()]
    }
}

impl Default for XmlParser {
    fn default() -> Self {
        Self::new()
    }
}
