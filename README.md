# Wiki Labs AI Copilot

**A context-aware desktop AI assistant for technical engineers.**

Version: `1.1.205` · License: MIT · Platform: Windows (Tauri v2 desktop app)

[Repository](https://github.com/wikilabs/wikilabs-ai-copilot) · [CHANGELOG](CHANGELOG.md)

---

## Overview

Wiki Labs AI Copilot is a **Tauri v2 + Rust workspace + React 18** desktop application that acts as a proactive AI assistant for technical engineers. It observes what you're doing on your screen, understands your intent, and provides context-aware guidance — like telling you "check MySQL status" instead of waiting for you to ask.

The app runs entirely on your machine with a lightweight system tray icon, a sidebar-powered UI, and an embedded Axum HTTP server on localhost. It connects to any OpenAI-compatible LLM endpoint (OpenRouter, local vLLM, Ollama, etc.).

### Key Features

- **🔍 Screen Observation Engine** — Captures and analyzes your screen in real-time, detecting activities like troubleshooting, deployment, monitoring, development, and browsing
- **🧠 Intent Analysis** — Synthesizes observations into structured intent summaries (what you're doing, likely goal, detected issues)
- **💬 AI Chat Assistant** — Full conversation interface with persistent history (SQLite), system prompts from an engineering persona, and observation context injected into every request
- **⚡ Proactive Guidance** — The guidance engine tracks workflows, produces recommendations with evidence, and pushes toast notifications when actionable advice is available
- **📚 Knowledge Packs** — Versioned, validated knowledge bundles (engineering foundations for Linux, Windows, networking, security, etc.) compiled into the app at build time
- **🛠️ Skills System** — SKILL.md-based skills (Ansible, checkmk, PostgreSQL, MySQL, VMware, OpenShift, Nagios, etc.) with lifecycle management (enable/disable/activate/validate)
- **🔐 Security** — Credential encryption (AES-GCM, ChaCha20-Poly1305), key management, injection defense, and audit logging
- **📊 Performance Benchmarking** — Built-in benchmark registry measuring startup time, AI response latency, knowledge indexing, skill loading, screen capture, OCR processing, and large conversation performance
- **⚙️ First-Run Setup Wizard** — Guides new users through AI provider configuration
- **🖥️ System Tray** — Minimizes to tray (not close), autostart support, global shortcuts
- **🔄 Auto-Updates** — Tauri updater plugin with MSI/NSIS installers

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                     REACT 18 FRONTEND                           │
│  Sidebar ─ Chat │ Guidance │ Skills │ Knowledge │ Activity │ Settings │ About
├──────────────────────────────────────────────────────────────────┤
│                   AXUM HTTP SERVER (:1420)                       │
│  REST API bridge ─ Tauri Commands ↔ HTTP POST /api/commands/*  │
├──────────────────────────────────────────────────────────────────┤
│                   TAURI V2 DESKTOP LAYER                         │
│  App State (SQLite + Settings) │ Observation │ Guidance Loop    │
│  Knowledge Panel │ Skill Management │ Security │ Config          │
├──────────────────────────────────────────────────────────────────┤
│                    RUST WORKSPACE CRATES                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │ ai       │ │ copilot  │ │ guidance │ │ intent   │           │
│  │ knowledge│ │ skill_*  │ │ workflow │ │ security │           │
│  │ └────────┘ └──────────┘ └──────────┘ └──────────┘           │
├──────────────────────────────────────────────────────────────────┤
│                     PERSISTENCE LAYER                            │
│  SQLite (rusqlite) ─ chat messages, workspaces, settings         │
└──────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
Screen Capture ─→ Image Analysis (Vision LLM) ─→ Activity Detection ─→ Intent Analysis
                                                                    │
                                                                    ▼
                                                          Structured Intent Summary
                                                           (current_activity, intent,
                                                            issues, infrastructure)
                                                                    │
                                                                    ▼
                                                    Injected into AI prompt
                                               as observation context ─→ Chat Response
                                                                    │
                                                                    ▼
                                                    Proactive recommendations
                                                    pushed via toast notifications
```

---

## Project Structure

```
wikilabs-ai-copilot/
├── src-tauri/              # Tauri v2 desktop app (Rust)
│   ├── src/
│   │   ├── main.rs          # App entry, shared state, Tauri commands, chat handler
│   │   ├── api_server.rs    # Axum HTTP server on localhost:1420
│   │   ├── observation.rs   # Screen capture + intent analysis engine
│   │   ├── guidance_loop.rs # Proactive guidance orchestration
│   │   ├── guidance_panel.rs      # Guidance workflow/recommendation CRUD
│   │   ├── knowledge_panel.rs     # Knowledge pack management (import/export/reindex)
│   │   ├── skill_management.rs    # Skill lifecycle (enable/disable/validate)
│   │   ├── skill_knowledge.rs     # Skill-knowledge bridge
│   │   ├── config.rs            # App settings + AI provider configuration
│   │   ├── security.rs          # Security panel
│   │   ├── api_ready.rs         # Health check endpoint
│   │   ├── logging.rs           # Structured logging setup
│   │   └── ...
│   ├── Cargo.toml
│   └── tauri.conf.json         # App config, bundle resources (skills/, engineering-foundations/)
├── src/                        # Rust workspace crates
│   ├── core/
│   │   ├── data_types/        # Shared types: ChatMessage, Skill, Knowledge, Intent, Tool, etc.
│   │   └── persistence/       # SQLite schema, repositories, migrations
│   ├── ai/                    # AI provider abstraction (OpenAI-compatible)
│   │                          #   Provider trait, OpenAICompatibleProvider, EngineeringPersona
│   ├── copilot/               # Copilot lifecycle, conversation management, memory, policy
│   ├── guidance/              # Guidance system: workflows, recommendations, evidence, feedback
│   ├── knowledge/             # Knowledge pipeline: discover → clean → parse → chunk → dedup → index
│   ├── observation/           # Observation engine: screen capture, activity categories, intent analysis
│   ├── intent/                # Intent extraction from observations
│   ├── workspace/             # Workspace management
│   ├── security/              # Encryption, keychain, credentials, audit, classification
│   ├── skill_*/               # Skills ecosystem:
│   │   ├── skill_discovery/   # Discover skills on disk
│   │   ├── skill_runtime/     # Execute/run skills
│   │   ├── skill_sdk/         # SDK for writing new skills
│   │   └── skill_activation/  # Activate/deactivate skills
│   ├── mcp/                   # MCP (Model Context Protocol):
│   │   ├── skill_manager/     # Skill management via MCP
│   │   └── registry/          # Skill registry
│   ├── workflow_engine/       # Workflow orchestration
│   ├── engineering_timeline/  # Engineering timeline tracking
│   ├── technology_recognition/# Tech stack recognition
│   ├── recommendation_readiness/# Recommendation readiness scoring
│   ├── context_fusion/        # Context fusion from multiple providers
│   ├── human_feedback/        # Human feedback loop
│   ├── intelligence_engine/   # Intelligence engine coordination
│   ├── benchmark/             # Performance benchmarking
│   ├── testing/               # Test fixtures and mocks
│   ├── frontend/              # React 18 + TypeScript + Vite
│   │   ├── src/
│   │   │   ├── main.tsx       # React root rendering
│   │   │   ├── App.tsx        # Router, setup wizard, main layout
│   │   │   ├── components/
│   │   │   │   ├── Sidebar.tsx
│   │   │   │   ├── GuidanceToast.tsx
│   │   │   │   └── ErrorBoundary.tsx
│   │   │   └── pages/
│   │   │       ├── ChatAssistant.tsx
│   │   │       ├── Guidance.tsx
│   │   │       ├── Skills.tsx
│   │   │       ├── Knowledge.tsx
│   │   │       ├── Activity.tsx
│   │   │       ├── Settings.tsx
│   │   │       ├── SetupWizard.tsx
│   │   │       ├── About.tsx
│   │   │       └── PreflightCheck.tsx
│   ├── skills/                # Pre-bundled skill packs (Ansible, MySQL, PostgreSQL, etc.)
│   └── engineering-foundations/ # Pre-bundled knowledge packs (Linux, Windows, networking, etc.)
├── .github/workflows/         # CI/CD
│   ├── ci.yml                 # Tests + clippy + docs on PR
│   ├── release.yml            # Release on version tags
│   └── dependency-review.yml  # Dependency security review
├── Cargo.toml                 # Workspace definition (31 crates)
├── CHANGELOG.md               # Release history
└── assets/                    # App assets (system prompt, icons, etc.)
```

---

## Rust Workspace Crates

The project is a Cargo workspace with **31 crates** organized into layers:

### Core Layer
| Crate | Purpose |
|-------|---------|
| `wikilabs-data-types` | Shared domain types: `ChatMessage`, `Skill`, `Knowledge`, `Intent`, `Tool`, `Technology`, `Timeline`, `EngineeringContext`, `Workspace`, `AI` |
| `wikilabs-persistence` | SQLite database layer with rusqlite — schema (INIT_SQL), repositories, migrations |

### AI & Copilot Layer
| Crate | Purpose |
|-------|---------|
| `wikilabs-ai` | AI provider abstraction — `Provider` trait, `OpenAICompatibleProvider`, `EngineeringPersona` system prompt, `AiProvider`, `AiRequest` |
| `wikilabs-copilot` | Copilot lifecycle, conversation management, memory, policy, recommendation, decision engine, explainability, proactive behavior |

### Guidance & Intelligence Layer
| Crate | Purpose |
|-------|---------|
| `wikilabs-guidance` | Guidance system — workflows, recommendations, evidence, timeline events, feedback system, command recommendation, safety framework |
| `wikilabs-intent` | Intent extraction from observations |
| `wikilabs-observation` | Screen capture, activity detection, intent analysis — `ActivityCategory` enum (Troubleshooting, Deployment, Monitoring, Development, Browsing, Communication, VisualInsight, VisualError) |
| `wikilabs-intelligence-engine` | Intelligence coordination |
| `wikilabs-context-fusion` | Fuse context from multiple providers |

### Knowledge & Skills Layer
| Crate | Purpose |
|-------|---------|
| `wikilabs-knowledge` | Knowledge pipeline — discover, clean, parse, chunk, dedup, normalize, validate, metadata extraction, version detection, incremental updates, index preparation |
| `wikilabs-skill-*` | Skills ecosystem — discovery, runtime, SDK, activation (4 crates) |
| `wikilabs-mcp` | Model Context Protocol — skill management, registry (2 crates) |

### Engineering Domain Layer
| Crate | Purpose |
|-------|---------|
| `wikilabs-workflow-engine` | Workflow orchestration |
| `wikilabs-engineering-timeline` | Engineering timeline tracking |
| `wikilabs-technology-recognition` | Technology stack recognition |
| `wikilabs-recommendation-readiness` | Recommendation readiness scoring |
| `wikilabs-human-feedback` | Human feedback loop |

### Infrastructure Layer
| Crate | Purpose |
|-------|---------|
| `wikilabs-security` | Encryption (AES-GCM, ChaCha20-Poly1305), keychain, credential management, injection defense, audit logging, classification |
| `wikilabs-benchmark` | Performance benchmarking — startup, AI response, knowledge indexing, skill loading, screen capture, OCR |
| `wikilabs-testing` | Test fixtures (workspace, knowledge) and mocks (OpenAI) |

---

## Desktop App (src-tauri)

### Technology Stack
- **Tauri v2** with plugins: shell, log, global-shortcut, updater, autostart, single-instance, notification, tray-icon
- **Axum 0.7** HTTP server on `localhost:1420` — serves as the bridge between the web frontend and Rust backend
- **SQLite (rusqlite)** for all persistent data
- **tracing** for structured logging

### How It Works

The app has **two communication paths**:

1. **Tauri Commands** (`#[tauri::command]`) — Direct Rust-to-JS IPC for basic operations (settings, providers, status)
2. **HTTP API** (`POST /api/commands/*`) — The frontend calls `fetch()` to the Axum server, which delegates to the Rust backend. This is the **primary communication path** for all UI interactions.

The frontend polls `http://127.0.0.1:1420/ready` to wait for the server to bind before showing the UI.

### Main Entry Point (`src-tauri/src/main.rs`)

The Tauri app initializes:
- `AppState` — shared state wrapping `AppHandle`, SQLite `Database`, `RepositoryFactory`, and `AppSettingsStore`
- Starts the Axum HTTP server on port 1420
- Registers Tauri commands for settings, AI providers, chat, workspaces, performance metrics
- Manages the observation engine and guidance loop

Key exposed commands:
- `get_settings` / `update_settings` — App configuration
- `list_providers` / `test_connection` / `list_models` — AI provider management
- `get_workspace_list` / `create_workspace` — Workspace CRUD
- `send_message` — Chat with AI (builds system prompt + observation context + conversation history, calls the configured LLM)
- `get_status` — App health check
- `get_performance_metrics` — Benchmark registry diagnostics

Panel commands (grouped by feature):
- **Guidance**: `guidance_*` — workflows, recommendations, evidence, feedback
- **Knowledge**: `knowledge_*` — import, export, list, enable/disable, reindex packs
- **Skills**: `skill_*` — list, get, enable/disable, validate, set active

### Observation Engine

The observation engine captures the screen, sends it to the vision/LLM model for analysis, and produces a structured `IntentSummary` containing:
- `current_activity` — List of detected activities with category, confidence, and description
- `intent` — The user's likely intent with category, confidence, goal, and infrastructure targets
- `issues` — Detected problems or errors
- `infrastructure_context` — Running services, system state
- `suggested_guidance` — Actionable recommendations

This summary is injected as a system message into every AI chat request, enabling proactive context-aware responses.

### Guidance Loop

The guidance loop runs periodically, analyzing the observation engine's intent summaries to:
- Start/update workflows
- Generate recommendations with evidence
- Push toast notifications when actionable advice is available
- Track workflow progress and timeline events

---

## Frontend (React 18 + TypeScript + Vite)

### Pages
| Route | Description |
|-------|-------------|
| `/` / `/assistant` | Chat Assistant — AI conversation with persistent history |
| `/guidance` | Proactive Guidance — workflows, recommendations, evidence |
| `/skills` | Skills Management — list, enable/disable, validate skills |
| `/knowledge` | Knowledge Packs — import, export, reindex, validate |
| `/activity` | Activity Timeline |
| `/settings` | App Settings — AI provider config, general settings |
| `/about` | About page |
| `/advice-chat` | Advice chat window (separate from main UI) |
| *None* | **SetupWizard** — First-run AI provider configuration |

### Components
- **Sidebar** — Navigation between pages
- **GuidanceToast** — Pop-up toast notifications for proactive guidance
- **ErrorBoundary** — React error boundary wrapper

### Startup Flow
1. Frontend loads → polls `http://127.0.0.1:1420/ready` (up to 30 attempts, 3s timeout each)
2. Server ready → fetches settings via `get_settings` command
3. If AI provider configured → hides main window, shows ChatAssistant
4. If not configured → shows SetupWizard

---

## AI Provider Integration

The app connects to any **OpenAI-compatible** LLM endpoint. Built-in provider presets:
- **OpenAI** — `https://api.openai.com/v1`
- **vLLM** — `http://localhost:8000/v1`
- **Ollama** — `http://localhost:11434/v1`
- **Custom** — Any endpoint supporting the OpenAI chat completions API (OpenRouter, local models, etc.)

Configuration (stored in `settings.json`):
- `name` — Provider display name
- `endpoint` — API URL
- `api_key` — Authentication key
- `model` — Model identifier
- `max_tokens` — Maximum response tokens
- `context_window` — Context window size

The engineering persona system prompt is loaded from `assets/system_prompt.md` and defines the AI as a "technical engineer" with a conversational, proactive teammate tone.

---

## Bundled Content

Skills and knowledge packs are **compiled into the app at build time** via `tauri.conf.json` bundle.resources:

```json
"bundle.resources": {
  "../src/skills": "skills/",           # Skill packs
  "../src/engineering-foundations": "knowledge/",  # Knowledge packs
  "assets": "assets/"                   # System prompts, icons, etc.
}
```

### Pre-bundled Skill Packs
Ansible, checkmk, PostgreSQL, MySQL, MSSQL, EDB PostgreSQL, NagiosLogServer, NagiosXI, Linux, Windows, VMware, OpenShift, Red Hat Virtualization, networking, security, storage

### Engineering Foundations
Foundation docs for Linux, Windows, networking, security, storage with quality standards and relationship tracking.

---

## Build & Development

### Prerequisites
- **Rust 1.77+** (with `cargo`, `rustc`)
- **Node.js 18+** and **npm** (for frontend)
- **Tauri CLI** (`cargo install tauri-cli --version "^2"`)
- **Windows** (target platform — Tauri webview dependencies for Windows)

### Build the App

```bash
# From project root:
cargo build                        # Build all workspace crates
cargo build -p wikilabs-desktop    # Build just the desktop app
```

### Build the Frontend

```bash
cd src/frontend
npm install
npm run build                       # Vite production build → dist/
npm run dev                         # Vite dev server
```

### Full Tauri Build (with frontend bundling)

```bash
cargo tauri build                  # Full desktop build (MSI + NSIS)
```

The Tauri build process:
1. Runs `npm run build` to compile the React frontend
2. Copies the dist output to `src-tauri/gen/frontend/`
3. Builds the Rust backend
4. Packages everything into MSI and NSIS installers

### Run in Development Mode

```bash
cargo tauri dev                    # Tauri dev mode (starts frontend + backend)
```

### CI/CD

GitHub Actions workflows in `.github/workflows/`:
- **ci.yml** — Runs on PR: `cargo test`, `cargo clippy`, `cargo doc` (with warnings as errors)
- **release.yml** — Runs on version tag push: builds and publishes release assets

### Testing

```bash
cargo test                         # Run all workspace tests
cargo clippy                       # Lint check (required before push)
```

---

## Configuration

### App Settings (`settings.json`)

Location: `<app-data-dir>/settings.json`

```json
{
  "ai_provider": {
    "name": "OpenRouter",
    "endpoint": "https://openrouter.ai/api/v1",
    "api_key": "sk-...",
    "model": "anthropic/claude-sonnet-4",
    "max_tokens": 4096,
    "context_window": 128000
  },
  "ui": {
    "minimize_to_tray": true,
    "show_toast_notifications": true,
    "toast_sound": true,
    "pre_startup_cleanup": false
  },
  "appearance": {
    "theme": "dark",
    "user_message_bg": "#1a1a2e",
    "user_message_text": "#fbbf24"
  }
}
```

### Database (`wikilabs.db`)

Location: `<app-data-dir>/wikilabs.db`

SQLite database with tables for:
- Chat messages (per workspace)
- Workspaces (name, customer, metadata)
- Settings (shared with settings.json for persistence)
- Various engine-specific tables initialized via `INIT_SQL`

### Logs

Structured JSON logs via `tracing-subscriber` with `tracing-appender` file rotation.

---

## Windows-Specific Details

- **Subsystem**: `windows` (system tray app, no console window)
- **Installers**: MSI and NSIS (via `tauri.conf.json` bundle.targets)
- **Dependencies**: `winreg` for Windows registry access
- ** DPI Awareness**: Handled via `dpi_awareness.rs`
- **Taskbar Integration**: Custom taskbar behavior via `taskbar.rs`
- **Window Cleanup**: Graceful window handling via `windows_cleanup.rs`

---

## Versioning

- **Rust version**: `1.1.205` (from `Cargo.toml` workspace.package)
- **Tauri app version**: `1.1.179` (from `tauri.conf.json` — may lag behind Rust version)
- **Frontend version**: `1.1.146` (from `src/frontend/package.json`)

The version numbers across layers may differ — the Rust workspace version is the authoritative one.

---

## License

MIT License — Copyright (c) Wiki Labs Team

See [CHANGELOG.md](CHANGELOG.md) for the complete release history.