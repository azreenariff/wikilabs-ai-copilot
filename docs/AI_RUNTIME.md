# AI Runtime

## Overview

The AI Runtime is the communication layer between Wiki Labs AI Copilot and configured AI providers. It provides a unified interface for sending prompts and receiving streaming responses, regardless of the underlying provider.

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Prompt      │────▶│  AI Runtime  │────▶│  Provider   │
│  Manager     │     │  (Client)    │────▶│  (OpenAI,   │
│              │     │              │     │   vLLM, etc)│
└─────────────┘     └──────────────┘     └─────────────┘
                          │
                          ▼
                    ┌──────────────┐
                    │  Streaming   │
                    │  Response    │
                    └──────────────┘
```

## Features

### Provider Abstraction

The runtime supports multiple AI providers through a unified interface:

- **OpenAI**: Standard OpenAI-compatible API endpoints
- **OpenAI-Compatible APIs**: Any API following the OpenAI message format
- **vLLM**: Self-hosted inference with vLLM
- **Future Providers**: Extensible design for adding new providers

### Provider Configuration

Each provider is configured with:
- Base URL (API endpoint)
- API key (secret, stored securely)
- Model name
- Timeout settings
- Optional headers for custom providers

### Streaming Responses

The runtime supports streaming responses, allowing the UI to display
progressive output as tokens are generated. This provides a better
user experience for long-running generations.

### Retry Logic

Failed API calls are automatically retried with exponential backoff:
- Transient network errors: retry up to 3 times
- Rate limit errors: respect retry-after headers
- Invalid responses: immediate failure (no retry)

### Timeout Handling

- Configurable per-request timeouts (default: 60 seconds)
- Stream timeouts for long-running generations
- Overall session timeout limits

### Cancellation

Streaming responses can be cancelled mid-generation:
- Cancellation propagates to the underlying HTTP request
- Partial responses are returned up to the cancellation point
- Clean shutdown with proper resource cleanup

### Health Checks

The runtime supports periodic health checks:
- Provider availability verification
- API key validation
- Model availability checks
- Latency monitoring

## API Reference

### `AiProvider` Trait

```rust
pub trait AiProvider: Send + Sync + Debug {
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
    fn max_context_tokens(&self) -> usize;
    fn supports_tools(&self) -> bool;
    fn supports_streaming(&self) -> bool;
    fn supports_structured_output(&self) -> bool;
    fn supports_vision(&self) -> bool;
    async fn health(&self) -> anyhow::Result<()>;
}
```

### `Message` Type

```rust
pub struct Message {
    pub role: MessageRole,  // System, User, Assistant
    pub content: String,
}
```

### `GenerationConfig`

```rust
pub struct GenerationConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
    pub presence_penalty: f32,
    pub frequency_penalty: f32,
    pub timeout_seconds: u64,
    pub stream: bool,
}
```

### `Response` Types

```rust
pub struct Chunk {
    pub content: String,
    pub stop_reason: Option<StopReason>,
    pub usage: Option<TokenUsage>,
}

pub struct CompleteResponse {
    pub content: String,
    pub stop_reason: StopReason,
    pub usage: TokenUsage,
}

pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}
```

## Error Handling

The runtime defines a comprehensive error type:

```rust
pub enum AIError {
    /// Provider returned an error (invalid API key, etc.)
    ProviderError(String),
    /// Network timeout
    Timeout,
    /// Request was cancelled
    Cancelled,
    /// Rate limit exceeded
    RateLimit,
    /// Malformed response from provider
    ParseError(String),
    /// Provider is unhealthy
    Unhealthy(String),
}
```

## Example Usage

```rust
// Create an OpenAI-compatible provider
let provider = OpenAICompatibleProvider::builder()
    .base_url("https://api.openai.com/v1")
    .api_key("sk-...")
    .model("gpt-4")
    .build()?;

// Stream a response
let mut stream = provider.stream(
    &[
        Message::system("You are a helpful assistant."),
        Message::user("Explain Rust ownership."),
    ],
    &GenerationConfig {
        model: "gpt-4".to_string(),
        temperature: 0.7,
        max_tokens: 1024,
        ..Default::default()
    },
).await?;

// Process streaming chunks
while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(Chunk { content, .. }) => print!("{}", content),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Testing

The runtime supports testing through mock providers:

```rust
#[tokio::test]
async fn test_streaming_response() {
    let mock = MockProvider::builder()
        .with_response("Rust ownership is a safety feature.")
        .build();

    let response = mock.complete(&messages, &config).await;
    assert!(response.is_ok());
}
```