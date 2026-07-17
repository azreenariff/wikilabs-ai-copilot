//! AI Provider trait and concrete implementations.
//!
//! Supports OpenAI, vLLM, and Ollama via a unified interface.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub url: String,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub max_tokens: usize,
    pub context_window: usize,
}

/// An AI message (user/assistant/system) for API requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

/// AI request payload sent to the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub model: String,
    pub messages: Vec<AiMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// AI response payload from the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub model: String,
    pub message: AiMessage,
    pub tool_calls: Vec<ToolCall>,
    pub usage: TokenUsage,
    #[serde(default)]
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// A tool call from the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    pub data: Vec<EmbeddingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub index: usize,
    pub embedding: Vec<f32>,
}

/// Abstraction over AI providers (OpenAI, vLLM, Ollama).
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    fn provider_info(&self) -> ProviderInfo;
    fn model_info(&self) -> ModelInfo;

    async fn chat(&self, request: AiRequest) -> anyhow::Result<AiResponse>;

    /// Streaming chat — returns an async receiver that yields text chunks.
    async fn chat_stream(
        &self,
        request: AiRequest,
    ) -> anyhow::Result<tokio::sync::mpsc::UnboundedReceiver<String>>;

    /// Generate an embedding vector for the given text.
    async fn embed(&self, request: EmbedRequest) -> anyhow::Result<EmbedResponse>;

    /// Count tokens in text (approximate).
    fn count_tokens(&self, text: &str) -> usize;

    /// Maximum context window size in tokens.
    fn max_context_tokens(&self) -> usize;

    /// Whether the provider supports function/tool calling.
    fn supports_tools(&self) -> bool;

    /// Whether the provider supports streaming.
    fn supports_streaming(&self) -> bool;

    /// Whether the provider supports structured/JSON output.
    fn supports_structured_output(&self) -> bool;

    /// Whether the provider supports vision (image inputs).
    fn supports_vision(&self) -> bool;

    /// Health check against the provider endpoint.
    async fn health(&self) -> anyhow::Result<()>;
}

/// OpenAI-compatible provider (works with OpenAI, vLLM, Ollama, etc.).
pub struct OpenAICompatibleProvider {
    info: ProviderInfo,
    model_info: ModelInfo,
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl OpenAICompatibleProvider {
    pub fn new(
        name: &str,
        base_url: &str,
        api_key: &str,
        model: &str,
        max_tokens: usize,
        context_window: usize,
    ) -> Self {
        Self {
            info: ProviderInfo {
                name: name.to_string(),
                url: base_url.to_string(),
                api_version: "v1".to_string(),
            },
            model_info: ModelInfo {
                name: model.to_string(),
                max_tokens,
                context_window,
            },
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for OpenAICompatibleProvider {
    fn provider_info(&self) -> ProviderInfo {
        self.info.clone()
    }

    fn model_info(&self) -> ModelInfo {
        self.model_info.clone()
    }

    async fn chat(&self, request: AiRequest) -> anyhow::Result<AiResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API error {}: {}", status, body));
        }

        let ai_response: AiResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        Ok(ai_response)
    }

    async fn chat_stream(
        &self,
        request: AiRequest,
    ) -> anyhow::Result<tokio::sync::mpsc::UnboundedReceiver<String>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let request = request.clone();

        tokio::spawn(async move {
            let url = format!("{}/chat/completions", base_url);

            let response = match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(format!("[error] {}", e));
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let _ = tx.send(format!("[error] API error: {}", status));
                return;
            }

            // Simple streaming: collect all text chunks
            // Note: Full SSE streaming requires a line parser
            if let Ok(body) = response.text().await {
                // For non-streaming responses, send the whole body
                let _ = tx.send(body);
            }
        });

        Ok(rx)
    }

    async fn embed(&self, request: EmbedRequest) -> anyhow::Result<EmbedResponse> {
        let url = format!("{}/embeddings", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Embedding API error {}: {}", status, body));
        }

        let embed_response: EmbedResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse embedding response: {}", e))?;

        Ok(embed_response)
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Approximate token count: ~4 chars per token
        (text.chars().count() / 4).max(1)
    }

    fn max_context_tokens(&self) -> usize {
        self.model_info.context_window
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_structured_output(&self) -> bool {
        // Depends on model; most modern models support it
        self.model_info.name.contains("gpt-4o")
            || self.model_info.name.contains("gpt-4-turbo")
            || self.model_info.name.contains("sonnet")
            || self.model_info.name.contains("claude")
    }

    fn supports_vision(&self) -> bool {
        self.model_info.name.contains("vision")
            || self.model_info.name.contains("gpt-4o")
            || self.model_info.name.contains("gpt-4-turbo")
    }

    async fn health(&self) -> anyhow::Result<()> {
        // Health check: ping the models endpoint
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Provider health check failed: {}",
                response.status()
            ));
        }

        Ok(())
    }
}