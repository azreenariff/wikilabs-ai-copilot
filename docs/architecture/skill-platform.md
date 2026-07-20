# Skill Platform — Architecture

## Overview

The Skill Platform enables the AI Copilot to dynamically discover, activate, and manage enterprise skills based on workspace technology signatures. It consists of three core components:

1. **Skill Discovery Engine** — Scans the workspace for technology signals
2. **Skill Activation Engine** — Dynamically activates skills based on detected signals
3. **Skill Runtime** — Manages loaded skills, validation, and lifecycle

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  AI Copilot                      │
├─────────────────────────────────────────────────┤
│              Skill Management UI                 │
│  (Tauri Panel: List, Enable/Disable, Validate)  │
├─────────────────────────────────────────────────┤
│              Skill Runtime                       │
│  ┌─────────────┐  ┌──────────────────────────┐  │
│  │ Skill SDK   │  │ Skill Platform Engines   │  │
│  │  (Templates)│  │  ┌────────────────────┐  │  │
│  │             │  │  │ Discovery Engine   │  │  │
│  │ Schema      │  │  │  - Scan workspace  │  │  │
│  │ Validation  │  │  │  - Match patterns  │  │  │
│  │ Generation  │  │  │  - Compute signals │  │  │
│  └─────────────┘  │  │                    │  │  │
│                   │  │  ┌────────────────┐ │  │  │
│                   │  │  │ Activation     │ │  │  │
│                   │  │  │  Engine        │ │  │  │
│                   │  │  │  - Match       │ │  │  │
│                   │  │  │  - Activate    │ │  │  │
│                   │  │  │  - Health      │ │  │  │
│                   │  │  └────────────────┘ │  │  │
│                   │  └────────────────────┘  │  │
│                   └──────────────────────────┘  │
├─────────────────────────────────────────────────┤
│              Knowledge Packs                     │
│         (YAML: manifest, detection, etc.)       │
├─────────────────────────────────────────────────┤
│              Workspace                           │
│         (Files, Commands, Configurations)       │
└─────────────────────────────────────────────────┘
```

## Data Flow

```
Workspace Scan → Technology Signals → Skill Matching → Dynamic Activation
     ↓                                          ↓
  File Patterns                           Activated Skills
  Command Outputs                         → Intent Definitions
  Config Files                            → Detection Rules
  Log Patterns                            → Workflow Definitions
                                          → Command References
```

## Component Responsibilities

### Skill Discovery Engine
- Scans workspace directories using glob patterns
- Matches file patterns against technology signatures
- Checks command presence in scripts and configs
- Computes confidence scores (0.0–1.0)
- Produces a `DiscoveryReport` with `TechSignal` entries

### Skill Activation Engine
- Receives `ActivationCandidate` list from discovery
- Validates skill dependencies are met
- Activates matching skills dynamically
- Performs periodic health checks
- Deactivates degraded skills after max failures

### Skill Runtime
- Loads and validates skill YAML manifests
- Manages skill enable/disable state
- Resolves skill dependencies
- Provides access to intents, workflows, detection rules, commands
- Integrates discovery and activation engines

### Skill SDK
- Generates skill templates from parameters
- Validates skill component YAML files
- Provides schema registry for all component types
- Supports skill packaging for distribution

## Lifecycle

```
Discover → Validate → Load → Enable → Activate → Monitor → (Deactivate/Update)
   ↑                                                        ↓
   └──────────────────────────── Update ←───────────────────┘
```

## Schema

Skills use YAML manifests with the following structure:

```yaml
id: <skill-id>
name: <Display Name>
version: <semantic version>
description: <Description>
author: <Author>
technology_domain: <Technology>
vendor: <Vendor>
category: <Category>
dependencies: [<list of dependent skill IDs>]
enabled: true
schema_version: "1.0"
tags: [<tags>]
keywords: [<keywords>]
icon: <emoji or icon name>
supported_environments: [<distros/environments>]
documentation_url: <URL>
```

## Error Handling

- **Discovery failures**: Logged as warnings, non-blocking
- **Activation failures**: Logged, skill stays inactive
- **Health check failures**: Counted toward max_failure_count
- **Schema validation errors**: Reported in validation report
- **Missing dependencies**: Blocking — skill cannot be activated

## Extensibility

New technology signatures can be registered via:

```rust
engine.register_signature(TechSignature {
    domain: "Kubernetes".to_string(),
    file_patterns: vec!["**/kubeconfig".to_string()],
    command_patterns: vec!["kubectl".to_string()],
    base_confidence: 0.85,
    priority: 9,
});
```

New skill definitions for activation:

```rust
activation.register_skill(SkillDefinition {
    id: "kubernetes-engineering".to_string(),
    name: "Kubernetes Engineering".to_string(),
    technology: "Kubernetes".to_string(),
    category: "Engineering".to_string(),
    enabled: true,
    dependencies: vec![],
});
```