//! MCP Skill Manager — consolidated runtime.
//!
//! - Single-process skill module loading
//! - Tool aggregation across all modules
//! - Context bus for cross-skill communication
//! - MCP server bridge (external protocol layer)

pub mod manager;
pub mod module;
pub mod context_bus;