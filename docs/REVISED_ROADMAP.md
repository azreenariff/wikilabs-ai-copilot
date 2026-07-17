---
description: "Revised implementation roadmap for Wiki Labs AI Copilot — phased delivery plan with validated milestones, dependency ordering, and testable outcomes."
icon: road
---

# Wiki Labs AI Copilot — Revised Implementation Roadmap

## Roadmap Philosophy

The revised roadmap is based on three principles:

1. **Deliver value early, iterate often**: Phase 1 delivers a working chat-based AI copilot in 3-4 months, not a full-featured product in 12+ months.
2. **Validate before building more**: User research and dogfooding phases validate assumptions before committing to complex features.
3. **Parallel workstreams**: Independent teams can work on Phase 1 (chat MVP), Phase 2 (observation engine), and Phase 3 (advanced features) in parallel.

---

## Timeline Overview

```
Q1           Q2           Q3           Q4           Q1
├────────────┼────────────┼────────────┼────────────┼────────────►
│  Phase 0   │  Phase 1   │  Dogfood   │  Phase 2   │  Phase 3   │
│  Foundation│  MVP       │  + Fix     │  Enhanced  │  Platform  │
│  (8 weeks) │  (12 weeks)│  (4 weeks) │  (12 weeks)│  (12 weeks)│
└────────────┴────────────┴────────────┴────────────┴────────────┘
```

---

## Phase 0: Foundation (Weeks 1-8)

### Goal
Establish the technical foundation, validate key decisions, and set up the development infrastructure.

### Deliverables

| Deliverable | Description | Acceptance Criteria |
|-------------|-------------|-------------------|
| Repository setup | Monorepo with Rust workspace, pnpm frontend, GitHub Actions CI | `cargo build`, `pnpm build`, CI passes on all platforms |
| Tauri v2 shell | Empty Tauri app with WebView, Rust core integration | App launches on Windows, macOS, Linux |
| Rust core harness | Event bus, RPC server, SQLite persistence | JSON-RPC call returns response, SQLite writes persist |
| Frontend skeleton | Chat UI, settings panel, workspace selector | Chat UI renders, sends RPC messages |
| CI/CD pipeline | Build, test, lint, coverage on all platforms | PR checks pass, installers produced |
| User research | 5-10 enterprise engineer interviews | Research report with validated assumptions |
| Local embedding model | all-MiniLM-L6-v2 via ONNX Runtime | Embedding generation works on CPU, < 100ms per query |

### Key Decisions to Make
- [ ] Confirm consolidated skill runtime architecture (ADR-002)
- [ ] Confirm SQLite VSS as vector database (ADR-003)
- [ ] Confirm expanded AiProvider trait (ADR-004)
- [ ] Confirm key derivation via OS keychain (ADR-005)
- [ ] Confirm Linux support strategy (ADR-006)
- [ ] Confirm Phase 1 scope (ADR-008)

### Dependencies
- **External**: ONNX Runtime Rust crate, rusqlite with VSS extension, Tauri v2 stable release
- **Internal**: Team ramp-up on Rust, Tauri, and React patterns

### Risks
| Risk | Mitigation |
|------|-----------|
| ONNX Runtime Rust bindings are immature | Fall back to llama.cpp for embedding inference |
| SQLite VSS Rust extension not available | Use pure Rust vector implementation or LanceDB |
| Tauri v2 has breaking changes during development | Pin Tauri v2 version, monitor changelog |

---

## Phase 1: MVP — Chat Copilot (Weeks 9-20)

### Goal
Deliver a working chat-based AI copilot with 3 bundled skills, knowledge base, and workspace management. No observation engine. No skill distribution.

### Architecture (Simplified)

```
User → Chat UI → RPC → Rust Core → AI Provider → Response
                   │                  │
                   │                  ▼
                   │           Prompt Injection Defense
                   │                  │
                   ▼                  ▼
            Skill Runtime     Knowledge System
            (3 modules)        (SQLite VSS + FTS5)
```

### Deliverables

