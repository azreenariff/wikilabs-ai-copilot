---
description: "Risk register for Wiki Labs AI Copilot — identified risks with impact assessment, probability scoring, and mitigation strategies."
icon: triangle-exclamation
---

# Wiki Labs AI Copilot — Architecture Risk Register

## Risk Assessment Methodology

| Dimension | Scale | Description |
|-----------|-------|-------------|
| **Impact** | Critical (5) | Product failure, security breach, data loss |
| | High (4) | Major feature blocked, significant user impact |
| | Medium (3) | Feature delayed, moderate user impact |
| | Low (2) | Minor degradation, cosmetic issue |
| | Negligible (1) | Inconvenience, no real impact |
| **Probability** | Certain (5) | >90% chance of occurring |
| | Likely (4) | 50-90% |
| | Possible (3) | 20-50% |
| | Unlikely (2) | 5-20% |
| | Rare (1) | <5% |
| **Exposure** | Impact × Probability | Higher = more urgent |

---

## Risk Register

---

### RISK-001: Skill-per-process Architecture Causes Unacceptable RAM Usage

| Field | Value |
|-------|-------|
| **ID** | RISK-001 |
| **Category** | Architecture / Performance |
| **Description** | Each skill runs as an independent MCP server process. With 12 initial skills, total RAM consumption is estimated at 900 MB–1.8 GB at idle, before any actual work. |
| **Root Cause** | Design decision to use separate OS processes for each MCP skill server, each with its own tokio runtime, memory allocator, and binary. |
| **Impact** | Critical (5) — Product is unusable on standard engineer laptops (8-16 GB RAM). User will not tolerate a background app consuming 1+ GB. |
| **Probability** | Certain (5) — Architecture is defined; this is a direct consequence of the current design. |
| **Exposure** | **25** (CRITICAL) |
| **Mitigation** | See RISK-001 Mitigation in REVISED_ARCHITECTURE.md: Consolidate all skills into a single MCP server process with dynamic module loading. Target: < 50 MB baseline for all skills combined. |
| **Contingency** | If consolidation is not feasible, implement lazy skill loading (only load 2-3 most relevant skills) and document the 12-skill limitation. |
| **Owner** | Architecture Lead |
| **Timeline** | Resolve before any implementation begins |

---

### RISK-002: ChromaDB Embedded Mode Fails in Production

| Field | Value |
|-------|-------|
| **ID** | RISK-002 |
| **Category** | Technology Selection / Stability |
| **Description** | ChromaDB's embedded mode has immature Rust bindings, documented memory leaks, and is not the primary use case for the project. The vector database is a critical dependency for knowledge search. |
| **Root Cause** | Selection of ChromaDB over more mature alternatives (SQLite VSS, LanceDB, FAISS) |
| **Impact** | Critical (5) — Knowledge search is a core feature. If the vector database is unreliable, the entire AI copilot loses its key differentiator. |
| **Probability** | Likely (4) — The Rust `chromadb` crate has < 500 GitHub stars, limited documentation, and known issues with embedded mode memory management. |
| **Exposure** | **20** (CRITICAL) |
| **Mitigation** | Replace ChromaDB with SQLite VSS extension (vector support built into SQLite, zero additional dependencies) or LanceDB (Rust-native, embedded, memory-mapped). |
| **Contingency** | Fall back to keyword-only search (FTS5) if vector search is unavailable. This degrades search quality but keeps the product functional. |
| **Owner** | Knowledge Team Lead |
| **Timeline** | Resolve before knowledge import pipeline is implemented |

---

### RISK-003: Key Derivation Strategy is Infeasible

| Field | Value |
|-------|-------|
| **ID** | RISK-003 |
| **Category** | Security / Cryptography |
| **Description** | The architecture specifies key derivation from "OS user credentials" (Argon2id). The application cannot access the user's OS login password. This is a fundamental design error in the encryption architecture. |
| **Root Cause** | Undefined "OS user credentials" — the architecture assumes the application can access the user's login credentials, which is not possible on any modern OS. |
| **Impact** | Critical (5) — All encryption features (data at rest, credential storage, memory encryption) depend on a key derivation strategy that cannot work. |
| **Probability** | Certain (5) — This is not a risk that might happen; the current design is impossible to implement. |
| **Exposure** | **25** (CRITICAL) |
| **Mitigation** | Generate a random master key on first launch, stored in the OS keychain (Windows Credential Manager / macOS Keychain / Linux Secret Service). The keychain is unlocked by the OS user session. |
| **Contingency** | User-provided master password at application startup (password manager pattern). Worst UX but functional. |
| **Owner** | Security Lead |
| **Timeline** | Resolve before any encryption code is written |

