---
description: "Architecture Decision Records (ADRs) for Wiki Labs AI Copilot — formal records of major architecture decisions, context, options, and rationale."
icon: scale-balanced
---

# Wiki Labs AI Copilot — Architecture Decision Records

## ADR Index

| ADR | Title | Status |
|-----|-------|--------|
| ADR-001 | Desktop Application Framework | ACCEPTED |
| ADR-002 | MCP Architecture — Consolidated Skill Runtime | REVISED |
| ADR-003 | Knowledge Architecture — Vector Database | REVISED |
| ADR-004 | AI Provider Abstraction | ACCEPTED with amendments |
| ADR-005 | Security Model — Key Derivation | REVISED |
| ADR-006 | Platform Support Matrix | NEW |
| ADR-007 | Observation Engine — Terminal Monitoring | NEW |
| ADR-008 | MVP Scope — Phase 1 Definition | REVISED |

---

## ADR-001: Desktop Application Framework

### Status
**ACCEPTED**

### Context
We need a cross-platform desktop application framework for an enterprise engineering copilot. The framework must support Windows and macOS (and ideally Linux) with minimal resource consumption, native OS integration, and a modern UI.

### Decision
Use **Tauri v2** with:
- **Frontend**: React 19 + TypeScript 5.8
- **Core**: Rust (2021 edition) running in-process as a tokio task
- **Communication**: JSON-RPC over HTTP (localhost) + WebSocket for events
- **Build**: Vite 7 for frontend, Cargo for Rust

### Key Requirements Met
| Requirement | Tauri v2 | Electron | .NET MAUI |
|-------------|----------|----------|-----------|
| Binary size | ~5 MB | ~150 MB | ~50 MB |
| RAM at idle | < 50 MB | ~150 MB | ~80 MB |
| GC pauses | None | V8 GC | .NET GC |
| Native OS integration | ✅ (keyring, IPC) | Limited | ✅ (Windows only) |
| Cross-platform | Win/Mac/Linux | Win/Mac/Linux | Win/Mac (poor) |
| Memory safety | ✅ (Rust) | ❌ (JS) | ❌ (.NET) |

### Consequences
**Positive**:
- Significantly smaller binary and memory footprint than Electron
- No GC pauses (critical for real-time observation)
- Rust memory safety prevents entire classes of vulnerabilities
- OS-native WebView eliminates Chromium dependency

**Negative**:
- Requires Rust development team (smaller talent pool than JS/TS)
- WebView compatibility varies between platforms (CSS, JS features)
- Less mature ecosystem than Electron (fewer pre-built components)
- Cross-platform testing requires physical hardware for Windows and macOS

### Rationale
The performance, security, and resource advantages of Tauri v2 over Electron are decisive for an enterprise engineering tool that must run alongside other resource-intensive applications. The Rust ecosystem is mature enough for this use case, and the patterns established by OpenHuman prove the approach works in production.

---

## ADR-002: MCP Architecture — Consolidated Skill Runtime

### Status
**REVISED** (from: Skill-per-process MCP servers)

### Context
The original architecture specified each skill as an independent MCP server process. With 12+ initial skills, this would consume 900 MB–1.8 GB of RAM at idle, which is unacceptable for a desktop tool.

### Decision
Replace the skill-per-process model with a **consolidated multi-skill MCP runtime**:

