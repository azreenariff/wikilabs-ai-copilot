# Backlog

This file tracks the high-level epics for the Wiki Labs AI Copilot project. Each epic maps to a phase in the REVISED_ROADMAP.md.

## Epics

### E-01: Workspace Foundation
Bootstrap the Rust workspace, CI/CD pipeline, and initial `data_types` crate. Establish the engineering baseline.
- **Phase:** Phase 3 (Repository Bootstrap)
- **Status:** ✅ Completed
- **Key Deliverables:** Workspace `Cargo.toml`, issue templates, CI workflows, `src/main.rs` entrypoint

### E-02: Core Persistence Layer
Implement SQLite-backed persistence with schema migrations, repository traits, and query builders.
- **Phase:** Phase 4 (Data Layer)
- **Status:** Pending
- **Key Deliverables:** `rusqlite` connection pool, schema migrations, `Repository<T>` trait, FTS5 and VSS indexes

### E-03: AI Runtime
Implement the `AiProvider` trait, context window manager, token counter, and streaming support.
- **Phase:** Phase 4 (Data Layer)
- **Status:** Pending
- **Key Deliverables:** OpenAI/vLLM/Ollama providers, context window trimming, token estimation, streaming responses

### E-04: MCP Skill Runtime
Consolidate multi-process MCP into a single-process skill module loader with tool aggregation.
- **Phase:** Phase 5 (Skill Runtime)
- **Status:** Pending
- **Key Deliverables:** `SkillManager`, `SkillModule` trait, tool registry, context bus, MCP server bridge

### E-05: Knowledge System
Implement vector search (SQLite VSS, 384-dim) + FTS5 keyword search + hybrid scoring (70/30 weighting).
- **Phase:** Phase 5 (Knowledge System)
- **Status:** Pending
- **Key Deliverables:** Document ingestion pipeline, ONNX embedding model (all-MiniLM-L6-v2), deduplication, quality scoring

### E-06: Intent Recognition
Build rule-based pattern matching engine for Phase 1, with extensibility for ML-based classification.
- **Phase:** Phase 5 (Intent & Observation)
- **Status:** Pending
- **Key Deliverables:** Intent enum (Troubleshooting, Configuration, Deployment, Documentation, Learning, Unknown), confidence scoring, human correction loop

### E-07: Observation Engine
Implement tiered observation pipeline: shell integration (instant), app monitoring (fast), screen capture (slow).
- **Phase:** Phase 6 (Observation System)
- **Status:** Pending
- **Key Deliverables:** Tier 1 (clipboard, shell), Tier 2 (app monitor, window detection), Tier 3 (screenshot, OCR)

### E-08: Security Hardening
Implement encryption layer, credential management, audit logging, and injection defense.
- **Phase:** Phase 6 (Security Hardening)
- **Status:** Pending
- **Key Deliverables:** OS keychain integration, HKDF-SHA256 derivation, AES-256-GCM encryption, PII redaction, audit trail

### E-09: Desktop Application (Tauri v2)
Build the cross-platform desktop UI with Tauri v2, including chat interface, workspace management, and skill configuration.
- **Phase:** Phase 7 (Desktop Application)
- **Status:** Pending
- **Key Deliverables:** Tauri v2 app shell, React/TypeScript frontend, chat UI, workspace sidebar, settings panel

### E-10: Release & Distribution
Prepare for initial release: binary packaging, update mechanism, documentation, and user onboarding.
- **Phase:** Phase 8 (Release)
- **Status:** Pending
- **Key Deliverables:** Code signing (macOS), NSIS installer (Windows), AppImage (Linux), update notifications, onboarding guide

## Epic Dependencies

```
E-01 → E-02 → E-03
               │
E-01 → E-04 → E-05 → E-06 → E-07
                          │
E-05 → E-08
                          │
E-09 → E-10
```

## Status Legend

- ✅ Completed
- 🔄 In Progress
- ⏳ Ready
- 📋 Planned
- 🔒 Blocked