---

### RISK-004: Prompt Injection Through Observation Data

| Field | Value |
|-------|-------|
| **ID** | RISK-004 |
| **Category** | Security / AI Safety |
| **Description** | The observation engine feeds screen content, terminal output, and clipboard data into the AI provider's prompt. An attacker who controls any visible data could inject malicious prompts. |
| **Root Cause** | No input sanitization, context separation, or injection detection in the observation-to-AI pipeline. |
| **Impact** | Critical (5) — Could lead to the AI recommending malicious commands, exfiltrating data, or taking actions outside the intended scope. |
| **Probability** | Possible (3) — Requires an attacker to control displayed content, which is a realistic scenario (compromised website, malicious log output, phishing email). |
| **Exposure** | **15** (HIGH) |
| **Mitigation** | Implement multi-layer prompt injection defense: (1) input normalization, (2) context separation in prompts, (3) output validation, (4) rate limiting, (5) user notification. |
| **Contingency** | Disable observation-based context injection; fall back to user-provided context only. |
| **Owner** | Security Lead |
| **Timeline** | Implement before the observation engine is integrated with the AI provider |

---

### RISK-005: Terminal Observation is Technically Infeasible

| Field | Value |
|-------|-------|
| **ID** | RISK-005 |
| **Category** | Implementation / Observation Engine |
| **Description** | The architecture specifies terminal monitoring (commands and output) but does not define how this is achieved. The three practical approaches (PTY hooking, shell integration, OCR) all have significant limitations. |
| **Root Cause** | Architecture assumes terminal observation is a simple API call, but it is a deeply platform-specific, technically challenging feature. |
| **Impact** | High (4) — Terminal observation is a key differentiator. Without it, the observation engine loses its most valuable data source. |
| **Probability** | Likely (4) — PTY hooking is fragile and OS-version-dependent. Shell integration requires user action. OCR has high latency. |
| **Exposure** | **16** (HIGH) |
| **Mitigation** | Implement shell integration as the primary path (bash `PROMPT_COMMAND`, zsh `preexec`, PowerShell `PSReadLine`). OCR fallback for terminals without integration. Clearly document the limitations. |
| **Contingency** | Rely on the user describing their terminal activity in the chat interface. This is a significant UX degradation but keeps the product functional. |
| **Owner** | Observation Engine Lead |
| **Timeline** | Investigate and define approach before Phase 1 MVP |

---

### RISK-006: MCP Protocol Changes Break the Skill Architecture

| Field | Value |
|-------|-------|
| **ID** | RISK-006 |
| **Category** | Technology Selection / Stability |
| **Description** | The MCP protocol is a draft specification (2024-11-05). It is actively evolving, and breaking changes are expected. The entire skill architecture depends on this protocol. |
| **Root Cause** | Building the core skill runtime on an immature protocol specification. |
| **Impact** | High (4) — Protocol changes could require significant rework of the MCP skill manager, all skill servers, and the tool catalog. |
| **Probability** | Possible (3) — MCP is being actively developed by Anthropic. Major version changes are expected within the project timeline. |
| **Exposure** | **12** (MEDIUM-HIGH) |
| **Mitigation** | Abstract the MCP protocol behind a trait/interface in the core. The internal skill representation should be MCP-agnostic. Pin the MCP protocol version at build time. |
| **Contingency** | Implement a custom JSON-RPC protocol for internal communication, with MCP as an optional external interface. |
| **Owner** | Architecture Lead |
| **Timeline** | Implement abstraction layer before any MCP server code is written |

---

### RISK-007: Screenshot OCR Latency Prevents Real-Time Suggestions

