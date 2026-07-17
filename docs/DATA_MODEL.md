---
description: "Core data entities, relationships, storage model, database schema, caching, knowledge/workspace/skill/audit data models, lifecycle, and encryption for Wiki Labs AI Copilot."
icon: database
---

# Wiki Labs AI Copilot — Data Model

## Design Principles

1. **Local-First**: All data stored locally on the engineer's laptop. No cloud storage by default.
2. **SQLite-Centric**: Single embedded SQLite database for all relational data — zero config, portable, ACID transactions.
3. **Vector-Augmented**: ChromaDB for knowledge embeddings; SQLite FTS5 for full-text search.
4. **Workspace-Isolated**: All data scoped to a workspace (customer context). Cross-workspace queries are supported for personal notes and settings.
5. **Audit-Logged**: All AI interactions and system events are logged immutably.

---

## Core Data Entities and Relationships

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│ Workspace│1   *│ Workspace│1   *│ Workspace│1   *│
│          │─────│ Stack    │     │ Session  │     │ Note │
│ - id     │     │ - id     │     │ - id     │     │ - id │
│ - name   │     │ - tech   │     │ - title  │     │ - id │
│ - created│     │ - skill  │     │ - started│     │ - content │
│ - updated│     │ - active │     │ - ended  │     │ - ws_id │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
       │                                      │
       │1                                   1 │ *
       ▼           ┌──────────┐              ▼
┌──────────┐       │  Skill   │       ┌──────────┐
│  User    │1   *  │ Config   │       │ Recommendation│
│ Settings │───────│ - id     │       │ - id       │
│ - id     │       │ - skill  │       │ - session  │
│ - ai     │       │ - config │       │ - content  │
│   provider│      │ - enabled│       │ - created  │
│ - obs    │       └──────────┘       └──────────┘
│   settings│                             │
│ - privacy│                    ┌─────────┼──────────┐
└──────────┘                    ▼         ▼          ▼
                         ┌──────────┐ ┌──────────┐ ┌──────────┐
                         │Chat Msg  │ │Audit Log │ │System Log│
                         │- id      │ │- id      │ │- id      │
                         │- session │ │- level   │ │- level   │
                         │- role    │ │- component│ │- message │
                         │- content │ │- details │ │- created │
                         │- created │ └──────────┘ └──────────┘
                         └──────────┘
```

---

## Data Storage Model

### SQLite Database

Single SQLite database file: `~/.local/share/wikilabs/wikilabs.db`

**Why SQLite**: Zero-config, embedded, ACID transactions, single-file portability, FTS5 support, well-tested in Rust via `rusqlite` (used by OpenHuman).

**Table groups**:

| Group | Tables | Purpose |
|-------|--------|---------|
| User | `users`, `user_settings` | User preferences and settings |
| Workspace | `workspaces`, `workspaces_stacks`, `workspaces_env` | Customer environments |
| Session | `sessions`, `chat_messages`, `recommendations` | AI conversations and suggestions |
| Knowledge | `knowledge_docs`, `knowledge_chunks`, `knowledge_sources` | Imported knowledge base |
| Skill | `skills`, `skill_configs`, `skill_tools` | Installed skills and tools |
| Workflow | `workflows`, `workflow_steps`, `workflow_runs` | Engineering workflow instances |
| Audit | `audit_logs`, `system_logs` | Immutable event logs |
| Memory | `short_term_memory`, `long_term_memory` | Context memory entries |

### Vector Store

ChromaDB embedded instance: `~/.local/share/wikilabs/vectors/`

**Purpose**: Store text embeddings for knowledge chunks. Supports semantic search.

**Why ChromaDB**: Embedded (no server), lightweight, supports persistence, Python/Rust bindings available, used by many AI applications.

**Collections**:
- `knowledge_chunks`: Vector embeddings of knowledge document chunks

### File System

Local file system for binary and large text files: `~/.local/share/wikilabs/files/`

**Subdirectories**:
```
~/.local/share/wikilabs/
├── wikilabs.db              # SQLite database
├── vectors/                 # ChromaDB data
│   └── chroma.sqlite3
├── files/
│   ├── knowledge/           # Imported knowledge source files
│   ├── screenshots/         # Optional saved screenshots (encrypted)
│   └── exports/             # Exported reports and summaries
├── config/
│   └── settings.json        # Application settings (JSON fallback)
└── logs/
    ├── audit.log            # Audit log (append-only)
    └── system.log           # System log
