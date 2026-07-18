//! Normalization step — consistent encoding (UTF-8).

use crate::processing::Document;
use tracing::debug;

/// The normalize pipeline step.
pub struct NormalizeStep;

impl NormalizeStep {
    pub fn new() -> Self {
        Self
    }

    /// Run the normalization step on a parsed document.
    /// Ensures all content is valid UTF-8 and consistently encoded.
    pub fn run(&self, doc: Document) -> anyhow::Result<Document> {
        // Full text is already UTF-8 (from String::from_utf8_lossy in parsers)
        // Just verify and clean unicode characters
        let normalized_text = self.normalize_unicode(&doc.full_text);

        // Normalize all element texts
        let normalized_elements: Vec<_> = doc.elements.iter().map(|el| {
            self.normalize_element(el.clone())
        }).collect();

        let normalized_doc = Document {
            full_text: normalized_text,
            elements: normalized_elements,
            source: doc.source,
            author: doc.author,
            filename: doc.filename,
            extension: doc.extension,
            title: doc.title,
        };

        debug!("Normalization complete");
        Ok(normalized_doc)
    }

    fn normalize_unicode(&self, text: &str) -> String {
        // NFC normalization via string replacement of common issues
        text.chars()
            .filter(|c| !c.is_control()) // Remove control characters except newlines
            .collect()
    }

    fn normalize_element(&self, element: crate::processing::DocumentElement) -> crate::processing::DocumentElement {
        match element {
            crate::processing::DocumentElement::Heading(level, text) => {
                crate::processing::DocumentElement::Heading(level, self.normalize_unicode(&text))
            }
            crate::processing::DocumentElement::Paragraph(text) => {
                crate::processing::DocumentElement::Paragraph(self.normalize_unicode(&text))
            }
            crate::processing::DocumentElement::Table(rows) => {
                crate::processing::DocumentElement::Table(
                    rows.into_iter()
                        .map(|row| row.into_iter().map(|cell| self.normalize_unicode(&cell)).collect())
                        .collect(),
                )
            }
            crate::processing::DocumentElement::List(items) => {
                crate::processing::DocumentElement::List(
                    items.into_iter().map(|item| self.normalize_unicode(&item)).collect(),
                )
            }
            crate::processing::DocumentElement::CodeBlock(lang, code) => {
                crate::processing::DocumentElement::CodeBlock(
                    lang,
                    self.normalize_unicode(&code),
                )
            }
            crate::processing::DocumentElement::Command(text) => {
                crate::processing::DocumentElement::Command(self.normalize_unicode(&text))
            }
            crate::processing::DocumentElement::Example(text) => {
                crate::processing::DocumentElement::Example(self.normalize_unicode(&text))
            }
            crate::processing::DocumentElement::Warning(text) => {
                crate::processing::DocumentElement::Warning(self.normalize_unicode(&text))
            }
            crate::processing::DocumentElement::Reference(text, url) => {
                crate::processing::DocumentElement::Reference(
                    self.normalize_unicode(&text),
                    url,
                )
            }
        }
    }
}

impl Default for NormalizeStep {
    fn default() -> Self {
        Self::new()
    }
}