| Field | Value |
|-------|-------|
| **ID** | RISK-007 |
| **Category** | Performance / UX |
| **Description** | The full observation-to-recommendation pipeline (capture → OCR → context → intent → skill → knowledge → AI → response) can take 5-15 seconds, far exceeding the 2-second capture interval. Suggestions will be stale. |
| **Root Cause** | Single-tier observation pipeline treats all data sources equally. |
| **Impact** | Medium (3) — The primary feature (real-time suggestions) will not be real-time. Users will see stale or irrelevant suggestions. |
| **Probability** | Likely (4) — The pipeline complexity alone guarantees 5+ seconds of latency. AI provider response time adds 2-5 seconds. |
| **Exposure** | **12** (MEDIUM-HIGH) |
| **Mitigation** | Implement tiered observation pipeline: Tier 1 (instant) for terminal/commands, Tier 2 (fast) for app context, Tier 3 (slow) for full screenshot analysis. |
| **Contingency** | Remove "real-time" requirement from suggestions. Display suggestions asynchronously when they become available. |
| **Owner** | Observation Engine Lead |
| **Timeline** | Design tiered pipeline before implementation |

---

### RISK-008: Linux Exclusion Limits Market and Creates Blind Spots

| Field | Value |
|-------|-------|
| **ID** | RISK-008 |
| **Category** | Product / Market |
| **Description** | The architecture explicitly excludes Linux desktop support. Enterprise engineers often use Linux. The product team has no visibility into how the product would work on Linux. |
| **Root Cause** | Scope decision to focus on Windows and macOS only. |
| **Impact** | High (4) — Excludes a significant portion of the target user base. Creates a blind spot where the product cannot be tested in Linux-heavy environments. |
| **Probability** | Likely (4) — The decision is explicit in the architecture; without change, Linux is excluded. |
| **Exposure** | **16** (HIGH) |
| **Mitigation** | Add Linux support to the architecture even if the initial release is Windows + macOS. Design all platform-specific code with an abstraction layer. |
| **Contingency** | Accept Linux as a Phase 2 target. Acknowledge that retrofitting Linux support will cost significantly more than designing for it now. |
| **Owner** | Product Manager |
| **Timeline** | Decision needed before architecture is finalized |

---

### RISK-009: MCP Servers Provide No Sandbox Isolation

| Field | Value |
|-------|-------|
| **ID** | RISK-009 |
| **Category** | Security / Isolation |
| **Description** | MCP servers run as the same OS user with full access to the engineer's files, credentials, and network. A compromised skill (or a malicious skill from the skill store) could access any data on the laptop. |
| **Root Cause** | No sandboxing or permission model for skill execution. The architecture assumes all skills are trusted. |
| **Impact** | Critical (5) — A compromised skill could exfiltrate all credentials, knowledge documents, and screen captures. |
| **Probability** | Possible (3) — Requires a skill to be compromised or malicious. With the skill distribution system, this is a realistic scenario. |
| **Exposure** | **15** (HIGH) |
| **Mitigation** | Implement per-platform sandboxing: AppContainer (Windows), sandbox entitlements (macOS), Bubblewrap/Landlock (Linux). Each skill has a minimum permission set. |
| **Contingency** | Skills are reviewed and signed by Wiki Labs before distribution. This reduces but does not eliminate the risk. |
| **Owner** | Security Lead |
| **Timeline** | Implement before the skill distribution system is released |

---

### RISK-010: MVP Scope is Too Large for First Release

| Field | Value |
|-------|-------|
| **ID** | RISK-010 |
| **Category** | Project Management / Delivery |
| **Description** | The MVP includes 12 skills, a full observation engine, skill distribution, and enterprise compliance features. Realistic delivery time for this scope is 12-18 months. |
| **Root Cause** | Insufficient prioritization. The architecture does not define a minimum viable product — it defines the full product. |
| **Impact** | High (4) — The project will miss delivery targets, leading to stakeholder frustration, funding pressure, and potential cancellation. |
| **Probability** | Likely (4) — The scope exceeds what a single team can deliver in a reasonable timeframe. |
| **Exposure** | **16** (HIGH) |
| **Mitigation** | Reduce MVP to 3 skills, chat-first (no observation engine), bundled skills (no distribution). Target 3 months for Phase 1. |
| **Contingency** | Extend the timeline and communicate the realistic scope to stakeholders. |
| **Owner** | Product Manager |
| **Timeline** | Immediately — scope decision must be made before development begins |

---

### RISK-011: Embedding Model is Tied to AI Provider

