---
description: "Formal architecture review of Wiki Labs AI Copilot — 14-area critical assessment, major findings, and recommended changes prior to implementation."
icon: clipboard-check
---

# Wiki Labs AI Copilot — Architecture Review

## Executive Summary

**Status: CONDITIONAL PASS — with significant changes required**

The architecture as designed is fundamentally sound in its vision and high-level layering. The Tauri v2 + Rust core + React frontend stack is the right choice for a desktop engineering copilot. The local-first, human-in-the-loop, privacy-by-design principles are well-articulated. However, the architecture contains several critical weaknesses in its current form that must be addressed before implementation begins.

**Three critical issues require immediate attention:**

1. **Skill-per-process model is unsustainable.** Running 12+ independent MCP server binaries, each as a separate OS process, will consume 500 MB–1.2 GB of RAM on the engineer's laptop at idle. This violates the "low resource usage" requirement. The architecture needs a consolidated skill runtime.

2. **ChromaDB embedded mode is a false promise.** The Rust bindings for ChromaDB are immature, undocumented, and known to have memory leaks in embedded mode. The architecture relies on a vector database that will not work reliably in production.

3. **Linux exclusion is a strategic error.** Enterprise engineers supporting production environments frequently work on Linux laptops (Fedora, RHEL, Ubuntu). Excluding Linux excludes a significant portion of the target user base and creates a fundamental blind spot for the product.

**Summary of findings by area:**

| Area | Assessment | Severity |
|------|-----------|----------|
| Product Direction | Well-positioned, minor refinements needed | 🟢 Low |
| Desktop Architecture | Tauri v2 correct, but needs Linux | 🟡 Medium |
| Observation Engine | Solid design, critical implementation risks | 🔴 High |
| Intent Recognition | Pragmatic, underspecified implementation | 🟡 Medium |
| MCP Architecture | Skill-per-process is unsustainable | 🔴 High |
| Skill Architecture | Good structure, missing lifecycle management | 🟡 Medium |
| Knowledge Architecture | ChromaDB is the wrong choice | 🔴 High |
| Workspace Architecture | Well-designed, minor improvements | 🟢 Low |
| Security Architecture | Good foundation, critical gaps | 🔴 High |
| AI Runtime | Overly simplistic abstraction | 🟡 Medium |
| Performance | Unacceptable resource estimates | 🔴 High |
| OpenHuman Comparison | Good adoption with caveats | 🟡 Medium |
| MVP Scope | Too ambitious by 2x | 🟡 Medium |
| Roadmap | Missing critical dependencies | 🟡 Medium |

---

## 1. Product Direction Review

### Assessment: 🟢 Well-Positioned

The architecture correctly positions the product as "an AI engineering advisor/copilot" rather than an autonomous agent. The human-in-the-loop principles are embedded in the architecture documentation and component design.

### Findings

**F1.1 — Human-in-the-loop is not technically enforced**
The architecture states that "the AI does NOT autonomously execute production changes" but this is a behavioral guideline, not an architectural constraint. Nothing in the MCP protocol or skill design prevents a future skill from including a tool that writes to production. The `is_command` / `requires_confirmation` flag in the capability definitions is a good start but needs to be enforced at the architecture level, not just the metadata level.

**Recommendation**: Implement a mandatory capability classification system at the core level. The core must enforce that `Command` and `Configuration` capability tools are never invoked without explicit human confirmation. This must be a hard architectural gate, not a convention documented in a markdown file.

**F1.2 — "Trust but verify" is missing**
The architecture assumes the engineer will always read and verify AI recommendations before acting. In practice, engineers under time pressure develop "automation bias" — they accept AI recommendations without verification. The architecture should include a recommendation verification pattern: for critical operations, the AI should be required to explain *why* its recommendation is correct, not just *what* to do.

**Recommendation**: Add a "verification step" to the AI reasoning pipeline for high-risk recommendations. For example, if the AI recommends restarting a production service, it should also provide the verification commands to check that the restart was successful.

### Changes Required
- Add capability enforcement to the core's MCP client (not just metadata)
- Add verification prompting pattern to the AI reasoning engine
- Add "recommendation justification" to the chat message schema

---

## 2. Desktop Application Architecture Review

### Assessment: 🟡 Good Choices, Critical Omission

### Findings

**F2.1 — Tauri v2 is the correct choice**
The evaluation of desktop frameworks is thorough and the selection of Tauri v2 over Electron is well-justified. The 10x binary size reduction, zero GC pauses, and native OS integration are compelling advantages for an enterprise engineering tool.

**F2.2 — Linux exclusion is a strategic error**
The architecture explicitly states "Linux support: Out of scope" for the desktop app. In the enterprise engineering context, this is a critical mistake. The target users are infrastructure and DevOps engineers who frequently work on Linux (Fedora, RHEL, Ubuntu, Debian). Many engineers at consulting firms and enterprise IT departments use Linux as their primary OS.

Even if the initial release targets Windows and macOS, the architecture must be designed with Linux in mind from the start. Retrofitting Linux support later is significantly more expensive than designing for it now.

