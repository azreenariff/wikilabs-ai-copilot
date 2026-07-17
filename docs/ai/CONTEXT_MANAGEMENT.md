# Context Management — Wiki Labs AI Copilot

## Overview

Context Management in the AI Copilot combines the **Context Manager** (`context_manager.rs`) and **Context Window** (`context.rs`) modules to provide a complete solution for assembling, allocating, and enforcing token budgets for AI prompts.

## Context Manager

The Context Manager (`context_manager.rs`) is the central aggregation engine that combines information from multiple sources into a coherent prompt for the AI.

### Context Sources

Sources have priorities that determine truncation order:

| Priority | Behavior |
|---|---|
| `High` | Never truncated |
| `Normal` | Truncated after High sources |
| `Low` | Truncated first |

### Source Types

```rust
// Regular source
ContextSource {
    id: Uuid,
    name: String,              // "Production Logs", "Kubernetes Events"
    content: String,           // Actual text content
    priority: ContextPriority, // High | Normal | Low
    is_manual: bool,           // Manually added by user?
    tags: Vec<String>,         // For filtering
}

// Manual source (user-added)
ContextSource::manual(
    "Production Logs",
    "2024-01-15 ERROR: OOMKilled",
    ContextPriority::High
)
```

### Manager API

```rust
let mut ctx = ContextManager::new(
    "You are a senior infrastructure engineer.",
    "gpt-4o"
);

// Technology stack
ctx.add_technology("kubernetes");
ctx.add_technology("prometheus");

// Current activity
ctx.set_current_activity("Debugging production OOM");

// Manual context
ctx.add_source(ContextSource::manual(
    "Logs",
    "OOMKilled at 10:30",
    ContextPriority::High
).with_tags(vec!["logs".to_string()]));

// Workspace context
ctx.set_workspace_context("ABC Bank / Production");
```

### Filtering Sources

```rust
// By priority
let high = ctx.sources_by_priority(ContextPriority::High);

// By tag
let logs = ctx.sources_by_tag("logs");

// By name
let matched = ctx.sources_by_name("production");

// All sources sorted by priority
let all = ctx.sources();
```

### Building Aggregated Context

```rust
let msgs = vec![
    json!({ "role": "user", "content": "What's causing the OOM?" }),
];
let context = ctx.build_context(msgs, 500);

// Get full prompt
let full = context.full_prompt();
// "You are a senior infrastructure engineer.\n\n## Technologies\nkubernetes, prometheus\n\n## Current Activity\nDebugging production OOM\n\n## Workspace Context\nABC Bank / Production\n\n## Sources\n\n### Production Logs\n2024-01-15 ERROR: OOMKilled\n\n## Conversation\n\n[user] What's causing the OOM?"
```

### AggregatedContext Structure

```rust
AggregatedContext {
    system_prompt: String,
    workspace_context: String,
    technology_stack: Vec<String>,
    current_activity: Option<String>,
    conversation_messages: Vec<serde_json::Value>,
    sources: Vec<ContextSource>,
    user_preferences: String,
    selected_model: String,
    estimated_tokens: usize,
}
```

### Context Builder Pattern

```rust
let context = ContextBuilder::new()
    .with_system_prompt("You are an engineer.")
    .with_workspace_context("ABC Bank / Prod")
    .with_user_preferences("Prefers bullet points")
    .with_technologies(vec!["rust".to_string()])
    .with_current_activity("Debugging")
    .with_selected_model("gpt-4o")
    .build(messages, estimated_tokens);
```

### JSON Support

```rust
let json = context.to_json()?;  // Serialize
let parsed: AggregatedContext = serde_json::from_str(&json)?;  // Deserialize
```

### Tests

20 unit tests covering:
- Context creation and defaults
- Technology add/remove/dedup
- Activity tracking with clear
- Source add/remove by ID
- Priority-based sorting
- Tag-based filtering
- Source name matching
- Aggregation formatting
- Full prompt assembly
- Builder pattern
- Manual source creation
- JSON serialization round-trip
- to_builder conversion

## Context Window

The Context Window (`context.rs`) manages token budget allocation and usage tracking within a provider's context window.

### Core Concept

```
Context Window: 32,768 tokens (configurable per model)
├── System Prompt:        10% → 3,277 tokens
├── Conversation History: 40% → 13,107 tokens
├── Knowledge Context:    20% → 6,554 tokens
├── Workspace Context:    20% → 6,554 tokens
└── Padding:              10% → 3,277 tokens
```

### ContextWindow API

```rust
let window = ContextWindow::new(32768);  // 32K context

// Usage tracking
let usage = window.usage();           // tokens used
let remaining = window.remaining();    // tokens left
let pct = window.usage_pct();         // 0.0 - 1.0

// Budget allocation
let conversation_budget = window.allocate_for(0.40);
// Returns: 13,107 tokens

// Check budget
let check = window.is_within_budget(5000);  // true
```

