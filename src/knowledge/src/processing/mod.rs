//! Document processing — parsers, document types, and element structures.
//!
//! Supports: Markdown, HTML, DOCX, TXT, YAML, JSON, XML, PDF.
//!
//! Each document is parsed into structured elements (headings, paragraphs,
//! code blocks, tables, lists, warnings, references) to preserve engineering
//! documentation structure rather than flattening to plain text.

mod markdown;
mod html;
mod txt;
mod yaml;
mod json;
mod xml;
mod pdf;
mod docx;

pub use markdown::MarkdownParser;
pub use html::HtmlParser;
pub use txt::TxtParser;
pub use yaml::YamlParser;
pub use json::JsonParser;
pub use xml::XmlParser;
pub use pdf::PdfParser;
pub use docx::DocxParser;

/// Parsed document — structured representation of any supported format.
#[derive(Debug, Clone)]
pub struct Document {
    pub full_text: String,
    pub author: String,
    pub source: String,
    pub filename: String,
    pub extension: String,
    pub title: String,
    pub element_count: usize,
    pub word_count: usize,
    pub line_count: usize,
    pub elements: Vec<DocumentElement>,
    pub metadata: DocumentMetadata,
    pub version_info: VersionInfo,
    pub detected_language: Language,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Language of a document.
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    None,
    En,
    Cn,
    Jp,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::None => write!(f, "en"),
            Language::En => write!(f, "en"),
            Language::Cn => write!(f, "zh"),
            Language::Jp => write!(f, "ja"),
        }
    }
}

impl Document {
    pub fn new(full_text: impl Into<String>, author: impl Into<String>, source: impl Into<String>) -> Self {
        let full_text = full_text.into();
        let author = author.into();
        let source = source.into();
        let now = chrono::Utc::now();
        let filename = source.rsplit('/').next().unwrap_or("unknown").to_string();
        let extension = source.rsplit('.').next().unwrap_or("").to_string();
        let title = filename.split('.').next().unwrap_or(&filename).to_string();
        let elements = Vec::new();
        let word_count = full_text.split_whitespace().count();
        let line_count = full_text.lines().count();
        Self {
            full_text: full_text.to_string(),
            author: author.to_string(),
            source: source.to_string(),
            filename,
            extension,
            title,
            element_count: 0,
            word_count,
            line_count,
            elements,
            metadata: DocumentMetadata::default(),
            version_info: VersionInfo::default(),
            detected_language: Language::None,
            created_at: now,
            modified_at: Some(now),
        }
    }

    pub fn set_derived(&mut self, filename: &str, extension: &str) {
        self.filename = filename.to_string();
        self.extension = extension.to_string();
        if self.title.is_empty() {
            self.title = filename.split('.').next().unwrap_or(filename).to_string();
        }
    }
}

/// Metadata extracted from a document.
#[derive(Debug, Clone, Default)]
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

/// Version information extracted from a document.
#[derive(Debug, Clone, Default)]
pub struct VersionInfo {
    pub has_version_header: bool,
    pub version: String,
    pub detected_history: Vec<VersionEntry>,
}

/// A detected version entry in the document.
#[derive(Debug, Clone)]
pub struct VersionEntry {
    pub version: String,
    pub date: String,
    pub summary: String,
}

/// Structured elements within a parsed document.
#[derive(Debug, Clone)]
pub enum DocumentElement {
    /// Level 1–6 heading.
    Heading(u32, String),
    /// Regular paragraph text.
    Paragraph(String),
    /// Code block with optional language tag.
    CodeBlock(String, String),
    /// Markdown table (rows of cells).
    Table(Vec<Vec<String>>),
    /// Unordered or ordered list items.
    List(Vec<String>),
    /// Inline code.
    InlineCode(String),
    /// Bold text.
    Bold(String),
    /// Warning or note block.
    Warning(String),
    /// Example block.
    Example(String),
    /// Command line (bash/cmd/pwsh).
    Command(String),
    /// Reference/link [text](url).
    Reference(String, String),
}

/// Provider trait for document format parsers.
pub trait ParserProvider: Send + Sync {
    /// Parse content into a structured Document.
    fn parse(&self, content: &str, author: &str, source: &str) -> Document;
    /// Supported file extensions (without leading dot).
    fn supported_extensions(&self) -> Vec<String>;
}

/// Processing result from parsing a single document.
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub document: Document,
    pub element_count: usize,
    pub parse_time_ms: u64,
}