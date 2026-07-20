# Wiki Labs AI Copilot — Product Roadmap

> **Last Updated:** 2026-07-20  
> **Version:** 1.1  
> **Status:** Phase 9 Complete (v0.6.0-alpha)  
> **Owner:** Technical Lead

---

## Table of Contents

1. [Product Vision](#product-vision)
2. [Strategic Principles](#strategic-principles)
3. [Architecture Overview](#architecture-overview)
4. [Phase-Based Roadmap](#phase-based-roadmap)
5. [Phase Dependencies](#phase-dependencies)
6. [Risk Mitigation](#risk-mitigation)
7. [Go-to-Market Strategy](#go-to-market-strategy)
8. [Growth and Expansion](#growth-and-expansion)
9. [Key Milestones](#key-milestones)

---

## Product Vision

Wiki Labs AI Copilot is an AI-powered enterprise engineering copilot for infrastructure and DevOps engineers. It operates as an experienced senior engineer sitting beside you — observing, understanding context, providing recommendations, explaining issues, suggesting commands, and guiding through best practices.

**Key differentiators:**
- **Human-in-the-loop philosophy:** The AI never executes autonomously. Every recommendation is reviewed and approved by the engineer.
- **Context-aware observation:** The copilot understands the engineer's current screen, terminal, applications, and clipboard to provide relevant, timely assistance.
- **MCP-based extensibility:** Domain expertise is modularized as Model Context Protocol skills, making it easy to add new technologies.
- **Local-first knowledge:** All knowledge bases (vendor docs, SOPs, internal guides) are stored locally with vector search for privacy and speed.
- **Workspace-centric:** Workspaces are customer-specific, containing all context, history, and knowledge for a given engagement.
- **Desktop-native:** Runs on Windows and macOS as a native desktop application, not a web app or server installed in customer environments.

---

## Strategic Principles

1. **Safety first:** No autonomous execution. All AI-generated commands, scripts, or actions require explicit human approval.
2. **Privacy by design:** No data leaves the engineer's machine without explicit consent. No plaintext credentials. Local encryption everywhere.
3. **Extensibility over rigidity:** MCP skills and knowledge bases are the primary extension mechanism, not code changes.
4. **Pragmatic MVP:** Ship the simplest valuable thing first. Prove the observation→recommendation loop works before adding bells and whistles.
5. **Enterprise-ready:** Credential management via OS-native stores, audit logging, config migration, and update mechanisms from day one.
6. **AI-provider agnostic:** Support OpenAI, OpenAI-compatible APIs, vLLM, and local models. The AI layer is replaceable.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Desktop Application                      │
│  UI │ AI Chat │ Suggestions │ Skills │ Knowledge │ Settings │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│              Intent Recognition Engine                       │
│  Screen + Terminal + App Context + Conversation + History   │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│          MCP Skill Selection → Knowledge Retrieval           │
│  Skills: OpenShift, Linux, Windows, Ansible, Nagios,         │
│  VMware, MySQL, PostgreSQL, MSSQL, Checkmk, RHV, etc.       │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                AI Reasoning Engine                           │
│  Replaceable AI provider: OpenAI, vLLM, local models, etc.  │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│            Advisor Interface (Chat + Suggestions)            │
└─────────────────────────────────────────────────────────────┘

Observation Layer (always running, user-controlled):
  ┌───────────┐ ┌────────────┐ ┌───────────┐ ┌──────────┐
  │  Screen    │ │  Terminal  │ │  App Info  │ │ Clipboard│
  │ OCR + Vision│ │ Cmd/Output │ │ URL/Window │ │(optional)│
  └───────────┘ └────────────┘ └───────────┘ └──────────┘
```

**Reference Architecture Inspiration:** The desktop shell borrows the Rust+Tauri v2+React pattern from OpenHuman — Rust core for performance-critical operations (observation, security, native integration) and React for UI, with IPC via Tauri commands. However, Wiki Labs AI Copilot is a completely independent project with its own architecture tailored to engineering copiloting rather than personal AI assistance.

---

## Phase-Based Roadmap

### Phase 0 — Foundation (Q1 2026)

**Goal:** Establish technical foundations, validate core assumptions, and prove the observation→recommendation loop works.

**Focus areas:**

| Area | Deliverables |
|------|-------------|
| **Desktop Shell** | Tauri v2 + React scaffold, Rust core process, basic window management, settings UI skeleton |
| **AI Provider Layer** | Abstract AI provider interface, OpenAI integration, OpenAI-compatible API integration, provider selection in settings |
| **Basic UI** | AI chat interface, workspace switcher, skill picker, settings panel layout |
| **Knowledge System** | Local vector store (Chroma or LanceDB), document ingestion pipeline (PDF, Markdown, TXT), basic retrieval |
| **Security Foundation** | Local encryption (AES-256-GCM), credential placeholder, audit log skeleton, privacy controls UI |
| **Deployment** | Tauri build pipeline for Windows (MSI) and macOS (DMG), basic auto-update mechanism (tauri-plugin-updater) |

**Acceptance criteria for Phase 0:**
- Desktop app launches on Windows and macOS
- User can select an AI provider and chat with the copilot
- User can create a workspace, add knowledge documents, and see them used in context
- Single MCP skill (Linux) with basic commands works end-to-end
- Auto-update triggers a download and prompts for restart

**Timeline:** 12 weeks (3 months)  
**Team size estimate:** 4-5 engineers (2 Rust/TS, 1 React frontend, 1 AI/MCP backend, 1 DevOps)

**Dependencies:** None (foundation phase)

---

### Phase 1 — MVP (Q2 2026)

**Goal:** Deliver the Minimum Viable Product — a working copilot that provides real value to infrastructure engineers.

**Focus areas:**

| Area | Deliverables |
|------|-------------|
| **Observation Engine (Core)** | Screen capture with privacy controls (screenshot on demand, not continuous), active app detection (window title, URL), terminal command detection |
| **Intent Recognition** | Rule-based intent classification based on screen content + terminal commands + app context |
| **MCP Skills** | 3 production-ready skills: Linux, OpenShift, VMware vSphere |
| **Knowledge System** | Advanced retrieval with hybrid search (BM25 + vectors), SOP ingestion, knowledge versioning |
| **Engineering Workflows** | Basic troubleshooting workflow engine (symptom → evidence → hypothesis → validation → remediation) |
| **Workspace** | Full workspace system: customer name, tech stack, notes, previous sessions, recommendations history |
| **Security** | Windows Credential Manager integration, macOS Keychain integration, no-plaintext-credential enforcement |
| **UI Enhancements** | Real-time suggestions panel, terminal command suggestions, alert context injection |

**MVP Skill Set (Phase 1):**

| Skill | Scope |
|-------|-------|
| **Linux** | Performance troubleshooting, service management, log analysis, package management, kernel tuning |
| **OpenShift** | Cluster status checks, pod debugging, resource management, upgrade awareness, common error patterns |
| **VMware vSphere** | VM performance, resource pools, host status, datastore monitoring, common alerts |

**MVP Acceptance Criteria:**
- Engineer can open the app while doing their daily work
- Copilot observes screen + terminal and provides relevant suggestions in real-time
- User can ask questions in chat and get context-aware answers drawn from knowledge base + MCP skill
- Troubleshooting workflow is actionable and guides the engineer through a real scenario
- All credentials are stored in OS-native credential managers
- Auto-update works seamlessly

**Timeline:** 16 weeks (4 months)  
**Team size estimate:** 6-8 engineers (3 Rust/TS, 2 React, 2 AI/MCP, 1 QA)  
**Cumulative team size (Phase 0 + Phase 1):** 7-9 engineers

**Dependencies:** Phase 0 (desktop shell, AI provider layer, basic UI, knowledge system, security foundation, deployment pipeline)

---

### Phase 2 — Expansion (Q3-Q4 2026)

**Goal:** Expand skill coverage, enhance observation capabilities, and improve the quality of AI recommendations.

**Focus areas:**

| Area | Deliverables |
|------|-------------|
| **MCP Skills Expansion** | Add: Windows, Ansible, Nagios XI, Nagios Log Server, Checkmk, MySQL, EDB PostgreSQL, Microsoft SQL Server, Red Hat Virtualization |
| **Observation Engine (Advanced)** | Continuous screen observation with intelligent sampling, OCR for non-Digital UI elements, deeper app integration (browser URL monitoring, terminal session tracking, clipboard analysis) |
| **Intent Recognition (ML)** | Train intent classification model on real engineering session data, improve accuracy for ambiguous contexts |
| **Knowledge System** | Webhook-based knowledge updates, knowledge quality scoring, cross-reference linking, incident knowledge capture |
| **Engineering Workflows** | Technology-specific workflows (OpenShift upgrade workflow, VMware perf troubleshooting, Nagios alert investigation), custom workflow editor |
| **Reports & Analytics** | Session summary reports, recommendation acceptance rate, time saved estimates, knowledge utilization metrics |
| **Collaboration** | Share workspace notes, export troubleshooting sessions, team knowledge sharing |
| **Performance** | < 500ms suggestion latency, < 2s full context load, optimized memory usage, background process resource limits |

**Timeline:** 20 weeks (5 months)  
**Team size estimate:** 8-10 engineers  
**Cumulative team size:** 8-12 engineers

**Dependencies:** Phase 1 (all MVP components must be stable before expansion)

---

### Phase 3 — Maturity (Q1-Q2 2027)

**Goal:** Enterprise-grade features, advanced AI capabilities, and scale for large engineering organizations.

**Focus areas:**

| Area | Deliverables |
|------|-------------|
| **AI Reasoning** | Multi-step reasoning for complex troubleshooting, self-correction loop, confidence scores on recommendations |
| **Knowledge System** | Auto-extraction of knowledge from troubleshooting sessions, knowledge quality AI review, multi-tenant knowledge base (organization-wide) |
| **Integration Ecosystem** | Integration with ITSM tools (ServiceNow, Jira Service Management), monitoring systems (Grafana, Datadog), CI/CD pipelines, Slack/Teams notifications |
| **Administration** | Centralized policy management, skill deployment at organization level, audit dashboard, compliance reporting |
| **Accessibility** | Full WCAG 2.1 AA compliance, screen reader support, keyboard navigation, high-contrast mode |
| **Multi-language UI** | Localization framework, initial translations (Japanese, Korean, Simplified Chinese, French, German) |
| **Plugin System** | Community skill plugins, custom observation plugins, third-party MCP server support |

**Timeline:** 24 weeks (6 months)  
**Team size estimate:** 10-14 engineers

**Dependencies:** Phase 2 (all expanded skills and observation capabilities must be operational)

---

### Phase 7 — Engineering Intelligence (Q3 2026)

**Goal:** Add domain-aware engineering intelligence on top of the MVP — technology recognition, intent classification, workflow management, and a declarative skill system.

**Focus areas:**

| Area | Deliverables |
|------|-------------|
| **Technology Recognition** | Evidence-based detection from browser URL/title, terminal commands, active app, file patterns, workspace context, conversation keywords. 14 technology domains. |
| **Intent Recognition** | Technology-aware intent classification with continuous updating from observations, conversation, workspace. Human override always wins. |
| **Engineering Workflow Engine** | State machine tracking states, transitions, required evidence, confidence requirements, completion criteria, validation rules — all declarative via Skills. |
| **Context Fusion Engine** | Unified engineering context: observations + conversation + workspace + technology + intent + workflow state + timeline + human corrections. |
| **Confidence Engine** | Confidence scoring on every inference with automatic confirmation for low-confidence detections. |
| **Engineering Timeline** | Chronological activity tracking with references to source observation events. |
| **Recommendation Readiness** | Determines whether sufficient information exists before generating advice. |
| **Human Feedback Loop** | Correction tracking and intent override from direct human input. |
| **Declarative Skill System** | Dynamic skill loading with manifest validation, version management, enable/disable lifecycle, dependency checking. |
| **Skill Runtime** | Skill discovery, loading, validation, version management, dependency resolution from configurable directories. |
| **Skill SDK** | Complete development kit: template generator (8 templates), schema validator (7 JSON schemas), CLI scaffolding, developer docs. |

**Acceptance criteria:**
- Copilot detects technology stack with ≥80% accuracy across 14 domains
- Intent classification achieves ≥85% accuracy on engineering workflows
- Skills load dynamically from directory without application restart
- Confidence scoring provides actionable thresholds for human confirmation
- Declarative skill manifests validate correctly with all 7 schemas

**Timeline:** 8 weeks (2 months)  
**Team size estimate:** 3-4 engineers  
**Dependencies:** Phase 2 (core observation + skills infrastructure must be operational)

---

### Phase 8 — Knowledge Management (Q4 2026) — v0.5.0-alpha

**Goal:** Full knowledge pack management, embedding provider abstraction, and CLI tooling for knowledge-based assistance.

**Focus areas:**

| Area | Deliverables |
|------|-------------|
| **Knowledge Management UI** | Full sidebar knowledge pack management: create, edit, delete, import, validate, package. Context-aware suggestions tied to active workspace and tech stack. |
| **Embedding Provider Abstraction** | Pluggable `EmbeddingProvider` trait with `LocalEmbeddingProvider` and mock implementations. Swappable backends (OpenAI, local models, custom APIs). |
| **Knowledge Pack CLI** | `knowledge-cli` binary: `create-pack`, `validate`, `package`, `list-templates`. 3 predefined templates (openshift, engineering, documentation). |
| **Context-Aware Suggestions** | Sidebar displays relevant suggestions based on active context (screen, terminal, workspace). |
| **Knowledge Pack Import** | Import documents into workspace knowledge base from `.wkl` archives with automatic metadata extraction and indexing. |

**Acceptance criteria:**
- Knowledge pack UI supports full CRUD lifecycle with zero errors
- Embedding provider can be swapped without code changes
- CLI validates knowledge pack structure against all schemas
- Import pipeline correctly extracts metadata and indexes documents
- Context-aware suggestions appear with <200ms latency

**Timeline:** 6 weeks (1.5 months)  
**Team size estimate:** 2-3 engineers  
**Dependencies:** Phase 7 (technology recognition and intent must be operational for context-aware suggestions)

---

### Phase 9 — Copilot Engine (Q1 2027) — v0.6.0-alpha

**Goal:** Complete the observation→recommendation→approval loop with a fully autonomous decision engine, human-in-the-loop enforcement, and proactive assistance.

**Focus areas:**

| Area | Deliverables |
|------|-------------|
| **Copilot Engine** | Central orchestration for observation→recommendation→approval loop. AI never executes autonomously. |
| **Decision Engine** | 9-rule filter pipeline: confidence, evidence, user state, session limits, frequency limits, repetition avoidance, workflow relevance. |
| **Recommendation Engine** | 6-class observation classification, template-based generation, context-aware recommendations, auto-evidence, deduplication. |
| **Policy Engine** | 5-level spectrum (Minimal→Expert→Silent). Policy-specific confidence, priority, and cooldown thresholds. |
| **Lifecycle Manager** | 7-state recommendation state machine with invalid transition prevention and terminal state enforcement. |
| **Session Memory** | Personalization via acceptance/dismissal/correction tracking with topic-based confidence adjustment. |
| **Proactive Assistance** | 5 signal types, flooding prevention, urgency classification. |
| **Follow-Up Suggestions** | Keyword-based contextual next steps across 5 domains. |
| **Human Approval** | Human-in-the-loop enforcement with approval request lifecycle. |
| **Explainability** | Traceable reasoning with reason trees and evidence mapping. |
| **Recommendation Cards** | Display cards with actions (Explain/Documentation/Dismiss/Complete). |

**Safety constraints:**
- No autonomous execution — AI never runs commands or makes changes
- Human approval required for all recommendations
- Every recommendation includes evidence
- All decisions are traceable and explainable
- No repetition — same recommendation not shown twice
- Frequency limits: max 2 recommendations per minute
- Session limits: max 5 recommendations per session

**Acceptance criteria:**
- All 132 tests pass across 14 modules
- Decision engine correctly evaluates all 9 filters in priority order
- Policy engine enforces all 5 levels correctly
- Lifecycle state machine prevents all invalid transitions
- Proactive assistance prevents flooding (>10 signals/minute)
- Human approval gate blocks all autonomous actions
- Session memory correctly adjusts confidence on corrections

**Timeline:** 6 weeks (1.5 months)  
**Team size estimate:** 2-3 engineers  
**Dependencies:** Phase 7 (engineering intelligence must be operational), Phase 8 (knowledge management for context-aware recommendations)

---

### Phase 3+ — Growth (Q3 2027 Onward)

**Goal:** Scale, platform expansion, and ecosystem growth.

**Focus areas:**
- **AI Training Pipeline:** Fine-tuned domain models trained on engineering session data (with consent)
- **Cloud Assist:** Optional cloud-hosted companion for cross-device sync, team collaboration, and centralized knowledge (opt-in, encrypted)
- **Mobile Companion:** iOS/Android app for on-call engineers to query knowledge and review workspace context remotely
- **Marketplace:** Skill marketplace for third-party MCP skills and knowledge packs
- **Advanced Analytics:** Platform-wide usage analytics, skill effectiveness metrics, ROI reporting for management

---

## Phase Dependencies

```
Phase 0 ──────────────────┐
                         ▼
Phase 1 (MVP) ────────────┤
                         ▼
|Phase 2 (Expansion) ──────┤
                         │
                         ▼
Phase 7 (Intelligence) ───┤
                         │
                         ▼
Phase 8 (Knowledge) ──────┤
                         │
                         ▼
Phase 9 (Copilot Engine)──┤
                         ▼
Phase 3+ (Growth)
```

**Critical path dependencies:**
1. Phase 0 must deliver a working desktop shell before Phase 1 can begin any component work
2. Phase 1 MVP skills (Linux, OpenShift, VMware) must be production-ready before Phase 2 expansion
3. Observation engine quality from Phase 1 directly impacts intent recognition accuracy for Phase 2 ML models
4. Knowledge system maturity in Phase 1 determines the quality of AI responses in all subsequent phases
5. Security foundation from Phase 0 is mandatory for all phases — no phase can ship without it

---

## Risk Mitigation

### Phase 0 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Tauri v2 stability issues on macOS | Medium | High | Maintain Windows-first development; allocate buffer weeks for macOS; have Electron fallback plan |
| Vector DB integration complexity | Medium | Medium | Start with SQLite+FTS5 as MVP store; migrate to Chroma/LanceDB in Phase 1 if needed |
| AI provider abstraction too rigid | Low | Medium | Keep abstraction simple; add flexibility incrementally |
| Team onboarding slow (new tech: Rust+Tauri) | High | Medium | Pair programming; dedicated onboarding sprint; reference project (OpenHuman) as learning resource |

### Phase 1 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Observation engine privacy concerns | High | Critical | Privacy-first design; user-facing controls; transparent data flow; no recording without consent |
| Intent recognition accuracy too low | Medium | High | Start with rule-based system; collect data for ML model in Phase 2; fallback to chat-based intent |
| MCP skill quality inconsistent | Medium | Medium | Define skill quality checklist; peer review; automated testing per skill |
| AI hallucination in recommendations | High | Critical | Ground responses in knowledge base + MCP context; confidence scoring; clear attribution |
| Auto-update breaks user workflow | Medium | High | Staged rollouts; rollback mechanism; clear version history; graceful update prompts |

### Phase 2 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Skill expansion out of scope | High | Medium | Strict scope gate; new skills enter a backlog; only pre-approved skills in Phase 2 |
| ML intent model training data insufficient | Medium | Medium | Rule-based model remains primary; ML model is supplementary; plan for longer data collection |
| Performance degradation with more skills | Medium | Medium | Performance budget per skill; benchmarking pipeline; resource limits |
| Credential manager integration delays | Low | High | Early spike for each platform; vendor documentation review; have fallback encryption |

### Phase 3 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Multi-tenant knowledge base introduces privacy risks | Medium | Critical | Security review by independent team; encryption in transit and at rest; per-tenant access controls |
| Plugin system security vulnerabilities | Medium | High | Sandboxed plugin execution; plugin review process; plugin signing and verification |
| Localization quality issues | Medium | Medium | Professional translation service; native speaker review; community feedback loop |

---

## Go-to-Market Strategy

### Target Market

**Primary:** Enterprise infrastructure and DevOps engineering teams (20-500 engineers)  
**Secondary:** System integration partners and managed service providers  
**Tertiary:** Individual engineers and small teams

### Positioning

> "Your experienced senior engineer, always beside you — without the travel budget."

### Channels

1. **Direct Enterprise Sales:** Target IT directors and VP Engineering at mid-to-large enterprises
2. **Channel Partners:** SI partners and MSPs who serve multiple enterprise customers
3. **Content Marketing:** Technical blog posts, case studies, engineering deep-dives
4. **Community:** Open-source skill packages, community MCP server contributions, conference talks
5. **Free Tier / Trial:** Individual engineer license to drive adoption within teams, then expand to workspace/enterprise

### Pricing Strategy (Tentative)

| Tier | Audience | Features | Pricing Model |
|------|----------|----------|---------------|
| **Individual** | Single engineer | 3 skills, local knowledge, basic observation | Per-engineer/month |
| **Team** | Up to 20 engineers | All skills, workspace collaboration, reports | Per-engineer/month (volume discount) |
| **Enterprise** | 20+ engineers | Admin controls, multi-tenant knowledge, custom skills, SSO | Annual contract + per-engineer |

### Early Adopter Criteria

- Already uses AI tools in their workflow
- Manages infrastructure across 3+ technology domains
- Has documented SOPs and troubleshooting guides
- Uses modern tech stack (containerized apps, cloud/hybrid infrastructure)

---

## Growth and Expansion Plans

### Near-term (6-12 months)
- Grow skill set from 3 MVP skills to 10+ technologies
- Build reference architecture and case studies
- Establish partnerships with 2-3 system integrators

### Mid-term (12-24 months)
- Expand to 25+ skill technologies
- Build organization-wide knowledge base platform
- Develop plugin marketplace for third-party skills
- Achieve SOC 2 Type II compliance

### Long-term (24+ months)
- Platform play: AI Copilot as a platform for engineering knowledge
- Cloud-assisted features (optional, encrypted)
- Mobile companion app for on-call scenarios
- Training and certification program for engineering AI

---

## Key Milestones

| Date | Milestone | Deliverable |
|------|-----------|-------------|
| **2026-03** | Phase 0 Complete | Working desktop shell, AI provider integration, basic knowledge system |
| **2026-06** | MVP Complete | Full MVP with 3 skills, observation engine, workspace system |
| **2026-09** | First Public Beta | Release to early adopter group (10-20 engineers) |
| **2026-11** | Phase 2 Skills Complete | 10+ skills, advanced observation, ML intent recognition |
| **2027-01** | Version 1.0 | General availability release |
| **2027-03** | Enterprise Features Complete | Admin controls, multi-tenant knowledge, compliance reporting |
| **2027-06** | Version 2.0 | Plugin system, mobile companion, localization |

---

*See also: [MVP_SCOPE.md](./MVP_SCOPE.md) · [BACKLOG.md](./BACKLOG.md) · [RELEASE_PLAN.md](./RELEASE_PLAN.md)*