```
┌──────────────────────────────────────────────────────────────┐
│                Rust Core (in-process)                         │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              MCP Skill Runtime (single process)         │ │
│  │                                                         │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │ │
│  │  │ OpenShift│ │   Linux  │ │  VMware  │ │  MySQL   │  │ │
│  │  │ Module   │ │  Module  │ │  Module  │ │  Module  │  │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │ │
│  │  │ Ansible  │ │  Nagios  │ │  Checkmk │ │  MSSQL   │  │ │
│  │  │ Module   │ │  Module  │ │  Module  │ │  Module  │  │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │ │
│  │  │ PostgreSQL│ │  RHV     │ │ Windows  │ │   EDB    │  │ │
│  │  │  Module  │ │  Module  │ │  Module  │ │  Module  │  │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │ │
│  │                                                         │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐    │ │
│  │  │   Tool       │ │  Resource    │ │  Context     │    │ │
│  │  │   Registry   │ │  Manager     │ │    Bus       │    │ │
│  │  └──────────────┘ └──────────────┘ └──────────────┘    │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  MCP Server Bridge (exposes single MCP server to core) │ │
│  └────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### Module Loading Options
| Option | Mechanism | Pros | Cons | Verdict |
|--------|-----------|------|------|---------|
| **Dynamic linking** | .so/.dll/.dylib | Fast, native, shared memory | Platform-specific build, complex API boundary | ✅ Phase 1 |
| **Wasm modules** | WebAssembly runtime | Sandboxed, portable, hot-reloadable | Performance overhead, limited syscall access | 📅 Phase 2 |
| **Rust workspace crates** | Compile-time cargo workspace | Type-safe, zero overhead | Requires recompile for new skills | ✅ Phase 1 (bundled skills) |

### Consequences
**Positive**:
- RAM consumption: ~50 MB baseline for all skills (vs. 900 MB+ for separate processes)
- Shared tokio runtime, memory allocator, and credential cache
- Simplified process management (no spawning, monitoring, or restarting 12 processes)
- Enables cross-skill context sharing (see ADR-002 Context Bus)

**Negative**:
- A crash in one skill module can affect other skills (mitigated by Wasm isolation in Phase 2)
- Module interface must be carefully designed to prevent cross-contamination
- Dynamic linking requires platform-specific build configuration

### Rationale
The resource consumption of the skill-per-process model is a showstopper. The consolidated runtime reduces RAM usage by 90%+ while maintaining the modularity benefits. For Phase 1 (bundled skills), compile-time cargo workspace modules are the simplest approach. Phase 2 can add Wasm isolation for third-party skills.

### Amendments to Original Architecture
- Remove "MCP Server per Skill" from the architecture
- Remove "MCP Transport Layer" as a separate layer (MCP is now a bridge, not a transport tier)
- Add "Skill Module" as a new concept (a Rust trait that implements tool, resource, and prompt handlers)
- Add "Context Bus" for cross-skill communication

---

## ADR-003: Knowledge Architecture — Vector Database

### Status
**REVISED** (from: ChromaDB embedded mode)

### Context
The knowledge system requires a vector database for semantic search over document embeddings. The original architecture selected ChromaDB in embedded mode. However, ChromaDB's Rust bindings are immature, embedded mode has known memory leaks, and the project loads the entire collection into memory.

### Decision
Replace ChromaDB with **SQLite VSS extension** (primary) or **LanceDB** (fallback).

### Options Evaluated

| Option | Embedded | Rust Bindings | Memory Usage | Maturity | Persistence | Filtering | Verdict |
|--------|----------|---------------|-------------|----------|-------------|-----------|---------|
| **SQLite VSS** | ✅ Yes | ✅ built-in | Low (memory-mapped) | 🟡 Medium | ✅ Single file | ✅ SQL queries | ✅ Phase 1 |
| **LanceDB** | ✅ Yes | ✅ Native | Low (columnar) | 🟢 High | ✅ Directory | ✅ Metadata | 📅 Phase 2 |
| **ChromaDB** | ⚠️ Experimental | ⚠️ Immature | High (full load) | 🟡 Medium | ✅ Single file | ✅ Metadata | ❌ Rejected |
| **FAISS** | ✅ Yes | ⚠️ C bindings | Medium | 🟢 High | ❌ Custom | ❌ None | ❌ Rejected |
| **Qdrant** | ❌ Server | ✅ Native | N/A | 🟢 High | ✅ Server | ✅ Rich | ❌ Rejected |

### Selected: SQLite VSS (Phase 1)

**Architecture**:
```
┌──────────────────────────────────────────────────────────────┐
│                    SQLite Database                            │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  knowledge_docs table (FTS5 indexed)                  │    │
│  │  - id, title, content, workspace_id                   │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  knowledge_chunks table (VSS indexed)                  │    │
│  │  - id, content, embedding VECTOR(384), workspace_id   │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  Hybrid Search:                                       │    │
│  │  - VSS: semantic similarity (vector)                  │    │
│  │  - FTS5: keyword match (full-text)                   │    │
│  │  - Weighted merge: 70% vector + 30% FTS5             │    │
│  └──────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
```

**Embedding Model**: `all-MiniLM-L6-v2` (384-dimensional, ONNX Runtime or llama.cpp)
- Local embedding, no dependency on AI provider
- ~50 MB model file, runs on CPU in < 100ms per query
- Enables offline semantic search

### Consequences
**Positive**:
- Zero additional dependencies (SQLite is already required)
- Memory-mapped file access (no memory pressure)
- VSS + FTS5 in a single database (no dual-write complexity)
- Embedding model is decoupled from AI provider (offline-capable)
- 384-dimensional embeddings are smaller and faster than OpenAI's 1536-dim

**Negative**:
- 384-dimensional embeddings have lower accuracy than 1536-dim for some tasks
- SQLite VSS is a relatively new extension (community maturity)
- Requires ONNX Runtime or llama.cpp for local embedding inference

### Rationale
The architectural benefits of keeping a single database (simplicity, transactional consistency, zero additional dependencies) outweigh the marginal accuracy trade-off of 384-dim vs 1536-dim embeddings. The decoupling of embeddings from the AI provider is a critical architectural improvement that enables offline mode and provider-independent knowledge.

---

## ADR-004: AI Provider Abstraction

### Status
**ACCEPTED with amendments**

### Context
The AI provider abstraction must support multiple providers (OpenAI, vLLM, Ollama, enterprise endpoints) with a consistent interface, tool integration, context management, and structured output.

### Decision
Accept the original AiProvider trait design but **expand it** to include context management, token counting, tool integration, and structured output.

### Revised Trait Definition

```rust
/// Core AI provider trait
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Provider identification
    fn provider_info(&self) -> ProviderInfo;
    fn model_info(&self) -> ModelInfo;

    /// Chat completion
    async fn chat(&self, request: AiRequest) -> Result<AiResponse>;
    async fn chat_stream(&self, request: AiRequest) -> Result<AiResponseStream>;

    /// Embedding
    async fn embed(&self, request: EmbedRequest) -> Result<EmbedResponse>;

    /// Token management
    fn count_tokens(&self, text: &str) -> Result<usize>;
    fn max_context_tokens(&self) -> usize;

    /// Capability detection
    fn supports_tools(&self) -> bool;
    fn supports_streaming(&self) -> bool;
    fn supports_structured_output(&self) -> bool;
    fn supports_vision(&self) -> bool;

    /// Health
    async fn health(&self) -> Result<HealthStatus>;
}

