# AI Provider — Wiki Labs AI Copilot

## Overview

The AI Provider (`provider.rs`) provides a unified trait-based abstraction over AI inference providers, enabling seamless switching between OpenAI, vLLM, Ollama, and any OpenAI-compatible API.

**Module:** `src/ai/src/provider.rs`  
**Lines of code:** ~352 (including tests)  
**Tests:** 18 unit tests

## Architecture

### Core Trait

```rust
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    // Core functionality
    async fn chat(&self, request: AiRequest) -> anyhow::Result<AiResponse>;
    async fn chat_stream(&self, request: AiRequest) -> anyhow::Result<UnboundedReceiver<String>>;
    async fn embed(&self, request: EmbedRequest) -> anyhow::Result<EmbedResponse>;
    async fn health(&self) -> anyhow::Result<()>;

    // Token management
    fn count_tokens(&self, text: &str) -> usize;
    fn max_context_tokens(&self) -> usize;

    // Feature detection
    fn supports_tools(&self) -> bool;
    fn supports_streaming(&self) -> bool;
    fn supports_structured_output(&self) -> bool;
    fn supports_vision(&self) -> bool;
}
```

### Request/Response Types

```rust
// Individual message
pub struct AiMessage {
    pub role: String,        // "system" | "user" | "assistant" | "tool"
    pub content: String,
}

// Chat request
pub struct AiRequest {
    pub model: String,
    pub messages: Vec<AiMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub tools: Vec<serde_json::Value>,
    pub stream: Option<bool>,
}

// Chat response
pub struct AiResponse {
    pub message: AiMessage,
    pub usage: TokenUsage,
}

// Token usage tracking
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

// Embedding request
pub struct EmbedRequest {
    pub model: String,
    pub input: String,
}

// Embedding response
pub struct EmbedResponse {
    pub model: String,
    pub embedding: Vec<f32>,
    pub usage: TokenUsage,
}
```

## OpenAICompatibleProvider

The default implementation for any OpenAI-compatible API.

### Constructor

```rust
pub fn new(
    name: &str,              // Display name: "OpenAI", "vLLM", "Ollama"
    base_url: &str,          // API base URL
    api_key: &str,           // API key (can be empty for local)
    model: &str,             // Default model name
    max_output_tokens: usize,// Max completion tokens
    context_window: usize,   // Context window size
) -> Self
```

### Examples

```rust
// OpenAI
let provider = OpenAICompatibleProvider::new(
    "OpenAI",
    "https://api.openai.com/v1",
    "sk-your-api-key",
    "gpt-4o",
    4096,
    128000,
);

// vLLM (self-hosted, no API key needed)
let provider = OpenAICompatibleProvider::new(
    "vLLM",
    "http://localhost:8000/v1",
    "",
    "meta-llama/Llama-3-70b",
    4096,
    8192,
);

// Ollama (local, fake API key)
let provider = OpenAICompatibleProvider::new(
    "Ollama",
    "http://localhost:11434/v1",
    "ollama",
    "llama3",
    4096,
    8192,
);
```

### Chat API

```rust
let request = AiRequest {
    model: "gpt-4o".to_string(),
    messages: vec![
        AiMessage { role: "system".into(), content: "You are helpful." },
        AiMessage { role: "user".into(), content: "What's the weather?" },
    ],
    temperature: Some(0.7),
    max_tokens: Some(4096),
    tools: vec![],
    stream: None,
};

let response = provider.chat(request).await?;
println!("Response: {}", response.message.content);
println!("Tokens: prompt={}, completion={}, total={}",
    response.usage.prompt_tokens,
    response.usage.completion_tokens,
    response.usage.total_tokens,
);
```

### Streaming API

```rust
let rx = provider.chat_stream(request).await?;
while let Some(chunk) = rx.recv().await {
    if chunk.starts_with("[error]") {
        eprintln!("Error: {}", &chunk[7..]);
        break;
    }
    print!("{}", chunk);  // Display incrementally
}
```

The streaming implementation:

1. Makes a `POST /chat/completions` request with `stream: true`
2. Spawns a background task that receives SSE chunks from the server
3. Sends each chunk through a `tokio::sync::mpsc::UnboundedSender`
4. The receiver channel (`UnboundedReceiver<String>`) is returned to the caller
5. When the receiver is dropped, the background task detects cancellation and exits

### Embedding API

```rust
let request = EmbedRequest {
    model: "text-embedding-3-small".to_string(),
    input: "Document text to embed".to_string(),
};

let response = provider.embed(request).await?;
println!("Embedding dimensions: {}", response.embedding.len());
println!("First 5 values: {:?}", &response.embedding[..5]);
```

