//! Tool definition and result types for function calling.

use serde::{Deserialize, Serialize};

/// Definition of a tool the AI can invoke.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    /// Unique name of the tool.
    pub name: String,
    /// Description of what the tool does.
    pub description: String,
    /// JSON schema describing the tool's parameters.
    pub parameters: serde_json::Value,
}

/// A tool invocation request from the AI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// Name of the tool to call.
    pub name: String,
    /// Arguments to pass to the tool.
    pub arguments: serde_json::Value,
}

/// Result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    /// The tool's output (as JSON value for complex results).
    pub content: serde_json::Value,
    /// Whether the tool execution failed.
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful result with a text message.
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: serde_json::Value::String(content.into()),
            is_error: false,
        }
    }

    /// Create an error result.
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: serde_json::Value::String(content.into()),
            is_error: true,
        }
    }

    /// Create a result with structured JSON data.
    pub fn json(data: serde_json::Value) -> Self {
        Self {
            content: data,
            is_error: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("Done");
        assert!(!result.is_error);
        assert_eq!(result.content, "Done");
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Failed");
        assert!(result.is_error);
    }
}