```

---

## Database Schema Design

### User Settings

```sql
CREATE TABLE users (
    id          TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE user_settings (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    user_id         TEXT NOT NULL REFERENCES users(id),
    ai_provider     TEXT NOT NULL DEFAULT 'openai',
    ai_model        TEXT NOT NULL DEFAULT 'gpt-4o',
    ai_api_key      TEXT NOT NULL,              -- encrypted (see Encryption section)
    ai_base_url     TEXT,
    ai_max_tokens   INTEGER NOT NULL DEFAULT 4096,
    ai_temperature  REAL NOT NULL DEFAULT 0.7,
    obs_screen_enabled   BOOLEAN NOT NULL DEFAULT 0,
    obs_screen_interval  REAL NOT NULL DEFAULT 2.0,
    obs_terminal_enabled BOOLEAN NOT NULL DEFAULT 0,
    obs_clipboard_enabled BOOLEAN NOT NULL DEFAULT 0,
    obs_clipboard_filter_creds BOOLEAN NOT NULL DEFAULT 1,
    privacy_screenshot_res TEXT NOT NULL DEFAULT '1920x1080',
    privacy_screen_locked BOOLEAN NOT NULL DEFAULT 0,
    privacy_obs_disabled BOOLEAN NOT NULL DEFAULT 0,
    theme TEXT NOT NULL DEFAULT 'dark',
    language TEXT NOT NULL DEFAULT 'en',
    auto_save_conversations BOOLEAN NOT NULL DEFAULT 1,
    audit_log_enabled BOOLEAN NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id)
);
```

### Workspace

```sql
CREATE TABLE workspaces (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    name            TEXT NOT NULL,
    description     TEXT,
    customer_name   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    last_active_at  TEXT NOT NULL DEFAULT (datetime('now')),
    is_active       BOOLEAN NOT NULL DEFAULT 0,
    color           TEXT DEFAULT '#6366f1'
);

CREATE INDEX idx_workspaces_is_active ON workspaces(is_active);
CREATE INDEX idx_workspaces_last_active ON workspaces(last_active_at);

