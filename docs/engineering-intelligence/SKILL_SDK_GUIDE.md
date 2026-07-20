# Skill SDK Guide

> Wiki Labs AI Copilot v0.8.0-alpha

## Purpose

The Skill SDK allows engineers to create new Technology Skills without modifying the core application.

## What the SDK Provides

### 1. Skill Template Generator

Create a new skill scaffold:

```bash
create-skill <name>
```

Example:

```bash
create-skill vmware
```

Produces:

```
vmware/
  manifest.yaml
  workflows.yaml
  intents.yaml
  tests/
```

### 2. Skill Scaffolding Templates

Templates for each Skill component:

- `manifest.yaml.template` — Skill metadata
- `technology.yaml.template` — Technology definition
- `intents.yaml.template` — Intent definitions
- `workflows.yaml.template` — Workflow definitions
- `detection_rules.yaml.template` — Detection rules
- `commands.yaml.template` — Common commands
- `known_issues.yaml.template` — Known issues

Templates are in `src/skill_sdk/templates/`.

### 3. Schema Validation

JSON schemas validate Skill files. Run validation:

```bash
skill-validator <path-to-skill>
```

Schemas are in `src/skill_sdk/schemas/`:

- `manifest.schema.json`
- `technology.schema.json`
- `intents.schema.json`
- `workflows.schema.json`
- `detection_rules.schema.json`
- `commands.schema.json`
- `best_practices.schema.json`

### 4. Implementation

- `src/skill_sdk/src/lib.rs` — SDK core with template generation and validation
- `src/skill_sdk/templates/` — YAML template files
- `src/skill_sdk/schemas/` — JSON schema files
- `src/skill_sdk/SKILL_DEVELOPMENT_GUIDE.md` — Full developer documentation

## Creating a New Skill — Quick Start

1. Run `create-skill <name>` to generate the scaffold
2. Fill in `manifest.yaml` with metadata
3. Fill in `technology.yaml` with technology details
4. Define intents in `intents.yaml`
5. Define workflows in `workflows.yaml`
6. Add detection rules in `detection_rules.yaml`
7. Validate with `skill-validator <path>`
8. Place the Skill in the Skills directory
9. Restart the application — Skill Runtime auto-discovers new Skills

## Constraints

Skills must not:

- Execute commands directly
- Access MCP tools autonomously
- Perform RAG or knowledge retrieval
- Automate actions without human approval
- Access customer environments directly

The AI remains advisory. The human engineer executes all actions.