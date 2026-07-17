---
description: "Repository organization and coding conventions for Wiki Labs AI Copilot — directory tree, crate layout, Tauri app structure, frontend, MCP skills, CI/CD, tests, and data directories."
icon: folder-tree
---

# Wiki Labs AI Copilot — Repository Structure

This document defines the canonical layout of the Wiki Labs AI Copilot repository. It covers every directory and file, Rust crate organization, Tauri app structure, frontend architecture, MCP skills layout, build scripts, tests, CI/CD, and data directories.

## Top-Level Directory Tree

```
wikilabs-ai-copilot/
├── Cargo.toml                          # Workspace manifest (Rust)
├── Cargo.lock                          # Lockfile (Rust)
├── pnpm-workspace.yaml                  # pnpm workspace (frontend)
├── package.json                        # Root package.json (scripts, linting)
├── frontend/                           # React frontend application
├── src/                                # Rust workspace crates
│   ├── obs_engine/                     # Observation Engine
│   ├── intent_recognition/             # Intent Recognition Engine
│   ├── mcp_client/                     # MCP Skill Manager & Client
│   ├── knowledge_store/                # Knowledge System (import, index, search)
│   ├── ai_provider/                    # AI Provider Abstraction
│   ├── security/                       # Security Manager (credentials, encryption)
│   ├── memory/                         # Memory System (short/long term)
│   ├── event_bus/                      # Event Bus (Tokio channels)
│   ├── rpc/                            # JSON-RPC Server
│   ├── core/                           # Core orchestrator (glues all crates)
│   ├── skills/                         # MCP Server binaries (skills)
│   └── main.rs                         # Tauri app entry point
├── tauri/                              # Tauri v2 configuration
│   ├── tauri.conf.json                 # Tauri app config
│   ├── tauri.capabilities.json         # Tauri capabilities / permissions
│   ├── permissions/                    # Fine-grained permission definitions
│   └── icons/                          # App icons (app-icon, tray)
├── skills/                             # MCP Skill repository (skills as packages)
│   └── <skill-id>/                     # Per-skill directory (see MCP Skills section)
├── docs/                               # Design documentation
├── tests/                              # Integration and E2E tests
├── .github/                            # GitHub Actions workflows
├── .husky/                             # Git hooks (pre-commit, pre-push)
├── .vscode/                            # VS Code settings / launch configs
├── migrations/                         # Database migration files
├── scripts/                            # Build and dev scripts
├── .gitignore                          # Git ignore rules
├── .prettierrc                         # Prettier config
├── .eslintrc                           # ESLint config
├── README.md                           # Project overview
└── CHANGELOG.md                        # Version history
```

---

## 1. Rust Core Crate Organization

The Rust workspace is organized into multiple crates under `src/`. Each crate is a separate library with its own `Cargo.toml`, focused on a single domain. The `core` crate orchestrates them all. The `main.rs` at workspace root is the Tauri app entry point.

