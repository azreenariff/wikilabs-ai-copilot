---
description: "System-level architecture overview for Wiki Labs AI Copilot — architecture pipeline, layered design, component diagram, data flows, deployment, and design principles."
icon: code-branch
---

# Wiki Labs AI Copilot — System Architecture

**AI-powered enterprise engineering copilot** for infrastructure and DevOps engineers. The AI acts as "an experienced senior engineer sitting beside you" — observing, understanding context, providing recommendations, explaining issues, suggesting commands, and guiding through best practices. The human remains responsible for ALL actions.

## Vision

Engineers need real-time AI assistance during their daily work — on OpenShift, VMware, Nagios, Linux systems, databases, and more. Instead of switching between browser tabs, documentation, and CLI, they want a desktop copilot that understands what they're doing and offers contextual help.

## Architecture Pipeline

```
Human Engineer
    │
    ▼
┌─────────────────────┐
│  Observation Layer   │  Screenshots, terminal activity, app context, clipboard
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Context Understanding │  Aggregates screen + terminal + app + conversation + history
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  Intent Recognition   │  Infers: OpenShift install? VMware perf issue? Nagios alert?
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  MCP Skill Selection │  Route to: OpenShift | Linux | VMware | Ansible | MySQL | ...
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  Knowledge Retrieval │  Vector search + FTS over vendor docs, SOPs, past incidents
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│     AI Reasoning     │  LLM reasoning with context, knowledge, skill workflows
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  Advisor Interface   │  Chat UI, real-time suggestions, inline annotations
└─────────────────────┘
```

## Component Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        WIKI LABS AI COPILOT                         │
│                     Desktop App (Windows / macOS)                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                  REACT FRONTEND LAYER                       │    │
│  │                                                             │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │    │
│  │  │ AI Chat  │ │Workspace │ │Skill Mgr │ │ Knowledge Mgr│   │    │
│  │  │ UI       │ │ Panel    │ │ UI       │ │ UI           │   │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │    │
│  │  │Suggest.  │ │Settings  │ │ Logs     │ │ Reports/     │   │    │
│  │  │Panel     │ │Panel     │ │Panel     │ │ Audit        │   │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │    │
│  └────────────────────────┬────────────────────────────────────┘    │
│                           │ JSON-RPC (HTTP over localhost)          │
│                           │ Tauri IPC (native OS)                   │
│  ┌────────────────────────▼────────────────────────────────────┐    │
│  │                  RUST CORE ENGINE                           │    │
│  │                                                             │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐              │    │
│  │  │ Event Bus  │ │  RPC Layer │ │  Persistence│              │    │
│  │  │(async msg) │ │(JSON-RPC)  │ │  (SQLite)   │              │    │
│  │  └────────────┘ └────────────┘ └────────────┘              │    │
│  │                                                             │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐              │    │
│  │  │  Observation │ │ Intent   │ │ AI Provider │              │    │
│  │  │   Engine     │ │ Engine   │ │  Abstraction│              │    │
│  │  └────────────┘ └────────────┘ └────────────┘              │    │
│  │                                                             │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐              │    │
│  │  │ MCP Skill  │ │ Knowledge  │ │ Workflow   │              │    │
│  │  │  Manager   │ │  System    │ │  Engine    │              │    │
│  │  └────────────┘ └────────────┘ └────────────┘              │    │
│  │                                                             │    │
│  │  ┌────────────┐ ┌────────────┐                             │    │
│  │  │Credential  │ │  Security   │                             │    │
│  │  │  Manager   │ │  Manager    │                             │    │
│  │  └────────────┘ └────────────┘                             │    │
│  └────────────────────────┬────────────────────────────────────┘    │
│                           │                                         │
│  ┌────────────────────────▼────────────────────────────────────┐    │
│  │                  MCP TRANSPORT LAYER                        │    │
│  │                                                             │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐              │    │
│  │  │  OpenShift │ │    Linux   │ │   VMware   │              │    │
│  │  │  MCP Server│ │  MCP Server│ │ MCP Server │              │    │
│  │  └────────────┘ └────────────┘ └────────────┘              │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐              │    │
│  │  │  Ansible   │ │    Nagios  │ │   MySQL    │              │    │
│  │  │  MCP Server│ │  MCP Server│ │ MCP Server │              │    │
│  │  └────────────┘ └────────────┘ └────────────┘              │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Architectural Principles

