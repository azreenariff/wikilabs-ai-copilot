# Skill Pack Development Guide — Wiki Labs AI Copilot v1.0.0

> How to create, test, and distribute skill packs. Skill pack format, hooks, catalog, and SDK usage.

## Table of Contents

1. [Introduction](#introduction)
2. [Skill Pack Overview](#skill-pack-overview)
3. [Skill Pack Structure](#skill-pack-structure)
4. [Core Manifest Files](#core-manifest-files)
5. [Skill Components](#skill-components)
6. [Technology Detection Rules](#technology-detection-rules)
7. [Workflows](#workflows)
8. [Commands](#commands)
9. [Knowledge Base](#knowledge-base)
10. [Skill SDK Usage](#skill-sdk-usage)
11. [Testing a Skill Pack](#testing-a-skill-pack)
12. [Distribution](#distribution)
13. [Quality Standards](#quality-standards)
14. [Best Practices](#best-practices)
15. [Example: Creating a New Skill Pack](#example-creating-a-new-skill-pack)

---

## Introduction

Skill packs are the primary mechanism for engineering knowledge in Wiki Labs AI Copilot. They bundle technology-specific detection rules, troubleshooting workflows, commands, knowledge base documents, reasoning guides, and best practices into a single distributable package.

This guide covers:
- The skill pack file format and structure
- Each component type and how to author them
- Using the Skill SDK for template generation and validation
- Testing skill packs before distribution
- Distribution and update procedures

---

## Skill Pack Overview

A skill pack is a directory in `src/skills/` containing YAML manifest files and Markdown documentation. The application discovers and loads skill packs at startup.

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Skill Pack** | A directory containing all files for one technology domain |
| **Manifest** | `manifest.yaml` — metadata and configuration |
| **Technology Definition** | `technology.yaml` — features, capabilities, and scope |
| **Detection Rules** | `detection_rules.yaml` — patterns for auto-detection |
| **Workflows** | `workflows.yaml` — troubleshooting workflows |
| **Commands** | `commands.yaml` — command knowledge base |
| **Knowledge** | `knowledge/` directory — deep technical documents |
| **Reasoning Guide** | `reasoning/` directory — how to analyze and diagnose |
| **Guidance Rules** | `guidance/rules.md` — engineering reasoning guidance |

### Pre-installed Skill Packs (v1.0.0)

| Skill Pack | Technology | Domain | Files |
|------------|-----------|--------|-------|
| openshift-skill-pack | Red Hat OpenShift 4.x | Containers | 40 |
| linux-engineering | Linux Administration | Infrastructure | 40 |
| vmware-vsphere-skill-pack | VMware vSphere | Virtualization | 40 |
| nagios-xi-skill-pack | Nagios XI | Monitoring | 19 |
| nagios-logserver-skill-pack | Nagios Log Server | Log Management | 20 |
| checkmk-skill-pack | Checkmk 2.3/2.4 | Monitoring | 21 |
| ansible-skill-pack | Ansible | Automation | 20 |
| mysql-skill-pack | MySQL DBA 8.0 | Database | 41 |
| edb-postgresql-skill-pack | EDB PostgreSQL 15/16 | Database | 34 |
| mssql-skill-pack | Microsoft SQL Server 2022 | Database | 28 |

---

## Skill Pack Structure

The standard skill pack structure follows the OpenShift skill pack pattern:

```
my-technology-skill-pack/
├── manifest.yaml              # [REQUIRED] Skill pack metadata and configuration
├── technology.yaml            # [RECOMMENDED] Technology coverage and features
├── workflows.yaml             # [RECOMMENDED] Troubleshooting workflows
├── detection_rules.yaml       # [OPTIONAL] Context detection patterns
├── commands.yaml              # [OPTIONAL] Command knowledge base
├── best_practices.md          # [OPTIONAL] Best practices and standards
├── known_issues.md            # [OPTIONAL] Known failure patterns and workarounds
├── <TECH>_SKILL_PACK.md       # [OPTIONAL] Master skill pack document
├── <TECH>_DETECTION.md        # [OPTIONAL] Detection documentation
├── <TECH>_WORKFLOWS.md        # [OPTIONAL] Workflow documentation
├── <TECH>_COMMAND_REFERENCE.md # [OPTIONAL] Command reference
├── <TECH>_GUIDANCE.md         # [OPTIONAL] Guidance documentation
├── <TECH>_BEST_PRACTICES.md   # [OPTIONAL] Best practices documentation
├── <TECH>_COMMON_FAILURES.md  # [OPTIONAL] Common failures documentation
├── <TECH>_REASONING_GUIDE.md  # [OPTIONAL] Reasoning guide documentation
├── SKILL_PACK_QUALITY_STANDARD.md # [OPTIONAL] Quality standards for this skill
├── knowledge/                 # [RECOMMENDED] Deep knowledge base
│   ├── cluster-architecture.md
│   ├── security.md
│   ├── backup-recovery.md
│   └── ...
├── documentation/             # [OPTIONAL] Reference documentation
├── examples/                  # [OPTIONAL] Worked examples
├── tests/                     # [RECOMMENDED] Validation tests
├── references/                # [OPTIONAL] Documentation references
├── architecture/              # [OPTIONAL] Architecture details
├── concepts/                  # [OPTIONAL] Concept explanations
├── workflows/                 # [OPTIONAL] Workflow documentation
├── reasoning/                 # [OPTIONAL] Reasoning guides
├── detection/                 # [OPTIONAL] Detection documentation
├── diagnostics/               # [RECOMMENDED] Diagnostic procedures
├── context/                   # [OPTIONAL] Context interpretation
├── commands/                  # [OPTIONAL] Command reference
├── guidance/                  # [OPTIONAL] Guidance reference
├── common-failures/           # [OPTIONAL] Failure pattern database
│   └── reference.md
└── overview/                  # [OPTIONAL] Overview documentation
    └── reference.md
```

### Minimum Viable Skill Pack

At minimum, a skill pack requires:

```
minimal-skill-pack/
├── manifest.yaml              # [REQUIRED]
└── best_practices.md          # [OPTIONAL but recommended]
```

The Skill SDK's `validate_skill()` function requires `manifest.yaml` with valid `id`, `name`, and `version` fields. All other components are optional.

---

## Core Manifest Files

### manifest.yaml

The manifest is the primary metadata file for a skill pack. It must include the required fields shown below.

```yaml
# manifest.yaml

# [REQUIRED] Unique identifier (lowercase, hyphens)
id: my-technology-skill-pack

# [REQUIRED] Human-readable name
name: My Technology Engineering

# [REQUIRED] Semantic version
version: 1.0.0

# [REQUIRED] Description
description: >
  Comprehensive engineering skill covering administration,
  troubleshooting, and best practices for My Technology.

# [REQUIRED] Author
author: Wiki Labs Team

# [REQUIRED] Technology domain
technology_domain: MyTechnology

# [REQUIRED] Vendor
vendor: Wiki Labs

# [REQUIRED] Category
category: Engineering

# [OPTIONAL] YAML list of dependency skill pack IDs
dependencies: []

# [REQUIRED] Whether the skill is enabled by default
enabled: true

# [REQUIRED] Schema version
schema_version: 1.0

# [OPTIONAL] Tags
tags:
  - mytech
  - engineering
  - troubleshooting

# [OPTIONAL] Keywords for search
keywords:
  - admin
  - configuration
  - deployment

# [OPTIONAL] Icon (emoji)
icon: 🔧

# [OPTIONAL] Supported environments
supported_environments:
  - mytech-4.x
  - mytech-5.x

# [OPTIONAL] Documentation URL
documentation_url: https://docs.wikilabs.ai/skills/my-technology-skill-pack
```

**Required fields validation:** The Skill SDK checks that `id`, `name`, and `version` are present and non-empty in `manifest.yaml`. Missing or empty required fields cause validation failure.

### technology.yaml

The technology definition describes the scope and features covered by the skill pack:

```yaml
# technology.yaml

domain: MyTechnology / MyPlatform
version: 1.0.0
description: >
  Enterprise platform for distributed systems management.
  Covers administration, troubleshooting, security, and lifecycle.

# [REQUIRED] Feature list
features:
  - name: Administration
    description: User management, configuration, policy settings
    related_commands:
      - mytech admin configure
      - mytech admin users
  - name: Deployment
    description: Service deployment, scaling, configuration
    related_commands:
      - mytech deploy
      - mytech scale
  - name: Monitoring
    description: Health checks, metrics, alerts, logging
    related_commands:
      - mytech monitor
      - mytech logs
  - name: Troubleshooting
    description: Diagnostic procedures, evidence collection, resolution
    related_commands:
      - mytech diagnostics
      - mytech troubleshoot

# [OPTIONAL] Related technology domains
related_domains:
  - Networking
  - Storage
  - Security

# [OPTIONAL] Documentation URL
documentation_url: https://docs.wikilabs.ai/technology/mytech
```

### workflows.yaml

Troubleshooting workflows define state machines for common problem resolution:

```yaml
# workflows.yaml

- id: service-restart-failed
  name: Service Restart Failure Troubleshooting
  description: >
    Diagnose and resolve services that fail to restart.
  states:
    - id: evidence_collection
      name: Collect Evidence
      description: Gather logs, status, and configuration details.
      initial: true
      terminal: false
      commands:
        - systemctl status <service>
        - journalctl -u <service> --since "1 hour ago"
    - id: diagnosis
      name: Root Cause Diagnosis
      description: Analyze evidence to determine failure cause.
      initial: false
      terminal: false
      commands:
        - journalctl -u <service> -n 50 --no-pager
        - cat /etc/<service>/config.yaml
    - id: remediation
      name: Apply Remediation
      description: Apply fix based on diagnosed root cause.
      initial: false
      terminal: false
      commands:
        - systemctl restart <service>
        - mytech deploy --force
    - id: verification
      name: Verify Resolution
      description: Confirm service is running and stable.
      initial: false
      terminal: true
      commands:
        - systemctl status <service>
        - mytech health

  transitions:
    - from: evidence_collection
      to: diagnosis
      condition: "evidence_collected"
      description: "Move to diagnosis when evidence is gathered"
    - from: diagnosis
      to: remediation
      condition: "root_cause_identified"
      description: "Move to remediation when root cause is determined"
    - from: remediation
      to: verification
      condition: "remediation_applied"
      description: "Move to verification after applying fix"
    - from: verification
      to: evidence_collection
      condition: "still_failed"
      description: "Return to evidence collection if still failing"

  evidence_requirements:
    - "Service status output"
    - "Recent log entries"
    - "Configuration file contents"
    - "Resource usage metrics"

  required: true
```

### detection_rules.yaml

Detection rules define patterns for automatically identifying when a technology is in use:

```yaml
# detection_rules.yaml

- id: mytech-detect-cli
  name: MyTech CLI Detection
  description: Detect MyTech CLI usage in terminal
  detection_type: Command
  pattern: '^mytech \w+'
  confidence: 0.95
  technology_domain: MyTechnology
  priority: 10
  flags: ''
  extract: null

- id: mytech-detect-config
  name: MyTech Config Detection
  description: Detect MyTech configuration files
  detection_type: File
  pattern: '/etc/mytech/'
  confidence: 0.9
  technology_domain: MyTechnology
  priority: 9
  flags: ''
  extract: null

- id: mytech-detect-dashboard
  name: MyTech Dashboard Detection
  description: Detect MyTech web dashboard in browser
  detection_type: Browser
  pattern: 'mytech-dashboard|mytech-admin-portal'
  confidence: 0.85
  technology_domain: MyTechnology
  priority: 8
  flags: ''
  extract: null
```

**Detection types:**

| Type | Description | Example |
|------|-------------|---------|
| `Command` | Terminal command patterns | `^mytech \w+` |
| `File` | File path patterns | `/etc/mytech/` |
| `Browser` | Browser URL patterns | `mytech-dashboard` |

**Pattern fields:**
- `pattern` — Regular expression or substring match
- `confidence` — Confidence score (0.0–1.0)
- `priority` — Priority of this rule (higher = more important)
- `extract` — Optional capture group to extract values from matches

### commands.yaml

Command definitions provide the AI with structured knowledge about CLI commands:

```yaml
# commands.yaml

commands:
  - command: mytech status
    description: "Show current status of all MyTech services"
    category: monitoring
    risk: low
    requires_privilege: false
    rollback: "N/A (read-only command)"
    examples:
      - "mytech status"
      - "mytech status --verbose"
      - "mytech status --format json"

  - command: mytech deploy
    description: "Deploy a new service or update an existing one"
    category: deployment
    risk: high
    requires_privilege: true
    rollback: "mytech deploy --rollback"
    examples:
      - "mytech deploy --app web --version 2.1"
      - "mytech deploy --app api --force"

  - command: mytech admin users
    description: "List and manage user accounts"
    category: administration
    risk: medium
    requires_privilege: true
    rollback: "N/A (user creation can be undone by deletion)"
    examples:
      - "mytech admin users --list"
      - "mytech admin users --add --username alice --role admin"
```

---

## Skill Components

### Knowledge Base (`knowledge/`)

Deep technical documents providing context-aware knowledge:

```
knowledge/
├── cluster-architecture.md     # Architecture overview
├── security.md                 # Security best practices
├── backup-recovery.md          # Backup and disaster recovery
├── configuration-management.md # Configuration procedures
└── performance-optimization.md # Performance tuning
```

Each document should:
- Be well-organized with clear headings
- Include relevant command examples
- Cover the most common scenarios
- Reference official documentation
- Be specific to the technology covered

### Reasoning Guides (`reasoning/`)

Documents that teach the AI how to reason about problems:

```
reasoning/
├── reference.md                # Primary reasoning guide
└── patterns.md                 # Common patterns to look for
```

Reasoning guides include:
- Diagnostic thinking patterns
- How to gather and interpret evidence
- When to recommend specific approaches
- Common mistakes to avoid

### Guidance Rules (`guidance/rules.md`)

Engineering reasoning and guidance rules:

```markdown
# Engineering Guidance Rules

## General Principles
- Always collect evidence before diagnosing
- Consider cascade effects of changes
- Always recommend rollback strategies
- Ask for confirmation before destructive actions

## Command Guidelines
- Prefer non-destructive commands first
- Verify changes after execution
- Document all manual steps taken
```

### Common Failures (`common-failures/`)

Database of known failure patterns:

```markdown
# Common Failure Patterns

## Pattern 1: Configuration Drift
**Symptom:** Service behaving differently than expected
**Cause:** Configuration changed outside of management tooling
**Detection:** Compare current config with deployed config
**Resolution:** Re-apply desired configuration or document drift
```

### Best Practices (`best-practices.md`)

Organization-wide best practices and standards:

```markdown
# Best Practices

## Deployment
- Use canary deployments for major changes
- Always test in staging before production
- Maintain a rollback plan for every deployment

## Security
- Follow least privilege for service accounts
- Rotate credentials on a regular schedule
- Audit access logs weekly
```

### Known Issues (`known_issues.md`)

Documented known failures and workarounds:

```markdown
# Known Issues

## Issue 1: Memory Leak in v3.2.0
**Description:** The metrics collector leaks memory over time
**Workaround:** Restart the collector daily during maintenance window
**Fixed in:** v3.3.0
```

### Quality Standards (`SKILL_PACK_QUALITY_STANDARD.md`)

Quality evaluation criteria for the skill pack:

```markdown
# Skill Pack Quality Standard

This skill pack is evaluated against:

- Knowledge coverage
- Workflow coverage
- Reasoning coverage
- Detection coverage
- Guidance quality
- Safety
- Documentation
- Examples
- Testing
- Maintainability
```

---

## Technology Detection Rules

The `detection_rules.yaml` file defines how the Technology Recognition Engine identifies the technology from observation events.

### Detection Rule Structure

```yaml
- id: unique-rule-id
  name: Human-readable name
  description: What this rule detects and why
  detection_type: Command | File | Browser
  pattern: Regular expression or substring
  confidence: 0.0-1.0
  technology_domain: Domain name
  priority: 1-10 (10 = highest)
  flags: ''
  extract: 'capture_group_pattern'
```

### Detection Rule Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier (lowercase, hyphens) |
| `name` | Yes | Human-readable name |
| `description` | Yes | What the rule detects |
| `detection_type` | Yes | Type of observation: `Command`, `File`, `Browser` |
| `pattern` | Yes | Regex pattern or substring to match |
| `confidence` | Yes | Confidence score (0.0 = uncertain, 1.0 = certain) |
| `technology_domain` | Yes | Technology domain this rule belongs to |
| `priority` | Yes | Priority (1–10, higher = more important) |
| `flags` | No | Additional flags (reserved for future use) |
| `extract` | No | Optional capture group to extract values |

### Detection Types

**Command detection** — Matches terminal shell commands:
```yaml
- id: mytech-detect-status
  detection_type: Command
  pattern: '^mytech status'
  confidence: 0.95
```

**File detection** — Matches file path patterns:
```yaml
- id: mytech-detect-config
  detection_type: File
  pattern: '/etc/mytech/'
  confidence: 0.9
```

**Browser detection** — Matches browser URLs and content:
```yaml
- id: mytech-detect-dashboard
  detection_type: Browser
  pattern: 'mytech-dashboard|mytech-admin'
  confidence: 0.85
```

### Confidence and Priority

- **Confidence** (0.0–1.0) — How certain the detection is. Higher confidence detections are weighted more heavily.
- **Priority** (1–10) — Relative importance. Higher priority rules are evaluated first.

The RecognitionEngine combines multiple detection rules using multi-pass evidence aggregation to produce a final technology inference with composite confidence.

---

## Workflows

Workflows are declarative state machines that guide troubleshooting from evidence collection through resolution.

### Workflow Structure

A workflow is a YAML sequence item with the following structure:

```yaml
- id: workflow-id
  name: Human-readable name
  description: What this workflow solves
  states:
    - id: state-id
      name: State name
      description: What to do in this state
      initial: true       # First state (only one)
      terminal: false     # End state (at least one)
      commands:           # Commands to run in this state
        - cmd1
        - cmd2
  transitions:
    - from: state-a
      to: state-b
      condition: condition-name
      description: When to transition
  evidence_requirements:
    - "What evidence is needed"
    - "What to look for"
  required: true          # Mark workflow as required
```

### Workflow States

Each state has:
- **id** — Unique identifier
- **name** — Human-readable name
- **description** — What to do in this state
- **initial** — True for the first state (exactly one)
- **terminal** — True for end states (at least one)
- **commands** — List of commands to execute in this state

### Workflow Transitions

Transitions define how to move between states:
- **from** — Source state id
- **to** — Target state id
- **condition** — Condition for the transition
- **description** — Human-readable transition description

### Workflow Evidence Requirements

List of evidence items that should be collected during the workflow. The RecommendationReadiness engine checks these to determine if a workflow has sufficient evidence before recommending actions.

### Pre-installed Workflow Examples

From the OpenShift skill pack:
- Pod CrashLoopBackOff Troubleshooting
- Pod Pending Troubleshooting
- Service Unavailability Troubleshooting
- Network Connectivity Troubleshooting
- Storage Issues Troubleshooting
- Cluster Upgrade Troubleshooting
- Node Failure Troubleshooting
- Operator Failure Troubleshooting

---

## Commands

The commands.yaml file provides structured knowledge about CLI commands for the AI to reference during recommendations.

### Command Structure

```yaml
commands:
  - command: full.command.name
    description: Clear description of what the command does
    category: administration | deployment | monitoring | troubleshooting | security
    risk: low | medium | high
    requires_privilege: true | false
    rollback: Description of how to undo the command
    examples:
      - "command --flag value"
    notes:
      - "Important notes about this command"
```

### Risk Levels

| Level | Description | Guidance |
|-------|-------------|----------|
| **Low** | Read-only, no system impact | Safe to recommend without special warning |
| **Medium** | Configuration changes, may affect running services | Recommend with explanation |
| **High** | Destructive operations, service restarts, data changes | Recommend with rollback strategy and confirmation |

---

## Knowledge Base

The `knowledge/` directory contains deep technical documents that provide context-aware information to the AI.

### Knowledge Document Structure

Each knowledge document should:
1. Start with a clear title and overview
2. Cover the most common scenarios first
3. Include command examples with explanations
4. Reference official documentation where applicable
5. Cover edge cases and common mistakes
6. Include diagrams or visual references where helpful

### Recommended Knowledge Categories

| Category | Content |
|----------|---------|
| `architecture/` | System architecture, components, data flow |
| `concepts/` | Terminology, models, design patterns |
| `diagnostics/` | Diagnostic procedures, evidence collection |
| `examples/` | Worked examples and tutorials |
| `guidance/` | Engineering guidance rules and practices |
| `knowledge/` | Deep technical reference documentation |
| `overview/` | High-level overview and getting started |
| `reasoning/` | Reasoning guides and diagnostic patterns |
| `references/` | Documentation references and links |
| `testing/` | Test procedures and validation |

---

## Skill SDK Usage

The Skill SDK (`src/skill_sdk/`) provides tools for creating, validating, and generating skill packs.

### SDK Components

The `SkillSDK` struct provides:

```rust
pub struct SkillSDK {
    templates_dir: PathBuf,   // Directory containing template files
    schemas_dir: PathBuf,     // Directory containing JSON schema files
    schema_registry: SchemaRegistry, // Registry of JSON schemas
}
```

### Template Generation

Generate a new skill pack template:

```rust
let sdk = SkillSDK::new("/path/to/sdk-dir")?;
let template = sdk.create_skill_template("MyTechnology")?;

// template contains:
// - skill_name: "my-technology"
// - directory_structure: ["my-technology", "templates/", "schemas/"]
// - generated_files: [manifest.yaml, technology.yaml, intents.yaml,
//   workflows.yaml, detection_rules.yaml, commands.yaml,
//   best_practices.yaml, known_issues.yaml]
```

The SDK generates the following files:
| File | Schema | Purpose |
|------|--------|---------|
| `manifest.yaml` | `manifest` | Skill metadata and configuration |
| `technology.yaml` | `technology` | Technology coverage and features |
| `intents.yaml` | `intents` | Intent definitions |
| `workflows.yaml` | `workflows` | Troubleshooting workflows |
| `detection_rules.yaml` | `detection_rules` | Context detection patterns |
| `commands.yaml` | `commands` | Command knowledge base |
| `best_practices.yaml` | `best_practices` | Best practices |
| `known_issues.yaml` | `known_issues` | Known issues and workarounds |

### Schema Registry

The SDK includes a schema registry for JSON schema validation:

```rust
// Get all registered schema names
let schemas = sdk.get_all_schema_names();
// Returns: ["best_practices", "commands", "detection_rules", "intents",
//            "known_issues", "manifest", "technology", "workflows"]

// Get a specific schema
let manifest_schema = sdk.get_schema("manifest");
```

### Skill Validation

Validate a skill pack:

```rust
let report = sdk.validate_skill("/path/to/my-technology-skill-pack")?;

if report.is_valid {
    println!("Skill pack is valid!");
} else {
    for error in &report.errors {
        eprintln!("ERROR: {}", error);
    }
    for warning in &report.warnings {
        eprintln!("WARNING: {}", warning);
    }
}

// report.check_schemas: List of schema names that were checked
```

The validation checks:
1. `manifest.yaml` exists and has required fields (`id`, `name`, `version`)
2. `technology.yaml` (if present) is valid YAML
3. `intents.yaml` (if present) is valid YAML with `id` and `patterns` fields
4. `detection_rules.yaml` (if present) is valid YAML
5. `workflows.yaml` (if present) is valid YAML
6. `commands.yaml` (if present) is valid YAML

---

## Testing a Skill Pack

### Step 1: Template Generation

Use the Skill SDK to generate a template:

```rust
let sdk = SkillSDK::new("/path/to/sdk")?;
let template = sdk.create_skill_template("MyTech")?;
// Write template files to a new directory
```

### Step 2: Validate

Run validation before distributing:

```bash
# If the application has a CLI validation command
# wikilabs validate-skill /path/to/skill-pack

# Or via the SDK programmatically
let report = sdk.validate_skill("/path/to/skill-pack")?;
assert!(report.is_valid, "Validation errors: {:?}", report.errors);
```

### Step 3: Manual Review

Check each component:

| Component | Review Checklist |
|-----------|-----------------|
| `manifest.yaml` | All required fields present? ID unique? Version correct? |
| `technology.yaml` | Features comprehensive? Related commands correct? |
| `detection_rules.yaml` | Patterns match real usage? Confidence reasonable? Priority correct? |
| `workflows.yaml` | States complete? Transitions logical? Evidence requirements sufficient? |
| `commands.yaml` | Commands documented? Risk levels accurate? Rollbacks specified? |
| `knowledge/` | Documents accurate? Examples relevant? Up-to-date? |
| `reasoning/` | Guides actionable? Patterns complete? Common mistakes covered? |
| `guidance/rules.md` | Rules enforceable? Safety-first approach? |
| `best-practices.md` | Practices actionable? Industry-aligned? |
| `known_issues.md` | Issues documented? Workarounds provided? Fixed versions noted? |

### Step 4: Integration Test

Copy the skill pack to the target directory and restart the application:

```powershell
# Copy skill pack to the application's skills directory
Copy-Item ".\mytech-skill-pack" "$env:PROJECT\src\skills\" -Recurse

# Restart the application
# The skill will be discovered and loaded on startup
```

### Step 5: Verification

After restart:
1. Open Settings → Skills — verify the skill appears in the list
2. Start using the AI chat — verify the skill provides relevant guidance
3. Test observation features — verify the skill activates when the technology is detected
4. Run the diagnostic package — verify no errors in the diagnostics report

---

## Distribution

### Installing a Skill Pack

1. **Stop the application** — Close all running instances
2. **Copy the skill pack** — Copy the directory to `src/skills/`
3. **Start the application** — Skills are loaded on startup only

```powershell
# Example: Install a new skill pack
Copy-Item ".\mytech-skill-pack" ".\src\skills\" -Recurse
# Then restart Wiki Labs AI Copilot
```

### Removing a Skill Pack

1. **Stop the application**
2. **Delete the skill pack directory** from `src/skills/`
3. **Start the application**

### Updating a Skill Pack

1. **Stop the application**
2. **Replace the files** in the skill pack directory (keep the directory name)
3. **Start the application**

### Distribution Formats

Skill packs can be distributed as:
- **Directory copy** — Direct copy to `src/skills/`
- **ZIP archive** — Extract to `src/skills/`
- **Installer script** — Automated copy to `src/skills/`

### Distribution Checklist

| Item | Status |
|------|--------|
| Skill pack validated via Skill SDK | ☐ |
| All YAML files parse correctly | ☐ |
| manifest.yaml has all required fields | ☐ |
| Detection rules match real usage | ☐ |
| Workflows cover major scenarios | ☐ |
| Knowledge base is accurate and complete | ☐ |
| Commands documented with risk levels | ☐ |
| Documentation URL configured in manifest | ☐ |
| Version number reflects changes | ☐ |
| Integration test passed | ☐ |

---

## Quality Standards

The OpenShift skill pack serves as the reference standard. All skill packs should meet these criteria:

| Criterion | Weight | Evaluation |
|-----------|--------|------------|
| Knowledge coverage | High | Comprehensive coverage of the technology |
| Workflow coverage | High | Troubleshooting workflows for major scenarios |
| Reasoning coverage | Medium | Clear reasoning guides for diagnosis |
| Detection coverage | High | Patterns for all major interaction modes |
| Guidance quality | Medium | Clear, actionable engineering guidance |
| Safety | High | Strict safety rules enforced |
| Documentation | Medium | Clear, well-organized documentation |
| Examples | Medium | Worked examples for common scenarios |
| Testing | Low | Validation tests |
| Maintainability | Medium | Clear structure, easy to update |

---

## Best Practices

### Naming Conventions

- **Directory name:** lowercase, hyphens (e.g., `my-technology-skill-pack`)
- **Manifest ID:** matches directory name
- **YAML keys:** snake_case
- **State IDs:** lowercase, hyphens
- **Rule IDs:** `{domain}-detect-{feature}` pattern

### Detection Rule Best Practices

- Use **specific patterns** that match real usage (not generic)
- Set **confidence scores** conservatively (0.85–0.95 for common patterns)
- Assign **priorities** based on how uniquely they identify the technology
- Use **capture groups** (`extract`) to extract relevant values
- Cover all **observation types** (Command, File, Browser)

### Workflow Best Practices

- Start with **evidence collection** before any diagnosis
- Include **at least one terminal state** for success
- Include **back-tracking transitions** for incomplete evidence
- Require **multiple evidence items** before recommending actions
- Document **evidence requirements** for each workflow

### Documentation Best Practices

- Use **clear headings** and subheadings
- Include **command examples** with explanations
- Reference **official documentation** where applicable
- Cover **edge cases** and common mistakes
- Keep documents **focused** on one topic

### Safety Best Practices

- **Never execute commands** — only recommend and explain
- **Always warn about risks** before recommending actions
- **Always provide rollback strategies** for destructive commands
- **Always recommend evidence collection** before diagnosis
- **Always consider cascade effects** of recommended actions

---

## Example: Creating a New Skill Pack

This example walks through creating a skill pack for "MyTech" — a fictional container orchestration platform.

### Step 1: Create Directory

```
mytech-skill-pack/
├── manifest.yaml
├── technology.yaml
├── detection_rules.yaml
├── workflows.yaml
├── commands.yaml
├── best_practices.md
├── known_issues.md
├── knowledge/
│   └── cluster-architecture.md
├── reasoning/
│   └── reference.md
└── diagnostics/
    └── guide.md
```

### Step 2: Write manifest.yaml

```yaml
id: mytech-skill-pack
name: MyTech Engineering
version: 1.0.0
description: >
  Comprehensive engineering skill for MyTech container
  orchestration platform. Covers deployment, scaling,
  troubleshooting, and best practices.
author: Wiki Labs Team
technology_domain: MyTech
vendor: Wiki Labs
category: Engineering
dependencies: []
enabled: true
schema_version: 1.0
tags:
  - mytech
  - containers
  - orchestration
icon: 🔷
supported_environments:
  - mytech-3.x
  - mytech-4.x
documentation_url: https://docs.wikilabs.ai/skills/mytech-skill-pack
```

### Step 3: Write technology.yaml

```yaml
domain: MyTech
version: 1.0.0
description: >
  MyTech container orchestration platform.
  Covers deployment, scaling, networking, and management.
features:
  - name: Deployment
    description: Service deployment, rolling updates, rollbacks
    related_commands:
      - mytech deploy
      - mytech rollback
  - name: Scaling
    description: Auto-scaling, manual scaling, resource limits
    related_commands:
      - mytech scale
      - mytech resources
  - name: Networking
    description: Services, routes, ingress, network policies
    related_commands:
      - mytech services
      - mytech routes
  - name: Troubleshooting
    description: Diagnostic procedures, evidence collection
    related_commands:
      - mytech diagnostics
      - mytech logs
related_domains:
  - Containers
  - Kubernetes
  - DevOps
```

### Step 4: Write detection_rules.yaml

```yaml
- id: mytech-detect-cli
  name: MyTech CLI Detection
  description: Detect MyTech CLI usage in terminal
  detection_type: Command
  pattern: '^mytech \w+'
  confidence: 0.95
  technology_domain: MyTech
  priority: 10
  flags: ''
  extract: null

- id: mytech-detect-config
  name: MyTech Config Detection
  description: Detect MyTech configuration files
  detection_type: File
  pattern: '/etc/mytech/'
  confidence: 0.9
  technology_domain: MyTech
  priority: 9
  flags: ''
  extract: null
```

### Step 5: Write workflows.yaml

```yaml
- id: service-deployment-failure
  name: Service Deployment Failure
  description: Diagnose and resolve service deployment failures.
  states:
    - id: evidence_collection
      name: Collect Evidence
      description: Gather deployment status and logs.
      initial: true
      terminal: false
      commands:
        - mytech deploy status
        - mytech logs --last 100
    - id: diagnosis
      name: Diagnosis
      description: Analyze evidence to determine root cause.
      initial: false
      terminal: false
      commands:
        - mytech deploy events
        - mytech describe service <name>
    - id: remediation
      name: Apply Fix
      description: Apply the appropriate fix.
      initial: false
      terminal: false
      commands:
        - mytech deploy --rollback
        - mytech deploy --force
    - id: verification
      name: Verify
      description: Confirm the service is running.
      initial: false
      terminal: true
      commands:
        - mytech deploy status
        - mytech health

  transitions:
    - from: evidence_collection
      to: diagnosis
      condition: "evidence_collected"
      description: "Move to diagnosis when evidence is gathered"
    - from: diagnosis
      to: remediation
      condition: "root_cause_identified"
      description: "Move to remediation when root cause is determined"
    - from: remediation
      to: verification
      condition: "fix_applied"
      description: "Move to verification after applying fix"
    - from: verification
      to: evidence_collection
      condition: "still_failed"
      description: "Return to evidence collection if still failing"

  evidence_requirements:
    - "Deployment status"
    - "Service logs"
    - "Recent events"
    - "Resource allocation"

  required: true
```

### Step 6: Write commands.yaml

```yaml
commands:
  - command: mytech deploy
    description: "Deploy a new service or update an existing one"
    category: deployment
    risk: high
    requires_privilege: true
    rollback: "mytech deploy --rollback"
    examples:
      - "mytech deploy --app web --version 2.1"
      - "mytech deploy --force"
```

### Step 7: Validate

```rust
let sdk = SkillSDK::new("/path/to/sdk")?;
let report = sdk.validate_skill("mytech-skill-pack")?;
if report.is_valid {
    println!("✓ Skill pack validated successfully");
} else {
    println!("✗ Validation failed:");
    for e in &report.errors { println!("  - {}", e); }
}
```

### Step 8: Install

```powershell
Copy-Item ".\mytech-skill-pack" ".\src\skills\" -Recurse
# Restart Wiki Labs AI Copilot
```

---

*For more information on the technology recognition engine, see the [Architecture Guide](ARCHITECTURE_GUIDE.md). For troubleshooting skill issues, see the [Troubleshooting Guide](TROUBLESHOOTING.md).*