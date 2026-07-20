# ADR-012: Skill Platform Architecture

## Status: Accepted

## Date: 2025-01-15

## Context

The AI Copilot needs a system to dynamically discover, activate, and manage enterprise skills based on workspace technology signatures. Previous approaches had skills hardcoded or manually configured. We need a system that:

1. Automatically detects technologies in the workspace
2. Matches detected technologies to skill definitions
3. Activates skills dynamically at runtime
4. Monitors skill health and deactivates on failure
5. Provides a management UI for administrators

## Decision

We will implement a three-component Skill Platform:

1. **Skill Discovery Engine** — Scans the workspace for technology signals using glob patterns and command detection
2. **Skill Activation Engine** — Dynamically activates matching skills with dependency checking and health monitoring
3. **Skill Runtime** — Manages loaded skills, validation, and lifecycle

### Key Design Decisions

1. **Separation of concerns**: Discovery, activation, and runtime are separate modules with clear interfaces
2. **Signal-based activation**: Skills activate based on detected workspace signals, not manual configuration
3. **Health monitoring**: Periodic health checks with configurable max failure count
4. **Dependency resolution**: Skills must have all dependencies loaded and enabled before activation
5. **YAML-based skill definitions**: Standardized manifest format for portability
6. **Dynamic lifecycle**: Skills can be activated/deactivated without restart

## Consequences

### Positive
- Skills activate automatically based on workspace state
- No manual configuration required for skill activation
- Health monitoring prevents stale or broken skills
- Clear module boundaries enable independent evolution
- YAML-based definitions are human-readable and editable

### Negative
- Adds complexity to the runtime
- Signal detection may have false positives/negatives
- Requires maintaining technology signatures
- Health checks add overhead to the runtime loop

### Mitigations
- Confidence scoring reduces false activation risk
- Auto-activation threshold configurable (default 0.5)
- Technology signatures are extensible via `register_signature()`
- Health check interval configurable (default 5 minutes)

## Alternatives Considered

### Alternative 1: Manual Skill Configuration
Skills are manually enabled/disabled via UI. **Rejected** because it doesn't leverage workspace context.

### Alternative 2: Single Monolithic Module
All discovery, activation, and runtime logic in one module. **Rejected** because it creates a tight coupling and makes testing difficult.

### Alternative 3: Plugin-Based Architecture
Skills are loaded as shared libraries (`.so`/`.dylib`). **Rejected** because it adds security risks and deployment complexity.

## Implementation Notes

- Discovery Engine: `src/skill_discovery/`
- Activation Engine: `src/skill_activation/`
- Skill Runtime: `src/skill_runtime/` (integrates both engines)
- Skill SDK: `src/skill_sdk/` (template generation and validation)
- Skill Management UI: `src-tauri/src/skill_management.rs`

## References

- [Skill Platform Architecture](../../docs/architecture/skill-platform.md)
- [Skill Platform Quick Start](../../docs/architecture/skill-platform-quickstart.md)
- [SKILL_ARCHITECTURE.md](../../docs/KNOWLEDGE_PLATFORM.md)