**Recommendation**: Change the scope to include Linux support from the beginning. Tauri v2 supports Linux (WebKitGTK). The Rust core is already cross-platform. The only additional work is:
- Linux packaging (AppImage, deb, rpm)
- Linux-specific observation (X11/Wayland screen capture, ptrace-free terminal monitoring)
- Linux credential storage (Secret Service / libsecret)

**F2.3 — Tauri v2 WebView limitations for screen capture**
The architecture uses Tauri v2, which runs the UI in a WebView. However, the WebView cannot natively access OS-level screen capture APIs. The screen capture must be implemented in the Rust core using platform-specific APIs (Windows: `DXGI`, macOS: `CGDisplay`, Linux: `X11/Wayland`). This is correctly noted in the component design but the architecture diagram suggests the frontend is the main interaction layer.

**Recommendation**: Clarify in the architecture that all observation capture happens in the Rust core, not the WebView. The frontend only controls observation settings and displays results.

### Changes Required
- Add Linux to the platform support matrix
- Add Linux-specific observation implementations to the architecture
- Clarify observation capture location (Rust core, not WebView)

---

## 3. Observation Engine Review

### Assessment: 🔴 Critical Implementation Risks

The observation engine is the most architecturally risky component in the entire system. It is also the component that provides the most differentiation value. The architecture documentation is well-structured but glosses over several critical implementation challenges.

### Findings

**F3.1 — Active window detection is OS-specific and fragile**
The architecture assumes a simple `get_active_window()` API that returns reliable metadata. In practice:
- On Windows, `SetWindowsHookEx` / `GetForegroundWindow` can miss window focus changes
- On macOS, accessibility permissions are required and can be revoked
- On Linux, Wayland compositors explicitly block window metadata access for security

The architecture does not account for these platform-specific differences.

**Recommendation**: Add a platform abstraction layer for window detection with documented fallback behavior. Define what happens when window metadata is unavailable (e.g., Wayland → fall back to OCR-based window title detection).

**F3.2 — Terminal observation by PTY injection is technically infeasible**
The Terminal Observer component design mentions monitoring terminal windows for command input and output. The architecture does not specify HOW this is achieved. The three practical approaches are:
1. **PTY hooking** (intercept `read`/`write` on pseudo-terminals) — requires kernel-level hooks or `LD_PRELOAD`, which is fragile and may break with OS updates
2. **Shell integration** (shell plugin that records commands) — requires user to install a shell plugin, which is a significant adoption barrier
3. **OCR from terminal screenshots** (read terminal content from screen captures) — the most reliable but high-latency and resource-intensive

The architecture documents none of these approaches.

**Recommendation**: Define a specific terminal observation strategy. The most practical approach for an enterprise engineering tool is a hybrid:
- **Shell integration** for the primary observation path (recommend shell plugin via `PROMPT_COMMAND` on bash, `preexec` on zsh, `PSReadLine` on PowerShell)
- **OCR fallback** for terminals without shell integration
- **Document the trade-offs** and make the approach configurable

**F3.3 — Screenshot-based observation has unacceptable latency for real-time suggestions**
The architecture specifies a 2-second capture interval for screenshots. However, the full pipeline (capture → OCR → context aggregation → intent recognition → skill selection → knowledge retrieval → AI reasoning → response) cannot complete in 2 seconds for a complex query. The user will experience a 5-15 second delay between taking an action and seeing a recommendation.

**Recommendation**: Implement a tiered observation pipeline:
- **Tier 1 (Instant)**: Terminal commands and clipboard content → immediate context analysis
- **Tier 2 (Fast)**: Active window title and app metadata → intent recognition within 1 second
- **Tier 3 (Slow)**: Full screenshot OCR → deep context analysis every 5-10 seconds
- The AI should respond to fast-tier inputs immediately while background-processing slow-tier data

**F3.4 — Credential detection and redaction is underspecified**
The architecture mentions "credential pattern detection and filtering" for clipboard and terminal data but provides no implementation details. What patterns are detected? What is the false positive rate? How is the user notified? What happens on false positive (critical data lost)?

**Recommendation**: Define a credential detection module with:
- Regex-based pattern library (API keys, tokens, passwords, private keys)
- ML-based detection for non-standard credential formats
- User notification and override mechanism for false positives
- Configurable strictness (low/medium/high)
- Audit logging of all detected and redacted content

**F3.5 — No privacy indicator on macOS**
The architecture mentions a "privacy indicator" in the system tray. On macOS, the system tray (menu bar extras) is the correct location, but the app must also handle the macOS Screen Recording permission indicator (orange dot). The app should not display a second indicator — it should integrate with the OS indicator.

**Recommendation**: Remove the custom privacy indicator requirement. Delegate to OS-level indicators:
- macOS: Orange dot when Screen Recording permission is active
- Windows: App icon in the system tray with color-coded status
- Linux: XDG StatusNotifierItem with indicator icon

### Changes Required
- Define platform-specific window detection strategy
- Define terminal observation strategy (shell integration + OCR hybrid)
- Implement tiered observation pipeline
- Define credential detection module with specific patterns
- Delegate privacy indicators to OS