1. **Human-in-the-Loop (HITL)**: The system NEVER executes actions autonomously. Every recommendation requires human confirmation. The AI is an advisor, not an operator.

2. **Local-First Architecture**: All data resides on the engineer's laptop. No cloud dependency for core functionality. Knowledge, workspaces, sessions, and settings are stored locally.

3. **Layered Separation**: Clear separation between UI (React), business logic (Rust core), domain expertise (MCP servers), and data storage (SQLite + vector store).

4. **Replaceable AI Provider**: No vendor lock-in. The AI provider abstraction allows switching between OpenAI, vLLM, local models, and enterprise APIs without changing application code.

5. **Privacy-By-Design**: Screen observation is opt-in, with granular controls. No sensitive data leaves the machine. Credentials stored in OS credential managers.

6. **Modular Domain Expertise**: Each technology domain (OpenShift, VMware, Linux, etc.) is an independent MCP server with its own knowledge base, tools, and troubleshooting workflows.

7. **Customer-Centric Organization**: Workspaces isolate customer contexts. Data, knowledge, and recommendations are scoped to the active workspace.

## Layered Architecture

### Layer 1: Presentation Layer (React Frontend)

The user-facing layer built on React + TypeScript. Renders all UI panels, manages user interactions, and communicates with the Rust core via JSON-RPC over HTTP and Tauri IPC for native OS operations.

**Responsibilities**:
- AI Chat interface with conversation history, thread branching, and inline references
- Real-time suggestion panel showing contextual recommendations as the engineer works
- Workspace management (create, switch, configure workspaces)
- Skill management UI (enable/disable, configure, view skill status)
- Knowledge management (import, search, organize knowledge sources)
- Settings (AI provider config, observation controls, appearance, keyboard shortcuts)
- Logs and audit trails for all AI interactions and system events

### Layer 2: Core Engine (Rust)

The in-process Rust runtime managed by Tauri. Hosts all business logic, event bus, communication protocols, persistence, and coordination between sub-components.

