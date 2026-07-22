//! Metadata extraction step — title, headings, tables, code blocks.

use crate::processing::Document;
use crate::processing::DocumentElement;
use tracing::debug;

/// Extracted metadata from a document.
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub headings: Vec<(u32, String)>,
    pub table_count: usize,
    pub code_block_count: usize,
    pub list_count: usize,
    pub command_count: usize,
    pub example_count: usize,
    pub warning_count: usize,
    pub reference_count: usize,
    pub language: String,
    pub word_count: usize,
    pub char_count: usize,
    pub estimated_reading_time: u64,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentMetadata {
    pub fn new() -> Self {
        Self {
            headings: Vec::new(),
            table_count: 0,
            code_block_count: 0,
            list_count: 0,
            command_count: 0,
            example_count: 0,
            warning_count: 0,
            reference_count: 0,
            language: String::new(),
            word_count: 0,
            char_count: 0,
            estimated_reading_time: 0,
        }
    }
}

/// The metadata extraction pipeline step.
pub struct MetadataExtractStep;

impl MetadataExtractStep {
    pub fn new() -> Self {
        Self
    }

    /// Run the metadata extraction step on a parsed document.
    pub fn run(&self, doc: &mut Document) {
        let mut metadata = DocumentMetadata::new();

        for element in &doc.elements {
            match element {
                DocumentElement::Heading(level, text) => {
                    metadata.headings.push((*level, text.clone()));
                }
                DocumentElement::Table(_) => {
                    metadata.table_count += 1;
                }
                DocumentElement::CodeBlock(_, _) => {
                    metadata.code_block_count += 1;
                }
                DocumentElement::List(_) => {
                    metadata.list_count += 1;
                }
                DocumentElement::Command(_) => {
                    metadata.command_count += 1;
                }
                DocumentElement::Example(_) => {
                    metadata.example_count += 1;
                }
                DocumentElement::Warning(_) => {
                    metadata.warning_count += 1;
                }
                DocumentElement::Reference(_, _) => {
                    metadata.reference_count += 1;
                }
                DocumentElement::Paragraph(text) => {
                    metadata.word_count += text.split_whitespace().count();
                    metadata.char_count += text.len();
                }
                DocumentElement::InlineCode(_) | DocumentElement::Bold(_) => {
                    // Inline code and bold are just formatting, no extra metadata needed
                }
            }
        }

        // Language detection
        metadata.language = crate::processing::Language::En.to_string();

        // Estimate reading time (average 200 words per minute)
        if metadata.word_count > 0 {
            metadata.estimated_reading_time = (metadata.word_count as f64 / 200.0).ceil() as u64;
        }

        doc.metadata = crate::processing::DocumentMetadata {
            headings: metadata.headings.clone(),
            table_count: metadata.table_count,
            code_block_count: metadata.code_block_count,
            list_count: metadata.list_count,
            command_count: metadata.command_count,
            example_count: metadata.example_count,
            warning_count: metadata.warning_count,
            reference_count: metadata.reference_count,
            language: metadata.language.clone(),
            word_count: metadata.word_count,
            char_count: metadata.char_count,
            estimated_reading_time: metadata.estimated_reading_time,
        };

        debug!(
            headings = metadata.headings.len(),
            tables = metadata.table_count,
            code_blocks = metadata.code_block_count,
            lists = metadata.list_count,
            "Metadata extracted"
        );
    }
}

impl Default for MetadataExtractStep {
    fn default() -> Self {
        Self::new()
    }
}
