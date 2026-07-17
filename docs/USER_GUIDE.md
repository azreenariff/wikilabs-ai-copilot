# Wiki Labs AI Copilot — User Guide

## Getting Started

### Prerequisites

- Rust 1.75+ (Edition 2021)
- Cargo
- A configured AI provider (OpenAI API key, vLLM endpoint, etc.)

### Building

```bash
cd wikilabs-ai-copilot
cargo build --release
```

### Testing

```bash
cargo test --all
```

## Configuration

### AI Provider

Configure the AI provider in your settings:

```yaml
ai:
  provider: openai          # openai, ollama, vllm, or custom
  base_url: https://api.openai.com/v1
  api_key: sk-...           # Store securely
  model: gpt-4              # Model name
  max_tokens: 4096          # Context window size
  temperature: 0.7          # Generation temperature
```

### Workspace

Point the copilot to a workspace directory:

```bash
# Initialize in current directory
./wikilabs-copilot init

# Or specify a path
./wikilabs-copilot init --workspace /path/to/project
```

## Usage

### Starting a Session

```bash
./wikilabs-copilot start
```

This creates a new AI session with:
- Your workspace context
- Your knowledge base
- Engineering persona (default AI behavior)
- Current conversation history

### Working with Conversations

The copilot manages multiple conversations:

- **New Conversation**: `new` command or `Ctrl+N`
- **Switch Conversation**: `switch <name>` or `Ctrl+S`
- **List Conversations**: `list` command
- **Delete Conversation**: `delete <name>`
- **Rename Conversation**: `rename <old> <new>`
- **Export Conversation**: `export <name>`

### Context Management

The copilot automatically gathers context from:

1. **Conversation History** — Recent messages
2. **Workspace** — Project files and structure
3. **Knowledge Base** — Pinned knowledge and best practices
4. **Technology Stack** — Detected technologies in workspace

You can also add manual context:

```bash
# Add manual context
context add "Running on OpenShift 4.12"

# View current context
context status

# Remove manual context
context remove <id>
```

### Skills

The copilot can use skills to enhance responses:

```bash
# List available skills
skills list

# Activate a skill
skills activate debug

# Deactivate a skill
skills deactivate debug
```

## Configuration Reference

### Context Window Allocation

```yaml
context:
  system_prompt_pct: 0.10      # 10% for system prompt
  conversation_history_pct: 0.40 # 40% for conversation
  knowledge_context_pct: 0.20    # 20% for knowledge
  workspace_context_pct: 0.20    # 20% for workspace
  padding_pct: 0.10              # 10% reserved
```

### Token Budget Policies

```yaml
token_budget:
  policy: strict           # strict, with_buffer, aggressive
  buffer_pct: 0.10         # Buffer percentage (for with_buffer)
```

### Persona Configuration

The default persona is **Senior Infrastructure Engineer**. You can customize:

```yaml
persona:
  role: "Senior Infrastructure Engineer"
  confidence_thresholds:
    high: 0.8
    medium: 0.5
    low: 0.3
```

## API

The copilot exposes an MCP-compatible interface for external tools:

```rust
// Using the AI provider
let provider = OpenAICompatibleProvider::new(
    "openai",
    "https://api.openai.com/v1",
    api_key,
    "gpt-4",
    8192,  // max_tokens
    32768, // context_window
);

// Send a chat request
let response = provider.chat(AiRequest {
    model: "gpt-4".to_string(),
    messages: vec![
        AiMessage { role: "system".into(), content: "Be helpful." },
        AiMessage { role: "user".into(), content: "What is Rust?" },
    ],
    temperature: Some(0.7),
    ..Default::default()
}).await?;

// Stream a response
let stream = provider.chat_stream(request).await?;
while let Some(chunk) = rx.recv().await {
    print!("{}", chunk);
}
```

## Troubleshooting

### Provider Errors

If you see API errors:
1. Verify your API key is correct
2. Check the base URL matches your provider
3. Ensure the model name is available on your provider

### Context Window Full

If context is near full:
1. Switch to a shorter conversation
2. Reduce knowledge context size
3. Use `context status` to check current allocation

### Session Issues

To reset sessions:
1. End the current session: `session end`
2. Create a new session: `session new`
3. Use `session list` to view all sessions