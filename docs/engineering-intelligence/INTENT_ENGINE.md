# Intent Recognition Engine

> Wiki Labs AI Copilot v0.4.0-alpha

## Purpose

Determine what the engineer is trying to accomplish and continuously update as new evidence arrives.

## How It Works

Intent recognition is technology-aware. Each technology Skill defines its own intents:

**OpenShift intents:**

- Installation
- Upgrade
- Deployment
- Troubleshooting
- Networking
- Storage
- Security

**Linux intents:**

- Performance analysis
- Service troubleshooting
- Package management
- Log investigation

**VMware intents:**

- VM deployment
- Performance troubleshooting
- Storage
- Networking

## Continuous Updating

The engine updates intents as new observation events arrive, combining:

- Observation events (terminal, browser, file, application)
- Conversation content
- Workspace context
- Technology inference
- Human feedback

## Output

```yaml
technology: OpenShift
intent: troubleshooting
confidence: 0.87
evidence:
  - "oc describe pod command"
  - "kubectl logs access"
  - "error messages in conversation"
```

## Human Override

Human input always overrides inference:

```
Engineer: "I am performing an upgrade, not troubleshooting."
Updated intent: upgrade
Updated workflow: Upgrade Preparation
```

## Implementation

- `src/intent/src/engine.rs` — IntentEngine core
- `src/intent/src/model.rs` — Intent model structures
- `src/intent/src/confidence.rs` — Confidence scoring
- `src/intent/src/correction.rs` — Human correction handling
- `src/intent/src/lib.rs` — Public API

## Testing

See `src/intent/src/engine.rs`, `src/intent/src/model.rs`, `src/intent/src/confidence.rs`, and `src/intent/src/correction.rs` for test coverage.