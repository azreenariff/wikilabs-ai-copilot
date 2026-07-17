# OpenHuman Architecture Analysis

**Purpose:** Study the OpenHuman project for architectural concepts relevant to Wiki Labs AI Copilot development. This document covers design patterns and ideas only — no source code is reproduced.

**Date:** 2026-07-16

---

## 1. Project Overview

OpenHuman is an AI assistant desktop application (Windows/macOS/Linux) built as:

- **Rust core** (`src/`): Business logic, domains, JSON-RPC server, memory engine, tool registry, 132+ domain modules.
- **Tauri v2 host** (`app/src-tauri/`): Desktop shell with CEF (Chromium Embedded Framework) for rendering.
- **React frontend** (`app/src/`): UI screens, navigation, routing, component library.
- **Vendored SDKs**: TinyAgents (agent orchestration), TinyCortex (memory engine), TinyJuice (code compression), TinyChannels (messaging), TinyFlows (workflow graphs).

The Rust core runs **in-process** as a tokio task inside the Tauri host — no sidecar process. The frontend communicates with the core via JSON-RPC 2.0 over HTTP and SSE (Server-Sent Events).

---

## 2. Desktop App Architecture (Tauri Setup)

### 2.1 Tauri v2 with CEF

OpenHuman uses Tauri v2 with the **CEF (Chromium Embedded Framework)** runtime instead of the system webview. This is a significant architectural decision:

- **Why CEF**: Consistent cross-platform rendering, access to Chrome DevTools Protocol (CDP) for browser automation and IndexedDB access, native Web Notification interception, and corporate CA/TLS-inspection proxy support on Windows.
- **CEF via vendored fork**: The standard Tauri CEF support is extended with a custom fork (`tauri-runtime-cef`) that adds notification interception and background processing hooks.
- **Security**: CSP is carefully configured to allow IPC communication between the webview and the Tauri host, while restricting external connections.

### 2.2 Single-Instance Guard

A multi-layer single-instance mechanism prevents race conditions:
1. **Mutex-based guard** before CEF init (Win32 `CreateMutexW`)
2. **Tauri single-instance plugin** with deep-link support
3. **CEF cache-lock wait** counts and waits for straggler processes

The deep-link plugin forwards second-launch payloads (e.g., OAuth callbacks) to the primary instance.

### 2.3 Feature Flag Architecture

Both the Rust core and the Tauri host use **compile-time feature flags** for domain gating:
- `voice`, `web3`, `media`, `flows`, `skills`, `meet` — each gates tool surface at compile time AND runtime via a `DomainSet` runtime flag.
- The Tauri host's `Cargo.toml` explicitly forwards default features from the core crate. A CI check (`check-feature-forwarding.mjs`) alerts when this list drifts.
- This creates **two layers of defense**: compile-time (no code compiled in) and runtime (feature flag checked before registration).

### 2.4 Frontend Architecture

- **React + Vite** with TypeScript
- **Routing**: Centralized `AppRoutes.tsx` with auth-aware and platform-aware routes (iOS, desktop, web)
- **State Management**: Custom hooks (`useX402Buy`, `useTinyplaceStream`) and services layer
- **Transport abstraction**: `LanHttpTransport`, `TunnelTransport` (E2E encrypted), `CloudHttpTransport` for multi-device sync
- **Configuration**: Centralized in `utils/config.ts` — never reads `import.meta.env` directly

### 2.5 Build Optimization

- Dependencies compile **without DWARF** in dev/test profiles for faster builds and smaller `target/` (~4.4GB → much less)
- Own crates keep full debuginfo for panic backtraces to file:line
- **CI build-once-then-fanout**: Full E2E is compiled once per OS, then test shards download the artifact
- Two-lane CI: CI Lite (per-changed-area, fast) and CI Full (complete matrix)

---

## 3. AI Runtime / Core Design

### 3.1 In-Process Core Architecture

The core runs as a **tokio task inside the Tauri host process**, eliminating the orphan-sidecar class of bugs:

