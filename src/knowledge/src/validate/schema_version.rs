//! Validate schema version compatibility.

use crate::sdk::schema::Manifest;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Current supported schema version.
pub const CURRENT_SCHEMA_VERSION: &str = "1.0";

/// Validates that the manifest schema version is compatible.
///
/// Checks:
/// - schema_version is a supported version
/// - schema_version is not empty
/// - Returns a warning if schema_version is deprecated
pub fn validate_schema_version(pack_path: &str) -> Result<SchemaVersionResult> {
    let path = Path::new(pack_path);
    let manifest_path = path.join("manifest.yaml");

    let content = fs::read_to_string(&manifest_path)
        .context("Failed to read manifest.yaml for schema version validation")?;

    let manifest: Manifest =
        serde_yaml::from_str(&content).context("manifest.yaml is not valid YAML")?;

    let mut result = SchemaVersionResult {
        pack_path: pack_path.to_string(),
        actual_version: manifest.schema_version.clone(),
        supported: true,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    let version = &manifest.schema_version;

    if version.is_empty() {
        result.errors.push("schema_version is empty".to_string());
        result.supported = false;
        return Ok(result);
    }

    if version == CURRENT_SCHEMA_VERSION {
        debug!(version = version, "Schema version is current");
        return Ok(result);
    }

    // Check for deprecated versions
    if version.starts_with("0.") || version == "1.0-beta" {
        result
            .errors
            .push(format!("schema version '{}' is deprecated", version));
        result.warnings.push(format!(
            "schema version '{}' is deprecated, consider upgrading to '{}'",
            version, CURRENT_SCHEMA_VERSION
        ));
        result.supported = false;
    } else if !version.starts_with("1.") {
        result.errors.push(format!(
            "unsupported schema version '{}', expected '{}'",
            version, CURRENT_SCHEMA_VERSION
        ));
        result.supported = false;
    } else {
        // Unknown version in 1.x range — warning only
        result.warnings.push(format!(
            "schema version '{}' not verified, may be incompatible with '{}'",
            version, CURRENT_SCHEMA_VERSION
        ));
    }

    debug!(
        version = version,
        supported = result.supported,
        "Schema version check completed"
    );

    Ok(result)
}

/// Schema version validation result.
#[derive(Debug, Default)]
pub struct SchemaVersionResult {
    pub pack_path: String,
    pub actual_version: String,
    pub supported: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl SchemaVersionResult {
    /// Returns true if the schema version is fully compatible.
    pub fn is_valid(&self) -> bool {
        self.supported && self.errors.is_empty()
    }

    /// Generates a human-readable report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Schema Version Report ({}):\n  Actual version: {}\n  Supported: {}",
            self.pack_path, self.actual_version, self.supported
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

    fn create_manifest_with_version(tmp: &TempDir, version: &str) {
        fs::write(
            tmp.path().join("manifest.yaml"),
            format!("schema_version: '{}'\nname: test\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n", version),
        )
        .unwrap();
    }

    #[test]
    fn test_current_version() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_version(&tmp, "1.0");

        let result = validate_schema_version(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.is_valid());
        assert!(result.supported);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_deprecated_version() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_version(&tmp, "0.9");

        let result = validate_schema_version(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.errors.is_empty());
        assert!(result.warnings.len() >= 1);
    }

    #[test]
    fn test_unsupported_version() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_version(&tmp, "2.0");

        let result = validate_schema_version(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid());
        assert!(!result.supported);
    }

    #[test]
    fn test_empty_version() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_version(&tmp, "");

        let result = validate_schema_version(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn test_report_format() {
        let tmp = TempDir::new().unwrap();
        create_manifest_with_version(&tmp, "1.0");

        let result = validate_schema_version(tmp.path().to_str().unwrap()).unwrap();
        let report = result.report();
        assert!(report.contains("Schema Version Report"));
        assert!(report.contains("1.0"));
    }
}