### Default Allocation

```rust
pub fn default_allocation() -> ContextAllocation {
    ContextAllocation {
        system_prompt_pct: 0.10,
        conversation_history_pct: 0.40,
        knowledge_context_pct: 0.20,
        workspace_context_pct: 0.20,
        padding_pct: 0.10,
    }
}
```

### Custom Allocation

```rust
let alloc = ContextAllocation {
    system_prompt_pct: 0.15,          // Larger system prompt
    conversation_history_pct: 0.50,   // More conversation history
    knowledge_context_pct: 0.10,      // Less knowledge context
    workspace_context_pct: 0.15,      // More workspace context
    padding_pct: 0.10,                // Maintain padding
};
let window = ContextWindow::new_with_allocation(32768, alloc);
```

### Context Entry Tracking

```rust
let mut window = ContextWindow::new(32768);

// Record context entries
window.record(&ContextEntry {
    label: "system_prompt".to_string(),
    tokens: 512,
});
window.record(&ContextEntry {
    label: "conversation".to_string(),
    tokens: 8192,
});
window.record(&ContextEntry {
    label: "knowledge_context".to_string(),
    tokens: 4096,
});

// Report
let entries = window.build_entries();
// [
//   { label: "system_prompt", tokens: 512 },
//   { label: "conversation", tokens: 8192 },
//   { label: "knowledge_context", tokens: 4096 },
//   { label: "padding", tokens: 3277 },
//   { label: "total", tokens: 17077 },
//   { label: "remaining", tokens: 15691 },
//   { label: "usage_pct", tokens: 52.08 },
// ]
```

### Token Budget Fitting

```rust
// Check if conversation fits within allocated budget
let fits = window.fit_within_budget(10000);

// Truncate to fit target
let truncated = window.truncate_to_fit(
    "long conversation text...",
    window.allocate_for(0.40)  // 13,107 tokens
);
```

### Near-Full Detection

```rust
if window.is_near_full() {
    // Context is >80% full, consider truncating
}
```

### Tests

18 unit tests covering:
- Default allocation calculation
- Total allocation must equal 1.0
- Token allocation per category
- Usage tracking (add/subtract)
- Remaining token calculation
- Usage percentage calculation
- Budget checking
- Budget fitting
- Truncation to target
- Entry recording and reporting
- Near-full detection
- Custom allocation support

## Integration Flow

The typical flow for assembling a prompt:

```
1. PromptManager            → Assemble template-based prompt components
2. ContextManager           → Aggregate all context sources
3. ContextWindow            → Track token usage against budget
4. TokenBudgetManager       → Enforce budget, recommend trimming
5. OpenAICompatibleProvider → Send assembled prompt to API
```

### Example: Complete Flow

```rust
// 1. Assemble prompt templates
let mut pm = PromptManager::new();
pm.add_system_template(PromptTemplate::new("sys", "You are {{role}}."));
let assembly = pm.assemble_prompt(sys_ctx, ws_ctx, ctx, "How do I debug this?");

// 2. Aggregate context
let mut cm = ContextManager::new(&assembly.system_prompt, "gpt-4o");
cm.add_source(manual_source);
cm.set_current_activity("Debugging");
let context = cm.build_context(messages, estimated_tokens);

// 3. Track tokens
let mut window = ContextWindow::new(32768);
window.record(&ContextEntry {
    label: "prompt".to_string(),
    tokens: count_tokens(&context.full_prompt()),
});

// 4. Enforce budget
let check = budget.check(
    system_tokens: window.estimate_system_tokens(),
    recent_tokens: window.estimate_recent_tokens(),
    ...
);
if !check.within_budget {
    // Trim conversation or drop low-priority context
}

// 5. Send to API
let provider = OpenAICompatibleProvider::new(...);
let response = provider.chat(AiRequest {
    messages: context.conversation_messages,
    ..default()
}).await?;
```

## Design Notes

1. **Priority-based truncation** — High-priority sources (system prompt, user context) are never truncated, while low-priority sources (auxiliary context, skill prompts) are removed first.

2. **Separation of concerns** — Context Manager handles *what* goes into the prompt, Context Window tracks *how many tokens* are consumed, Token Budget decides *what to trim* when over budget.

3. **Fluent builders** — Both `ContextBuilder` and `BudgetBuilder` use the builder pattern for intuitive, chainable construction of complex configurations.

4. **Tag-based filtering** — Sources can be tagged for flexible filtering, enabling "only show logs from production" or "only show Kubernetes context" queries.

5. **JSON export** — `AggregatedContext` serializes to JSON for persistence, debugging, and API requests.