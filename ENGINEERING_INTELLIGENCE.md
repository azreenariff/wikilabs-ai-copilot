# Engineering Intelligence Engine

> Architecture Overview — Wiki Labs AI Copilot v0.4.0-alpha

## Purpose

The Engineering Intelligence Engine transforms raw observation events into engineering understanding. It answers:

1. What technology is the engineer working with?
2. What is the engineer trying to accomplish?
3. What stage of an engineering workflow is the engineer currently in?
4. What evidence has already been collected?
5. What information is missing?
6. Whether enough context exists to provide advice?

## Architecture

```
Observation Framework
    |
    v
Engineering Context Engine
    |
    v
Engineering Intelligence Engine
    |
    v
Skill Runtime
    |
    v
Technology Skills
    |
    v
(Optional) MCP Tools
```

## Components

### Technology Recognition Engine

Determines which technologies the engineer is working with by analyzing evidence from:

- Active application
- Browser title and URL
- Terminal commands
- Workspace context
- User conversation
- Observation events

### Intent Recognition Engine

Determines the engineer's objective (installation, upgrade, troubleshooting, etc.) and continuously updates as new evidence arrives.

### Engineering Workflow Engine

Tracks where the engineer is in their engineering process using state machines defined by Skills, not hardcoded in the core.

### Context Fusion Engine

Combines observation events, conversation, workspace, technology, intent, workflow state, timeline, and human corrections into a unified engineering context.

### Confidence Engine

Every inference includes a confidence score. Low confidence triggers confirmation requests rather than assumptions.

### Engineering Timeline

Maintains chronological engineering activity with references to source observation events.

### Recommendation Readiness Engine

Determines if enough information exists before generating advice — without generating advice itself.

### Human Feedback Loop

Allows the human engineer to correct AI assumptions. Human input always overrides inference.

## Architectural Principle

**The core application remains completely technology-agnostic.**

All technology knowledge comes from Skills. The core contains no OpenShift, VMware, Linux, Nagios, or database-specific logic.

## Testing

- Technology detection tests
- Intent detection tests
- Workflow transition tests
- Skill loading tests
- Skill validation tests
- SDK generation tests
- Context update tests
- Human correction tests
- Confidence scoring tests

## Constraints

The Intelligence Engine does NOT implement:

- Knowledge Retrieval / RAG
- MCP execution
- Command execution
- Automation
- Screen AI analysis / OCR
- Customer environment access
- Autonomous actions

The AI remains advisory. The human engineer remains responsible for all actions.