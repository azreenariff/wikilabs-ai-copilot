//! MCP protocol definition — internal abstraction.

pub trait McpProtocol: Send + Sync {
    async fn initialize(&self) -> anyhow::Result<()>;
    async fn list_resources(&self) -> anyhow::Result<Vec<ResourceDefinition>>;
    async fn read_resource(&self, _uri: &str) -> anyhow::Result<String>;
    async fn list_prompts(&self) -> anyhow::Result<Vec<PromptDefinition>>;
    async fn get_prompt(&self, _name: &str, _arguments: serde_json::Value) -> anyhow::Result<Vec<PromptMessage>>;
}

#[derive(Clone, Debug)]
pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct PromptDefinition {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Clone, Debug)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct PromptMessage {
    pub role: String,
    pub content: String,
}