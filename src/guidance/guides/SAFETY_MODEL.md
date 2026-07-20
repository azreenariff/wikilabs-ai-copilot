# Safety Model

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

The Command Safety Framework classifies recommended commands by risk level and generates appropriate warnings. The AI must warn before showing risky commands.

## Design Principles

1. **Safety first** — Every command is evaluated before being suggested.
2. **Warning on risk** — Risky commands include explicit warnings.
3. **No execution** — The AI never executes commands; the engineer does.
4. **Transparent** — Engineers see the risk level and reasoning.

## Architecture

```
Command Recommendation Engine
    ↓
┌─────────────────────────────────┐
│  Safety Framework               │
│                                 │
│  • CommandRiskLevel             │
│  • CommandRisk                  │
│  • SafetyEvaluation             │
│  • WarningGenerator             │
└─────────────────────────────────┘
    ↓
Desktop UI (displays warning banners)
```

## Risk Levels

| Level | Description | Warnings |
|-------|-------------|----------|
| `Safe` | Read-only, no side effects | None |
| `PotentiallyDisruptive` | May affect running services | Warning banner |
| `Dangerous` | Irreversible action | Warning + confirmation request |

## Key Types

### CommandRisk

```rust
pub struct CommandRisk {
    pub level: CommandRiskLevel,
    pub warning_message: String,
    pub mitigation_steps: Vec<String>,
    pub requires_confirmation: bool,
}
```

### CommandRiskLevel

Five levels of command risk:

| Level | Description |
|-------|-------------|
| `Safe` | Information gathering — no risk |
| `Diagnostic` | May affect running services |
| `Configuration` | Changes configuration |
| `PotentiallyDisruptive` | May cause downtime |
| `Dangerous` | Irreversible action |

### SafetyEvaluation

```rust
pub struct SafetyEvaluation {
    pub command: String,
    pub risk: CommandRisk,
    pub warning_shown: bool,
    pub engineer_confirmed: bool,
}
```

## Usage

### Evaluating a Command

```rust
let evaluator = SafetyEvaluator::new();
let evaluation = evaluator.evaluate("rm -rf /var/log/*");

println!("Risk level: {}", evaluation.risk.level);
// → Dangerous
println!("Warning: {}", evaluation.risk.warning_message);
// → "This command permanently deletes log files. This action cannot be undone."
```

### Generating Warnings

```rust
let warning = WarningGenerator::generate(&evaluation);
println!("{}", warning);
// "⚠️ DANGEROUS COMMAND
//  This command permanently deletes log files.
//  This action cannot be undone.
//
//  Mitigation steps:
//  1. Ensure you have backup copies of important logs
//  2. Verify the target path is correct
//  3. Consider using a dry-run first if available"
```

## Validation Checklist

- ✅ Commands classified into five risk levels
- ✅ Warnings generated for PotentiallyDisruptive+ commands
- ✅ Confirmation requested for Dangerous commands
- ✅ Mitigation steps included in warnings
- ✅ No autonomous execution of risky commands
- ✅ 41 unit tests covering all risk levels and warning generation