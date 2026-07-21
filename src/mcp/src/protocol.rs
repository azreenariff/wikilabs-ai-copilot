//! MCP protocol definition — internal abstraction.

use std::future::Future;

pub trait McpProtocol: Send + Sync {
    fn initialize(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn list_resources(&self) -> impl Future<Output = anyhow::Result<Vec<ResourceDefinition>>> + Send;
    fn read_resource(&self, _uri: &str) -> impl Future<Output = anyhow::Result<String>> + Send;
    fn list_prompts(&self) -> impl Future<Output = anyhow::Result<Vec<PromptDefinition>>> + Send;
    fn get_prompt(
        &self,
        _name: &str,
        _arguments: serde_json::Value,
    ) -> impl Future<Output = anyhow::Result<Vec<PromptMessage>>> + Send;
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