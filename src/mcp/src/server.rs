//! MCP Protocol bridge — draft spec (2024-11-05).

pub struct McpServer;

impl McpServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        // TODO: Implement MCP handshake
        anyhow::bail!("Not yet implemented")
    }

    pub async fn list_tools(&self) -> anyhow::Result<Vec<ToolDefinition>> {
        // TODO: Aggregate tools from all skill modules
        anyhow::bail!("Not yet implemented")
    }

    pub async fn call_tool(
        &self,
        _name: &str,
        _arguments: serde_json::Value,
    ) -> anyhow::Result<ToolResult> {
        // TODO: Route to skill module
        anyhow::bail!("Not yet implemented")
    }
}

#[derive(Clone, Debug)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct ToolResult {
    pub content: serde_json::Value,
    pub is_error: bool,
}
