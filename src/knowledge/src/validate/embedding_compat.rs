//! Validate embedding compatibility for a knowledge pack.

use crate::sdk::schema::Metadata;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Known compatible embedding models and their default dimensions.
const KNOWN_MODELS: &[(&str, u32)] = &[
    ("all-MiniLM-L6-v2", 384),
    ("all-MiniLM-L12-v2", 384),
    ("all-mpnet-base-v2", 768),
    ("BAAI/bge-small-en", 384),
    ("BAAI/bge-base-en", 768),
    ("BAAI/bge-large-en", 1024),
    ("text-embedding-ada-002", 1536),
    ("nomic-embed-text", 768),
    ("jina-embeddings-v2-base-en", 768),
];

/// Validates that the embedding configuration is compatible with known models.
///
/// Checks:
/// - embedding_model is a known model or has valid dimensions
/// - embedding_dimensions matches the model's expected dimensions (if known)
/// - Dimensions are within reasonable bounds (1-4096)
pub fn validate_embedding_compat(pack_path: &str) -> Result<EmbeddingCompatResult> {
    let path = Path::new(pack_path);
    let metadata_path = path.join("metadata.yaml");

    let content = fs::read_to_string(&metadata_path)
        .context("Failed to read metadata.yaml for embedding validation")?;

    let metadata: Metadata =
        serde_yaml::from_str(&content).context("metadata.yaml is not valid YAML")?;

    let mut result = EmbeddingCompatResult {
        pack_path: pack_path.to_string(),
        model: metadata.embedding_model.clone(),
        dimensions: metadata.embedding_dimensions,
        is_compatible: true,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    let model = metadata.embedding_model.as_str();

    // Check if model is known
    let known_model = KNOWN_MODELS.iter().find(|(m, _)| *m == model);

    if let Some((_, expected_dims)) = known_model {
        // Model is known — check dimensions match
        if metadata.embedding_dimensions != *expected_dims {
            result.warnings.push(format!(
                "model '{}' expects {} dimensions, but metadata says {}",
                model, expected_dims, metadata.embedding_dimensions
            ));
            result.errors.push(format!(
                "expected {} dimensions for model '{}', got {}",
                expected_dims, model, metadata.embedding_dimensions
            ));
            result.is_compatible = false;
            debug!(
                model = model,
                expected = expected_dims,
                actual = metadata.embedding_dimensions,
                "Dimension mismatch for known model"
            );
        }
    } else {
        // Model is unknown — just check dimensions are reasonable
        if metadata.embedding_dimensions < 64 {
            result
                .errors
                .push("embedding_dimensions too low (< 64) for unknown model".to_string());
            result.is_compatible = false;
        }
        if metadata.embedding_dimensions > 4096 {
            result
                .errors
                .push("embedding_dimensions too high (> 4096) for unknown model".to_string());
            result.is_compatible = false;
        }
    }

    // Check dimensions are within universal bounds
    if metadata.embedding_dimensions == 0 {
        result
            .errors
            .push("embedding_dimensions cannot be zero".to_string());
        result.is_compatible = false;
    }

    debug!(
        model = model,
        dimensions = metadata.embedding_dimensions,
        compatible = result.is_compatible,
        "Embedding compatibility check completed"
    );

    Ok(result)
}

/// Embedding compatibility check result.
#[derive(Debug, Default)]
pub struct EmbeddingCompatResult {
    pub pack_path: String,
    pub model: String,
    pub dimensions: u32,
    pub is_compatible: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl EmbeddingCompatResult {
    /// Returns true if the embedding config is fully valid.
    pub fn is_valid(&self) -> bool {
        self.is_compatible && self.errors.is_empty()
    }

    /// Generates a human-readable report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Embedding Compatibility Report ({}):\n  Model: {}\n  Dimensions: {}\n  Compatible: {}",
            self.pack_path, self.model, self.dimensions, self.is_compatible
        )];

        if !self.warnings.is_empty() {
            lines.push("  Warnings:".to_string());
            for w in &self.warnings {
                lines.push(format!("    - {}", w));
            }
        }

        if !self.errors.is_empty() {
            lines.push("  Errors:".to_string());
            for e in &self.errors {
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

    fn create_metadata_with_model(tmp: &TempDir, model: &str, dims: u32) {
        fs::write(
            tmp.path().join("metadata.yaml"),
            format!("pack_name: test\npack_version: '1.0.0'\ndescription: test\nembedding_model: {}\nembedding_dimensions: {}\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n", model, dims),
        )
        .unwrap();
    }

    #[test]
    fn test_known_model_correct_dims() {
        let tmp = TempDir::new().unwrap();
        create_metadata_with_model(&tmp, "all-MiniLM-L6-v2", 384);

        let result = validate_embedding_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.is_valid());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_known_model_wrong_dims() {
        let tmp = TempDir::new().unwrap();
        create_metadata_with_model(&tmp, "all-MiniLM-L6-v2", 768);

        let result = validate_embedding_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.errors.is_empty());
        assert!(result.warnings.len() >= 1);
    }

    #[test]
    fn test_unknown_model_valid_dims() {
        let tmp = TempDir::new().unwrap();
        create_metadata_with_model(&tmp, "custom-model", 768);

        let result = validate_embedding_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_unknown_model_too_low() {
        let tmp = TempDir::new().unwrap();
        create_metadata_with_model(&tmp, "custom-model", 32);

        let result = validate_embedding_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("too low")));
    }

    #[test]
    fn test_unknown_model_too_high() {
        let tmp = TempDir::new().unwrap();
        create_metadata_with_model(&tmp, "custom-model", 8192);

        let result = validate_embedding_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("too high")));
    }

    #[test]
    fn test_zero_dimensions() {
        let tmp = TempDir::new().unwrap();
        create_metadata_with_model(&tmp, "model", 0);

        let result = validate_embedding_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid());
    }

    #[test]
    fn test_report_format() {
        let tmp = TempDir::new().unwrap();
        create_metadata_with_model(&tmp, "all-MiniLM-L6-v2", 384);

        let result = validate_embedding_compat(tmp.path().to_str().unwrap()).unwrap();
        let report = result.report();
        assert!(report.contains("Embedding Compatibility Report"));
        assert!(report.contains("all-MiniLM-L6-v2"));
    }
}
