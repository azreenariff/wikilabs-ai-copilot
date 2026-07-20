//! SDK-level validation tool for knowledge packs.
//!
/// Validates that a knowledge pack directory has the correct structure.
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Validates a knowledge pack directory.
///
/// Checks:
/// - manifest.yaml exists and is valid YAML
/// - metadata.yaml exists and is valid YAML
/// - documents/ directory exists (even if empty)
/// - All documents referenced in manifest.yaml exist as files
pub fn validate_pack(pack_path: &str) -> Result<ValidationResult> {
    let path = Path::new(pack_path);
    debug!(pack_path = %pack_path, "Validating knowledge pack");

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check manifest.yaml
    let manifest_path = path.join("manifest.yaml");
    let manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(e) => {
            errors.push(format!("manifest.yaml missing or unreadable: {}", e));
            String::new()
        }
    };

    // Parse and validate manifest
    let parsed_manifest: crate::sdk::schema::Manifest =
        match serde_yaml::from_str::<crate::sdk::schema::Manifest>(&manifest_content) {
            Ok(m) => {
                if let Err(e) = m.validate() {
                    errors.push(format!("manifest validation failed: {}", e));
                }
                debug!("manifest.yaml parsed and validated");
                m
            }
            Err(e) => {
                errors.push(format!("manifest.yaml is not valid YAML: {}", e));
                crate::sdk::schema::Manifest {
                    schema_version: "1.0".to_string(),
                    name: "unknown".to_string(),
                    version: "0.0.0".to_string(),
                    description: String::new(),
                    author: String::new(),
                    license: String::new(),
                    format_version: "1.0".to_string(),
                    documents: Vec::new(),
                    dependencies: Vec::new(),
                }
            }
        };

    // Check metadata.yaml
    let metadata_path = path.join("metadata.yaml");
    let metadata_content = match fs::read_to_string(&metadata_path) {
        Ok(content) => content,
        Err(e) => {
            errors.push(format!("metadata.yaml missing or unreadable: {}", e));
            String::new()
        }
    };

    let metadata: crate::sdk::schema::Metadata =
        match serde_yaml::from_str::<crate::sdk::schema::Metadata>(&metadata_content) {
            Ok(m) => {
                if let Err(e) = m.validate() {
                    errors.push(format!("metadata validation failed: {}", e));
                }
                debug!("metadata.yaml parsed and validated");
                m
            }
            Err(e) => {
                errors.push(format!("metadata.yaml is not valid YAML: {}", e));
                crate::sdk::schema::Metadata::new("unknown", "0.0.0", "Unknown", "all-MiniLM-L6-v2")
            }
        };

    // Check documents directory
    let documents_dir = path.join("documents");
    if !documents_dir.exists() {
        errors.push("documents/ directory is missing".to_string());
    } else {
        debug!(documents_dir = %documents_dir.display(), "Documents directory exists");

        // Check that all referenced documents exist
        for doc in &parsed_manifest.documents {
            let doc_full_path = documents_dir.join(&doc.path);
            if !doc_full_path.exists() {
                errors.push(format!("referenced document not found: {}", doc.path));
            } else {
                debug!(doc_path = %doc.path, "Referenced document exists");
            }
        }
    }

    // Validate embedding compatibility
    if metadata.embedding_dimensions == 0 {
        warnings.push("embedding_dimensions is 0, may cause indexing issues".to_string());
    }

    // Count documents
    let doc_count = if documents_dir.exists() {
        fs::read_dir(&documents_dir)
            .map(|mut entries| {
                let mut count = 0;
                for entry in entries.by_ref() {
                    if let Ok(e) = entry {
                        if e.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                            count += 1;
                        }
                    }
                }
                count
            })
            .unwrap_or(0)
    } else {
        0
    };

    let is_valid = errors.is_empty();

    Ok(ValidationResult {
        is_valid,
        manifest: parsed_manifest,
        metadata,
        errors,
        warnings,
        document_count: doc_count,
        pack_path: pack_path.to_string(),
    })
}

