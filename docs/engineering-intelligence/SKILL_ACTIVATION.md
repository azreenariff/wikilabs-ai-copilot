# Skill Activation

> Wiki Labs AI Copilot v0.8.0-alpha  
> Phase 11 — Enterprise Skill Platform

## Purpose

The Skill Activation Engine receives discovery reports from the Skill Discovery Engine, matches detected technologies to skill definitions, resolves dependencies, and activates or deactivates skills dynamically at runtime.

## Activation Flow

```
DiscoveryReport from Discovery Engine
        │
        ▼
┌──────────────────────────┐
│  1. Receive candidates    │  ← Build ActivationCandidates from
│     (DiscoveredSkills)    │    DiscoveredSkills in the report
└─────────┬────────────────┘
          │
          ▼
┌──────────────────────────┐
│  2. Match to definitions  │  ← Match detected technology →
│                          │    skill definitions in the
│                          │    Skill Runtime registry
└─────────┬────────────────┘
          │
          ▼
┌──────────────────────────┐
│  3. Resolve dependencies  │  ← Check all prerequisite skills
│                          │    are loaded and enabled
└─────────┬────────────────┘
          │
          ▼
┌──────────────────────────┐
│  4. Score & threshold     │  ← Verify confidence >=
│     check                  │    activation threshold
└─────────┬────────────────┘
          │
          ▼
┌──────────────────────────┐
│  5. Activate / Deactivate │  ← Transition skill state,
│                          │    send notifications
└──────────────────────────┘
          │
          ▼
┌──────────────────────────┐
│  6. Health monitoring     │  ← Periodic health checks,
│                          │    degrade/recover cycles
└──────────────────────────┘
```

## Core Types

### ActivationCandidate

A detected skill ready for activation evaluation.

```rust
pub struct ActivationCandidate {
    pub skill_id: String,         // Skill ID to activate
    pub skill_name: String,       // Human-readable name
    pub technology: String,       // Technology domain
    pub confidence: f64,          // Confidence from discovery (0.0–1.0)
    pub auto_activate: bool,      // Should auto-activate?
    pub reason: String,           // Why this skill was detected
}
```

### ActivatedSkill

A skill that has been activated with its current lifecycle state.

```rust
pub struct ActivatedSkill {
    pub skill_id: String,
    pub skill_name: String,
    pub activated_at: String,     // ISO timestamp
    pub confidence: f64,
    pub state: ActivationState,
    pub last_health_check: Option<String>,
    pub failure_count: u32,
}
```

### ActivationState

```rust
pub enum ActivationState {
    Active,       // Skill is active and healthy
    Inactive,     // Skill has been deactivated
    Degraded,     // Skill failed a health check
    Removed,      // Skill has been removed
}
```

### SkillDefinition

The skill definition loaded from its manifest.

```rust
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub technology: String,
    pub category: String,
    pub enabled: bool,
    pub dependencies: Vec<String>,
}
```

### ActivationConfig

```rust
pub struct ActivationConfig {
    pub auto_activate: bool,            // Auto-activate on detection
    pub min_confidence: f64,            // Minimum confidence threshold (default: 0.7)
    pub max_failure_count: u32,         // Max health check failures before deactivation (default: 3)
    pub health_check_interval_ms: u64,  // Interval between health checks
    pub health_check_timeout_ms: u64,   // Timeout per health check
    pub notification_callbacks: Vec<Callback>, // Called on state changes
}
```

## Activation States

```
                    ┌──────────┐
                    │  Loaded   │  (in runtime registry)
                    └────┬─────┘
                         │
                    ┌────▼─────┐
         enabled → │ Enabled  │ ← Manifest says enabled
                    └────┬─────┘
                         │
         detected        │
         + confident      │
    ┌────▼────────────────▼─────┐
    │         Active            │ ← Auto-activated on detection
    └────┬──────────────────────┘
         │
    ┌────▼──────────────────────┐
    │       Degraded            │ ← Health check failed > max times
    └────┬──────────────────────┘
         │ recovery
    ┌────▼─────┐
    │  Active   │ ← Health checks pass again
    └──────────┘
         │
    ┌────▼─────┐
    │ Disabled  │ ← Manually disabled or manifest says enabled=false
    └──────────┘
```