```
src/
├── main.rs                             # Tauri app: setup, event loop, tray, launch
│
├── obs_engine/                         # Observation Engine
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                      # Public API, feature gates
│   │   ├── screen/                     # Screenshot capture
│   │   │   ├── mod.rs                  # Module definition
│   │   │   ├── capturer.rs             # Cross-platform screenshot (screenshoter)
│   │   │   ├── region.rs               # Capture region types
│   │   │   └── resolution.rs           # Resolution limiting (privacy)
│   │   ├── app/                        # Application monitor
│   │   │   ├── mod.rs
│   │   │   ├── window.rs               # Window detection (rdev / native API)
│   │   │   ├── browser.rs              # Browser URL extraction
│   │   │   └── terminal.rs             # Terminal session detection
│   │   ├── terminal/                   # Terminal observer
│   │   │   ├── mod.rs
│   │   │   ├── watcher.rs              # Command/output watcher
│   │   │   ├── shell.rs                # Shell type detection
│   │   │   └── record.rs               # Command record types
│   │   ├── clipboard/                  # Clipboard observer
│   │   │   ├── mod.rs
│   │   │   ├── monitor.rs              # Change notification
│   │   │   └── sanitizer.rs            # Credential filtering
│   │   ├── event.rs                    # ObservationEvent types
│   │   ├── engine.rs                   # Main observation engine (orchestrator)
│   │   ├── policy.rs                   # Privacy policy / permission checks
│   │   └── scheduler.rs                # Polling interval management
│   └── tests/
│       └── integration.rs              # Observation integration tests
│
├── intent_recognition/                 # Intent Recognition Engine
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── engine.rs                   # Main intent analysis engine
│   │   ├── pattern.rs                  # Rule-based pattern matching
│   │   ├── hypothesis.rs               # Technology hypothesis scoring
│   │   ├── temporal.rs                 # Temporal smoothing / history
│   │   ├── confidence.rs               # Confidence thresholding
│   │   ├── evidence.rs                 # Evidence accumulation
│   │   └── intent.rs                   # Intent struct and serialization
│   └── tests/
│       └── pattern_tests.rs            # Pattern recognition unit tests
│
├── mcp_client/                         # MCP Skill Manager & Client
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── manager.rs                  # Skill lifecycle: discover, load, start, stop
│   │   ├── registry.rs                 # Skill registry (SQLite-backed)
│   │   ├── client.rs                   # MCP stdio client (JSON-RPC)
│   │   ├── transport.rs                # Transport abstraction (stdio/HTTP)
│   │   ├── tools.rs                    # Tool invocation & routing
│   │   ├── resources.rs                # Resource fetching
│   │   ├── prompts.rs                  # Prompt templates
│   │   ├── health.rs                   # Health check & auto-restart
│   │   ├── relevance.rs                # Skill relevance scoring for intent
│   │   ├── install.rs                  # Skill install/uninstall/update
│   │   └── skill.rs                    # Skill metadata & config types
│   └── tests/
│       └── client_tests.rs             # MCP client communication tests
│
├── knowledge_store/                    # Knowledge System
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── importer/                   # Knowledge import
│   │   │   ├── mod.rs
│   │   │   ├── file.rs                 # File import (PDF, MD, HTML, TXT, DOCX)
│   │   │   ├── url.rs                  # URL import
│   │   │   └── parser.rs               # Format-specific parsers
│   │   ├── processor/                  # Document processing
│   │   │   ├── mod.rs
│   │   │   ├── chunker.rs              # Text chunking (512 tokens, 64 overlap)
│   │   │   ├── metadata.rs             # Metadata extraction
│   │   │   └── embedder.rs             # Embedding generation via ai_provider
│   │   ├── search/                     # Knowledge search
│   │   │   ├── mod.rs
│   │   │   ├── vector.rs               # ChromaDB vector search
│   │   │   ├── fts.rs                  # SQLite FTS5 search
│   │   │   └── hybrid.rs               # Weighted merge (70/30)
│   │   ├── document.rs                 # Knowledge doc & chunk types
│   │   ├── index.rs                    # Index management (refresh, reindex)
│   │   └── system.rs                   # Public knowledge system API
│   └── tests/
│       ├── chunker_tests.rs            # Chunking correctness
│       └── search_tests.rs             # Search quality tests
│
├── ai_provider/                        # AI Provider Abstraction
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── provider.rs                 # AiProvider trait definition
│   │   ├── chat.rs                     # Chat message types & params
│   │   ├── embed.rs                    # Embedding types & params
│   │   ├── openai/                     # OpenAI provider
│   │   │   ├── mod.rs
│   │   │   ├── chat.rs                 # OpenAI chat API
│   │   │   └── embed.rs                # OpenAI embedding API
│   │   ├── openai_compatible/          # OpenAI-compatible provider (vLLM, LM Studio)
│   │   │   ├── mod.rs
│   │   │   └── client.rs               # Generic OpenAI-compatible client
│   │   ├── vllm/                       # vLLM provider
│   │   │   ├── mod.rs
│   │   │   └── client.rs               # vLLM-specific client
│   │   ├── ollama/                     # Ollama provider
│   │   │   ├── mod.rs
│   │   │   └── client.rs               # Ollama API client
│   │   ├── enterprise/                 # Enterprise custom provider
│   │   │   ├── mod.rs
│   │   │   └── client.rs               # Custom auth/endpoint client
│   │   └── factory.rs                  # Provider factory (config → provider)
│   └── tests/
│       └── provider_tests.rs           # Provider trait implementation tests
│
├── security/                           # Security Manager
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── credential_manager.rs       # OS credential storage (keyring crate)
│   │   ├── encryption.rs               # AES-256-GCM + Argon2id encryption
│   │   ├── permissions.rs              # Permission evaluation / sandbox gating
│   │   ├── audit_log.rs                # Audit logging (SQLite + append-only file)
│   │   ├── system_log.rs               # System logging (structured)
│   │   └── sanitization.rs             # Input sanitization (credential detection)
│   └── tests/
│       └── security_tests.rs           # Encryption, credential, permission tests
│
├── memory/                             # Memory System
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── short_term.rs               # Short-term memory (session-scoped, TTL)
│   │   ├── long_term.rs                # Long-term memory (workspace-scoped, persistent)
│   │   ├── storage.rs                  # SQLite-backed memory storage
│   │   ├── compaction.rs               # Periodic compaction / pruning
│   │   └── types.rs                    # Memory entry types & serialization
│   └── tests/
│       └── memory_tests.rs             # Memory lifecycle tests
│
├── event_bus/                          # Event Bus (Tokio channels)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── bus.rs                      # Central event bus (pub/sub)
│   │   ├── channels.rs                 # Typed channel definitions
│   │   ├── events.rs                   # Event type definitions
│   │   └── handler.rs                  # Handler subscription management
│   └── tests/
│       └── bus_tests.rs                # Event routing correctness tests
│
├── rpc/                                # JSON-RPC Server
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── server.rs                   # JSON-RPC server (axum)
│   │   ├── handlers/                   # RPC request handlers
│   │   │   ├── chat.rs                 # rpc:chat_send, rpc:chat_stream
│   │   │   ├── workspace.rs            # Workspace CRUD
│   │   │   ├── suggestion.rs           # Suggestion accept/dismiss
│   │   │   ├── skill.rs                # Skill enable/disable/config
│   │   │   ├── knowledge.rs            # Knowledge import/search/list
│   │   │   ├── settings.rs             # Settings get/update
│   │   │   ├── logs.rs                 # Audit/system log queries
│   │   │   └── observation.rs          # Observation toggle
│   │   ├── websocket.rs                # WebSocket event forwarding
│   │   ├── types.rs                    # Request/response types
│   │   └── error.rs                    # RPC error handling
│   └── tests/
│       └── rpc_tests.rs                # RPC endpoint tests
│
├── core/                               # Core Orchestrator (glues all crates)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                      # Core public API
│   │   ├── app.rs                      # App lifecycle: init, run, shutdown
│   │   ├── config.rs                   # Configuration management
│   │   ├── scheduler.rs                # Background task scheduling
│   │   ├── memory_manager.rs           # Memory lifecycle management
│   │   └── state.rs                    # Shared application state
│   └── tests/
│       └── core_tests.rs               # Full pipeline tests
│
├── skills/                             # MCP Server Binaries (skill implementations)
│   ├── openshift/                      # OpenShift MCP Server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs                 # MCP server entry point
│   │   │   ├── tools.rs                # Tool definitions (oc, kubectl helpers)
│   │   │   ├── resources.rs            # Resource definitions (docs, templates)
│   │   │   ├── prompts.rs              # Prompt templates
│   │   │   └── workflow.rs             # OpenShift troubleshooting workflow
│   │   └── knowledge/                  # Knowledge base files
│   │       ├── installation.md
│   │       ├── troubleshooting.md
│   │       ├── best-practices.md
│   │       └── incidents/              # Past incident reference docs
│   │
│   ├── linux/                          # Linux MCP Server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── tools.rs
│   │   │   ├── resources.rs
│   │   │   ├── prompts.rs
│   │   │   └── workflow.rs
│   │   └── knowledge/
│   │       ├── administration.md
│   │       ├── performance.md
│   │       ├── networking.md
│   │       └── security.md
│   │
│   ├── vmware/                         # VMware vSphere MCP Server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── tools.rs
│   │   │   ├── resources.rs
│   │   │   ├── prompts.rs
│   │   │   └── workflow.rs
│   │   └── knowledge/
│   │       ├── administration.md
│   │       ├── performance.md
│   │       └── troubleshooting.md
│   │
│   ├── ansible/                        # Ansible MCP Server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── tools.rs
│   │   │   ├── resources.rs
│   │   │   ├── prompts.rs
│   │   │   └── workflow.rs
│   │   └── knowledge/
│   │       ├── getting-started.md
│   │       ├── playbooks.md
│   │       └── best-practices.md
│   │
│   ├── nagios/                         # Nagios MCP Server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── tools.rs
│   │   │   ├── resources.rs
│   │   │   ├── prompts.rs
│   │   │   └── workflow.rs
│   │   └── knowledge/
│   │       ├── monitoring.md
│   │       └── alerting.md
│   │
│   ├── mysql/                          # MySQL MCP Server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── tools.rs
│   │   │   ├── resources.rs
│   │   │   ├── prompts.rs
│   │   │   └── workflow.rs
│   │   └── knowledge/
│   │       ├── administration.md
│   │       ├── performance.md
│   │       └── backup-recovery.md
│   │
│   └── ...                             # Additional skills as needed
│
└── migrations/                         # Database migrations (SQL)
    ├── 001_initial_schema.sql          # Base schema (workspaces, sessions, skills, etc.)
    ├── 002_knowledge_tables.sql        # Knowledge tables + FTS5 virtual table
    ├── 003_memory_tables.sql           # Short-term / long-term memory tables
    ├── 004_workflow_tables.sql         # Workflow & workflow_runs tables
    ├── 005_audit_tables.sql            # Audit log & system log tables
    └── README.md                       # Migration conventions
```

