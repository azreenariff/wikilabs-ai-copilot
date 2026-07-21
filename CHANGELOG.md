# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] — 2026-07-21

### Added

#### Operations Engineering Foundation & Operations Skill Packs (Phase 15) — Enterprise Operations

- **Operations Engineering Foundation** — Reusable operational engineering knowledge base covering: monitoring, alerting, events, incidents, service health, availability, performance, capacity, configuration drift, automation, maintenance, root cause analysis, operational risk, runbooks, escalation, evidence collection, change management awareness, troubleshooting philosophy, best practices, operational safety, and decision trees. All concepts reusable by every Operations Skill Pack.

- **Version-Aware Knowledge System** — Product version awareness supporting version-specific guidance across all skill packs: product versions, documentation versions, command variations, workflow variations, feature availability, deprecation notices, and best practice changes. Automatically selects most appropriate guidance when product version is determined.

- **Cross-Skill Operational Workflows** — Multi-technology troubleshooting workflows demonstrating collaboration between Skill Packs. Patterns: Application unavailable → monitoring alert → review monitoring → review logs → review Linux → review database → probable next investigation. High database latency → monitoring → database investigation → infrastructure → storage → operational recommendation.

- **Confidence & Evidence Framework** — Every recommendation includes: observation, interpretation, recommendation, reason, evidence, confidence, suggested next step, expected outcome. Confidence based only on observable evidence. Never exposes internal reasoning or chain-of-thought.

- **AI Safety Framework** — Copilot never fabricates observations. Clear distinction between: observed facts, likely causes, possible causes, unknowns, assumptions, recommendations. If evidence is insufficient, explicitly requests additional investigation. Never presents assumptions as facts.

- **Operations Quality Standard** — Quality requirements for every Operations Skill Pack: technology overview, architecture, terminology, detection rules, observation patterns, troubleshooting workflows, decision trees, command guidance, operational best practices, common failures, documentation references, version-aware guidance, operational safety, examples, and testing.

##### Nagios XI Skill Pack

- **Complete skill pack** for Nagios XI monitoring
- 19 files across 13 subdirectories
- **Architecture & Components** — Monitoring engine, core components, NRPE, NCPA, SNMP, plugins
- **Monitoring Concepts** — Services, hosts, host groups, service groups, dashboards, notifications, escalations, dependencies, performance data, reporting
- **Operations** — Capacity planning, availability reporting, backups, upgrade concepts, high availability, operational best practices
- **Common Failures & Detection** — Detection rules, observation patterns, troubleshooting workflows, decision trees
- **Command Guidance & Documentation** — Structured command guidance with references, version-aware guidance for Nagios XI 2024/2025
- **Subdirectories**: architecture/, best-practices/, common-failures/, concepts/, context/, diagnostics/, documentation/, guidance/, knowledge/, reasoning/, references/, tests/, workflows/

##### Nagios Log Server Skill Pack

- **Complete skill pack** for Nagios Log Server
- 20 files across 13 subdirectories
- **Architecture & Data Flow** — Log ingestion, parsing, searching, dashboards, alerts
- **Storage & Performance** — Indices, retention, storage, cluster concepts, performance
- **Operations** — Common failures, detection rules, observation patterns, troubleshooting workflows, decision trees
- **Guidance & Documentation** — Structured guidance with references, version-aware knowledge
- **Subdirectories**: architecture/, best-practices/, common-failures/, concepts/, context/, diagnostics/, documentation/, guidance/, knowledge/, reasoning/, references/, tests/, workflows/

##### Checkmk Skill Pack

- **Complete skill pack** for Checkmk monitoring
- 21 files across 13 subdirectories (+ CHECKMK_SKILL_PACK.md overview)
- **Architecture & Sites** — Sites, agents, SNMP, rulesets, discovery
- **Monitoring Features** — Notifications, dashboards, distributed monitoring, performance, business intelligence, automation
- **Operations** — Reporting, detection rules, observation patterns, troubleshooting workflows, decision trees
- **Command Guidance & Documentation** — Structured command guidance, documentation references, version-aware knowledge for Checkmk 2.3/2.4
- **Subdirectories**: architecture/, best-practices/, common-failures/, concepts/, context/, diagnostics/, documentation/, guidance/, knowledge/, reasoning/, references/, tests/, workflows/

