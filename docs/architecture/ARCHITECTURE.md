# Architecture — Wiki Labs AI Copilot

> This is the consolidated architecture document combining the original ARCHITECTURE.md with
> all revised decisions from the architecture review. See individual sections for changes.

## Overview

Wiki Labs AI Copilot is a Tauri v2 desktop application with a React frontend and a Rust core engine.
The application runs on Windows, macOS, and Linux.

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
│  Platforms: Windows, macOS, Linux                            │
│  Installer: MSI, DMG, AppImage, deb, rpm                     │
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
| Package Format | MSI (Windows), DMG (macOS), AppImage (Linux) |

## Data Storage

All data stored in a single SQLite database at `~/.local/share/wikilabs/wikilabs.db`:

- Workspaces and configuration
- Chat history (per workspace)
- Knowledge documents and chunks (VSS indexed)
- Audit log entries (hash-chain signed)
- Credential hashes (referenced from OS keychain)

## Security Model

- **Key Derivation**: Random 256-bit master key in OS keychain
- **Data Encryption**: AES-256-GCM for confidential/restricted data
- **Credential Storage**: OS keychain (Credential Manager / Keychain / Secret Service)
- **Prompt Injection**: Multi-layer defense (normalize, separate, validate)
- **Data Classification**: Public, Internal, Confidential, Restricted types

## Platform Support

| Feature | Windows | macOS | Linux |
|---------|---------|-------|-------|
| Desktop App | ✅ | ✅ | ✅ |
| Installer | MSI | DMG | AppImage, deb, rpm |
| WebView | Edge WebView2 | WKWebView | WebKitGTK |
| Credential Storage | Credential Manager | Keychain | Secret Service |

See [ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md) for all ADRs.