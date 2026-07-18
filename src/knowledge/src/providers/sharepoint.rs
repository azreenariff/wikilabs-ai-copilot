//! SharePoint provider (stub) — future integration with Microsoft SharePoint.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use tracing::debug;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider for Microsoft SharePoint sites and documents.
pub struct SharePointProvider {
    enabled: bool,
    site_url: String,
}

impl Default for SharePointProvider {
    fn default() -> Self {
        Self {
            enabled: false,
            site_url: String::new(),
        }
    }
}

impl SharePointProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.site_url = url;
        self
    }
}

#[async_trait]
impl KnowledgeProvider for SharePointProvider {
    fn name(&self) -> &str {
        "sharepoint"
    }

    fn supported_formats(&self) -> &[&str] {
        &["sharepoint-document", "sharepoint-site", "sharepoint-page"]
    }

    fn get_enabled(&self, enabled: bool) -> bool {
        enabled
    }

    async fn discover(&self, _path: &str) -> Result<Vec<ProviderDocument>> {
        tracing::warn!("SharePoint provider is a stub — not yet implemented");
        Ok(Vec::new())
    }

    async fn parse(&self, _path: &str) -> Result<ProviderDocument> {
        tracing::warn!("SharePoint provider is a stub — not yet implemented");
        anyhow::bail!("SharePoint provider is not yet implemented")
    }
}