| Deliverable | Description | Acceptance Criteria |
|-------------|-------------|-------------------|
| **AI Chat** | Full chat interface with streaming responses | User types question → AI responds with streaming text |
| **Workspace Management** | Create, switch, configure workspaces | User creates "Acme Corp" workspace, adds "OpenShift" to stack |
| **Knowledge Import** | Import markdown, PDF, text files | Import 10 docs → search returns relevant results |
| **Knowledge Search** | Hybrid search (VSS + FTS5) with citations | AI response includes inline citations to knowledge docs |
| **Linux Skill Module** | Disk, memory, process, network tools | "check disk usage" → returns disk stats |
| **OpenShift Skill Module** | Pod, deployment, cluster tools | "list pods in default" → returns pod list |
| **MySQL Skill Module** | Query, performance, backup tools | "show databases" → returns database list |
| **AI Provider** | OpenAI integration | Chat and tool calls work with OpenAI API |
| **Context Window Manager** | Token budget allocation, sliding window | 100+ message conversation doesn't exceed context window |
| **Prompt Injection Defense** | Input normalization, context separation, output validation | Known injection patterns are blocked |
| **OS Keychain** | Windows Credential Manager, macOS Keychain, Linux Secret Service | API keys stored and retrieved from keychain |
| **Settings** | AI provider config, appearance, skill toggles | User can change settings, settings persist across restarts |
| **Installers** | MSI (Windows), DMG (macOS), AppImage (Linux) | Clean install and uninstall on all platforms |

### Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold startup | < 5 seconds | Time from double-click to usable UI |
| Idle RAM | < 150 MB | OS resource monitor |
| AI response latency | < 3 seconds first token | Time from Enter to first streaming character |
| Knowledge search | < 500 ms | Time from query to results |
| Knowledge import (10 MB PDF) | < 30 seconds | Time from file selection to indexed |
| Workspace switch | < 1 second | Time from click to workspace loaded |

### Testing Requirements

| Test Type | Coverage Target | Tool |
|-----------|----------------|------|
| Unit tests (Rust) | 80%+ | cargo test + cargo-llvm-cov |
| Unit tests (React) | 80%+ | Vitest + React Testing Library |
| Integration tests | All MCP tool handlers | Rust integration tests with mock |
| E2E tests | Core user workflows | Playwright |
| Security audit | All critical findings resolved | Third-party security review |
| Performance benchmarks | All targets met | Custom benchmark suite |

### Dependencies
- Phase 0 completion (foundation, CI/CD, user research)
- OpenAI API access (or enterprise LLM endpoint)
- Third-party security audit scheduling

---

## Dogfooding Phase (Weeks 21-24)

### Goal
Wiki Labs engineering team uses Phase 1 MVP in their daily work. Collect real-world feedback, fix bugs, and validate the product direction before investing in Phase 2.

### Activities

| Activity | Duration | Success Criteria |
|----------|----------|-----------------|
| **Team dogfooding**: All engineers use the copilot for 4 weeks | 4 weeks | 80%+ of engineers use it daily |
| **Bug triage**: All reported bugs triaged and prioritized | Ongoing | P0 bugs fixed within 24 hours |
| **UX feedback**: Weekly feedback sessions | 4 sessions | Top 5 UX issues identified and addressed |
| **Feature requests**: Collect and prioritize Phase 2 features | Ongoing | Prioritized feature backlog |
| **Performance profiling**: Identify bottlenecks | Week 2-3 | Performance improvements for Phase 2 |
| **Security review**: Internal security audit | Week 4 | All medium+ findings addressed |

### Exit Criteria

Before Phase 2 begins:
- [ ] All P0 and P1 bugs resolved
- [ ] Phase 1 runs stably for 7 consecutive days with no crashes
- [ ] Performance targets are met (or documented as known limitations)
- [ ] User research report with Phase 2 recommendations
- [ ] Prioritized feature backlog for Phase 2

---

## Phase 2: Enhanced — Observation + Skills (Weeks 25-36)

### Goal
Add the observation engine (terminal + app context), 3 additional skills, and basic intent recognition. No screenshot OCR in this phase.

### Architecture Addition

```
User → Chat UI → RPC → Rust Core → AI Provider → Response
              ↑            │              │
              │            ▼              ▼
              │     Observation Engine  Knowledge System
              │     (Tier 1 + Tier 2)   (SQLite VSS + FTS5)
              │            │
              │            ▼
              │     Intent Recognition (basic)
              │            │
              └────────────┘
              (context-aware suggestions)
```

### Deliverables

| Deliverable | Description | Priority |
|-------------|-------------|----------|
| **Shell Integration** | bash, zsh, PowerShell command capture | P0 |
| **App Monitor** | Active window detection, title, browser URL | P0 |
| **Tier 1 Observation** | Instant command capture → immediate context analysis | P0 |
| **Tier 2 Observation** | App context → intent recognition within 1-2 seconds | P1 |
| **Basic Intent Recognition** | Rule-based pattern matching with "Unknown" state | P0 |
| **VMware Skill Module** | vSphere, ESXi, VM tools | P0 |
| **Ansible Skill Module** | Playbook, inventory, module tools | P0 |
| **Windows Skill Module** | Event log, service, process tools | P0 |
| **Local AI Provider** | Ollama integration (text-only, no vision) | P1 |
| **Adaptive Observation** | 1s active interval, 10s idle interval, 25% CPU cap | P1 |
| **Credential Detection** | Password, API key, token pattern filtering | P0 |
| **Audit Log Integrity** | Hash chain or Ed25519-signed audit entries | P1 |
| **Workspace Export/Import** | JSON export of workspace configuration | P2 |

