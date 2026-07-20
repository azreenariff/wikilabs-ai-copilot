//! Knowledge validation framework — comprehensive pack validation.
//!
//! Validates knowledge packs at multiple levels:
//! - manifest.yaml schema compliance
//! - metadata.yaml schema compliance
//! - Document existence and readability
//! - Embedding compatibility
//! - Schema version compatibility
//! - Duplicate identifiers
//! - Dependencies
//! - Broken internal references
//! - Version compatibility
//! - Generates human-readable validation reports

pub mod broken_refs;
pub mod dependencies;
pub mod documents;
pub mod duplicate_id;
pub mod embedding_compat;
pub mod manifest;
pub mod metadata;
pub mod report;
pub mod schema_version;
pub mod version_compat;

pub use broken_refs::check_broken_refs;
pub use dependencies::validate_dependencies;
pub use documents::validate_documents;
pub use duplicate_id::check_duplicate_ids;
pub use embedding_compat::validate_embedding_compat;
pub use manifest::validate_manifest;
pub use metadata::validate_metadata;
pub use report::{ValidationReport, ValidationResult, ValidationResult as ValidationStatus, validate_pack_comprehensive};
pub use schema_version::validate_schema_version;
pub use version_compat::validate_version_compat;