- **Lifecycle**: `CoreProcessHandle` in Tauri lib coordinates core lifecycle with GUI.
- **Transport**: HTTP + JSON-RPC 2.0 on `127.0.0.1:<port>`, bearer token handed in-memory.
- **Event Bus**: Singleton bus with two surfaces:
  - **Broadcast Pub/Sub**: `tokio::sync::broadcast` for fire-and-forget notifications (many-to-many, decoupled).
  - **Native Request/Response**: Typed one-to-one calls passing raw pointers/Arcs without serialization (zero-copy, in-process only).

### 3.2 JSON-RPC Server

- Built on **Axum** (Rust async web framework)
- **Method dispatching**: Registered controllers per domain namespace
- **SSE streaming**: Real-time event streaming for agent turn progress, memory updates, etc.
- **Structured errors**: Controllers emit structured error envelopes decoded at the transport boundary
- **Health checks, schema discovery, auth routes** as companion endpoints
- **WebSocket upgrade** support for real-time bidirectional communication

### 3.3 Core Runtime Builder Pattern

The runtime uses a **builder pattern** (`CoreBuilder`) that assembles:
- **DomainSet**: Compile-time + runtime feature flags
- **ServiceSet**: Background services (cron, channels, heartbeat, update scheduler)
- **TokenSource**: Authentication/credentials management
- **Shared tokio tuning**: Agent worker stack sized to 16MB (prevents stack overflow from deeply nested agent turns)

### 3.4 Domain-Driven Design

The core is organized into **132+ domain modules**, each a self-contained namespace:

| Domain Category | Examples |
|----------------|----------|
| Agent System | `agent`, `agent_orchestration`, `agent_harness`, `agent_tool_policy` |
| Memory | `agent_memory`, `embeddings`, `memory_tree` |
| Communication | `channels`, `audio_toolkit`, `agent_meetings` |
| Infrastructure | `config`, `encryption`, `keyring`, `cron` |
| Integration | `integrations`, `credentials`, `composio` |
| Safety | `approval`, `cwd_jail`, `emergency_stop` |

Each domain follows a consistent structure: `mod.rs` (doc), `types.rs` (data shapes), `ops.rs` (operations), `schemas.rs` (RPC schemas), `bus.rs` (events), tests alongside implementation.

---

## 4. Plugin / Skill Patterns

### 4.1 Skills System

Skills are **procedural memory** — reusable approaches for recurring task types:

- **Discovery**: Scans directories for `SKILL.md` files with YAML frontmatter
- **Scope Resolution**: User vs Project vs Legacy scope with collision precedence
- **Trust Markers**: Enforced trust boundaries for skill execution
- **Isolated Execution**: Skills run in isolated workers, bodies are not spliced into chat turns
- **Resource Reading**: Bounded payload size (128KB per resource)
- **Remote Catalogs**: Skills can be installed from URLs; registry manages catalogs
- **Lifecycle RPCs**: `skills_list`, `skills_create`, `skills_install_from_url`, `skills_uninstall`, `skills_read_resource`

### 4.2 Model Context Protocol (MCP)

OpenHuman integrates with **5,000+ MCP servers**:

- The `model_context.rs` module in the inference domain handles MCP transport
- MCP provides a standardized interface for tools, prompts, and resources
- Skills and MCP tools are surfaced to agents through a unified catalog
- The agent orchestrator renders `## Installed Skills` and MCP tool lists in system prompts

### 4.3 Vendor SDK Patterns

OpenHuman maintains vendored submodules for core SDKs:

| SDK | Purpose | Pattern |
|-----|---------|---------|
| **TinyAgents** | Agent orchestration, durable state graphs, agent-loop harness | Adapter seam in `tinyagents/` — host-side wraps SDK, SDK-side implements traits |
| **TinyCortex** | Memory engine (store/chunks/tree/retrieval/score/ingest/sync) | Adapter seam — memory tree lives in crate; RPC/tools/sync/gating in host |
| **TinyJuice** | Code compression (AST-aware signature extraction) | Tree-sitter grammars for Rust/TS/Python |
| **TinyFlows** | Workflow engine (typed node graph → validate → compile → run) | Graph-based automation with checkpointing |
| **TinyChannels** | Messaging protocol (WhatsApp, etc.) | Protocol adapters for different channels |

