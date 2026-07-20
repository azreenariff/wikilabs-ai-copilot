//! Citation — track, verify, and generate source citations.
//!
//! Supports: inline citations, reference lists, link verification,
//! and cross-references between knowledge documents.

pub mod crossref;
pub mod link;
pub mod manager;

pub use crossref::CrossReference;
pub use link::CitationLink;
pub use manager::CitationManager;

/// Citation type for different source categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationType {
    /// Documentation reference.
    Documentation,
    /// Academic paper.
    Paper,
    /// Specification or standard.
    Specification,
    /// Blog post or article.
    Blog,
    /// Code repository.
    Repository,
    /// API reference.
    ApiReference,
    /// Community resource.
    Community,
    /// Internal document.
    Internal,
    /// Unspecified.
    Other(String),
}

/// Citation metadata.
#[derive(Debug, Clone)]
pub struct Citation {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub citation_type: CitationType,
    pub author: Option<String>,
    pub publication_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_verified: Option<chrono::DateTime<chrono::Utc>>,
    pub verified: bool,
    pub related_ids: Vec<String>,
    pub metadata: serde_json::Value,
}

impl Citation {
    pub fn new(id: &str, title: &str, citation_type: CitationType) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            url: None,
            citation_type,
            author: None,
            publication_date: None,
            last_verified: None,
            verified: false,
            related_ids: Vec::new(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn with_author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Link tracking for citations.
#[derive(Debug, Clone)]
pub struct LinkTracking {
    pub url: String,
    pub status: LinkStatus,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub http_status: Option<u16>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkStatus {
    /// Link is healthy.
    Healthy,
    /// Link returned an error.
    Broken(String),
    /// Link was not checked yet.
    Unchecked,
}

/// Generate a citation from a knowledge chunk.
pub fn generate_citation_from_chunk(
    chunk: &wikilabs_data_types::KnowledgeChunk,
    knowledge_pack: &str,
) -> Citation {
    let title = chunk.content.chars().take(80).collect::<String>();

    let mut citation = Citation::new(&chunk.id.to_string(), &title, CitationType::Documentation);

    citation
}

/// Format a citation for display.
pub fn format_citation(citation: &Citation, format: &str) -> String {
    match format {
        "html" => format!(
            r#"<cite title="{}">{}{}{}</cite>"#,
            citation.title,
            citation.title,
            citation
                .url
                .as_ref()
                .map(|url| format!(", <a href=\"{}\">link</a>", url))
                .unwrap_or_default(),
            citation
                .author
                .as_ref()
                .map(|a| format!(" by {}", a))
                .unwrap_or_default(),
        ),
        "markdown" => format!(
            "**{}**{}{}",
            citation.title,
            citation
                .url
                .as_ref()
                .map(|url| format!(" [link]({url})"))
                .unwrap_or_default(),
            citation
                .author
                .as_ref()
                .map(|a| format!(" by {a}"))
                .unwrap_or_default(),
        ),
        _ => format!(
            "{}{}{}",
            citation.title,
            citation
                .url
                .as_ref()
                .map(|url| format!(" ({url})"))
                .unwrap_or_default(),
            citation
                .author
                .as_ref()
                .map(|a| format!(" by {a}"))
                .unwrap_or_default(),
        ),
    }
}
