---
description: "Technology evaluation and selection for Wiki Labs AI Copilot — desktop frameworks, frontend, AI abstraction, vector DB, storage, observability, testing, and CI/CD."
icon: lightbulb
---

# Wiki Labs AI Copilot — Technology Selection

## Overview

This document documents the technology evaluation and selection for Wiki Labs AI Copilot. Each section presents the requirements, evaluated options, the selected technology, and the rationale. The selections are influenced by the patterns established by the OpenHuman project while being adapted for the enterprise engineering copilot domain.

---

## Desktop Framework

### Requirements

| Requirement | Priority | Notes |
|------------|----------|-------|
| Cross-platform (Windows, macOS) | Must | Engineer laptops on both platforms |
| Small binary footprint | Must | Engineers run resource-intensive tools alongside |
| Native OS integration | Must | Credential Manager (Windows), Keychain (macOS) |
| Low resource usage | Must | No Chrome/Chromium overhead |
| No GC pauses | Must | Real-time observation must not be blocked by GC |
| Native UI feel | Nice-to-have | Should feel like a native app, not a web app |
| Mature ecosystem | Nice-to-have | Well-documented, active community, good tooling |
| Linux support | Out of scope | Desktop app only for Windows and macOS |

### Evaluated Options

| Framework | Pros | Cons | Verdict |
|-----------|------|------|---------|
| **Tauri v2** | Small binary (~5 MB vs ~150 MB), Rust memory safety, zero GC pauses, native OS integration, WebView uses OS system WebView, in-process core | Newer ecosystem, fewer React wrappers, requires Rust knowledge, Linux WebView fragmentation | ✅ **SELECTED** |
| Electron | Largest ecosystem, React wrappers, mature tooling, predictable UI across platforms | 150+ MB memory usage, Chrome overhead, GC pauses, large binary, security concerns with Chromium | ❌ Rejected — too heavy for engineer laptops |
| Neutralino | Small footprint, lightweight, supports multiple backends | Smaller ecosystem, less mature, limited native OS integration | ❌ Rejected — ecosystem too small |
| Flutter Desktop | Single codebase, good performance, growing ecosystem | Dart language (unfamiliar team), WebView replacement needed, smaller Rust integration ecosystem | ❌ Rejected — language and ecosystem mismatch |
| .NET MAUI | Good Windows integration, C# ecosystem | Poor macOS support, Windows-centric, not suited for cross-platform desktop | ❌ Rejected — macOS support inadequate |
| Native (Win32 / AppKit) | Full control, native performance | Double the work (Windows + macOS), no shared codebase, much slower development | ❌ Rejected — too much platform-specific code |

### Selected Technology: Tauri v2

