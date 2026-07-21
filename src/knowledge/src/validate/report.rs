//! Generate validation reports for knowledge packs.

use crate::validate::broken_refs::check_broken_refs;
use crate::validate::dependencies::validate_dependencies;
use crate::validate::documents::validate_documents;
use crate::validate::duplicate_id::check_duplicate_ids;
use crate::validate::embedding_compat::validate_embedding_compat;
use crate::validate::manifest::validate_manifest;
use crate::validate::metadata::validate_metadata;
use crate::validate::schema_version::validate_schema_version;
use crate::validate::version_compat::validate_version_compat;
use anyhow::Result;
use tracing::debug;

/// Overall validation status of a knowledge pack.
#[derive(Debug, PartialEq, Clone)]
pub enum ValidationResult {
    /// All validations passed.
    Valid,
    /// Some validations failed with errors.
    Invalid {
        error_count: usize,
        warning_count: usize,
    },
    /// Validation itself encountered an error.
    Error { message: String },
}

/// Comprehensive validation report for a knowledge pack.
#[derive(Debug)]
pub struct ValidationReport {
    pub pack_path: String,
    pub overall_status: ValidationResult,
    pub manifest_ok: bool,
    pub metadata_ok: bool,
    pub documents_ok: bool,
    pub embedding_ok: bool,
    pub schema_version_ok: bool,
    pub duplicate_ids_ok: bool,
    pub dependencies_ok: bool,
    pub broken_refs_ok: bool,
    pub version_compat_ok: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    /// Generates a human-readable validation report.
    pub fn summary(&self) -> String {
        let status = match &self.overall_status {
            ValidationResult::Valid => "VALID",
            ValidationResult::Invalid { .. } => "INVALID",
            ValidationResult::Error { .. } => "ERROR",
        };

        let mut lines = vec![format!(
            "Knowledge Pack Validation Report: {}\n  Pack: {}\n  Manifest: {}\n  Metadata: {}\n  Documents: {}\n  Embedding: {}\n  Schema Version: {}\n  Duplicate IDs: {}\n  Dependencies: {}\n  Broken Refs: {}\n  Version Compat: {}\n  Errors: {}\n  Warnings: {}",
            status,
            self.pack_path,
            self.manifest_ok,
            self.metadata_ok,
            self.documents_ok,
            self.embedding_ok,
            self.schema_version_ok,
            self.duplicate_ids_ok,
            self.dependencies_ok,
            self.broken_refs_ok,
            self.version_compat_ok,
            self.error_count,
            self.warning_count
        )];

        if !self.errors.is_empty() {
            lines.push("  Errors:".to_string());
            for e in &self.errors {
                lines.push(format!("    - {}", e));
            }
        }

        if !self.warnings.is_empty() {
            lines.push("  Warnings:".to_string());
            for w in &self.warnings {
                lines.push(format!("    - {}", w));
            }
        }

        lines.join("\n")
    }

    /// Returns true if the pack is fully valid.
    pub fn is_valid(&self) -> bool {
        matches!(&self.overall_status, ValidationResult::Valid)
    }

    /// Returns true if the pack has warnings but no errors.
    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0 && self.error_count == 0
    }
}