### Crate Dependency Graph

```
main.rs
  └── core
       ├── event_bus
       │    └── (no deps — foundation)
       ├── obs_engine
       │    └── event_bus
       ├── intent_recognition
       │    └── event_bus
       ├── mcp_client
       │    ├── event_bus
       │    └── (depends on rpc types)
       ├── knowledge_store
       │    ├── ai_provider
       │    ├── event_bus
       │    └── security (encryption)
       ├── ai_provider
       │    └── (reqwest + serde)
       ├── security
       │    └── (keyring + crypto)
       ├── memory
       │    └── (rusqlite)
       └── rpc
            ├── event_bus
            ├── ai_provider
            ├── mcp_client
            ├── obs_engine
            ├── knowledge_store
            ├── security
            └── memory
```

### Cargo Workspace Manifest

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "src/core",
    "src/obs_engine",
    "src/intent_recognition",
    "src/mcp_client",
    "src/knowledge_store",
    "src/ai_provider",
    "src/security",
    "src/memory",
    "src/event_bus",
    "src/rpc",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Wiki Labs AI Copilot Team"]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled", "fts5"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
tauri = { version = "2", features = [] }
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.10"
keyring = "3"
```

---

## 2. Tauri App Structure

```
src/main.rs                          # Tauri app entry point
```

```rust
// src/main.rs — Key responsibilities:
// 1. Initialize Tokio runtime
// 2. Create core app state (all crates)
// 3. Start RPC server on localhost
// 4. Register Tauri commands
// 5. Set up Tauri window + system tray
// 6. Launch event loop