**Rationale**:
- Tauri v2 uses the OS system WebView (EdgeWebView on Windows, WKWebView on macOS) — no bundled Chromium
- Rust core runs in-process as a tokio task (same pattern as OpenHuman) — zero inter-process overhead for core communication
- The `keyring` crate provides seamless OS credential manager integration
- Binary size is typically 5-20 MB (vs Electron's 150+ MB)
- Memory usage is < 50 MB at idle (vs Electron's 150+ MB)
- Rust provides memory safety and zero GC pauses
- Active development, Tauri v2 is production-ready

**How OpenHuman Informs This Choice**:
OpenHuman already uses Tauri v2 successfully on Windows, macOS, and Linux. The patterns for in-process core, JSON-RPC, and Tauri IPC are already proven. Wiki Labs AI Copilot adopts the same framework with adapted components.

---

## Frontend Framework

### Requirements

| Requirement | Priority | Notes |
|------------|----------|-------|
| Component model | Must | Reusable, composable components |
| Type safety | Must | Large codebase, multiple contributors |
| State management | Must | Complex UI state across panels |
| Performance | Must | Fast rendering, smooth animations |
| Ecosystem | Must | Rich component library, good tooling |
| Streaming support | Must | Progressive rendering of AI responses |
| Developer experience | Nice-to-have | Fast HMR, good debugging tools |

### Evaluated Options

| Framework | Pros | Cons | Verdict |
|-----------|------|------|---------|
| **React 19** | Largest ecosystem, TypeScript support, mature state management, streaming via Suspense and concurrent features, Tauri WebView fully compatible | Learning curve for hooks, server-side rendering not needed (SPA only) | ✅ **SELECTED** |
| Vue 3 | Easier learning curve, composables, good performance | Smaller ecosystem, fewer component libraries, smaller AI-related tooling | ❌ Rejected — ecosystem too small for AI copilot |
| Svelte 5 | Best runtime performance, simplest syntax | Smaller ecosystem, no mature state management library, smaller community | ❌ Rejected — ecosystem not mature enough |
| SolidJS | Excellent performance, fine-grained reactivity | Smallest ecosystem, fewer libraries, smaller community | ❌ Rejected — ecosystem too small |

### Selected Technology: React 19 + TypeScript 5.8

**Rationale**:
- React 19 has the largest ecosystem and most mature component libraries (Radix UI, Headless UI, etc.)
- TypeScript 5.8 provides full type safety for the complex data models and AI interfaces
- Redux Toolkit for state management (proven pattern from OpenHuman)
- Vite 7 for build tooling — sub-second HMR, optimized production builds
- Tailwind CSS for styling — utility-first, consistent design system
- Streaming support via React 19's concurrent features and Suspense

**State Management**: Redux Toolkit + Redux Persist (same pattern as OpenHuman). Critical UI state (active workspace, settings, enabled skills) persists across restarts.

---

## AI Provider Abstraction

### Requirements

| Requirement | Priority | Notes |
|------------|----------|-------|
| Multiple provider support | Must | OpenAI, vLLM, local models, enterprise APIs |
| Replaceability | Must | Switch providers without changing application code |
| Streaming support | Must | Progressive AI response display |
| Embedding support | Must | Generate text embeddings for knowledge search |
| Consistent interface | Must | Same API regardless of underlying provider |
| Health checking | Nice-to-have | Verify provider availability before use |
| Cost tracking | Nice-to-have | Track token usage per provider |

### Architecture

```rust
pub trait AiProvider: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>, params: ChatParams) -> Result<ChatResponse>;
    async fn chat_stream(&self, messages: Vec<ChatMessage>, params: ChatParams) -> Result<ChatStream>;
    async fn embed(&self, text: String) -> Result<Vec<f32>>;
    fn info(&self) -> ProviderInfo;
    async fn health(&self) -> Result<()>;
}
```

### Implemented Providers

| Provider | Implementation | When to Use |
|----------|---------------|-------------|
| OpenAI | OpenAI SDK or direct API | Default choice, best model quality |
| OpenAI-Compatible | Any compatible endpoint (vLLM, LM Studio, Ollama) | When using a compatible inference server |
| vLLM | Self-hosted high-throughput inference | Enterprise on-prem deployment |
| Ollama | Local model serving | Air-gapped environments, full data privacy |
| Enterprise Custom | Custom provider implementation | On-prem models with custom authentication |

### Rationale

**Why an abstraction layer?**
- Engineers may have different data residency requirements (some need on-prem, some can use cloud)
- Model quality varies — some scenarios benefit from different models
- Cost optimization — use smaller/cheaper models for simple tasks
- Future-proofing — new providers can be added without changing application code

**Why not hardcode to OpenAI?**
- Some enterprises require on-prem models for compliance
- Different models excel at different tasks (coding, reasoning, analysis)
- Cost varies significantly between providers and models

---

## Local Vector Database

### Requirements

| Requirement | Priority | Notes |
|------------|----------|-------|
| Embedded (no server) | Must | Zero-config, local-only |
| Rust bindings | Must | Integrates with Rust core |
| Semantic search | Must | Vector similarity search |
| Persistence | Must | Survives app restarts |
| Lightweight | Must | < 50 MB overhead |
| Metadata filtering | Nice-to-have | Filter by workspace, doc type, etc. |

### Evaluated Options

| Technology | Pros | Cons | Verdict |
|-----------|------|------|---------|
| **ChromaDB** | Embedded mode, Python + Rust bindings, lightweight, active development, good metadata filtering, persistence | Smaller Rust ecosystem than Python | ✅ **SELECTED** |
| Qdrant | Excellent Rust-native implementation, good performance, rich filtering | Requires server (not truly embedded), larger footprint | ❌ Rejected — server requirement conflicts with local-first philosophy |
| Weaviate | Rich features, good filtering, hybrid search | Server-based, heavier, overkill for local use | ❌ Rejected — server requirement |
| SQLite VSS | Built into SQLite, zero additional dependencies | Limited feature set, less mature, smaller community | ❌ Rejected — insufficient functionality |
| FAISS | Excellent performance, Facebook-backed | No built-in persistence, no metadata filtering, C++ library | ❌ Rejected — too low-level, requires custom persistence layer |

### Selected Technology: ChromaDB

**Rationale**:
- ChromaDB offers a true embedded mode — no server process required
- Python ecosystem is rich; Rust bindings exist via `chromadb` crate
- Supports metadata filtering (essential for workspace-scoped search)
- Lightweight — the embedded database is a single file
- Active development and good documentation
- Hybrid search (vector + keyword) can be implemented by combining ChromaDB vector search with SQLite FTS5

### Hybrid Search Strategy

```
Query: "OpenShift pod CrashLoopBackOff"
    │
    ├──► ChromaDB (Vector Search, 70% weight)
    │      └──► Semantic similarity: "pod restart loop", "container crash"
    │
    └──► SQLite FTS5 (Keyword Search, 30% weight)
           └──► Exact match: "CrashLoopBackOff", "pod"
    │
    └──► Weighted Merge ──► Ranked Results
```

---

## Knowledge Storage Technology

### Requirements

| Requirement | Priority | Notes |
|------------|----------|-------|
| Embedded database | Must | No external server dependency |
| ACID transactions | Must | Data integrity |
| Full-text search | Must | FTS5 for keyword search |
| Single file | Nice-to-have | Portable, easy backup |
| Rust bindings | Must | Integrated with Rust core |

### Selected Technology: SQLite (via rusqlite)

**Rationale**:
- SQLite is the industry standard for embedded databases
- `rusqlite` crate provides safe Rust bindings (used by OpenHuman)
- FTS5 extension provides full-text search without external dependencies
- Single-file database — portable, easy backup, easy migration
- ACID transactions ensure data integrity
- Zero configuration — no server to install or manage
- Well-tested, battle-hardened, used by millions of applications

**Why not PostgreSQL/MySQL?**
- Server-based — requires installation and configuration
- Overkill for local-only data storage
- Adds deployment complexity for a desktop app

**Why not a NoSQL embedded DB (SQLite alternative)?**
- SQLite has FTS5 built-in, which is essential for knowledge search
- SQLite ecosystem in Rust is mature (`rusqlite`, `sqlx`)
- SQLite's relational model fits the structured data (workspaces, sessions, knowledge docs)

---

## Logging and Observability

### Requirements

| Requirement | Priority | Notes |
|------------|----------|-------|
| Structured logging | Must | Machine-readable for log analysis |
| Log levels | Must | Debug, Info, Warn, Error |
| Audit logging | Must | Immutable event log |
| Console output | Must | Development debugging |
| File output | Nice-to-have | Persistent logs for production |

### Selected Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Logging** | `tracing` + `tracing-subscriber` | Rust standard, structured, performant, filters by target/level |
| **Audit Logger** | Custom `rusqlite` + append-only file | Immutable, queryable, exportable |
| **Console Output** | `tracing-subscriber` with `fmt` subscriber | Development-friendly colored output |
| **File Output** | `tracing-subscriber` with `JsonWriter` subscriber | Production log files, rotated |
| **Observability** | Prometheus metrics (optional) | For enterprise deployments with monitoring |

### Log Rotation

| Log Type | Rotation Policy | Retention |
|----------|----------------|-----------|
| Application logs | Daily rotation, gzip compression | 30 days |
| Audit logs | Append-only, no rotation | 90 days minimum (configurable) |
| System logs | Daily rotation, gzip compression | 30 days |

---

## Testing Framework Selection

### Frontend Testing

| Tool | Purpose | Selected |
|------|---------|----------|
| **Vitest** | Unit tests, integration tests | ✅ Yes — same build tool (Vite), fast, Jest-compatible |
| **React Testing Library** | Component tests | ✅ Yes — best practices for testing user behavior |
| **MSW (Mock Service Worker)** | API mocking | ✅ Yes — mock Tauri IPC and RPC calls |
| **Playwright** | E2E testing | ✅ Yes — cross-platform, reliable, good Tauri support |

### Rust Testing

| Tool | Purpose | Selected |
|------|---------|----------|
| **cargo test** (libtest) | Unit tests, integration tests | ✅ Yes — Rust standard, built-in |
| **cargo-llvm-cov** | Code coverage | ✅ Yes — LLVM-based, accurate, integrates with CI |
| **tokio-test** | Async testing | ✅ Yes — tokio-specific test utilities |

### How OpenHuman Informs This Selection

OpenHuman uses:
- **Vitest** for frontend tests — proven effective
- **libtest** (`cargo test`) for Rust tests — proven effective
- **Playwright** for E2E tests — proven effective
- **cargo-llvm-cov** for coverage — proven effective
- Mock backend for Rust integration tests — adapted for Wiki Labs

---

## CI/CD Toolchain

### Requirements

| Requirement | Priority | Notes |
|------------|----------|-------|
| GitHub integration | Must | PR checks, merge gates |
| Multi-platform builds | Must | Windows, macOS |
| Parallel execution | Must | Speed up CI |
| Caching | Must | Fast builds, fast tests |
| Artifact management | Must | Installers, packages |
| Code coverage gate | Must | ≥ 80% on changed lines |

### Selected Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **CI/CD** | GitHub Actions | Built into GitHub, rich marketplace, good caching |
| **Package** | cargo-tauri | Tauri v2's official build tool, produces installers |
| **Code Signing** | Windows: Microsoft SGC, macOS: Apple Developer ID | Industry standard for desktop apps |
| **Update Server** | GitHub Releases + S3 (or similar) | Proven distribution pattern |
| **Package Signing** | Ed25519 for skills, standard code signing for installers | Security-focused |

### CI Model (Adapted from OpenHuman's Two-Lane CI)

| Lane | Trigger | Scope | Speed |
|------|---------|-------|-------|
| **CI Lite** | Pushes to main, PRs to main/release | Quality checks + unit tests for changed files only | Fast (~5 min) |
| **CI Full** | PRs to release, weekly scheduled | Complete test suite + E2E on both platforms | Slow (~30 min) |

**CI Lite**:
- Frontend: ESLint, Prettier, Vitest for changed files only (`vitest related`)
- Rust: `cargo check`, unit tests for changed crates (`cargo llvm-cov` with file filter)
- Coverage gate: ≥ 80% on changed lines via `diff-cover`

**CI Full**:
- Frontend: Full Vitest suite
- Rust: Full cargo test suite + integration tests
- E2E: Playwright tests on Windows and macOS
- Build: Production build and installer generation

---

## Build Toolchain

### Frontend Build

| Tool | Version | Purpose |
|------|---------|---------|
| **Vite** | 7 | Build tool, HMR, bundling |
| **TypeScript** | 5.8 | Type checking and compilation |
| **pnpm** | Latest | Package manager (fast, disk-efficient) |
| **ESLint** | Latest | Static analysis |
| **Prettier** | Latest | Code formatting |
| **Husky** | Latest | Git hooks (pre-commit, pre-push) |

### Rust Build

| Tool | Version | Purpose |
|------|---------|---------|
| **cargo** | Latest | Build, test, run |
| **cargo-tauri** | Latest | Tauri build and release |
| **rustfmt** | Latest | Code formatting |
| **clippy** | Latest | Static analysis |
| **cargo-llvm-cov** | Latest | Code coverage |
| **cargo audit** | Latest | Dependency vulnerability scanning |

---

## Package Manager and Dependency Management

| Layer | Tool | Rationale |
|-------|------|-----------|
| **Rust** | cargo + Cargo.lock | Standard Rust dependency management |
| **Frontend** | pnpm | Fast, disk-efficient, strict, pnpm-workspace.yaml for monorepo |
| **Skills** | Versioned packages (zip) | Custom distribution format for MCP skills |

---

## Technology Selection Summary

| Layer | Selected | Alternative Considered | Rationale for Selection |
|-------|----------|----------------------|------------------------|
| Desktop Framework | Tauri v2 | Electron | Small footprint, no GC pauses, native OS integration |
| Frontend | React 19 + TS 5.8 | Vue 3 | Largest ecosystem, mature patterns |
| State Management | Redux Toolkit | Zustand | Proven pattern from OpenHuman |
| Build | Vite 7 | Webpack | Sub-second HMR, optimized builds |
| Styling | Tailwind CSS | CSS Modules | Utility-first, consistent design |
| Core Language | Rust (2021) | N/A (Tauri requires Rust) | Memory safety, performance |
| Async Runtime | Tokio | N/A (Rust standard) | Industry standard for async Rust |
| Database | SQLite + rusqlite | N/A | Zero-config, embedded, ACID |
| Vector DB | ChromaDB (embedded) | Qdrant | Local-first, embedded, metadata filtering |
| Search | ChromaDB + SQLite FTS5 | FAISS | Hybrid semantic + keyword search |
| AI Abstraction | Custom trait | OpenAI SDK only | Multi-provider support, replaceability |
| Logging | tracing + tracing-subscriber | slog | Rust standard, structured, performant |
| Frontend Tests | Vitest + RTL | Jest | Vite-native, fast, modern |
| E2E Tests | Playwright | Cypress | Cross-platform, reliable |
| Rust Tests | cargo test + cargo-llvm-cov | N/A | Rust standard |
| CI/CD | GitHub Actions | GitLab CI | GitHub-native, rich ecosystem |
| Package Manager | pnpm | npm/yarn | Fast, strict, disk-efficient |
| TLS | rustls + reqwest | OpenSSL | No OpenSSL dependency, pure Rust |
| Credential Mgr | keyring crate | Custom implementation | OS-native, battle-tested |

## References

- [ARCHITECTURE.md](ARCHITECTURE.md) — System architecture
- [REPOSITORY_STRUCTURE.md](REPOSITORY_STRUCTURE.md) — Repository organization
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) — Development workflows
- [TESTING_STRATEGY.md](TESTING_STRATEGY.md) — Testing approach