/// Runs all validation checks on a knowledge pack and generates a comprehensive report.
///
/// This is the main entry point for pack validation. It runs all individual validators
/// and aggregates the results into a single report.
pub fn validate_pack_comprehensive(pack_path: &str) -> Result<ValidationReport> {
    debug!(pack_path = %pack_path, "Running comprehensive validation");

    let mut report = ValidationReport {
        pack_path: pack_path.to_string(),
        overall_status: ValidationResult::Valid,
        manifest_ok: false,
        metadata_ok: false,
        documents_ok: false,
        embedding_ok: false,
        schema_version_ok: false,
        duplicate_ids_ok: false,
        dependencies_ok: false,
        broken_refs_ok: false,
        version_compat_ok: false,
        error_count: 0,
        warning_count: 0,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Validate manifest
    match validate_manifest(pack_path) {
        Ok(_) => report.manifest_ok = true,
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("manifest: {}", e));
        }
    }

    // Validate metadata
    match validate_metadata(pack_path) {
        Ok(_) => report.metadata_ok = true,
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("metadata: {}", e));
        }
    }

    // Validate documents
    match validate_documents(pack_path) {
        Ok(doc_result) => {
            report.documents_ok = doc_result.is_valid();
            if !doc_result.is_valid() {
                report.error_count += doc_result.issue_count();
                report
                    .errors
                    .push(format!("documents: {}", doc_result.report()));
            }
        }
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("documents: {}", e));
        }
    }

    // Validate embedding compatibility
    match validate_embedding_compat(pack_path) {
        Ok(embed_result) => {
            report.embedding_ok = embed_result.is_valid();
            report.warning_count += embed_result.warnings.len();
            report.error_count += embed_result.errors.len();
            for w in embed_result.warnings {
                report.warnings.push(format!("embedding: {}", w));
            }
            for e in embed_result.errors {
                report.errors.push(format!("embedding: {}", e));
            }
        }
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("embedding: {}", e));
        }
    }

    // Validate schema version
    match validate_schema_version(pack_path) {
        Ok(sv_result) => {
            report.schema_version_ok = sv_result.is_valid();
            report.warning_count += sv_result.warnings.len();
            report.error_count += sv_result.errors.len();
            for w in sv_result.warnings {
                report.warnings.push(format!("schema_version: {}", w));
            }
            for e in sv_result.errors {
                report.errors.push(format!("schema_version: {}", e));
            }
        }
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("schema_version: {}", e));
        }
    }

    // Check duplicate IDs
    match check_duplicate_ids(pack_path) {
        Ok(dup_result) => {
            report.duplicate_ids_ok = dup_result.is_valid;
            if !dup_result.is_valid {
                report.error_count += 1;
                report
                    .errors
                    .push(format!("duplicate_ids: {}", dup_result.report()));
            }
        }
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("duplicate_ids: {}", e));
        }
    }

    // Validate dependencies
    match validate_dependencies(pack_path) {
        Ok(dep_result) => {
            report.dependencies_ok = dep_result.valid;
            if !dep_result.valid {
                report.error_count += 1;
                report
                    .errors
                    .push(format!("dependencies: {}", dep_result.report()));
            }
        }
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("dependencies: {}", e));
        }
    }

    // Check broken references
    match check_broken_refs(pack_path) {
        Ok(ref_result) => {
            report.broken_refs_ok = ref_result.valid;
            if !ref_result.valid {
                report.error_count += 1;
                report
                    .errors
                    .push(format!("broken_refs: {}", ref_result.report()));
            }
        }
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("broken_refs: {}", e));
        }
    }

    // Validate version compatibility
    match validate_version_compat(pack_path) {
        Ok(vc_result) => {
            report.version_compat_ok = vc_result.valid;
            report.warning_count += vc_result.warnings.len();
            if !vc_result.valid {
                report.error_count += 1;
                report
                    .errors
                    .push(format!("version_compat: {}", vc_result.report()));
            }
            for w in vc_result.warnings {
                report.warnings.push(format!("version_compat: {}", w));
            }
        }
        Err(e) => {
            report.error_count += 1;
            report.errors.push(format!("version_compat: {}", e));
        }
    }

    // Determine overall status
    if report.error_count > 0 {
        report.overall_status = ValidationResult::Invalid {
            error_count: report.error_count,
            warning_count: report.warning_count,
        };
    } else if report.warning_count > 0 {
        report.overall_status = ValidationResult::Invalid {
            error_count: 0,
            warning_count: report.warning_count,
        };
    }

    debug!(
        status = ?report.overall_status,
        errors = report.error_count,
        warnings = report.warning_count,
        "Comprehensive validation completed"
    );

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_valid_pack(tmp: &TempDir) {
        fs::create_dir_all(tmp.path().join("documents")).unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test-pack\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("metadata.yaml"), "pack_name: test-pack\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n").unwrap();
        fs::write(tmp.path().join("documents/doc1.md"), "# Test\n\nContent.\n").unwrap();
    }

    #[test]
    fn test_valid_pack() {
        let tmp = TempDir::new().unwrap();
        create_valid_pack(&tmp);

        let report = validate_pack_comprehensive(tmp.path().to_str().unwrap()).unwrap();
        assert!(report.is_valid());
        assert!(report.manifest_ok);
        assert!(report.metadata_ok);
        assert!(report.documents_ok);
    }

    #[test]
    fn test_invalid_pack() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("manifest.yaml"), "schema_version: '1.0'\nname: test-pack\nversion: '1.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: missing-doc\n    path: missing.md\n    format: markdown\n    embed: true\ndependencies: []\n").unwrap();
        fs::write(tmp.path().join("metadata.yaml"), "pack_name: test-pack\npack_version: '1.0.0'\ndescription: test\nembedding_model: all-MiniLM-L6-v2\nembedding_dimensions: 384\ntags: []\ncategories: []\nreferences: []\ncreated_at: '2024-01-01T00:00:00Z'\nupdated_at: '2024-01-01T00:00:00Z'\n").unwrap();

        let report = validate_pack_comprehensive(tmp.path().to_str().unwrap()).unwrap();
        assert!(!report.is_valid());
        assert!(report.error_count > 0);
    }

    #[test]
    fn test_summary_format() {
        let tmp = TempDir::new().unwrap();
        create_valid_pack(&tmp);

        let report = validate_pack_comprehensive(tmp.path().to_str().unwrap()).unwrap();
        let summary = report.summary();
        assert!(summary.contains("Knowledge Pack Validation Report"));
        assert!(summary.contains("VALID"));
    }
}