**Vendor Pattern**: Core SDK logic lives in the submodule; the host provides transport, RPC, security gating, and integration. The adapter seam allows testing SDK changes before publishing.

### 4.4 Agent Harness

- **Graph-based execution**: Agent turns run as **checkpointed graphs** (not simple loops)
- **Checkpointing**: Runs pause for human approval, survive restart, resume mid-run
- **Sub-agent fleets**: Specialists spawn up to three levels deep
- **Root-cause reporting**: Stuck agents generate diagnostic reports instead of hanging
- **Replayable journals**: Every run is replayable with per-call cost accounting

---

## 5. Model Abstraction Layer

### 5.1 Provider Architecture

The `inference/provider/` domain abstracts model access:

- **Multiple providers**: OpenAI-compatible APIs, local models, custom endpoints
- **OAuth integration**: OAuth token management for provider authentication
- **Model routing**: Built-in routing picks the right LLM per workload on one subscription
- **Local AI support**: Optional local model inference (llama.cpp/whisper-rs)
- **Voice support**: STT (Whisper with Metal on macOS) and TTS providers

### 5.2 Model Context Management

The `model_context.rs` module handles:
- Model parameter presets per use case
- System prompt composition
- Context window management
- Sentiment/tone detection for adaptive responses

### 5.3 TokenJuice Compression

Before output reaches the model:
- **AST-aware compression**: Uses tree-sitter for language-specific compression
- **Brace-depth fallback**: Language-agnostic compression when treesitter unavailable
- **Token savings**: Up to 80% fewer tokens for the same information
- **Build-time feature**: `tokenjuice-treesitter` feature gates tree-sitter grammars

---

## 6. Local-First Design Patterns

### 6.1 Memory Tree

The memory system is the architectural centerpiece of local-first design:

- **Karpathy-inspired Obsidian wiki**: Memory stored as Markdown files in an Obsidian-compatible vault on the local machine
- **SQLite-backed**: Memory compressed into scored Markdown trees stored in SQLite
- **Compressed context**: Auto-fetch loop pulls data from integrations (Gmail, GitHub, Slack, etc.) every 20 minutes, compresses it into the memory tree
- **Git-backed change ledger**: Memory diffs use git commits, checkpoints as tags, read markers as refs
- **No vector-soup**: Interpretable Markdown structure, not opaque embeddings

### 6.2 Subconscious Processing

- **Background loop**: Diffs the user's world, advances goals, writes morning briefings
- **Persistent state**: Thinks and works after the user stops interacting
- **Goal management**: Long-term goals, per-thread goals, shared kanban board

### 6.3 Local-Only Mode (Privacy Mode)

- **One-switch enforcement**: Flipping Privacy Mode in the UI enforces that **no inference leaves the machine**, enforced in the Rust core
- **On-device encrypted data**: AES-GCM and ChaCha20-Poly1305 encryption for stored data
- **OS-keyring secrets**: Native keyring (Windows Credential Manager, macOS Keychain, Linux libsecret) for secret storage
- **Zeroize on drop**: Master keys and decrypted secret buffers are wiped from memory

### 6.4 Local Data Persistence

- **SQLite bundled**: `rusqlite` with `bundled` feature — no system SQLite dependency
- **Git integration**: Vendored libgit2 for versioning memory changes
- **File system**: Files stored in standard user directories (`directories` crate)
- **Auto-save**: Memory trees auto-save and sync to Obsidian vault

---

## 7. Security Model

### 7.1 Authentication

- **JWT-based auth**: API authentication with JWT tokens
- **OAuth integration**: 100+ OAuth integrations for third-party services
- **Per-launch tokens**: Hex bearer tokens generated per session, shared in-memory between frontend and core
- **Deep-link OAuth**: Second-launch OAuth callbacks forwarded to primary instance

### 7.2 Encryption

