# Architecture Guide — Wiki Labs AI Copilot v1.0.0

> System architecture, components, data flow, and design decisions.

## Table of Contents

1. [System Overview](#system-overview)
2. [Technology Stack](#technology-stack)
3. [Application Architecture](#application-architecture)
4. [Component Architecture](#component-architecture)
5. [Data Architecture](#data-architecture)
6. [Security Architecture](#security-architecture)
7. [AI Engine Architecture](#ai-engine-architecture)
8. [Observation Architecture](#observation-architecture)
9. [Copilot Architecture](#copilot-architecture)
10. [Guidance Engine Architecture](#guidance-engine-architecture)
11. [Knowledge Management Architecture](#knowledge-management-architecture)
12. [Session Architecture](#session-architecture)
13. [Error Handling Architecture](#error-handling-architecture)
14. [Logging Architecture](#logging-architecture)
15. [Database Schema](#database-schema)
16. [Data Flow](#data-flow)
17. [Integration Points](#integration-points)
18. [Extensibility](#extensibility)
19. [Performance Considerations](#performance-considerations)
20. [Configuration Management](#configuration-management)

## System Overview

Wiki Labs AI Copilot is a cross-platform desktop application that assists enterprise infrastructure engineers with real-time, context-aware guidance. The application integrates AI-powered chat, knowledge management, skill packs, and desktop observation into a unified copilot experience.

### Key Design Principles

1. **Local-first:** All data stored locally in SQLite; only AI requests go to remote providers
2. **Privacy-by-default:** Observation features disabled by default; user-controlled privacy toggles
3. **Modular architecture:** Decoupled components with clear interfaces for easy maintenance and testing
4. **Security-first:** AES-256-GCM / ChaCha20 encryption, credential manager integration, secret redaction
5. **Evidence-based:** All recommendations grounded in observable evidence with confidence scoring

### Deployment Model

```
┌─────────────────────────────────────────────┐
│              User Desktop                   │
│                                             │
│  ┌─────────────────────────────────────┐    │
│  │         Wiki Labs AI Copilot        │    │
│  │                                     │    │
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐  │    │
│  │  │Chat │ │Knowl│ │Skills│ │Guide│  │    │
│  │  │Panel│ │edge │ │Panel │ │Panel│  │    │
│  │  └──┬──┘ └──┬──┘ └──┬──┘ └──┬──┘  │    │
│  │     │       │        │        │      │    │
│  │     └───────┴────────┴────────┘      │    │
│  │                │                      │    │
│  │     ┌──────────▼──────────┐           │    │
│  │     │  Copilot Engine     │           │    │
│  │     │  (Orchestration)    │           │    │
│  │     └──────────┬──────────┘           │    │
│  │                │                      │    │
│  │     ┌──────────▼──────────┐           │    │
│  │     │  SQLite Database    │           │    │
│  │     │  wikilabs.db        │           │    │
│  │     └─────────────────────┘           │    │
│  └─────────────────────────────────────┘    │
│                                             │
│  Data: %APPDATA%\com.wikilabs.copilot       │
└──────────────────┬──────────────────────────┘
                   │ HTTPS (TLS 1.2+)
┌──────────────────▼──────────────────────────┐
│         AI Provider (Remote)                │
│  (OpenAI / vLLM / Ollama)                   │
└─────────────────────────────────────────────┘
```

## Technology Stack

| Layer | Technology | Version | Purpose |
|-------|-----------|---------|---------|
| Desktop | Tauri v2 | 2.x | Cross-platform desktop shell |
| Frontend | React + TypeScript | 18+ | User interface |
| Core Language | Rust | 2021 Edition | Core engine, desktop backend |
| Database | SQLite + rusqlite | bundled | Local data persistence |
| Vector Search | SQLite VSS extension | bundled | Semantic search |
| Embedding | all-MiniLM-L6-v2 | ONNX Runtime | Local embedding generation |
| AI Runtime | OpenAI-compatible | 1.x | AI model interaction |
| Logging | tracing + tracing-subscriber | latest | Structured logging |
| CI/CD | GitHub Actions | latest | Automated testing and builds |

## Application Architecture

### Tauri Application Structure

```
src-tauri/
├── src/
│   ├── main.rs              # Entry point, app setup
│   ├── config.rs            # Settings management (8 sections)
│   ├── security.rs          # Encryption, key derivation
│   ├── logging.rs           # Structured logging with redaction
│   ├── error_handling.rs    # Error handler, crash recovery
│   ├── guidance_panel.rs    # Guidance panel Tauri commands
│   ├── knowledge_panel.rs   # Knowledge panel Tauri commands
│   ├── skill_management.rs  # Skills panel Tauri commands
│   └── copilot_panel.rs     # Copilot panel Tauri commands
├── tauri.conf.json          # Application metadata, capabilities
├── Cargo.toml               # Dependencies
└── capabilities/            # WebView capabilities
```

### Entry Point Flow (`main.rs`)

```
main()
  ├── setup_tracing()              # Initialize logging
  ├── initialize_crypto_provider() # Set up Rust crypto
  ├── setup_panic_hook()           # Custom panic handler
  ├── setup_error_handler()        # Register error handler
  ├── setup_shutdown_handlers()    # Register cleanup callbacks
  └── setup_app()
      ├── initialize_security()    # Set up encryption/crypto
      ├── initialize_logger()      # Initialize structured logging
      ├── initialize_error_handler()
      └── register_commands()
          ├── guidance_panel_commands()
          ├── knowledge_panel_commands()
          ├── copilot_panel_commands()
          └── settings_commands()
```

## Component Architecture

### Core Component Map

```
┌──────────────────────────────────────────────────────┐
│                    REACT UI                          │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌──────────┐  │
│  │   Chat  │ │Knowledge│ │  Skills │ │ Guidance │  │
│  │  Panel  │ │  Panel  │ │  Panel  │ │   Panel  │  │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬─────┘  │
│       │           │           │            │         │
└───────┼───────────┼───────────┼────────────┼────────┘
        │   Tauri IPC Commands    │
        └───────────┬─────────────┘
                    │
┌───────────────────▼───────────────────────────────┐
│              RUST BACKEND                          │
│  ┌────────────┐ ┌────────────┐ ┌───────────────┐  │
│  │ AI Engine  │ │Copilot Engine│ │ Guidance Engine│ │
│  │            │ │             │ │               │  │
│  │-Provider   │ │-Decision    │ │-Guidance Panel │  │
│  │-Conversation│ │-Recommend.  │ │-Session Ctx   │  │
│  │-Context    │ │-Lifecycle   │ │-Skill Ctx     │  │
│  │-Session    │ │-Policy      │ │-Cross-Skill Ctx│  │
│  │-Token Budget│ │-Explainability│ │               │  │
│  └─────┬──────┘ └─────┬───────┘ └───────┬───────┘  │
│        │               │                  │          │
│  ┌─────▼───────────────▼──────────────────▼───────┐  │
│  │              DATA LAYER                         │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────┐  │  │
│  │  │ Knowledge   │ │ Observation │ │ Security│  │  │
│  │  │  System     │ │   Engine    │ │  Module │  │  │
│  │  └──────┬──────┘ └──────┬──────┘ └────┬────┘  │  │
│  │         │               │              │       │  │
│  │  ┌──────▼───────────────▼──────────────▼───────┐  │
│  │  │           SQLite DATABASE                   │  │
│  │  │     wikilabs.db                             │  │
│  │  └─────────────────────────────────────────────┘  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Module | Responsibility |
|-----------|--------|----------------|
| **AI Engine** | `ai` | Provider abstraction, conversation management, context building |
| **Copilot Engine** | `copilot` | Recommendation orchestration, human approval, policy enforcement |
| **Guidance Engine** | `guidance` | Context-aware guidance panel, skill context providers |
| **Knowledge System** | `knowledge` | Document management, vector search, import/export |
| **Observation Engine** | `observation` | Screen, terminal, app context, clipboard observation |
| **Security Module** | `security` | Encryption, credential storage, log redaction |
| **Error Handler** | `error_handling` | Global error handling, crash reporting, recovery |
| **Configuration** | `config` | Settings management, profile system, persistence |

## Data Architecture

### Data Storage Hierarchy

```
%APPDATA%\com.wikilabs.copilot\
├── wikilabs.db          # SQLite database
├── settings.json        # User settings
├── credentials.enc      # Encrypted credential store
├── logs/                # Application logs
│   └── wikilabs-copilot.log
├── backups/             # Settings backups
├── crash/               # Crash reports
│   └── last_crash.json
└── .gitkeep
```

### Data Ownership

| Data Type | Storage | Encrypted | Backup |
|-----------|---------|-----------|--------|
| Workspaces | SQLite | No | Yes |
| Chat messages | SQLite | No | Yes |
| Knowledge documents | SQLite | No | Yes |
| API keys | Credential Manager / file | Yes (AES-256-GCM) | Automatic |
| Settings | JSON file | No | Automatic (on save) |
| Logs | File | Redacted | Manual |
| Crash reports | File | No | Manual |

## Security Architecture

### Encryption Layer

```
API Key (plaintext)
  │
  ├── Key Derivation
  │   ├── System Fingerprint (CPU, disk, OS)
  │   ├── Optional PIN (user-provided)
  │   └── SHA-256 → 256-bit key
  │
  ├── Encryption
  │   ├── AES-256-GCM (default)
  │   └── ChaCha20-Poly1305 (alternative)
  │
  ├── Storage Options
  │   ├── Windows Credential Manager (DPAPI) — preferred
  │   └── Encrypted file (credentials.enc) — fallback
  │
  └── Encrypted Blob → Store
```

### Secret Redaction

Automatic redaction patterns in logs and output:
- `password` → `PASSWORD_REDACTED`
- `secret` → `SECRET_REDACTED`
- `token` → `TOKEN_REDACTED`
- `api_key` → `API_KEY_REDACTED`
- `authorization` → `AUTH_REDACTED`
- Any field matching `.*password|secret|token|api_key.*` pattern

### Privacy Controls

| Feature | Default | Toggled By |
|---------|---------|------------|
| Screen observation | Disabled | User setting |
| Clipboard observation | Disabled | User setting |
| OCR | Enabled | User setting |
| Diagnostics | Enabled | User setting |
| Telemetry | Disabled | User setting |
| Privacy Mode | Off | One-click toggle |

## AI Engine Architecture

### Provider Abstraction

```
┌─────────────────────────────────────────────────┐
│                 Provider Trait                  │
│                                                 │
│  fn chat(request: AiRequest) -> ChatResponse    │
│  fn stream_chat(request: AiRequest) -> Stream   │
│  fn health() -> Result<()>                      │
│  fn supported_features() -> FeatureFlags        │
│                                                 │
└─────────────────────┬───────────────────────────┘
                      │ implements
        ┌─────────────┼─────────────────┐
        │             │                 │
   ┌────▼────┐  ┌────▼────┐    ┌──────▼──────┐
   │  OpenAI  │  │  vLLM   │    │   Ollama    │
   │ Compatible │ Compatible │  Compatible   │
   └───────────┘ └──────────┘    └─────────────┘
```

### Conversation Manager

```
Conversation (UUID)
├── Title (auto-generated)
├── Created At (ISO 8601)
├── Updated At (ISO 8601)
├── Tags (vector of strings)
├── Summarized (bool)
├── Messages
│   ├── Message (UUID)
│   │   ├── Role: user | assistant | system
│   │   ├── Content: String
│   │   ├── Created At: ISO 8601
│   │   └── Tool Calls: Optional JSON
│   └── ... (ordered by created_at)
└── Current Conversation (UUID)
```

### Context Manager

```
Context Builder Flow:
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Manual     │     │  Technology │     │  Activity   │
│  Context    │ ──▶ │  Context    │ ──▶ │  Context     │
│  (tags)     │     │  (stack)    │     │  (tracking) │
└─────────────┘     └─────────────┘     └─────────────┘
                                              │
                                              ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Current    │     │  Priority   │     │  Final      │
│  Activity   │ ──▶ │  Sorting    │ ──▶ │  Context     │
│  (task)     │     │  (H/N/L)    │     │              │
└─────────────┘     └─────────────┘     └─────────────┘
```

### Session Management

```
Session State Machine:

  Active ──────────────▶ Suspended ──────▶ Ended
    │                       │                 │
    │ idle_timeout          │ user_request    │
    │                       ▼                 │
    │                  Paused ────────────────┘
    │                       │
    │                       │ resume
    │                       ▼
    └──────────────────▶ Active
```

### Token Budget Management

Three policies for managing token usage:

| Policy | Behavior | Use Case |
|--------|----------|----------|
| **Strict** | Enforce exact budget, trim aggressively | Cost-sensitive deployments |
| **With Buffer** | Allow 10% over budget | Balanced cost/performance |
| **Aggressive** | Allow unlimited, recommend later | Maximum context retention |

## Observation Architecture

### Tiered Observation System

```
┌──────────────────────────────────────────────────┐
│              Observation Engine                   │
│                                                  │
│  Tier 1: Screen                                   │
│  ┌─────────────────────────────────────────────┐  │
│  │ - Screenshot capture                        │  │
│  │ - UI element detection                      │  │
│  │ - Window identification                     │  │
│  └─────────────────────────────────────────────┘  │
│                                                  │
│  Tier 2: Terminal / Shell                         │
│  ┌─────────────────────────────────────────────┐  │
│  │ - Command history                           │  │
│  │ - Terminal output capture                   │  │
│  │ - Process monitoring                        │  │
│  └─────────────────────────────────────────────┘  │
│                                                  │
│  Tier 3: Application Context                      │
│  ┌─────────────────────────────────────────────┐  │
│  │ - Active application detection              │  │
│  │ - Window title / URL analysis               │  │
│  │ - Configuration file detection              │  │
│  └─────────────────────────────────────────────┘  │
│                                                  │
│  Tier 4: Clipboard                                │
│  ┌─────────────────────────────────────────────┐  │
│  │ - Clipboard content capture                 │  │
│  │ - Clipboard change monitoring               │  │
│  └─────────────────────────────────────────────┘  │
│                                                  │
│  OCR Layer (Optional)                             │
│  ┌─────────────────────────────────────────────┐  │
│  │ - OCR on screen captures                    │  │
│  │ - Text extraction from images               │  │
│  └─────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

### Technology Recognition

Detection sources with confidence scoring:

| Source | Detection Method | Confidence |
|--------|-----------------|------------|
| Browser URL | URL pattern matching | High (0.85-0.95) |
| Terminal | Command detection | Medium-High (0.70-0.90) |
| Window Title | Title pattern matching | Medium (0.60-0.80) |
| Configuration Files | File path matching | High (0.80-0.95) |

### Intent Recognition

```
Input (observation data)
  │
  ├── Technology Classifier
  │   ├── Pre-trained ML model
  │   └── Confidence output
  │
  ├── Rule-based Classifier
  │   ├── Technology-aware rules
  │   └── Confidence output
  │
  ├── Confidence Engine
  │   ├── Auto-confirm (High confidence)
  │   └── Ask user (Low confidence)
  │
  └── Human Feedback
      ├── Corrections tracked
      └── Pattern learned for future
```

## Copilot Architecture

### Copilot Engine

The copilot is the central orchestrator of the engineering assistance system:

```
┌──────────────────────────────────────────────────────┐
│                 Copilot Engine                        │
│                                                     │
│  ┌──────────┐ ┌──────────┐ ┌────────────────────┐   │
│  │ Decision │ │ Recommend│ │    Policy          │   │
│  │   Engine │ │  Engine  │ │    Engine          │   │
│  │          │ │          │ │                    │   │
│  │ Evaluates│ │ Templates│ │ 5 Operating Modes: │   │
│  │ if rec   │ │ generate │ │  - Minimal         │   │
│  │ should   │ │ advice   │ │  - Balanced        │   │
│  │ display  │ │ from     │ │  - Teaching        │   │
│  │ based on │ │ context  │ │  - Expert          │   │
│  │ 9 rules  │ │ & skills │ │  - Silent          │   │
│  └────┬─────┘ └────┬─────┘ │  - Expert          │   │
│       │             │       └────────────────────┘   │
│  ┌────▼─────────────▼────────────────────────────┐   │
│  │           Lifecycle Manager                    │   │
│  │                                              │   │
│  │  Candidate → Ready → Displayed →             │   │
│  │    Accepted → Completed                       │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
│  ┌────────────────────────────────────────────┐     │
│  │        Human Approval System                │     │
│  │                                            │     │
│  │  Pending → Approved → Completed             │     │
│  │       → Denied                             │     │
│  │       → AutoApproved (after timeout)        │     │
│  └────────────────────────────────────────────┘     │
│                                                     │
│  ┌────────────────────────────────────────────┐     │
│  │        Explainability Engine                │     │
│  │                                            │     │
│  │  Reason Trees + Evidence Mapping            │     │
│  │  → Human-readable explanations              │     │
│  └────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────┘
```

### Decision Engine

Nine evaluation rules in priority order:

1. **Session active** — Is the engineering session active?
2. **Context relevant** — Does context match the recommendation?
3. **User preference** — Respects user's recommendation preferences?
4. **Policy allowed** — Allowed by current operating mode?
5. **Not redundant** — Not a duplicate of recent recommendations?
6. **Priority sufficient** — Meets priority threshold?
7. **No user override** — User hasn't dismissed this type?
8. **Timing appropriate** — Right time for this recommendation?
9. **Technical match** — Matches detected technology?

### Policy Engine — Operating Modes

| Mode | Recommendations | Detail Level | Use Case |
|------|----------------|--------------|----------|
| **Minimal** | Only critical | Summary | Experienced engineers |
| **Balanced** | Critical + important | Detailed | General use |
| **Teaching** | All with explanations | Very detailed | Learning mode |
| **Expert** | All, no explanations | Technical | Expert engineers |
| **Silent** | None (observation only) | N/A | Passive monitoring |

### Session Memory

Personalization based on interaction patterns:
- Acceptance rate tracking
- Dismissal reasons
- User corrections
- Preferred detail level
- Technology preferences

## Guidance Engine Architecture

### Context Providers

Three context providers feed into guidance recommendations:

```
┌─────────────────────────────────────────────────────┐
│              Guidance Engine                         │
│                                                     │
│  ┌─────────────────────┐                            │
│  │   Session Context   │                            │
│  │                     │                            │
│  │  Current task       │                            │
│  │  Session duration   │                            │
│  │  Prior decisions    │                            │
│  │  Pending approvals  │                            │
│  └─────────────────────┘                            │
│         │                           │                │
│  ┌─────▼──────────┐       ┌────────▼──────────┐     │
│  │  Skill Context │       │ Cross-Skill Ctx   │     │
│  │                │       │                   │     │
│  │ Active tech    │       │ Multi-skill       │     │
│  │ Available cmds │       │ Interactions      │     │
│  │ Workflow state │       │ Shared state      │     │
│  └────────────────┘       │ Cross-cutting     │     │
│                           │ concerns          │     │
│                           └───────────────────┘     │
│                                                     │
│              Guidance Panel                         │
│  ┌───────────────────────────────────────────────┐  │
│  │  Recommendations │ Warnings │ Tips │ Explains │  │
│  │  Priority: High │ Med │ Low│             │     │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Guidance Item Types

| Type | Example | Priority |
|------|---------|----------|
| Recommendation | "Consider increasing buffer pool size" | High |
| Warning | "Replication lag exceeds 30 seconds" | Medium |
| Suggestion | "Check slow query log for long-running queries" | Medium |
| Tip | "Use EXPLAIN ANALYZE to verify plan" | Low |
| Explanation | "This error indicates a lock timeout" | Low |

## Knowledge Management Architecture

### Hybrid Search Architecture

```
┌──────────────────────────────────────────────────────┐
│                 Knowledge System                     │
│                                                      │
│  Input: .wkl Knowledge Archive                       │
│         (.txt, .md, .json files)                     │
│                                                      │
│  ┌─────────────────────────────────────────────────┐ │
│  │  Document Processing Pipeline                    │ │
│  │                                                  │ │
│  │  1. Import → Parse → Chunk                       │ │
│  │  2. Embed → Vectorize → Index                    │ │
│  │  3. FTS5 → Tokenize → Index                     │ │
│  │  4. Quality → Score → Store                     │ │
│  └─────────────────────────────────────────────────┘ │
│                                                      │
│  ┌─────────────────────────────────────────────────┐ │
│  │  Search Architecture                             │ │
│  │                                                  │ │
│  │  Vector Search (VSS)        Keyword Search (FTS5)│ │
│  │  ──────────────────        ─────────────────────│ │
│  │  384-dim embeddings         Full-text index      │ │
│  │  Semantic matching          Exact term match      │ │
│  │  Top-N similarity          Relevance scoring     │ │
│  └─────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

### Knowledge Data Model

```
workspaces (1) ──< (N) knowledge_documents (1) ──< (N) knowledge_chunks
```

| Table | Key Columns |
|-------|------------|
| `knowledge_documents` | id, title, source, workspace_id, author, created_at, updated_at |
| `knowledge_chunks` | id, document_id, content, embedding (VECTOR 384), vector_id |

## Session Architecture

### AI Session Lifecycle

```
  ┌───────────┐
  │   Active   │ ◀── Session created
  │            │ ──▶ Idle timeout ──┐
  └───────────┘                   │
         │                        ▼
         │                  ┌───────────┐
         │ resume             │ Suspended │
         │                    └───────────┘
         │                        │
         │                    ──▶ User request ──▶
         │                        │                  │
         │                        ▼                  │
         │                  ┌───────────┐            │
         └──────────────────│   Paused   │            │
                            └───────────┘            │
                                 │                   │
                                 │ User closes       │
                                 ▼                   │
                            ┌───────────┐            │
                            │    Ended   │ ◀─────────┘
                            └───────────┘
```

## Error Handling Architecture

### Error Severity Levels

```
  Warning ──▶ Degraded ──▶ Error ──▶ Fatal
    │           │             │          │
    │           │             │          │
    │           │           Retry      Shutdown
    │           │           Fallback
    │           │           UserPrompt
    │           ▼
    │         Log + Continue
    ▼
  Log + Continue
```

### Error Handler Chain

```
┌─────────────────────────────────────────────┐
│           Error Handler                      │
│                                              │
│  Error Occurs                                 │
│    │                                          │
│    ├── Severity Assessment (Warning/          │
│    │  Degraded/Error/Fatal)                   │
│    │                                          │
│    ├── Recovery Strategy Selection:           │
│    │  ├── Retry (with backoff)               │
│    │  ├── Fallback (alternative path)        │
│    │  ├── UserPrompt (notify user)           │
│    │  └── Shutdown (graceful exit)           │
│    │                                          │
│    ├── Action Execution                       │
│    │                                          │
│    └── Crash Report (for Fatal/Error)         │
│       └── saved to crash/last_crash.json      │
└─────────────────────────────────────────────┘
```

## Logging Architecture

### Structured Logging System

```
┌─────────────────────────────────────────────┐
│            Logging System                    │
│                                              │
│  trace → debug → info → warn → error         │
│  ──────── ▶ ──────── ▶ ──────── ▶ ──── ▶ ────
│                                              │
│  Log Targets:                                │
│  ├── Console (stderr)                         │
│  └── File: logs/wikilabs-copilot.log          │
│                                              │
│  Log Rotation:                               │
│  ├── Daily rotation                           │
│  ├── Max file size: 10 MB                     │
│  ├── Max files: 3                             │
│  └── Structured JSON format                   │
│                                              │
│  Secret Redaction:                           │
│  ├── password → PASSWORD_REDACTED             │
│  ├── secret → SECRET_REDACTED                 │
│  ├── token → TOKEN_REDACTED                   │
│  ├── api_key → API_KEY_REDACTED               │
│  └── authorization → AUTH_REDACTED            │
└─────────────────────────────────────────────┘
```

## Database Schema

### Complete Schema

```sql
-- Workspaces
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,       -- UUID
    name TEXT NOT NULL,        -- Workspace name
    customer_name TEXT,        -- Optional customer name
    technology_stack TEXT,     -- JSON array of tech names
    created_at TEXT,           -- ISO 8601
    updated_at TEXT            -- ISO 8601
);

-- Chat Messages
CREATE TABLE chat_messages (
    id TEXT PRIMARY KEY,       -- UUID
    workspace_id TEXT,         -- FK to workspaces.id
    role TEXT NOT NULL,        -- user, assistant, system
    content TEXT,              -- Message content
    created_at TEXT            -- ISO 8601
);

-- Knowledge Documents
CREATE TABLE knowledge_documents (
    id TEXT PRIMARY KEY,       -- UUID
    title TEXT,                -- Document title
    source TEXT,               -- File path or URL
    workspace_id TEXT,         -- FK to workspaces.id
    author TEXT,               -- Document author
    created_at TEXT,           -- ISO 8601
    updated_at TEXT            -- ISO 8601
);

-- Knowledge Chunks (VSS indexed)
CREATE TABLE knowledge_chunks (
    id TEXT PRIMARY KEY,       -- UUID
    document_id TEXT,          -- FK to knowledge_documents.id
    content TEXT,              -- Chunk text
    embedding VECTOR(384),     -- Embedding vector
    vector_id TEXT             -- VSS index identifier
);

-- Audit Log
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,       -- UUID
    timestamp TEXT,            -- ISO 8601
    action TEXT,               -- Action description
    actor TEXT,                -- User/system identifier
    hash TEXT,                 -- SHA-256 of previous entry
    signature TEXT             -- Ed25519 signature (optional)
);
```

### Indexes

| Table | Index | Purpose |
|-------|-------|---------|
| `chat_messages` | `workspace_id + created_at` | Efficient per-workspace queries |
| `knowledge_documents` | `workspace_id` | Efficient per-workspace knowledge |
| `audit_log` | `timestamp` | Time-range queries |

## Data Flow

### Primary Data Flow (AI Chat)

```
User types message
       │
       ▼
┌─────────────┐
│ Conversation  │──▶ Store in chat_messages table
│   Manager     │
└───────┬─────┘
        │
        ▼
┌─────────────┐
│   Context     │──▶ Collect workspace context
│   Manager     │──▶ Collect technology context
│               │──▶ Collect activity context
└───────┬─────┘
        │
        ▼
┌─────────────┐
│ Token Budget  │──▶ Check budget policy
│   Manager     │──▶ Trim if needed
└───────┬─────┘
        │
        ▼
┌─────────────┐
│   AI          │──▶ Send request to provider
│   Provider    │──▶ Receive response (streaming)
│               │──▶ Handle tool calls
└───────┬─────┘
        │
        ▼
┌─────────────┐
│   Copilot     │──▶ Evaluate recommendation
│    Engine     │──▶ Decide visibility
└───────┬─────┘
        │
        ▼
┌─────────────┐
│   Guidance    │──▶ Display guidance panel
│    Panel      │
└───────┬─────┘
        │
        ▼
┌─────────────┐
│   Store       │──▶ Store assistant response
│   Response    │──▶ Update chat_messages
└─────────────┘
```

### Knowledge Import Data Flow

```
Import .wkl archive
       │
       ▼
┌─────────────┐
│   Parse &     │──▶ Extract .txt/.md/.json files
│   Validate    │──▶ Validate document structure
└───────┬─────┘
        │
        ▼
┌─────────────┐
│   Chunk &     │──▶ Split into chunks
│   Embed       │──▶ Generate embeddings (ONNX)
│               │──▶ 384-dim vectors
└───────┬─────┘
        │
        ▼
┌─────────────┐
│   Index &     │──▶ Store in SQLite
│   Store       │──▶ VSS index (vector)
│               │──▶ FTS5 index (keyword)
└─────────────┘
```

## Integration Points

### External APIs

| Integration | Direction | Protocol | Purpose |
|------------|-----------|----------|---------|
| AI Provider | Outbound | HTTPS/TLS 1.2+ | Chat, streaming, embeddings |
| Auto-update | Outbound | HTTPS | Download installer updates |

### System Integrations

| Integration | Platform | Purpose |
|------------|----------|---------|
| Windows Credential Manager | Windows | Secure credential storage |
| WebView2 Runtime | Windows | HTML rendering |
| .NET Desktop Runtime 8.0 | Windows | Desktop support |
| SQLite VSS | Internal | Vector search |
| ONNX Runtime | Internal | Local embeddings |

## Extensibility

### Skill Pack SDK

The Skill Development Kit enables external development of technology-specific knowledge:

1. **Create skill pack** with manifest, technology definitions, and detection rules
2. **Package** as `.wkl` archive
3. **Install** by placing in `src/skills/` directory
4. **Auto-detected** by Skill Discovery Engine at runtime

### Custom Providers

New AI providers can be implemented by:
1. Implementing the `AiProvider` trait
2. Registering the provider with the AI runtime
3. Adding provider configuration to settings

### Plugin Architecture

Future versions may support plugins via:
- Tauri plugins for native functionality
- Web-based plugins through the frontend
- Custom skill packs as the primary extension mechanism

## Performance Considerations

### Database Performance

- SQLite provides excellent performance for the expected data volume
- VSS queries return in milliseconds for knowledge bases under 10,000 chunks
- FTS5 full-text search is optimized with trigram indexing

### Memory Footprint

| Component | Baseline | With Feature |
|-----------|----------|-------------|
| Application shell | ~50 MB | — |
| + AI conversation | — | +10-50 MB |
| + Observation engine | — | +20-50 MB |
| + Knowledge base loaded | — | +100-200 MB |
| + Multiple workspaces | — | +5-10 MB per WS |

### Optimization Recommendations

1. **Archive old conversations** to reduce database size
2. **Split large knowledge bases** across workspaces
3. **Disable unused observation features** to reduce memory
4. **Use VACUUM** periodically for database optimization
5. **Configure log rotation** appropriately for production use

## Configuration Management

### Settings Profile System

The application supports named configuration profiles:

```
Settings File: settings.json
├── current_profile: "default"
└── profiles:
    ├── "default": { settings... }
    ├── "work": { settings... }
    └── "home": { settings... }
```

Profiles provide:
- Independent AI provider configurations
- Different observation preferences
- Separate settings per context
- Import/export as JSON for backup and migration

### Configuration Sections

1. **AI Provider** — Provider name, endpoint, API key, model
2. **UI** — Theme, font size, zoom, language, shortcuts
3. **Privacy** — Screen/OCR/clipboard/diagnostics/telemetry
4. **Security** — Encryption, credential manager, auto-lock
5. **Update** — Auto-check, channel, dialog behavior
6. **Logging** — Level, file logging, rotation
7. **Window** — Dimensions, position, maximized state
8. **Profile** — Named profiles with independent settings

---

*For security configuration, see [Security Guide](SECURITY_GUIDE.md).*
*For development workflow, see [Developer Guide](DEVELOPER_GUIDE.md).*
*For architecture decision records, see [adr/ARCHITECTURE_DECISIONS.md](adr/ARCHITECTURE_DECISIONS.md).*