### Performance Targets (Phase 2 Addition)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Observation CPU usage | < 10% average | OS resource monitor |
| Command capture latency | < 10 ms | From command enter to engine receipt |
| Intent recognition | < 500 ms | From observation to intent |
| Idle RAM (with observation) | < 250 MB | OS resource monitor |
| All skills loaded | < 100 MB additional | OS resource monitor |

### Testing Requirements (Phase 2 Addition)

| Test Type | Coverage Target | Tool |
|-----------|----------------|------|
| Shell integration tests | All 3 shells, all platforms | Custom test harness |
| Observation engine tests | All tiers, error conditions | Rust integration tests |
| Intent recognition tests | Pattern match accuracy, confidence calibration | Custom test suite |
| Credential detection tests | False positive rate < 5% | Curated test dataset |

### Parallel Workstream (Phase 3 Prep)

During Phase 2, a separate team can begin work on:
- Screenshot OCR pipeline (Tier 3 observation)
- Vision model integration (Phase 3)
- Remaining skill modules (Nagios, Checkmk, PostgreSQL, MS SQL, EDB, RHV)
- WebAssembly skill runtime (for sandboxed third-party skills)

---

## Phase 3: Platform — Vision + Distribution (Weeks 37-48)

### Goal
Add screenshot OCR, full vision-based context, the remaining 6 skills, and the skill distribution platform. Deliver the complete product vision.

### Architecture Addition

```
User → Chat UI → RPC → Rust Core → AI Provider → Response
              ↑            │              │
              │            ▼              ▼
              │     Observation Engine  Knowledge System
              │     (Tier 1 + 2 + 3)    (SQLite VSS + FTS5)
              │            │
              │            ▼
              │     Intent Recognition (ML-enhanced)
              │            │
              │            ▼
              │     Skill Store + Distribution
```

### Deliverables

| Deliverable | Description | Priority |
|-------------|-------------|----------|
| **Screenshot OCR** | Tesseract-based OCR with configurable resolution | P0 |
| **Tier 3 Observation** | Full screen analysis at 5-10 second intervals | P0 |
| **Vision Model Integration** | Screenshot analysis via vision-capable AI provider | P0 |
| **Nagios XI Skill Module** | Alert, host, service monitoring tools | P0 |
| **Checkmk Skill Module** | Site, host, service monitoring tools | P0 |
| **EDB PostgreSQL Skill Module** | Query, performance, replication tools | P0 |
| **MS SQL Server Skill Module** | Query, performance, backup tools | P0 |
| **Red Hat Virtualization Skill Module** | VM, cluster, storage tools | P0 |
| **Skill Distribution** | Package format, signing, update server | P0 |
| **Skill Store UI** | Browse, install, update skills | P1 |
| **WebAssembly Runtime** | Wasmtime-based sandbox for third-party skills | P1 |
| **Cross-Skill Context Bus** | Skills publish context events for collaboration | P1 |
| **Enterprise Compliance** | SOC 2 / GDPR / HIPAA documentation | P1 |
| **ML-Enhanced Intent** | ML-based intent recognition with feedback loop | P2 |

### Performance Targets (Phase 3 Addition)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Screenshot OCR latency | < 1 second | From capture to text output |
| Vision model inference | < 5 seconds | From screenshot to analysis |
| All skills loaded | < 200 MB total | OS resource monitor |
| Skill install | < 10 seconds | From download to ready |
| Cold startup (with all features) | < 10 seconds | From double-click to usable UI |

---

## Post-Phase 3: Future Features

### Candidate Features (Not Prioritized)

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Multi-user** | Share workspaces, knowledge, and skills across team | Enterprise requirement, but adds significant complexity |
| **Cloud sync** | Optional encrypted sync of workspace configurations | Post-enterprise adoption feature |
| **Custom skill creation** | SDK and editor for creating custom skills | Requires skill distribution to be mature first |
| **Mobile companion** | iOS/Android app for alerts and notifications | Requires cloud infrastructure |
| **Team analytics** | Dashboard showing common issues, resolution patterns | Enterprise add-on feature |
| **SSO / SAML integration** | Enterprise authentication | Customer request-driven |
| **On-prem server** | Shared AI inference server for team deployments | Enterprise request-driven |
| **Custom branding** | White-label for enterprise customers | Sales-driven |

