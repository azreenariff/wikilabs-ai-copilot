//! Testing utilities for knowledge packs.

use crate::sdk::schema::{Manifest, Metadata};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Tests a knowledge pack and returns test results.
///
/// Runs all available tests:
/// - Structure test: verifies manifest.yaml and metadata.yaml exist and are valid
/// - Document test: verifies all documents referenced in the manifest exist
/// - Embedding test: verifies embedding configuration is valid
/// - Integration test: verifies the pack can be validated via the SDK validation tool
pub fn test_pack(pack_path: &str) -> TestResults {
    debug!(pack_path = %pack_path, "Running knowledge pack tests");

    let mut results = TestResults {
        pack_path: pack_path.to_string(),
        tests: Vec::new(),
        passed: 0,
        failed: 0,
    };

    // Test 1: Structure test
    let structure = test_structure(pack_path);
    results.tests.push(structure.clone());
    if structure.passed {
        results.passed += 1;
    } else {
        results.failed += 1;
    }

    // Test 2: Document test
    let documents = test_documents(pack_path);
    results.tests.push(documents.clone());
    if documents.passed {
        results.passed += 1;
    } else {
        results.failed += 1;
    }

    // Test 3: Embedding test
    let embedding = test_embedding_config(pack_path);
    results.tests.push(embedding.clone());
    if embedding.passed {
        results.passed += 1;
    } else {
        results.failed += 1;
    }

    // Test 4: SDK validation test
    let sdk_valid = test_sdk_validation(pack_path);
    results.tests.push(sdk_valid.clone());
    if sdk_valid.passed {
        results.passed += 1;
    } else {
        results.failed += 1;
    }

    debug!(
        passed = results.passed,
        failed = results.failed,
        "Knowledge pack tests completed"
    );

    results
}

/// Tests that the pack has the correct structure.
fn test_structure(pack_path: &str) -> TestResult {
    let path = Path::new(pack_path);

    if !path.exists() {
        return TestResult {
            name: "structure".to_string(),
            passed: false,
            message: format!("Pack directory does not exist: {}", pack_path),
        };
    }

    let manifest_path = path.join("manifest.yaml");
    let metadata_path = path.join("metadata.yaml");

    if !manifest_path.exists() {
        return TestResult {
            name: "structure".to_string(),
            passed: false,
            message: "manifest.yaml is missing".to_string(),
        };
    }

    if !metadata_path.exists() {
        return TestResult {
            name: "structure".to_string(),
            passed: false,
            message: "metadata.yaml is missing".to_string(),
        };
    }

    // Try to parse both files
    let manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(e) => {
            return TestResult {
                name: "structure".to_string(),
                passed: false,
                message: format!("Failed to read manifest.yaml: {}", e),
            };
        }
    };

    let metadata_content = match fs::read_to_string(&metadata_path) {
        Ok(content) => content,
        Err(e) => {
            return TestResult {
                name: "structure".to_string(),
                passed: false,
                message: format!("Failed to read metadata.yaml: {}", e),
            };
        }
    };

    let manifest: Manifest = match serde_yaml::from_str(&manifest_content) {
        Ok(m) => m,
        Err(e) => {
            return TestResult {
                name: "structure".to_string(),
                passed: false,
                message: format!("manifest.yaml is not valid YAML: {}", e),
            };
        }
    };

    let metadata: Metadata = match serde_yaml::from_str(&metadata_content) {
        Ok(m) => m,
        Err(e) => {
            return TestResult {
                name: "structure".to_string(),
                passed: false,
                message: format!("metadata.yaml is not valid YAML: {}", e),
            };
        }
    };

    if let Err(e) = manifest.validate() {
        return TestResult {
            name: "structure".to_string(),
            passed: false,
            message: format!("manifest validation failed: {}", e),
        };
    }

    if let Err(e) = metadata.validate() {
        return TestResult {
            name: "structure".to_string(),
            passed: false,
            message: format!("metadata validation failed: {}", e),
        };
    }

    TestResult {
        name: "structure".to_string(),
        passed: true,
        message: "Pack structure is valid".to_string(),
    }
}

/// Tests that all documents referenced in the manifest exist.
fn test_documents(pack_path: &str) -> TestResult {
    let path = Path::new(pack_path);
    let manifest_path = path.join("manifest.yaml");

    let manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(e) => {
            return TestResult {
                name: "documents".to_string(),
                passed: false,
                message: format!("Failed to read manifest.yaml: {}", e),
            };
        }
    };

    let manifest: Manifest = match serde_yaml::from_str(&manifest_content) {
        Ok(m) => m,
        Err(e) => {
            return TestResult {
                name: "documents".to_string(),
                passed: false,
                message: format!("manifest.yaml is not valid YAML: {}", e),
            };
        }
    };

    let documents_dir = path.join("documents");
    if !documents_dir.exists() {
        return TestResult {
            name: "documents".to_string(),
            passed: false,
            message: "documents/ directory is missing".to_string(),
        };
    }

    let mut all_found = true;
    let mut missing_docs = Vec::new();

    for doc in &manifest.documents {
        let doc_full_path = documents_dir.join(&doc.path);
        if !doc_full_path.exists() {
            all_found = false;
            missing_docs.push(doc.path.clone());
        }
    }

    if !all_found {
        return TestResult {
            name: "documents".to_string(),
            passed: false,
            message: format!("Missing documents: {:?}", missing_docs),
        };
    }

    TestResult {
        name: "documents".to_string(),
        passed: true,
        message: format!(
            "All {} document(s) referenced in manifest found",
            manifest.documents.len()
        ),
    }
}

