# Prompt Manager — Wiki Labs AI Copilot

## Overview

The Prompt Manager (`prompt_manager.rs`) provides template-driven prompt assembly with versioning and placeholder substitution. It enables dynamic prompt construction from reusable templates, with support for system prompts, workspace prompts, context prompts, user prompts, and skill prompts.

**Module:** `src/ai/src/prompt_manager.rs`  
**Lines of code:** ~636 (including tests)  
**Tests:** 26 unit tests

## Architecture

### Core Types

```rust
// A versioned prompt template
PromptTemplate {
    id: Uuid,
    name: String,
    content: String,           // Template with {{placeholders}}
    version: PromptVersion,    // Default | Numbered(usize) | Named(String)
    active: bool,
    description: Option<String>,
}

// Version tracking for templates
enum PromptVersion {
    Default,                // Unversioned
    Numbered(usize),        // v1, v2, v3...
    Named(String),          // "stable", "experimental", "beta"
}

// Placeholder values for template substitution
TemplateContext {
    context: HashMap<String, String>,  // key → value
}

// A fully assembled prompt
PromptAssembly {
    prompt: String,                  // Complete assembled prompt
    system_prompt: String,           // System prompt portion
    workspace_prompt: String,        // Workspace context portion
    context_prompt: String,          // Context portion
    user_prompt: String,             // User message portion
    placeholders_replaced: usize,    // Count of {{key}} substitutions
    template_versions_used: Vec<String>,  // Which templates were used
}

// Manager for all prompt templates
PromptManager {
    system_templates: Vec<PromptTemplate>,
    workspace_templates: Vec<PromptTemplate>,
    context_templates: Vec<PromptTemplate>,
    user_template: Option<PromptTemplate>,
    skill_templates: Vec<PromptTemplate>,
}
```

## Template Syntax

Templates use `{{key}}` placeholder syntax:

```
You are a {{role}}.
Customer: {{customer}}
Task: {{task}}

Debug: {{diagnosis}}
```

Placeholders are replaced by `TemplateContext`:

```rust
let context = TemplateContext::new()
    .with("role", "Senior Engineer")
    .with("customer", "ABC Bank")
    .with("task", "debug the application")
    .with("diagnosis", "Configuration mismatch");

let (result, count) = context.apply_to(template_content);
// result: "You are a Senior Engineer.\nCustomer: ABC Bank..."
// count: 4
```

## Template Categories

### System Templates

Core behavioral definition for the AI:

```rust
pm.add_system_template(PromptTemplate::new(
    "Engineering",
    "You are a senior infrastructure engineer.\n\nRules:\n1. Explain reasoning step-by-step\n2. Prefer evidence-based recommendations"
).with_description("Default engineering persona"));
```

### Workspace Templates

Per-workspace context injection:

```rust
pm.add_workspace_template(PromptTemplate::new(
    "Customer Context",
    "Customer: {{customer}}\nEnvironment: {{env}}\nRegion: {{region}}\nStack: {{stack}}"
));
```

### Context Templates

Dynamic context blocks added based on current activity:

```rust
pm.add_context_template(PromptTemplate::new(
    "Error Context",
    "## Errors Found\n{{error_summary}}\n\n## Affected Systems\n{{affected_systems}}"
));
```

### User Templates

Formatting for user messages:

```rust
pm.set_user_template(PromptTemplate::new(
    "User Question",
    "## User Message\n{{message}}\n\n## Context\n{{additional_context}}"
));
```

### Skill Templates

Instructions from loaded skills (future):

```rust
pm.add_skill_template(PromptTemplate::new(
    "Linux Skill",
    "When working with Linux systems, always check: disk usage, memory, and process status."
));
```

## Assembly Process

The `PromptAssembler` builds prompts incrementally:

```rust
let assembly = PromptAssembler::new()
    .with_system_prompt("You are an engineer.")
    .with_workspace_prompt("Customer: ABC Bank")
    .with_context_prompt("Debugging pod crash")
    .with_user_prompt("How do I fix this?")
    .add_skill_prompt("Always check logs first")
    .with_template_context("diagnosis", "OOM")
    .assemble();
```