---

## Resource Allocation

### Phase 1 Team (Recommended)

| Role | Count | Focus |
|------|-------|-------|
| Rust Engineer | 2 | Core engine, RPC, persistence, skill runtime |
| Frontend Engineer | 1 | React UI, Tauri integration, streaming |
| AI/ML Engineer | 1 | AI provider abstraction, context window, prompt defense |
| Tech Lead | 1 | Architecture, reviews, integration |
| Product Manager | 1 | Requirements, user research, prioritization |
| QA Engineer | 1 | Testing, CI/CD, performance benchmarking |

### Phase 2 Addition

| Role | Count | Focus |
|------|-------|-------|
| Rust Engineer (Observation) | 1 | Shell integration, app monitor, tiered pipeline |
| AI/ML Engineer (Intent) | 1 | Pattern engine, confidence calibration |
| Skill Engineer | 1 | VMware, Ansible, Windows skill modules |

### Phase 3 Addition

| Role | Count | Focus |
|------|-------|-------|
| Rust Engineer (OCR) | 1 | Screenshot capture, OCR pipeline |
| Skill Engineer (x2) | 2 | Remaining 6 skill modules |
| DevOps Engineer | 1 | Skill distribution, update server, CI/CD expansion |

---

## Dependency Graph

```
Phase 0 ──────────► Phase 1 ──────────► Dogfood ──────────► Phase 2 ──────────► Phase 3
  │                      │                    │                    │                    │
  │  ┌───────────────────┤                    │                    │                    │
  │  │                   │                    │                    │                    │
  │  ▼                   ▼                    ▼                    ▼                    ▼
  │  Repo Setup     AI Chat + Workspace   Bug Fixes           Shell Integration    Screenshot OCR
  │  CI/CD          Knowledge Import      UX Feedback         App Monitor          Vision Model
  │  Tauri Shell    3 Skills              Performance         3 Skills (VMware,    6 Skills
  │  User Research  Security              Feature Backlog     Ansible, Windows)    Skill Distribution
  │  Embedding      Prompt Injection      7-Day Stability     Intent Recognition   Wasm Runtime
  │                                    Exit Criteria         Credential Filtering  Context Bus
```

---

## Risk-Adjusted Timeline

| Phase | Optimistic | Realistic | Pessimistic |
|-------|-----------|-----------|-------------|
| Phase 0 | 6 weeks | 8 weeks | 10 weeks |
| Phase 1 | 10 weeks | 12 weeks | 16 weeks |
| Dogfood | 3 weeks | 4 weeks | 6 weeks |
| Phase 2 | 10 weeks | 12 weeks | 16 weeks |
| Phase 3 | 10 weeks | 12 weeks | 16 weeks |
| **Total** | **39 weeks** | **48 weeks** | **64 weeks** |

### Key Risk Factors

| Factor | Impact | Mitigation |
|--------|--------|-----------|
| Rust team ramp-up time | +2-4 weeks Phase 0 | Pair programming, Rust mentorship |
| SQLite VSS integration issues | +2 weeks Phase 1 | Fall back to LanceDB |
| ONNX Runtime Rust bindings | +1 week Phase 0 | Fall back to llama.cpp |
| Shell integration platform issues | +2 weeks Phase 2 | Accept OCR fallback for problematic platforms |
| MCP protocol breaking changes | +2 weeks Phase 1 | Abstract MCP behind trait (ADR-002) |
| User research reveals major pivot | +4 weeks Phase 0 | Start Phase 1 without observation (ADR-008) |

---

## Success Metrics

### Phase 1 Success
- 🟢 **Green**: MVP delivered on time, performance targets met, positive user feedback
- 🟡 **Yellow**: MVP delivered with scope reduction, some targets missed, mixed feedback
- 🔴 **Red**: MVP not delivered, critical bugs, negative user feedback → Reassess Phase 2 investment

### Phase 2 Success
- 🟢 **Green**: Observation engine works reliably, intent recognition is useful, 6 skills working
- 🟡 **Yellow**: Shell integration works on 2/3 platforms, intent recognition needs improvement
- 🔴 **Red**: Observation engine is not reliable, user adoption drops → Focus on Phase 1 stabilization

### Phase 3 Success
- 🟢 **Green**: Full product vision delivered, skill store is active, enterprise compliance met
- 🟡 **Yellow**: Screenshot OCR is slower than expected, some skills lack key features
- 🔴 **Red**: Skill distribution is not adopted, vision features are not used → Reassess product direction