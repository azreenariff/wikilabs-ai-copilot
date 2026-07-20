# Skill Development Guide

> Wiki Labs AI Copilot v0.8.0-alpha  
> Phase 11 — Enterprise Skill Platform

## Overview

This guide covers how to create, develop, test, and distribute skills for Wiki Labs AI Copilot. Skills are the primary extension mechanism — they encapsulate all technology knowledge without modifying the core application.

## Prerequisites

- Rust toolchain (edition 2021)
- `skill-sdk` CLI (included with the copilot)
- Understanding of the target technology domain

## Quick Start

```bash
# Generate a new skill scaffold
skill-sdk create-skill my-skill --template technology

# Navigate into the generated skill
cd my-skill/

# Validate the skill
skill-sdk validate .

# Package the skill
skill-sdk package . -o my-skill.wls

# Install the skill
skill-sdk install my-skill.wls --target ~/.wikilabs/skills/
```

## Skill Structure

Every skill follows a standard directory structure:

```
skill-name/
  manifest.yaml              # Required: metadata and configuration
  technology.yaml            # Optional: technology definition
  intents.yaml               # Optional: technology-specific intents
  workflows.yaml             # Optional: engineering workflows
  detection_rules.yaml       # Optional: technology detection rules
  commands.yaml              # Optional: command reference
  best_practices.md          # Optional: engineering best practices
  known_issues.md            # Optional: known issues and workarounds
  guidance/
    rules.md                 # Optional: AI guidance rules
  knowledge/
    <topic>.md               # Optional: knowledge base documents
  prompts/
    <category>.yaml          # Optional: AI prompt templates
  examples/
    <scenario>.md            # Optional: example scenarios
  documentation/             # Optional: extended technical docs
  tests/                     # Optional: skill validation tests
```

## Step 1: Write the Manifest

The manifest is the only strictly required file. It declares your skill's identity and metadata.

```yaml
id: my-skill
name: My Engineering Skill
version: 1.0.0
description: >
  A comprehensive skill for managing XYZ technology.
  Provides expert-level guidance for engineers working
  with this technology stack.
author: Your Name or Team
technology_domain: MyTechnology
vendor: Wiki Labs           # or "customer" / "partner-<name>"
category: Engineering       # Engineering, Infrastructure, Security, etc.
dependencies:               # Skills this one depends on
  - linux-engineering       # e.g., depends on Linux basics
required_context:           # Context providers needed
  - terminal
  - browser
supported_environments:
  - ubuntu
  - rhel
schema_version: 1.0
tags:
  - mytechnology
  - infrastructure
keywords:
  - mytool
  - mycommand
icon: 🛠
documentation_url: https://docs.wikilabs.ai/skills/my-skill
enabled: true
```

**Key rules:**
- `id` must be lowercase, hyphenated, unique
- `version` must be valid semver
- `dependencies` must resolve to existing, loaded skills
- `category` must be one of the accepted values

## Step 2: Define Technology Details

```yaml
# technology.yaml
domain: MyTechnology
version: 2.x
description: >
  MyTechnology is a distributed system for managing...
features:
  - name: Cluster Management
    description: Node orchestration and scaling
    related_commands:
      - myctl cluster
      - myctl scale
  - name: Configuration
    description: Configuration management
    related_commands:
      - myctl config
      - myctl validate
```

## Step 3: Create Detection Rules

Detection rules tell the Skill Discovery Engine how to identify this technology in the engineer's environment.

```yaml
# detection_rules.yaml
- id: myctl-binary
  name: MyCLI Binary Detection
  description: Detect myctl command-line tool
  detection_type: Command
  pattern: ^myctl$
  confidence: 0.85
  technology_domain: MyTechnology
  priority: 10
  flags: ""
  extract: null

- id: myctl-config
  name: MyCLI Configuration File
  description: Detect myctl configuration directory
  detection_type: File
  pattern: ~/.myctl/config.yaml
  confidence: 0.9
  technology_domain: MyTechnology
  priority: 8
  flags: ""
  extract: null
```

**Confidence levels:**
- 0.9+: Strong indicators (CLI binary, config files in standard locations)
- 0.7–0.89: Moderate indicators (config files in non-standard locations)
- 0.5–0.69: Weak indicators (generic patterns)

## Step 4: Define Workflows

Workflows define engineering processes with states and transitions.

```yaml
# workflows.yaml
- id: cluster-deployment
  name: Cluster Deployment
  description: Deploy and verify a new cluster
  states:
    - id: planning
      name: Planning Phase
      description: Design cluster topology and requirements
      initial: true
      terminal: false
      commands:
        - myctl plan --dry-run
    - id: deployment
      name: Deployment Phase
      description: Deploy the cluster to target nodes
      initial: false
      terminal: false
      commands:
        - myctl deploy --config plan.yaml
    - id: verification
      name: Verification Phase
      description: Verify deployment health and connectivity
      initial: false
      terminal: true
      commands:
        - myctl cluster status
        - myctl health check
  transitions:
    - from: planning
      to: deployment
      condition: "plan_validated"
      description: "Proceed to deployment after plan review"
    - from: deployment
      to: verification
      condition: "deployment_complete"
      description: "Move to verification after deploy finishes"
    - from: verification
      to: deployment
      condition: "verification_failed"
      description: "Return to deployment if health checks fail"
  evidence_requirements:
    - "Cluster topology plan"
    - "Node status output"
    - "Health check results"
  required: true
```

**State properties:**
- `initial`: Only one state per workflow can be `true`
- `terminal`: Terminal states are end-states (no transitions from them unless they loop back)
- `commands`: Suggested diagnostic/investigative commands for each state

