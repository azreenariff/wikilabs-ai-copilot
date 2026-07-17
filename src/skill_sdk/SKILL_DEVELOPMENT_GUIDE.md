# Skill SDK Development Guide

Welcome to the Wiki Labs AI Copilot Skill SDK — your toolkit for building domain-specific AI expertise modules.

## Table of Contents

1. [What is a Skill?](#what-is-a-skill)
2. [Skill Directory Structure](#skill-directory-structure)
3. [Creating a New Skill](#creating-a-new-skill)
4. [Component Reference](#component-reference)
5. [Validating Your Skill](#validating-your-skill)
6. [Testing Your Skill](#testing-your-skill)
7. [Deploying Your Skill](#deploying-your-skill)

---

## What is a Skill?

A Skill is a declarative, self-contained unit of expertise that teaches the AI Copilot about a specific technology domain (e.g., OpenShift, Kubernetes, Linux, MySQL). Skills are loaded at runtime by the Skill Runtime and provide:

- **Intent patterns** — Tell the AI what the engineer might want to do
- **Detection rules** — Teach the AI how to identify the technology in the environment
- **Workflows** — Define step-by-step procedures for common tasks
- **Commands** — Define safe CLI commands the AI can suggest
- **Best practices** — Ground rules for using the skill effectively

### Core Principles

- **Declarative**: Skills are defined in YAML files — no code required
- **Self-contained**: Each skill is a directory with its own manifest and components
- **Composable**: Skills can declare dependencies on other skills
- **Validatable**: SDK tools validate skills before they are loaded
- **Domain-scoped**: Each skill targets a single technology domain

---

## Skill Directory Structure

Every skill lives in a directory with this structure:

```
my-skill/
├── manifest.yaml              # Required — skill metadata
├── technology.yaml            # Recommended — technology definition
├── intents.yaml               # Optional — intent recognition patterns
├── workflows.yaml             # Optional — workflow definitions
├── detection_rules.yaml       # Optional — artifact detection rules
├── commands.yaml              # Optional — CLI command definitions
├── best_practices.yaml        # Optional — usage guidelines
└── known_issues.yaml          # Optional — documented limitations
```

### Minimal Viable Skill

A skill with only `manifest.yaml` is loadable, though not very useful. The recommended minimum is:

```
my-skill/
├── manifest.yaml
├── technology.yaml
├── detection_rules.yaml
└── intents.yaml
```

---

## Creating a New Skill

### Option 1: Use the SDK Template Generator

```rust
use wikilabs_skill_sdk::SkillSDK;

let sdk = SkillSDK::new("/path/to/skill_sdk")?;
let template = sdk.create_skill_template("my-technology")?;

// Write each generated file to disk
for file in template.generated_files {
    let path = format!("/path/to/skills/my-technology/{}", file.path);
    std::fs::write(&path, &file.content)?;
}
```

### Option 2: Copy the Template Files

1. Copy from `src/skill_sdk/templates/` into your new skill directory
2. Replace all `{{SKILL_ID}}`, `{{SKILL_NAME}}`, `{{SKILL_DOMAIN}}`, and `{{VERSION}}` placeholders
3. Customize patterns and workflows to your domain

### Option 3: Write from Scratch

Use the schema files in `src/skill_sdk/schemas/` as a reference for valid YAML structure.

---

## Component Reference

### manifest.yaml

The skill's identity. Required fields: `id`, `name`, `version`, `description`, `author`, `technology_domain`, `schema_version`, `enabled`.

```yaml
id: my-technology
name: "My Technology Skill"
version: "0.1.0"
description: "Teaches the AI about My Technology"
author: "Wiki Labs Team"
technology_domain: "my-technology"
dependencies: []
enabled: true
schema_version: "1.0"
keywords: [deploy, config, troubleshoot]
tags: [cloud, infrastructure]
```

### technology.yaml

Defines the technology being covered.

```yaml
domain: "my-technology"
version: "4.2"
description: "My Technology is a container orchestration platform"
features:
  - "Container orchestration"
  - "Auto-scaling"
  - "Service mesh"
related_domains: ["kubernetes", "linux"]
documentation_url: "https://docs.mytech.io"
```

### intents.yaml

Pattern-based intent recognition. Each intent defines a regex pattern that matches user input.

```yaml
- id: my-tech-deploy
  name: "Deploy"
  description: "User wants to deploy My Technology"
  patterns:
    - "(?i)(deploy|release|rollout).*my-technology"
  confidence_boost: 0.85
  required_domain: "my-technology"
  priority: 10
```

**Key fields:**
- `id`: Unique intent identifier
- `patterns`: Array of regex patterns (use `(?i)` for case-insensitive)
- `confidence_boost`: How much to boost confidence when matched (0.0–1.0)
- `required_domain`: Must match the skill's technology domain
- `priority`: Higher values are evaluated first

### workflows.yaml

Step-by-step procedures with state transitions.

```yaml
- id: my-tech-troubleshoot
  name: "Troubleshoot My Tech"
  description: "Diagnose and fix My Technology issues"
  states:
    - id: discovery
      name: "Discovery"
      description: "Gather symptoms and context"
      initial: true
      terminal: false
    - id: analysis
      name: "Analysis"
      description: "Analyze root causes"
      initial: false
      terminal: false
    - id: resolution
      name: "Resolution"
      description: "Apply fix"
      initial: false
      terminal: true
  transitions:
    - from: discovery
      to: analysis
      condition: "sufficient_data"
      description: "Move to analysis when enough data collected"
  evidence_requirements:
    - "Error logs"
    - "Configuration files"
  required: true
```

### detection_rules.yaml

Teach the AI what files, commands, and patterns indicate the technology.

```yaml
- id: my-tech-config-file
  name: "My Tech config file"
  detection_type: File
  pattern: "/etc/my-tech/"
  confidence: 0.85
  technology_domain: "my-tech"
  priority: 10
```

**Detection types:** `File`, `Command`, `Pattern`, `Argument`, `Environment`

### commands.yaml

Safe CLI commands the AI can suggest.

```yaml
- id: my-tech-status
  name: "status"
  command: "my-tech status"
  description: "Show current My Technology status"
  requires_elevation: false
  safety_level: ReadOnly
  technology_domain: "my-tech"
  read_only: true
  allowed_args: ["--json", "--verbose"]
```

**Safety levels:** `ReadOnly` (never modifies state), `Safe` (may modify but generally safe), `Destructive` (may cause data loss)

### best_practices.yaml

Guidelines for using the skill effectively.

```yaml
- Verify all detections before acting
- Ask for confirmation before destructive actions
- Document any manual steps taken
- Cross-reference multiple evidence sources
```

### known_issues.yaml

Documented limitations.

```yaml
- Detection confidence may be low for custom configurations
- Pattern matching may miss non-standard naming conventions
- Always verify critical detections with manual inspection
```

---

## Validating Your Skill

Use the Skill SDK to validate before deploying:

```rust
use wikilabs_skill_sdk::SkillSDK;

let sdk = SkillSDK::new("/path/to/skill_sdk")?;
let report = sdk.validate_skill("/path/to/my-skill")?;

if report.is_valid {
    println!("Skill '{}' is valid!", report);
} else {
    for error in &report.errors {
        eprintln!("Error: {}", error);
    }
    for warning in &report.warnings {
        eprintln!("Warning: {}", warning);
    }
}
```

### What Validation Checks

1. **manifest.yaml exists** and contains required fields (`id`, `name`, `version`)
2. **YAML syntax** is valid for all component files
3. **Schema compliance** — required fields present, values within bounds
4. **Intent pattern compilation** — regex patterns compile successfully
5. **Detection rule patterns** — regex patterns compile successfully

---

## Testing Your Skill

### Unit Testing with the Runtime

```rust
use wikilabs_skill_runtime::SkillRuntime;

let mut runtime = SkillRuntime::new("/path/to/skills");
runtime.discover_skills()?;
let skill = runtime.load_skill("my-tech")?;

// Verify loaded data
assert_eq!(skill.manifest.id, "my-tech");
assert!(!skill.detection_rules.is_empty());
assert!(!skill.intents.is_empty());
```

### Integration Testing

1. **Discovery test**: Verify the skill appears in `discover_skills()`
2. **Load test**: Verify all YAML files parse correctly
3. **Intent test**: Feed test strings to the intent engine and verify matches
4. **Detection test**: Verify detection rules match expected artifacts
5. **Dependency test**: Verify dependency resolution works correctly

---

## Deploying Your Skill

### 1. Validate

```bash
# Use SDK validation
python scripts/validate_skill.py /path/to/my-skill
```

### 2. Load into Runtime

```rust
let mut runtime = SkillRuntime::new("/opt/wikilabs/skills");
runtime.discover_and_load_all()?;

// Or load individually
runtime.load_skill("my-tech")?;
runtime.enable_skill("my-tech")?;
```

### 3. Verify

```rust
let skill = runtime.get_skill("my-tech").unwrap();
assert!(!skill.validation_errors.is_empty()); // Should be empty if valid

// Check intents
let intents = runtime.get_all_intents();
assert!(intents.iter().any(|i| i.required_domain == "my-tech"));
```

---

## Tips and Best Practices

### Pattern Writing

- Use `(?i)` prefix for case-insensitive matching
- Keep patterns specific enough to avoid false positives
- Test patterns against common real-world input before deploying
- Order patterns by specificity — most specific first

### Detection Rule Design

- Use file paths that are unique to your technology
- Prefer exact paths over broad patterns
- Set confidence based on how uniquely the artifact identifies the technology
- Use `extract` to capture version numbers or other data

### Workflow Design

- Start with 3 states: Discovery → Analysis → Resolution
- Keep state transitions simple — avoid complex branching
- Require minimal evidence — the AI can always ask for more
- Mark workflows as `required: false` for optional procedures

### Versioning

- Follow semantic versioning: `MAJOR.MINOR.PATCH`
- `PATCH`: Bug fixes, minor improvements
- `MINOR`: New features (new intents, rules, commands)
- `MAJOR`: Breaking changes (schema changes, removed patterns)

---

## Troubleshooting

### Common Validation Errors

| Error | Cause | Fix |
|-------|-------|-----|
| "Missing required file: manifest.yaml" | No manifest.yaml in skill dir | Create manifest.yaml |
| "'id' field is empty" | Empty or missing id in manifest | Set a non-empty id |
| "YAML parse error" | Invalid YAML syntax | Validate YAML with a linter |
| "Invalid regex" | Pattern doesn't compile as regex | Fix the regex syntax |
| "Schema version mismatch" | Schema version not 1.0 | Update to "1.0" or update runtime |

### Debugging Intent Patterns

```rust
// Test individual patterns
let test_strings = vec![
    "deploy my-app to production",
    "fix this error",
    "configure the settings",
];

for intent in &skill.intents {
    for s in &test_strings {
        if intent.patterns.iter().any(|p| Regex::new(p).unwrap().is_match(s)) {
            println!("Intent '{}' matched: {}", intent.name, s);
        }
    }
}
```

### Debugging Detection Rules

```rust
// Test detection against file paths
let test_paths = vec![
    "/etc/my-tech/config.yaml",
    "/usr/local/bin/my-tech",
    "var/log/my-tech/error.log",
];

for rule in &skill.detection_rules {
    if let Ok(re) = regex::Regex::new(&rule.pattern) {
        for path in &test_paths {
            if re.is_match(path) {
                println!("Rule '{}' matches: {}", rule.name, path);
            }
        }
    }
}
```

---

## Example: Creating a Complete Skill

Let's create a skill for "Rust" technology:

```
rust-skill/
├── manifest.yaml
├── technology.yaml
├── intents.yaml
├── workflows.yaml
├── detection_rules.yaml
├── commands.yaml
├── best_practices.yaml
└── known_issues.yaml
```

**manifest.yaml:**
```yaml
id: rust-skill
name: "Rust Programming Language"
version: "0.1.0"
description: "Expertise for Rust development, compilation, and debugging"
author: "Wiki Labs Team"
technology_domain: "rust"
dependencies: []
enabled: true
schema_version: "1.0"
keywords: [rust, cargo, compile, debug]
tags: [programming, systems]
```

**detection_rules.yaml:**
```yaml
- id: rust-cargo-toml
  name: "Cargo.toml"
  detection_type: File
  pattern: "Cargo\\.toml$"
  confidence: 0.95
  technology_domain: "rust"
  priority: 20
- id: rust-cargo-lock
  name: "Cargo.lock"
  detection_type: File
  pattern: "Cargo\\.lock$"
  confidence: 0.90
  technology_domain: "rust"
  priority: 15
- id: rust-rustc
  name: "rustc command"
  detection_type: Command
  pattern: "^rustc "
  confidence: 0.95
  technology_domain: "rust"
  priority: 20
```

---

## Further Reading

- [Skill Architecture](SKILL_ARCHITECTURE.md) — How skills fit into the system
- [Skill Schema Reference](SKILL_SCHEMA_REFERENCE.md) — Complete schema documentation
- [Intent Engine](INTENT_ENGINE.md) — How intent recognition works
- [Technology Recognition](TECHNOLOGY_RECOGNITION.md) — How technology detection works