The resulting prompt is assembled in sections:

```
You are an engineer.

## Workspace Context
Customer: ABC Bank

## Context
Debugging pod crash

## Skill
Always check logs first

## User Message
How do I fix this?
```

Each section is prefixed with a Markdown header (except system prompt), and sections are separated by blank lines.

## PromptManager API

### Adding Templates

```rust
let mut pm = PromptManager::new();

pm.add_system_template(template);
pm.add_workspace_template(template);
pm.add_context_template(template);
pm.set_user_template(template);
pm.add_skill_template(template);
```

### Getting Active Templates

Only one template per category can be active at a time:

```rust
let sys = pm.active_system_template();      // Option<&PromptTemplate>
let ws = pm.active_workspace_template();
let ctx = pm.active_context_template();
```

When the first template is added, it is automatically active. Deactivating it switches to the next:

```rust
pm.deactivate_system_template(template_id);
```

### Assembling Full Prompts

```rust
let assembly = pm.assemble_prompt(
    TemplateContext::new(),                  // System vars
    TemplateContext::new().with("customer", "ABC"),  // Workspace vars
    TemplateContext::new(),                  // Context vars
    "How do I debug this?",                  // User message
);
```

## Versioning

### Bumping Versions

```rust
let mut template = PromptTemplate::new("Debug", "{{diagnosis}}");
assert_eq!(template.version, PromptVersion::Default);

template.bump_version();
assert_eq!(template.version, PromptVersion::Numbered(1));

template.bump_version();
assert_eq!(template.version, PromptVersion::Numbered(2));
```

### Named Versions

```rust
template.set_version(PromptVersion::Named("stable"));
// Display: "stable"
```

### Version Display

```rust
PromptVersion::Default.to_string()       → "default"
PromptVersion::Numbered(3).to_string()   → "v3"
PromptVersion::Named("stable").to_string() → "stable"
```

## Placeholder Counting

Track how many placeholders are present (without substituting):

```rust
let count = context.count_placeholders("Process {{target}} in {{env}}");
assert_eq!(count, 2);
```

And how many were actually replaced:

```rust
let (result, count) = context.apply_to("Process {{target}}");
assert_eq!(count, 1);
```

## Placeholder Syntax

| Pattern | Meaning |
|---|---|
| `{{key}}` | Replace `key` with value from TemplateContext |
| `{{unknown}}` | Left as-is if not in context |
| `plain text` | Passed through unchanged |
| `{{a}} {{b}} {{c}}` | Three placeholders in one string |

## Tests

26 unit tests covering:

| Category | Tests |
|---|---|
| **Template basics** | Creation, versioning, description, activate/deactivate |
| **Placeholder substitution** | Basic replacement, no placeholders, counting |
| **PromptAssembly** | Basic assembly, all parts, skill prompts, template context |
| **PromptManager** | Creation, adding templates, active template selection, deactivation |
| **Full assembly** | With workspace vars, no system template, all sections |
| **Version display** | Default, numbered, named versions |

## Usage in the Copilot

The Prompt Manager integrates with:

- **Context Manager** — Provides workspace context as template variables
- **Session Manager** — Each session can have its own system prompt template
- **Persona Module** — The engineering persona is delivered as a system template
- **Tauri Commands** — Prompt templates can be stored/loaded via persistence layer

## Design Notes

1. **Active template per category** — Simplifies prompt assembly by always using the active template, while keeping alternatives available for switching.

2. **Template versioning** — Enables safe evolution of prompt templates with clear version tracking. Useful for A/B testing prompts or rolling out changes gradually.

3. **Placeholder flexibility** — `{{key}}` syntax is simple, well-understood, and compatible with LLM tokenization. The regex `{{(\w+)}}` handles common placeholder patterns.

4. **Section-based assembly** — Each prompt component gets a Markdown header, making the assembled prompt readable and structured for the LLM. This mirrors the structure used in the Tauri app's prompt construction.

5. **No persistence in module** — Template storage is handled by the persistence layer. The PromptManager focuses on template logic and assembly.