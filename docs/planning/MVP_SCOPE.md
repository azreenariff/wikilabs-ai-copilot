# Wiki Labs AI Copilot — MVP Scope

> **Last Updated:** 2025-07-16  
> **Version:** 1.0  
> **Status:** Planning  
> **Owner:** Technical Lead

---

## Table of Contents

1. [Definition of MVP](#definition-of-mvp)
2. [MVP Scope Boundaries](#mvp-scope-boundaries)
3. [MVP Feature List](#mvp-feature-list)
4. [MVP Acceptance Criteria](#mvp-acceptance-criteria)
5. [MVP Technology Stack](#mvp-technology-stack)
6. [MVP Deployment Model](#mvp-deployment-model)
7. [MVP Success Metrics](#mvp-success-metrics)
8. [MVP User Testing Plan](#mvp-user-testing-plan)
9. [Risks and Mitigations](#risks-and-mitigations)
10. [MVP Timeline Estimate](#mvp-timeline-estimate)

---

## Definition of MVP

The MVP (Minimum Viable Product) of Wiki Labs AI Copilot is the **smallest set of features that delivers genuine, measurable value to an infrastructure engineer doing their daily work**, while validating the core product hypothesis:

> **Core Hypothesis:** If an AI tool can observe what an infrastructure engineer is doing (screen, terminal, applications) and provide context-aware, skill-grounded recommendations and answers in real-time, the engineer will use it consistently and find it valuable.

The MVP is **not** the finished product. It is a focused experiment that proves:
1. Engineers want the copilot experience
2. The observation→intent→recommendation loop provides real value
3. The MCP skill system works as designed
4. The knowledge system retrieves relevant information effectively
5. The security and privacy model works in practice

**Philosophy reminder:** The MVP is a tool, not an agent. The human remains responsible for ALL actions. The AI provides recommendations, not commands it executes itself.

---

## MVP Scope Boundaries

### What IS in the MVP

| Category | In Scope |
|----------|----------|
| **Platform** | Windows 11, macOS Sonoma (latest major versions) |
| **Desktop App** | Tauri v2 + React desktop application with native shell |
| **AI Chat** | Full conversational interface with context awareness |
| **Observation Engine** | Screen capture (on-demand), active app detection, terminal command tracking, optional clipboard observation |
| **Intent Recognition** | Rule-based intent classification (screen content + terminal + app context) |
| **MCP Skills** | 3 skills: Linux, OpenShift, VMware vSphere |
| **Knowledge System** | Local vector store, document ingestion (PDF, Markdown, TXT), hybrid search (BM25 + vectors) |
| **Engineering Workflows** | Basic troubleshooting workflow (symptom → evidence → hypothesis → validation → remediation → verify → document) |
| **Workspace** | Per-customer workspace with tech stack, notes, session history, recommendation history |
| **Security** | Local encryption (AES-256-GCM), Windows Credential Manager integration, macOS Keychain integration, audit log, privacy controls |
| **AI Provider** | OpenAI API and OpenAI-compatible API support |
| **Update Mechanism** | Basic auto-update (download prompt + restart) |
| **Settings** | AI provider config, skill toggles, observation toggles, privacy controls, workspace management |

### What IS NOT in the MVP

| Category | Explicitly Out of Scope |
|----------|------------------------|
| **Additional Skills** | Windows, Ansible, Nagios XI, Nagios Log Server, Checkmk, MySQL, EDB PostgreSQL, Microsoft SQL Server, Red Hat Virtualization — these are Phase 2 items |
| **ML-Based Intent Recognition** | Rule-based only in MVP; ML model is Phase 2 |
| **Continuous Screen Observation** | On-demand/screen-on-focus only; no background screen recording or intelligent sampling |
| **Advanced Browser Integration** | No DOM inspection, no page content extraction, no browser extension |
| **Webhook-Based Knowledge Updates** | Manual document ingestion only; no webhook/automated updates |
| **Knowledge Quality AI Review** | Manual curation only |
| **Reports & Analytics** | No session reports, acceptance rate metrics, or time-saved estimates |
| **Collaboration Features** | No sharing notes, no export, no team knowledge sharing |
| **Multi-Tenant Knowledge** | Single workspace per engineer; no organization-wide knowledge base |
| **Plugin System** | No community or third-party plugins |
| **Multi-Language UI** | English-only |
| **Accessibility Compliance** | WCAG compliance is Phase 3 |
| **Mobile Companion** | No mobile app |
| **Cloud-Assisted Features** | Everything is local-only; no cloud sync |
| **ITSM Integrations** | No ServiceNow, Jira, Grafana, Datadog integrations |
| **SSO / Enterprise Auth** | No SAML, OIDC, or LDAP integration |
| **Admin Dashboard** | No centralized admin controls |
| **AI Fine-Tuning Pipeline** | No custom-trained models |

---

## MVP Feature List

### F1 — Desktop Application Shell

| Feature | Description | Priority |
|---------|-------------|----------|
| **F1.1 — Main Window** | Tauri v2 window with resizable layout, sidebar navigation, and content area | P0 |
| **F1.2 — Sidebar** | Navigation for Chat, Suggestions, Skills, Knowledge, Workspaces, Settings, Logs | P0 |
| **F1.3 — Settings Panel** | AI provider configuration, API key input (encrypted), skill toggles, privacy controls | P0 |
| **F1.4 — Workspace Manager** | Create, switch, rename, and delete workspaces | P0 |
| **F1.5 — About/Version** | Display app version, build info, check for updates | P1 |
| **F1.6 — Dark Theme** | Professional dark theme with engineering-tool aesthetic | P1 |

### F2 — AI Chat Interface

| Feature | Description | Priority |
|---------|-------------|----------|
| **F2.1 — Chat Window** | Full conversational chat with message history | P0 |
| **F2.2 — Context Injection** | Automatically include relevant screen/app/terminal context in prompts | P0 |
| **F2.3 — Knowledge Citation** | Show knowledge sources used in AI responses | P0 |
| **F2.4 — Command Suggestions** | Display suggested terminal commands in chat with copy button | P0 |
| **F2.5 — Message Actions** | Copy, regenerate, flag as unhelpful (feedback) | P1 |
| **F2.6 — Streaming Responses** | Real-time token streaming for AI responses | P0 |
| **F2.7 — Conversation History** | Persist and load chat sessions per workspace | P0 |
| **F2.8 — Clear Context** | Button to clear context from current conversation | P1 |
| **F2.9 — Error Handling** | Graceful error display for AI provider failures | P0 |

### F3 — Observation Engine

| Feature | Description | Priority |
|---------|-------------|----------|
| **F3.1 — Screen Capture** | Capture current screen on-demand or when window receives focus | P0 |
| **F3.2 — Screen OCR** | Extract text from screen capture for context | P0 |
| **F3.3 — Active App Detection** | Detect foreground application, window title, and process name | P0 |
| **F3.4 — Browser URL Detection** | Detect current browser and URL in active window | P1 |
| **F3.5 — Terminal Command Tracking** | Monitor terminal windows for command input and output | P0 |
| **F3.6 — Clipboard Observation** | (Optional, user-enabled) Observe clipboard content | P1 |
| **F3.7 — Observation Controls** | User-facing enable/disable toggles for each observation source | P0 |
| **F3.8 — Privacy Mode** | Single-click "pause observation" that hides all data collection | P0 |

### F4 — Intent Recognition Engine

| Feature | Description | Priority |
|---------|-------------|----------|
| **F4.1 — Rule-Based Classification** | Classify current engineering context based on screen content + terminal commands + app info | P0 |
| **F4.2 — Intent Display** | Show detected intent to user in UI (e.g., "Troubleshooting Linux performance issue") | P1 |
| **F4.3 — Intent Confidence** | Show confidence level of intent classification | P1 |
| **F4.4 — Intent Override** | User can correct misidentified intent | P1 |
| **F4.5 — Context History** | Maintain intent history across sessions | P1 |
| **F4.6 — Intent-Driven Suggestions** | Surface relevant skills and knowledge based on detected intent | P0 |

### F5 — MCP Skill Architecture

| Feature | Description | Priority |
|---------|-------------|----------|
| **F5.1 — Skill System** | MCP-based skill execution framework with metadata, knowledge refs, and troubleshooting workflows | P0 |
| **F5.2 — Linux Skill** | Full skill package: Linux performance troubleshooting, service management, log analysis, package management | P0 |
| **F5.3 — OpenShift Skill** | Full skill package: cluster checks, pod debugging, resource management, common errors | P0 |
| **F5.4 — VMware Skill** | Full skill package: VM performance, resource pools, host status, datastore monitoring | P0 |
| **F5.5 — Skill Selector** | UI to view available skills, toggle on/off, see skill status | P0 |
| **F5.6 — Skill Context Injection** | Automatically include relevant skill context when skill is applicable to detected intent | P0 |
| **F5.7 — Skill Output Formatting** | Present skill results (commands, analysis) in readable format in chat | P0 |
| **F5.8 — Command Safety** | All commands displayed for review before execution; never auto-executed | P0 |
| **F5.9 — Skill Tests** | Automated test suite for each skill's knowledge and troubleshooting workflows | P1 |
| **F5.10 — Skill Documentation** | Internal documentation for each skill's scope, knowledge sources, and workflows | P1 |

### F6 — Knowledge System

| Feature | Description | Priority |
|---------|-------------|----------|
| **F6.1 — Local Vector Store** | Embedded vector database for knowledge storage (Chroma or LanceDB) | P0 |
| **F6.2 — Document Ingestion** | Upload and ingest PDF, Markdown, TXT files into knowledge base | P0 |
| **F6.3 — Knowledge Chunking** | Automatic document splitting with context preservation | P0 |
| **F6.4 — Embedding Generation** | Embed knowledge documents using configurable embedding model | P0 |
| **F6.5 — Hybrid Search** | BM25 + vector hybrid retrieval for knowledge queries | P0 |
| **F6.6 — Knowledge UI** | Browse, search, edit, and delete knowledge entries | P0 |
| **F6.7 — Knowledge in Chat** | Show cited knowledge sources in AI responses | P0 |
| **F6.8 — Knowledge Versioning** | Track versions of knowledge documents; rollback capability | P1 |
| **F6.9 — Knowledge Quality** | Basic deduplication and conflict detection for overlapping knowledge | P1 |
| **F6.10 — Knowledge Export** | Export knowledge base in portable format | P2 |

### F7 — Engineering Workflow Engine

| Feature | Description | Priority |
|---------|-------------|----------|
| **F7.1 — Workflow Steps** | Core workflow: Understand symptom → Gather evidence → Form hypothesis → Validate hypothesis → Recommend remediation → Verify result → Document findings | P0 |
| **F7.2 — Workflow Progress** | Visual progress indicator through workflow stages in chat | P0 |
| **F7.3 — Evidence Collection** | Prompt engineer to gather relevant evidence (logs, metrics, config) | P0 |
| **F7.4 — Hypothesis Builder** | Guide engineer through forming and testing hypotheses | P0 |
| **F7.5 — Remediation Checker** | Before recommending action, show safety assessment and verification steps | P0 |
| **F7.6 — Session Documentation** | Auto-generate session notes based on workflow completion | P1 |

### F8 — Workspace System

| Feature | Description | Priority |
|---------|-------------|----------|
| **F8.1 — Create Workspace** | Create a new customer workspace with name, description, and tech stack | P0 |
| **F8.2 — Tech Stack Tags** | Tag workspace with applicable technologies (Linux, OpenShift, VMware, etc.) | P0 |
| **F8.3 — Workspace Switching** | Switch between workspaces; context and knowledge are workspace-scoped | P0 |
| **F8.4 — Workspace Notes** | Add free-form notes to workspace; notes are included in context | P0 |
| **F8.5 — Session History** | View and replay previous chat sessions per workspace | P0 |
| **F8.6 — Recommendation History** | Track and review all recommendations made in workspace | P1 |
| **F8.7 — Workspace Info** | View workspace metadata (created date, skills, knowledge count) | P1 |

### F9 — Security

| Feature | Description | Priority |
|---------|-------------|----------|
| **F9.1 — Local Encryption** | AES-256-GCM encryption for all local data (knowledge, sessions, credentials) | P0 |
| **F9.2 — Credential Manager** | Windows Credential Manager integration for credential storage | P0 |
| **F9.3 — Keychain Integration** | macOS Keychain integration for credential storage | P0 |
| **F9.4 — No Plaintext Credentials** | Enforcement: credentials never stored in plaintext anywhere (disk, memory dumps, logs) | P0 |
| **F9.5 — Audit Log** | Log all significant actions (skill execution, credential access, knowledge changes) | P0 |
| **F9.6 — Privacy Controls** | User-facing toggles for each observation source; clear data flow display | P0 |
| **F9.7 — Data Export** | Export all local data (knowledge, sessions, notes) for backup/migration | P1 |
| **F9.8 — Data Purge** | Purge all local data on uninstall or explicit user request | P1 |

### F10 — AI Provider Layer

| Feature | Description | Priority |
|---------|-------------|----------|
| **F10.1 — OpenAI Integration** | Connect to OpenAI API (gpt-4o, gpt-4o-mini) | P0 |
| **F10.2 — OpenAI-Compatible API** | Connect to any OpenAI-compatible endpoint (vLLM, local models) | P0 |
| **F10.3 — Provider Selection** | Choose and switch between AI providers in settings | P0 |
| **F10.4 — Model Selection** | Select specific model within each provider | P0 |
| **F10.5 — API Key Management** | Securely store API keys in credential manager | P0 |
| **F10.6 — Connection Test** | Test AI provider connectivity before saving | P0 |
| **F10.7 — Rate Limit Handling** | Graceful handling of API rate limits with retry logic | P1 |
| **F10.8 — API Cost Display** | Display estimated token usage per response | P1 |

### F11 — Update Mechanism

| Feature | Description | Priority |
|---------|-------------|----------|
| **F11.1 — Version Check** | Check for updates on startup and manually | P0 |
| **F11.2 — Update Download** | Download update packages in background | P0 |
| **F11.3 — Update Prompt** | Notify user of available update with changelog | P0 |
| **F11.4 — Update Installation** | Install update with user confirmation and auto-restart | P0 |

---

## MVP Acceptance Criteria

The MVP is considered complete when **all** of the following criteria are met:

### Functional Criteria

- [ ] **F0.1** Engineer can install the desktop app on Windows 11 and macOS Sonoma
- [ ] **F0.2** Engineer can create a workspace, add knowledge documents (PDF, Markdown), and configure an AI provider
- [ ] **F0.3** When the engineer opens the copilot while working, it detects the active application, terminal commands, and screen content
- [ ] **F0.4** The copilot displays relevant suggestions based on detected intent (screen + terminal + app context)
- [ ] **F0.5** The engineer can ask questions in chat and receive answers grounded in knowledge base content and MCP skill knowledge
- [ ] **F0.6** The Linux skill provides actionable troubleshooting guidance for at least 10 common Linux scenarios
- [ ] **F0.7** The OpenShift skill provides actionable troubleshooting guidance for at least 10 common OpenShift scenarios
- [ ] **F0.8** The VMware skill provides actionable troubleshooting guidance for at least 10 common VMware scenarios
- [ ] **F0.9** The engineering workflow engine guides the engineer through a complete troubleshooting session from symptom to documented findings
- [ ] **F0.10** All credentials (API keys, connection details) are stored exclusively in OS-native credential managers
- [ ] **F0.11** Screen capture requires user consent and can be paused at any time
- [ ] **F0.12** Auto-update downloads and installs new versions with user confirmation
- [ ] **F0.13** The audit log records all significant user actions and system events

### Quality Criteria

- [ ] **Q0.1** AI response latency is under 3 seconds for simple queries, under 10 seconds for complex queries
- [ ] **Q0.2** Suggestion latency (time from screen capture to displayed suggestion) is under 2 seconds
- [ ] **Q0.3** Knowledge retrieval returns relevant results in under 1 second for knowledge bases up to 500 documents
- [ ] **Q0.4** App startup time is under 3 seconds on a standard engineer laptop (8GB RAM, SSD)
- [ ] **Q0.5** No JavaScript console errors or Rust panics in normal operation
- [ ] **Q0.6** The app handles API provider unavailability gracefully (clear error message, no crash)

### Security Criteria

- [ ] **S0.1** No plaintext credentials found in any local file, log, or memory snapshot
- [ ] **S0.2** All local data is encrypted at rest (known files only)
- [ ] **S0.3** Screen capture and clipboard observation are off by default
- [ ] **S0.4** Audit log entries include timestamp, action, user, and outcome

---

## MVP Technology Stack

### Decision Matrix

| Component | Decision | Rationale |
|-----------|----------|-----------|
| **Desktop Framework** | Tauri v2 | Rust core for performance/security, React for UI, native OS integration, small binary size, open-source. Reference: OpenHuman uses same pattern successfully. |
| **Backend Language** | Rust | Performance-critical operations (observation, encryption, native integrations). Safety guarantees. Small memory footprint. |
| **Frontend** | React 18 + TypeScript | Rich UI components, Tauri TypeScript bindings, large ecosystem, maintainable. |
| **State Management** | Zustand | Lightweight, TypeScript-first, minimal boilerplate compared to Redux. |
| **Build Tool** | Vite (frontend), Cargo (Rust) | Fast builds, excellent DX, industry standard. |
| **Vector Database** | Chroma (embedded) | Local-first, Python/Rust support, good embedding model compatibility, embeddable (no server process). Alternative: LanceDB. |
| **Embedding Model** | OpenAI text-embedding-3-small (configurable) | Best accuracy/performance ratio for MVP. Configurable to local models in settings. |
| **UI Framework** | shadcn/ui + Tailwind CSS | Professional, accessible, customizable, Tauri-friendly, dark theme ready. |
| **IPC** | Tauri Commands | Rust backend commands exposed to React frontend via Tauri IPC. |
| **Local Storage** | SQLite (local) + Encrypted Files (sensitive) | SQLite for structured data (sessions, workspaces, settings). Encrypted files for knowledge documents and credentials. |
| **Auto-Update** | tauri-plugin-updater | Built-in Tauri update plugin; supports code signing for Windows notarization and macOS notarization. |
| **Logging** | tracing (Rust) + Logtail (frontend) | Structured Rust logging; simple frontend logging. Local audit log file. |
| **Testing** | Vitest (frontend), cargo test (Rust), Playwright (E2E) | Unified test framework; browser-based E2E for critical user flows. |
| **CI/CD** | GitHub Actions | Free for open-source, GitHub-hosted runners for Windows/macOS/Linux. |

### Why Not Other Options

| Option | Why Not Chosen |
|--------|---------------|
| Electron + Node.js | Larger binary, higher memory usage, less safe for credential management |
| Flutter | Smaller ecosystem for AI/ML integration, less mature Rust FFI |
| Native (Win32 + AppKit) | Much slower development, duplicated effort across platforms |
| Web app only | No native OS integration (credential manager, screen capture), no offline-first capability |
| SQLite only (no vector) | No semantic search capability; hybrid search requires vector component |

---

## MVP Deployment Model

### Distribution

| Platform | Format | Mechanism |
|----------|--------|-----------|
| **Windows 11** | MSI installer | Microsoft Store (long-term), direct MSI download (MVP) |
| **macOS Sonoma** | DMG package | Direct DMG download, notarized with Apple notarization service |
| **Linux** | _Not in MVP scope_ | Planned for Phase 2+ |

### Installation

1. Engineer downloads installer from wiki-labs.ai (or internal portal for enterprise)
2. Runs installer (MSI/DMG) — installs to standard user location
3. First launch: onboarding wizard (create workspace, configure AI provider, accept privacy policy)
4. App appears in system tray (Windows) / menu bar (macOS)

### Configuration

- All configuration stored in `~/.config/wiki-labs-copilot/` (cross-platform path abstraction)
- Workspace data in `~/.config/wiki-labs-copilot/workspaces/`
- Knowledge documents in `~/.config/wiki-labs-copilot/knowledge/` (encrypted)
- Sessions in `~/.config/wiki-labs-copilot/sessions/` (encrypted)
- AI provider keys in OS credential manager (not in config files)

### Updates

- Auto-update check on startup and via "Check for Updates" in Settings
- Updates downloaded in background, user notified and can install with confirmation
- Configuration preserved across updates
- Version migration handled automatically

---

## MVP Success Metrics

### User Engagement Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Daily Active Users (DAU)** | 70% of pilot users use copilot daily | In-app telemetry (opt-in) |
| **Sessions per Day** | Average 5+ sessions per active user | Local session log |
| **Time to First Value** | Under 5 minutes from install to first useful interaction | User testing observation |
| **Feature Adoption** | 80% of users use observation + suggestions within first week | Feature usage log |
| **Retention** | 60% of pilot users still active after 30 days | Weekly active user tracking |

### Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Intent Recognition Accuracy** | 70% accuracy on known scenarios | Labelled test set evaluation |
| **Knowledge Retrieval Relevance** | 80% of retrieved documents are relevant to query | Manual evaluation of sample queries |
| **Recommendation Helpfulness** | 70% of users rate recommendations as "helpful" or "very helpful" | In-app feedback (thumbs up/down) |
| **AI Response Latency** | Under 3 seconds for simple queries, under 10 seconds for complex | Performance monitoring |
| **Suggestion Latency** | Under 2 seconds from context change to displayed suggestion | Performance monitoring |
| **Crash-Free Rate** | 99%+ crash-free sessions | Error reporting |

### Business Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Pilot User Satisfaction** | Net Promoter Score (NPS) ≥ 30 | Post-MVP survey |
| **Time Saved** | Engineers report ≥ 30 min/day saved | Survey (self-reported) |
| **Reduction in Escalations** | 15% reduction in Level 1→Level 2 escalations | Customer feedback (if applicable) |
| **Pilot-to-Paid Conversion** | ≥ 50% of pilot users upgrade to paid tier | Conversion tracking |

---

## MVP User Testing Plan

### Phase 1 — Internal Testing (Weeks 1-6 of MVP timeline)

**Participants:** Development team (3-5 engineers)  
**Duration:** 2 weeks  
**Method:** Developers use copilot during their daily work sessions

**Test Scenarios:**
1. Developer debugging a Linux server issue — copilot observes terminal, suggests commands
2. Developer troubleshooting an OpenShift pod crash — copilot identifies context, provides skill-based guidance
3. Developer investigating VMware host performance — copilot detects app, surfaces relevant knowledge
4. Developer asking knowledge-base question about internal SOP
5. Developer testing privacy controls (pause observation, disable clipboard)

**Success Criteria:**
- No crashes or data loss during testing
- At least 3 usability issues found and fixed
- AI responses are relevant and actionable
- Privacy controls work as expected

### Phase 2 — Alpha Testing (Weeks 7-10 of MVP timeline)

**Participants:** 5 internal engineers (not on dev team)  
**Duration:** 2 weeks  
**Method:** Engineers use copilot for real work tasks, with daily check-ins

**Test Scenarios:**
1. Real-world troubleshooting session with Linux server issue
2. Real-world OpenShift deployment issue investigation
3. Real-world VMware performance investigation
4. Ad-hoc knowledge questions about internal SOPs
5. Multi-workspace workflow (switching between customer contexts)

**Data Collection:**
- Daily 15-minute interviews
- In-app feedback (thumbs up/down on every response)
- Session recordings (with consent) of copilot usage
- Pre- and post-test survey on perceived time savings

**Success Criteria:**
- At least 3 engineers report finding the copilot "useful" or "very useful"
- Average helpfulness rating ≥ 3.5/5
- At least 50% of users say they would use it again tomorrow
- No security or privacy incidents

### Phase 3 — Beta Testing (Weeks 11-14 of MVP timeline)

**Participants:** 10 external engineers (early adopter partners)  
**Duration:** 2 weeks  
**Method:** Engineers use copilot in production environment with structured testing protocol

**Test Scenarios:**
- Engineers choose their own real-world tasks (no scripted scenarios)
- Pre-task and post-task surveys
- Weekly structured interview sessions
- End-of-betasurvey

**Success Criteria:**
- NPS ≥ 25
- 60%+ of beta users say "I can't work without this tool" or "This is essential to my workflow"
- Knowledge base relevance ≥ 75%
- Intent recognition accuracy ≥ 65% on real-world scenarios
- No critical security issues
- Zero data leaks or privacy violations

### Post-MVP Review

After beta testing concludes, the team conducts a structured review:
1. What worked well (keep investing)
2. What didn't work (fix or de-scope)
3. What surprised us (unexpected use cases, features)
4. What we learned about our users
5. Adjustments to Phase 2 roadmap based on findings

---

## Risks and Mitigations

### Technical Risks

| Risk | Likelihood | Impact | Severity | Mitigation |
|------|-----------|--------|----------|------------|
| **Observation accuracy too low** — copilot misinterprets screen content or terminal commands | High | High | Critical | Start conservative: focus on high-confidence detections. User can correct intent. Collect data for ML model in Phase 2. |
| **AI hallucination in recommendations** — AI generates plausible but incorrect commands or advice | High | Critical | Critical | Ground all responses in knowledge base + skill context. Show citations. Confidence scoring. Warning: "Always verify commands before execution." |
| **Rust + Tauri learning curve** — team unfamiliar with Rust ecosystem for Tauri development | Medium | Medium | Medium | Reference project (OpenHuman) as learning resource. Pair programming. Dedicated onboarding sprint. |
| **Vector DB memory usage too high** — Chroma/LanceDB consumes excessive RAM on engineer laptops | Medium | Medium | Medium | Set memory limits. Use efficient embedding models. Benchmark on reference hardware early. |
| **Screen OCR accuracy poor on complex UIs** — terminal output, Grafana dashboards, VMware UIs are hard to OCR | Medium | High | High | Use vision models (not just OCR) for screen understanding. Allow manual context entry as fallback. |

### Security & Privacy Risks

| Risk | Likelihood | Impact | Severity | Mitigation |
|------|-----------|--------|----------|------------|
| **Credential leak through screen capture** — sensitive passwords visible on screen when captured | Low | Critical | Critical | Privacy-first design: no screen capture without user action. Blur detection for credential patterns. User can pause at any time. |
| **Local encryption insufficient** — keys stored in a way that allows extraction | Low | Critical | Critical | Use OS-native credential stores as primary. Fallback encryption uses system key material, not user-defined passwords. |
| **Knowledge base contains sensitive data** — user uploads documents with secrets, credentials, or classified info | Medium | High | High | No automatic scraping of knowledge documents. User explicitly curates knowledge. Warning: "Don't upload documents containing secrets." |
| **Audit log contains sensitive data** — logged actions include terminal output or screen content | Medium | High | High | Audit log contains metadata only (what happened, when, outcome). No raw terminal output, no screenshots in logs. |

### Product Risks

| Risk | Likelihood | Impact | Severity | Mitigation |
|------|-----------|--------|----------|------------|
| **Engineers don't want ongoing observation** — privacy anxiety or privacy concerns outweigh value | High | High | Critical | Make privacy controls obvious, accessible, and reversible. Start with on-demand observation, add continuous in Phase 2. |
| **AI provides generic advice** — copilot feels like a generic chatbot, not domain-specific | Medium | High | High | Strong MCP skill integration. Knowledge grounding. Context injection from screen + terminal. Specific skill output formatting. |
| **Observation feels "creepy"** — engineers feel watched rather than assisted | Medium | Medium | High | Transparent UI showing what's being observed and why. User-controlled observation toggles. Clear explanation of data flow. |
| **MVP scope too broad** — trying to build too much in MVP, delaying delivery | Medium | High | Medium | Strict scope boundary (this document). Any scope creep requires product lead approval. Weekly scope review. |
| **AI provider dependency** — OpenAI API changes or pricing changes disrupt the product | Low | Medium | Medium | AI-provider abstraction allows switching. OpenAI-compatible API support for self-hosted alternatives. Multiple provider options from MVP. |

### Operational Risks

| Risk | Likelihood | Impact | Severity | Mitigation |
|------|-----------|--------|----------|------------|
| **Team velocity lower than expected** — Rust/Tauri development slower than anticipated | Medium | Medium | Medium | Buffer weeks built into timeline. Parallel work streams where possible. Early spike for complex integration areas. |
| **Auto-update breaks** — MSI or DMG installation fails for some users | Medium | High | High | Extensive testing across OS versions. Rollback mechanism. Manual installer as fallback. |
| **Knowledge base curation takes too long** — users struggle to find and upload relevant documents | Medium | Medium | Medium | Provide sample knowledge packs. Knowledge ingestion wizard with guidance. Suggest knowledge sources. |
| **User education gap** — engineers don't understand how to use the copilot effectively | Medium | Medium | Medium | Onboarding wizard with guided tour. In-app tips and tooltips. Example workflows. User documentation. |

---

## MVP Timeline Estimate

### Timeline Overview

```
Week:  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16
       |----------------------------------------------|

Phase 0: ████████████████████ (12 weeks, completed before MVP)
Phase 1: ████████████████████ (16 weeks, this is the MVP)

MVP Work Breakdown:
  Desktop Shell:    ████████████████████ (weeks 1-8)
  AI Chat:           ████████████████████ (weeks 3-12)
  Observation Engine:  ████████████████ (weeks 2-10)
  Intent Recognition:    ██████████ (weeks 6-10)
  MCP Skills:          ████████████████████ (weeks 4-14)
  Knowledge System:     ████████████████ (weeks 2-10)
  Engineering Workflows:   ██████████ (weeks 8-12)
  Workspace:             ████████████████ (weeks 3-10)
  Security:              ████████████████████ (weeks 1-12)
  AI Provider:             ██████████████ (weeks 2-8)
  Update Mechanism:        ██████████ (weeks 10-14)
  Integration/QA:          ████████████████ (weeks 12-16)
  Internal Testing:                          ████████ (weeks 13-14)
  Alpha Testing:                           ████████ (weeks 14-15)
  Beta Testing:                          ██████████ (weeks 15-16)
```

### Detailed Week-by-Week Breakdown

#### Weeks 1-4: Foundation

- Complete desktop shell scaffold (Tauri v2 + React + Rust core)
- Implement AI provider layer (OpenAI + OpenAI-compatible)
- Set up knowledge system infrastructure (Chroma + embedding pipeline)
- Begin Linux skill development (knowledge base + troubleshooting workflows)
- Implement local encryption and credential manager integration
- Set up CI/CD pipeline (GitHub Actions)

#### Weeks 5-8: Core Features

- Complete AI chat interface (streaming, citations, command suggestions)
- Complete observation engine (screen capture, OCR, app detection, terminal tracking)
- Implement intent recognition (rule-based engine)
- Complete OpenShift and VMware skill development
- Implement workspace system (create, switch, notes, history)
- Complete security features (encryption, credential managers, audit log)
- Implement basic update mechanism

#### Weeks 9-12: Integration

- Complete engineering workflow engine
- Implement privacy controls and observation toggles
- Integrate all components (chat ↔ observation ↔ skills ↔ knowledge ↔ workspace)
- Complete knowledge UI (browse, search, ingest, edit)
- Implement error handling and graceful degradation
- Begin internal testing

#### Weeks 13-14: Testing & Polish

- Internal testing by development team
- Fix bugs and usability issues
- Performance optimization
- Documentation and onboarding wizard
- Alpha testing with 5 internal engineers

#### Weeks 15-16: Beta & Evaluation

- Beta testing with 10 external engineers
- Collect feedback, measure success metrics
- Post-MVP review and Phase 2 planning
- Prepare for public beta release

### Total MVP Duration: **16 weeks (4 months)**

### Team Size: **6-8 engineers**

- 2 Senior Rust/TS engineers (core + Tauri)
- 2 React frontend engineers
- 2 AI/MCP engineers (skills, knowledge, intent)
- 1 QA engineer
- 1 DevOps (CI/CD, deployment, infrastructure)

---

*See also: [ROADMAP.md](./ROADMAP.md) · [BACKLOG.md](./BACKLOG.md) · [RELEASE_PLAN.md](./RELEASE_PLAN.md)*