use tauri::Manager;
use wikilabs_core::Core;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize core engine
            let core = Core::new(app.path_resolver());
            app.manage(core);
            
            // Start RPC server in background tokio task
            // Connect frontend to http://127.0.0.1:<port>/rpc
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Workspace commands
            rpc::handlers::workspace::create,
            rpc::handlers::workspace::list,
            rpc::handlers::workspace::get,
            rpc::handlers::workspace::update,
            rpc::handlers::workspace::delete,
            rpc::handlers::workspace::get_context,
            
            // Chat commands
            rpc::handlers::chat::send,
            rpc::handlers::chat::stream,
            rpc::handlers::chat::list,
            rpc::handlers::chat::delete,
            
            // Suggestion commands
            rpc::handlers::suggestion::accept,
            rpc::handlers::suggestion::dismiss,
            rpc::handlers::suggestion::list,
            
            // Skill commands
            rpc::handlers::skill::enable,
            rpc::handlers::skill::disable,
            rpc::handlers::skill::get_config,
            rpc::handlers::skill::set_config,
            rpc::handlers::skill::discover,
            rpc::handlers::skill::get_status,
            rpc::handlers::skill::call_tool,
            
            // Knowledge commands
            rpc::handlers::knowledge::import,
            rpc::handlers::knowledge::search,
            rpc::handlers::knowledge::list,
            rpc::handlers::knowledge::delete,
            rpc::handlers::knowledge::refresh,
            rpc::handlers::knowledge::get_document,
            
            // Settings commands
            rpc::handlers::settings::get,
            rpc::handlers::settings::update,
            
            // Log commands
            rpc::handlers::logs::audit_list,
            rpc::handlers::logs::system_list,
            
            // Observation commands
            rpc::handlers::observation::toggle_screen,
            rpc::handlers::observation::toggle_terminal,
            rpc::handlers::observation::toggle_clipboard,
            rpc::handlers::observation::get_status,
            
            // Native OS commands (Tauri IPC)
            tauri::plugin::shell::commands::open,
            // File dialogs, notifications, etc.
        ])
        .system_tray(tauri::SystemTray::new())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Graceful shutdown: stop MCP servers, save state
            }
        });
}
```

### Tauri Configuration

```json
// tauri/tauri.conf.json
{
  "productName": "Wiki Labs AI Copilot",
  "version": "0.1.0",
  "identifier": "com.wikilabs.copilot",
  "build": {
    "frontendDist": "../frontend/dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Wiki Labs AI Copilot",
        "width": 1280,
        "height": 800,
        "resizable": true,
        "center": true,
        "decorations": true,
        "transparent": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'wss://*'; connect-src 'self' https://*; style-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "dmg"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.ico",
      "icons/icon.png"
    ]
  }
}
```

### Permissions Model

```
tauri/permissions/
├── default.json                    # Default permissions (baseline)
├── observation-screen.json         # Screen capture permission
├── observation-terminal.json       # Terminal monitoring permission
├── observation-clipboard.json      # Clipboard monitoring permission
├── file-dialog.json                # File dialog access
├── notification.json               # System notifications
├── deep-link.json                  # Deep link handling
├── credential-manager.json         # OS credential manager access
└── net.json                        # Network access (AI providers, updates)
```

### System Tray

```
src/system_tray.rs                   # System tray with native integration
├── show_window                       # Show main window
├── quit                              # Quit application
├── toggle_observation                # Toggle observation on/off
├── new_conversation                  # Create new conversation
└── open_logs                         # Open logs panel
```

---

## 3. Frontend Structure

```
frontend/
├── package.json                      # Frontend dependencies & scripts
├── vite.config.ts                    # Vite configuration
├── tsconfig.json                     # TypeScript configuration
├── tailwind.config.ts                # Tailwind CSS configuration
├── postcss.config.js                 # PostCSS configuration
├── index.html                        # Entry HTML
├── public/                           # Static assets (served by Vite)
│   └── favicon.svg
│
├── src/
│   ├── main.tsx                      # React entry point (renders App)
│   ├── App.tsx                       # Root component + provider setup
│   ├── AppRoutes.tsx                 # Route definitions
│   │
│   ├── components/                   # Reusable UI components
│   │   ├── ui/                       # Base UI primitives (Radix + Tailwind)
│   │   │   ├── button.tsx
│   │   │   ├── dialog.tsx
│   │   │   ├── input.tsx
│   │   │   ├── select.tsx
│   │   │   ├── tooltip.tsx
│   │   │   ├── tabs.tsx
│   │   │   ├── badge.tsx
│   │   │   ├── toast.tsx
│   │   │   ├── alert.tsx
│   │   │   ├── card.tsx
│   │   │   ├── sidebar.tsx
│   │   │   ├── scroll-area.tsx
│   │   │   └── separator.tsx
│   │   ├── layout/                   # Layout components
│   │   │   ├── app-shell.tsx         # Main app shell with sidebar
│   │   │   ├── sidebar.tsx           # Left sidebar (panel navigation)
│   │   │   ├── panel-tabs.tsx        # Panel tab bar
│   │   │   └── workspace-indicator.tsx
│   │   ├── chat/                     # Chat UI components
│   │   │   ├── message-bubble.tsx    # Single message (user/assistant/system)
│   │   │   ├── message-list.tsx      # Conversation message list
│   │   │   ├── code-block.tsx        # Syntax-highlighted code
│   │   │   ├── citation-link.tsx     # Inline knowledge citation
│   │   │   ├── suggestion-card.tsx   # AI suggestion accept/dismiss
│   │   │   ├── streaming-cursor.tsx  # Blinking cursor for streaming
│   │   │   └── message-input.tsx     # Chat input with toolbar
│   │   ├── workspace/                # Workspace UI components
│   │   │   ├── workspace-list.tsx    # Workspace list in sidebar
│   │   │   ├── workspace-card.tsx    # Individual workspace card
│   │   │   ├── workspace-selector.tsx
│   │   │   └── stack-tags.tsx        # Technology stack tags
│   │   ├── skills/                   # Skill management UI
│   │   │   ├── skill-card.tsx
│   │   │   ├── skill-toggle.tsx
│   │   │   ├── skill-config-dialog.tsx
│   │   │   └── skill-status-indicator.tsx
│   │   ├── settings/                 # Settings UI
│   │   │   ├── settings-panel.tsx
│   │   │   ├── ai-provider-form.tsx
│   │   │   ├── observation-controls.tsx
│   │   │   ├── privacy-settings.tsx
│   │   │   └── appearance-settings.tsx
│   │   ├── knowledge/                # Knowledge UI
│   │   │   ├── knowledge-panel.tsx
│   │   │   ├── import-dialog.tsx
│   │   │   ├── search-results.tsx
│   │   │   └── document-card.tsx
│   │   ├── logs/                     # Log UI
│   │   │   ├── audit-log-panel.tsx
│   │   │   ├── system-log-panel.tsx
│   │   │   └── log-entry.tsx
│   │   └── suggestions/              # Suggestion UI
│   │       ├── suggestion-panel.tsx
│   │       └── suggestion-list.tsx
│   │
│   ├── pages/                        # Full-page views
│   │   ├── chat-page.tsx             # AI Chat page (primary interaction surface)
│   │   ├── workspace-page.tsx        # Workspace configuration page
│   │   ├── skills-page.tsx           # Skills management page
│   │   ├── knowledge-page.tsx        # Knowledge base page
│   │   ├── settings-page.tsx         # Settings page
│   │   ├── logs-page.tsx             # Logs & audit page
│   │   └── report-page.tsx           # Reports & audit trail page
│   │
│   ├── hooks/                        # Custom React hooks
│   │   ├── use-rpc.ts                # JSON-RPC client with retry logic
│   │   ├── use-websocket.ts          # WebSocket event subscription
│   │   ├── use-ai-stream.ts          # Streaming AI response handler
│   │   ├── use-obs-status.ts         # Observation status tracker
│   │   ├── use-workspace-context.ts  # Active workspace context
│   │   ├── use-suggestions.ts        # Real-time suggestion receiver
│   │   ├── use-theme.ts              # Theme toggle (dark/light)
│   │   └── use-connection-status.ts  # Core connection health
│   │
│   ├── stores/                       # Redux Toolkit slices
│   │   ├── index.ts                  # Store configuration (configureStore)
│   │   ├── chat-slice.ts             # Chat state (messages, sessions, streaming)
│   │   ├── workspace-slice.ts        # Workspace state (list, active, switching)
│   │   ├── skills-slice.ts           # Skills state (list, enabled, config)
│   │   ├── knowledge-slice.ts        # Knowledge state (docs, search results)
│   │   ├── settings-slice.ts         # Settings state
│   │   ├── logs-slice.ts             # Log state
│   │   ├── suggestions-slice.ts      # Real-time suggestions
│   │   ├── notification-slice.ts     # Toast/notification state
│   │   └── connection-slice.ts       # Connection status state
│   │
│   ├── services/                     # API/RPC clients
│   │   ├── rpc-client.ts             # JSON-RPC over HTTP client
│   │   ├── websocket-client.ts       # WebSocket event client
│   │   ├── ai-service.ts             # AI streaming service
│   │   ├── workspace-service.ts      # Workspace CRUD service
│   │   ├── skill-service.ts          # Skill management service
│   │   ├── knowledge-service.ts      # Knowledge import/search service
│   │   ├── settings-service.ts       # Settings service
│   │   └── logs-service.ts           # Log query service
│   │
│   ├── types/                        # TypeScript type definitions
│   │   ├── chat.ts                   # ChatMessage, ChatResponse, ChatChunk
│   │   ├── workspace.ts              # Workspace, WorkspaceStack
│   │   ├── skill.ts                  # SkillMetadata, SkillConfig, ToolDef
│   │   ├── knowledge.ts              # KnowledgeDoc, KnowledgeResult
│   │   ├── settings.ts               # AppSettings, ObservationPrefs
│   │   ├── log.ts                    # AuditLogEntry, SystemLogEntry
│   │   ├── suggestion.ts             # Suggestion types
│   │   └── intent.ts                 # Intent, Evidence types
│   │
│   ├── utils/                        # Utility functions
│   │   ├── markdown.ts               # Markdown rendering helpers
│   │   ├── code-highlight.ts         # Syntax highlighting config
│   │   ├── format.ts                 # Timestamp, size formatting
│   │   └── error.ts                  # Error formatting & display
│   │
│   └── assets/                       # Static assets
│       ├── images/                   # Images & illustrations
│       ├── styles/
│       │   ├── globals.css           # Global styles + Tailwind imports
│       │   └── themes/
│       │       ├── dark.css          # Dark theme CSS variables
│       │       └── light.css         # Light theme CSS variables
│       └── fonts/                    # Custom fonts (if any)
│
├── tests/                            # Frontend tests
│   ├── setup.ts                      # Test setup (RTL, vitest globals)
│   ├── mocks/                        # MSW handlers
│   │   ├── rpc-mocks.ts              # Mock JSON-RPC endpoints
│   │   └── websocket-mocks.ts        # Mock WebSocket events
│   ├── unit/                         # Unit tests
│   │   ├── chat.test.tsx             # Message bubble rendering
│   │   ├── workspace.test.tsx        # Workspace list rendering
│   │   └── skills.test.tsx           # Skill toggle rendering
│   └── integration/                  # Integration tests
│       └── chat-flow.test.tsx        # Multi-step conversation flow
│
└── vitest.config.ts                  # Vitest configuration
```

---

## 4. MCP Skills Repository Layout

Each MCP skill is a self-contained package under `skills/`. The pattern follows the MCP specification and mirrors the OpenHuman domain-driven approach.

```
skills/
├── README.md                         # Skills overview & contributing guide
│
├── openshift/                        # OpenShift MCP Skill
│   ├── skill.json                    # Skill metadata (id, name, version, description, icon)
│   ├── Cargo.toml                    # Rust crate manifest for the server binary
│   ├── src/
│   │   └── main.rs                   # MCP server entry point
│   ├── knowledge/                    # Knowledge base files
│   │   ├── installation.md           # Installation guides
│   │   ├── administration.md         # Administration docs
│   │   ├── troubleshooting.md        # Troubleshooting guides
│   │   ├── best-practices.md         # Best practices & patterns
│   │   └── incidents/                # Past incident reference docs
│   │       ├── 2024-03-15-crashloop.md
│   │       └── 2024-07-02-etcd.md
│   └── tools/                        # Tool definitions (referenced in main.rs)
│       ├── oc_execute.md             # Tool: run oc/kubectl commands
│       ├── pod_analyze.md            # Tool: analyze pod status
│       └── event_list.md             # Tool: list events
│
├── linux/                          # Linux MCP Skill
│   ├── skill.json
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   ├── knowledge/
│   │   ├── administration.md
│   │   ├── performance.md
│   │   ├── networking.md
│   │   ├── security.md
│   │   └── incidents/
│   └── tools/
│       ├── command_execute.md
│       ├── log_analyze.md
│       └── system_info.md
│
├── vmware/                           # VMware vSphere MCP Skill
│   ├── skill.json
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   ├── knowledge/
│   │   ├── administration.md
│   │   ├── performance.md
│   │   ├── troubleshooting.md
│   │   └── incidents/
│   └── tools/
│       ├── vm_inspect.md
│       ├── esxi_health.md
│       └── datastore_analyze.md
│
├── ansible/                          # Ansible MCP Skill
│   ├── skill.json
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   ├── knowledge/
│   │   ├── getting-started.md
│   │   ├── playbooks.md
│   │   └── best-practices.md
│   └── tools/
│       ├── playbook_validate.md
│       ├── inventory_analyze.md
│       └── module_lookup.md
│
├── nagios/                           # Nagios MCP Skill
│   ├── skill.json
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   ├── knowledge/
│   │   ├── monitoring.md
│   │   └── alerting.md
│   └── tools/
│       ├── status_check.md
│       ├── history_query.md
│       └── contact_lookup.md
│
├── mysql/                            # MySQL MCP Skill
│   ├── skill.json
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   ├── knowledge/
│   │   ├── administration.md
│   │   ├── performance.md
│   │   └── backup-recovery.md
│   └── tools/
│       ├── query_execute.md
│       ├── slow_query_analyze.md
│       └── backup_restore.md
│
└── ...                               # Additional skills (EKS, Docker, Kubernetes, etc.)
```

### Skill Metadata Schema (`skill.json`)

```json
{
  "id": "openshift",
  "name": "OpenShift",
  "version": "1.0.0",
  "description": "Red Hat OpenShift cluster management and troubleshooting",
  "icon": "🔴",
  "category": "cloud-infrastructure",
  "technologies": ["openshift", "kubernetes", "ocp"],
  "tools": [
    {"name": "oc_execute", "description": "Run OpenShift CLI commands", "requires_auth": true},
    {"name": "pod_analyze", "description": "Analyze pod status and events", "requires_auth": false},
    {"name": "event_list", "description": "List recent cluster events", "requires_auth": false}
  ],
  "knowledge_files": ["installation.md", "troubleshooting.md", "best-practices.md"],
  "intent_patterns": [
    {"signal": "oc", "weight": 0.9},
    {"signal": "openshift", "weight": 0.8},
    {"signal": "ocp", "weight": 0.7},
    {"signal": "kubernetes", "weight": 0.5},
    {"signal": "kubectl", "weight": 0.4}
  ]
}
```

---

## 5. Config and Assets

```
tauri/
├── tauri.conf.json                   # Tauri app configuration
├── tauri.capabilities.json           # Default Tauri capabilities
└── permissions/                      # Permission definition files (see Section 2)

