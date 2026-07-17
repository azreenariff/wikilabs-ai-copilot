# AI Runtime — Wiki Labs AI Copilot

## Overview

The AI Runtime is the intelligence layer of Wiki Labs AI Copilot. It provides the abstraction, management, and reasoning infrastructure that powers every conversation, prompt assembly, and context aggregation in the system.

**Crate:** `wikilabs-ai`  
**Module path:** `src/ai/src/`  
**Lines of code:** ~4,299 across 10 modules  
**Tests:** 148 unit tests — all passing

---

## Architecture

The AI Runtime is organized into 10 modules, each with a focused responsibility:

```
src/ai/src/
├── lib.rs                  # Public API surface + re-exports
├── provider.rs             # AI Provider trait + OpenAICompatibleProvider impl
├── response.rs             # Streaming response helpers
├── token_counter.rs        # Token counting utilities
├── context.rs              # ContextWindow + ContextAllocation (token budget)
├── conversation_manager.rs # Conversation CRUD + lifecycle
├── context_manager.rs      # Multi-source context aggregation
├── prompt_manager.rs       # Template-driven prompt assembly
├── persona.rs              # Engineering persona + confidence
├── session_manager.rs      # Session lifecycle + state transitions
└── token_budget.rs         # Token budget enforcement + trimming
```

---

## Feature 1: AI Provider Abstraction

### Trait: `AiProvider`

The `AiProvider` trait defines the contract for any AI backend:

```rust
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(&self, request: AiRequest) -> anyhow::Result<AiResponse>;
    async fn chat_stream(&self, request: AiRequest) -> anyhow::Result<UnboundedReceiver<String>>;
    async fn embed(&self, request: EmbedRequest) -> anyhow::Result<EmbedResponse>;
    fn count_tokens(&self, text: &str) -> usize;
    fn max_context_tokens(&self) -> usize;
    fn supports_tools(&self) -> bool;
    fn supports_streaming(&self) -> bool;
    fn supports_structured_output(&self) -> bool;
    fn supports_vision(&self) -> bool;
    async fn health(&self) -> anyhow::Result<()>;
}
```

### Concrete Implementation: `OpenAICompatibleProvider`

Implements the `AiProvider` trait for any OpenAI-compatible API (OpenAI, vLLM, Ollama, etc.):

- **Chat:** Sends `POST /chat/completions` with Bearer auth
- **Streaming:** Spawns a background task that streams chunks via a channel
- **Embeddings:** Sends `POST /embeddings`
- **Health:** Pings `GET /models` to verify connectivity
- **Token counting:** Approximate heuristic (~4 chars per token)

### Supported Backends

| Provider | Endpoint | Notes |
|---|---|---|
| OpenAI | `https://api.openai.com/v1` | gpt-4o, gpt-4-turbo |
| vLLM | `http://localhost:8000/v1` | Self-hosted, supports vision |
| Ollama | `http://localhost:11434/v1` | Local models |

---

## Feature 2: Conversation Manager

Manages multiple concurrent conversations with full CRUD lifecycle.

### Key Types

- **`ConversationMessage`** — Individual messages with role (user/assistant/system), content, timestamp, optional tool calls
- **`Conversation`** — A named conversation with messages, tags, and system prompt override
- **`ConversationManager`** — Manages multiple conversations, tracks the active one

### API Highlights

```rust
let mut mgr = ConversationManager::new();

// Create
let id = mgr.create("Database Debug");

// Add messages
mgr.add_message(ConversationMessage::user("Why is this query slow?")).unwrap();

// Switch conversations
mgr.switch(other_id).unwrap();

// Export/Restore
let json = mgr.export(id).unwrap();
let restored_id = mgr.restore(&json).unwrap();
```

### Built-in Features

- Message role counting
- Conversation renaming with timestamp tracking
- Tag-based categorization
- JSON export/import with full state preservation
- Preview generation for conversation listings
- Auto-select first conversation as active on creation

---

## Feature 3: Context Manager

