# Command Guidance

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

The Command Recommendation Engine suggests CLI commands, API queries, and configuration checks. Every command includes a purpose, expected output, risk level, and explanation.

## Design Principles

1. **Suggestion only** — Commands are suggestions; the engineer executes them manually.
2. **Context-aware** — Commands include expected output and purpose.
3. **Risk-classified** — Every command is classified by risk level.
4. **Explained** — Engineers understand what each command does and why.

## Architecture

```
Decision Engine (decides to suggest a command)
    ↓
┌─────────────────────────────────┐
│  Command Recommendation         │
│  Engine                         │
│                                 │
│  • CommandSuggestion            │
│  • CommandRiskLevel             │
│  • CommandCategory              │
│  • CommandSuggestionBuilder     │
└─────────────────────────────────┘
    ↓
Safety Framework (classifies risk)
    ↓
Desktop UI (displays command cards)
```

## Key Types

### CommandSuggestion

```rust
pub struct CommandSuggestion {
    pub id: Uuid,                         // Unique identifier
    pub command: String,                  // The command itself
    pub purpose: String,                  // What it does
    pub expected_output: String,          // What to look for in the output
    pub risk_level: CommandRiskLevel,     // Risk classification
    pub explanation: String,              // Why run this command
    pub technology: String,               // e.g. "OpenShift"
    pub prerequisites: Vec<String>,       // What to do before running
    pub follow_up: Option<String>,        // What to do after
    pub reference_docs: Vec<ReferenceDoc>,
}
```

### CommandRiskLevel

```rust
pub enum CommandRiskLevel {
    Informational,   // Read-only, no side effects
    Diagnostic,      // May affect running services
    Configuration,   // Changes configuration
    PotentiallyDisruptive, // May cause downtime
    Dangerous,       // Irreversible action
}
```

### CommandCategory

```rust
pub enum CommandCategory {
    LinuxCLI,    // Linux commands
    PowerShell,  // Windows PowerShell
    SQL,         // SQL queries
    OpenShift,   // OpenShift CLI commands
    Kubectl,     // Kubernetes CLI commands
    Network,     // Network diagnostics
    VMware,      // VMware commands
    Database,    // Database management
    SystemAdmin, // General system administration
}
```

## Usage

### Building a Command Suggestion

```rust
let suggestion = CommandSuggestionBuilder::new(
    "oc get pods --field-selector=status.phase=Failed",
    CommandCategory::OpenShift,
    "Linux",
)
.purpose("List failed pods in a namespace")
.expected_output("Pod names, restart counts, and exit codes for failed pods")
.risk(CommandRiskLevel::Informational)
.explanation("Failed pods indicate application errors. This helps identify which pods are unhealthy and why.")
.prerequisites(vec!["Set the correct namespace with -n <namespace>"])
.follow_up(Some("Use 'oc describe pod <pod-name>' for detailed diagnostics".to_string()))
.build();
```

### Displaying a Command

```rust
println!("📋 {}", suggestion.purpose);
println!("⚡ {}\n", suggestion.command);
println!("Expected output: {}", suggestion.expected_output);
println!("Risk: {}", suggestion.risk_level);
println!("{}", suggestion.explanation);
```

## Risk Classification Rules

| Level | Description | Examples |
|-------|-------------|---------|
| `Informational` | Read-only, no side effects | `df -h`, `ps aux`, `oc get pods` |
| `Diagnostic` | May affect running services | `oc debug pod`, `vmstat` |
| `Configuration` | Changes configuration | `oc set resources`, `sysctl` |
| `PotentiallyDisruptive` | May cause downtime | `oc delete pod`, `systemctl restart` |
| `Dangerous` | Irreversible action | `oc delete pvc`, `dd`, `rm -rf` |

## Validation Checklist

- ✅ Every command includes purpose, expected output, risk, explanation
- ✅ Risk levels are classified per spec
- ✅ Commands are suggestions only (no execution)
- ✅ Technology classification on each command
- ✅ Prerequisites and follow-up steps included
- ✅ Integration with Safety Framework for risk warnings
- ✅ 41 unit tests covering all command types and risk levels