| Field | Value |
|-------|-------|
| **ID** | RISK-011 |
| **Category** | Architecture / Coupling |
| **Description** | The knowledge embedding model is tied to the configured AI provider. Switching providers requires re-embedding the entire knowledge base. Local-only mode cannot use semantic search. |
| **Root Cause** | Using the AI provider's embedding endpoint for knowledge indexing. |
| **Impact** | Medium (3) — Provider switching is expensive. Offline mode has degraded search. |
| **Probability** | Possible (3) — Enterprise requirements for data residency may force provider switching. |
| **Exposure** | **9** (MEDIUM) |
| **Mitigation** | Use a local embedding model (e.g., `all-MiniLM-L6-v2` via ONNX or llama.cpp) for knowledge indexing. The AI provider is only used for chat. |
| **Contingency** | Cache embeddings and re-use them across provider switches. Add a re-indexing progress indicator. |
| **Owner** | Knowledge Team Lead |
| **Timeline** | Resolve before knowledge import pipeline is implemented |

---

### RISK-012: No User Research Validates the Product Direction

| Field | Value |
|-------|-------|
| **ID** | RISK-012 |
| **Category** | Product / Validation |
| **Description** | The product concept (observation engine, intent recognition, real-time suggestions) is based on assumptions about how engineers work. No user research has been conducted to validate these assumptions. |
| **Root Cause** | Architecture was designed without user research input. |
| **Impact** | High (4) — The product may solve problems engineers don't have, or solve them in a way engineers don't want. |
| **Probability** | Possible (3) — The assumptions are reasonable but unvalidated. |
| **Exposure** | **12** (MEDIUM-HIGH) |
| **Mitigation** | Conduct user research (5-10 enterprise engineers) before Phase 1 implementation. Validate the observation engine concept, skill priorities, and chat interface. |
| **Contingency** | Build the MVP without observation engine (chat-first). This reduces the risk of building the wrong thing. |
| **Owner** | Product Manager |
| **Timeline** | Before Phase 1 implementation begins |

---

### RISK-013: Audit Logs Can Be Tampered With

| Field | Value |
|-------|-------|
| **ID** | RISK-013 |
| **Category** | Security / Compliance |
| **Description** | Audit logs are stored in SQLite and described as "immutable." SQLite is not immutable — anyone with file write access can modify the database. Enterprise compliance requirements demand tamper-proof audit logs. |
| **Root Cause** | The architecture relies on SQLite's append-only insert pattern, which provides no cryptographic integrity verification. |
| **Impact** | Medium (3) — Audit logs cannot be used as evidence in compliance investigations. |
| **Probability** | Possible (3) — The likelihood of tampering depends on the threat model. For enterprise use, the architecture should assume logs will be scrutinized. |
| **Exposure** | **9** (MEDIUM) |
| **Mitigation** | Implement hash chain audit logging (each entry includes the hash of the previous entry) or Ed25519-signed audit entries. |
| **Contingency** | Document that audit logs are not tamper-proof and rely on OS-level file permissions. |
| **Owner** | Security Lead |
| **Timeline** | Implement before Phase 1 release |

---

### RISK-014: AI Provider Abstraction is Too Simple

| Field | Value |
|-------|-------|
| **ID** | RISK-014 |
| **Category** | Architecture / AI Runtime |
| **Description** | The AiProvider trait (`chat()`, `chat_stream()`, `embed()`, `info()`, `health()`) does not include context management, token counting, tool integration, or structured output. This will require significant refactoring when these features are needed. |
| **Root Cause** | Oversimplification of the AI provider interface. |
| **Impact** | Medium (3) — Refactoring the AI provider abstraction mid-project will cause delays and potential regressions. |
| **Probability** | Likely (4) — The current trait is too simple to support the planned features. |
| **Exposure** | **12** (MEDIUM-HIGH) |
| **Mitigation** | Expand the AiProvider trait to include context management, token counting, tool integration, and structured output before implementation begins. |
| **Contingency** | Add the missing features incrementally, accepting the refactoring cost. |
| **Owner** | AI Runtime Lead |
| **Timeline** | Resolve before the AI provider is integrated with the skill system |

---

### RISK-015: No Cross-Skill Context Sharing

