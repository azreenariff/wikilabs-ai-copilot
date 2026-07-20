//! MCP Tool Registry.
//!
//! - Global tool catalog from all loaded modules
//! - Namespace resolution: "openshift__list_pods"
//! - Dynamic tool registration per skill module

pub mod catalog;
pub mod resolver;