- **In transit**: TLS via rustls + Mozilla webpki-roots (macOS/Linux) or native-tls/schannel (Windows)
- **At rest**: AES-GCM and ChaCha20-Poly1305 for encrypted data storage
- **E2E tunnel encryption**: XChaCha20-Poly1305 for cross-device tunnel transport
- **Key management**: x25519 key exchange, HKDF key derivation, ring-based cryptography

### 7.3 Sandboxing

- **Landlock** (Linux): Linux Landlock LSM sandboxing (optional feature)
- **Bubblewrap** (Linux/FreeBSD): `bwrap` sandboxing (optional feature)
- **AppContainer** (Windows): Windows AppContainer process isolation for `cwd_jail`
- **CWD jail**: Working-directory sandboxing to restrict what files the agent can access

### 7.4 Security Controls

- **Approval gates**: Side effects gated behind user approval
- **Trust markers**: Skills and tools must pass trust verification
- **Emergency stop**: Global emergency stop mechanism for runaway agents
- **Permission model**: Tauri permissions and capabilities define what the app can do
- **Sentry integration**: Separate Sentry projects for core, Tauri shell, and frontend — crash reporting with source-line symbolication in release builds

### 7.5 Cross-Platform Security

- **Windows**: AppContainer profiles, ACL editing, COM initialization for UI Automation, registry-based deep-link verification
- **macOS**: Private API usage, TCC grants (managed via .app bundle), native macOS notification framework
- **Linux**: D-Bus notifications, xdg-portal for file dialogs, Landlock/Bubblewrap sandboxing

---

## 8. Comparison: OpenHuman vs Wiki Labs AI Copilot

### 8.1 Fundamental Differences

| Aspect | OpenHuman | Wiki Labs AI Copilot |
|--------|-----------|---------------------|
| **Autonomy** | Autonomous agent with subconscious that continues working after you stop typing | Human-in-the-loop only — never autonomous |
| **Primary domain** | Personal AI assistant (general purpose) | Enterprise engineering copilot (specialized) |
| **Skill focus** | 90,000+ general skills, 5,000+ MCP servers | Domain-specific: OpenShift, Linux, VMware, Nagios, Ansible, databases |
| **Observation** | Memory tree from integrations (email, calendar, messages) | Terminal, browser, clipboard observation |
| **Action model** | Can execute commands and workflows on behalf of user | Provides **recommendations only** — never executes commands |
| **Orchestration** | Multi-agent fleets, sub-agent delegation, graph-based runs | Single agent with human review |
| **Memory** | Persistent memory tree, subconscious processing | Context window management for infrastructure tasks |
| **Privacy** | Privacy Mode (optional local-only) | Enterprise data security (assumed always-local) |

### 8.2 Where They Overlap

- Both are **desktop apps** (Windows/macOS)
- Both use **Tauri** for the desktop shell
- Both use **React** for the frontend
- Both need **Rust core** for performance-critical operations
- Both need **JSON-RPC** for core ↔ frontend communication
- Both need **model abstraction** for LLM routing
- Both benefit from **event bus** for component decoupling
- Both need **skill/plugin** extensibility

---

## 9. Architectural Ideas Worth Adopting

### 9.1 High-Priority Adoptitions

**1. In-Process Core (No Sidecar)**
- Running the Rust core as a tokio task inside the Tauri process eliminates sidecar lifecycle bugs (orphaned processes, race conditions, port conflicts)
- The `CoreProcessHandle` pattern for lifecycle management is directly applicable
- Simpler deployment and cleanup

**2. Domain-Driven Architecture**
- The 132+ domain module pattern provides clear separation of concerns
- Each domain has a consistent internal structure (`mod.rs`, `types.rs`, `ops.rs`, `schemas.rs`, `bus.rs`, tests)
- This scales well and makes onboarding new engineers predictable
- Apply to Wiki Labs: domains could include `openshift`, `linux_sysadmin`, `vmware`, `nagios`, `ansible`, `databases`, `terminal_observation`, `browser_observation`, `clipboard_monitor`

**3. Event Bus Pattern**
- Two-surface event bus (pub/sub + request/response) provides excellent decoupling
- Pub/sub for notifications (e.g., "terminal output received", "clipboard changed")
- Request/response for typed operations (e.g., "get terminal output since X")
- Zero-copy for in-process requests via `Arc` and raw pointers

