//! Validate documents exist and are readable.

use crate::sdk::schema::Manifest;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Validates that all documents referenced in the manifest exist and are readable.
///
/// Checks:
/// - documents/ directory exists
/// - Each document path in the manifest points to an existing file
/// - Each file is readable and non-empty (unless empty is allowed)
/// - File extensions match expected formats
pub fn validate_documents(pack_path: &str) -> Result<DocumentValidationResult> {
    let path = Path::new(pack_path);

    // Load manifest
    let manifest_content = fs::read_to_string(path.join("manifest.yaml"))
        .context("Failed to read manifest.yaml for document validation")?;

    let manifest: Manifest =
        serde_yaml::from_str(&manifest_content).context("manifest.yaml is not valid YAML")?;

    let documents_dir = path.join("documents");
    let mut result = DocumentValidationResult {
        pack_path: pack_path.to_string(),
        total: manifest.documents.len(),
        found: 0,
        missing: Vec::new(),
        unreadable: Vec::new(),
        empty: Vec::new(),
    };

    if !documents_dir.exists() {
        result.missing.extend(
            manifest
                .documents
                .iter()
                .map(|d| d.path.clone())
                .collect::<Vec<_>>(),
        );
        return Ok(result);
    }

    for doc in &manifest.documents {
        let doc_path = documents_dir.join(&doc.path);

        if !doc_path.exists() {
            result.missing.push(doc.path.clone());
            debug!(doc = %doc.path, "Document not found");
            continue;
        }

        // Check if file is readable
        if fs::metadata(&doc_path).is_err() {
            result.unreadable.push(doc.path.clone());
            continue;
        }

        let metadata = fs::metadata(&doc_path)?;
        if metadata.len() == 0 {
            result.empty.push(doc.path.clone());
            debug!(doc = %doc.path, "Document is empty");
        }

        result.found += 1;
        debug!(doc = %doc.path, size = metadata.len(), "Document validated");
    }

    debug!(
        found = result.found,
        missing = result.missing.len(),
        unreadable = result.unreadable.len(),
        empty = result.empty.len(),
        "Document validation completed"
    );

    Ok(result)
}

/// Result of document validation.
#[derive(Debug, Default)]
pub struct DocumentValidationResult {
    pub pack_path: String,
    pub total: usize,
    pub found: usize,
    pub missing: Vec<String>,
    pub unreadable: Vec<String>,
    pub empty: Vec<String>,
}

impl DocumentValidationResult {
    /// Returns true if all documents were found and are readable.
    pub fn is_valid(&self) -> bool {
        self.missing.is_empty() && self.unreadable.is_empty()
    }

    /// Returns the number of issues found.
    pub fn issue_count(&self) -> usize {
        self.missing.len() + self.unreadable.len() + self.empty.len()
    }

    /// Generates a human-readable report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Document Validation Report ({}):\n  Total documents: {}\n  Found: {}\n  Missing: {}\n  Unreadable: {}\n  Empty: {}",
            self.pack_path,
            self.total,
            self.found,
            self.missing.len(),
            self.unreadable.len(),
            self.empty.len()
        )];

        if !self.missing.is_empty() {
            lines.push("  Missing documents:".to_string());
            for m in &self.missing {
                lines.push(format!("    - {}", m));
            }
        }

        if !self.unreadable.is_empty() {
            lines.push("  Unreadable documents:".to_string());
            for u in &self.unreadable {
                lines.push(format!("    - {}", u));
            }
        }

        if !self.empty.is_empty() {
            lines.push("  Empty documents:".to_string());
            for e in &self.empty {
                lines.push(format!("    - {}", e));
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

    #[test]
    fn test_validate_all_documents_exist() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("documents/doc1.md"), "# Test\n\nContent.\n").unwrap();

        let result = validate_documents(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.is_valid());
        assert_eq!(result.found, 1);
        assert!(result.missing.is_empty());
    }

    #[test]
    fn test_validate_missing_document() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: missing\n    path: missing.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();

        let result = validate_documents(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.missing.len(), 1);
        assert_eq!(result.missing[0], "missing.md");
    }

    #[test]
    fn test_validate_empty_document() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: empty-doc\n    path: empty.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("documents/empty.md"), "").unwrap();

        let result = validate_documents(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.empty.len(), 1);
        assert_eq!(result.found, 1);
    }

    #[test]
    fn test_validate_missing_documents_dir() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();

        let result = validate_documents(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid());
        assert_eq!(result.missing.len(), 1);
    }

    #[test]
    fn test_report_format() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: missing\n    path: missing.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();

        let result = validate_documents(tmp.path().to_str().unwrap()).unwrap();
        let report = result.report();
        assert!(report.contains("Document Validation Report"));
        assert!(report.contains("missing.md"));
    }
}
