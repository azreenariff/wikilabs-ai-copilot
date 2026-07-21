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
        // Stub: placeholder. Return a mock AI response for testing.
        unimplemented!()
    }

    pub async fn embed(
        &self,
        _request: wikilabs_ai::provider::EmbedRequest,
    ) -> anyhow::Result<wikilabs_ai::provider::EmbedResponse> {
        // Stub: placeholder. Return a mock embedding response for testing.
        unimplemented!()
    }
}