## Dependency Resolution

Before activating a skill, the engine verifies all dependencies:

1. Each dependency `id` in the skill's manifest must exist in the runtime's loaded skills
2. Each dependency must be in the `Enabled` or `Active` state
3. If any dependency is missing or disabled, the skill is **not activated** and logged as dependency-unmet
4. Circular dependencies are rejected at load time

### Example

```yaml
# openshift-engineering manifest
dependencies:
  - linux-engineering
  - kubernetes-basics
```

When the openshift skill is detected for activation:
- Check `linux-engineering` is loaded and enabled ✓
- Check `kubernetes-basics` is loaded and enabled ✓
- If both pass → activate openshift
- If either fails → log, skip activation

## Confidence Threshold

The `min_confidence` config field (default `0.7`) prevents activating skills on weak signals:

| Confidence Range | Action |
|-----------------|--------|
| ≥ 0.9 | Auto-activate immediately |
| 0.7 – 0.89 | Auto-activate with medium confidence |
| < 0.7 | Skip activation; signal too weak |

## Health Monitoring

Activated skills are periodically health-checked:

1. Check runs at `health_check_interval_ms`
2. Each check has a `health_check_timeout_ms` limit
3. If a check fails: increment `failure_count`
4. If `failure_count >= max_failure_count`: transition to `Degraded`
5. If subsequent checks pass: decrement `failure_count` back toward 0
6. If fully recovered (failure_count == 0): return to `Active`

## Notifications

The activation engine supports callbacks for state transitions:

```rust
pub enum SkillEvent {
    Activated { skill_id: String, confidence: f64 },
    Deactivated { skill_id: String, reason: String },
    Degraded { skill_id: String, failure_count: u32 },
    Recovered { skill_id: String },
}
```

Callbacks are invoked synchronously when a skill's state changes.

## API Reference

```rust
// Process discovery report
pub fn process_discovery(
    &mut self,
    candidates: Vec<ActivationCandidate>,
) -> Result<Vec<ActivatedSkill>>;

// Activate a specific skill by ID
pub fn activate(&mut self, skill_id: &str) -> Result<ActivatedSkill>;

// Deactivate a skill by ID
pub fn deactivate(&mut self, skill_id: &str) -> Result<()>;

// Get all currently active skills
pub fn get_active_skills(&self) -> Vec<&ActivatedSkill>;

// Check health of a specific skill
pub fn health_check(&self, skill_id: &str) -> Result<ActivationState>;

// Get full state for a skill
pub fn get_skill_state(&self, skill_id: &str) -> Option<ActivatedSkill>;

// Register a skill definition (called by Skill Runtime)
pub fn register_skill(&mut self, definition: SkillDefinition);
```

## Testing

The activation engine includes unit tests for:

- `test_activate_no_candidates` — No candidates, no skills activated
- `test_auto_activate_high_confidence` — High-confidence skill auto-activates
- `test_deactivate_skill` — Skill can be deactivated after activation
- `test_health_check` — Health check returns Active for healthy skill
- `test_dependency_resolution` — Skills with unmet dependencies are not activated
- `test_confidence_threshold` — Low-confidence signals are skipped
- `test_circular_dependency_detection` — Circular deps are rejected
- `test_health_degradation` — Skill transitions to Degraded after too many failures
- `test_health_recovery` — Skill recovers to Active after passing checks
- `test_multiple_skills_active` — Multiple skills can be active simultaneously

## Integration with Copilot

Activated skills provide their capabilities to the Copilot Recommendation Engine:

1. Discovery Engine detects technologies → produces report
2. Activation Engine processes report → activates matching skills
3. Active skills' knowledge, workflows, and guidance rules are available to the Copilot
4. The Copilot uses active skill data to generate context-aware recommendations
5. When the engineer's context changes, the cycle repeats

The engine never executes commands or modifies systems. Skills are pure knowledge sources.