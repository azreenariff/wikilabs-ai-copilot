//! Duplicate detection step — content hashing (sha256).

use super::discover::DiscoveredDoc;
use super::compute_sha256;
use std::fs;
use tracing::debug;

/// The deduplication pipeline step.
pub struct DedupStep;

impl DedupStep {
    pub fn new() -> Self {
        Self
    }

    /// Run the deduplication check on a discovered document.
    /// Returns the SHA-256 hash of the document content.
    pub fn run(&self, doc: &DiscoveredDoc) -> anyhow::Result<String> {
        let contents = fs::read(&doc.path)?;
        let hash = compute_sha256(&String::from_utf8_lossy(&contents));
        debug!(path = ?doc.path, hash = %hash, "Computed content hash");
        Ok(hash)
    }
}

impl Default for DedupStep {
    fn default() -> Self {
        Self::new()
    }
}