icons/                                # App icons
├── 32x32.png                         # App icon 32x32
├── 64x64.png                         # App icon 64x64
├── 128x128.png                       # App icon 128x128
├── 128x128@2x.png                    # App icon 256x256
├── icon.ico                          # Windows ICO (all sizes)
├── icon.png                          # macOS icon (1024x1024)
└── tray/
    ├── tray-icon.png                 # System tray icon
    └── tray-icon-dark.png            # System tray icon (dark mode)

frontend/src/assets/
├── styles/
│   └── themes/
│       ├── dark.css                  # Dark theme CSS variables
│       └── light.css                 # Light theme CSS variables
```

### App Settings Defaults

```
scripts/
├── default-settings.json             # Default settings for first-run
└── update-check.sh                   # Script to check for updates
```

---

## 6. Test Directory Structure

```
tests/                                # Rust integration tests & E2E tests
│
├── integration/                      # Rust integration tests (tests/ dir in workspace)
│   ├── mod.rs                        # Integration test module
│   ├── observation_integration.rs    # End-to-end observation pipeline
│   ├── intent_integration.rs         # Intent recognition pipeline
│   ├── mcp_integration.rs            # MCP client-server communication
│   ├── knowledge_integration.rs      # Knowledge import and search
│   ├── rpc_integration.rs            # RPC handler tests
│   └── skill_lifecycle.rs            # Skill start/stop/restart lifecycle
│
├── e2e/                              # Playwright end-to-end tests
│   ├── package.json                  # E2E test dependencies (Playwright)
│   ├── playwright.config.ts          # Playwright configuration
│   ├── fixtures/                     # Test fixtures
│   │   ├── app.ts                    # Test app fixture (launch Tauri app)
│   │   ├── workspace.ts              # Workspace creation fixture
│   │   └── skill.ts                  # Skill installation fixture
│   ├── chat.spec.ts                  # Chat flow E2E tests
│   ├── workspace.spec.ts             # Workspace management E2E tests
│   ├── skills.spec.ts                # Skill management E2E tests
│   ├── knowledge.spec.ts             # Knowledge import/search E2E tests
│   ├── settings.spec.ts              # Settings E2E tests
│   └── observation.spec.ts           # Observation control E2E tests
│
├── fixtures/                         # Shared test fixtures
│   ├── sample-docs/                  # Sample knowledge documents
│   │   ├── sample-install-guide.md
│   │   ├── sample-error-log.txt
│   │   └── sample-config.yaml
│   ├── skill-packages/               # Mock skill packages for testing
│   │   ├── mock-openshift/
│   │   │   ├── skill.json
│   │   │   └── Cargo.toml
│   │   └── mock-linux/
│   │       ├── skill.json
│   │       └── Cargo.toml
│   └── db-dumps/                     # Pre-populated database snapshots
│       └── sample-workspace.sqlite
│
frontend/tests/                       # Frontend tests (see Section 3)
├── setup.ts
├── mocks/
├── unit/
└── integration/
```

---

## 7. CI/CD Configuration

```
.github/
├── workflows/
│   ├── ci-lite.yml                   # Fast CI: lint + unit tests on changed files
│   ├── ci-full.yml                   # Full CI: complete test suite + E2E + builds
│   ├── release.yml                   # Release workflow: versioning + publish
│   └── skill-release.yml             # MCP skill package release workflow
│
├── actions/                          # Reusable GitHub Actions
│   └── setup-toolchain/              # Shared Rust + frontend setup
│       └── action.yml
│
├── codeql.yml                        # CodeQL security analysis
├── dependabot.yml                    # Dependabot for automated updates
├── PULL_REQUEST_TEMPLATE.md          # PR template
└── ISSUE_TEMPLATE/
    ├── bug_report.md
    ├── feature_request.md
    └── config.yml