---

## 4. Intent Recognition Architecture Review

### Assessment: 🟡 Pragmatic, Underspecified

### Findings

**F4.1 — The rule-based + ML hybrid is pragmatic but the rule engine is undefined**
The architecture mentions "pattern matching on terminal commands" and "window titles" as the rule engine. However, the rule format, storage, and update mechanism are not specified. How are rules added? Who maintains them? How do they ship with skills?

**Recommendation**: Define the rule engine as part of the skill package. Each skill should include an `intent_patterns.yaml` file that defines the patterns for detecting that technology:
```yaml
patterns:
  - type: terminal_command
    match: "^oc\\s+"  # regex
    weight: 0.8
  - type: window_title
    match: "OpenShift|OKD|CRI-O"
    weight: 0.5
  - type: browser_url
    match: "console\\.openshift\\.com"
    weight: 0.7
```

**F4.2 — The confidence threshold of 0.6 is arbitrary**
No justification is provided for the 0.6 confidence threshold. This value should be determined empirically and should be adjustable per workspace or per skill.

**Recommendation**: Make confidence thresholds configurable and provide a calibration mechanism. The MVP should use a conservative threshold (0.7) and provide a feedback loop for the user to correct false positives.

**F4.3 — No "don't know" state**
The intent engine transitions between intents but has no mechanism for saying "I don't know what the engineer is doing." This is a critical UX failure — the system should be capable of acknowledging uncertainty rather than guessing incorrectly.

**Recommendation**: Add an `Unknown` intent state with a confidence threshold. When confidence is below the threshold, the system should:
- Not display suggestions
- Display a "I'm not sure what you're working on" indicator
- Ask the user for context ("Which technology are you troubleshooting?")

### Changes Required
- Define intent pattern rules as part of skill package metadata
- Make confidence thresholds configurable
- Add `Unknown` intent state with graceful degradation

---

## 5. MCP Architecture Review

### Assessment: 🔴 Critical Design Flaw

### Findings

**F5.1 — Skill-per-process model is architecturally unsustainable**
This is the single most critical issue in the architecture. The initial skill list includes 12 skills, each running as an independent MCP server process. Each process:
- Is a separate Rust binary (5-15 MB each on disk)
- Requires its own tokio runtime (5-10 MB baseline RAM per process)
- Maintains its own MCP protocol connection (stdio pipe, buffered I/O)
- May hold credential decryption state in memory

**Realistic resource estimate: 75–150 MB RAM per skill process at idle** (Rust binary + tokio runtime + MCP client + potential credential state). For 12 skills: **900 MB – 1.8 GB** of RAM at idle. This is unacceptable for a desktop app that must run alongside other engineering tools.

**Recommendation**: Consolidate the skill runtime into a single multi-skill MCP server process:
- One MCP server process that hosts all skill modules
- Skills are loaded as dynamic libraries (.so/.dll/.dylib) or Wasm modules
- The core sends skill discovery and tool calls to the single process
- RAM savings: 90%+ (single process instead of 12)
- Alternative: Use a single Rust process with multiple skill modules behind a shared MCP server, similar to how OpenHuman uses a single tool registry

**F5.2 — MCP Rust implementation is immature**
The MCP protocol specification is a draft standard (2024-11-05). The Rust ecosystem for MCP (mcproto crate) is very new and likely unstable. Building the entire skill architecture on an immature protocol creates significant risk.

**Recommendation**: 
- Abstract the MCP protocol behind a trait/interface so the implementation can be swapped without changing the skill architecture
- Consider using a simpler JSON-RPC protocol internally, with MCP as an optional external interface
- Pin the MCP protocol version and provide a clear upgrade path

**F5.3 — No skill-to-skill communication**
The architecture has no mechanism for skills to collaborate. In practice, problem diagnosis often spans multiple domains (e.g., a slow database query on an OpenShift pod running on VMware). The intent engine selects a primary and secondary skill, but there is no framework for skills to share context or results.

**Recommendation**: Add a "skill context bus" that allows skills to share relevant context:
- Skill A (OpenShift) identifies a pod crash → publishes event to context bus
- Skill B (Linux) receives the event and can analyze the pod's node
- Skill C (MySQL) receives the event and can check if the database was the root cause
- The context bus is read-only — skills can publish context but cannot invoke other skills' tools

**F5.4 — MCP transport options are over-engineered**
The architecture defines three transport options (stdio, HTTP, WebSocket) for MCP servers. For a desktop app where all skills run locally, only stdio is needed. The other transports add complexity without benefit.

**Recommendation**: Remove HTTP and WebSocket transport options from the MVP. Add them only if enterprise requirements for remote skill servers emerge. This simplifies the MCP manager significantly.

### Changes Required
- **CRITICAL**: Replace skill-per-process with consolidated multi-skill runtime
- Abstract MCP protocol behind a swappable interface
- Add skill context bus for cross-skill collaboration
- Strip unnecessary transport options from MVP

---

## 6. Skill Architecture Review

### Assessment: 🟡 Good Structure, Missing Lifecycle