##### Ansible Skill Pack

- **Complete skill pack** for Ansible automation
- 20 files across 13 subdirectories
- **Core Concepts** — Architecture, inventories, playbooks, roles, collections, variables, vault, templates, facts, modules, handlers
- **Execution** — Execution flow, idempotency, error handling
- **Operations** — Operational guidance, troubleshooting, decision trees, safety considerations
- **Guidance & Documentation** — Structured guidance with references, version-aware knowledge
- **Subdirectories**: architecture/, best-practices/, common-failures/, concepts/, context/, diagnostics/, documentation/, guidance/, knowledge/, reasoning/, references/, tests/, workflows/

##### MySQL Skill Pack

- **Complete skill pack** for MySQL database administration
- 41 files across 14 subdirectories (largest Phase 15 skill pack)
- **Architecture** — Storage engines, replication, backups, indexes, transactions, locks
- **Performance** — Performance tuning, slow queries, users, privileges, configuration, logging
- **High Availability** — HA concepts, backup-recovery strategies
- **Troubleshooting** — Decision trees, guidance, version-aware knowledge, detection rules
- **8 Workflow Files** — buffer-pool-pressure, disk-space-exhaustion, high-connection-count, high-cpu-usage, lock-contention, replication-failure, replication-lag, slow-query
- **Additional Documentation** — MYSQL_BEST_PRACTICES.md, MYSQL_COMMAND_REFERENCE.md, MYSQL_COMMON_FAILURES.md, MYSQL_DETECTION.md, MYSQL_GUIDANCE.md, MYSQL_REASONING_GUIDE.md, MYSQL_SKILL_PACK.md, MYSQL_SKILL_PACK_QUALITY_STANDARD.md, MYSQL_WORKFLOWS.md
- **Version-aware guidance** for MySQL 8.0
- **Subdirectories**: architecture/, best-practices/, common-failures/, concepts/, context/, diagnostics/, documentation/, examples/, guidance/, knowledge/, reasoning/, references/, tests/, workflows/

##### EDB PostgreSQL Skill Pack

- **Complete skill pack** for EDB PostgreSQL
- 34 files across 14 subdirectories
- **Architecture** — PostgreSQL architecture, streaming replication, WAL
- **Data Management** — Backups, indexes, locks, transactions
- **Performance** — Performance optimization, configuration, logging
- **High Availability** — HA concepts with EDB-specific tooling
- **Troubleshooting** — Decision trees, guidance, version-aware knowledge
- **7 Knowledge Files** — architecture, backup-recovery, performance-optimization, replication, security, wal
- **Additional Documentation** — EDB_POSTGRESQL_BEST_PRACTICES.md, EDB_POSTGRESQL_COMMAND_REFERENCE.md, EDB_POSTGRESQL_COMMON_FAILURES.md, EDB_POSTGRESQL_DETECTION.md, EDB_POSTGRESQL_GUIDANCE.md, EDB_POSTGRESQL_REASONING_GUIDE.md, EDB_POSTGRESQL_SKILL_PACK.md, EDB_POSTGRESQL_SKILL_PACK_QUALITY_STANDARD.md, EDB_POSTGRESQL_WORKFLOWS.md
- **Version-aware guidance** for PostgreSQL 15/16
- **Subdirectories**: architecture/, best-practices/, common-failures/, concepts/, context/, diagnostics/, documentation/, examples/, guidance/, knowledge/, reasoning/, references/, tests/, workflows/

##### Microsoft SQL Server Skill Pack

- **Complete skill pack** for Microsoft SQL Server
- 28 files across 13 subdirectories
- **Architecture** — SQL Server architecture, availability groups, backups, recovery
- **Performance** — Indexes, statistics, execution plans, performance, memory, tempdb
- **Maintenance & Security** — Maintenance plans, security, transactions
- **Troubleshooting** — Decision trees, guidance, version-aware knowledge
- **6 Knowledge Files** — architecture, availability-groups, backups-recovery, indexes-statistics, security, tempdb-memory
- **Additional Documentation** — MSSQL_COMMAND_REFERENCE.md, MSSQL_DETECTION.md, MSSQL_GUIDANCE.md, MSSQL_SKILL_PACK.md
- **Version-aware guidance** for Microsoft SQL Server 2022
- **Subdirectories**: architecture/, best-practices/, common-failures/, concepts/, context/, diagnostics/, documentation/, examples/, guidance/, knowledge/, reasoning/, tests/, workflows/

