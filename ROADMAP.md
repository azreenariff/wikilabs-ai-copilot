# Wiki Labs AI Copilot — Product Roadmap

> **Last Updated:** 2026-07-21
> **Version:** 1.3.0
> **Status:** Phase 15 Complete
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

### Phase 1 — Foundation (Q1 2026)

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

**Acceptance criteria for Phase 1:**
- Desktop app launches on Windows and macOS
- User can select an AI provider and chat with the copilot
- User can create a workspace, add knowledge documents, and see them used in context
- Single MCP skill (Linux) with basic commands works end-to-end
- Auto-update triggers a download and prompts for restart

**Timeline:** 12 weeks (3 months)
**Team size estimate:** 4-5 engineers (2 Rust/TS, 1 React frontend, 1 AI/MCP backend, 1 DevOps)

**Dependencies:** None (foundation phase)

---

### Phase 2 — MVP (Q2 2026)

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

**MVP Skill Set (Phase 2):**

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
**Cumulative team size (Phase 1 + Phase 2):** 7-9 engineers

**Dependencies:** Phase 1 (desktop shell, AI provider layer, basic UI, knowledge system, security foundation, deployment pipeline)

---

### Phase 3 — Engineering Skills Pack (Q2 2026)

**Goal:** Deliver production-ready MCP skill packs with comprehensive knowledge bases, troubleshooting workflows, and engineering guidance.

**Focus areas:**

| Area | Deliverables |
|------|-------------|
| **Skill Pack Framework** | Structured skill pack format with manifest, technology definitions, detection rules, workflows, commands, guidance, best practices, and known issues |
| **Engineering Reasoning** | Evidence-based reasoning framework: observation → hypothesis → validation → remediation. Confidence scoring, risk assessment, safety constraints |
| **Knowledge Bases** | Technology-specific knowledge in structured markdown: cluster architecture, container runtime, networking, security, RBAC, SCC, operators |
| **Workflows** | State machine-based troubleshooting workflows with evidence collection, decision trees, commands, and risk levels |
| **Detection Rules** | CLI command detection, browser URL detection, text pattern matching with confidence scoring |
| **Command Knowledge** | Structured command reference with purpose, risk assessment, parameters, output interpretation |
| **Context Interpretation** | Context-aware guidance: terminal, browser, text patterns, mixed contexts |

**Phase 3 Deliverables:**

| Deliverable | Description |
|-------------|-------------|
| **OpenShift Engineering Skill Pack** | Complete skill pack for Red Hat OpenShift 4.x: 40 files, 17 subdirectories, 16+ detection rules, 811+ line workflow definitions, 160+ technical commands |
| **Linux Engineering Skill Pack** | Complete skill pack for Linux administration: 40 files, 17 subdirectories, 10+ detection rules, 1400+ line workflow definitions, 120+ technical commands |
| **VMware vSphere Engineering Skill Pack** | Complete skill pack for VMware vSphere: 40 files, 17 subdirectories, 8+ detection rules, 800+ line workflow definitions, 60+ technical commands |
| **Skill Pack Template** | Reusable template for creating new skill packs in the Wiki Labs format |

**Acceptance Criteria:**
- Each skill pack contains all 7 core sections (manifest, technology, detection, workflows, commands, guidance, knowledge)
- Detection rules achieve ≥0.90 confidence for known patterns
- Workflows include risk assessment, safety warnings, and rollback steps
- Knowledge base covers at least 80% of common failure scenarios
- Skills load and activate correctly via the Skill Discovery and Activation engines
- Skills integrate with the Guidance Engine for context-aware recommendations

**Timeline:** 4 weeks (1 month)
**Team size estimate:** 2-3 engineers (1 Rust/backend, 1 OpenShift expert, 1 technical writer)
**Cumulative team size:** 9-12 engineers

**Dependencies:** Phase 2 (Skill Discovery Engine, Skill Activation Engine, Skill Runtime, Guidance Engine, Engineering Intelligence Engine must all be operational)

---

### Phase 4 — Expansion (Q3-Q4 2026)

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

**Dependencies:** Phase 3 (all skills must be stable before expansion)

---

### Phase 5 — Maturity (Q1-Q2 2027)

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

**Dependencies:** Phase 4 (all expanded skills and observation capabilities must be operational)

---

### Phase 6 — Growth (Q3 2027 Onward)

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
Phase 1 ──────────────────┐
                         ▼
