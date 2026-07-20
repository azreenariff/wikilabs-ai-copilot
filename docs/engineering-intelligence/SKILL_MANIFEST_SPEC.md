# Skill Manifest Specification

> Wiki Labs AI Copilot v0.8.0-alpha  
> Phase 11 — Enterprise Skill Platform

## Purpose

The Skill Manifest (`manifest.yaml`) is the single source of truth for every skill in the Wiki Labs AI Copilot. It declares what the skill is, what it does, what it depends on, and how the platform should treat it.

## File Location

```
skill-name/
  manifest.yaml          ← REQUIRED
  technology.yaml
  workflows.yaml
  detection_rules.yaml
  commands.yaml
  ...
```

The Skill Runtime loads and validates every skill's manifest before registering it.

## Full Schema

### Required Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `id` | string | Unique, lowercase, hyphenated identifier | `linux-engineering` |
| `name` | string | Human-readable display name | `Linux Engineering` |
| `version` | string | Semantic version | `1.0.0` |
| `description` | string | Multi-line description of the skill's purpose | `Comprehensive Linux engineering skill...` |
| `author` | string | Who developed the skill | `Wiki Labs Team` |
| `technology_domain` | string | Primary technology this skill covers | `Linux` |
| `vendor` | string | Skill vendor (always `Wiki Labs` for built-in skills) | `Wiki Labs` |
| `category` | string | Skill category | `Engineering` |
| `schema_version` | string | Manifest schema version this file conforms to | `1.0` |
| `enabled` | boolean | Whether the skill is available for use | `true` |

### Optional Fields

| Field | Type | Description | Default | Example |
|-------|------|-------------|---------|---------|
| `dependencies` | array of strings | List of skill `id`s this skill depends on | `[]` | `["linux-engineering"]` |
| `tags` | array of strings | Searchable tags for discovery and filtering | `[]` | `["linux", "sysadmin"]` |
| `keywords` | array of strings | Keywords used by the discovery engine for matching | `[]` | `["systemctl", "journalctl"]` |
| `required_context` | array of strings | Context providers the skill needs to function | `[]` | `["browser", "terminal"]` |
| `supported_environments` | array of strings | Operating systems / platforms this skill supports | all | `["ubuntu", "rhel"]` |
| `icon` | string | Emoji or icon for UI display | none | `🐧` |
| `documentation_url` | string | URL to skill documentation | none | `https://docs.wikilabs.ai/skills/linux-engineering` |
| `compatibility` | object | Compatibility metadata | `{}` | — |

### Compatibility Object

| Field | Type | Description |
|-------|------|-------------|
| `copilot_min` | string | Minimum copilot version this skill supports |
| `copilot_max` | string | Maximum copilot version this skill supports |
| `rust_version` | string | Minimum Rust version required |

### Environment Object

| Field | Type | Description |
|-------|------|-------------|
| `os` | array of strings | Required operating systems |
| `architecture` | array of strings | Required CPU architectures |
| `privileges` | string | Required privilege level (`user`, `sudo`, `root`) |

## Example Manifest

```yaml
id: linux-engineering
name: Linux Engineering
version: 1.0.0
description: >
  Comprehensive Linux engineering skill covering system administration,
  troubleshooting, kernel management, networking, storage, and security.
  Provides expert-level guidance for Linux administrators and DevOps engineers.
author: Wiki Labs Team
technology_domain: Linux
vendor: Wiki Labs
category: Engineering
dependencies: []
required_context:
  - terminal
  - screen
supported_environments:
  - ubuntu
  - debian
  - centos
  - rhel
  - fedora
  - arch
  - alpine
  - sles
schema_version: 1.0
tags:
  - linux
  - sysadmin
  - devops
keywords:
  - systemctl
  - systemd
  - lvm
  - filesystem
  - ssh
  - kernel
icon: 🐧
documentation_url: https://docs.wikilabs.ai/skills/linux-engineering
```

## Field Validation Rules

The Skill SDK validates manifests against the JSON schema `manifest.schema.json`. Key validation rules:

1. **`id`** — Must be lowercase alphanumeric with hyphens only. No spaces or special characters.
2. **`version`** — Must be valid semantic versioning (`MAJOR.MINOR.PATCH`).
3. **`schema_version`** — Must match the runtime's supported schema version (`1.0`).
4. **`dependencies`** — Each dependency `id` must resolve to an existing, loaded skill.
5. **`enabled`** — Boolean value (`true`/`false`). Controls whether the skill can be activated.
6. **`category`** — Must be one of: `Engineering`, `Infrastructure`, `Security`, `Monitoring`, `Database`, `Development`, `Networking`, `Virtualization`, `Automation`.
7. **`vendor`** — Built-in skills: `Wiki Labs`. Custom skills: `customer` or `partner-<name>`.
8. **`required_context`** — Each provider must be a known context source (`terminal`, `browser`, `screen`, `clipboard`, `mcp`).

## Validation Commands

```bash
# Validate a skill manifest
skill-validator manifest /path/to/skill/manifest.yaml

# Validate the entire skill package
skill-validator package /path/to/skill/
```

## Schema Evolution

When the manifest schema changes:

1. Increment `schema_version` in the runtime
2. Update `manifest.schema.json` with new fields
3. Add migration guidance to the changelog
4. Skills with old `schema_version` values are flagged as outdated

The Skill Runtime rejects skills whose `schema_version` exceeds the runtime's supported schema version.

## Lifecycle States

A manifest declares a skill's initial state. The Skill Runtime manages transitions:

```
Loaded → Enabled → Active → Suspended → Disabled
         ↓          ↓          ↓
       (health failure) → Degraded → Active
```

The manifest's `enabled: false` maps directly to the runtime's `Disabled` state. All other state transitions happen dynamically at runtime.