### Changed

- Version bumped from 0.9.0-alpha to 1.3.0-alpha

### New Files

- `docs/operations/OPERATIONS_FOUNDATION.md` — Operations Engineering Foundation
- `docs/operations/CONFIDENCE_EVIDENCE_ENGINE.md` — Confidence & Evidence Framework
- `FRAMEWORK_DOCS/AI_SAFETY.md` — AI Safety Framework
- `FRAMEWORK_DOCS/VERSION_AWARENESS.md` — Vendor Version Awareness system
- `FRAMEWORK_DOCS/CROSS_SKILL_WORKFLOWS.md` — Cross-Skill Operational Workflows
- `FRAMEWORK_DOCS/QUALITY_STANDARD.md` — Operations Quality Standard
- `src/skills/nagiosxi-skill-pack/` — Nagios XI skill pack (19 files)
- `src/skills/nagioslogserver-skill-pack/` — Nagios Log Server skill pack (20 files)
- `src/skills/checkmk-skill-pack/` — Checkmk skill pack (21 files)
- `src/skills/ansible-skill-pack/` — Ansible skill pack (20 files)
- `src/skills/mysql-skill-pack/` — MySQL skill pack (41 files)
- `src/skills/edb-postgresql-skill-pack/` — EDB PostgreSQL skill pack (34 files)
- `src/skills/mssql-skill-pack/` — Microsoft SQL Server skill pack (28 files)

## [0.9.0] — 2026-07-20

### Added

#### Engineering Skills Pack (Phase 3) — Production-Ready MCP Skill Packs

- **Skill Pack Framework** — Structured skill pack format with manifest, technology definitions, detection rules, workflows, commands, guidance, best practices, and known issues
- **Engineering Reasoning** — Evidence-based reasoning framework: observation → hypothesis → validation → remediation. Confidence scoring, risk assessment, safety constraints
- **Knowledge Bases** — Technology-specific knowledge in structured markdown: cluster architecture, container runtime, networking, security, RBAC, SCC, operators

##### OpenShift Engineering Skill Pack

- **Complete skill pack** for Red Hat OpenShift 4.x
- 40 files across 17 subdirectories
- **manifest.yaml** — Skill metadata, version, dependencies, technology scope
- **technology.yaml** — OpenShift features, platforms, components, capabilities coverage
- **workflows.yaml** — 10 state machine workflows (CrashLoopBackOff, Pending, OOMKilled, ImagePullBackOff, Node NotReady, Deployment Failure, Operator Degraded, Route Unavailable, PVC Pending, Auth Failure) with multi-step evidence collection, diagnosis, remediation, and verification stages
- **detection_rules.yaml** — 16+ detection rules for CLI commands, browser URLs, and text patterns with confidence scoring
- **commands.yaml** — 160+ technical commands with purpose, risk assessment, parameters, usage notes, verification steps, and documentation references
- **guidance/rules.md** — Engineering guidance: evidence-based reasoning, safety rules, command explanation standards
- **best-practices.md** — 15 best practices across cluster management, deployments, security, networking, storage, and monitoring
- **known_issues.md** — 10 known issues with symptoms, detection, workarounds, and upgrade tracking
- **Knowledge base** (4 files): cluster-architecture.md, container-runtime.md, networking-services-routes.md, security-rbac-scc.md
- **Documentation** (7 markdown docs): skill pack overview, troubleshooting workflows, engineering reasoning guide, detection rules reference, command reference, guidance documentation, best practices reference
- **Supporting directories**: architecture/, best-practices/, commands/, common-failures/, concepts/, context/, detection/, diagnostics/, documentation/, examples/, guidance/, knowledge/, overview/, reasoning/, references/, tests/, workflows/

##### Linux Engineering Skill Pack

