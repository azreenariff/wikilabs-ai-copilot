//! Validate manifest.yaml schema.

use crate::sdk::schema::Manifest;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Validates a manifest.yaml file at the given path.
///
/// Checks:
/// - File exists and is readable
/// - File is valid YAML
/// - Deserializes to a valid Manifest
/// - Manifest.validate() passes
pub fn validate_manifest(pack_path: &str) -> Result<Manifest> {
    let path = Path::new(pack_path);
    let manifest_path = path.join("manifest.yaml");

    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read manifest.yaml at {}", pack_path))?;

    let manifest: Manifest =
        serde_yaml::from_str(&content).context("manifest.yaml is not valid YAML")?;

    manifest
        .validate()
        .context("manifest.yaml failed schema validation")?;

    tracing::debug!(
        pack_name = %manifest.name,
        version = %manifest.version,
        doc_count = manifest.documents.len(),
        "manifest.yaml validated successfully"
    );

    Ok(manifest)
}

/// Validates a manifest from a raw YAML string.
pub fn validate_manifest_from_str(yaml: &str) -> Result<Manifest> {
    let manifest: Manifest = serde_yaml::from_str(yaml).context("Input is not valid YAML")?;

    manifest
        .validate()
        .context("Manifest failed schema validation")?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_valid_manifest() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("manifest.yaml"),
            "schema_version: '1.0'\nname: test-pack\nversion: '1.0.0'\ndescription: Test pack\nauthor: Test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\ndependencies: []\n",
        )
        .unwrap();

        let manifest = validate_manifest(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(manifest.name, "test-pack");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.documents.len(), 1);
    }

    #[test]
    fn test_validate_missing_manifest() {
        let tmp = TempDir::new().unwrap();
        let result = validate_manifest(tmp.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_schema_version() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("manifest.yaml"),
            "schema_version: '2.0'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n",
        )
        .unwrap();

        let result = validate_manifest(tmp.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_name() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("manifest.yaml"),
            "schema_version: '1.0'\nname: ''\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n",
        )
        .unwrap();

        let result = validate_manifest(tmp.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_from_str() {
        let yaml = "schema_version: '1.0'\nname: test-pack\nversion: '1.0.0'\ndescription: Test pack\nauthor: Test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n";
        let manifest = validate_manifest_from_str(yaml).unwrap();
        assert_eq!(manifest.name, "test-pack");
    }
}
