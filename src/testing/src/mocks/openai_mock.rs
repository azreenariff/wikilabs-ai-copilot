//! Mock OpenAI provider for unit testing.

pub struct OpenAIMock;

impl OpenAIMock {
    pub fn new() -> Self {
        Self
    }

    pub async fn chat(
        &self,
        _request: wikilabs_ai::provider::AiRequest,
    ) -> anyhow::Result<wikilabs_ai::provider::AiResponse> {
        // TODO: Return mock response
        anyhow::bail!("Not yet implemented")
    }

    pub async fn embed(
        &self,
        _request: wikilabs_ai::provider::EmbedRequest,
    ) -> anyhow::Result<wikilabs_ai::provider::EmbedResponse> {
        // TODO: Return mock embedding
        anyhow::bail!("Not yet implemented")
    }
}
