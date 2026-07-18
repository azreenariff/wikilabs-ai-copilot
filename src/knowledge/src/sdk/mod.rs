//! Knowledge SDK — tools for creating, packaging, validating, and testing knowledge packs.
//!
//! - `create_pack`: scaffolds new knowledge pack directories
//! - `templates`: predefined pack templates
//! - `validate`: SDK-level validation of pack structure
//! - `packager`: produces .wkl archive files
//! - `testing`: utilities for testing knowledge packs
//! - `schema`: JSON schema definitions for pack manifests

pub mod schema;
pub mod templates;
pub mod create_pack;
pub mod validate;
pub mod packager;
pub mod testing;

pub use schema::*;
pub use templates::*;
pub use create_pack::*;
pub use packager::*;