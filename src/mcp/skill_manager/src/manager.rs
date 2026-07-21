//! MCP Skill Manager — consolidated runtime.

pub struct SkillManager;

impl SkillManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn load_module(&mut self, _name: &str) -> anyhow::Result<()> {
        // Stub: placeholder. Load skill module.
        unimplemented!()
    }

    pub async fn unload_module(&mut self, _name: &str) -> anyhow::Result<()> {
        // Stub: placeholder. Unload skill module.
        unimplemented!()
    }

    pub async fn call_tool(
        &self,
        _name: &str,
        _args: serde_json::Value,
    ) -> anyhow::Result<wikilabs_mcp::server::ToolResult> {
        // Stub: placeholder. Route to appropriate module.
        unimplemented!()
    }

    pub async fn list_tools(&self) -> anyhow::Result<Vec<wikilabs_mcp::server::ToolDefinition>> {
        // Stub: placeholder. Aggregate all tools.
        unimplemented!()
    }
}