## Step 5: Document Commands

```yaml
# commands.yaml
- id: myctl-cluster-status
  name: myctl cluster status
  description: Show the current status of all cluster nodes
  syntax: myctl cluster status [--output json|yaml]
  category: cluster-management
  sudo: false
  example: myctl cluster status --output json
  note: Use JSON output for programmatic parsing
```

## Step 6: Write Knowledge Base

```markdown
# File: knowledge/cluster-scaling.md

# Cluster Auto-Scaling

## Overview
Auto-scaling adjusts cluster capacity based on workload demands...

## Configuration
```yaml
autoscaling:
  min_nodes: 3
  max_nodes: 10
  target_utilization: 70
```

## Troubleshooting
If scaling is not triggering...

## Best Practices
1. Set conservative minimums...
2. Monitor utilization trends...
```

## Step 7: Add Guidance Rules

```markdown
# File: guidance/rules.md

# AI Guidance Rules for My Technology

## General
- Always check cluster health before making changes
- Recommend commands, never execute them
- Provide evidence-based recommendations
- Include confidence scores with recommendations

## Safety
- Never recommend production changes without backup verification
- Always suggest dry-run mode first for destructive operations
- Flag any command requiring `sudo` or `root` access

## Investigation Steps
1. Gather current state (status, logs, metrics)
2. Identify the failure domain (node, network, config)
3. Narrow down the root cause
4. Recommend specific remediation steps
5. Verify fix after application
```

## Step 8: Write Tests

Create validation tests in the `tests/` directory. Tests validate the skill's own structure and content.

```rust
// tests/skill_validation.rs
#[test]
fn test_manifest_valid() {
    let manifest = load_manifest("./manifest.yaml").unwrap();
    assert_eq!(manifest.id, "my-skill");
    assert!(manifest.version.parse::<semver::Version>().is_ok());
    assert!(!manifest.tags.is_empty());
}

#[test]
fn test_detection_rules_valid() {
    let rules = load_detection_rules("./detection_rules.yaml").unwrap();
    for rule in &rules {
        assert!(!rule.id.is_empty());
        assert!(!rule.pattern.is_empty());
        assert!(rule.confidence >= 0.0 && rule.confidence <= 1.0);
    }
}

#[test]
fn test_workflows_valid() {
    let workflows = load_workflows("./workflows.yaml").unwrap();
    for wf in &workflows {
        let initial_states: Vec<_> = wf.states.iter()
            .filter(|s| s.initial)
            .collect();
        assert_eq!(initial_states.len(), 1, "Exactly one initial state");
        let terminal_states: Vec<_> = wf.states.iter()
            .filter(|s| s.terminal)
            .collect();
        assert!(!terminal_states.is_empty(), "At least one terminal state");
    }
}
```

## SDK Commands Reference

```bash
# Create a new skill scaffold
skill-sdk create-skill <name> --template <type>

# Validate a skill package
skill-sdk validate <path>

# Package into .wls archive
skill-sdk package <path> -o <output.wls>

# Extract a .wls package
skill-sdk extract <package.wls> -o <output-dir>

# Install a skill (from .wls or directory)
skill-sdk install <path> --target <skills-dir>

# Uninstall a skill
skill-sdk uninstall <skill-id> --target <skills-dir>

# Update a skill from a new package
skill-sdk update <skill-id> --package <new.wls>

# List available templates
skill-sdk list-templates

# List installed skills
skill-sdk list --target <skills-dir>
```

### Template Types

| Type | Description | Files |
|------|-------------|-------|
| `technology` | Full technology skill | All files |
| `workflow` | Workflow-only skill | manifest, workflows |
| `command` | Command reference | manifest, commands |
| `detection` | Detection rules | manifest, detection_rules |
| `intent` | Intent definitions | manifest, intents |
| `knowledge` | Knowledge base | manifest, knowledge/ |
| `policy` | Policy/guidelines | manifest, best_practices |
| `guidance` | AI guidance | manifest, guidance/ |

## Design Principles

All skills must follow these principles:

1. **No autonomous execution** — Skills recommend commands; the engineer runs them
2. **No system modification** — Skills never change configuration or perform remediation
3. **No automation** — Skills are knowledge sources, not automation tools
4. **Evidence-based** — All recommendations must cite evidence (files, logs, metrics)
5. **Confidence-scoring** — Every detection and recommendation includes a confidence value
6. **Human-in-the-loop** — The engineer approves all actions before they happen
7. **Technology-agnostic core** — The application core contains no technology-specific logic

## Publishing to the Copilot

Built-in skills are shipped with the copilot application. To add a skill:

1. Develop and test the skill locally
2. Validate the skill with `skill-sdk validate`
3. Package the skill with `skill-sdk package`
4. Include the `.wls` file in the copilot's skills directory
5. Update CHANGELOG.md with the new skill entry
6. Commit to the repository

## Troubleshooting

### Skill won't activate

Check these in order:
1. Is the skill `enabled: true` in its manifest?
2. Are all dependencies loaded and enabled?
3. Is the discovery confidence ≥ the activation threshold (default 0.7)?
4. Does the `technology_domain` match a registered signature?

### Validation fails

Run `skill-sdk validate` and check each reported error:
- Missing required fields → add them to `manifest.yaml`
- Duplicate IDs → ensure all IDs are unique within each YAML array
- Invalid YAML → check syntax with a YAML validator
- Dependency resolution failure → verify dependency IDs exist

### Confidence too low

Add more detection rules or lower the confidence of existing rules:
- Strong signals: CLI binary presence, config file in standard location
- Moderate signals: Package manager detection, config directory
- Weak signals: Generic file patterns, process names