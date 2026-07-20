//! Filter knowledge retrieval results by workspace-enabled packs.

use anyhow::Result;
use tracing::debug;

/// Filters knowledge document retrieval results to only include
/// documents from packs that are enabled for the given workspace.
pub struct WorkspaceKnowledgeFilter {
    /// Enabled pack names for the current workspace.
    enabled_packs: Vec<String>,
}

impl WorkspaceKnowledgeFilter {
    /// Creates a new filter with the given enabled packs.
    pub fn new(enabled_packs: Vec<String>) -> Self {
        Self { enabled_packs }
    }

    /// Returns true if the given pack name is enabled for the workspace.
    pub fn is_pack_enabled(&self, pack_name: &str) -> bool {
        self.enabled_packs
            .iter()
            .any(|p| p.eq_ignore_ascii_case(pack_name))
    }

    /// Filters knowledge documents, keeping only those from enabled packs.
    ///
    /// Knowledge documents typically include a `source` field that encodes the pack name.
    /// This filters based on that source field.
    pub fn filter_documents<T: AsSource>(&self, documents: Vec<T>) -> Vec<T> {
        let total = documents.len();
        let enabled_count = self.enabled_packs.len();

        if enabled_count == 0 {
            // No packs enabled — return all (fallback: allow everything)
            debug!("No packs enabled, returning all documents");
            return documents;
        }

        let filtered: Vec<T> = documents
            .into_iter()
            .filter(|doc| self.is_pack_enabled(doc.source()))
            .collect();

        debug!(
            total,
            filtered = filtered.len(),
            "Applied workspace pack filter"
        );

        filtered
    }

    /// Returns the list of enabled pack names.
    pub fn enabled_packs(&self) -> &[String] {
        &self.enabled_packs
    }
}

/// Trait for types that have a source field representing the pack name.
pub trait AsSource {
    fn source(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestDoc {
        id: u32,
        source: String,
    }

    impl AsSource for TestDoc {
        fn source(&self) -> &str {
            &self.source
        }
    }

    #[test]
    fn test_filter_packs() {
        let filter =
            WorkspaceKnowledgeFilter::new(vec!["pack-a".to_string(), "pack-b".to_string()]);

        let docs = vec![
            TestDoc {
                id: 1,
                source: "pack-a".to_string(),
            },
            TestDoc {
                id: 2,
                source: "pack-b".to_string(),
            },
            TestDoc {
                id: 3,
                source: "pack-c".to_string(),
            },
            TestDoc {
                id: 4,
                source: "pack-a".to_string(),
            },
        ];

        let filtered = filter.filter_documents(docs);
        assert_eq!(filtered.len(), 3);
        assert!(!filtered.iter().any(|d| d.source == "pack-c"));
    }

    #[test]
    fn test_no_enabled_packs() {
        let filter = WorkspaceKnowledgeFilter::new(vec![]);

        let docs = vec![
            TestDoc {
                id: 1,
                source: "pack-a".to_string(),
            },
            TestDoc {
                id: 2,
                source: "pack-b".to_string(),
            },
        ];

        // When no packs are enabled, return all documents
        let filtered = filter.filter_documents(docs);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_is_pack_enabled() {
        let filter = WorkspaceKnowledgeFilter::new(vec!["pack-a".to_string()]);

        assert!(filter.is_pack_enabled("pack-a"));
        assert!(!filter.is_pack_enabled("pack-b"));
        assert!(!filter.is_pack_enabled("pack-c"));
    }

    #[test]
    fn test_case_insensitive_pack_name() {
        let filter = WorkspaceKnowledgeFilter::new(vec!["Pack-A".to_string()]);

        // Pack names should match case-insensitively
        assert!(filter.is_pack_enabled("pack-a"));
        assert!(filter.is_pack_enabled("PACK-A"));
    }
}
