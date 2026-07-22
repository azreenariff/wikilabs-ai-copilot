//! MCP Tool Registry — global tool catalog.

pub struct ToolCatalog;

impl Default for ToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolCatalog {
    pub fn new() -> Self {
        Self
    }

    pub fn register(&mut self, _tool: wikilabs_mcp::server::ToolDefinition) -> anyhow::Result<()> {
        // Stub: placeholder. Register tool in global catalog.
        unimplemented!()
    }

    pub fn resolve(&self, _name: &str) -> anyhow::Result<&wikilabs_mcp::server::ToolDefinition> {
        // Stub: placeholder. Resolve tool by name (namespace format: "skill__tool").
        unimplemented!()
    }
}