**4. Feature Flag Architecture (Compile-Time + Runtime)**
- Dual-layer defense: compile-time gates (no code compiled in) + runtime gates (`DomainSet`)
- Prevents silent feature absence (the `voice` domain shipped broken because of a missing feature forward)
- CI check to prevent drift
- Apply to Wiki Labs: `features = ["openshift", "vmware", "nagios", "ansible", "terminal", "browser", "clipboard"]`

**5. Structured JSON-RPC with SSE**
- Axum-based JSON-RPC 2.0 server for core ↔ frontend communication
- SSE for real-time streaming (terminal output, recommendations, progress)
- Structured error envelopes for meaningful error reporting
- Health check and schema discovery endpoints

**6. Transport Abstraction Layer**
- OpenHuman's `Transport` trait (LanHttpTransport, TunnelTransport, CloudHttpTransport) provides a clean abstraction
- For Wiki Labs: could abstract between local process, remote server, or multi-instance sync

**7. Build Optimization Practices**
- Compile dependencies without DWARF in dev/test profiles
- Keep full debuginfo for own crates
- CI build-once-then-fanout pattern for E2E tests

### 9.2 Medium-Priority Adoptitions

**8. Permission/Capability Model**
- Tauri's permission system maps to specific capabilities (read terminal, access clipboard, observe browser)
- Use Tauri capabilities to define what the copilot can observe and recommend

**9. Model Routing**
- Built-in model routing that picks the right LLM per workload
- Local model support for enterprise environments that can't send data to cloud APIs
- Token counting and cost accounting

**10. Compression Before Model**
- TokenJuice pattern: compress tool output before it hits the model
- AST-aware compression could be applied to terminal output, Ansible playbooks, etc.
- Significant token savings for infrastructure engineering context

**11. Observability Integration**
- Sentry for crash reporting with separate projects per layer
- Tracing for structured logging
- Prometheus metrics for monitoring

**12. Vendor SDK Adaptation Pattern**
- Maintain adapter seams between host logic and third-party SDKs
- Allows testing SDK changes without touching host code
- Useful if Wiki Labs integrates any third-party SDKs

### 9.3 Low-Priority / Experimental