/// Result of a pack validation.
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub manifest: crate::sdk::schema::Manifest,
    pub metadata: crate::sdk::schema::Metadata,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub document_count: usize,
    pub pack_path: String,
}

impl ValidationResult {
    /// Returns a human-readable summary of the validation.
    pub fn summary(&self) -> String {
        let status = if self.is_valid { "VALID" } else { "INVALID" };
        let mut lines = vec![
            format!("Knowledge Pack Validation: {}", status),
            format!("  Path: {}", self.pack_path),
            format!("  Pack Name: {}", self.metadata.pack_name),
            format!("  Version: {}", self.metadata.pack_version),
            format!("  Documents: {}", self.document_count),
        ];

        if !self.errors.is_empty() {
            lines.push(format!("  Errors ({}):", self.errors.len()));
            for err in &self.errors {
                lines.push(format!("    - {}", err));
            }
        }

        if !self.warnings.is_empty() {
            lines.push(format!("  Warnings ({}):", self.warnings.len()));
            for warn in &self.warnings {
                lines.push(format!("    - {}", warn));
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

    fn create_valid_pack(tmp: &TempDir) -> String {
        let pack_dir = tmp.path().join("valid-pack");
        fs::create_dir_all(pack_dir.join("documents")).unwrap();
        fs::write(
            pack_dir.join("manifest.yaml"),
            "schema_version: '1.0'\nname: valid-pack\nversion: '1.0.0'\ndescription: A valid pack\nauthor: Test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\ndependencies: []\n",
        )
        .unwrap();
        fs::write(
            pack_dir.join("metadata.yaml"),
            "pack_name: valid-pack\npack_version: '1.0.0'\ndescription: A valid pack\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();
        fs::write(
            pack_dir.join("documents/doc1.md"),
            "# Test Doc\n\nContent.\n",
        )
        .unwrap();
        tmp.path().join("valid-pack").to_string_lossy().to_string()
    }

    #[test]
    fn test_validate_valid_pack() {
        let tmp = TempDir::new().unwrap();
        let pack_path = create_valid_pack(&tmp);
        let result = validate_pack(&pack_path).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.document_count, 1);
    }

    #[test]
    fn test_validate_missing_manifest() {
        let tmp = TempDir::new().unwrap();
        let pack_dir = tmp.path().join("bad-pack");
        fs::create_dir_all(pack_dir.join("documents")).unwrap();

        let result = validate_pack(pack_dir.to_str().unwrap()).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("manifest.yaml")));
    }

    #[test]
    fn test_validate_missing_metadata() {
        let tmp = TempDir::new().unwrap();
        let pack_dir = tmp.path().join("bad-pack");
        fs::create_dir_all(pack_dir.join("documents")).unwrap();
        fs::write(pack_dir.join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n").unwrap();

        let result = validate_pack(pack_dir.to_str().unwrap()).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("metadata.yaml")));
    }

    #[test]
    fn test_validate_missing_documents_dir() {
        let tmp = TempDir::new().unwrap();
        let pack_dir = tmp.path().join("bad-pack");
        fs::create_dir_all(&pack_dir).unwrap();
        fs::write(pack_dir.join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n").unwrap();
        fs::write(
            pack_dir.join("metadata.yaml"),
            "pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();

        let result = validate_pack(pack_dir.to_str().unwrap()).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("documents/")));
    }

    #[test]
    fn test_validate_missing_document_reference() {
        let tmp = TempDir::new().unwrap();
        let pack_dir = tmp.path().join("bad-pack");
        fs::create_dir_all(pack_dir.join("documents")).unwrap();
        fs::write(pack_dir.join("manifest.yaml"), "schema_version: '1.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: missing\n    path: missing.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(
            pack_dir.join("metadata.yaml"),
            "pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();

        let result = validate_pack(pack_dir.to_str().unwrap()).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("missing.md")));
    }

    #[test]
    fn test_validation_summary() {
        let tmp = TempDir::new().unwrap();
        let pack_path = create_valid_pack(&tmp);
        let result = validate_pack(&pack_path).unwrap();
        let summary = result.summary();
        assert!(summary.contains("VALID"));
        assert!(summary.contains("valid-pack"));
    }
}
