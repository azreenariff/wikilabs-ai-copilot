# Skill Architecture

> Wiki Labs AI Copilot v0.8.0-alpha

## Design Principle

The core application must remain completely technology-agnostic. All technology knowledge comes from Skills.

## Skill Definition

A Skill represents an engineering domain (OpenShift, VMware, Linux, Nagios, etc.) and contains declarative configuration that the Skill Runtime loads and validates.

## Skill Package Structure

```
skill-name/
  manifest.yaml           # Metadata, version, dependencies
  technology.yaml         # Technology definition
  intents.yaml            # Technology-specific intents
  workflows.yaml          # Engineering workflow definitions
  detection_rules.yaml    # Rules to detect this technology
  commands.yaml           # Common commands and patterns
  best_practices.yaml     # Engineering best practices
  known_issues.yaml       # Known issues and workarounds
  prompts/                # AI prompt templates
  examples/               # Example scenarios
  documentation/          # Technical documentation
  tests/                  # Skill validation tests
  mcp/                    # (Optional) MCP tool definitions
```

## Skill Components

### manifest.yaml

```yaml
id: openshift
name: "Red Hat OpenShift"
version: "0.4.0"
display_name: "Red Hat OpenShift"
description: "OpenShift cluster operations and troubleshooting"
author: "Wiki Labs"
dependencies: []
```

### technology.yaml

Defines what the technology is:

```yaml
technology:
  name: OpenShift
  domain: infrastructure
  vendor: Red Hat
  version: "4.x"
```

### intents.yaml

Technology-specific goals:

```yaml
intents:
  - name: installation
    description: "Install and configure OpenShift cluster"
  - name: troubleshooting
    description: "Diagnose and resolve issues"
  - name: upgrade
    description: "Upgrade cluster version"
```

### workflows.yaml

Engineering process states and transitions (see WORKFLOW_ENGINE.md).

### detection_rules.yaml

Rules that identify this technology from observations (see TECHNOLOGY_RECOGNITION.md).

## Skill Runtime

The Skill Runtime is responsible for:

1. **Discover** — Find available Skills in configured directories
2. **Load** — Parse and validate Skill files
3. **Validate** — Check schema compliance and version compatibility
4. **Version management** — Track Skill versions
5. **Enable/disable** — Toggle Skills without removing them
6. **Dependency checking** — Ensure required Skills are available

## Implementation

- `src/skill_runtime/src/lib.rs` — Skill Runtime implementation
- `src/skill_sdk/src/lib.rs` — Skill Development Kit
- `src/skill_sdk/templates/` — Skill scaffolding templates
- `src/skill_sdk/schemas/` — JSON schemas for validation

## Testing

See `src/skill_runtime/src/lib.rs` for:

- Skill discovery tests
- Skill loading tests
- Skill validation tests
- Dependency resolution tests
- Enable/disable lifecycle tests