**13. Git-Backed Memory**
- Interesting pattern but not directly applicable (Wiki Labs doesn't need persistent memory across sessions in the same way)
- Could be useful for session history versioning

**14. Agent-to-Agent Orchestration**
- Signal-encrypted agent-to-agent communication is overkill for a single-copilot product
- The checkpointed graph execution model could inspire human-in-the-loop approval flows

---

## 10. Concepts That Do NOT Fit Wiki Labs

### 10.1 Autonomous Agent Patterns

- **Subconscious processing**: Background loops that work after the user stops — Wiki Labs is explicitly human-in-the-loop
- **Multi-agent fleets**: Sub-agent delegation and specialist spawning — one copilot per user
- **Workflow automation**: Agent proposes and executes workflows — Wiki Labs provides recommendations, not autonomous execution
- **Goal management with kanban**: Long-term goals that the agent pursues independently

### 10.2 Personal-Assistant Features

- **Memory tree / Obsidian wiki**: Persistent memory of personal data — Wiki Labs context is task-scoped
- **Auto-fetch from integrations**: 20-minute sync loops pulling from email, calendar, messages
- **Meeting agents**: Joining Meet/Zoom/Teams/Webex calls
- **Voice/stt/tts**: While potentially useful, it's secondary to the core engineering workflow
- **Browser automation**: OpenHuman automates browser interactions; Wiki Labs observes but doesn't act
- **Messaging channels** (Telegram, Discord, Slack): OpenHuman connects to 17 channels — Wiki Labs is desktop-local
- **Web3 / crypto wallet**: Not relevant to enterprise engineering

### 10.3 Over-Engineering Risks

- **132+ domain modules**: Wiki Labs likely needs 10-20 domains, not 132. The full DDD structure may be overkill for a smaller product.
- **Vendor SDK complexity**: Five vendored SDKs (TinyAgents, TinyCortex, TinyJuice, TinyFlows, TinyChannels) is significant maintenance overhead. Wiki Labs should start lean.
- **CEF over system webview**: CEF adds build complexity and binary size. For Wiki Labs, the system webview may suffice unless CDP access is critical.
- **Multi-device sync**: Tunnel transport, cloud transport, and cross-device encryption add complexity that may not be needed.

---

## 11. Recommended Architecture for Wiki Labs AI Copilot

Based on the OpenHuman analysis, a recommended architecture would be:

```
┌─────────────────────────────────────────────────────┐
│                    Tauri Desktop                     │
│                                                     │
│  ┌─────────────────┐    ┌─────────────────────────┐ │
│  │   React UI      │    │   Rust Core (in-process) │ │
│  │                 │    │                         │ │
│  │  - Terminal     │    │  - JSON-RPC server      │ │
│  │    observation  │    │  - Domain modules:      │ │
│  │  - Browser      │    │    openshift/           │ │
│  │    observation  │    │    linux/               │ │
│  │  - Clipboard    │    │    vmware/              │ │
│  │    monitor      │    │    nagios/              │ │
│  │  - Rec panels   │    │    ansible/             │ │
│  │  - Settings     │    │    databases/           │ │
│  │  - Skills       │    │    terminal_obs/        │ │
│  │    config       │    │    browser_obs/         │ │
│  │                 │    │    clipboard_mon/       │ │
│  │                 │    │                         │ │
│  └─────────────────┘    │  - Event bus            │ │
│                          │  - Model abstraction    │ │
│                          │  - Skill runtime        │ │
│                          │  - Approval gating      │ │
│                          └─────────────────────────┘ │
│                                                     │
│          JSON-RPC + SSE communication               │
└─────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **In-process Rust core** via tokio task — no sidecar
2. **Lean domain structure** — start with 10-15 domains, expand as needed
3. **JSON-RPC 2.0 + SSE** for core ↔ frontend — proven pattern
4. **Event bus** for terminal/browser/clipboard observation — decoupled from recommendation engine
5. **Skill system** — YAML-frontmatter skills for OpenShift, Linux, VMware, Nagios, Ansible, databases
6. **Model abstraction** — configurable provider routing, local model support for enterprise environments
7. **Human-in-the-loop only** — recommendation output, never command execution
8. **Tauri permissions** — explicitly declare what the copilot can observe (terminal, browser, clipboard)
9. **Build optimization** — skip DWARF for dependencies in dev builds

---

## 12. Summary

### What to Adopt from OpenHuman

- In-process Rust core architecture
- Domain-driven module organization (scaled down)
- Two-surface event bus pattern
- JSON-RPC 2.0 with Axum + SSE
- Feature flag architecture (compile + runtime)
- Skill discovery/execution system (adapt for engineering skills)
- Model abstraction and routing
- Compression before model inference
- Permission/capability model via Tauri
- Build optimization practices
- Observability (Sentry, tracing, Prometheus)

### What to Adapt for Wiki Labs

- **132+ domains** → **10-20 domains** (openshift, linux, vmware, nagios, ansible, databases, terminal_obs, browser_obs, clipboard_mon)
- **Memory tree** → **session-scoped context** (not persistent memory)
- **Autonomous agent** → **human-in-the-loop copilot**
- **Workflow automation** → **recommendation system**
- **90k skills** → **domain-specific engineering skills**
- **CEF webview** → **evaluate system webview vs CEF** (only if CDP needed for browser observation)

### What to Skip

- Autonomous subconscious processing
- Multi-agent orchestration
- Memory tree / Obsidian wiki
- Auto-fetch from integrations
- Meeting agents
- Voice/stt/tts (secondary)
- Web3/crypto
- Messaging channels
- Multi-device sync
- Five vendor SDKs (start lean)
- 132+ domain modules (scale down)

---

*This analysis is based on architectural study of the OpenHuman codebase structure, Cargo configuration, README, AGENTS.md, module organization, and key module interfaces. No source code was reproduced — only architectural concepts and design patterns.*