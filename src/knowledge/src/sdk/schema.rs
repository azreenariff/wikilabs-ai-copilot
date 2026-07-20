//! JSON schema definitions for knowledge pack manifest.yaml and metadata.yaml.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ── Errors ──────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("invalid manifest: {0}")]
    Manifest(String),

    #[error("invalid metadata: {0}")]
    Metadata(String),
}

// ── Manifest Schema ────────────────────────────────────────────────

/// Top-level manifest.yaml structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub format_version: String,
    pub documents: Vec<DocumentManifest>,
    pub dependencies: Vec<String>,
}

impl Manifest {
    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.schema_version != "1.0" {
            return Err(SchemaError::Manifest(format!(
                "unsupported schema version: {}, expected 1.0",
                self.schema_version
            )));
        }
        if self.name.is_empty() {
            return Err(SchemaError::Manifest("name must not be empty".to_string()));
        }
        if self.version.is_empty() {
            return Err(SchemaError::Manifest(
                "version must not be empty".to_string(),
            ));
        }
        if self.license.is_empty() {
            return Err(SchemaError::Manifest(
                "license must not be empty".to_string(),
            ));
        }
        for doc in &self.documents {
            if doc.id.is_empty() {
                return Err(SchemaError::Manifest(
                    "document id must not be empty".to_string(),
                ));
            }
            if doc.path.is_empty() {
                return Err(SchemaError::Manifest(format!(
                    "document '{}' path must not be empty",
                    doc.id
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentManifest {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub embed: bool,
}

// ── Metadata Schema ────────────────────────────────────────────────

/// Top-level metadata.yaml structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub pack_name: String,
    pub pack_version: String,
    pub description: String,
    pub embedding_model: String,
    #[serde(default)]
    pub embedding_dimensions: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub references: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Metadata {
    pub fn new(
        pack_name: &str,
        pack_version: &str,
        description: &str,
        embedding_model: &str,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            pack_name: pack_name.to_string(),
            pack_version: pack_version.to_string(),
            description: description.to_string(),
            embedding_model: embedding_model.to_string(),
            embedding_dimensions: 384,
            tags: Vec::new(),
            categories: Vec::new(),
            references: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.pack_name.is_empty() {
            return Err(SchemaError::Metadata(
                "pack_name must not be empty".to_string(),
            ));
        }
        if self.embedding_model.is_empty() {
            return Err(SchemaError::Metadata(
                "embedding_model must not be empty".to_string(),
            ));
        }
        if self.embedding_dimensions == 0 {
            return Err(SchemaError::Metadata(
                "embedding_dimensions must be positive".to_string(),
            ));
        }
        Ok(())
    }
}

// ── JSON Schema Constants ──────────────────────────────────────────

/// JSON Schema draft-07 for manifest.yaml.
pub fn manifest_json_schema() -> String {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["schema_version", "name", "version", "description", "author", "license", "format_version", "documents"],
        "properties": {
            "schema_version": { "type": "string", "enum": ["1.0"] },
            "name": { "type": "string", "minLength": 1 },
            "version": { "type": "string", "minLength": 1 },
            "description": { "type": "string" },
            "author": { "type": "string" },
            "license": { "type": "string" },
            "format_version": { "type": "string" },
            "documents": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["id", "path"],
                    "properties": {
                        "id": { "type": "string", "minLength": 1 },
                        "path": { "type": "string", "minLength": 1 },
                        "format": { "type": "string" },
                        "embed": { "type": "boolean" }
                    }
                }
            },
            "dependencies": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "additionalProperties": false
    })
    .to_string()
}

/// JSON Schema draft-07 for metadata.yaml.
pub fn metadata_json_schema() -> String {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["pack_name", "pack_version", "description", "embedding_model", "embedding_dimensions"],
        "properties": {
            "pack_name": { "type": "string", "minLength": 1 },
            "pack_version": { "type": "string", "minLength": 1 },
            "description": { "type": "string" },
            "embedding_model": { "type": "string" },
            "embedding_dimensions": { "type": "integer", "minimum": 1 },
            "tags": { "type": "array", "items": { "type": "string" } },
            "categories": { "type": "array", "items": { "type": "string" } },
            "references": { "type": "array", "items": { "type": "string" } },
            "created_at": { "type": "string", "format": "date-time" },
            "updated_at": { "type": "string", "format": "date-time" }
        },
        "additionalProperties": false
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> Manifest {
        Manifest {
            schema_version: "1.0".to_string(),
            name: "test-pack".to_string(),
            version: "1.0.0".to_string(),
            description: "A test pack".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            format_version: "1.0".to_string(),
            documents: vec![DocumentManifest {
                id: "doc1".to_string(),
                path: "documents/doc1.md".to_string(),
                format: "markdown".to_string(),
                embed: true,
            }],
            dependencies: vec![],
        }
    }

    fn valid_metadata() -> Metadata {
        Metadata::new("test-pack", "1.0.0", "A test pack", "all-MiniLM-L6-v2")
    }

    #[test]
    fn test_valid_manifest() {
        assert!(valid_manifest().validate().is_ok());
    }

    #[test]
    fn test_empty_name_rejected() {
        let mut m = valid_manifest();
        m.name = String::new();
        assert!(m.validate().is_err());
    }

    #[test]
    fn test_valid_metadata() {
        assert!(valid_metadata().validate().is_ok());
    }

    #[test]
    fn test_empty_model_rejected() {
        let mut meta = valid_metadata();
        meta.embedding_model = String::new();
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_json_schema_not_empty() {
        let schema = manifest_json_schema();
        assert!(schema.contains("schema_version"));
        assert!(schema.contains("documents"));
    }

    #[test]
    fn test_metadata_json_schema_not_empty() {
        let schema = metadata_json_schema();
        assert!(schema.contains("pack_name"));
        assert!(schema.contains("embedding_model"));
    }
}
