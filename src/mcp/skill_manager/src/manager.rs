//! MCP Skill Manager — consolidated runtime.

pub struct SkillManager;

impl SkillManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn load_module(&mut self, _name: &str) -> anyhow::Result<()> {
        // TODO: Load skill module
        anyhow::bail!("Not yet implemented")
    }

    pub async fn unload_module(&mut self, _name: &str) -> anyhow::Result<()> {
        // TODO: Unload skill module
        anyhow::bail!("Not yet implemented")
    }

    pub async fn call_tool(&self, _name: &str, _args: serde_json::Value) -> anyhow::Result<wikilabs_mcp::server::ToolResult> {
        // TODO: Route to appropriate module
        anyhow::bail!("Not yet implemented")
    }

    pub async fn list_tools(&self) -> anyhow::Result<Vec<wikilabs_mcp::server::ToolDefinition>> {
        // TODO: Aggregate all tools
        anyhow::bail!("Not yet implemented")
    }
}