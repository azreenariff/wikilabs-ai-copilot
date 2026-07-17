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

#### Engineering Intelligence Engine & Skill SDK Foundation (Phase 7) — v0.4.0-alpha

- **Technology Recognition Engine** — Evidence-based technology detection from browser URL/title, terminal commands, active application, file patterns, workspace context, conversation keywords. Supports 14 technology domains across infrastructure, monitoring, database, and development.
- **Intent Recognition Engine** — Technology-aware intent classification with continuous updating from observation events, conversation, and workspace context. Human override always takes precedence.
- **Engineering Workflow Engine** — State machine workflow tracking with states, transitions, required evidence, confidence requirements, completion criteria, and validation rules — all defined declaratively by Skills.
- **Context Fusion Engine** — Unified engineering context combining observations, conversation, workspace, technology, intent, workflow state, timeline, and human corrections.
- **Confidence Engine** — Confidence scoring on every inference with automatic confirmation requests for low-confidence detections.
- **Engineering Timeline** — Chronological activity tracking with references to source observation events.
- **Recommendation Readiness Engine** — Determines whether sufficient information exists for recommendations before generating advice.
- **Human Feedback Loop** — Correction tracking and intent override from direct human input.
- **Declarative Skill System** — Dynamic skill loading with manifest validation, version management, enable/disable lifecycle, and dependency checking.
- **Skill Runtime** — Skill discovery, loading, validation, version management, and dependency resolution from configurable directories.
- **Skill SDK** — Complete Skill Development Kit with template generator (8 templates), schema validator (7 JSON schemas), CLI scaffolding, and developer documentation.
- **Engineering Context** — New `EngineeringContext` type combining technologies, confidence scores, primary/secondary intents, and source tracking.
- **Technology Definition** — New `TechnologyDefinition` and `DetectionRule` types in `data_types` crate.
- **Timeline** — New `TimelineEntry` type with chronology support.

#### Tests (Phase 7)

- Technology detection tests (engine, pipeline, aggregator)
- Intent detection tests (engine, model, confidence, correction)
- Workflow transition tests (state machine, evidence validation)
- Skill loading tests (discovery, validation, dependency resolution)
- SDK generation tests (template rendering, schema validation)
- Context update tests (fusion, priority, source tracking)
- Human correction tests (override tracking, intent reclassification)
- Confidence scoring tests (formula verification, threshold behavior)
- Timeline tests (chronological ordering, event references)
- Recommendation readiness tests (evidence gap detection, confidence thresholds)

#### Documentation (Phase 7)

- `docs/engineering-intelligence/ENGINEERING_INTELLIGENCE.md` — Architecture overview
- `docs/engineering-intelligence/TECHNOLOGY_RECOGNITION.md` — Detection engine reference
- `docs/engineering-intelligence/INTENT_ENGINE.md` — Intent recognition reference
- `docs/engineering-intelligence/WORKFLOW_ENGINE.md` — Workflow engine reference
- `docs/engineering-intelligence/SKILL_ARCHITECTURE.md` — Skill system design
- `docs/engineering-intelligence/SKILL_SDK_GUIDE.md` — SDK usage guide
- `docs/engineering-intelligence/SKILL_SCHEMA_REFERENCE.md` — Schema field reference

#### New Crates

- `src/intent/` — Intent recognition engine with confidence and correction tracking
- `src/technology_recognition/` — Technology detection from observation events
- `src/engineering_timeline/` — Chronological engineering activity tracking
- `src/recommendation_readiness/` — Advice readiness assessment
- `src/human_feedback/` — Human correction and override handling
- `src/skill_runtime/` — Skill discovery, loading, validation, lifecycle
- `src/intelligence_engine/` — Cross-cutting intelligence analysis
- `src/context_fusion/` — Unified context aggregation
- `src/skill_sdk/` — Skill Development Kit (templates, schemas, validators)
- `src/workflow_engine/` — Declarative workflow state machine
- `src/core/data_types/src/engineering_context.rs` — EngineeringContext type
- `src/core/data_types/src/technology.rs` — TechnologyDefinition type
- `src/core/data_types/src/timeline.rs` — TimelineEntry type
- `src/observation/src/browser.rs` — Browser observation provider
- `src/observation/src/terminal.rs` — Terminal observation provider
- `src/observation/src/file_observer.rs` — File observer provider
- `src/observation/src/event.rs` — ObservationEvent and EventType
- `src/observation/src/event_bus.rs` — Event bus for event distribution
- `src/observation/src/provider.rs` — Provider trait and base implementations
- `src/observation/src/privacy.rs` — Privacy controls and PII filtering
- `src/observation/src/screen_capture.rs` — Screen capture provider
- `src/observation/src/app_monitor.rs` — Application monitoring
- `src/observation/src/clipboard.rs` — Clipboard observation
- `src/observation/src/engine.rs` — Observation engine coordination

#### Constraints Enforced

- No knowledge retrieval or RAG
- No MCP execution
- No command execution
- No automation
- No screen AI analysis or OCR reasoning
- No customer environment access
- No autonomous actions
- No technology-specific logic in core application
- All technology knowledge loaded from Skills dynamically

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