/// Full AI request with context management
pub struct AiRequest {
    pub messages: Vec<ChatMessage>,
    pub system_prompt: String,
    pub tools: Vec<ToolDefinition>,
    pub tool_choice: ToolChoice,
    pub max_tokens: usize,
    pub temperature: f32,
    pub response_format: ResponseFormat,
    pub context_window: ContextWindow,
}

pub struct AiResponse {
    pub message: ChatMessage,
    pub tool_calls: Vec<ToolCall>,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub latency_ms: u64,
}

pub struct ContextWindow {
    pub total_tokens: usize,
    pub max_tokens: usize,
    pub usage_pct: f32,
    pub allocation: ContextAllocation,
}

pub struct ContextAllocation {
    pub system_prompt_pct: f32,     // 20%
    pub observation_context_pct: f32, // 10%
    pub knowledge_context_pct: f32,  // 15%
    pub conversation_history_pct: f32, // 40%
    pub tool_results_pct: f32,       // 10%
    pub padding_pct: f32,            // 5%
}
```

### Consequences
**Positive**:
- Comprehensive interface supports all planned features
- Token counting enables context window management
- Capability detection allows graceful degradation (e.g., no vision → no screenshot OCR)
- Structured output support is essential for reliable tool calling

**Negative**:
- More complex trait implementation for each provider
- Some providers may not support all capabilities (graceful degradation required)
- Token counting accuracy varies between providers

### Rationale
The original trait is too simple for production use. The expanded trait adds approximately 50% more code but prevents a fundamental refactoring that would be required when tool integration, context management, and structured output are needed. The cost of designing the interface correctly now is far lower than the cost of retrofitting it later.

---

## ADR-005: Security Model — Key Derivation

### Status
**REVISED** (from: Argon2id from "OS user credentials")

### Context
The original architecture specified key derivation from "OS user credentials" using Argon2id. The application cannot access the user's OS login password. A new key derivation strategy is required.

### Decision
Use **random master key with OS keychain storage** as the primary key derivation strategy.

### Key Derivation Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Key Derivation Flow                        │
│                                                              │
│  First Launch:                                               │
│  ┌────────────────────┐                                      │
│  │ Generate random     │  OS CSPRNG (getrandom crate)         │
│  │ 256-bit master key  │                                      │
│  └─────────┬──────────┘                                      │
│            │                                                  │
│            ▼                                                  │
│  ┌────────────────────┐                                      │
│  │ Store in OS         │  Windows Credential Manager          │
│  │ Keychain            │  macOS Keychain                      │
│  │                     │  Linux Secret Service (libsecret)    │
│  └─────────┬──────────┘                                      │
│            │                                                  │
│            ▼                                                  │
│  ┌────────────────────┐                                      │
│  │ Derive sub-keys     │  HKDF-SHA256(master_key, context)    │
│  │ via HKDF            │                                      │
│  └─────────┬──────────┘                                      │
│            │                                                  │
│     ┌──────┴──────┐                                          │
│     ▼              ▼                                          │
│  ┌──────────┐ ┌──────────┐                                   │
│  │ Encryption│ │ Memory   │                                   │
│  │ Key       │ │ Auth Key │                                   │
│  └──────────┘ └──────────┘                                   │
│                                                              │
│  Subsequent Launches:                                        │
│  ┌────────────────────┐                                      │
│  │ Read from OS        │  Keychain is unlocked by OS session  │
│  │ Keychain            │  No user interaction required       │
│  └─────────┬──────────┘                                      │
│            ▼                                                  │
│  ┌────────────────────┐                                      │
│  │ Re-derive sub-keys  │  Same HKDF process                   │
│  └────────────────────┘                                      │
└──────────────────────────────────────────────────────────────┘
```