### Health Check

```rust
// Verifies connectivity by hitting GET /models
provider.health().await?;
```

### Feature Detection

```rust
provider.supports_tools();      // true if provider advertises function calling
provider.supports_streaming();  // true if provider supports SSE streaming
provider.supports_structured_output();  // true for JSON mode / response_format
provider.supports_vision();     // true for image-capable models
```

### Token Counting

Uses a simple heuristic: ~4 characters per token for MVP:

```rust
let tokens = provider.count_tokens("Hello, world!");
// Returns ~3 tokens for "Hello, world!"
```

The implementation delegates to `token_counter::count_tokens()` which uses `text.chars().count() / 4` with a minimum of 1.

## Supported Providers

| Provider | Base URL | API Key | Notes |
|---|---|---|---|
| OpenAI | `https://api.openai.com/v1` | `sk-...` | gpt-4o, gpt-4-turbo, text-embedding-3-* |
| vLLM | `http://localhost:8000/v1` | (empty) | Self-hosted, configurable models |
| Ollama | `http://localhost:11434/v1` | `ollama` | Local models via Ollama API server |

### Adding a New Provider

To add support for a new provider, implement the `AiProvider` trait:

```rust
struct MyProvider {
    base_url: String,
    // ... other config
}

#[async_trait::async_trait]
impl AiProvider for MyProvider {
    async fn chat(&self, request: AiRequest) -> anyhow::Result<AiResponse> {
        // Adapt request format to your API
        // Send HTTP request
        // Parse response
    }

    async fn chat_stream(&self, request: AiRequest) -> anyhow::Result<UnboundedReceiver<String>> {
        // Implement SSE streaming
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Implement token counting for your model
    }

    fn max_context_tokens(&self) -> usize {
        // Return your model's context window
    }

    // Implement remaining trait methods...
}
```

## Tests

18 unit tests covering:

| Test | What It Verifies |
|---|---|
| `test_provider_trait_methods` | All trait methods exist on provider |
| `test_openai_compatible_provider_health` | Health check succeeds |
| `test_provider_supports` | Feature flags are correct |
| `test_token_counting` | Token counting works |
| `test_token_counting_empty` | Empty string returns 0 |
| `test_request_serialization` | AiRequest serializes to JSON |
| `test_request_with_tool` | Tool calls included in request |
| `test_request_serialization_with_stream` | Stream flag included |
| `test_response_serialization` | AiResponse serializes to JSON |
| `test_response_deserialization` | AiResponse deserializes from JSON |
| `test_tool_usage_serialization` | TokenUsage round-trips |
| `test_token_usage_totals` | total = prompt + completion |
| `test_token_usage_default` | Default totals are 0 |
| `test_ai_message_serialization` | AiMessage round-trips |
| `test_ai_message_role_variants` | All role strings work |
| `test_embed_request_serialization` | EmbedRequest round-trips |
| `test_embed_response_serialization` | EmbedResponse round-trips |
| `test_provider_info` | Provider info fields are correct |

## Error Handling

All provider methods return `anyhow::Result<T>`:

- **HTTP errors** — Status code + response body forwarded
- **Network errors** — Connection timeout, DNS failure
- **Serialization errors** — JSON parse failures
- **API errors** — Provider-specific error messages

```rust
match provider.chat(request).await {
    Ok(response) => { /* handle success */ }
    Err(e) => eprintln!("API error: {}", e),
}
```

## Usage in the Copilot

The AI Provider integrates with:

- **AppSettingsStore** — Thread-safe provider access via `Arc<Mutex<...>>`
- **ConversationManager** — Sends assembled messages via `chat()` or `chat_stream()`
- **TokenBudgetManager** — Uses `count_tokens()` for budget estimation
- **ContextWindow** — Uses `max_context_tokens()` for budget allocation
- **Tauri Commands** — `send_message`, `test_connection`, `list_providers` commands

## Design Notes

1. **Trait-based abstraction** — `AiProvider` trait enables easy testing (mock providers) and future extension (new providers without modifying existing code).

2. **OpenAI-compatible default** — Most AI APIs (vLLM, Ollama, LocalAI, llama.cpp) implement the OpenAI API format, so a single implementation covers the majority of use cases.

3. **Async-first** — All network operations are async via `reqwest` + `tokio`, keeping the UI responsive during API calls.

4. **Streaming via channels** — Uses `tokio::sync::mpsc::UnboundedSender` for backpressure-free streaming. The caller can drop the receiver to cancel at any time.

5. **Approximate token counting** — Uses chars/4 heuristic for MVP. In production, replace with actual tokenizers (tiktoken for OpenAI, etc.) for accurate counts.