//! Skill Module trait — the interface each skill module must implement.

#[async_trait::async_trait]
pub trait SkillModule: Send + Sync {
    fn id(&self) -> &str;
    fn metadata(&self) -> &str;
    fn tools(&self) -> Vec<wikilabs_mcp::server::ToolDefinition>;
    async fn call_tool(&self, _name: &str, _args: serde_json::Value) -> anyhow::Result<wikilabs_mcp::server::ToolResult>;
}