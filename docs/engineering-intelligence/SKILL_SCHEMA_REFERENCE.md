# Skill Schema Reference

> Wiki Labs AI Copilot v0.4.0-alpha

## Overview

All Skill files are validated against JSON schemas. This document describes the structure and required fields for each schema.

## manifest.yaml

**Schema:** `schemas/manifest.schema.json`

Required fields:

| Field | Type | Description |
|---|---|---|
| `name` | string | Unique skill identifier (lowercase, hyphenated) |
| `version` | string | Semantic version (e.g., "0.4.0") |
| `display_name` | string | Human-readable name |
| `description` | string | One-line description |
| `author` | string | Skill author |

Optional fields:

| Field | Type | Description |
|---|---|---|
| `depends_on` | string[] | Other skill names this skill requires |
| `tags` | string[] | Categorization tags |
| `keywords` | string[] | Search keywords |
| `documentation_url` | string | URL to external documentation |

## technology.yaml

**Schema:** `schemas/technology.schema.json`

| Field | Type | Description |
|---|---|---|
| `name` | string | Technology name |
| `domain` | string | Category: infrastructure, monitoring, database, development |
| `vendor` | string | Vendor/organization |
| `version` | string | Target version or version range |

## intents.yaml

**Schema:** `schemas/intents.schema.json`

Array of intent objects:

| Field | Type | Description |
|---|---|---|
| `name` | string | Intent identifier |
| `description` | string | What this intent means |
| `priority` | number | Default priority (default: 5) |

## workflows.yaml

**Schema:** `schemas/workflows.schema.json`

| Field | Type | Description |
|---|---|---|
| `name` | string | Workflow identifier |
| `display_name` | string | Human-readable name |
| `description` | string | What this workflow does |
| `states` | object[] | Engineering states with transitions, evidence requirements, and completion criteria |

## detection_rules.yaml

**Schema:** `schemas/detection_rules.schema.json`

Array of rule objects:

| Field | Type | Description |
|---|---|---|
| `name` | string | Rule identifier |
| `detection_type` | string | browser_url, browser_title, terminal_command, active_application, file_pattern, workspace_context, conversation_keyword |
| `pattern` | string | Regex or literal pattern to match |
| `confidence` | number | Confidence score (0.0 – 1.0) |
| `enabled` | boolean | Whether the rule is active |

## commands.yaml

**Schema:** `schemas/commands.schema.json`

Array of command objects:

| Field | Type | Description |
|---|---|---|
| `name` | string | Command identifier |
| `description` | string | What the command does |
| `syntax` | string | Command syntax example |
| `categories` | string[] | Functional categories |
| `risk_level` | string | low, medium, high — execution risk assessment |

## best_practices.yaml

**Schema:** `schemas/best_practices.schema.json`

Array of practice objects:

| Field | Type | Description |
|---|---|---|
| `id` | string | Unique identifier |
| `title` | string | Practice title |
| `description` | string | Detailed description |
| `category` | string | Practice category |
| `relevance` | number | How relevant (0.0 – 1.0) |

## Validation

All schemas are stored in `src/skill_sdk/schemas/` and validated programmatically by the Skill Runtime. The `skill-validator` tool provides a CLI interface.

Example validation:

```bash
skill-validator openshift/
# Output: validation passed ✓
```