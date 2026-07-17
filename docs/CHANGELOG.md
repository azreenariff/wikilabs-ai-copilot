# Changelog

## [0.2.0] — Phase 5 — AI Runtime (2025-01-XX)

### Added

**AI Runtime** (`src/ai/`)

- **Provider Abstraction** — `AiProvider` trait with `OpenAICompatibleProvider` implementation
  - Support for OpenAI, vLLM, Ollama, and any OpenAI-compatible API
  - Chat completions and streaming via `chat_stream()`
  - Embedding generation via `embed()`
  - Health checks via `health()`
  - Feature detection (`supports_tools()`, `supports_streaming()`, etc.)

- **Streaming Responses** — `StreamBuilder` and `StreamItem` types for progressive output
  - Async channel-based streaming via tokio `mpsc::UnboundedReceiver`
  - Stream builder for constructing multi-chunk responses
  - Cancellation support via channel drop

- **Response Types** — `AiResponse`, `TokenUsage`, `ToolCall`, `EmbedResponse`
  - Structured response parsing with serde
  - Tool call extraction and argument parsing

- **Token Counter** — Approximate token counting via character heuristic
  - `count_tokens()` and `count_tokens_messages()`
  - ~4 chars per token approximation for English text

- **Context Window** — Token budget tracking and allocation
  - `ContextWindow` with total/max token tracking
  - `ContextAllocation` with configurable percentages
  - Token consumption, remaining tokens, and budget allocation
  - Near-full detection (>85% usage)
  - Truncation for budget enforcement
  - Context entry recording and building

- **Conversation Manager** — Multi-conversation management
  - CRUD operations for conversations
  - Message addition with role tracking (system/user/assistant)
  - Tool call support in messages
  - JSON export/import and text export
  - Conversation listing with summaries
  - Active conversation switching
  - Conversation tagging
  - Role-based message counting

- **Context Manager** — Central context aggregation
  - Multiple context sources (conversation, workspace, knowledge, manual)
  - Priority-based context sources (High/Normal/Low)
  - Technology stack tracking
  - Current activity tracking
  - Tag-based source filtering
  - Source removal and management
  - JSON serialization of aggregated context
  - Builder pattern for context construction
  - Full prompt assembly from all sources

- **Prompt Manager** — Template management and prompt assembly
  - Multiple template categories (system, workspace, skill, context, user)
  - Template versioning (Default, Numbered, Named)
  - Template activation/deactivation
  - Template context with `{{placeholder}}` replacement
  - Prompt assembly with all components
  - Template bumping and version tracking
  - Placeholder counting
  - Template description and metadata

- **Engineering Persona** — Default AI behavior specification
  - Role: Senior Infrastructure Engineer
  - Confidence assessment system (High/Medium/Low)
  - Suggested verification for uncertain responses
  - Custom persona support
  - Persona activation/deactivation
  - System prompt text generation

- **Session Manager** — AI session lifecycle management
  - Session creation, switching, and deletion
  - State management (Active, Paused, Suspended, Idle, Ended)
  - Token and message recording per session
  - Session cleanup of ended sessions
  - JSON export/import for sessions
  - Session filtering by state
  - Tags and technologies per session
  - Session notes and descriptions

- **Token Budget Manager** — Token estimation and budget enforcement
  - Budget policies: Strict, WithBuffer, Aggressive
  - Token estimation for messages
  - Intelligent conversation trimming
  - System prompt token accounting
  - Budget checks with breakdown
  - Recommended actions (NoOp, Trim, Summarize, Drop, Reject)
  - Per-source token accounting with priorities

### Tests

- 148 unit tests across all AI modules
- Comprehensive coverage of:
  - Context window allocation and token tracking
  - Conversation CRUD and export/import
  - Context source management and aggregation
  - Prompt template management and assembly
  - Persona confidence assessment
  - Session state management
  - Token budget policies and trimming
  - Provider API simulation
  - Streaming response construction

### Documentation

- `docs/AI_RUNTIME.md` — AI Runtime architecture and API reference
- `docs/CONTEXT_MANAGER.md` — Context Manager architecture and usage
- `docs/PROMPT_MANAGER.md` — Prompt Manager architecture and usage
- `docs/ARCHITECTURE.md` — Overall system architecture
- `docs/USER_GUIDE.md` — User-facing documentation
- `docs/ROADMAP.md` — Development roadmap
- `docs/CHANGELOG.md` — This file

### Changes

- Updated `src/ai/Cargo.toml` with Phase 5 dependencies:
  - `async-trait` for async provider trait
  - `reqwest` with streaming support
  - `tokio` with sync primitives
  - `anyhow` for error handling
  - `chrono` for timestamps
  - `uuid` for IDs
  - `futures` for async streams
  - `thiserror` for error types
  - `serde` and `serde_json` for serialization

- Updated `src/ai/src/lib.rs` with 10 module declarations and public re-exports

## [0.1.0] — Foundation

### Added

- Project structure and Cargo workspace
- Core data types
- Persistence layer
- Application entry point
- MCP protocol implementation
- Intent recognition
- Knowledge base