### Key Hierarchy
| Key | Derivation | Purpose | Lifetime |
|-----|-----------|---------|----------|
| Master Key | Random (256-bit) | Root of all derived keys | Permanent (keychain) |
| Data Encryption Key | HKDF-Expand(master, "data-enc") | AES-256-GCM for column/file encryption | Derived on use |
| Memory Auth Key | HKDF-Expand(master, "memory-auth") | Memory integrity verification | Derived on use |
| Session Key | HKDF-Expand(master, "session-N") | Per-session ephemeral encryption | Session lifetime |

### Consequences
**Positive**:
- Key derivation is feasible (all platforms support OS keychain access)
- No user interaction required after first launch (keychain is unlocked by OS session)
- Master key never leaves the keychain (sub-keys are derived in-memory per use)
- HKDF enables purpose-specific sub-keys without exposing the master key

**Negative**:
- First launch requires OS keychain write permission (implied by app installation)
- Keychain availability depends on OS (no keychain on headless Linux, for example)
- Remote desktop sessions may not have keychain access

### Rationale
The OS keychain is the only secure, OS-supported mechanism for storing secrets on a desktop application. The keychain is unlocked by the OS user session, which provides the correct security guarantee: the application's data is protected at rest (when the user is logged out) and available when the user is logged in.

---

## ADR-006: Platform Support Matrix

### Status
**NEW**

### Context
The original architecture excludes Linux desktop support. Enterprise engineers frequently use Linux, and the product's target users (infrastructure and DevOps engineers) are the most likely to use Linux.

### Decision
**Support Linux from the beginning** as a co-equal platform alongside Windows and macOS.

### Platform Support Matrix

| Feature | Windows | macOS | Linux |
|---------|---------|-------|-------|
| **Desktop App** | ✅ Phase 1 | ✅ Phase 1 | ✅ Phase 1 |
| **Installer** | MSI | DMG | AppImage, deb, rpm |
| **WebView** | Edge WebView2 | WKWebView | WebKitGTK |
| **Screen Capture** | DXGI | CGDisplay | X11 (xcb) / Wayland (pipewire) |
| **Terminal Observation** | Windows Terminal API | Terminal.app/iTerm2 | Shell integration |
| **Credential Storage** | Credential Manager | Keychain | Secret Service (libsecret) |
| **Code Signing** | Microsoft SGC | Apple Developer ID | GPG (optional) |
| **Auto Update** | ✅ | ✅ | ✅ |

### Linux-Specific Considerations

**WebKitGTK**:
- Available on all major Linux distributions
- Slightly different rendering behavior than Edge/WKWebView
- Requires WebKit2GTK runtime package

**Screen Capture**:
- X11: `xcb` for screenshot capture, `_NET_ACTIVE_WINDOW` for window detection
- Wayland: `pipewire` portal for screen capture (requires user permission via portal)
- Fallback: `xdotool` for X11, no Wayland fallback (portal is required)

