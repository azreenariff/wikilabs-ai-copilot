//! Knowledge SDK README.

//! # Wiki Labs Knowledge SDK
//!
//! The Knowledge SDK provides tools for creating, packaging, validating, and testing
//! knowledge packs (.wkl files) used by the Wiki Labs AI Copilot.
//!
//! ## Components
//!
//! - **Templates** — Predefined knowledge pack templates (OpenShift, Engineering, Documentation)
//! - **Create Pack** — Scaffolds new knowledge pack directories from templates
//! - **Validate** — SDK-level validation of pack structure and contents
//! - **Packager** — Creates and extracts .wkl archive files
//! - **Testing** — Runs a battery of tests on knowledge packs
//! - **Schema** — JSON schema definitions for manifest.yaml and metadata.yaml
//!
//! ## Quick Start
//!
//! ```rust
//! use wikilabs_knowledge::sdk::*;
//!
//! // Create a new knowledge pack
//! let output_dir = "/tmp/knowledge-packs";
//! let pack_path = create_openshift(output_dir).unwrap();
//!
//! // Validate a pack
//! let result = validate::validate_pack(&pack_path).unwrap();
//! assert!(result.is_valid);
//!
//! // Package a pack
//! package_pack(&pack_path, "/tmp/knowledge-pack.wkl").unwrap();
//!
//! // Test a pack
//! let test_results = testing::test_pack(&pack_path);
//! assert!(test_results.all_passed());
//!
//! // Extract a pack
//! extract_pack("/tmp/knowledge-pack.wkl", "/tmp/extracted").unwrap();
//! ```
//!
//! ## Knowledge Pack Structure
//!
//! ```text
//! my-knowledge-pack/
//! ├── manifest.yaml          # Pack manifest with document list and dependencies
//! ├── metadata.yaml          # Pack metadata (embedding config, tags, categories)
//! ├── documents/             # Knowledge documents (markdown, PDF, text)
//! │   ├── doc1.md
//! │   └── doc2.md
//! ├── tests/                 # Validation and integration tests
//! │   └── validation.test
//! └── documentation/         # Additional pack documentation
//!     └── README.md
//! ```
//!
//! ## Template System
//!
//! The SDK includes three predefined templates:
//!
//! - **openshift** — Knowledge pack for OpenShift/container platform documentation
//! - **engineering** — Engineering best practices and design patterns
//! - **documentation** — Project documentation and API references
//!
//! Templates can be created via:
//! - `create_openshift(output_dir)` — Creates an OpenShift knowledge pack
//! - `create_pack("openshift", output_dir, Some("custom-name"))` — Creates a pack with custom name
//!
//! ## Validation
//!
//! The validation tool checks:
//! 1. manifest.yaml exists and is valid YAML
//! 2. metadata.yaml exists and is valid YAML
//! 3. documents/ directory exists
//! 4. All documents referenced in manifest.yaml exist as files
//! 5. Embedding configuration is valid
//!
//! ## Schema
//!
//! manifest.yaml schema:
//! - `schema_version`: Must be "1.0"
//! - `name`: Pack name (non-empty string)
//! - `version`: Semantic version string
//! - `description`: Human-readable description
//! - `author`: Pack author
//! - `license`: SPDX license identifier
//! - `format_version`: Format version string
//! - `documents`: Array of document entries (id, path, format, embed)
//! - `dependencies`: Array of pack name strings
//!
//! metadata.yaml schema:
//! - `pack_name`: Must match manifest name
//! - `pack_version`: Must match manifest version
//! - `description`: Human-readable description
//! - `embedding_model`: Model identifier (e.g., "all-MiniLM-L6-v2")
//! - `embedding_dimensions`: Vector dimensions (must be positive)
//! - `tags`: Optional tags array
//! - `categories`: Optional categories array
//! - `references`: Optional reference URLs
//! - `created_at`: ISO 8601 timestamp
//! - `updated_at`: ISO 8601 timestamp
//!
//! ## Archive Format (.wkl)
//!
//! Knowledge pack archives are binary files with the following structure:
//! - 4-byte entry count (little-endian u32)
//! - For each entry:
//!   - 4-byte path length (little-endian u32)
//!   - Path bytes (UTF-8 string)
//!   - 4-byte data length (little-endian u32)
//!   - Data bytes (file contents)

#![doc = include_str!("README.md")]