### Findings

**F6.1 — SKILL.md and metadata.json duplication**
The architecture defines both a `SKILL.md` manifest and a `metadata.json` file for each skill. These contain overlapping information (name, version, tools, resources). This creates a maintenance burden and a source of truth problem.

**Recommendation**: Eliminate `metadata.json`. Use `SKILL.md` as the single source of truth. The YAML frontmatter in `SKILL.md` contains all structured metadata. The `metadata.json` was redundant from the start.

**F6.2 — No skill versioning strategy**
The architecture defines a `version` field in skill metadata but provides no strategy for:
- Semantic versioning (major.minor.patch)
- Breaking vs. non-breaking changes
- Backward compatibility guarantees
- Version migration

**Recommendation**: Adopt a formal semantic versioning policy for skills:
- **Major**: Breaking changes to tool interfaces (required parameters, removed tools)
- **Minor**: New tools, resources, or workflows (backward compatible)
- **Patch**: Knowledge updates, bug fixes, documentation (fully compatible)
- Version compatibility should be enforced by the core's MCP client

**F6.3 — Missing skill testing framework**
The architecture mentions `tests/` in the skill package structure but does not define a testing strategy. Skills run arbitrary commands in the engineer's environment. Without testing, a buggy skill could:
- Execute incorrect commands
- Display incorrect troubleshooting steps
- Access wrong systems

**Recommendation**: Define a skill testing framework:
- Unit tests for tool handlers (mocked environment)
- Integration tests with a sandboxed MCP server
- Command output validation (expected vs. actual output)
- Knowledge content validation (correctness, formatting)
- Security scanning (no hardcoded credentials, no dangerous default commands)

**F6.4 — Skill distribution is underspecified**
The architecture mentions "download from update server" but does not define:
- Package format (zip, compressed tarball, custom format)
- Checksum verification
- Code signing requirements
- Update channel (stable, beta, enterprise)
- Offline installation (USB key, network share)

**Recommendation**: Define a complete skill distribution pipeline:
- Package format: `.wls` (Wiki Labs Skill) — signed zip archive
- Distribution: via update server (preferred), local file import (offline)
- Verification: Ed25519 signature on every package
- Update channels: `stable` (default), `enterprise` (verified by enterprise admin)
- Dependency resolution: skills may depend on other skills or shared libraries

### Changes Required
- Eliminate metadata.json, consolidate into SKILL.md
- Define semantic versioning policy for skills
- Define skill testing framework
- Define skill distribution pipeline

---

## 7. Knowledge Architecture Review

### Assessment: 🔴 ChromaDB is the Wrong Choice

### Findings

**F7.1 — ChromaDB embedded mode is not production-ready for this use case**
The architecture selects ChromaDB in embedded mode as the vector database. This is a critical risk:

1. **Rust bindings are immature**: The `chromadb` crate has limited documentation, a small community, and known issues with memory management in embedded mode
2. **Embedded mode is experimental**: ChromaDB's embedded mode is not the primary use case (server mode is). The embedded mode has known memory leaks that are not prioritized for fix
3. **No Rust-native client**: ChromaDB's primary client is Python. The Rust client is a thin wrapper that may not expose all features
4. **Performance issues**: ChromaDB embedded mode loads the entire collection into memory on startup, which is problematic for large knowledge bases

**Recommendation**: Replace ChromaDB with one of:
- **Primary choice: SQLite VSS extension** — Zero additional dependencies, embedded in SQLite, supports vector similarity search (`sqlite-vss` or `libsql` with vector support). This eliminates the separate database entirely.
- **Alternative: LanceDB** — Rust-native, embedded, columnar vector database designed for the agent/ML use case. Well-maintained Rust bindings.
- **Fallback: FAISS with custom persistence layer** — More work but fully controlled and production-proven in the ML community.

**F7.2 — Embedding model coupling to AI provider**
The architecture ties the embedding model to the configured AI provider. This means:
- Switching AI providers re-embeds the entire knowledge base
- Local-only knowledge search requires a cloud provider for embeddings
- Offline mode cannot perform semantic search

**Recommendation**: Decouple the embedding model from the AI provider:
- Use a local embedding model for knowledge indexing (e.g., `all-MiniLM-L6-v2` via `llama.cpp` or ONNX runtime)
- The AI provider is only used for chat/response generation
- This enables offline semantic search and provider-independent knowledge

**F7.3 — No knowledge deduplication strategy**
The architecture provides no mechanism for detecting duplicate or near-duplicate knowledge documents. Engineers who import similar documents (e.g., multiple versions of the same SOP) will create redundant search results.

**Recommendation**: Add a content-based deduplication step to the knowledge import pipeline:
- Compute SHA-256 hash of document content
- Detect near-duplicate documents via vector similarity
- Flag duplicate documents for user review
- Merge or skip duplicate content

**F7.4 — No knowledge quality scoring**
All knowledge documents are treated equally in search results. In practice, some documents are higher quality (official vendor docs, validated SOPs) than others (user notes, outdated references).

