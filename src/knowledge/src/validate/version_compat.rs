//! Validate version compatibility for a knowledge pack.

use crate::sdk::schema::{Manifest, Metadata};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Validates that the pack version is compatible with the pack name and metadata.
///
/// Checks:
/// - Manifest version matches metadata pack_version
/// - Manifest name matches metadata pack_name
/// - Version follows semantic versioning (major.minor.patch)
/// - Version is not empty
pub fn validate_version_compat(pack_path: &str) -> Result<VersionCompatResult> {
    let path = Path::new(pack_path);

    // Load manifest
    let manifest_content = fs::read_to_string(path.join("manifest.yaml"))
        .context("Failed to read manifest.yaml for version compatibility check")?;

    let manifest: Manifest =
        serde_yaml::from_str(&manifest_content).context("manifest.yaml is not valid YAML")?;

    // Load metadata
    let metadata_content = fs::read_to_string(path.join("metadata.yaml"))
        .context("Failed to read metadata.yaml for version compatibility check")?;

    let metadata: Metadata =
        serde_yaml::from_str(&metadata_content).context("metadata.yaml is not valid YAML")?;

    let mut result = VersionCompatResult {
        pack_path: pack_path.to_string(),
        valid: true,
        name_mismatch: false,
        version_mismatch: false,
        invalid_semver: false,
        warnings: Vec::new(),
    };

    // Check name consistency
    if manifest.name != metadata.pack_name {
        result.name_mismatch = true;
        result.valid = false;
        debug!(
            manifest_name = %manifest.name,
            metadata_name = %metadata.pack_name,
            "Name mismatch between manifest and metadata"
        );
    }

    // Check version consistency
    if manifest.version != metadata.pack_version {
        result.version_mismatch = true;
        result.valid = false;
        debug!(
            manifest_version = %manifest.version,
            metadata_version = %metadata.pack_version,
            "Version mismatch between manifest and metadata"
        );
    }

    // Check semantic versioning
    let manifest_ver = &manifest.version;
    let metadata_ver = &metadata.pack_version;

    if manifest_ver.is_empty() {
        result.invalid_semver = true;
        result.valid = false;
        result.warnings.push("manifest version is empty".to_string());
    }

    if metadata_ver.is_empty() {
        result.invalid_semver = true;
        result.valid = false;
        result.warnings.push("metadata version is empty".to_string());
    }

    if !is_semver(manifest_ver) {
        result.invalid_semver = true;
        result.valid = false;
        result
            .warnings
            .push(format!("manifest version '{}' is not valid semver", manifest_ver));
    }

    if !is_semver(metadata_ver) {
        result.invalid_semver = true;
        result.valid = false;
        result
            .warnings
            .push(format!("metadata version '{}' is not valid semver", metadata_ver));
    }

    debug!(
        valid = result.valid,
        name_match = !result.name_mismatch,
        version_match = !result.version_mismatch,
        semver_valid = !result.invalid_semver,
        "Version compatibility check completed"
    );

    Ok(result)
}

/// Checks if a string follows semantic versioning (major.minor.patch).
fn is_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| {
        // Strip any pre-release or build metadata suffixes
        let base = p.split('-').next().unwrap_or(p);
        let base = base.split('+').next().unwrap_or(base);
        !base.is_empty() && base.parse::<u64>().is_ok()
    })
}

/// Version compatibility check result.
#[derive(Debug, Default)]
pub struct VersionCompatResult {
    pub pack_path: String,
    pub valid: bool,
    pub name_mismatch: bool,
    pub version_mismatch: bool,
    pub invalid_semver: bool,
    pub warnings: Vec<String>,
}

impl VersionCompatResult {
    /// Generates a human-readable report.
    pub fn report(&self) -> String {
        let mut lines = vec![format!(
            "Version Compatibility Report ({}):\n  Valid: {}\n  Name mismatch: {}\n  Version mismatch: {}\n  Invalid semver: {}",
            self.pack_path, self.valid, self.name_mismatch, self.version_mismatch, self.invalid_semver
        )];

        if !self.warnings.is_empty() {
            lines.push("  Warnings:".to_string());
            for w in &self.warnings {
                lines.push(format!("    - {}", w));
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

    fn create_matching_pack(tmp: &TempDir) {
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test-pack\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("metadata.yaml"), "pack_name: test-pack\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n").unwrap();
    }

    #[test]
    fn test_matching_versions() {
        let tmp = TempDir::new().unwrap();
        create_matching_pack(&tmp);

        let result = validate_version_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(result.valid);
        assert!(!result.name_mismatch);
        assert!(!result.version_mismatch);
        assert!(!result.invalid_semver);
    }

    #[test]
    fn test_name_mismatch() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: pack-a\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("metadata.yaml"), "pack_name: pack-b\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n").unwrap();

        let result = validate_version_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.valid);
        assert!(result.name_mismatch);
    }

    #[test]
    fn test_version_mismatch() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test-pack\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("metadata.yaml"), "pack_name: test-pack\npack_version: '2.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n").unwrap();

        let result = validate_version_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.valid);
        assert!(result.version_mismatch);
    }

    #[test]
    fn test_invalid_semver() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test-pack\nversion: 'abc'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments: []\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("metadata.yaml"), "pack_name: test-pack\npack_version: 'abc'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n").unwrap();

        let result = validate_version_compat(tmp.path().to_str().unwrap()).unwrap();
        assert!(!result.valid);
        assert!(result.invalid_semver);
    }

    #[test]
    fn test_is_semver_valid() {
        assert!(is_semver("1.0.0"));
        assert!(is_semver("0.1.0"));
        assert!(is_semver("10.20.30"));
        assert!(is_semver("1.0.0-alpha"));
        assert!(is_semver("1.0.0+build"));
    }

    #[test]
    fn test_is_semver_invalid() {
        assert!(!is_semver("1.0"));
        assert!(!is_semver("v1.0.0"));
        assert!(!is_semver("abc"));
        assert!(!is_semver("1.0.0.0"));
        assert!(!is_semver(""));
    }

    #[test]
    fn test_report_format() {
        let tmp = TempDir::new().unwrap();
        create_matching_pack(&tmp);

        let result = validate_version_compat(tmp.path().to_str().unwrap()).unwrap();
        let report = result.report();
        assert!(report.contains("Version Compatibility Report"));
    }
}