Phase 2 (MVP) ────────────┤
                         ▼
Phase 3 (Skills Pack) ────┤
                         ▼
Phase 4 (Expansion) ──────┤
                         ▼
Phase 5 (Maturity) ───────┤
                         ▼
Phase 6 (Growth)
```

**Critical path dependencies:**
1. Phase 1 must deliver a working desktop shell before Phase 2 can begin any component work
2. Phase 2 MVP skills (Linux, OpenShift, VMware) must be production-ready before Phase 4 expansion
3. Phase 3 engineering skills packs must be complete before Phase 4 expansion
4. Observation engine quality from Phase 2 directly impacts intent recognition accuracy for Phase 4 ML models
5. Knowledge system maturity in Phase 2 determines the quality of AI responses in all subsequent phases
6. Security foundation from Phase 1 is mandatory for all phases — no phase can ship without it

---

## Risk Mitigation

### Phase 1 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Tauri v2 stability issues on macOS | Medium | High | Maintain Windows-first development; allocate buffer weeks for macOS; have Electron fallback plan |
| Vector DB integration complexity | Medium | Medium | Start with SQLite+FTS5 as MVP store; migrate to Chroma/LanceDB in Phase 2 if needed |
| AI provider abstraction too rigid | Low | Medium | Keep abstraction simple; add flexibility incrementally |
| Team onboarding slow (new tech: Rust+Tauri) | High | Medium | Pair programming; dedicated onboarding sprint; reference project (OpenHuman) as learning resource |

### Phase 2 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Observation engine privacy concerns | High | Critical | Privacy-first design; user-facing controls; transparent data flow; no recording without consent |
| Intent recognition accuracy too low | Medium | High | Start with rule-based system; collect data for ML model in Phase 4; fallback to chat-based intent |
| MCP skill quality inconsistent | Medium | Medium | Define skill quality checklist; peer review; automated testing per skill |
| AI hallucination in recommendations | High | Critical | Ground responses in knowledge base + MCP context; confidence scoring; clear attribution |
| Auto-update breaks user workflow | Medium | High | Staged rollouts; rollback mechanism; clear version history; graceful update prompts |

### Phase 3 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Skill pack template too rigid for diverse technologies | Medium | Medium | Validate template against 3 diverse skill packs (Linux, OpenShift, VMware) before lock-down |
| Knowledge base content outdated or incomplete | Medium | High | Expert review process; versioned knowledge with change tracking; update triggers from vendor releases |
| Detection rules miss real-world patterns | Medium | High | Test detection rules against production clusters; expand rule set based on failure feedback |
| Workflow coverage incomplete for edge cases | Low | Medium | Document known gaps; allow skill authors to extend workflows; gather feedback from pilot users |
| Large skill packs exceed memory limits | Low | High | Implement lazy loading for knowledge sub-directories; profile memory usage with full skill pack loaded |

### Phase 4 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Skill expansion out of scope | High | Medium | Strict scope gate; new skills enter a backlog; only pre-approved skills in Phase 4 |
| ML intent model training data insufficient | Medium | Medium | Rule-based model remains primary; ML model is supplementary; plan for longer data collection |
| Performance degradation with more skills | Medium | Medium | Performance budget per skill; benchmarking pipeline; resource limits |
| Credential manager integration delays | Low | High | Early spike for each platform; vendor documentation review; have fallback encryption |

### Phase 5 Risks

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
| **2026-03** | Phase 1 Complete | Working desktop shell, AI provider integration, basic knowledge system |
| **2026-06** | Phase 2 Complete | Full MVP with 3 skills, observation engine, workspace system |
| **2026-07** | Phase 3 Complete | 3 production-ready skill packs (Linux, OpenShift, VMware) with comprehensive knowledge bases |
| **2026-08** | First Public Beta | Release to early adopter group (10-20 engineers) |
| **2026-11** | Phase 4 Skills Complete | 10+ skills, advanced observation, ML intent recognition |
| **2027-01** | Version 1.0 | General availability release |
| **2027-03** | Phase 5 Complete | Admin controls, multi-tenant knowledge, compliance reporting |
| **2027-06** | Phase 6 Complete | Plugin system, mobile companion, localization |

---

*See also: [MVP_SCOPE.md](./MVP_SCOPE.md) · [BACKLOG.md](./BACKLOG.md) · [RELEASE_PLAN.md](./RELEASE_PLAN.md)*