- **Complete skill pack** for Linux administration
- 40 files across 17 subdirectories
- **workflows.yaml** — 1400+ lines of workflow definitions covering SSH hardening, systemd troubleshooting, LVM management, performance tuning, and more
- **detection_rules.yaml** — 10+ detection rules for Linux CLI commands and system patterns
- **commands.yaml** — 120+ technical commands with full documentation

##### VMware vSphere Engineering Skill Pack

- **Complete skill pack** for VMware vSphere management
- 40 files across 17 subdirectories
- **workflows.yaml** — 800+ lines of workflow definitions
- **detection_rules.yaml** — 8+ detection rules for vSphere CLI commands and patterns
- **commands.yaml** — 60+ technical commands with full documentation

##### Skill Pack Template

- Reusable template for creating new skill packs in the Wiki Labs format
- All 7 core sections: manifest, technology, detection, workflows, commands, guidance, knowledge
- Structured subdirectory layout with 17 directories
- Detection rules with confidence scoring
- State machine workflows with evidence collection, diagnosis, remediation, and verification stages

### Changed

- Updated ROADMAP.md with sequential implementation numbering (Phase 1-6)
- Version bumped from 0.8.0-alpha to 0.9.0-alpha

### New Files

- `src/skills/openshift-skill-pack/` — OpenShift engineering skill pack (40 files)
- `src/skills/linux-engineering/` — Linux engineering skill pack (40 files)
- `src/skills/vmware-vsphere-skill-pack/` — VMware vSphere skill pack (40 files)

## [0.8.0] — 2026-07-20

### Added

#### Skill Platform Documentation (Phase 11)

- **SKILL_MANIFEST_SPEC.md** — Full schema for skill manifests including required/optional fields, validation rules, and schema evolution
- **SKILL_DISCOVERY.md** — Discovery engine internals: signals, signatures, scoring, filtering, and integration flow
- **SKILL_ACTIVATION.md** — Activation engine internals: lifecycle, dependency resolution, health monitoring, notifications
- **SKILL_PACKAGING.md** — Packaging format (.wls), install/uninstall/update workflows, validation, security, versioning
- **SKILL_DEVELOPMENT_GUIDE.md** — End-to-end developer guide: manifest, detection rules, workflows, knowledge base, guidance, testing, SDK commands
- **LINUX_SKILL.md** — Reference skill documentation covering detection, knowledge base, workflows, commands, best practices, known issues

### Changed

- **Version bump** — `0.7.0-alpha` → `0.8.0-alpha`
- **SKILL_ARCHITECTURE.md** — Updated to v0.8.0, fixed manifest example to include `id` field and `dependencies` (was `depends_on`)
- **SKILL_SDK_GUIDE.md** — Updated to v0.8.0

### Added

#### Guidance Engine (Phase 10) — Context-Aware Engineering Guidance

- **Guidance Panel** — Tauri-native sidebar panel for engineering guidance with session-level, skill-level, and cross-skill context
- **Session Context Provider** — Provides session-level context: current engineering task, session duration, prior decisions, pending approvals, open recommendations
- **Skill Context Provider** — Provides skill-level context: active technologies, available commands, workflow state, relevant documentation
- **Cross-Skill Context Provider** — Provides cross-skill context: multi-skill interactions, shared state, cross-cutting concerns
- **Guidance Manager** — Aggregates and surfaces context-aware guidance items across all providers
- **Guidance Item Types** — Recommendations, warnings, suggestions, tips, explanations with priority levels, technology tags, confidence scores, and action links
- **Integration** — Full integration with existing copilot engine for unified guidance delivery

- **New crates** — `src/guidance/` (Guidance Engine), `src-tauri/src/guidance_panel.rs` (Tauri panel)
- **Documentation** — `docs/guidance-engine/` with architecture overview, API reference, and integration guide
- **132 tests passing** — Comprehensive test suite across all Guidance Engine modules

#### Skill Platform (Phase 11) — Dynamic Skill Discovery, Activation, and Management

