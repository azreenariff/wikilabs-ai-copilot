//! JSON parser preserving structure.
//!
//! Parses JSON into structured Document elements representing
//! objects, arrays, key-value pairs, and code blocks.

use super::Document;
use super::{DocumentElement, ParserProvider};
use regex::Regex;
use serde_json::Value;
use tracing::debug;

/// JSON parser.
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }

    fn convert_value(&self, value: &Value, path: &str, level: u32) -> Vec<DocumentElement> {
        let mut elements = Vec::new();

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };

                    match val {
                        Value::String(s) => {
                            elements
                                .push(DocumentElement::Paragraph(format!("{}: {}", new_path, s)));
                        }
                        Value::Number(n) => {
                            elements
                                .push(DocumentElement::Paragraph(format!("{}: {}", new_path, n)));
                        }
                        Value::Bool(b) => {
                            elements
                                .push(DocumentElement::Paragraph(format!("{}: {}", new_path, b)));
                        }
                        Value::Null => {
                            elements
                                .push(DocumentElement::Paragraph(format!("{}: (null)", new_path)));
                        }
                        Value::Array(arr) => {
                            if arr.len() <= 10 {
                                // Small array, listify
                                let items: Vec<String> = arr
                                    .iter()
                                    .enumerate()
                                    .map(|(i, v)| format!("- [{}] {}", i, v))
                                    .collect();
                                elements.push(DocumentElement::List(items));
                            } else {
                                // Large array, heading + items
                                elements.push(DocumentElement::Heading(
                                    level + 1,
                                    format!("{} (array, {} items)", new_path, arr.len()),
                                ));
                                let sub = self.convert_value(val, &new_path, level + 2);
                                elements.extend(sub);
                            }
                        }
                        Value::Object(_) => {
                            elements.push(DocumentElement::Heading(level + 1, key.clone()));
                            let sub = self.convert_value(val, &new_path, level + 2);
                            elements.extend(sub);
                        }
                    }
                }
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    elements.push(DocumentElement::Paragraph(format!("{}: []", path)));
                } else {
                    let items: Vec<String> = arr
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            if let Value::String(s) = v {
                                format!("- {} [{}]: {}", path, i, s)
                            } else {
                                format!("- {} [{}]: {}", path, i, v)
                            }
                        })
                        .collect();
                    elements.push(DocumentElement::List(items));
                }
            }
            Value::String(s) => {
                if s.contains('\n') {
                    elements.push(DocumentElement::CodeBlock("json".to_string(), s.clone()));
                } else {
                    elements.push(DocumentElement::Paragraph(s.clone()));
                }
            }
            Value::Number(n) => {
                elements.push(DocumentElement::Paragraph(format!("{}", n)));
            }
            Value::Bool(b) => {
                elements.push(DocumentElement::Paragraph(format!("{}", b)));
            }
            Value::Null => {
                elements.push(DocumentElement::Paragraph("null".to_string()));
            }
        }

        elements
    }
}

impl ParserProvider for JsonParser {
    fn parse(&self, content: &str, author: &str, source: &str) -> Document {
        let value: Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                debug!(error = %e, "JSON parse failed, returning as paragraph");
                let mut doc = Document::new(content, author, source);
                doc.elements = vec![DocumentElement::Paragraph(content.to_string())];
                return doc;
            }
        };

        let mut elements = self.convert_value(&value, "", 1);

        // Extract string values that look like examples
        let re = regex::Regex::new(r"(?i)(example|sample|demo):?\s*(.+)").unwrap();
        for cap in re.captures_iter(content) {
            if let Some(text_cap) = cap.get(2) {
                if text_cap.as_str().len() < 500 {
                    elements.push(DocumentElement::Example(text_cap.as_str().to_string()));
                }
            }
        }

        let full_text = content.to_string();
        let mut doc = Document::new(full_text, author, source);
        let filename = source.rsplit('/').next().unwrap_or("unknown");
        let extension = source.rsplit('.').next().unwrap_or("json");
        doc.set_derived(filename, extension);
        doc.elements = elements;
        doc
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["json".to_string()]
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}
