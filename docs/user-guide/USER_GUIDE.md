# User Guide — Wiki Labs AI Copilot

## Installation

### Prerequisites

- **Rust 1.70+** — `rustup install stable`
- **Cargo** — Included with Rust toolchain

### Building from Source

```bash
# Clone the repository
git clone https://github.com/wikilabs/wikilabs-ai-copilot.git
cd wikilabs-ai-copilot

# Build the project
cargo build --release

# Run the CLI
cargo run --release
```

### Testing

```bash
# Run all workspace tests
cargo test --workspace

# Run tests for a specific package
cargo test --package wikilabs-ai

# Run tests with output
cargo test --package wikilabs-ai -- --nocapture
```

## Quick Start

### 1. Configure an AI Provider

The AI Copilot uses OpenAI-compatible APIs. Configure your provider by setting:

- **Base URL** — API endpoint (e.g., `https://api.openai.com/v1` or `http://localhost:8000/v1`)
- **API Key** — Your API key (can be empty for local providers like vLLM or Ollama)
- **Model** — Default model to use (e.g., `gpt-4o`, `llama3`)

### 2. Create a Workspace

Workspaces organize your AI sessions around specific customers or projects:

1. Click "New Workspace" in the workspace sidebar
2. Enter a workspace name (e.g., "ABC Bank")
3. Select technologies (e.g., Kubernetes, MySQL, Linux)
4. Click "Create"

Each workspace maintains its own:
- Technology stack configuration
- Engineering task focus
- AI session state

### 3. Start a Conversation

1. Select or create a workspace
2. Click "New Conversation"
3. Type your question in the chat input
4. Press Enter or click "Send"

The AI will respond with analysis, recommendations, and step-by-step debugging guidance.

### 4. Manage Conversations

- **Rename** — Right-click a conversation in the sidebar and select "Rename"
- **Tags** — Add tags to categorize conversations (e.g., "bugfix", "production", "urgent")
- **Export** — Export a conversation as JSON for backup or sharing
- **Delete** — Right-click and select "Delete" to remove a conversation

## AI Runtime

### Provider Integration

The AI Copilot supports any OpenAI-compatible provider:

| Provider | URL | API Key |
|---|---|---|
| OpenAI | `https://api.openai.com/v1` | `sk-...` |
| vLLM | `http://localhost:8000/v1` | (empty) |
| Ollama | `http://localhost:11434/v1` | `ollama` |

### Streaming Responses

The AI Copilot displays responses progressively as they are generated. This provides:

- Immediate feedback while the AI is thinking
- Ability to read while the response continues generating
- Better perceived performance for long responses

### Conversation History

Each conversation maintains a full message history:

- **System messages** — AI instructions and behavioral constraints
- **User messages** — Your questions and inputs
- **Assistant messages** — AI responses with reasoning and recommendations

Messages are stored in memory and can be exported as JSON.

## Workspace Management

### Technology Stack

Each workspace has a technology stack that influences the AI's responses:

1. Click the workspace settings icon
2. Add or remove technologies (e.g., Rust, Kubernetes, Docker, MySQL)
3. The AI uses this context to provide more relevant suggestions

### Current Task

Set your current engineering task to give the AI more context:

1. Click the task field in the workspace sidebar
2. Type your current task (e.g., "Debugging OOM in production")
3. The AI will tailor its responses to this context

### Workspace Sessions

Each workspace maintains its own AI sessions:

- **Active** — Currently in progress
- **Paused** — Suspended (e.g., you switched to another task)
- **Suspended** — Paused due to workspace switch
- **Ended** — Conversation closed

Sessions track token consumption and message counts for cost tracking.

## Manual Context

You can add manual context sources to influence AI responses:

1. Click "Add Context" in the conversation
2. Enter a name (e.g., "Production Logs", "Kubernetes Events")
3. Enter the context content (e.g., error logs, config snippets)
4. Set the priority (High, Normal, Low)
5. Add tags for filtering

### Context Priority

| Priority | Behavior |
|---|---|
| High | Never truncated, always included in prompt |
| Normal | Included if budget allows |
| Low | First to be removed when budget is tight |

### Context Tags

Tags enable filtering context by category:

- `production` — Production environment context
- `kubernetes` — Kubernetes-related context
- `logs` — Log output context
- `config` — Configuration context

## Token Budget

The AI Copilot manages token usage to prevent context window overflow:

### Budget Policies

| Policy | Behavior |
|---|---|
| Strict | Never exceed budget, reject if over |
| With Buffer | Allow small overflow (configurable percentage) |
| Aggressive | Always trim to fit within budget |

### Trimming Strategy

When the context window is full, the AI Copilot uses this order:

1. **Trim conversation** — Remove older messages first
2. **Summarize** — Replace old messages with summaries
3. **Drop low-priority context** — Remove low-priority context sources
4. **Reject** — Refuse to send the request (Strict mode only)

### Token Tracking

Each session tracks:
- **Prompt tokens** — Tokens in the input prompt
- **Completion tokens** — Tokens in the AI response
- **Total tokens** — Sum of prompt and completion tokens
- **Message count** — Number of messages exchanged

## Engineering Persona

The AI Copilot uses an engineering persona that:

- Explains reasoning step-by-step
- Provides evidence-based recommendations
- Suggests verification steps
- States confidence levels (HIGH / MEDIUM / LOW)
- Never claims to observe things not explicitly provided
- Asks clarifying questions rather than guessing

### Confidence Levels

| Level | Meaning |
|---|---|
| HIGH | Strong evidence, standard practice |
| MEDIUM | Reasonable inference, note uncertainty |
| LOW | Speculative, requires verification |

## Security

### Data Residency

- All data is stored locally on your machine
- API keys are stored in the OS keychain (if available)
- No data is sent to third parties except the configured AI provider

### API Key Protection

- API keys are encrypted using OS keychain or derived key encryption
- Keys are never logged or displayed in plain text
- Keys are only sent to the configured provider

### Audit Logging

All API calls are logged for audit purposes:
- Timestamp
- Provider used
- Model used
- Token usage
- Error status (no content logged)

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Enter` | Send message |
| `Shift + Enter` | New line in message |
| `Ctrl + N` | New conversation |
| `Ctrl + W` | New workspace |
| `Ctrl + S` | Save current conversation |
| `Ctrl + E` | Export conversation |
| `Ctrl + K` | Clear conversation |

## Troubleshooting

### Provider Connection Failed

1. Check your API key is correct
2. Verify the base URL is accessible
3. Check your network connection
4. Try the "Test Connection" button in provider settings

### Slow Responses

1. Check if you're using a local provider (slower than cloud)
2. Try a smaller model (faster but less capable)
3. Reduce context window size
4. Check system resources (CPU, memory)

### Context Window Overflow

1. Reduce the number of conversations
2. Archive old conversations
3. Remove low-priority manual context
4. Switch to a provider with a larger context window

## Getting Help

- Check the documentation in `docs/`
- Open an issue on GitHub
- Join the community Discord