//! Incremental update step — check file mtime vs indexed timestamp.

use super::discover::DiscoveredDoc;
use tracing::debug;

/// The incremental update pipeline step.
pub struct IncrementalStep;

impl IncrementalStep {
    pub fn new() -> Self {
        Self
    }

    /// Check if a document needs to be re-indexed based on modification time.
    ///
    /// Returns `true` if the document should be processed, `false` if it's up to date.
    pub fn check_incremental(
        &self,
        doc: &DiscoveredDoc,
        indexed_mtimes: &std::collections::HashMap<String, std::time::SystemTime>,
    ) -> anyhow::Result<bool> {
        let path_str = doc.path.to_string_lossy().to_string();

        // If never indexed before, needs processing
        if !indexed_mtimes.contains_key(&path_str) {
            debug!(path = ?doc.path, "New document, needs indexing");
            return Ok(true);
        }

        let current_mtime = doc.modified_time;
        let indexed_mtime = indexed_mtimes.get(&path_str).expect("checked above");

        // Compare modification times
        let needs_update = current_mtime > *indexed_mtime;
        if needs_update {
            debug!(path = ?doc.path, "File changed, needs re-indexing");
        } else {
            debug!(path = ?doc.path, "File unchanged, skip re-indexing");
        }

        Ok(needs_update)
    }
}

impl Default for IncrementalStep {
    fn default() -> Self {
        Self::new()
    }
}