```

### CI Lite (`ci-lite.yml`)

```yaml
# .github/workflows/ci-lite.yml
name: CI Lite
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with: { node-version: '22', cache: 'pnpm' }
      - run: pnpm install
      - run: pnpm run lint
      - run: pnpm run typecheck
      - run: pnpm run test:unit -- --related
        env: { CI: true }

  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo check --all
      - run: cargo llvm-cov --changed --lcov --output-path lcov.info
        env: { CARGO_LLVM_COV_TARGET_DIR: target/llvm-cov }
      - uses: 57bits/coverage-github-action@v1
        with: { filename: lcov.info, threshold: 80 }
```

### CI Full (`ci-full.yml`)

```yaml
# .github/workflows/ci-full.yml
name: CI Full
on:
  pull_request:
    branches: [release]
  schedule:
    - cron: '0 6 * * 1'  # Every Monday 6 AM UTC

jobs:
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pnpm install && pnpm run test -- --coverage

  rust:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --all
      - run: cargo llvm-cov --lcov --output-path lcov.info

  e2e-windows:
    runs-on: windows-latest
    needs: [frontend, rust]
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: pnpm install && cargo build --release
      - run: npx playwright test --project=windows

  e2e-macos:
    runs-on: macos-latest
    needs: [frontend, rust]
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: pnpm install && cargo build --release
      - run: npx playwright test --project=macos

  build:
    runs-on: macos-latest
    needs: [frontend, rust]
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo tauri build
      - uses: actions/upload-artifact@v4
        with: { name: dmg, path: target/release/bundle/dmg/ }

  build-windows:
    runs-on: windows-latest
    needs: [frontend, rust]
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo tauri build
      - uses: actions/upload-artifact@v4
        with: { name: msi, path: target/release/bundle/msi/ }