**Credential Storage**:
- `libsecret` via the `keyring` crate (same as macOS Keychain API)
- Requires `libsecret-1-dev` package installed
- Falls back to encrypted file in `~/.local/share/wikilabs/` if keychain unavailable

### Consequences
**Positive**:
- Product addresses the full enterprise engineering market
- Architecture is designed for cross-platform from the start (no retrofitting)
- Linux testing identifies platform-specific issues early

**Negative**:
- Additional development effort for Linux-specific observation (X11/Wayland)
- Three-platform testing matrix (requires more CI resources)
- Linux packaging (AppImage, deb, rpm) adds build complexity
- Wayland's security model complicates screen capture and window detection

### Rationale
Excluding Linux would exclude a significant portion of the target user base and create a blind spot in the product's understanding of engineering workflows. The additional development cost is manageable (primarily screen capture and packaging) and the long-term benefit is substantial.

---

## ADR-007: Observation Engine — Terminal Monitoring

### Status
**NEW**

### Context
The observation engine must monitor terminal activity (commands and output) to provide context-aware AI assistance. The original architecture does not specify HOW this is achieved. Terminal observation is one of the most technically challenging features in the system.

### Decision
Use **shell integration** as the primary terminal observation strategy, with **OCR fallback** for terminals without integration support.

### Approach Comparison

| Approach | Reliability | User Setup | Latency | Platform Support | Privacy | Verdict |
|----------|-------------|-----------|---------|-----------------|---------|---------|
| **Shell integration** | High | Moderate (one-time) | Low (sub-ms) | All platforms | User-controlled | ✅ Primary |
| **PTY hooking** | Low (OS updates) | None | Low | Linux only | Hidden | ❌ Fragile |
| **OCR from screen** | Medium | None | High (500ms+) | All platforms | Limited | 📅 Fallback |
| **Accessibility API** | Medium | OS permission | Medium | macOS/Windows | OS-controlled | 📅 Secondary |

### Shell Integration Design

**Bash** (Linux/macOS):
```bash
# ~/.bashrc or PROMPT_COMMAND
export WIKILABS_PREEXEC=true
preexec() {
    curl -s -X POST "http://127.0.0.1:${WIKILABS_PORT}/terminal/command" \
        -H "Content-Type: application/json" \
        -d "{\"command\": $(printf '%s' "$1" | jq -Rs .), \"cwd\": \"$PWD\", \"shell\": \"bash\"}" &
}
precmd() {
    local exit_code=$?
    local last_output=$(history 1 | cut -d' ' -f4-)
    curl -s -X POST "http://127.0.0.1:${WIKILABS_PORT}/terminal/result" \
        -H "Content-Type: application/json" \
        -d "{\"exit_code\": $exit_code, \"cwd\": \"$PWD\"}" &
}
```

**Zsh** (Linux/macOS):
```zsh
# ~/.zshrc
preexec() { ... }  # Same pattern as bash
precmd() { ... }
```

**PowerShell** (Windows):
```powershell
# $PROFILE
$Global:PSReadLine = @{
    CommandHistory = @{}
}
Register-EngineEvent -SourceIdentifier PowerShell.OnExecuteCommand -Action {
    $command = $EventArgs.CommandLine
    Invoke-RestMethod -Uri "http://127.0.0.1:$env:WIKILABS_PORT/terminal/command" \
        -Method Post -Body (@{command=$command; cwd=$PWD; shell="powershell"} | ConvertTo-Json)
}
```

### Privacy Considerations
- Shell integration is opt-in (user must add the hook to their shell profile)
- The copilot provides a setup wizard that adds the hook automatically
- Users can disable terminal observation at any time via Settings
- All terminal data is stored locally; never transmitted
- Credential detection filters passwords, API keys, and tokens from stored data

### Consequences
**Positive**:
- Reliable, low-latency command capture
- Works across all platforms (bash, zsh, PowerShell)
- User explicitly opts in (privacy-compliant)
- Can capture working directory and exit code

**Negative**:
- Requires user action (one-time shell profile setup)
- Does not capture command output (only commands and exit codes)
- Shell integration is bypassed by some tools (tmux, screen, SSH)
- OCR fallback is needed for complete coverage

### Rationale
Shell integration is the only approach that provides reliable, low-latency, cross-platform terminal observation without requiring fragile OS-level hooks. The one-time setup cost is acceptable for an enterprise tool. Output capture is deferred to the OCR fallback, which is acceptable for Phase 2.