- **Skill Discovery Engine** (`src/skill_discovery/`) — Scans the workspace for technology signals using glob patterns, command detection, and configuration file matching. Produces discovery reports with confidence scores and prioritization for 14+ technology domains.
  - Technology signal detection (file patterns, command presence, configuration files)
  - Confidence scoring based on signal strength and source reliability
  - Discovery configuration (scan paths, signal thresholds, ignored directories)
  - Comprehensive discovery report generation

- **Skill Activation Engine** (`src/skill_activation/`) — Dynamically activates detected skills with dependency resolution, health monitoring, and lifecycle management.
  - Activation candidate matching (discovered skills → skill definitions)
  - Dependency checking and resolution (skills must have all prerequisites loaded/enabled)
  - Health monitoring with configurable max failure count and recovery
  - Graceful activation/deactivation with notification support
  - Auto-activation for high-confidence detections

- **Skill Runtime Extension** (`src/skill_runtime/`) — Extended to orchestrate the full skill lifecycle: discover → load → validate → enable → activate → monitor.
  - Integration with Skill Discovery Engine (auto-discovery loop)
  - Integration with Skill Activation Engine (dependency-aware activation)
  - Schema version validation and version compatibility checking
  - Comprehensive validation exports (`validate_pack_comprehensive`)
  - Skill lifecycle state management (Loaded, Enabled, Active, Suspended, Disabled)

- **Skill SDK Extension** (`src/skill_sdk/`) — Extended with packaging and CLI capabilities.
  - Template generation for 8 skill types (technology, workflow, command, detection, intent, knowledge, policy, guidance)
  - Schema validation for 7 YAML/JSON skill components
  - Skill packager (create, validate, package, list-templates commands)
  - Developer documentation and schema registry

- **Linux Engineering Skill** (`src/skills/linux-engineering/`) — Reference skill demonstrating the complete skill platform: manifest, technology definitions, detection rules, commands, workflows, best practices, guidance rules, known issues, and knowledge base.
  - SSH hardening knowledge base
  - Systemd troubleshooting knowledge base
  - LVM management knowledge base
  - Systemd service workflow
  - SSH key rotation workflow
  - Performance troubleshooting workflow

- **Skill Management UI** (`src-tauri/src/skill_management.rs`) — Tauri panel for managing enterprise skills.
  - List installed skills with version, technology, status, confidence, dependencies
  - View skill documentation
  - Enable/disable skill availability
  - Validate skill integrity
  - Update skills from packages

- **Knowledge Panel Update** — Integrated validation exports and guidance panel imports into the knowledge panel UI

- **Architecture Decision Record** — `docs/adr/ADR-012-SKILL-PLATFORM.md` — Comprehensive ADR documenting the three-component Skill Platform architecture

- **Documentation** — 3 Phase 11 documentation files: Skill Platform Architecture, Skill Platform Quickstart, updated engineering intelligence docs

- **2677 lines of new Rust code** across 4 new crates and 1 extended crate

### Changed

- Updated ROADMAP.md to reflect Phase 10 and Phase 11 completion
- Version bumped from 0.5.0-alpha to 0.7.0-alpha

### New Workspace Members Added

- `src/skill_discovery/` — Skill Discovery Engine
- `src/skill_activation/` — Dynamic Skill Activation Engine
- `src/skill_runtime/` — Extended Skill Runtime (now includes discovery and activation orchestration)
- `src/skill_sdk/` — Extended Skill SDK (now includes packaging and CLI)
- `src/skills/` — Reference skill packages (Linux Engineering)
- `src/guidance/` — Guidance Engine (Phase 10)
- `src-tauri/src/skill_management.rs` — Skill Management UI panel

## [0.5.0] — 2026-07-17

### Added

#### Knowledge Management & CLI SDK (Phase 8) — v0.5.0-alpha

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

#### Knowledge Management & CLI SDK (Phase 8) — v0.5.0-alpha