CREATE TABLE workspaces_stacks (
    id          TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    technology   TEXT NOT NULL,    -- 'OpenShift', 'VMware', 'Linux', etc.
    version      TEXT,
    active       BOOLEAN NOT NULL DEFAULT 1,
    added_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE workspaces_env (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    environment     TEXT NOT NULL,  -- 'production', 'staging', 'development'
    region          TEXT,
    notes           TEXT,
    metadata        TEXT,         -- JSON: arbitrary key-value pairs
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Sessions and Chat

```sql
CREATE TABLE sessions (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id),
    title           TEXT NOT NULL DEFAULT 'New Conversation',
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at        TEXT,
    intent_at_start TEXT,         -- JSON: initial intent recognition result
    intent_at_end   TEXT,         -- JSON: final intent recognition result
    workflow_id     TEXT,         -- references workflows.id if a workflow was active
    metadata        TEXT          -- JSON: arbitrary session metadata
);

CREATE INDEX idx_sessions_workspace ON sessions(workspace_id);
CREATE INDEX idx_sessions_started ON sessions(started_at);

CREATE TABLE chat_messages (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role            TEXT NOT NULL,       -- 'user', 'assistant', 'system', 'tool'
    content         TEXT NOT NULL,       -- message content (markdown supported)
    attachments     TEXT,               -- JSON: array of {type, url, text}
    references      TEXT,               -- JSON: array of {doc_id, title, snippet}
    suggestions     TEXT,               -- JSON: array of suggestion objects
    error           TEXT,               -- JSON: error details if any
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    sequence        INTEGER NOT NULL    -- ordering within session
);

CREATE INDEX idx_messages_session ON chat_messages(session_id);
CREATE INDEX idx_messages_sequence ON chat_messages(session_id, sequence);
CREATE INDEX idx_messages_created ON chat_messages(created_at);

CREATE TABLE recommendations (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    type            TEXT NOT NULL,       -- 'command', 'investigation', 'fix', 'note'
    content         TEXT NOT NULL,
    confidence      REAL,               -- 0.0 to 1.0
    source          TEXT,               -- 'skill', 'knowledge', 'workflow', 'ai'
    metadata        TEXT,               -- JSON: skill_id, tool_name, etc.
    accepted        BOOLEAN NOT NULL DEFAULT 0,
    dismissed       BOOLEAN NOT NULL DEFAULT 0,
    dismissed_at    TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Knowledge

```sql
CREATE TABLE knowledge_sources (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    type            TEXT NOT NULL,      -- 'file', 'directory', 'url', 'import'
    source_path     TEXT,               -- original file path or URL
    status          TEXT NOT NULL DEFAULT 'pending',  -- pending, indexing, ready, error
    error_message   TEXT,
    doc_count       INTEGER NOT NULL DEFAULT 0,
    chunk_count     INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_knowledge_sources_workspace ON knowledge_sources(workspace_id);
CREATE INDEX idx_knowledge_sources_status ON knowledge_sources(status);

CREATE TABLE knowledge_docs (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    source_id       TEXT NOT NULL REFERENCES knowledge_sources(id) ON DELETE CASCADE,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    subtitle        TEXT,
    content         TEXT,              -- raw text content (for FTS5 indexing)
    file_path       TEXT,              -- path to original file if local
    mime_type       TEXT,             -- 'text/markdown', 'application/pdf', etc.
    chunk_count     INTEGER NOT NULL DEFAULT 0,
    embedding_status TEXT NOT NULL DEFAULT 'pending',  -- pending, indexed, error
    error_message   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_id, file_path)
);

CREATE INDEX idx_knowledge_docs_source ON knowledge_docs(source_id);
CREATE INDEX idx_knowledge_docs_workspace ON knowledge_docs(workspace_id);

CREATE VIRTUAL TABLE knowledge_docs_fts USING fts5(
    title,
    subtitle,
    content,
    content=knowledge_docs,
    content_rowid=id
);

CREATE TABLE knowledge_chunks (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    doc_id          TEXT NOT NULL REFERENCES knowledge_docs(id) ON DELETE CASCADE,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    content         TEXT NOT NULL,
    token_count     INTEGER NOT NULL,
    start_line      INTEGER,
    end_line        INTEGER,
    embedding_id    TEXT,             -- ChromaDB document ID
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_chunks_doc ON knowledge_chunks(doc_id);
CREATE INDEX idx_chunks_workspace ON knowledge_chunks(workspace_id);
```

### Skills

```sql
CREATE TABLE skills (
    id              TEXT PRIMARY KEY,     -- skill ID (e.g., 'openshift', 'linux')
    name            TEXT NOT NULL,
    version         TEXT NOT NULL DEFAULT '1.0.0',
    description     TEXT,
    icon            TEXT,               -- emoji or icon name
    category        TEXT NOT NULL DEFAULT 'infrastructure',
    enabled         BOOLEAN NOT NULL DEFAULT 1,
    installed       BOOLEAN NOT NULL DEFAULT 1,
    installed_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    auto_start      BOOLEAN NOT NULL DEFAULT 0  -- spawn on launch vs. lazy load
);

CREATE TABLE skill_configs (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    skill_id        TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    key             TEXT NOT NULL,
    value           TEXT,               -- stored as JSON or raw value
    encrypted       BOOLEAN NOT NULL DEFAULT 0,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(skill_id, key)
);

CREATE TABLE skill_tools (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    skill_id        TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    tool_name       TEXT NOT NULL,
    description     TEXT NOT NULL,
    input_schema    TEXT NOT NULL,      -- JSON Schema definition
    output_schema   TEXT,               -- JSON Schema definition
    requires_auth   BOOLEAN NOT NULL DEFAULT 0,
    is_public       BOOLEAN NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(skill_id, tool_name)
);

CREATE INDEX idx_skills_enabled ON skills(enabled);
```

### Workflows

```sql
CREATE TABLE workflows (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    skill_id        TEXT NOT NULL REFERENCES skills(id),
    name            TEXT NOT NULL,
    description     TEXT,
    technology      TEXT NOT NULL,
    steps_schema    TEXT NOT NULL,      -- JSON: workflow step definitions
    commands        TEXT,               -- JSON: example commands
    checklists      TEXT,               -- JSON: verification checklists
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE workflow_runs (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    workflow_id     TEXT NOT NULL REFERENCES workflows(id),
    status          TEXT NOT NULL DEFAULT 'running',  -- running, paused, completed, aborted
    current_step    INTEGER NOT NULL DEFAULT 0,
    completed_steps TEXT,               -- JSON: array of completed step IDs
    result          TEXT,               -- JSON: final result / summary
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id)
);

CREATE INDEX idx_workflow_runs_session ON workflow_runs(session_id);
CREATE INDEX idx_workflow_runs_status ON workflow_runs(status);
```

### Audit and System Logs

```sql
CREATE TABLE audit_logs (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    timestamp       TEXT NOT NULL DEFAULT (datetime('now')),
    level           TEXT NOT NULL DEFAULT 'info',  -- info, warn, error, debug
    component       TEXT NOT NULL,      -- which component generated this log
    event           TEXT NOT NULL,      -- event type (chat_sent, suggestion_shown, skill_enabled, etc.)
    workspace_id    TEXT,               -- associated workspace (nullable)
    session_id      TEXT,               -- associated session (nullable)
    user_id         TEXT,               -- user action (nullable)
    details         TEXT,               -- JSON: structured event details
    user_action     TEXT                -- user's action that triggered this (nullable)
);

CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_component ON audit_logs(component);
CREATE INDEX idx_audit_workspace ON audit_logs(workspace_id);

CREATE TABLE system_logs (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    timestamp       TEXT NOT NULL DEFAULT (datetime('now')),
    level           TEXT NOT NULL DEFAULT 'info',
    component       TEXT NOT NULL,
    message         TEXT NOT NULL,
    stack_trace     TEXT,
    metadata        TEXT                -- JSON: additional structured data
);

CREATE INDEX idx_system_timestamp ON system_logs(timestamp);
CREATE INDEX idx_system_component ON system_logs(component);
```

### Memory System

Adapted from OpenHuman's short-term, long-term, and subconscious memory model:

```sql
CREATE TABLE short_term_memory (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id),
    content         TEXT NOT NULL,
    memory_type     TEXT NOT NULL,      -- 'observation', 'intent', 'recommendation', 'fact'
    importance      REAL NOT NULL DEFAULT 0.5,  -- 0.0 to 1.0
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at      TEXT                -- NULL = permanent
);

CREATE INDEX idx_short_memory_session ON short_term_memory(session_id);
CREATE INDEX idx_short_memory_type ON short_term_memory(memory_type);

CREATE TABLE long_term_memory (
    id              TEXT PRIMARY KEY DEFAULT (lower(replace(uuid(), '-', ''))),
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    content         TEXT NOT NULL,
    memory_type     TEXT NOT NULL,
    tags            TEXT,               -- JSON: array of tags
    importance      REAL NOT NULL DEFAULT 0.5,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_long_memory_workspace ON long_term_memory(workspace_id);
CREATE INDEX idx_long_memory_type ON long_term_memory(memory_type);
```

---

## Caching Strategy

| Cache | Technology | Purpose | TTL |
|-------|-----------|---------|-----|
| AI Provider response | In-memory HashMap | Avoid duplicate AI calls for identical prompts | 1 hour |
| Intent result | In-memory LRU | Avoid re-analyzing unchanged context | 30 seconds |
| Skill tool catalog | In-memory | Cache tool definitions per MCP server | Until skill restart |
| Knowledge search results | In-memory LRU | Cache recent knowledge search results | 5 minutes |
| Workspace context | In-memory | Cached workspace metadata | Until workspace change |
| Settings | In-memory + SQLite | User settings cache | Until settings change |

**Cache Invalidation**:
- Settings change → Invalidate settings cache
- Workspace switch → Invalidate workspace context, short-term memory
- Skill restart → Invalidate tool catalog cache
- Knowledge refresh → Invalidate knowledge search cache
- Explicit user action → Invalidate all caches (e.g., manual AI retry)

---

## Knowledge Base Data Model — Details

### Document Processing Pipeline

```
File / URL
    │
    ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Parser      │────►│  Chunker     │────►│  Embedder    │────►│  Indexer     │
│  (format     │     │  (512 tok    │     │  (AI provider│     │  (SQLite +   │
│   specific)  │     │  chunks, 64   │     │   vector    │     │   ChromaDB)  │
│              │     │  tok overlap) │     │   model)    │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
    │                      │                      │                      │
    ▼                      ▼                      ▼                      ▼
knowledge_docs    knowledge_chunks          embedding              FTS5 +
table             table (vector_id)          stored in            ChromaDB
                                        ChromaDB collection
```

### Chunking Parameters
- **Chunk size**: 512 tokens (configurable per document type)
- **Overlap**: 64 tokens (12.5% overlap for context continuity)
- **Headers preserved**: Markdown headers and section boundaries maintained for context
- **Max chunks per doc**: 10,000 (documents larger than this are split into sub-documents)

---

## Workspace Data Model — Details

### Workspace Lifecycle

```
Create → Configure Stack → Import Knowledge → Active Use → Archive / Delete

1. Create: User creates workspace, provides name and customer info
2. Configure: User adds technologies to stack (OpenShift, Linux, VMware, etc.)
3. Import: User imports knowledge sources (vendor docs, SOPs, past incidents)
4. Active: Daily use — sessions, conversations, recommendations, workflow runs
5. Archive: Mark workspace as inactive when customer project ends
6. Delete: Permanently remove workspace and all associated data
```

### Workspace Context Snapshot

When a session starts, a **workspace context snapshot** is assembled:
```json
{
  "workspace_id": "ws-acme-001",
  "customer_name": "Acme Corp",
  "technology_stack": ["OpenShift 4.14", "VMware vSphere 8", "Linux RHEL 9"],
  "environments": ["production", "staging"],
  "active_knowledge_sources": 12,
  "active_skills": ["openshift", "linux", "vmware"],
  "recent_sessions": 3,
  "active_workflow": null,
  "previous_intents": ["OpenShift pod crash", "VMware performance"]
}
```

---

## Skill Metadata Model — Details

### Skill Package Structure

```
skills/
├── openshift/
│   ├── SKILL.md              # Skill definition (name, description, tools, workflows)
│   ├── metadata.json         # Structured skill metadata
│   ├── tools/
│   │   ├── list_pods.md      # Tool description
│   │   ├── describe_pod.md   # Tool description
│   │   └── check_health.md   # Tool description
│   ├── workflows/
│   │   ├── pod_crash.md      # Troubleshooting workflow
│   │   └── cluster_upgrade.md # Upgrade workflow
│   └── knowledge/
│       ├── best_practices.md # Best practices reference
│       └── common_issues.md  # Common issue patterns
├── linux/
│   ├── SKILL.md
│   └── ...
└── vmware/
    ├── SKILL.md
    └── ...
```

### Skill Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique skill identifier (e.g., `openshift`) |
| `name` | string | Display name (e.g., `Red Hat OpenShift`) |
| `version` | string | Semantic version (e.g., `1.2.0`) |
| `description` | string | One-line description |
| `category` | string | Category: `infrastructure`, `virtualization`, `database`, `monitoring`, `automation` |
| `icon` | string | Emoji or icon name |
| `enabled` | boolean | Whether the skill is active |
| `auto_start` | boolean | Whether to spawn MCP server on app launch |
| `tools` | array | List of tool definitions (name, description, schema) |
| `workflows` | array | List of workflow definitions |
| `knowledge_files` | array | List of bundled knowledge files |
| `required_credential_services` | array | Credential services needed (e.g., `openshift_api`) |
| `tags` | array | Tags for intent matching (e.g., `kubernetes`, `openshift`, `pods`) |

---

## Audit Log Data Model — Details

### Audit Log Schema

All audit logs are **append-only**. No updates or deletions (except through workspace deletion).

| Field | Description |
|-------|-------------|
| `timestamp` | ISO 8601 timestamp of the event |
| `level` | Event severity: `info`, `warn`, `error`, `debug` |
| `component` | Source component: `chat`, `observation`, `intent`, `skill`, `knowledge`, `workflow`, `credential`, `security` |
| `event` | Event type identifier |
| `workspace_id` | Associated workspace (nullable for system-wide events) |
| `session_id` | Associated session (nullable) |
| `details` | JSON object with structured event data |
| `user_action` | If the event was triggered by a user action, what was it |

### Key Audit Events

| Event | Trigger |
|-------|---------|
| `chat_sent` | User sends a chat message |
| `chat_response` | AI generates a response |
| `suggestion_shown` | A real-time suggestion is displayed |
| `suggestion_accepted` | User accepts a suggestion |
| `suggestion_dismissed` | User dismisses a suggestion |
| `skill_enabled` | User enables a skill |
| `skill_disabled` | User disables a skill |
| `skill_error` | An MCP server error occurs |
| `obs_screen_on` | Screen observation enabled |
| `obs_screen_off` | Screen observation disabled |
| `obs_clipboard_capture` | Clipboard content captured |
| `credential_accessed` | A credential is retrieved from storage |
| `knowledge_imported` | Knowledge documents are imported |
| `workspace_created` | A workspace is created |
| `workspace_switched` | User switches workspace |

---

## User Preferences Model — Details

### Preference Categories

| Category | Settings |
|----------|----------|
| **AI Provider** | Provider type, model, API key, base URL, max tokens, temperature |
| **Observation** | Screen (on/off, interval), Terminal (on/off), Clipboard (on/off, credential filter) |
| **Privacy** | Screenshot resolution, screen-locked auto-pause, observation disabled flag |
| **UI** | Theme (dark/light), language, panel layout, font size |
| **Behavior** | Auto-save conversations, auto-enable skills, streaming toggle |
| **Data** | Audit log enabled, log retention period, data export options |

### Persistence
- Settings stored in `user_settings` table (SQLite)
- Sensitive settings (API keys) encrypted
- Settings synced to in-memory cache on load
- Changes written to SQLite immediately

---

## Data Lifecycle and Retention

### Retention Policies

| Data Type | Retention | Policy |
|-----------|-----------|--------|
| Chat messages | Until workspace deleted | No automatic deletion |
| Short-term memory | 24 hours | Auto-compact, move important entries to long-term |
| Long-term memory | Until workspace deleted | Manual cleanup by user |
| Knowledge chunks | Until source deleted | Re-indexed when source updated |
| Audit logs | 90 days minimum | Configurable retention, export before purge |
| System logs | 30 days | Automatic rotation and compression |
| Session transcripts | Until workspace deleted | Compaction: older messages summarized |
| Workflows | Until skill uninstalled | Workflow runs retained with sessions |
| Recommendations | Until session ended | Auto-archived 30 days after acceptance |

### Compaction Strategy
- **Memory compaction**: Every 6 hours, short-term memory entries older than 12 hours are evaluated:
  - Importance < 0.3 → Delete
  - Importance ≥ 0.3 → Move to long-term memory
- **Session compaction**: When session exceeds 500 messages, oldest messages are summarized into a summary blob
- **Knowledge refresh**: Periodic background job checks for updated knowledge source files and re-indexes

---

## Encryption Model for Sensitive Data

### Encryption Scope

| Data | Encryption Method | Key Source |
|------|------------------|-----------|
| AI API Key | AES-256-GCM | User login credentials (Argon2id derived) |
| Server Credentials (in skill configs) | AES-256-GCM | User login credentials |
| Screenshot Images (if saved) | AES-256-GCM | Per-workspace key (Argon2id derived) |
| Clipboard Content | Not stored, processed in-memory | N/A |
| Chat Messages (if marked sensitive) | AES-256-GCM | User login credentials |

### Key Derivation

```
User Credentials (password / login)
    │
    ▼
┌──────────────┐
│ Argon2id     │
│ (salt, params│
│  from       │
│  settings)  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Master Key   │
│ (32 bytes)   │
└──────┬───────┘
       │
       ├──► AES-256-GCM key for API keys
       ├──► AES-256-GCM key for credentials
       └──► AES-256-GCM key for files (workspace-scoped)
```

### OS Credential Manager Integration

**For server credentials** (OpenShift API URL, VMware vCenter, database connections):
- Stored in OS credential manager (Windows Credential Manager / macOS Keychain)
- NOT stored in the application database
- Accessed via `keyring` crate (same as OpenHuman)
- App uses service name = `"wikilabs-copilot:<workspace_id>:<skill_id>"`

**For AI API keys**:
- Stored encrypted in `user_settings.ai_api_key` (SQLite)
- Decrypted in-memory when making API calls
- Never logged or transmitted to any component other than the AI provider

### Data at Rest

All SQLite database files should be considered readable by the OS. Therefore:
- The database itself is NOT encrypted at the file level
- Sensitive columns (API keys, credentials) are encrypted at the column level
- The OS credential manager handles the most sensitive credentials
- Users should enable full-disk encryption (BitLocker / FileVault) on their devices

## References

- [ARCHITECTURE.md](ARCHITECTURE.md) — System architecture overview
- [COMPONENT_DESIGN.md](COMPONENT_DESIGN.md) — Component descriptions
- [SECURITY_ARCHITECTURE.md](SECURITY_ARCHITECTURE.md) — Security model and threat analysis
- [MCP_ARCHITECTURE.md](MCP_ARCHITECTURE.md) — Skill architecture
- [TECHNOLOGY_SELECTION.md](TECHNOLOGY_SELECTION.md) — Technology choices (SQLite, ChromaDB rationale)