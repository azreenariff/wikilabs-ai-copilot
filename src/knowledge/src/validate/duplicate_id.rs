//! Check for duplicate identifiers in a knowledge pack.

use crate::sdk::schema::Manifest;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Checks for duplicate IDs in manifest documents and metadata.
///
/// Checks:
/// - Document IDs in manifest.yaml are unique
/// - Document IDs are non-empty
/// - No duplicate tags in metadata.yaml
/// - No duplicate categories in metadata.yaml
pub fn check_duplicate_ids(pack_path: &str) -> Result<DuplicateIdResult> {
    let path = Path::new(pack_path);

    // Check manifest for duplicate document IDs
    let manifest_path = path.join("manifest.yaml");
    let manifest_content = fs::read_to_string(&manifest_path)
        .context("Failed to read manifest.yaml for duplicate ID check")?;

    let manifest: Manifest =
        serde_yaml::from_str(&manifest_content).context("manifest.yaml is not valid YAML")?;

    let mut result = DuplicateIdResult {
        pack_path: pack_path.to_string(),
        duplicate_doc_ids: Vec::new(),
        duplicate_tags: Vec::new(),
        duplicate_categories: Vec::new(),
        is_valid: true,
    };

    // Check for duplicate document IDs
    let mut seen_doc_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for doc in &manifest.documents {
        if doc.id.is_empty() {
            result.duplicate_doc_ids.push("<empty>".to_string());
            result.is_valid = false;
            debug!(doc_id = "", "Empty document ID found");
            continue;
        }

        if !seen_doc_ids.insert(doc.id.clone()) {
            result.duplicate_doc_ids.push(doc.id.clone());
            result.is_valid = false;
            debug!(doc_id = %doc.id, "Duplicate document ID found");
        }
    }

    // Check metadata for duplicate tags and categories
    let metadata_path = path.join("metadata.yaml");
    if metadata_path.exists() {
        let metadata_content =
            fs::read_to_string(&metadata_path).context("Failed to read metadata.yaml")?;

        #[derive(serde::Deserialize)]
        struct MetadataTags {
            tags: Vec<String>,
            categories: Vec<String>,
            #[serde(default)]
            references: Vec<String>,
        }

        let tags_data: MetadataTags = match serde_yaml::from_str(&metadata_content) {
            Ok(d) => d,
            Err(_) => {
                result.is_valid = false;
                return Ok(result);
            }
        };

        // Check duplicate tags
        let mut seen_tags: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for tag in &tags_data.tags {
            if !seen_tags.insert(tag.as_str()) {
                result.duplicate_tags.push(tag.clone());
                result.is_valid = false;
            }
        }

        // Check duplicate categories
        let mut seen_cats: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for cat in &tags_data.categories {
            if !seen_cats.insert(cat.as_str()) {
                result.duplicate_categories.push(cat.clone());
                result.is_valid = false;
            }
        }
    }

    debug!(
        valid = result.is_valid,
        dup_doc_ids = result.duplicate_doc_ids.len(),
        dup_tags = result.duplicate_tags.len(),
        dup_categories = result.duplicate_categories.len(),
        "Duplicate ID check completed"
    );

    Ok(result)
}

/// Duplicate ID check result.
#[derive(Debug, Default)]
pub struct DuplicateIdResult {
    pub pack_path: String,
    pub duplicate_doc_ids: Vec<String>,
    pub duplicate_tags: Vec<String>,
    pub duplicate_categories: Vec<String>,
    pub is_valid: bool,
}

impl DuplicateIdResult {
    /// Generates a human-readable report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Duplicate ID Report ({}):\n  Valid: {}\n  Duplicate document IDs: {}\n  Duplicate tags: {}\n  Duplicate categories: {}",
            self.pack_path,
            self.is_valid,
            self.duplicate_doc_ids.len(),
            self.duplicate_tags.len(),
            self.duplicate_categories.len()
        )];

        if !self.duplicate_doc_ids.is_empty() {
            lines.push("  Duplicate document IDs:".to_string());
            for d in &self.duplicate_doc_ids {
                lines.push(format!("    - {}", d));
            }
        }

        if !self.duplicate_tags.is_empty() {
            lines.push("  Duplicate tags:".to_string());
            for t in &self.duplicate_tags {
                lines.push(format!("    - {}", t));
            }
        }

        if !self.duplicate_categories.is_empty() {
            lines.push("  Duplicate categories:".to_string());
            for c in &self.duplicate_categories {
                lines.push(format!("    - {}", c));
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
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\n  - id: doc2\n    path: doc2.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(
            tmp.path().join("metadata.yaml"),
            "pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: [tag1, tag2]\ncategories: [cat1, cat2]\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();
    }

    #[test]
    fn test_no_duplicates() {
        let tmp = TempDir::new().unwrap();
        create_valid_pack(&tmp);

        let result = check_duplicate_ids(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.is_valid);
        assert!(result.duplicate_doc_ids.is_empty());
    }

    #[test]
    fn test_duplicate_doc_ids() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\n  - id: doc1\n    path: doc2.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("metadata.yaml"), "pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n").unwrap();

        let result = check_duplicate_ids(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.duplicate_doc_ids.len(), 1);
    }

    #[test]
    fn test_duplicate_tags() {
        let tmp = TempDir::new().unwrap();
        create_valid_pack(&tmp);
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(
            tmp.path().join("metadata.yaml"),
            "pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: [tag1, tag1]\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();

        let result = check_duplicate_ids(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.duplicate_tags.len(), 1);
    }

    #[test]
    fn test_duplicate_categories() {
        let tmp = TempDir::new().unwrap();
        create_valid_pack(&tmp);
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(
            tmp.path().join("metadata.yaml"),
            "pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: [cat1, cat1]\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();

        let result = check_duplicate_ids(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.duplicate_categories.len(), 1);
    }

    #[test]
    fn test_report_format() {
        let tmp = TempDir::new().unwrap();
        create_valid_pack(&tmp);

        let result = check_duplicate_ids(tmp.path().to_str().unwrap()).unwrap();
        let report = result.report();
        assert!(report.contains("Duplicate ID Report"));
    }
}