**Recommendation**: Add a knowledge quality scoring system:
- **Source authority**: High (vendor docs), Medium (SOPs), Low (user notes)
- **Freshness**: Recent documents ranked higher
- **Usage signals**: Frequently referenced documents ranked higher
- **User feedback**: Explicit upvote/downvote on knowledge results

### Changes Required
- **CRITICAL**: Replace ChromaDB with SQLite VSS or LanceDB
- Decouple embedding model from AI provider
- Add knowledge deduplication
- Add knowledge quality scoring

---

## 8. Workspace Architecture Review

### Assessment: 🟢 Well-Designed, Minor Improvements

### Findings

**F8.1 — Workspace "stack" model is too rigid**
The `workspaces_stacks` table defines a fixed list of technologies per workspace. In practice, enterprise environments are complex and heterogeneous. A customer may have OpenShift on vSphere, running on RHEL, with MySQL and PostgreSQL. The stack model does not capture these relationships.

**Recommendation**: Replace the flat stack list with a graph-based environment model:
- Represent the customer environment as a directed graph of systems
- Each node is a system (OpenShift cluster, VMware vCenter, MySQL instance)
- Each edge is a relationship (runs_on, connects_to, manages)
- This enables the AI to reason about system dependencies

**F8.2 — Workspace switching UX is undefined**
The architecture defines workspace CRUD operations but does not address:
- How does the user switch between workspaces?
- What happens to in-progress conversations?
- How does the observation engine know which workspace is active?
- Can the AI suggest workspace switches based on detected context?

**Recommendation**: Define a workspace switching protocol:
- Workspace switching is an explicit user action (dropdown or keyboard shortcut)
- On workspace switch, the current conversation is saved and a new session begins
- The observation engine tags all observations with the active workspace ID
- AI can suggest workspace switches ("This looks like an OpenShift issue. Switch to the Acme Corp workspace?")

**F8.3 — No workspace export/import capability**
Enterprise engineers need to share workspace configurations with colleagues. The architecture provides no mechanism for exporting or importing workspaces.

**Recommendation**: Add workspace export/import as a post-MVP feature:
- Export: workspace configuration + skill settings + knowledge association (not knowledge documents themselves)
- Import: workspace structure from a JSON file
- Knowledge documents are too large and potentially sensitive to include in workspace exports

### Changes Required
- Replace flat stack model with graph-based environment representation
- Define workspace switching UX
- Add workspace export/import (post-MVP)

---

## 9. Security Architecture Review

### Assessment: 🔴 Good Foundation, Critical Gaps

### Findings

**F9.1 — Prompt injection is not addressed**
The architecture has no mention of prompt injection prevention. This is a critical security gap for a product that sends user data (screen content, terminal output, clipboard) to an AI provider. An attacker who controls any data visible on the engineer's screen could potentially inject malicious prompts into the AI system.

**Recommendation**: Implement a multi-layer prompt injection defense:
1. **Input normalization**: Strip control characters and known injection patterns from observation data before it reaches the AI
2. **Context separation**: Clearly separate user input (chat messages) from observed data (screen, terminal) in the AI prompt
3. **Output validation**: Scan AI responses for command injection (e.g., the AI suggesting a command that exfiltrates data)
4. **Rate limiting**: Prevent rapid injection attempts through observation data
5. **User notification**: Alert the user if injection patterns are detected

**F9.2 — MCP server process isolation is insufficient**
The architecture states that MCP servers run as the same user. This means:
- A compromised MCP server has full access to the engineer's files
- A compromised MCP server can access the OS keychain
- There is no sandbox between skills

With the recommendation to consolidate skills into a single process (F5.1), the isolation problem becomes even more acute.

**Recommendation**: Implement process-level sandboxing for MCP servers:
- **Windows**: Use AppContainer or Integrity Levels to restrict MCP server permissions
- **macOS**: Use sandbox entitlements (com.apple.security.*) for skill processes
- **Linux**: Use Bubblewrap or Landlock (as OpenHuman does) for skill process sandboxing
- Each skill should only have access to the minimum required set of OS resources

**F9.3 — Key derivation from "OS user credentials" is undefined**
The architecture states: "Key derivation: User's login session key (Argon2id from OS user credentials)." This is not a defined credential. The user's OS login password is not accessible to the application (and should not be). This is a showstopper for the encryption design.

