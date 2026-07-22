//! Confluence provider (stub) — future integration with Atlassian Confluence.

use anyhow::Result;
use async_trait::async_trait;

use super::{KnowledgeProvider, ProviderDocument};

/// A provider for Atlassian Confluence pages and attachments.
#[derive(Default)]
pub struct ConfluenceProvider {
    #[allow(dead_code)]
    enabled: bool,
    instance_url: String,
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