**Responsibilities**:
- Event bus for inter-component communication (async message passing)
- JSON-RPC server for frontend ↔ core communication
- Process management for MCP server lifecycle (start, stop, restart, health check)
- Memory management (short-term, long-term, subconscious memory analogous to OpenHuman's model)
- Configuration management and settings persistence
- Scheduling (periodic background tasks: knowledge index refresh, memory compaction)

### Layer 3: Domain Engines

Specialized processing engines that transform raw observation data into actionable AI context:

| Engine | Purpose |
|--------|---------|
| Observation Engine | Captures and processes screen data (screenshots, OCR, vision), app context (active app, window title, browser URL, terminal sessions), terminal activity (commands, output, errors), and clipboard content |
| Intent Recognition Engine | Analyzes multi-source context to infer the engineer's current task and goal |
| MCP Skill Manager | Discovers, loads, and routes to the appropriate MCP skill server |
| Knowledge System | Retrieves relevant vendor docs, SOPs, and past incident data via vector search and FTS |
| Workflow Engine | Applies structured engineering troubleshooting workflows to guide AI reasoning |
| AI Provider Abstraction | Routes AI requests to the configured provider with consistent interface |

### Layer 4: Data Storage

| Store | Technology | Purpose |
|-------|-----------|---------|
| SQLite | Embedded database | Workspaces, skills config, user preferences, audit logs, session transcripts |
| Vector Store | ChromaDB (embedded) | Knowledge embeddings for semantic search |
| File System | Local disk | Knowledge source files, skill definitions, screenshots (optional, encrypted) |
| OS Keychain | Windows Credential Manager / macOS Keychain | Encrypted credential storage |
| SQLite FTS5 | Embedded | Full-text search over knowledge content |

### Layer 5: MCP Servers

Independent processes per skill domain. Each is a self-contained MCP server providing tools, capabilities, and knowledge for a specific technology. Communicates with the core via stdio (default) or HTTP/WebSocket transport.

## Communication Patterns

### Core ↔ Frontend

```
┌──────────────┐                    ┌──────────────┐
│ React Frontend│                    │  Rust Core    │
│              │ JSON-RPC over HTTP │               │
│  (Browser)   │◄──────────────────►│  (in-process) │
│              │  127.0.0.1:port    │               │
└──────────────┘                    └──────────────┘

Request/Response:
  - rpc:chat_send → {messages: [...], workspace_id: "..."} → {response, context, references}
  - rpc:obs_status → {screen: boolean, terminal: boolean} → {enabled, permissions}
  - rpc:workspace_list → [] → [{id, name, created_at, stack: [...]}]
  - rpc:skill_discover → [] → [{id, name, status, tool_count}]
  - rpc:knowledge_search → {query, workspace_id} → [{doc, relevance, source}]
  - rpc:suggestion_get → {context} → [{type, text, action}]

Events (WebSocket):
  - event:ai_progress → {session_id, progress, stage}
  - event:suggestion_new → {workspace_id, type, content}
  - event:obs_alert → {type, severity, message}
  - event:skill_error → {skill_id, error}
```

### Core ↔ MCP Servers

```
┌──────────────┐                    ┌─────────────────┐
│  Rust Core    │                    │ MCP Skill Server│
│              │ stdio (JSON-RPC)   │ (independent     │
│              │───────────────────►│  process)        │
│              │◄───────────────────│                  │
└──────────────┘                    └─────────────────┘

Protocol (MCP spec):
  - Initialize → Initialized
  - tools/list → [{name, description, inputSchema}]
  - tools/call → {name, arguments} → {content: [{type, text}]}
  - resources/list → [{uri, name, description, mimeType}]
  - prompts/list → [{name, description, arguments}]
```

### Inter-Engine Communication (Event Bus)

```
┌─────────────────────────────────────────────────┐
│                 EVENT BUS (Tokio channels)        │
│                                                   │
│  ObservationEvent  ─┐                             │
│  IntentEvent        ├─► Intent Recognition Engine │
│  ConversationEvent  ┘                             │
│                                                   │
│  IntentEvent ─┐                                   │
│  WorkspaceId  ├─► MCP Skill Manager               │
│  SkillContext ┘                                   │
│                                                   │
│  SkillContext ─┐                                  │
│  Query          ├─► Knowledge System               │
│  WorkspaceId    ┘                                  │
│                                                   │
│  KnowledgeContext ─┐                              │
│  ReasoningInput     ├─► AI Reasoning               │
│  EngineeringContext ┘                              │
│                                                   │
│  Recommendation ──► Event Bus ──► Frontend         │
└─────────────────────────────────────────────────┘
```

## Data Flow — End-to-End Example

Scenario: Engineer opens a VMware vSphere console showing a slow VM, types a terminal command, and asks the Copilot for help.

```
1. Observation Engine captures:
   - Screen: VMware console window with VM performance dashboard
   - Terminal: "vmstat 1 5" command detected
   - Clipboard: No relevant content
   - Active App: VMware Workstation, window title includes "prod-web-01"

2. Context Understanding aggregates:
   - Current: VMware vSphere dashboard, vmstat command, hostname "prod-web-01"
   - Previous: Engineer was in "Acme Corp" workspace
   - Conversation: Empty new session

3. Intent Recognition infers:
   - Technology: VMware vSphere + Linux
   - Goal: Performance troubleshooting of VM "prod-web-01"
   - Confidence: 0.87

4. MCP Skill Selection routes to:
   - Primary: VMware vSphere MCP Server (tool: investigate_vm_performance)
   - Secondary: Linux MCP Server (tool: analyze_vmstat_output)

5. Knowledge Retrieval fetches:
   - VMware KB article: "VM CPU ready time troubleshooting"
   - Acme Corp SOP: "Production VM performance checklist"
   - Past incident: "2024-03-15: prod-web-01 high CPU wait"

6. AI Reasoning synthesizes:
   - Analyzes vmstat output for high %wa (wait time)
   - References VMware KB for CPU ready correlation
   - Compares with past incident pattern
   - Generates recommendation

7. Advisor Interface presents:
   - Chat response with root cause hypothesis
   - Real-time suggestion: "Run 'esxtop' on host to check %rdy"
   - Inline link to relevant VMware KB article
   - Step-by-step troubleshooting guide
```

## Deployment Architecture

```
┌──────────────────────────────────────────────────────┐
│              Engineer's Laptop                        │
│                                                      │
│  ┌────────────────────────────────────────────────┐   │
│  │           Wiki Labs AI Copilot Desktop          │   │
│  │                                                │   │
│  │  ┌──────────────────────────────────────────┐   │   │
│  │  │  Tauri v2 Host (Native Window)           │   │   │
│  │  │  ┌────────────────────────────────────┐   │   │   │
│  │  │  │  React 19 Frontend (WebView)       │   │   │   │
│  │  │  │  Vite dev / Production bundle      │   │   │   │
│  │  │  └────────────────────────────────────┘   │   │   │
│  │  │  ┌────────────────────────────────────┐   │   │   │
│  │  │  │  Rust Core (in-process tokio task) │   │   │   │
│  │  │  │  - Event bus, RPC, persistence     │   │   │   │
│  │  │  │  - Observation, intent, skill mgr  │   │   │   │
│  │  │  └────────────────────────────────────┘   │   │   │
│  │  │  ┌────────────────────────────────────┐   │   │   │
│  │  │  │  MCP Servers (spawned processes)   │   │   │   │
│  │  │  │  - OpenShift, Linux, VMware, etc.  │   │   │   │
│  │  │  └────────────────────────────────────┘   │   │   │
│  │  └──────────────────────────────────────────┘   │   │
│  │                                                │   │
│  │  Local Storage:                                │   │
│  │  - SQLite (~/.local/share/wikilabs/)           │   │
│  │  - Vector DB   (~/.local/share/wikilabs/vectors/)│
│  │  - Files       (~/.local/share/wikilabs/docs/) │   │
│  │  - OS Keychain (credentials)                   │   │
│  └────────────────────────────────────────────────┘   │
│                                                      │
│  External (optional):                                │
│  - AI Provider API (OpenAI, vLLM, enterprise)       │
│  - Knowledge update endpoint (HTTPS)                 │
│  - Update server (HTTPS)                             │
└──────────────────────────────────────────────────────┘
```

**Installer Formats**:
- Windows: MSI or EXE installer with WiX/NSIS
- macOS: DMG with notarized code signature
- Self-update via delta updates downloaded from the update server

## Technology Rationale

See [TECHNOLOGY_SELECTION.md](TECHNOLOGY_SELECTION.md) for detailed evaluations.

Key decisions summarized:
- **Tauri v2 over Electron**: ~10x smaller binary, zero GC pauses, Rust memory safety, native OS integration (Credential Manager, Keychain)
- **React 19 over alternatives**: Largest ecosystem, type safety with TypeScript, mature state management patterns
- **Rust for core**: Memory safety, async performance (Tokio), single-threaded in-process execution, native FFI for MCP transport
- **SQLite + ChromaDB**: Zero-config embedded database for relational data, lightweight embedded vector store for semantic search
- **JSON-RPC for core↔frontend**: Type-safe request/response, well-established pattern (used in OpenHuman)

## Scalability and Extensibility

### Adding a New Skill

1. Create new MCP server binary with tool/resource/prompt definitions
2. Register skill metadata in core skill registry
3. Package as installable unit (download from update server or local)
4. User enables skill; MCP server process is spawned on first use

### Adding a New AI Provider

1. Implement `AiProvider` trait with `chat()`, `embed()`, `model_info()` methods
2. Add provider configuration to settings
3. No changes to core or frontend required

### Adding New Observation Sources

1. Implement observation source trait
2. Register source in observation engine
3. Add permission controls for privacy compliance

## Non-Functional Architecture

### Performance
- Core runs in-process as a single Tokio task — no inter-process overhead for most operations
- Cold startup target: < 2 seconds
- Observation capture: async, non-blocking, bounded by user-configurable rate
- Vector search: indexed, < 500ms latency for typical knowledge queries
- AI response streaming: progressive display in chat UI

### Reliability
- MCP servers isolated in separate processes; crash of one does not affect others
- Core restart recovery: state persisted in SQLite, memory reconstructed on restart
- Auto-restart of failed MCP servers with exponential backoff
- Graceful degradation: if AI provider is unavailable, copilot shows status without crashing

### Security
- All data local-first; no network calls by default
- Credentials never stored in plaintext; use OS credential managers
- Screen observation fully opt-in with granular controls
- AES-256-GCM encryption for sensitive local files
- Full audit logging of all AI interactions and system events
- See [SECURITY_ARCHITECTURE.md](SECURITY_ARCHITECTURE.md) for complete details

## Inspirations from OpenHuman

This architecture draws several architectural concepts from the OpenHuman project:

| OpenHuman Pattern | Wiki Labs AI Copilot Adoption |
|------------------|------------------------------|
| Rust core running in-process as tokio task | ✓ Core runs in-process as tokio task |
| JSON-RPC for frontend ↔ core communication | ✓ Same pattern for all RPC calls |
| Event bus for inter-component async communication | ✓ Tokio-based event bus |
| Memory system (short-term, long-term, subconscious) | ✓ Adapted for session/context memory |
| MCP (Model Context Protocol) for tool integration | ✓ Extended to MCP skill architecture |
| Tauri v2 desktop host | ✓ Same framework, adapted for engineering copilot |
| Domain-driven design (100+ domains in src/openhuman/) | ✓ Adapted for domain-specific skill engines |
| OS keychain integration via `keyring` crate | ✓ Credential Manager / Keychain for secrets |
| AES-256-GCM + Argon2id for encryption | ✓ Same algorithms for local data protection |
| Security policy with sandbox gating | ✓ Permission system for observation and skill tools |
| Local-first SQLite storage | ✓ Same embedded database approach |
| Hybrid search (vector + FTS5) | ✓ Vector + SQLite FTS5 for knowledge retrieval |

## References

- [COMPONENT_DESIGN.md](COMPONENT_DESIGN.md) — Detailed component descriptions and interfaces
- [DATA_MODEL.md](DATA_MODEL.md) — Data entities, schemas, and storage model
- [SECURITY_ARCHITECTURE.md](SECURITY_ARCHITECTURE.md) — Security model and threat analysis
- [MCP_ARCHITECTURE.md](MCP_ARCHITECTURE.md) — MCP skill architecture and tool design
- [TECHNOLOGY_SELECTION.md](TECHNOLOGY_SELECTION.md) — Technology evaluation and rationale
- [REPOSITORY_STRUCTURE.md](REPOSITORY_STRUCTURE.md) — Code organization and conventions
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) — Developer setup and workflows
- [TESTING_STRATEGY.md](TESTING_STRATEGY.md) — Testing approach across all levels