Central aggregation engine that combines context from multiple sources into a coherent prompt.

### Context Sources

| Source | Priority | Truncated |
|---|---|---|
| System prompt | High | Never |
| Workspace context | Normal | After High |
| User preferences | Normal | After High |
| Technology stack | Low | First |
| Manual user context | Configurable | Configurable |
| Screen observation | Low | First (future) |
| MCP results | Low | First (future) |

### Key Types

- **`ContextSource`** — Individual context block with name, content, priority, manual flag, tags
- **`AggregatedContext`** — All sources combined into a structured output
- **`ContextBuilder`** — Fluent builder for incremental context construction
- **`ContextManager`** — Central manager with add/remove/filter operations

### API Highlights

```rust
let mut ctx = ContextManager::new(system_prompt, model);
ctx.add_technology("rust");
ctx.add_technology("kubernetes");
ctx.set_current_activity("Debugging OOM in production");
ctx.add_source(ContextSource::manual("Custom Context", "...", ContextPriority::Normal));

let aggregated = ctx.build_context(messages, estimated_tokens);
let full_prompt = aggregated.full_prompt();
```

---

## Feature 4: Prompt Manager

Template-driven prompt assembly with versioning and placeholder replacement.

### Template System

Prompts are built from templates using `{{placeholder}}` syntax:

```rust
let template = PromptTemplate::new("system", "You are a {{role}}.\nTask: {{task}}");
template.bump_version(); // default → v1

let context = TemplateContext::new()
    .with("role", "Senior Engineer")
    .with("task", "debug the application");

let (assembled, count) = context.apply_to(&template.content);
// assembled: "You are a Senior Engineer.\nTask: debug the application"
// count: 2
```

### Template Categories

- **System templates** — Core behavior definition
- **Workspace templates** — Per-workspace context injection
- **Context templates** — Dynamic context blocks
- **User templates** — User message formatting
- **Skill templates** — Skill-specific instructions (future)

### Versioning

```rust
enum PromptVersion {
    Default,     // Unversioned
    Numbered(usize),  // v1, v2, v3...
    Named(String),     // "stable", "experimental"
}
```

---

## Feature 5: Engineering Persona

Defines the AI's behavioral identity: Senior Infrastructure Engineer, Technical Advisor, Enterprise Consultant, and Troubleshooting Mentor.

### Persona Definition

```rust
pub struct EngineeringPersona {
    role: String,
    system_prompt: String,
    active: bool,
    confidence_thresholds: ConfidenceThresholds,
}
```

### Behavioral Rules

1. Explain reasoning step-by-step
2. Prefer evidence-based recommendations over assumptions
3. Suggest verification steps for the engineer to confirm
4. Always state confidence level (HIGH / MEDIUM / LOW)
5. Never claim to observe something not explicitly provided
6. Ask clarifying questions rather than guessing

### Confidence Thresholds

| Level | Threshold | Behavior |
|---|---|---|
| HIGH | ≥ 0.9 | Standard practice, proceed |
| MEDIUM | ≥ 0.6 | Reasonable inference, note uncertainty |
| LOW | < 0.3 | Speculative, require verification |

### Confidence Assessment

The persona can produce structured confidence assessments:

```rust
let assessment = ConfidenceAssessment::medium(
    "Based on logs, likely a config issue",
    "Check application logs for specific error codes"
);
```

---

## Feature 6: Workspace Context

Per-workspace context is integrated into the Context Manager. Each workspace contributes a context block with:

- Technology stack configuration
- Active engineering task
- Workspace file references
- Project-specific constraints

The Context Manager includes a dedicated `workspace_context` field that is assembled into every prompt.

### Token Allocation for Workspace

Default allocation of the context window:

| Category | Percentage |
|---|---|
| System prompt | 10% |
| Conversation history | 40% |
| Knowledge context | 20% |
| Workspace context | 20% |
| Padding | 10% |

---

## Feature 7: Memory Architecture

### Session Manager

Manages AI session lifecycle with state transitions:

