//! Knowledge Provider trait and shared types.
//!
//! Providers discover, parse, and supply `ProviderDocument` objects from
//! various sources (filesystem, PDFs, Markdown, etc.).
//!
//! The `KnowledgeProvider` trait uses `#[async_trait]` for async methods.
//! The `ProviderRegistry` avoids `dyn` by tracking enabled kinds and using
//! factory methods.

use anyhow::Result;
use async_trait::async_trait;
use wikilabs_data_types::ProviderDocument;
use std::collections::HashMap;

/// File extension to provider kind mapping.
pub const EXTENSION_PROVIDER_MAP: &[(&str, ProviderKind)] = &[
    ("txt", ProviderKind::Txt),
    ("md", ProviderKind::Markdown),
    ("markdown", ProviderKind::Markdown),
    ("html", ProviderKind::Html),
    ("htm", ProviderKind::Html),
    ("yaml", ProviderKind::Yaml),
    ("yml", ProviderKind::Yaml),
    ("json", ProviderKind::Json),
    ("xml", ProviderKind::Xml),
    ("pdf", ProviderKind::Pdf),
    ("docx", ProviderKind::Docx),
    ("csv", ProviderKind::Filesystem),
    ("log", ProviderKind::Filesystem),
    ("conf", ProviderKind::Filesystem),
    ("cfg", ProviderKind::Filesystem),
    ("ini", ProviderKind::Filesystem),
    ("toml", ProviderKind::Filesystem),
    ("sh", ProviderKind::Filesystem),
    ("bash", ProviderKind::Filesystem),
    ("py", ProviderKind::Filesystem),
    ("rs", ProviderKind::Filesystem),
    ("go", ProviderKind::Filesystem),
    ("java", ProviderKind::Filesystem),
    ("js", ProviderKind::Filesystem),
    ("ts", ProviderKind::Filesystem),
    ("c", ProviderKind::Filesystem),
    ("cpp", ProviderKind::Filesystem),
    ("h", ProviderKind::Filesystem),
    ("hpp", ProviderKind::Filesystem),
    ("rb", ProviderKind::Filesystem),
    ("php", ProviderKind::Filesystem),
    ("pl", ProviderKind::Filesystem),
    ("sql", ProviderKind::Filesystem),
    ("r", ProviderKind::Filesystem),
    ("scala", ProviderKind::Filesystem),
    ("kt", ProviderKind::Filesystem),
    ("swift", ProviderKind::Filesystem),
    ("m", ProviderKind::Filesystem),
];

/// Knowledge provider trait — the core interface all providers implement.
///
/// All methods are async via async_trait to allow providers to do I/O.
#[async_trait]
pub trait KnowledgeProvider: Send + Sync {
    /// Human-readable provider name.
    fn name(&self) -> &str;

    /// File extensions / MIME types this provider supports.
    fn supported_formats(&self) -> &[&str];

    /// Get or set the enabled state.
    fn get_enabled(&self, enabled: bool) -> bool {
        enabled
    }

    /// Discover files matching the provider's criteria.
    async fn discover(&self, path: &str) -> Result<Vec<ProviderDocument>>;

    /// Parse a single file into a `ProviderDocument`.
    async fn parse(&self, path: &str) -> Result<ProviderDocument>;
}

/// Provider kind enum for registry lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Filesystem,
    Markdown,
    Html,
    Pdf,
    Docx,
    Txt,
    Yaml,
    Json,
    Xml,
    Git,
    Confluence,
    SharePoint,
}

