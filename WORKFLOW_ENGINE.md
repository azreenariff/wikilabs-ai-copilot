# Workflow Engine

> Wiki Labs AI Copilot v0.4.0-alpha

## Purpose

Track where the engineer is in their engineering workflow using state machines defined by Skills — not hardcoded in the core.

## Architecture

Workflows support:

- **States** — Engineering phases (gather_information, inspect_events, inspect_logs, etc.)
- **Transitions** — Valid state changes
- **Required evidence** — Data needed to proceed
- **Confidence requirements** — Minimum confidence to enter/exit states
- **Completion criteria** — When a workflow is considered complete
- **Validation rules** — Guards on transitions

## Example: OpenShift Troubleshooting

```yaml
workflow:
  name: openshift_troubleshooting

states:
  - gather_information
  - inspect_events
  - inspect_logs
  - root_cause_analysis
  - resolution_validation

transitions:
  - from: gather_information
    to: inspect_events
    required_evidence:
      - cluster_access
      - pod_identification
```

## States Are Dynamic

States, transitions, and evidence requirements are defined in each Technology Skill's `workflows.yaml`. The core engine has no hardcoded workflows.

## Implementation

- `src/workflow_engine/src/lib.rs` — WorkflowEngine with state machine
- `src/workflow_engine/Cargo.toml` — Dependencies

## Testing

See `src/workflow_engine/src/lib.rs` for workflow transition, state validation, and evidence requirement tests.