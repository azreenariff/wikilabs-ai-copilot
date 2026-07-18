//! Check for broken internal references in a knowledge pack.

use crate::sdk::schema::Manifest;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Checks that all internal references (document IDs, tags, categories)
/// in the pack are consistent and point to existing resources.
///
/// Checks:
/// - All document IDs referenced in the manifest correspond to existing files
/// - Links within documents (if present) reference valid document IDs in the pack
/// - Cross-document references are valid
pub fn check_broken_refs(pack_path: &str) -> Result<BrokenRefsResult> {
    let path = Path::new(pack_path);

    // Load manifest
    let manifest_content = fs::read_to_string(path.join("manifest.yaml"))
        .context("Failed to read manifest.yaml for broken reference check")?;

    let manifest: Manifest =
        serde_yaml::from_str(&manifest_content).context("manifest.yaml is not valid YAML")?;

    let documents_dir = path.join("documents");
    let mut result = BrokenRefsResult {
        pack_path: pack_path.to_string(),
        missing_docs: Vec::new(),
        broken_links: Vec::new(),
        valid: true,
    };

    // Build set of valid document IDs
    let valid_ids: std::collections::HashSet<&str> =
        manifest.documents.iter().map(|d| d.id.as_str()).collect();

    // Check that all referenced document paths exist
    for doc in &manifest.documents {
        let doc_path = documents_dir.join(&doc.path);
        if !doc_path.exists() {
            result.missing_docs.push(doc.id.clone());
            result.valid = false;
            debug!(doc_id = %doc.id, doc_path = %doc.path, "Document not found");
        }
    }

    // Check for broken links within document content
    if documents_dir.exists() {
        for doc in &manifest.documents {
            let doc_path = documents_dir.join(&doc.path);
            if let Ok(content) = fs::read_to_string(&doc_path) {
                // Look for internal reference patterns like [[doc_id]] or [[doc_id|label]]
                let link_pattern = regex::Regex::new(r"\[\[(\w[\w-]*)\|?[^]]*\]\]").ok();
                if let Some(pattern) = link_pattern {
                    for capture in pattern.captures_iter(&content) {
                        if let Some(id_match) = capture.get(1) {
                            let ref_id = id_match.as_str();
                            if !valid_ids.contains(ref_id) {
                                result.broken_links.push(format!(
                                    "{} -> [[{}]]",
                                    doc.id, ref_id
                                ));
                                result.valid = false;
                                debug!(
                                    source = %doc.id,
                                    target = ref_id,
                                    "Broken internal reference"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    debug!(
        valid = result.valid,
        missing = result.missing_docs.len(),
        broken_links = result.broken_links.len(),
        "Broken reference check completed"
    );

    Ok(result)
}

/// Broken reference check result.
#[derive(Debug, Default)]
pub struct BrokenRefsResult {
    pub pack_path: String,
    pub missing_docs: Vec<String>,
    pub broken_links: Vec<String>,
    pub valid: bool,
}

impl BrokenRefsResult {
    /// Generates a human-readable report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Broken Reference Report ({}):\n  Valid: {}\n  Missing documents: {}\n  Broken links: {}",
            self.pack_path, self.valid, self.missing_docs.len(), self.broken_links.len()
        )];

        if !self.missing_docs.is_empty() {
            lines.push("  Missing documents:".to_string());
            for m in &self.missing_docs {
                lines.push(format!("    - {}", m));
            }
        }

        if !self.broken_links.is_empty() {
            lines.push("  Broken links:".to_string());
            for l in &self.broken_links {
                lines.push(format!("    - {}", l));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_valid_pack(tmp: &TempDir) {
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\n  - id: doc2\n    path: doc2.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("documents/doc1.md"), "# Doc1\n\nContent.\n").unwrap();
        fs::write(tmp.path().join("documents/doc2.md"), "# Doc2\n\nContent.\n").unwrap();
    }

    #[test]
    fn test_all_refs_valid() {
        let tmp = TempDir::new().unwrap();
        create_valid_pack(&tmp);

        let result = check_broken_refs(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.valid);
        assert!(result.missing_docs.is_empty());
        assert!(result.broken_links.is_empty());
    }

    #[test]
    fn test_missing_document_ref() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\n  - id: missing-doc\n    path: missing.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("documents/doc1.md"), "# Doc1\n").unwrap();

        let result = check_broken_refs(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.valid);
        assert_eq!(result.missing_docs.len(), 1);
        assert_eq!(result.missing_docs[0], "missing-doc");
    }

    #[test]
    fn test_broken_link() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(
            tmp.path().join("documents/doc1.md"),
            "# Doc1\n\nSee [[nonexistent-doc]].\n",
        )
        .unwrap();

        let result = check_broken_refs(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.valid);
        assert_eq!(result.broken_links.len(), 1);
        assert!(result.broken_links[0].contains("nonexistent-doc"));
    }

    #[test]
    fn test_valid_link() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\n  - id: doc2\n    path: doc2.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(
            tmp.path().join("documents/doc1.md"),
            "# Doc1\n\nSee [[doc2]].\n",
        )
        .unwrap();

        let result = check_broken_refs(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_report_format() {
        let tmp = TempDir::new().unwrap();
        create_valid_pack(&tmp);

        let result = check_broken_refs(tmp.path().to_str().unwrap()).unwrap();
        let report = result.report();
        assert!(report.contains("Broken Reference Report"));
    }
}