impl ProviderKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Filesystem => "filesystem",
            Self::Markdown => "markdown",
            Self::Html => "html",
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Txt => "txt",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Git => "git",
            Self::Confluence => "confluence",
            Self::SharePoint => "sharepoint",
        }
    }

    /// Create a default instance of this provider kind.
    pub fn create(&self) -> Box<dyn KnowledgeProvider> {
        match self {
            Self::Filesystem => Box::new(FilesystemProvider::new()),
            Self::Markdown => Box::new(MarkdownProvider::new()),
            Self::Html => Box::new(HtmlProvider::new()),
            Self::Pdf => Box::new(PdfProvider::new()),
            Self::Docx => Box::new(DocxProvider::new()),
            Self::Txt => Box::new(TxtProvider::new()),
            Self::Yaml => Box::new(YamlProvider::new()),
            Self::Json => Box::new(JsonProvider::new()),
            Self::Xml => Box::new(XmlProvider::new()),
            Self::Git => Box::new(GitProvider::new()),
            Self::Confluence => Box::new(ConfluenceProvider::new()),
            Self::SharePoint => Box::new(SharePointProvider::new()),
        }
    }

    /// Find the provider kind for a file extension.
    pub fn for_extension(ext: &str) -> Option<Self> {
        let ext = ext.trim_start_matches('.').to_lowercase();
        EXTENSION_PROVIDER_MAP
            .iter()
            .find(|(e, _)| *e == ext)
            .map(|(_, kind)| *kind)
    }

    /// Get all supported formats for this provider kind.
    pub fn supported_formats(&self) -> Vec<String> {
        match self {
            Self::Filesystem => vec![
                "txt", "md", "markdown", "html", "htm", "yaml", "yml", "json",
                "xml", "csv", "log", "conf", "cfg", "ini", "toml", "sh", "bash",
                "py", "rs", "go", "java", "js", "ts", "c", "cpp", "h", "hpp",
                "rb", "php", "pl", "sql", "r", "scala", "kt", "swift", "m",
            ],
            Self::Markdown => vec!["md", "markdown"],
            Self::Html => vec!["html", "htm"],
            Self::Pdf => vec!["pdf"],
            Self::Docx => vec!["docx"],
            Self::Txt => vec!["txt", "text", "log", "conf", "cfg", "ini", "md", "rst"],
            Self::Yaml => vec!["yaml", "yml"],
            Self::Json => vec!["json"],
            Self::Xml => vec!["xml"],
            Self::Git => vec![
                "md", "txt", "html", "yaml", "yml", "json", "xml", "sh", "bash",
                "py", "rs", "go", "java", "js", "ts", "toml", "conf", "cfg",
                "ini", "dockerfile", "gitignore", "gitattributes",
            ],
            Self::Confluence => vec!["confluence-page", "confluence-space", "confluence-attachment"],
            Self::SharePoint => vec!["sharepoint-document", "sharepoint-site", "sharepoint-page"],
        }
    }
}

/// Registry of all available providers.
///
/// Tracks enabled/disabled state per provider kind. Uses factory methods
/// to create concrete provider instances.
pub struct ProviderRegistry {
    enabled: HashMap<ProviderKind, bool>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        let mut registry = Self {
            enabled: HashMap::new(),
        };
        for kind in &[
            ProviderKind::Filesystem,
            ProviderKind::Markdown,
            ProviderKind::Html,
            ProviderKind::Txt,
            ProviderKind::Yaml,
            ProviderKind::Json,
            ProviderKind::Xml,
            ProviderKind::Git,
            ProviderKind::Pdf,
            ProviderKind::Docx,
            ProviderKind::Confluence,
            ProviderKind::SharePoint,
        ] {
            registry.enabled.insert(*kind, true);
        }
        registry
    }
}

impl ProviderRegistry {
    /// Create a new registry with all providers enabled by default.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a provider kind is enabled.
    pub fn is_enabled(&self, kind: ProviderKind) -> bool {
        self.enabled.get(&kind).copied().unwrap_or(false)
    }

    /// Enable a provider kind.
    pub fn enable(&mut self, kind: ProviderKind, value: bool) {
        self.enabled.insert(kind, value);
    }

    /// Disable a provider kind.
    pub fn disable(&mut self, kind: ProviderKind) {
        self.enabled.insert(kind, false);
    }

    /// Get the enabled state for a kind.
    pub fn get_enabled_state(&self, kind: ProviderKind) -> bool {
        self.enabled.get(&kind).copied().unwrap_or(false)
    }

    /// Create an enabled provider instance for the given kind.
    pub fn create_provider(&self, kind: ProviderKind) -> Option<Box<dyn KnowledgeProvider>> {
        if self.is_enabled(kind) {
            Some(kind.create())
        } else {
            None
        }
    }

    /// Return a list of all known supported formats from enabled providers.
    pub fn all_supported_formats(&self) -> Vec<String> {
        let mut formats: Vec<String> = Vec::new();
        for kind in &self.enabled_kinds() {
            formats.extend(kind.supported_formats());
        }
        formats.sort();
        formats.dedup();
        formats
    }

    /// Get all enabled provider kinds.
    pub fn enabled_kinds(&self) -> Vec<ProviderKind> {
        self.enabled
            .iter()
            .filter(|(_, &v)| v)
            .map(|(k, _)| *k)
            .collect()
    }
}

// Re-export the trait and concrete providers.
pub use confluence::ConfluenceProvider;
pub use docx::DocxProvider;
pub use filesystem::FilesystemProvider;
pub use git::GitProvider;
pub use html::HtmlProvider;
pub use json::JsonProvider;
pub use markdown::MarkdownProvider;
pub use pdf::PdfProvider;
pub use sharepoint::SharePointProvider;
pub use txt::TxtProvider;
pub use xml::XmlProvider;
pub use yaml::YamlProvider;

mod filesystem;
mod markdown;
mod html;
mod pdf;
mod docx;
mod txt;
mod yaml;
mod json;
mod xml;
mod git;
mod confluence;
mod sharepoint;

