# User Stories — Wiki Labs AI Copilot

## Core User Journey

```
Engineer opens the application
  → Workspace selector (default: "Default Workspace")
  → Chat interface is ready
  → User types: "Why is my pod in CrashLoopBackOff?"
  → AI responds with diagnosis + knowledge citations
  → User can invoke skill tools for deeper investigation
  → All conversation history is stored in the workspace
```

## Detailed User Stories

### US-01: Chat with AI

| Field | Value |
|-------|-------|
| **Story** | As an engineer, I want to chat with an AI that knows my customer's infrastructure |
| **Priority** | P0 |
| **Acceptance Criteria** | AI responds with citations to knowledge documents; skills can be invoked via tool calls |

### US-02: Import Knowledge

| Field | Value |
|-------|-------|
| **Story** | As an engineer, I want to import my customer's documentation so the AI can reference it |
| **Priority** | P0 |
| **Acceptance Criteria** | Import markdown, PDF, text files; search returns relevant results with citations |

### US-03: Manage Workspaces

| Field | Value |
|-------|-------|
| **Story** | As an engineer, I want to create workspaces for different customers with different tech stacks |
| **Priority** | P0 |
| **Acceptance Criteria** | Create, switch, delete workspaces; each has isolated knowledge and history |

### US-04: Invoke Skill Tools

| Field | Value |
|-------|-------|
| **Story** | As an engineer, I want to invoke infrastructure skill tools through chat |
| **Priority** | P0 |
| **Acceptance Criteria** | Tools suggest commands; human confirms before execution |

### US-05: Observe Terminal Activity

| Field | Value |
|-------|-------|
| **Story** | As an engineer, I want the AI to see what I'm doing in the terminal |
| **Priority** | P1 |
| **Acceptance Criteria** | Shell integration captures commands; provides context-aware suggestions |

### US-06: View Interaction History

| Field | Value |
|-------|-------|
| **Story** | As an engineer, I want to review my previous AI interactions |
| **Priority** | P1 |
| **Acceptance Criteria** | Full conversation history stored per workspace; scrollable |

### US-07: Secure Credential Storage

| Field | Value |
|-------|-------|
| **Story** | As an engineer, I want to store my API keys and credentials securely |
| **Priority** | P0 |
| **Acceptance Criteria** | Credentials stored in OS keychain; encrypted at rest; never transmitted |