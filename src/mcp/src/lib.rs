//! MCP protocol bridge.
//!
//! - Exposes consolidated skill runtime as MCP server
//! - Implements draft MCP spec (2024-11-05)
//! - Internal protocol abstraction (MCP-agnostic)

pub mod server;
pub mod protocol;
pub mod transport;