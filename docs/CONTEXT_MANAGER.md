# Context Manager

## Overview

The Context Manager (`ContextManager`) is the central aggregation engine for all context sources that inform AI responses. It collects, prioritizes, and assembles information from conversations, workspace data, user preferences, and external sources.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Context Manager                    │
│                                                      │
│  ┌────────────┐ ┌──────────┐ ┌──────────┐           │
│  │ Conversation│ │ Workspace│ │  Knowledge│           │
│  │   History   │ │  Context │ │ Context  │           │
│  └──────┬─────┘ └────┬─────┘ └────┬─────┘           │
│         │             │             │                 │
│         ▼             ▼             ▼                 │
│  ┌────────────────────────────────────────┐          │
│  │        AggregatedContext               │          │
│  │  • system_prompt                       │          │
│  │  • technology_stack                    │          │
│  │  • current_activity                    │          │
│  │  • workspace_context                   │          │
│  │  • conversation_messages               │          │
│  │  • selected_model                      │          │
│  └────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────┘
```

## Features

### Context Sources

Context comes from multiple sources, each with a priority level:

- **Conversation Messages**: Recent messages from the current conversation
- **Workspace Context**: Project-specific data, file contents, structure
- **Knowledge Context**: Pinned knowledge, best practices, templates
- **Manual Context**: User-provided additional context
- **Session Context**: Current session configuration and preferences

### Context Priorities

```rust
pub enum ContextPriority {
    High,     // Critical context, always included
    Normal,   // Standard context, included if budget allows
    Low,      // Optional context, included only with spare budget
}
```

### Context Window Budgeting

The `ContextWindow` tracks token usage and allocates budgets:

```rust
pub struct ContextWindow {
    total_tokens: usize,
    max_tokens: usize,
    allocation: ContextAllocation,
}

pub struct ContextAllocation {
    system_prompt_pct: f32,      // 10%
    conversation_history_pct: f32, // 40%
    knowledge_context_pct: f32,   // 20%
    workspace_context_pct: f32,   // 20%
    padding_pct: f32,             // 10%
}
```

### AggregatedContext

The output of context aggregation — a complete snapshot of all information to send to the AI:

```rust
pub struct AggregatedContext {
    pub system_prompt: String,
    pub conversation_messages: Vec<serde_json::Value>,
    pub workspace_context: String,
    pub knowledge_context: String,
    pub technology_stack: Vec<String>,
    pub current_activity: Option<String>,
    pub selected_model: String,
    pub estimated_tokens: usize,
}
```

## API Reference

### Creating a Context Manager

```rust
let mut cm = ContextManager::new("You are an engineer.", "gpt-4");
```

### Adding Context Sources

```rust
cm.add_source(ContextSource::new(
    "debug_log",
    "Error: connection refused",
    ContextPriority::High,
));

cm.add_technology("kubernetes");
cm.add_technology("rust");
```

### Setting Additional Context

```rust
cm.set_current_activity("Troubleshooting API latency");
cm.set_workspace_context("/react/pin is the active workspace");
```

### Building Context

```rust
let context = cm.build_context(conversation_messages, estimated_tokens);
let prompt = context.full_prompt();
```

## Token Management

The context manager works with the Token Budget Manager to ensure all context fits within the model's context window. When budget is exceeded:

1. Older conversation messages are trimmed first
2. Low-priority context sources are removed
3. If still over budget, the operation is rejected

## Testing

The `ContextManager` has extensive test coverage:

- Source management (add, remove, filter by priority/tag)
- Technology stack management
- Context aggregation and JSON serialization
- Builder pattern for context construction
- Full prompt assembly