| Field | Value |
|-------|-------|
| **ID** | RISK-015 |
| **Category** | Architecture / MCP |
| **Description** | Skills are isolated silos. When a problem spans multiple technologies (e.g., a slow MySQL query on an OpenShift pod running on VMware), there is no mechanism for skills to share context or collaborate. |
| **Root Cause** | Architecture does not define a skill-to-skill communication mechanism. |
| **Impact** | Medium (3) — The AI cannot reason about multi-domain problems holistically. |
| **Probability** | Likely (4) — Multi-domain problems are the norm in enterprise engineering, not the exception. |
| **Exposure** | **12** (MEDIUM-HIGH) |
| **Mitigation** | Add a skill context bus: skills can publish context events that other skills can subscribe to. The context bus is read-only (skills cannot invoke other skills' tools). |
| **Contingency** | The AI reasoning engine itself handles multi-domain problems by calling multiple skills sequentially. This is less efficient but functional. |
| **Owner** | Architecture Lead |
| **Timeline** | Design context bus for Phase 2 |

---

### RISK-016: No CRITICAL skills are vague

Actually let me fix this entry.

---

### RISK-016: Workspace Exports Are Not Supported

| Field | Value |
|-------|-------|
| **ID** | RISK-016 |
| **Category** | Product / Data Portability |
| **Description** | There is no mechanism to export or import workspace configurations. Engineers cannot share configurations with colleagues, and data migration between laptops is manual. |
| **Root Cause** | Workspace export/import is not in the architecture. |
| **Impact** | Low (2) — Not a blocker for MVP but a significant pain point for enterprise adoption. |
| **Probability** | Likely (4) — The need for data portability will emerge quickly in enterprise deployments. |
| **Exposure** | **8** (MEDIUM) |
| **Mitigation** | Add workspace export/import as a Phase 2 feature. The workspace data model already supports serialization. |
| **Contingency** | Users can manually copy the SQLite database file. This is a poor UX but technically possible. |
| **Owner** | Product Manager |
| **Timeline** | Post-MVP |

---

## Risk Heat Map

```
Probability
    │
    │  RISK-001  RISK-003
    │  (25)      (25)
    │
    │  RISK-002  RISK-005  RISK-008  RISK-010
    │  (20)      (16)      (16)      (16)
    │
    │  RISK-004  RISK-006  RISK-007  RISK-012  RISK-014  RISK-015
    │  (15)      (12)      (12)      (12)      (12)      (12)
    │
    │  RISK-009  RISK-011  RISK-013
    │  (15)      (9)       (9)
    │
    │                                 RISK-016
    │                                 (8)
    └────────────────────────────────────────────────── Impact
        Low        Medium     High      Critical
```

## Priority Action Items

| Priority | Risk ID | Action | Deadline |
|----------|---------|--------|----------|
| 🔴 IMMEDIATE | RISK-001 | Consolidate skill processes | Before implementation |
| 🔴 IMMEDIATE | RISK-003 | Fix key derivation strategy | Before encryption code |
| 🔴 HIGH | RISK-002 | Replace ChromaDB | Before knowledge pipeline |
| 🔴 HIGH | RISK-004 | Add prompt injection defense | Before observation-AI integration |
| 🔴 HIGH | RISK-005 | Define terminal observation strategy | Before Phase 1 |
| 🔴 HIGH | RISK-008 | Decide on Linux support | Before architecture finalization |
| 🔴 HIGH | RISK-010 | Reduce MVP scope | Immediately |
| 🟡 MEDIUM | RISK-006 | Abstract MCP protocol | Before MCP server code |
| 🟡 MEDIUM | RISK-007 | Design tiered observation pipeline | Before observation implementation |
| 🟡 MEDIUM | RISK-009 | Implement sandboxing | Before skill distribution |
| 🟡 MEDIUM | RISK-011 | Decouple embeddings from AI provider | Before knowledge pipeline |
| 🟡 MEDIUM | RISK-012 | Conduct user research | Before Phase 1 |
| 🟡 MEDIUM | RISK-013 | Implement audit log integrity | Before Phase 1 release |
| 🟡 MEDIUM | RISK-014 | Expand AiProvider trait | Before AI-skill integration |
| 🟡 MEDIUM | RISK-015 | Design cross-skill context bus | Phase 2 |
| 🟢 LOW | RISK-016 | Add workspace export/import | Phase 2 |