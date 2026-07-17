# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### AI Runtime (Phase 5) — Intelligence Layer

- **AI Provider Abstraction** — Unified `AiProvider` trait with `OpenAICompatibleProvider` implementation supporting OpenAI, vLLM, and Ollama
  - Chat, streaming chat, embeddings, health checks
  - Feature detection (tools, streaming, structured output, vision)
  - Approximate token counting

- **Conversation Manager** — Multi-conversation lifecycle with CRUD operations
  - Message roles (user, assistant, system) with timestamps
  - Tool call tracking on assistant messages
  - Tag-based categorization, renaming, export/restore
  - Conversation summaries for listing

- **Context Manager** — Central aggregation of information from multiple sources
  - Priority-based context sources (High, Normal, Low)
  - Manual context injection with tagging and filtering
  - Technology stack selection and current activity tracking
  - Fluent `ContextBuilder` for incremental construction
  - Source sorting by priority, tag-based filtering

- **Prompt Manager** — Template-driven prompt assembly with versioning
  - `{{placeholder}}` syntax with `TemplateContext` for replacement
  - Template categories: system, workspace, context, user, skill
  - Version tracking: Default, Numbered (v1, v2...), Named ("stable", "experimental")
  - Active template selection and version bumping
  - Full prompt assembly with breakdown by section

- **Engineering Persona** — AI behavioral definition with confidence management
  - Roles: Senior Infrastructure Engineer, Technical Advisor, Enterprise Consultant, Troubleshooting Mentor
  - Behavioral rules: evidence-based, step-by-step reasoning, verification suggestions
  - Confidence levels: HIGH (≥0.9), MEDIUM (≥0.6), LOW (<0.3)
  - Structured `ConfidenceAssessment` output
  - Custom persona support

- **Context Window Manager** — Token budget tracking with allocation percentages
  - Configurable allocation: system prompt (10%), conversation history (40%), knowledge context (20%), workspace context (20%), padding (10%)
  - Usage tracking with percentage and remaining token reporting
  - Budget-aware fitting checks
  - Intelligent truncation with target token budget
  - Context entry recording and reporting

- **Session Manager** — AI session lifecycle with state transitions
  - States: Active → Paused → Suspended → Ended
  - Session configuration: model, temperature, max tokens, workspace, technologies
  - Message and token consumption tracking
  - Idle detection with configurable timeout
  - Session duration calculation

- **Token Budget Manager** — Token estimation, intelligent trimming, summarization
  - Three policies: Strict, WithBuffer (configurable overflow), Aggressive
  - Budget breakdown by source with priority labels (System, Recent, Older, Workspace, Low)
  - Recommended actions: TrimConversation, Summarize, DropLowPriorityContext, Reject
  - Fluent `BudgetBuilder` API for constructing checks

- **AI Streaming** — Progressive response display with cancellation support
  - `tokio::sync::mpsc::UnboundedReceiver` for chunk streaming
  - Background task spawning for non-blocking reception
  - Error message forwarding to receiver channel

#### Tests (148 total across AI Runtime)

- **AI Runtime module**: 181 unit tests across all 12 crates — every module has comprehensive tests
- Workspace manager: full CRUD lifecycle (create, switch, get, list, delete, multiple workspaces)
- Knowledge: document types, search, embedding, import, dedup, quality scoring
- Security: classification, keychain, key derivation, encryption, credentials, injection defense, audit
- Observation: tiers 1-3, shell, app monitor, clipboard, capture, OCR, credential filter
- AI: context window allocation, token counting, response streaming, tool calls, provider traits
- Intent: model prediction, engine pattern matching, confidence scoring, correction tracking
- Fixed: `Intent` enum missing `Hash` derive (used in correction HashMap)
- Fixed: `chrono` dependency missing from `wikilabs-intent` Cargo.toml
- Fixed: `Debug` derives missing from `Secret`, `AppContext`, `CaptureResult`, `KnowledgeChunk`, `KnowledgeDocument`, `SearchQuery`, `SearchResult`, `EmbeddingResult`, `QualityScore`
- Fixed: confidence formula in `recognize_with_confidence` for multiple matches

### Documentation

- Product Vision document
- Architecture Review with 16 risk items
- 8 Architecture Decision Records (ADRs)
- Revised architecture specification
- Phased implementation roadmap (4 phases, ~40 weeks)
- Backlog with 10 epics and dependency graph
- Coding standards (Rust style, naming, error handling, async, testing)
- Development guide with setup, build, and CI/CD instructions
- GitHub issue templates (bug, feature, security, architecture)
- AI Runtime documentation (`AI_RUNTIME.md`)
- Conversation Manager documentation (`CONVERSATION_MANAGER.md`)
- Prompt Manager documentation (`PROMPT_MANAGER.md`)

### Changed

- Consolidated multi-process MCP into single-process skill module loader
- Simplified architecture to deliver MVP chat copilot in 12 weeks instead of 12+ months

## [0.1.0] — 2024-XX-XX

### Added

- Initial repository bootstrap
- Workspace `Cargo.toml` with dependency management
- CI/CD workflow skeleton (`pr.yml`, `main.yml`, `release.yml`)
- `src/main.rs` entrypoint
- Skeleton crates for all planned modules
- GitHub issue templates

[Unreleased]: https://github.com/wikilabs/wikilabs-ai-copilot/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/wikilabs/wikilabs-ai-copilot/releases/tag/v0.1.0