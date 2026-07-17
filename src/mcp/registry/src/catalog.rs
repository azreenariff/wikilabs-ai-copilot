//! MCP Tool Registry — global tool catalog.

pub struct ToolCatalog;

impl ToolCatalog {
    pub fn new() -> Self {
        Self
    }

    pub fn register(&mut self, _tool: wikilabs_mcp::server::ToolDefinition) -> anyhow::Result<()> {
        // TODO: Register tool in global catalog
        anyhow::bail!("Not yet implemented")
    }

    pub fn resolve(&self, _name: &str) -> anyhow::Result<&wikilabs_mcp::server::ToolDefinition> {
        // TODO: Resolve tool by name (namespace format: "skill__tool")
        anyhow::bail!("Not yet implemented")
    }
}