//! YAML parser preserving structure.
//!
//! Parses YAML into structured Document elements representing
//! key-value pairs, lists, nested structures, and code blocks.

use super::Document;
use super::{DocumentElement, ParserProvider};
use regex::Regex;
use serde_yaml::Value;
use tracing::debug;

/// YAML parser.
pub struct YamlParser;

impl YamlParser {
    pub fn new() -> Self {
        Self
    }

    fn convert_value(&self, value: &Value, path: &str, level: u32) -> Vec<DocumentElement> {
        let mut elements = Vec::new();

        match value {
            Value::Mapping(map) => {
                for (key, val) in map {
                    let key_str = key.as_str().unwrap_or("");
                    let new_path = if path.is_empty() {
                        key_str.to_string()
                    } else {
                        format!("{}.{}", path, key_str)
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
                        Value::Sequence(seq) => {
                            elements.push(DocumentElement::Heading(level + 1, key_str.to_string()));
                            for (idx, item) in seq.iter().enumerate() {
                                if let Value::String(s) = item {
                                    elements.push(DocumentElement::Paragraph(format!(
                                        "- {} [{}]: {}",
                                        new_path, idx, s
                                    )));
                                } else {
                                    let sub = self.convert_value(
                                        item,
                                        &format!("{}.{}", new_path, idx),
                                        level + 2,
                                    );
                                    elements.extend(sub);
                                }
                            }
                        }
                        Value::Mapping(_) => {
                            elements.push(DocumentElement::Heading(level + 1, key_str.to_string()));
                            let sub = self.convert_value(val, &new_path, level + 2);
                            elements.extend(sub);
                        }
                        Value::Tagged(_) => {
                            elements.push(DocumentElement::Paragraph(format!(
                                "{}: (tagged value)",
                                new_path
                            )));
                        }
                    }
                }
            }
            Value::Sequence(seq) => {
                let mut items = Vec::new();
                for (idx, item) in seq.iter().enumerate() {
                    if let Value::String(s) = item {
                        items.push(format!("- {} [{}]: {}", path, idx, s));
                    } else {
                        let converted =
                            self.convert_value(item, &format!("{}.{}", path, idx), level + 1);
                        for el in converted {
                            if let DocumentElement::Paragraph(text) = el {
                                items.push(text);
                            }
                        }
                    }
                }
                if !items.is_empty() {
                    elements.push(DocumentElement::List(items));
                }
            }
            Value::String(s) => {
                if s.contains('\n') {
                    // Multi-line string as code block
                    elements.push(DocumentElement::CodeBlock("yaml".to_string(), s.clone()));
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
            _ => {
                elements.push(DocumentElement::Paragraph(format!("{:?}", value)));
            }
        }

        elements
    }
}

impl ParserProvider for YamlParser {
    fn parse(&self, content: &str, author: &str, source: &str) -> Document {
        let value: Value = match serde_yaml::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                debug!(error = %e, "YAML parse failed, returning as paragraph");
                let mut doc = Document::new(content, author, source);
                doc.elements = vec![DocumentElement::Paragraph(content.to_string())];
                return doc;
            }
        };

        let mut elements = self.convert_value(&value, "", 1);

        // Add code blocks for inline code examples
        let code_blocks_re = regex::Regex::new(r"```(\w*)\s*\n(.*?)```").unwrap();
        for cap in code_blocks_re.captures_iter(content) {
            let lang = cap[1].to_string();
            let code = cap[2].to_string();
            elements.push(DocumentElement::CodeBlock(lang, code));
        }

        let full_text = content.to_string();
        let mut doc = Document::new(full_text, author, source);
        let filename = source.rsplit('/').next().unwrap_or("unknown");
        let extension = source.rsplit('.').next().unwrap_or("yaml");
        doc.set_derived(filename, extension);
        doc.elements = elements;
        doc
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["yaml".to_string(), "yml".to_string()]
    }
}

impl Default for YamlParser {
    fn default() -> Self {
        Self::new()
    }
}
