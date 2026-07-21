//! Parse step — delegate to appropriate provider.

use super::discover::DiscoveredDoc;
use crate::processing::Document;
use crate::processing::{
    DocxParser, HtmlParser, JsonParser, MarkdownParser, ParserProvider, PdfParser,
    TxtParser, XmlParser, YamlParser,
};
use tracing::debug;

/// The parse pipeline step.
pub struct ParseStep;

impl ParseStep {
    pub fn new() -> Self {
        Self
    }

    /// Run the parse step on a discovered document.
    /// Selects the appropriate parser based on file extension.
    pub fn run(&self, doc: &DiscoveredDoc, author: &str) -> anyhow::Result<Document> {
        let parser = self.select_parser(&doc.extension)?;
        let content = std::fs::read_to_string(&doc.path)?;

        let document = parser.parse(&content, author, &doc.path.to_string_lossy().to_string());

        debug!(
            path = ?doc.path,
            extension = doc.extension,
            element_count = document.elements.len(),
            "Parsing complete"
        );

        Ok(document)
    }

    fn select_parser(&self, extension: &str) -> anyhow::Result<Box<dyn ParserProvider>> {
        match extension {
            "md" | "markdown" => Ok(Box::new(MarkdownParser::new())),
            "html" | "htm" => Ok(Box::new(HtmlParser::new())),
            "txt" | "text" => Ok(Box::new(TxtParser::new())),
            "yaml" | "yml" => Ok(Box::new(YamlParser::new())),
            "json" => Ok(Box::new(JsonParser::new())),
            "xml" => Ok(Box::new(XmlParser::new())),
            "pdf" => Ok(Box::new(PdfParser::new())),
            "docx" => Ok(Box::new(DocxParser::new())),
            _ => anyhow::bail!("Unsupported file type for parsing: {}", extension),
        }
    }
}

impl Default for ParseStep {
    fn default() -> Self {
        Self::new()
    }
}
