//! Confluence provider (stub) — future integration with Atlassian Confluence.

use anyhow::Result;
use async_trait::async_trait;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider for Atlassian Confluence pages and attachments.
pub struct ConfluenceProvider {
    enabled: bool,
    instance_url: String,
}

impl Default for ConfluenceProvider {
    fn default() -> Self {
        Self {
            enabled: false,
            instance_url: String::new(),
        }
    }
}

impl ConfluenceProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.instance_url = url;
        self
    }
}

#[async_trait]
impl KnowledgeProvider for ConfluenceProvider {
    fn name(&self) -> &str {
        "confluence"
    }

    fn supported_formats(&self) -> &[&str] {
        &[
            "confluence-page",
            "confluence-space",
            "confluence-attachment",
        ]
    }

    fn get_enabled(&self, enabled: bool) -> bool {
        enabled
    }

    async fn discover(&self, _path: &str) -> Result<Vec<ProviderDocument>> {
        tracing::warn!("Confluence provider is a stub — not yet implemented");
        Ok(Vec::new())
    }

    async fn parse(&self, _path: &str) -> Result<ProviderDocument> {
        tracing::warn!("Confluence provider is a stub — not yet implemented");
        anyhow::bail!("Confluence provider is not yet implemented")
    }
}
