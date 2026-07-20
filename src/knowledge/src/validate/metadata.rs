//! Validate metadata.yaml schema.

use crate::sdk::schema::Metadata;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Validates a metadata.yaml file at the given path.
///
/// Checks:
/// - File exists and is readable
/// - File is valid YAML
/// - Deserializes to a valid Metadata
/// - Metadata.validate() passes
pub fn validate_metadata(pack_path: &str) -> Result<Metadata> {
    let path = Path::new(pack_path);
    let metadata_path = path.join("metadata.yaml");

    let content = fs::read_to_string(&metadata_path)
        .with_context(|| format!("Failed to read metadata.yaml at {}", pack_path))?;

    let metadata: Metadata =
        serde_yaml::from_str(&content).context("metadata.yaml is not valid YAML")?;

    metadata
        .validate()
        .context("metadata.yaml failed schema validation")?;

    tracing::debug!(
        pack_name = %metadata.pack_name,
        embedding_model = %metadata.embedding_model,
        dimensions = metadata.embedding_dimensions,
        "metadata.yaml validated successfully"
    );

    Ok(metadata)
}

/// Validates metadata from a raw YAML string.
pub fn validate_metadata_from_str(yaml: &str) -> Result<Metadata> {
    let metadata: Metadata = serde_yaml::from_str(yaml).context("Input is not valid YAML")?;

    metadata
        .validate()
        .context("Metadata failed schema validation")?;

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_valid_metadata() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("metadata.yaml"),
            "pack_name: test-pack\npack_version: '1.0.0'\ndescription: Test pack\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();

        let metadata = validate_metadata(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(metadata.pack_name, "test-pack");
        assert_eq!(metadata.embedding_model, "all-MiniLM-L6-v2");
    }

    #[test]
    fn test_validate_missing_metadata() {
        let tmp = TempDir::new().unwrap();
        let result = validate_metadata(tmp.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_embedding_model() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("metadata.yaml"),
            "pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: ''\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();

        let result = validate_metadata(tmp.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_zero_dimensions() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("metadata.yaml"),
            "pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: model\nembedding_dimensions: 0\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n",
        )
        .unwrap();

        let result = validate_metadata(tmp.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_from_str() {
        let yaml = "pack_name: test-pack\npack_version: '1.0.0'\ndescription: Test pack\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n";
        let metadata = validate_metadata_from_str(yaml).unwrap();
        assert_eq!(metadata.pack_name, "test-pack");
    }
}