```

---

## 8. Build Scripts

```
scripts/
├── build-rust.sh                     # Rust build: check, format, lint, test, coverage
│
├── build-frontend.sh                 # Frontend build: install, typecheck, lint, test, bundle
│
├── build-tauri.sh                    # Tauri bundler: dev build + release build
│
├── dev.sh                            # Development: run frontend dev + Tauri dev
│
├── setup.sh                          # One-time setup: install system deps
│
├── migrate-db.sh                     # Apply pending database migrations
│
├── export-skill.sh                   # Package a skill as distributable zip
│
├── check-update.sh                   # Check for version updates
│
└── sign-macos.sh                     # macOS code signing script
```

### Build Script Summaries

```bash
# scripts/build-rust.sh
# ───────────────────
# Usage: ./scripts/build-rust.sh [target]
# Targets: check, format, lint, test, coverage, all
#
set -euo pipefail
TARGET="${1:-all}"

case "$TARGET" in
  check)   cargo check --all ;;
  format)  cargo fmt --check --all ;;
  lint)    cargo clippy -- -D warnings ;;
  test)    cargo test --all ;;
  coverage) cargo llvm-cov --all-features --lcov ;;
  all)     cargo fmt --check && cargo clippy -- -D warnings && cargo test --all && cargo llvm-cov --all-features --lcov ;;
esac

# scripts/build-frontend.sh
# ──────────────────────────
# Usage: ./scripts/build-frontend.sh [target]
# Targets: install, typecheck, lint, test, build, all
#
set -euo pipefail
TARGET="${1:-all}"
cd frontend

case "$TARGET" in
  install)  pnpm install ;;
  typecheck) pnpm run typecheck ;;
  lint)     pnpm run lint ;;
  test)     pnpm run test ;;
  build)    pnpm run build ;;
  all)      pnpm install && pnpm run typecheck && pnpm run lint && pnpm run test && pnpm run build ;;
esac

# scripts/build-tauri.sh
# ──────────────────────
# Usage: ./scripts/build-tauri.sh [dev|release|debug]
#
set -euo pipefail
MODE="${1:-dev}"

# Prerequisite: frontend build
./scripts/build-frontend.sh build

case "$MODE" in
  dev)    cargo tauri dev ;;
  debug)  cargo tauri build --debug ;;
  release) cargo tauri build ;;
esac
```

---

## 9. Migration and Data Directory Layout

### Database Migrations

```
migrations/                          # SQL migration files (applied in order)
├── 001_initial_schema.sql           # Base tables: users, user_settings, workspaces,
│                                    #   sessions, chat_messages, recommendations, skills,
│                                    #   skill_configs, skill_tools
├── 002_knowledge_tables.sql         # Knowledge tables: knowledge_sources, knowledge_docs,
│                                    #   knowledge_docs_fts (FTS5), knowledge_chunks
├── 003_memory_tables.sql            # Memory tables: short_term_memory, long_term_memory
├── 004_workflow_tables.sql          # Workflow tables: workflows, workflow_runs
├── 005_audit_tables.sql             # Audit tables: audit_logs, system_logs
└── README.md                        # Migration conventions
```

### Migration Conventions

```sql
-- migrations/001_initial_schema.sql (example)
-- Migration: 001_initial_schema
-- Description: Initial database schema with core tables
-- Applied: First time (never re-applied)

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- (All DDL from DATA_MODEL.md — users, workspaces, sessions, chat_messages, etc.)
-- ...
```

Runtime data layout on the engineer's machine:

```
~/.local/share/wikilabs/
├── wikilabs.db                     # SQLite database (applies all migrations)
├── vectors/
│   └── chroma.sqlite3              # ChromaDB embedded vector store
├── files/
│   ├── knowledge/                  # Imported knowledge source files (indexed)
│   ├── screenshots/                # Optional saved screenshots (encrypted, AES-256-GCM)
│   └── exports/                    # Exported reports and summaries
├── config/
│   └── settings.json               # Application settings (JSON fallback)
├── skills/                         # Installed MCP skill packages
│   ├── openshift/                  # Unpacked skill directory
│   ├── linux/
│   └── vmware/
└── logs/
    ├── audit.log                   # Audit log (append-only)
    └── system.log                  # System log (daily rotation)
