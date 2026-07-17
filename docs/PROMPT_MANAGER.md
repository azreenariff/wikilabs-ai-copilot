# Prompt Manager

## Overview

The Prompt Manager (`PromptManager`) handles prompt template management, versioning, and assembly of complete prompts from multiple components.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Prompt Manager                         │
│                                                           │
│  ┌─────────────────┐ ┌─────────────────┐                 │
│  │ System Templates│ │ Workspace       │                 │
│  │ (versioned)     │ │ Templates       │                 │
│  └────────┬────────┘ └────────┬────────┘                 │
│           │                   │                           │
│  ┌────────┴────────┐ ┌────────┴────────┐                 │
│  │ Skill Templates │ │ User Templates  │                 │
│  └────────┬────────┘ └────────┬────────┘                 │
│           │                   │                           │
│           ▼                   ▼                           │
│  ┌────────────────────────────────────────┐               │
│  │       PromptAssembler                  │               │
│  │  • System prompt                       │               │
│  │  • Workspace context                   │               │
│  │  • Skill prompts                       │               │
│  │  • Context prompts                     │               │
│  │  • User message                        │               │
│  └────────────────────────────────────────┘               │
└──────────────────────────────────────────────────────────┘
```

## Features

### Template Management

Prompt templates are managed in categories:

- **System Templates**: System prompts that define AI behavior
- **Workspace Templates**: Context about the current workspace
- **Skill Templates**: Prompts that apply when specific skills are active
- **Context Templates**: Additional context templates
- **User Templates**: User message formatting templates

### Template Versioning

Each template has a version for tracking changes:

```rust
pub enum PromptVersion {
    Default,       // Version 1, default
    Numbered(u32), // Versioned (v2, v3, ...)
    Named(String), // Named version ("stable", "experimental")
}
```

### Template Context

Templates can use placeholders that are replaced during assembly:

```rust
// Template
"You are an engineer working on {{project}}."

// Template context
project -> "wikilabs"

// Result
"You are an engineer working on wikilabs."
```

### Prompt Assembly

The `PromptAssembler` combines all prompt components:

```rust
let assembly = PromptAssembler::new()
    .with_system_prompt("You are helpful.")
    .with_workspace_prompt("Workspace: /my/project")
    .add_skill_prompt("Skill: debug")
    .add_context_prompt("Context: error log")
    .with_user_prompt("Explain the error.")
    .with_template_context("project", "wikilabs")
    .assemble();
```

## API Reference

### Creating a Prompt Manager

```rust
let mut pm = PromptManager::new();
```

### Adding Templates

```rust
pm.add_system_template(PromptTemplate::new(
    "sys",
    "You are a senior infrastructure engineer."
));

pm.add_workspace_template(PromptTemplate::new(
    "ws",
    "Workspace: {{workspace_path}}"
));
```

### Setting Active Templates

```rust
pm.activate_system_template("sys_v2");
pm.set_user_template(PromptTemplate::new("user", "{{message}}"));
```

### Assembling Prompts

```rust
let assembly = pm.assemble_user_message("What is this error?", None);
```

## Template Syntax

Templates use `{{key}}` placeholders:

```
You are an engineer. Project: {{project_name}}.
Technology stack: {{technology}}.
```

Placeholders are replaced from the `TemplateContext`:

```rust
TemplateContext::new()
    .with("project_name", "wikilabs")
    .with("technology", "rust,kubernetes")
    .apply_to(template);
```

## Testing

Comprehensive test coverage for:

- Template creation and management
- Version bumping and tracking
- Placeholder replacement
- Prompt assembly with all components
- Template deactivation and reactivation