---

## ADR-008: MVP Scope — Phase 1 Definition

### Status
**REVISED** (from: Full product with 12 skills, observation engine, and skill distribution)

### Context
The original architecture defines the full product vision but does not distinguish between MVP and future features. The resulting scope is too large for a single Phase 1 release.

### Decision
**Phase 1 (MVP) is a chat-first AI copilot with 3 bundled skills, no observation engine, and no skill distribution.**

### Phase 1 Scope

| Feature | Included | Rationale |
|---------|----------|-----------|
| AI Chat Interface | ✅ | Core value proposition |
| Workspace Management | ✅ | Organize customer contexts |
| Knowledge Base (manual import) | ✅ | Differentiator from generic AI chat |
| 3 Bundled Skills (Linux, OpenShift, MySQL) | ✅ | Prove the skill pattern |
| Basic AI Provider (OpenAI only) | ✅ | Start simple, add providers later |
| Windows + macOS | ✅ | Covers majority of enterprise desktops |
| Linux | ⬜ Deferred | Phase 2 (but architecture supports it) |
| Observation Engine | ⬜ Deferred | Phase 2 |
| Intent Recognition | ⬜ Deferred | Phase 2 |
| Skill Distribution | ⬜ Deferred | Phase 3 |
| Remaining 9 Skills | ⬜ Deferred | Phase 2-3 |
| Screenshot OCR | ⬜ Deferred | Phase 3 |
| Audit Log Integrity | ⬜ Deferred | Phase 1.5 (not critical for MVP) |
| Prompt Injection Defense | ✅ Phase 1 | Critical for any AI product |

### Phase 1 Architecture (Simplified)

```
┌──────────────────────────────────────────────────────────────┐
│                    Wiki Labs AI Copilot v1 (MVP)               │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  Frontend (React + Tauri v2)                         │    │
│  │  - Chat interface with streaming responses           │    │
│  │  - Workspace selector                                │    │
│  │  - Knowledge management (import, search)              │    │
│  │  - Skill enable/disable                              │    │
│  │  - Settings (AI provider, appearance)                │    │
│  └──────────────────────┬───────────────────────────────┘    │
│                         │ JSON-RPC                           │
│  ┌──────────────────────▼───────────────────────────────┐    │
│  │  Rust Core                                           │    │
│  │  - Event bus, RPC layer, SQLite persistence          │    │
│  │  - AI Provider Abstraction (OpenAI)                  │    │
│  │  - Context Window Manager                            │    │
│  │  - Prompt Injection Defense                          │    │
│  │  - Knowledge System (SQLite VSS + FTS5)              │    │
│  │  - MCP Skill Runtime (consolidated, 3 modules)       │    │
│  │  - Workspace Manager                                 │    │
│  │  - Credential Manager (OS keychain)                  │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                              │
│  Bundled Skills (compiled into core):                        │
│  - Linux Module (disk, memory, process, network tools)       │
│  - OpenShift Module (pod, deployment, cluster tools)         │
│  - MySQL Module (query, performance, backup tools)           │
└──────────────────────────────────────────────────────────────┘
```

### Phase 1 Success Criteria
1. User can create a workspace and configure the customer technology stack
2. User can import knowledge documents (markdown, PDF, text)
3. User can ask questions and receive AI responses with knowledge citations
4. User can invoke skill tools (e.g., "list pods in namespace production")
5. AI tool calls require human confirmation
6. All data stays local; credentials are stored in OS keychain
7. Cold startup < 5 seconds
8. Idle RAM < 150 MB

### Consequences
**Positive**:
- Phase 1 can be delivered in 3-4 months (vs. 12+ months for full scope)
- Core value proposition (AI-assisted troubleshooting) is delivered early
- User feedback from Phase 1 informs Phase 2 observation engine design
- Reduced risk: observation engine is the most complex component

**Negative**:
- No observation engine means the user must describe their problem in chat
- No intent recognition means the AI relies on explicit user context
- 3 skills limits the initial use cases
- Some users may expect the "observe and recommend" behavior from the first release

### Rationale
An MVP that delivers core value quickly and iterates based on user feedback is more likely to succeed than a 12-month development cycle that delivers everything at once. The chat-first approach validates the product concept with minimal investment in the most complex features (observation, intent, distribution). Observation engine developers can work in parallel with the Phase 1 team, targeting Phase 2 integration.