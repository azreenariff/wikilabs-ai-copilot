# Skill Pack Development Guide — Wiki Labs AI Copilot v1.0.0

> How to create, validate, package, and distribute skill packs.

## Table of Contents

1. [What Is a Skill Pack?](#what-is-a-skill-pack)
2. [Skill Pack Anatomy](#skill-pack-anatomy)
3. [Development Workflow](#development-workflow)
4. [Manifest Format](#manifest-format)
5. [Technology Definitions](#technology-definitions)
6. [Detection Rules](#detection-rules)
7. [Workflow Definitions](#workflow-definitions)
8. [Command References](#command-references)
9. [Guidance Rules](#guidance-rules)
10. [Knowledge Base](#knowledge-base)
11. [Packaging and Distribution](#packaging-and-distribution)
12. [Validation](#validation)
13. [Testing](#testing)
14. [Update and Versioning](#update-and-versioning)
15. [SDK Commands](#sdk-commands)
16. [Examples](#examples)
17. [Best Practices](#best-practices)

## What Is a Skill Pack?

A skill pack is a collection of technology-specific expertise that enables Wiki Labs AI Copilot to recognize, understand, and provide guidance for a specific technology. Skill packs are the primary extension mechanism for the application.

### What Skill Packs Provide

- **Detection rules** — Identify when a technology is in use
- **Best practices** — Engineering best practices for the technology
- **Common failures** — Known issues, detection rules, and remediation
- **Command references** — Technical commands and usage patterns
- **Knowledge base** — Architecture documentation and troubleshooting
- **Reasoning guides** — Diagnostic reasoning patterns
- **Workflows** — State machine workflows for common tasks
- **Quality standard documentation** — Quality requirements for the skill

### Available Skill Packs (v1.0.0)

| Skill Pack | Files | Subdirectories | Technology |
|------------|-------|---------------|------------|
| OpenShift | 40 | 13 | Red Hat OpenShift 4.x |
| Linux Engineering | 40 | 13 | Linux Administration |
| VMware vSphere | 40 | 13 | VMware vSphere |
| Nagios XI | 19 | 13 | Nagios XI Monitoring |
| Nagios Log Server | 20 | 13 | Nagios Log Server |
| Checkmk | 21 | 13 | Checkmk Monitoring |
| Ansible | 20 | 13 | Ansible Automation |
| MySQL | 41 | 14 | MySQL 8.0 |
| EDB PostgreSQL | 34 | 14 | EDB PostgreSQL 15/16 |
| Microsoft SQL Server | 28 | 13 | SQL Server 2022 |

## Skill Pack Anatomy

### Directory Structure

```
my-skill-pack/
├── manifest.yaml                    # Skill pack metadata (REQUIRED)
├── technology.yaml                  # Technology definitions (REQUIRED)
├── detection_rules.yaml             # Detection rules (REQUIRED)
├── workflows.yaml                   # State machine workflows (REQUIRED)
├── commands.yaml                    # Technical commands (REQUIRED)
├── best-practices.md                # Best practices guide
├── known-issues.md                  # Known issues and workarounds
├── README.md                        # Skill pack overview
│
├── concepts/
│   ├── overview.md                  # Architecture overview
│   └── terminology.md              # Glossary of terms
│
├── detection/
│   └── reference.md                 # Detection rules documentation
│
├── diagnostics/
│   └── guide.md                     # Diagnostic procedures
│
├── common-failures/
│   └── reference.md                 # Known failure modes
│
├── examples/
│   └── worked-examples.md           # Real-world scenarios
│
├── knowledge/
│   ├── architecture.md              # Detailed architecture docs
│   └── best-practices.md            # Engineering best practices
│
├── references/
│   └── reference.md                 # External references
│
├── reasoning/
│   └── reference.md                 # Diagnostic reasoning
│
├── tests/
│   └── reference.md                 # Validation tests
│
└── context/
    └── interpretation.md            # Context interpretation
```

### Minimum Viable Skill Pack

A minimal skill pack requires only:

```
my-skill-pack/
├── manifest.yaml
├── technology.yaml
├── detection_rules.yaml
├── workflows.yaml
├── commands.yaml
└── README.md
```

## Development Workflow

### Step-by-Step Process

1. **Research and gather expertise**
   - Collect documentation, SOPs, and best practices
   - Identify key commands and troubleshooting patterns
   - Document common failure modes and remediation

2. **Create the skill pack directory**
   ```bash
   mkdir -p src/skills/my-skill-pack/{concepts,detection,diagnostics,common-failures,examples,knowledge,references,reasoning,tests,context}
   ```

3. **Create the manifest**
   - Define ID, name, version, description
   - List supported technologies and versions

4. **Define technologies**
   - Specify platforms, components, and versions
   - Document technology relationships

5. **Write detection rules**
   - Browser URL patterns
   - Terminal command patterns
   - Window title patterns
   - Configuration file patterns

6. **Define workflows**
   - Create state machine workflows for common tasks
   - Define transitions and conditions

7. **Add command references**
   - Document key technical commands
   - Include usage examples and flags

8. **Write guidance rules**
   - Engineering guidance based on the technology
   - Best practices and troubleshooting steps

9. **Build knowledge base**
   - Architecture documentation
   - Best practices guide
   - Common failures and detection rules

10. **Validate and test**
    ```bash
    cargo run --bin knowledge-cli validate src/skills/my-skill-pack
    ```

11. **Package for distribution**
    ```bash
    cargo run --bin knowledge-cli package src/skills/my-skill-pack
    ```

## Manifest Format

The manifest is the core metadata file for a skill pack.

```yaml
# manifest.yaml

id: "my-skill-pack"                    # Unique identifier
version: "1.0.0"                        # Semantic version
name: "My Technology"                  # Display name
description: "Expert knowledge for My Technology administration"
author: "Wiki Labs Engineering"         # Skill pack author
license: "Proprietary"                  # License type
tags:                                   # Search tags
  - "infrastructure"
  - "administration"
  - "troubleshooting"

technologies:                           # Technologies this skill covers
  - name: "my-technology"
    platform: "Linux"
    version: "1.0"
    components:
      - "service-a"
      - "service-b"

supported_versions:                     # Version awareness
  minimum: "2024"                       # Minimum supported version
  tested: "2025"                        # Latest tested version
  current: "1.0"                        # Current major version

deprecated_parameters:                  # Deprecated parameters with alternatives
  - parameter: "old_setting"
    since_version: "2.1"
    alternative: "new_setting"
    action: "Update configuration"

dependencies:                           # Other skill packs this depends on
  - "shared-operations"

file_count: 40                          # Total files in skill pack
directory_count: 13                     # Total subdirectories
```

## Technology Definitions

Technology definitions describe the platforms, components, and versions a skill pack supports.

```yaml
# technology.yaml

technologies:
  - name: "my-technology"
    platform: "Linux"
    version: "1.0"
    description: "My Technology enterprise platform"
    categories:
      - "infrastructure"
      - "virtualization"
    components:
      - name: "service-a"
        description: "Core service A"
        default_port: 8080
      - name: "service-b"
        description: "Core service B"
        default_port: 8443
    configuration_files:
      - "/etc/my-technology/main.conf"
      - "/etc/my-technology/service-a.conf"
    log_files:
      - "/var/log/my-technology/service-a.log"
      - "/var/log/my-technology/service-b.log"
    data_directories:
      - "/var/lib/my-technology/data"
      - "/var/lib/my-technology/cluster"
```

## Detection Rules

Detection rules enable the Skill Discovery Engine to identify when a technology is in use.

### Rule Types

| Type | Source | Detection Method |
|------|--------|-----------------|
| `browser-url` | Browser | URL pattern matching |
| `terminal-command` | Terminal | Command name detection |
| `window-title` | Desktop | Window title pattern matching |
| `config-file` | File system | Configuration file path matching |
| `process-name` | Process list | Running process detection |
| `log-pattern` | Log files | Pattern matching in logs |

### Detection Rule Format

```yaml
# detection_rules.yaml

detection_rules:
  - name: "browser-url-detection"
    type: "browser-url"
    confidence: 0.90
    pattern: "my-portal.example.com"
    description: "Detected via browser URL"
    action: "Activate skill pack for My Technology"

  - name: "terminal-command-detection"
    type: "terminal-command"
    confidence: 0.85
    pattern: "myctl"
    description: "Detected via terminal command"
    action: "Activate skill pack for My Technology"

  - name: "config-file-detection"
    type: "config-file"
    confidence: 0.95
    pattern: "/etc/my-technology/*.conf"
    description: "Detected via configuration files"
    action: "Activate skill pack for My Technology"

  - name: "window-title-detection"
    type: "window-title"
    confidence: 0.75
    pattern: "My Technology Portal"
    description: "Detected via window title"
    action: "Activate skill pack for My Technology"

  - name: "process-name-detection"
    type: "process-name"
    confidence: 0.80
    pattern: "my-technology-service"
    description: "Detected via process name"
    action: "Activate skill pack for My Technology"
```

### Confidence Scoring

| Confidence Level | Range | Behavior |
|-----------------|-------|----------|
| **High** | 0.85 - 1.0 | Auto-activate skill pack |
| **Medium** | 0.60 - 0.84 | Auto-activate with confirmation |
| **Low** | 0.30 - 0.59 | Ask user to confirm |
| **Very Low** | 0.00 - 0.29 | Ignore, no activation |

## Workflow Definitions

Workflows define state machine-based processes for common engineering tasks.

```yaml
# workflows.yaml

workflows:
  - name: "troubleshoot-service"
    description: "Troubleshoot My Technology service issues"
    category: "diagnostic"
    priority: "high"

    states:
      - id: "evidence_collection"
        description: "Collect diagnostic evidence"
        tasks:
          - "Check service status"
          - "Review log files"
          - "Verify configuration"
          - "Check system resources"
        transitions:
          - to: "root_cause_analysis"
            condition: "evidence sufficient"
            confidence_threshold: 0.70
          - to: "ask_user"
            condition: "insufficient evidence"

      - id: "root_cause_analysis"
        description: "Analyze evidence to determine root cause"
        tasks:
          - "Correlate symptoms"
          - "Check for known issues"
          - "Evaluate configuration changes"
        transitions:
          - to: "remediation"
            condition: "root cause identified"
          - to: "evidence_collection"
            condition: "insufficient evidence"

      - id: "remediation"
        description: "Apply fix and verify"
        tasks:
          - "Apply recommended fix"
          - "Verify service is running"
          - "Monitor for recurrence"
        transitions:
          - to: "completed"
            condition: "remediation verified"

      - id: "ask_user"
        description: "Ask user for additional information"
        tasks:
          - "Present available evidence"
          - "Request additional information"
        transitions:
          - to: "evidence_collection"
            condition: "new evidence received"
          - to: "completed"
            condition: "user provides enough info"

      - id: "completed"
        description: "Workflow completed"
        is_terminal: true
```

## Command References

Command references document technical commands for the technology.

```yaml
# commands.yaml

commands:
  - name: "status"
    command: "myctl status"
    description: "Check service status"
    category: "operations"
    usage: "myctl status [SERVICE]"
    flags:
      - name: "--verbose"
        description: "Show detailed status information"
        required: false
      - name: "--json"
        description: "Output in JSON format"
        required: false
    example: |
      $ myctl status service-a
      Service: service-a
      Status: Running
      Uptime: 2d 14h 30m
      Version: 1.0.0
    output_fields:
      - "service_name"
      - "status"
      - "uptime"
      - "version"

  - name: "restart"
    command: "myctl restart"
    description: "Restart a service"
    category: "operations"
    usage: "myctl restart SERVICE"
    flags:
      - name: "--graceful"
        description: "Graceful restart (wait for connections to drain)"
        required: false
      - name: "--force"
        description: "Force restart (kill and restart immediately)"
        required: false
    example: |
      $ myctl restart service-a --graceful
      Restarting service-a...
      Service restarted successfully.
    warnings:
      - "Restarting during peak hours may cause downtime"
      - "Always check status after restart"
```

## Guidance Rules

Guidance rules provide engineering guidance based on technology context.

```markdown
# guidance/rules.md

## Service Troubleshooting

### High CPU on Service

**Evidence:**
- `myctl status` shows high CPU usage
- System monitoring shows elevated CPU

**Actions:**
1. Check for runaway queries or processes
2. Review recent configuration changes
3. Check disk space for temporary files
4. Review slow query log

### Service Unavailable

**Evidence:**
- Service shows "Stopped" or "Failed" in status
- Port is not listening

**Actions:**
1. Check service logs: `myctl logs service-a`
2. Verify configuration file syntax: `myctl config validate`
3. Check system resources: `df -h`, `free -m`
4. Restart service: `myctl restart service-a`
```

## Knowledge Base

Build the knowledge base with documentation files:

```
knowledge/
├── architecture.md           # Detailed architecture documentation
└── best-practices.md         # Engineering best practices guide
```

### Architecture Documentation

Include:
- System architecture overview
- Component interactions
- Data flow diagrams
- Configuration model
- Scaling considerations

### Best Practices Guide

Include:
- Deployment best practices
- Configuration best practices
- Monitoring and alerting
- Backup and recovery
- Security best practices
- Performance tuning

## Packaging and Distribution

### Package Format

Skill packs are packaged as `.wkl` archives (similar to `.tar.gz`):

```bash
# Package a skill pack
cargo run --bin knowledge-cli package src/skills/my-skill-pack

# Output: src/skills/my-skill-pack.wkl
```

### Distribution Methods

| Method | Description |
|--------|-------------|
| **File system** | Copy `.wkl` or directory to `src/skills/` |
| **Manual distribution** | Share `.wkl` file directly |
| **Internal repository** | Store `.wkl` files in internal artifact repository |
| **Update mechanism** | Future: automatic skill pack updates |

### Installing a Skill Pack

1. **Directory installation:**
   ```bash
   cp -r my-skill-pack/ src/skills/
   ```

2. **Archive installation (future):**
   ```bash
   cp my-skill-pack.wkl src/skills/
   ```

3. **Restart the application** — the Skill Discovery Engine will detect the new skill pack

## Validation

### Validation Process

The knowledge CLI validates skill packs against these criteria:

| Check | Description |
|-------|-------------|
| **Manifest** | Required fields present and valid |
| **Technology** | Valid technology definitions |
| **Detection Rules** | Valid rule types and patterns |
| **Workflows** | Valid state machines, no dead ends |
| **Commands** | Valid command definitions |
| **Guidance** | Guidance rules present and valid |
| **File Structure** | Expected directories exist |
| **Version** | Version follows semantic versioning |

### Running Validation

```bash
# Validate a skill pack
cargo run --bin knowledge-cli validate src/skills/my-skill-pack

# Expected output:
# Validating: src/skills/my-skill-pack
# ✓ manifest.yaml - Valid
# ✓ technology.yaml - Valid
# ✓ detection_rules.yaml - Valid
# ✓ workflows.yaml - Valid
# ✓ commands.yaml - Valid
# ✓ guidance/rules.md - Valid
# ✓ File structure - Valid
# Result: VALID
```

### Common Validation Errors

| Error | Fix |
|-------|-----|
| "Missing required field: id" | Add `id` to manifest.yaml |
| "Invalid detection rule type" | Use one of: `browser-url`, `terminal-command`, `window-title`, `config-file`, `process-name`, `log-pattern` |
| "Invalid workflow state transition" | Ensure all `to` targets exist as state IDs |
| "Empty commands.yaml" | Add at least one command definition |

## Testing

### Detection Testing

Test detection rules by:
1. Opening a browser with the target URL
2. Running a target command in terminal
3. Opening a window with the target title
4. Checking the skill pack activates correctly

### Guidance Testing

Test guidance by:
1. Creating a workspace with the target technology
2. Engaging in a conversation about the technology
3. Verifying the guidance panel shows relevant recommendations
4. Checking confidence levels are appropriate

### Workflow Testing

Test workflows by:
1. Triggering the detection conditions
2. Following the workflow states
3. Verifying transitions work correctly
4. Checking completion produces appropriate output

## Update and Versioning

### Semantic Versioning

Skill packs use semantic versioning (MAJOR.MINOR.PATCH):

| Version | Change Type | Example |
|---------|------------|---------|
| 1.0.0 → 1.1.0 | New guidance, commands, detection | Minor |
| 1.0.0 → 1.0.1 | Bug fixes, documentation updates | Patch |
| 1.0.0 → 2.0.0 | Breaking changes, new technology | Major |

### Update Process

1. **Create a branch** from the current skill pack
2. **Make changes** to guidance, detection, or other files
3. **Bump version** in manifest.yaml
4. **Validate** the updated skill pack
5. **Replace** the old skill pack directory

### Deprecation

Mark deprecated features in the manifest:

```yaml
deprecated_parameters:
  - parameter: "old_setting"
    since_version: "2.1"
    alternative: "new_setting"
    action: "Update configuration"
```

## SDK Commands

### Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `create-template` | Generate a skill pack template | `skill-sdk create-template --type technology` |
| `validate` | Validate a skill pack | `knowledge-cli validate src/skills/my-skill-pack` |
| `package` | Package a skill pack | `knowledge-cli package src/skills/my-skill-pack` |

### SDK Usage

```bash
# Create a template
cargo run --bin skill-sdk create-template \
  --name "My Technology" \
  --type technology \
  --output src/skills/my-skill-pack

# Validate
cargo run --bin knowledge-cli validate src/skills/my-skill-pack

# Package
cargo run --bin knowledge-cli package src/skills/my-skill-pack
```

## Examples

### Minimal Skill Pack

```yaml
# manifest.yaml
id: "my-skill-pack"
version: "1.0.0"
name: "My Technology"
description: "Expert knowledge for My Technology"
technologies:
  - name: "my-technology"
    platform: "Linux"
    version: "1.0"

# detection_rules.yaml
detection_rules:
  - name: "terminal"
    type: "terminal-command"
    confidence: 0.85
    pattern: "myctl"

# technology.yaml
technologies:
  - name: "my-technology"
    platform: "Linux"
    version: "1.0"
    components:
      - name: "service-a"
```

### Full Skill Pack Example

See the existing skill packs in `src/skills/` for complete examples:
- `src/skills/mysql-skill-pack/` — 41 files, 14 subdirectories
- `src/skills/openshift-skill-pack/` — 40 files, 13 subdirectories
- `src/skills/nagiosxi-skill-pack/` — 19 files, 13 subdirectories

## Best Practices

### Content Quality

1. **Be specific:** Include exact commands, paths, and configuration values
2. **Provide context:** Explain why a recommendation matters
3. **Include evidence:** Support recommendations with observable evidence
4. **Use confidence scoring:** Rate confidence levels accurately
5. **Version-aware:** Note version-specific differences

### Detection Quality

1. **Multiple detection sources:** Use at least 2 detection rule types
2. **High confidence thresholds:** Aim for 0.80+ confidence on critical detections
3. **Low false positive rate:** Ensure patterns are specific enough
4. **Cover all platforms:** Detect across browsers, terminals, and file systems

### Guidance Quality

1. **Follow the framework:** Use the Confidence & Evidence Engine format
2. **Include recommendation, why, confidence, evidence:** Every recommendation needs all four
3. **Actionable steps:** Provide specific commands and configuration changes
4. **Safety warnings:** Include warnings for risky operations
5. **Cross-skill aware:** Note interactions with other skill packs

### Documentation Quality

1. **Clear headings:** Use descriptive section titles
2. **Tables for comparisons:** Use tables for structured information
3. **Code blocks for commands:** Always format commands in code blocks
4. **Cross-references:** Link to related skill packs and documents
5. **Version history:** Note what changed between versions

---

*For skill pack development in the codebase, see [Developer Guide](DEVELOPER_GUIDE.md).*
*For the skill system architecture, see [Architecture Guide](ARCHITECTURE_GUIDE.md).*
*For the confidence engine, see [docs/operations/CONFIDENCE_EVIDENCE_ENGINE.md](operations/CONFIDENCE_EVIDENCE_ENGINE.md).*