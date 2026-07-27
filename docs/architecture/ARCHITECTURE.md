# Architecture — Wiki Labs AI Copilot

> This is the consolidated architecture document combining the original ARCHITECTURE.md with
> all revised decisions from the architecture review. See individual sections for changes.

## Overview

Wiki Labs AI Copilot is a Tauri v2 desktop application with a React frontend and a Rust core engine.
The application is built for Windows 10 and Windows 11 (64-bit).

## Component Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    WIKI LABS AI COPILOT v2                    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │                  REACT FRONTEND LAYER                 │    │
│  │  - Chat interface, streaming responses               │    │
│  │  - Workspace selector, knowledge management           │    │
│  │  - Skill enable/disable, settings                     │    │
│  └──────────────────────┬───────────────────────────────┘    │
│                         │                                      │
│  ┌──────────────────────▼──────────────────────────────┐    │
│  │                  RUST CORE ENGINE                    │    │
│  │  - Event bus, RPC layer, SQLite persistence           │    │
│  │  - AI Provider Abstraction                            │    │
│  │  - MCP Skill Runtime (consolidated)                   │    │
│  │  - Knowledge System (SQLite VSS + FTS5)               │    │
│  │  - Observation Engine (tiered)                        │    │
│  │  - Intent Engine                                      │    │
│  │  - Workspace Manager                                  │    │    │
│  │  - Security Layer (keychain, encryption, audit)       │    │
│  │  - [NEW] Prompt Injection Defense Layer               │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                              │
│  Platforms: Windows 10/11 (64-bit)                    │
│  Installer: MSI, NSIS (Windows)                        │
└──────────────────────────────────────────────────────────────┘
```

## Architecture Principles

1. **Local-first** — All data stays local; cloud is optional for AI inference.
2. **Human-in-the-loop** — AI advises; engineer executes.
3. **Single database** — SQLite VSS + FTS5 for relational + vector data.
4. **Consolidated skills** — Single-process skill runtime (< 50 MB baseline).
5. **Defense-in-Depth** — Multi-layer prompt injection defense.
6. **Progressive Disclosure** — Low-confidence intent → acknowledge uncertainty.
7. **Embedding Independence** — Local embeddings (not tied to AI provider).
8. **Enterprise Security** — Encryption, audit, data classification.
9. **Open Standards** — MCP protocol for skill interoperability.
10. **Modularity** — Skills are independent modules; no cross-module coupling.

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | Tauri v2 |
| Frontend | React 19 + TypeScript 5.8 |
| Core Language | Rust 2021 |
| Database | SQLite + rusqlite (VSS extension) |
| Vector Search | SQLite VSS (384-dim embeddings) |
| Local Embedding | all-MiniLM-L6-v2 (ONNX Runtime) |
| AI Providers | OpenAI, vLLM, Ollama (abstracted) |
| Logging | tracing + tracing-subscriber |
| CI/CD | GitHub Actions |
| Package Format | MSI, NSIS (Windows) |

## Data Storage

All data stored in a single SQLite database on Windows at `%LOCALAPPDATA%\Wikilabs\wikilabs.db`:

- Workspaces and configuration
- Chat history (per workspace)
- Knowledge documents and chunks (VSS indexed)
- Audit log entries (hash-chain signed)
- Credential hashes (referenced from OS keychain)

## Security Model

- **Key Derivation**: Random 256-bit master key in OS keychain
- **Data Encryption**: AES-256-GCM for confidential/restricted data
- **Credential Storage**: Windows Credential Manager (default), fallback to encrypted SQLite
- **Prompt Injection**: Multi-layer defense (normalize, separate, validate)
- **Data Classification**: Public, Internal, Confidential, Restricted types

## Platform Support

Windows 10/11 (64-bit) is the only supported platform.

| Feature | Windows 10/11 (64-bit) |
|---------|----------------------|
| Desktop App | ✅ |
| Installer | MSI, NSIS |
| WebView | Edge WebView2 |
| Credential Storage | Windows Credential Manager |

See [ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md) for all ADRs.