/// Tests that the embedding configuration is valid.
fn test_embedding_config(pack_path: &str) -> TestResult {
    let path = Path::new(pack_path);
    let metadata_path = path.join("metadata.yaml");

    let metadata_content = match fs::read_to_string(&metadata_path) {
        Ok(content) => content,
        Err(e) => {
            return TestResult {
                name: "embedding".to_string(),
                passed: false,
                message: format!("Failed to read metadata.yaml: {}", e),
            };
        }
    };

    let metadata: Metadata = match serde_yaml::from_str(&metadata_content) {
        Ok(m) => m,
        Err(e) => {
            return TestResult {
                name: "embedding".to_string(),
                passed: false,
                message: format!("metadata.yaml is not valid YAML: {}", e),
            };
        }
    };

    if metadata.embedding_model.is_empty() {
        return TestResult {
            name: "embedding".to_string(),
            passed: false,
            message: "embedding_model is empty".to_string(),
        };
    }

    if metadata.embedding_dimensions == 0 {
        return TestResult {
            name: "embedding".to_string(),
            passed: false,
            message: "embedding_dimensions must be positive".to_string(),
        };
    }

    TestResult {
        name: "embedding".to_string(),
        passed: true,
        message: format!(
            "Embedding config valid: model={}, dimensions={}",
            metadata.embedding_model, metadata.embedding_dimensions
        ),
    }
}

/// Tests that the pack passes the SDK validation tool.
fn test_sdk_validation(pack_path: &str) -> TestResult {
    let validation_result = crate::sdk::validate::validate_pack(pack_path);

    match validation_result {
        Ok(result) if result.is_valid => TestResult {
            name: "sdk_validation".to_string(),
            passed: true,
            message: "SDK validation passed".to_string(),
        },
        Ok(result) => TestResult {
            name: "sdk_validation".to_string(),
            passed: false,
            message: format!("SDK validation failed: {}", result.summary()),
        },
        Err(e) => TestResult {
            name: "sdk_validation".to_string(),
            passed: false,
            message: format!("SDK validation error: {}", e),
        },
    }
}

/// A single test result.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

/// All test results for a pack test run.
#[derive(Debug)]
pub struct TestResults {
    pub pack_path: String,
    pub tests: Vec<TestResult>,
    pub passed: usize,
    pub failed: usize,
}

impl TestResults {
    /// Returns true if all tests passed.
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Returns a human-readable test report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Knowledge Pack Test Report ({}): {} passed, {} failed",
            self.pack_path, self.passed, self.failed
        )];

        for test in &self.tests {
            let status = if test.passed { "PASS" } else { "FAIL" };
            lines.push(format!("  [{}] {}: {}", status, test.name, test.message));
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
    fn test_pack_all_pass() {
        let tmp = TempDir::new().unwrap();
        let pack_path = create_valid_pack(&tmp);
        let results = test_pack(&pack_path);

        assert!(results.all_passed());
        assert_eq!(results.passed, 4);
        assert_eq!(results.failed, 0);
    }

    #[test]
    fn test_pack_structure_fail() {
        let tmp = TempDir::new().unwrap();
        let pack_dir = tmp.path().join("broken-pack");
        fs::create_dir_all(&pack_dir).unwrap();

        let results = test_pack(pack_dir.to_str().unwrap());
        assert!(!results.all_passed());
        assert!(results.failed > 0);
    }

    #[test]
    fn test_pack_documents_fail() {
        let tmp = TempDir::new().unwrap();
        let pack_dir = tmp.path().join("broken-pack");
        fs::create_dir_all(pack_dir.join("documents")).unwrap();
        fs::write(
            pack_dir.join("manifest.yaml"),
            "schema_version: '1.0'\nname: broken\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: missing\n    path: missing.md\n    format: markdown\n    embed: true\ndependencies: []\n",
        )
        .unwrap();
        fs::write(
            pack_dir.join("metadata.yaml"),
            "pack_name: broken\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();

        let results = test_pack(pack_dir.to_str().unwrap());
        assert!(!results.all_passed());
        assert!(results.failed > 0);
    }

    #[test]
    fn test_report_format() {
        let tmp = TempDir::new().unwrap();
        let pack_path = create_valid_pack(&tmp);
        let results = test_pack(&pack_path);

        let report = results.report();
        assert!(report.contains("Test Report"));
        assert!(report.contains("PASS"));
    }

    #[test]
    fn test_nonexistent_pack() {
        let results = test_pack("/nonexistent/path");
        assert!(!results.all_passed());
        assert_eq!(results.failed, 4);
    }
}
