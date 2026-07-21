//! Citation link tracking.

use super::LinkStatus;
use chrono::Utc;

/// Represents a tracked link within a citation.
#[derive(Debug, Clone)]
pub struct CitationLink {
    pub url: String,
    pub title: Option<String>,
    pub status: LinkStatus,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub http_status: Option<u16>,
    pub error: Option<String>,
    pub is_internal: bool,
}

impl CitationLink {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            title: None,
            status: LinkStatus::Unchecked,
            last_checked: Utc::now(),
            http_status: None,
            error: None,
            is_internal: false,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_status(mut self, status: LinkStatus) -> Self {
        self.status = status;
        self.last_checked = Utc::now();
        self
    }
}