```

---

## 10. Documentation Structure

```
docs/                                # Design and reference documentation
├── ARCHITECTURE.md                  # System architecture (read ✅)
├── COMPONENT_DESIGN.md              # Detailed component design (read ✅)
├── DATA_MODEL.md                    # Data entities, schemas, storage model (read ✅)
├── TECHNOLOGY_SELECTION.md          # Technology evaluation (read ✅)
├── REPOSITORY_STRUCTURE.md          # This file — repository layout
├── SECURITY_ARCHITECTURE.md         # Security model and threat analysis
├── MCP_ARCHITECTURE.md              # MCP skill architecture and tool design
├── DEVELOPMENT_GUIDE.md             # Developer setup and workflows
├── TESTING_STRATEGY.md              # Testing approach across all levels
├── RELEASE_PROCESS.md               # Release process and versioning
├── CONTRIBUTING.md                  # Contribution guidelines
└── api/
    ├── rpc-api.md                   # JSON-RPC API reference
    ├── websocket-events.md          # WebSocket event reference
    └── tauri-ipc.md                 # Tauri IPC command reference
```

---

## Coding Conventions

### Rust

| Rule | Convention |
|------|-----------|
| Edition | `2021` |
| Formatting | `cargo fmt` (mandatory, checked in CI) |
| Linting | `cargo clippy` with `-D warnings` (mandatory, checked in CI) |
| Error handling | `thiserror` for error types, `anyhow` for application-level errors |
| Async | `tokio` runtime, `async_trait` for async traits |
| Naming | `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants |
| Module organization | One public type per file (or related group), `lib.rs` re-exports |
| Logging | `tracing` crate for structured logging |
| Testing | Unit tests co-located with code (`#[cfg(test)]` modules), integration tests in `tests/` |

### Frontend

| Rule | Convention |
|------|-----------|
| Formatting | Prettier (mandatory, Husky pre-commit hook) |
| Linting | ESLint + TypeScript strict mode |
| Naming | `PascalCase` for components, `camelCase` for hooks/functions, `kebab-case` for files |
| Styling | Tailwind CSS utility classes (no custom CSS except theme variables) |
| State | Redux Toolkit slices with RTK Query where applicable |
| Testing | Vitest + React Testing Library (unit), Playwright (E2E) |
| File organization | Co-locate related files (component.tsx, component.test.tsx, component.stories.tsx) |

### General

| Rule | Convention |
|------|-----------|
| Version control | Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`) |
| Branch strategy | `main` (stable) → `release/*` (feature branches) → PR to `release` → merged to `main` |
| Security | No secrets in code. All secrets via OS credential managers or Tauri config. |
| Dependencies | No unvetted crates for Rust. Use pnpm audit for frontend. |
| MCP protocol | Follow upstream MCP specification (protocol version 2025-01-01 or later) |

---

## File Index (Quick Reference)

| File | Purpose |
|------|---------|
| `Cargo.toml` | Rust workspace manifest |
| `pnpm-workspace.yaml` | pnpm workspace config |
| `package.json` | Root-level scripts (lint, format) |
| `src/main.rs` | Tauri app entry point |
| `src/*/Cargo.toml` | Individual crate manifests |
| `src/core/src/lib.rs` | Core orchestrator library |
| `tauri/tauri.conf.json` | Tauri app configuration |
| `tauri/permissions/*.json` | Tauri permission definitions |
| `frontend/vite.config.ts` | Vite build configuration |
| `frontend/src/main.tsx` | React entry point |
| `frontend/src/App.tsx` | Root component |
| `frontend/src/stores/index.ts` | Redux store configuration |
| `frontend/src/services/rpc-client.ts` | JSON-RPC HTTP client |
| `frontend/src/hooks/use-rpc.ts` | RPC hook with retry |
| `frontend/src/hooks/use-ai-stream.ts` | Streaming response handler |
| `skills/*/skill.json` | MCP skill metadata |
| `migrations/*.sql` | Database migration files |
| `.github/workflows/ci-lite.yml` | Fast CI pipeline |
| `.github/workflows/ci-full.yml` | Full CI pipeline |
| `.github/workflows/release.yml` | Release pipeline |
| `scripts/build-rust.sh` | Rust build script |
| `scripts/build-frontend.sh` | Frontend build script |
| `scripts/build-tauri.sh` | Tauri bundler script |
| `scripts/dev.sh` | Development environment script |
| `scripts/migrate-db.sh` | Database migration script |
| `docs/ARCHITECTURE.md` | System architecture |
| `docs/COMPONENT_DESIGN.md` | Component design |
| `docs/DATA_MODEL.md` | Data model |
| `docs/TECHNOLOGY_SELECTION.md` | Technology selection |
| `docs/SECURITY_ARCHITECTURE.md` | Security architecture |
| `docs/DEVELOPMENT_GUIDE.md` | Developer guide |
| `docs/TESTING_STRATEGY.md` | Testing strategy |
| `tests/fixtures/` | Shared test data |
| `tests/e2e/` | Playwright E2E tests |
| `icons/` | App and tray icons |