**Recommendation**: Define a clear key derivation strategy:
- **Option A (Recommended)**: Generate a random master key on first launch, encrypted and stored in the OS keychain. The keychain is unlocked by the OS user session, which is managed by the OS.
- **Option B**: Derive key from the AI provider API key (user provides it, it's already in the application). This ties data encryption to the AI provider, which is not ideal.
- **Option C**: Derive key from a user-provided "master password" at application startup (password manager pattern). Highest security but worst UX.

**F9.4 — Audit logs are not tamper-proof**
The audit log is stored in SQLite and described as "immutable." SQLite is not immutable — anyone with write access to the file can modify the database. The architecture provides no mechanism for audit log integrity verification.

**Recommendation**: Implement audit log integrity:
- **Option A**: Append-only log file with periodic hash chain (each entry includes the hash of the previous entry)
- **Option B**: Cryptographic signature on audit log entries (Ed25519 signature per entry or per batch)
- **Option C**: SQLite table with write-once triggers (can be bypassed but provides a deterrent)

**F9.5 — No data classification enforcement**
The data classification table (Public, Internal, Confidential, Restricted) is a policy document, not an architectural constraint. Nothing in the architecture prevents writing Confidential data to a non-encrypted column.

**Recommendation**: Enforce data classification at the database layer:
- Separate encrypted tables for Confidential/Restricted data
- Type-safe Rust structs that enforce encryption at the data access layer
- Compile-time checks: `ConfidentialString` vs `PublicString` types
- Database views that automatically mask or omit sensitive columns

### Changes Required
- **CRITICAL**: Add prompt injection defense layer
- Add MCP server sandboxing (per-platform)
- **CRITICAL**: Fix key derivation strategy (cannot use OS login password)
- Implement audit log integrity verification
- Enforce data classification at the type level

---

## 10. AI Runtime Review

### Assessment: 🟡 Overly Simplistic Abstraction

### Findings

**F10.1 — AiProvider trait is too simple**
The `AiProvider` trait defines `chat()`, `chat_stream()`, `embed()`, `info()`, and `health()`. This is insufficient for a production copilot:

- **No context management**: The architecture does not define how the AI provider manages the conversation context window
- **No token counting**: The provider trait should expose token counting for prompt management
- **No tool integration**: The trait does not define how tools (MCP skills) are passed to the AI
- **No structured output**: The trait does not support structured output (JSON schema) for tool calls
- **No system prompt management**: The trait does not define how system prompts are constructed

**Recommendation**: Expand the AiProvider trait:
```rust
trait AiProvider: Send + Sync {
    async fn chat(&self, request: AiRequest) -> Result<AiResponse>;
    async fn chat_stream(&self, request: AiRequest) -> Result<AiStream>;
    async fn embed(&self, request: EmbedRequest) -> Result<EmbedResponse>;
    fn count_tokens(&self, text: &str) -> Result<usize>;
    fn model_info(&self) -> ModelInfo;
    fn supports_tools(&self) -> bool;
    fn supports_streaming(&self) -> bool;
    fn supports_structured_output(&self) -> bool;
    async fn health(&self) -> Result<HealthStatus>;
}

struct AiRequest {
    messages: Vec<ChatMessage>,
    system_prompt: String,
    tools: Vec<ToolDefinition>,  // MCP tool definitions
    tool_choice: ToolChoice,     // auto, required, none
    max_tokens: usize,
    temperature: f32,
    response_format: Option<ResponseFormat>,  // text, json, json_schema
    context_window: ContextWindow,  // current context management
}
```

**F10.2 — No context window management strategy**
The architecture does not address how the system manages the AI provider's context window. This is a critical omission:

- How are conversation history, observation data, knowledge results, and system prompts all packed into the context window?
- What is the token budget for each component?
- What happens when the context window is full?
- How are tool call results managed?

**Recommendation**: Implement a context window manager:
- **Token budget allocation**: System prompt (20%) | Observation context (10%) | Knowledge results (15%) | Conversation history (40%) | Tool results (10%) | Padding (5%)
- **Sliding window**: Older conversation messages are compressed or dropped as the window fills
- **Observation snapshots**: Only the most recent observation is included; older observations are summarized
- **Knowledge results**: Only the top-k most relevant results are included
- **Tool result pruning**: Large tool results are summarized before being included in the next turn

**F10.3 — Local models are impractical for the observation use case**
The architecture suggests local models (via Ollama) as an option for air-gapped environments. However, the observation engine relies on vision models (OCR, screenshot analysis) and intent recognition, which require capable models. Running a vision-capable model locally (e.g., LLaVA, Qwen-VL) requires a GPU with 8-16 GB VRAM, which is not a standard engineer laptop configuration.

**Recommendation**: Be honest about the local model limitation:
- **Local models are only viable for text-only chat** (using small models like Llama 3.1 8B or Qwen 2.5 7B)
- **Vision and observation features require cloud AI providers** or a GPU-equipped workstation
- Document this limitation clearly in the architecture
- For enterprise air-gapped deployments, recommend a shared GPU server running vLLM

### Changes Required
- Expand AiProvider trait with context management, tools, and structured output
- Implement context window manager with token budget allocation
- Be honest about local model limitations for vision features

---

## 11. Performance Review

### Assessment: 🔴 Resource Estimates Are Unacceptable

### Findings

**F11.1 — 12 skill processes will consume 500MB–1.2GB RAM**
As discussed in F5.1, the skill-per-process model is the most significant performance issue. A consolidated skill runtime (single process with dynamic skill modules) addresses this.

**F11.2 — ChromaDB memory usage is unbounded**
ChromaDB's embedded mode loads the entire collection into memory. For a knowledge base with thousands of documents, this could consume 200-500 MB. The recommended replacement (SQLite VSS or LanceDB) uses memory-mapped files and only loads search results into memory.

**F11.3 — Cold startup target of 2 seconds is unrealistic**
The architecture targets a 2-second cold startup. Realistic startup for the current architecture:
- Tauri v2 WebView initialization: 0.5-1 second
- Rust core initialization: 0.2-0.5 seconds
- SQLite connection and schema migration: 0.1-0.3 seconds
- Workspace and skill loading from disk: 0.2-0.5 seconds
- MCP server process spawning (lazy or eager): 0.5-2 seconds per skill

For the consolidated skill architecture, a realistic cold startup target is 3-5 seconds, with background skill loading completing within 10 seconds.

**F11.4 — Observation at 2-second intervals is too aggressive**
The architecture specifies a 2-second screenshot capture interval. Each capture:
- Screenshot capture: 10-50ms (platform dependent)
- OCR processing: 100-500ms (depends on image size and OCR engine)
- Intent analysis: 50-200ms
- Context aggregation: 10-50ms
- Total per cycle: 170-800ms

At 2-second intervals, the observation engine could consume 10-40% CPU continuously. This is unacceptable for a background tool.

**Recommendation**: Implement adaptive observation intervals:
- Idle detection: No significant context change → increase interval to 10 seconds
- Activity detection: User switches app, types command → decrease interval to 1-2 seconds
- Rate limiting: Never exceed 25% CPU utilization for observation
- User-configurable: Allow the user to set preferred observation aggressiveness

### Changes Required
- Consolidate skill processes (addresses 500MB+ RAM)
- Replace ChromaDB with memory-efficient alternative
- Set realistic cold startup target (3-5 seconds)
- Implement adaptive observation intervals

---

## 12. OpenHuman Architecture Comparison

### Assessment: 🟡 Good Adoption with Caveats

### Findings

**F12.1 — Concepts worth adopting — already done well**
- **Local-first design**: ✓ Correctly adopted
- **Tauri v2 + Rust core**: ✓ Correctly adopted
- **JSON-RPC pattern**: ✓ Correctly adopted
- **Event bus for inter-component communication**: ✓ Correctly adopted
- **OS keychain integration**: ✓ Correctly adopted
- **Hybrid search (vector + FTS5)**: ✓ Correctly adopted

**F12.2 — OpenHuman's memory model is over-engineered for this use case**
The architecture adapts OpenHuman's short-term, long-term, and subconscious memory model. OpenHuman's memory system is designed for a persistent AI assistant that maintains state across days of interaction. For a task-specific engineering copilot, this level of memory complexity is unnecessary:
- **Short-term memory**: Session context (useful, keep)
- **Long-term memory**: Workspace-specific patterns (useful, keep)
- **Subconscious memory**: Background pattern recognition (unnecessary for MVP, adds complexity)

**Recommendation**: Keep short-term and long-term memory. Remove subconscious memory from the MVP. The subconscious memory concept adds significant complexity (embeddings, periodic consolidation, importance scoring) without clear value for task-specific engineering assistance.

**F12.3 — OpenHuman's skill model is too complex**
OpenHuman's skill system is designed for a crypto community assistant with 100+ domains. It uses a metadata-only skill registry with tool injection into agent prompts. The Wiki Labs architecture proposes a more structured approach (MCP servers with packaged tools, resources, workflows). This is the right direction, but the architecture should not over-borrow from OpenHuman's complexity.

**Recommendation**: Adopt the simpler MCP-based skill model from the Wiki Labs architecture, not the OpenHuman skill registry pattern. The OpenHuman pattern is optimized for a different use case (many small skills, heavy tool injection) and is not appropriate for enterprise engineering skills.

### Changes Required
- Remove subconscious memory from MVP
- Keep the MCP-based skill model (not OpenHuman's skill registry)

---

## 13. MVP Review

### Assessment: 🟡 Scope Is 2x Too Large

### Findings

**F13.1 — MVP should not include all 12 skills**
The architecture lists 12 initial skills. For an MVP, this is excessive. Building 12 MCP servers creates 12x the work for tool handler implementation, testing, packaging, and documentation.

**Recommendation**: MVP Phase 1 should include only 3 skills:
1. **Linux** — The most universally applicable skill for enterprise engineers
2. **OpenShift** — The flagship skill (differentiator for the product)
3. **MySQL** — Representative database skill (proves the pattern for database skills)

Additional skills follow in Phase 2 and beyond.

**F13.2 — Full observation engine is not MVP-required**
The observation engine (screen capture, OCR, terminal monitoring, clipboard) is the most complex component. Building it for MVP adds months of development time for a feature that:
- Requires OS-level permissions that users may not grant initially
- Has significant privacy and compliance implications
- Is not strictly necessary for the core value proposition (AI-assisted troubleshooting)

**Recommendation**: MVP should focus on the **chat-first interaction model**:
- User describes their problem in chat
- AI uses context from the conversation + workspace configuration + knowledge base
- AI provides recommendations, commands, and troubleshooting steps
- Observation engine is added in Phase 2 as an enhancement

**F13.3 — MVP should be a working product, not a platform**
The architecture defines skill creation, distribution, and installation as core features. For MVP, the skills should be bundled with the application, not distributed as installable packages.

**Recommendation**: For MVP:
- Skills are compiled into the application binary
- No skill store, no skill download, no skill update mechanism
- The skill management UI is reduced to enable/disable toggles
- Skill distribution infrastructure is Phase 3

### MVP Recommendation

**Phase 1 (MVP — 3 months)**:
- Chat-based AI copilot
- 3 bundled skills (Linux, OpenShift, MySQL)
- Knowledge base import (manual file import)
- Workspace management
- Basic AI provider abstraction (OpenAI only)
- No observation engine
- No skill distribution
- Single user, no multi-user
- Windows + macOS (Linux if feasible)

**Phase 2 (3 months)**:
- Observation engine (terminal + app context, no screenshot OCR)
- 3 additional skills (VMware, Ansible, Windows)
- Local AI provider support (Ollama, text-only)
- Knowledge base with hybrid search
- Intent recognition (basic, rule-based)
- Skill distribution framework

**Phase 3 (3 months)**:
- Screenshot OCR and vision-based context
- Remaining skills (Nagios, Checkmk, EDB PostgreSQL, MS SQL Server, RHV)
- Skill store and update mechanism
- Audit log export and compliance features
- Multi-workspace management
- Linux support

---

## 14. Development Roadmap Review

### Assessment: 🟡 Missing Critical Dependencies

### Findings

**F14.1 — No internal dogfooding phase**
The roadmap has no phase where the Wiki Labs team uses the product themselves before releasing to customers. For a product that is an "engineering copilot," the most important test is whether the engineering team finds it useful.

**Recommendation**: Add a **4-week dogfooding phase** after Phase 1 MVP:
- Wiki Labs engineers use the copilot in their daily work
- Bug reports, UX feedback, and feature requests are collected
- The observation engine is tested against real engineering workflows
- Skills are validated against real customer scenarios

**F14.2 — No explicit testing phase**
The roadmap does not define when testing happens. The CI/CD pipeline is well-defined in the technology selection, but there is no milestone for "testing complete."

**Recommendation**: Add explicit testing milestones:
- **Unit test coverage**: 80%+ before Phase 1 MVP release
- **Integration test suite**: All MCP servers have integration tests before Phase 2
- **E2E test suite**: Core user workflows tested end-to-end before Phase 1 release
- **Security audit**: Third-party security review before Phase 1 release
- **Performance benchmarks**: Baseline performance measurements before Phase 1 release

**F14.3 — No user research phase**
The roadmap assumes the architecture is correct without user validation. The observation engine, intent recognition, and suggestion panel are all based on assumptions about how engineers work.

**Recommendation**: Add a **user research phase** before Phase 1:
- Interview 5-10 enterprise engineers about their workflow
- Show them mockups of the copilot interface
- Validate the observation engine approach (would they use it?)
- Validate the skill concept (what skills are most valuable?)
- Use findings to adjust the Phase 1 scope

**F14.4 — No rollback or disaster recovery plan**
The roadmap does not address what happens when an update breaks the application. Desktop applications that cannot be rolled back or recovered create significant risk for enterprise users.

**Recommendation**: Add infrastructure for:
- **Versioned application data**: The database schema is versioned; old versions can be rolled back to
- **Update channel**: Stable (automatic) and Beta (opt-in) channels
- **Rollback mechanism**: Last-known-good version is preserved on disk
- **Crash recovery**: Automatic recovery on crash, with data integrity verification

### Changes Required
- Add 4-week dogfooding phase after Phase 1
- Add explicit testing milestones
- Add user research phase before Phase 1
- Add rollback and disaster recovery plan

---

## Summary of Critical Changes Required

| Priority | Change | Area | Effort |
|----------|--------|------|--------|
| 🔴 P0 | Replace skill-per-process with consolidated multi-skill runtime | MCP Architecture | High |
| 🔴 P0 | Replace ChromaDB with SQLite VSS or LanceDB | Knowledge Architecture | Medium |
| 🔴 P0 | Fix key derivation strategy (cannot use OS login password) | Security | Low |
| 🔴 P0 | Add prompt injection defense layer | Security | Medium |
| 🟡 P1 | Add Linux support | Desktop Architecture | Medium |
| 🟡 P1 | Define terminal observation strategy (shell integration) | Observation Engine | High |
| 🟡 P1 | Implement tiered observation pipeline | Observation Engine | High |
| 🟡 P1 | Add capability enforcement at the core level | Product Direction | Low |
| 🟡 P1 | Add context window manager | AI Runtime | Medium |
| 🟡 P1 | Reduce MVP scope to 3 skills, chat-first, no observation | MVP | — |
| 🟢 P2 | Eliminate metadata.json, consolidate into SKILL.md | Skill Architecture | Low |
| 🟢 P2 | Add skill verification with Ed25519 signatures | Security | Medium |
| 🟢 P2 | Add audit log integrity verification | Security | Low |
| 🟢 P2 | Add workspace export/import | Workspace | Low |
| 🟢 P2 | Add cross-skill context bus | MCP Architecture | Medium |