# Wiki Labs AI Copilot — Architecture

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Wiki Labs AI Copilot                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌────────────┐  │
│  │  Intent  │   │  MCP     │   │  AI      │   │Knowledge  │  │
│  │ Manager  │   │ Registry │   │ Runtime  │   │ Manager    │  │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   └─────┬──────┘  │
│       │              │              │               │          │
│       └──────────────┴──────────────┴───────────────┘          │
│                            │                                    │
│                   ┌────────┴────────┐                          │
│                   │  Persistence    │                          │
│                   │  Layer          │                          │
│                   └─────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### Core (`src/core/`)

- **data_types**: Shared types for the entire system (chat, tools, skills, workspace, knowledge, intent)
- **persistence**: Database layer with repositories, schema, and migrations

### AI Runtime (`src/ai/`)

The AI intelligence layer (Phase 5):

1. **Provider Abstraction** — Unified interface for OpenAI, vLLM, and other providers
2. **Conversation Manager** — Structured conversations with CRUD and history
3. **Context Manager** — Central aggregation of conversation, workspace, and knowledge context
4. **Prompt Manager** — Template-based prompt assembly with versioning
5. **Engineering Persona** — Default behavior specification for the AI
6. **Workspace Context** — Per-workspace context with session history
7. **Memory Architecture** — Short-term and long-term memory
8. **AI Streaming** — Progressive response display with cancellation
9. **Token Budget Manager** — Token estimation and intelligent trimming
10. **Manual Context Selection** — User-configurable context influence

### MCP (`src/mcp/`)

- **Registry**: Tool and skill discovery/registration
- **Skill Manager**: Skill lifecycle and activation
- **Transport**: MCP protocol transport layer
- **Protocol**: MCP message types

### Observation (`src/observation/`)

Screen and file observation utilities.

### Intent (`src/intent/`)

User intent recognition and classification.

### Knowledge (`src/knowledge/`)

Knowledge base management with vector storage and retrieval.

## Data Flow

```
User Input → Intent Manager → MCP (Tool Selection) → AI Runtime → Response
                   │                                    │
                   ▼                                    ▼
              Knowledge Base ←———— Persistence Layer —→
```

## Technology Stack

- **Language**: Rust (Edition 2021)
- **Async Runtime**: Tokio
- **HTTP**: reqwest with JSON and streaming support
- **Serialization**: serde, serde_json
- **Database**: SQLx (via persistence layer)
- **Logging**: tracing
- **UUIDs**: uuid
- **Date/Time**: chrono
- **Regex**: regex
- **Utilities**: anyhow, thiserror, once_cell, futures

## Testing

The project uses Rust's built-in testing framework with comprehensive unit tests:
- 148+ tests across the AI module alone
- Tests cover provider abstraction, context management, prompt assembly,
  token budgeting, session management, and conversation lifecycle
- Mock providers for testing without real API calls

## Project Structure

```
wikilabs-ai-copilot/
├── src/
│   ├── main.rs              # Application entry point
│   ├── ai/                  # AI Runtime (Phase 5)
│   │   ├── src/
│   │   │   ├── lib.rs       # Module declarations and re-exports
│   │   │   ├── provider.rs  # AI provider trait and implementations
│   │   │   ├── response.rs  # Response types and streaming helpers
│   │   │   ├── token_counter.rs # Token counting utilities
│   │   │   ├── context.rs   # Context window and allocation
│   │   │   ├── conversation_manager.rs # Conversation CRUD
│   │   │   ├── context_manager.rs      # Context aggregation
│   │   │   ├── prompt_manager.rs       # Template management
│   │   │   ├── persona.rs              # Engineering persona
│   │   │   ├── session_manager.rs      # Session lifecycle
│   │   │   └── token_budget.rs         # Token budget management
│   ├── core/
│   │   ├── data_types/      # Shared types
│   │   └── persistence/     # Database layer
│   ├── mcp/                 # MCP protocol
│   ├── observation/         # Observation utilities
│   ├── intent/              # Intent recognition
│   └── knowledge/           # Knowledge base
└── docs/
    ├── AI_RUNTIME.md        # AI Runtime documentation
    ├── CONTEXT_MANAGER.md   # Context Manager documentation
    └── PROMPT_MANAGER.md    # Prompt Manager documentation
```