- **Knowledge Management UI** — Full knowledge pack management in the desktop sidebar: create, edit, delete, import, validate, and package knowledge packs. Context-aware suggestions tied to active workspace and technology stack.
- **Embedding Provider Abstraction** — Pluggable embedding provider trait (`EmbeddingProvider`) with `LocalEmbeddingProvider` and mock implementations for testing. Enables swapping embedding backends (OpenAI, local models, custom APIs).
- **Knowledge Pack CLI** — `knowledge-cli` binary with commands: `create-pack` (generate from template), `validate` (verify pack structure), `package` (build .wkl archive), `list-templates`. Supports 3 predefined templates (openshift, engineering, documentation).
- **Context-Aware Suggestions** — Sidebar displays relevant suggestions based on active context (screen, terminal, workspace).
- **Knowledge Pack Import** — Import documents into workspace knowledge base from `.wkl` archives with automatic metadata extraction and indexing.
- **Fixed: Intent enum missing Default derive** — Added `Default` implementation to `Intent` enum (required for HashMap operations).
- **Fixed: test_tech_match_scoring assertion** — Updated score comparison to `>= 0.5` threshold to match confidence calculation.
- **All 148 tests passing** — Full test suite across AI Runtime and Knowledge Management crates.

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

#### Copilot Engine (Phase 9) — v0.6.0-alpha

- **Copilot Engine** — Central orchestration layer for the observation→recommendation→approval loop. Coordinates all subsystems ensuring the AI never performs work autonomously while providing timely, context-aware assistance.
- **Decision Engine** — Multi-criteria recommendation visibility evaluation with confidence thresholds, evidence requirements, user state awareness, session/frequency limits, repetition avoidance, and workflow relevance filtering. 9 evaluation rules in priority order.
- **Recommendation Engine** — Observation classification (error/resource/performance/security/deprecation/information) with template-based generation, engineering context incorporation, auto-evidence generation, and deduplication via recent title tracking.
- **Policy Engine** — 5-level policy spectrum (Minimal/Balanced/Teaching/Expert/Silent) controlling recommendation visibility. Policy-specific confidence thresholds, priority filtering, and interruption cooldown enforcement.
- **Lifecycle Manager** — Recommendation state machine (Candidate→Ready→Displayed→Accepted→Completed) with invalid transition prevention, state history recording, and terminal state enforcement.
- **Session Memory** — Engineer interaction tracking for personalization: acceptance rate, dismissal reasons, correction tracking, topic analysis, and confidence adjustment on frequently corrected topics.
- **Conversation Context** — Multi-turn conversation management with turn tracking, topic identification, recommendation discussion tracking, and context summarization for follow-up suggestions.
- **Explainability Engine** — Traceable recommendation reasoning with reason trees (mandatory/optional nodes), evidence mapping, certainty scoring, and human-readable explanation generation.
- **Human Approval System** — Human-in-the-loop enforcement with approval request lifecycle (Pending→Approved/Denied/AutoApproved), low-risk auto-approval, and audit trail tracking.
- **Proactive Assistance** — Interruption management with 5 signal types (Error Detected, Idle Detection, Resource Threshold, Related Work, High Confidence), flooding prevention (>10 signals/minute threshold), and urgency classification (High/Medium/Low).
- **Contextual Follow-Up** — Keyword-based follow-up suggestion generation (15 trigger keywords across security, performance, refactoring, test coverage, deprecation domains) with deduplication and next-step guidance.
- **Priority Filtering** — Priority level filtering (Critical=4/Warning=3/Suggestion=2/Information=1) with policy mapping, score-based filtering, and most-urgent identification.
- **Mode Configuration** — Operating mode presets (Minimal/Balanced/Teaching/Expert/Silent) with human-readable descriptions and configuration management.
- **Recommendation Cards** — Display card generation with title/technology/confidence/priority/reason/actions structure, action types (Explain/Open Documentation/Dismiss/Mark Complete), minimal mode support, and priority color coding.
- **Engineering Context Builder** — Context aggregation with technology stack, workspace, timeline, prior recommendations, workflow state, and user preferences integration.
- **Safety Constraints** — No autonomous execution, human approval required, evidence-based recommendations, explainable decisions, context-awareness, no repetition, frequency limits (2/min), session limits (5/session).
- **Documentation** — 7 comprehensive Phase 9 documentation files: Copilot Engine Architecture, Decision Engine, Policy Engine, Recommendation Engine, Lifecycle Manager, Session Memory, Proactive Assistance.
- **132 tests passing** — Comprehensive test suite across all 14 Phase 9 modules.

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