```rust
enum SessionState { Active → Paused → Suspended → Ended }
```

### Session Tracking

Each `Session` tracks:
- Configuration (model, temperature, max tokens, workspace, technologies)
- State transitions with timestamps
- Message count and token consumption
- Idle detection (configurable timeout)
- Session duration
- Optional notes

### API Highlights

```rust
let mut sm = SessionManager::new();
let id = sm.create(SessionConfig::new("Debug Session", system_prompt, "gpt-4o"));
sm.active_mut().unwrap().record_message(100, 50); // user + assistant tokens
sm.pause_active().unwrap();
sm.resume_active().unwrap();
```

---

## Feature 8: AI Streaming

Supports progressive response display through `AiProvider::chat_stream()`:

```rust
let rx = provider.chat_stream(request).await?;
while let Some(chunk) = rx.recv().await {
    display.append(chunk);
}
```

### Cancellation

Since streaming uses `tokio::sync::mpsc::UnboundedReceiver`, the receiving task can be cancelled at any point to stop receiving further chunks. The spawned background task detects cancellation and exits cleanly.

---

## Feature 9: Token Budget Manager

Prevents context window overflow with intelligent trimming.

### Budget Policies

```rust
enum BudgetPolicy {
    Strict,           // Never exceed budget
    WithBuffer,       // Allow small overflow (configurable %)
    Aggressive,       // Always trim to fit
}
```

### Trimming Strategy

When budget is exceeded, the manager recommends actions in this order:

1. **Trim conversation** — Remove older messages first
2. **Summarize** — Replace old messages with summaries
3. **Drop low-priority context** — Remove low-priority ContextSources
4. **Reject** — Refuse to send the request (Strict/Aggressive mode)

### Budget Builder

Fluent API for constructing budget checks:

```rust
let check = BudgetBuilder::new()
    .with_system(512)
    .with_recent_conversation(2048)
    .with_older_conversation(4096)
    .with_workspace_context(1024)
    .with_other(256)
    .check(8192, &BudgetPolicy::Strict);
```

### Output

The budget check returns:
- Total estimated tokens
- Whether within budget
- Recommended action
- Token breakdown by source with priority labels

---

## Feature 10: Manual Context Selection

Users can manually add context sources that influence the AI's responses:

```rust
let manual = ContextSource::manual(
    "Production Logs",
    "Error: connection refused at 10:30",
    ContextPriority::Normal
).with_tags(vec!["logs", "production"]);
```

Sources are:
- Filterable by tag
- Sortable by priority
- Excluded from truncation based on priority level (High is never truncated)

---

## Integration with Tauri Desktop

The Tauri app uses the AI Runtime through these Tauri commands:

| Command | Description |
|---|---|
| `send_message` | Send a chat message, get AI response |
| `get_history` | Retrieve conversation history |
| `stream_message` | Stream response in real-time |
| `list_providers` | List available AI providers |
| `test_connection` | Verify provider connectivity |
| `get_workspace_list` | List workspaces |
| `create_workspace` | Create a new workspace |

The `AppState` wraps the AI provider via `AppSettingsStore` for thread-safe access.

---

## Dependencies

- `reqwest` — HTTP client for API calls
- `tokio` — Async runtime for streaming
- `serde` / `serde_json` — Serialization
- `anyhow` — Error handling
- `async_trait` — Async trait support
- `chrono` — Timestamps
- `uuid` — Unique identifiers
- `regex` — Placeholder matching

---

## Test Coverage

148 unit tests across all modules:

- **provider** — Health checks, token counting, feature flags
- **context** — Window allocation, consumption, truncation, budget
- **conversation_manager** — CRUD, switching, export/import, tags
- **context_manager** — Source management, aggregation, filtering
- **prompt_manager** — Template versioning, placeholder replacement, assembly
- **persona** — Default/custom persona, activation, confidence thresholds
- **session_manager** — State transitions, message recording, idle detection
- **token_budget** — Budget checks, trimming logic, policy enforcement

All tests compile and run: `cargo test --package wikilabs-ai`