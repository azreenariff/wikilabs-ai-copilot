//! Cleaning step — strip excess whitespace, normalize.

use crate::processing::Document;
use crate::processing::DocumentElement;
use regex::Regex;
use tracing::debug;

/// The clean pipeline step.
pub struct CleanStep;

impl CleanStep {
    pub fn new() -> Self {
        Self
    }

    /// Run the cleaning step on a parsed document.
    pub fn run(&self, doc: Document) -> anyhow::Result<Document> {
        // Clean all text elements
        let mut cleaned_elements = Vec::new();

        for element in doc.elements {
            let cleaned = self.clean_element(element);
            cleaned_elements.push(cleaned);
        }

        // Clean full_text
        let full_text = self.clean_text(&doc.full_text);

        let cleaned_doc = Document {
            full_text,
            elements: cleaned_elements,
            ..doc
        };

        debug!("Cleaning complete");
        Ok(cleaned_doc)
    }

    fn clean_element(&self, element: DocumentElement) -> DocumentElement {
        match element {
            DocumentElement::Heading(level, text) => {
                DocumentElement::Heading(level, self.clean_text(&text))
            }
            DocumentElement::Paragraph(text) => DocumentElement::Paragraph(self.clean_text(&text)),
            DocumentElement::Table(rows) => DocumentElement::Table(
                rows.into_iter()
                    .map(|row| row.into_iter().map(|cell| self.clean_text(&cell)).collect())
                    .collect(),
            ),
            DocumentElement::List(items) => DocumentElement::List(
                items
                    .into_iter()
                    .map(|item| self.clean_text(&item))
                    .collect(),
            ),
            DocumentElement::CodeBlock(lang, code) => {
                DocumentElement::CodeBlock(lang, self.clean_code(&code))
            }
            DocumentElement::Command(text) => DocumentElement::Command(self.clean_text(&text)),
            DocumentElement::Example(text) => DocumentElement::Example(self.clean_text(&text)),
            DocumentElement::Warning(text) => DocumentElement::Warning(self.clean_text(&text)),
            DocumentElement::Reference(text, url) => {
                DocumentElement::Reference(self.clean_text(&text), url)
            }
            DocumentElement::InlineCode(text) => {
                DocumentElement::InlineCode(self.clean_text(&text))
            }
            DocumentElement::Bold(text) => DocumentElement::Bold(self.clean_text(&text)),
        }
    }

    fn clean_text(&self, text: &str) -> String {
        let re = Regex::new(r"[ \t]+").unwrap();
        let mut result = re.replace_all(text, " ").to_string();
        // Collapse multiple newlines to single newline
        let re2 = Regex::new(r"\n{3,}").unwrap();
        result = re2.replace_all(&result, "\n\n").to_string();
        result.trim().to_string()
    }

    fn clean_code(&self, code: &str) -> String {
        // For code blocks, preserve structure but strip leading/trailing blank lines
        let trimmed = code.trim();
        // Collapse internal excessive whitespace (more than 4 spaces/tabs) to 4 spaces
        let re = Regex::new(r"[ \t]{4,}").unwrap();
        re.replace_all(trimmed, "    ").to_string()
    }
}

impl Default for CleanStep {
    fn default() -> Self {
        Self::new()
    }
}
