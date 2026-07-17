---
description: "Detailed component descriptions, interfaces, interactions, lifecycle, dependency matrix, thread model, error handling, and state management for Wiki Labs AI Copilot."
icon: cube
---

# Wiki Labs AI Copilot — Component Design

## Component Overview

The system consists of **9 major components** organized across 4 architectural layers:

```
┌──────────────────────────────────────────────────────┐
│                    PRESENTATION                      │
│  ┌────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │Chat UI │ │Workspace │ │Settings  │ │Reports   │  │
│  │Component│ │ Component│ │ Component│ │ Component │  │
│  └────────┘ └──────────┘ └──────────┘ └──────────┘  │
│  ┌────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │Suggest │ │Skill Mgr │ │Knowledge │ │Logs/Audit │  │
│  │. Panel │ │ Component│ │ Component │ │ Component │  │
│  └────────┘ └──────────┘ └──────────┘ └──────────┘  │
├──────────────────────────────────────────────────────┤
│                    CORE ENGINE                        │
│  ┌────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │Event   │ │  RPC     │ │PERSIST.  │ │  Config  │  │
│  │  Bus   │ │  Manager │ │  Manager │ │  Manager  │  │
│  └────────┘ └──────────┘ └──────────┘ └──────────┘  │
├──────────────────────────────────────────────────────┤
│                   DOMAIN ENGINES                     │
│  ┌────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │Observ. │ │Intent    │ │  AI      │ │Workflow  │  │
│  │ Engine │ │  Engine  │ │Provider  │ │  Engine  │  │
│  └────────┘ └──────────┘ └──────────┘ └──────────┘  │
│  ┌────────┐ ┌──────────┐ ┌──────────┐               │
│  │MCP     │ │ Knowledge│ │Credential│               │
│  │Manager │ │  System  │ │ Manager  │               │
│  └────────┘ └──────────┘ └──────────┘               │
├──────────────────────────────────────────────────────┤
│                   MCP SERVERS                         │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐      │
│  │Open  │ │Linux │ │VMware│ │Ansible│ │MySQL │ ... │
│  │Shift │ │      │ │      │ │      │ │      │      │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘      │
└──────────────────────────────────────────────────────┘
```

---

## Component 1: React Frontend

### Description
The user-facing presentation layer. A React 19 + TypeScript single-page application loaded inside a Tauri WebView. Renders all UI panels and communicates with the Rust core.

### Responsibilities
- Render and manage all UI panels (chat, workspace, suggestions, settings, etc.)
- Handle user input: chat messages, configuration changes, skill toggles
- Display AI streaming responses progressively
- Show real-time suggestion cards as they arrive
- Manage local routing/navigation between panels
- Render embedded knowledge sources and skill documentation
- Display audit log entries and system notifications
- Manage workspace switching with visual context indicators

### Interfaces

**Outgoing (to Rust Core)**:
```typescript
interface FrontendRpcClient {
  // AI interaction
  rpcChatSend(messages: ChatMessage[], workspaceId: string): Promise<ChatResponse>
  rpcChatStream(messages: ChatMessage[], workspaceId: string): AsyncIterator<ChatChunk>
  
  // Workspace
  rpcWorkspaceList(): Promise<Workspace[]>
  rpcWorkspaceCreate(name: string): Promise<Workspace>
  rpcWorkspaceGet(id: string): Promise<Workspace>
  rpcWorkspaceUpdate(id: string, patch: WorkspacePatch): Promise<Workspace>
  rpcWorkspaceDelete(id: string): Promise<void>
  rpcWorkspaceGetContext(id: string): Promise<WorkspaceContext>
  
  // Suggestions
  rpcSuggestionAccept(id: string): Promise<void>
  rpcSuggestionDismiss(id: string): Promise<void>
  
  // Skills
  rpcSkillEnable(skillId: string): Promise<void>
  rpcSkillDisable(skillId: string): Promise<void>
  rpcSkillGetConfig(skillId: string): Promise<SkillConfig>
  rpcSkillSetConfig(skillId: string, config: SkillConfig): Promise<void>
  
  // Knowledge
  rpcKnowledgeImport(source: KnowledgeSource): Promise<KnowledgeDoc[]>
  rpcKnowledgeSearch(query: string, workspaceId: string): Promise<KnowledgeResult[]>
  rpcKnowledgeList(workspaceId: string): Promise<KnowledgeDoc[]>
  
  // Settings
  rpcSettingsGet(): Promise<AppSettings>
  rpcSettingsUpdate(partial: Partial<AppSettings>): Promise<void>
  
  // Logs
  rpcAuditLogList(filter: AuditFilter): Promise<AuditLogEntry[]>
  rpcSystemLogList(filter: LogFilter): Promise<SystemLogEntry[]>
  
  // Observation controls
  rpcObsToggleScreen(enabled: boolean): Promise<void>
  rpcObsToggleTerminal(enabled: boolean): Promise<void>
  rpcObsToggleClipboard(enabled: boolean): Promise<void>
}
```

**Incoming (from Rust Core)**:
```typescript
interface FrontendEventHandlers {
  onAiProgress: (data: { sessionId: string; progress: string; stage: string }) => void
  onSuggestionNew: (data: { id: string; workspaceId: string; type: string; content: string }) => void
  onObsAlert: (data: { type: string; severity: string; message: string }) => void
  onSkillError: (data: { skillId: string; error: string }) => void
  onKnowledgeRefresh: (data: { workspaceId: string; docsIndexed: number }) => void
  onWorkspaceChanged: (data: { workspaceId: string }) => void
  onConnectionStatus: (data: { coreConnected: boolean; aiProviderOnline: boolean }) => void
}
```

### Interactions
- Sends JSON-RPC requests to core via `http://127.0.0.1:<port>/rpc`
- Subscribes to WebSocket events from core for streaming data
- Uses Tauri IPC for native OS commands (file dialogs, notifications, deep linking)
- Maintains Redux Toolkit state for UI state; persists critical state with Redux Persist

### Lifecycle
1. **Mount**: Tauri creates WebView, loads Vite-built React bundle
2. **Init**: Frontend sends `rpcSettingsGet` and `rpcWorkspaceList` to core
3. **Runtime**: Active for the lifetime of the desktop session
4. **Shutdown**: On Tauri app close, sends `rpcSettingsSave` to persist settings

### Error Handling
- Network errors: Display offline banner, queue RPC calls, replay on reconnect
- Core unavailable: Show connection error panel, attempt auto-reconnect with backoff
- Invalid responses: Log to audit, show generic error to user
- Permission denied: Request user to grant observation permissions via OS dialog

---

## Component 2: AI Chat Component

### Description
The primary interaction surface. Renders a chat interface with streaming responses, conversation threading, inline references to knowledge sources, and action buttons for suggestion acceptance.

### Responsibilities
- Render conversation messages (user and AI) with proper formatting (markdown, code blocks)
- Support streaming responses with progressive rendering
- Show inline citations linking to knowledge sources
- Render suggestion cards within the chat flow
- Support multi-turn conversation with context
- Allow user to ask follow-up questions
- Support workspace context switching mid-conversation

### State
- Conversation history (per workspace, stored in memory and persisted to SQLite)
- Active conversation thread ID
- Cursor position for streaming content
- Citation references resolved from knowledge system

---

## Component 3: Observation Engine

### Description
Captures the engineer's current context from multiple sources: screen, applications, terminal, and clipboard. This is the system's "eyes and ears."

### Sub-Components

#### 3a. Screen Observer
Captures screenshots at a configurable interval (default: 2 seconds) of the active window or full screen.

**Data captured**:
- Screenshot image (RGB, configurable resolution capped at 1920x1080 for privacy)
- Window metadata: title, class, process name, bounds
- Active application name

**Interfaces**:
```rust
trait ScreenObserver {
  async fn take_screenshot(&self, region: CaptureRegion) -> Result<Image>;
  async fn get_active_window(&self) -> Result<WindowInfo>;
  fn set_interval(&self, interval: Duration);
  fn is_enabled(&self) -> bool;
}

enum CaptureRegion {
  ActiveWindow,
  FullScreen,
  Rect(Rectangle),
}
```

#### 3b. Application Monitor
Detects the currently active application and extracts relevant metadata.

**Data captured**:
- Active app name and bundle ID
- Window title and class
- Browser URL (if Chrome/Firefox/Safari detected)
- Terminal session detection (tmux, screen, bash, PowerShell)

**Interfaces**:
```rust
trait AppMonitor {
  async fn get_active_app(&self) -> Result<ActiveApp>;
  fn subscribe(&self, handler: Box<dyn AppEventHandler>);
}

struct ActiveApp {
  name: String,
  bundle_id: Option<String>,
  window_title: String,
  process_id: u32,
  metadata: AppMetadata,  // app-specific info (browser URL, terminal sessions)
}
```

#### 3c. Terminal Observer
Monitors terminal windows for command input and output.

**Data captured**:
- Commands entered (raw text)
- Command output (stdout, stderr)
- Exit codes
- Working directory
- Shell type (bash, zsh, PowerShell, fish)

**Interfaces**:
```rust
trait TerminalObserver {
  async fn get_active_terminals(&self) -> Result<Vec<TerminalSession>>;
  fn subscribe_command(&self, handler: Box<dyn CommandHandler>);
  fn subscribe_output(&self, handler: Box<dyn OutputHandler>);
}

struct TerminalSession {
  pid: u32,
  shell: String,
  working_dir: PathBuf,
  title: String,
  commands: Vec<CommandRecord>,
}
```

#### 3d. Clipboard Observer
Optionally monitors clipboard for copied logs, errors, configs, or code snippets.

**Data captured**:
- Text content (sanitized — no credential detection)
- Timestamp
- Content type detection (log, error, config, code)

**Interfaces**:
```rust
trait ClipboardObserver {
  async fn get_clipboard_text(&self) -> Result<Option<String>>;
  fn subscribe_change(&self, handler: Box<dyn ClipboardHandler>);
  fn is_enabled(&self) -> bool;
}
```

### Privacy Controls
All observation sources have independent on/off toggles in Settings. Default: all disabled. User must explicitly enable each source. Screenshot resolution capped at 1920x1080. Screenshots processed in-memory and never persisted unless the user explicitly saves a conversation that includes them.

### Lifecycle
- **Start**: On app launch if enabled in settings, or on user toggle
- **Poll**: At configured interval, captures data and publishes `ObservationEvent` to event bus
- **Stop**: When disabled by user or on app shutdown
- **Recovery**: If a capture fails (e.g., screen lock), logs warning and continues with next interval

### Error Handling
- Permission denied: Show OS-level permission prompt, update toggle state to off
- Capture failure: Log warning, continue with last known state
- High CPU usage: Auto-debounce capture interval
- Screen locked: Skip capture, log notice

---

## Component 4: Intent Recognition Engine

### Description
Analyzes multi-source context to infer what the engineer is trying to accomplish. Combines observations, conversation history, workspace context, and previous interactions.

### Input Sources
1. **Current observation**: Active app, screen content, terminal commands
2. **Recent commands**: Last N commands in terminal
3. **Conversation context**: Previous AI interactions in current session
4. **Workspace metadata**: Active workspace tech stack, customer info
5. **Historical patterns**: Engineer's typical workflow sequences

### Output
```rust
struct Intent {
  technology: String,           // e.g., "OpenShift", "VMware", "Nagios"
  goal: String,                 // e.g., "troubleshoot performance", "deploy cluster"
  confidence: f32,              // 0.0 to 1.0
  evidence: Vec<Evidence>,      // signals that led to this intent
  workspace_id: String,         // active workspace
  timestamp: Instant,
}

struct Evidence {
  signal_type: SignalType,      // terminal_command, window_title, clipboard, conversation
  signal_value: String,
  weight: f32,
}
```

### Interfaces
```rust
trait IntentEngine {
  async fn analyze(&self, context: AnalysisContext) -> Result<Intent>;
  fn subscribe(&self, handler: Box<dyn IntentHandler>);
  fn clear_history(&self);
  fn update_history(&self, history: Vec<HistoryEntry>);
}

struct AnalysisContext {
  observation: ObservationSnapshot,
  recent_commands: Vec<CommandRecord>,
  conversation: Vec<ChatMessage>,
  workspace: WorkspaceContext,
  previous_intents: Vec<Intent>,
}
```

### Algorithm
A lightweight rule-based + ML hybrid:
1. **Rule engine**: Pattern matching on terminal commands (e.g., `oc get pods` → OpenShift), window titles (e.g., "vSphere Client" → VMware), and clipboard content
2. **Context aggregation**: Scores technology hypotheses based on evidence weight
3. **Temporal smoothing**: Maintains intent history; transitions between intents are smoothed to avoid flickering
4. **Confidence threshold**: Only emits intent if confidence > 0.6; otherwise keeps last known intent

### Lifecycle
- **Initialize**: Load intent patterns from skill metadata
- **Run**: On each observation event, re-analyze if significant context change detected
- **Transition**: When confidence in new intent exceeds old intent by > 0.2, emit intent change
- **Reset**: When workspace changes or user explicitly starts a new conversation

---

## Component 5: MCP Skill Manager

### Description
Manages the lifecycle of MCP skill servers: discovery, loading, starting, stopping, health checking, and routing.

### Interfaces
```rust
trait SkillManager {
  async fn discover_skills(&self) -> Result<Vec<SkillMetadata>>;
  async fn load_skill(&self, skill_id: &str) -> Result<SkillInfo>;
  async fn start_skill(&self, skill_id: &str) -> Result<SkillHandle>;
  async fn stop_skill(&self, skill_id: &str) -> Result<()>;
  async fn restart_skill(&self, skill_id: &str) -> Result<()>;
  async fn get_skill_status(&self, skill_id: &str) -> Result<SkillStatus>;
  async fn list_tools(&self, skill_id: &str) -> Result<Vec<ToolDefinition>>;
  async fn call_tool(&self, skill_id: &str, tool_name: &str, args: JsonValue) -> Result<ToolResult>;
  async fn find_relevant_skills(&self, intent: &Intent) -> Result<Vec<SkillId>>;
  async fn install_skill(&self, package: SkillPackage) -> Result<SkillMetadata>;
  async fn uninstall_skill(&self, skill_id: &str) -> Result<()>;
  async fn update_skill(&self, skill_id: &str) -> Result<SkillMetadata>;
}

enum SkillStatus {
  Uninstalled,
  Installed,
  Running,
  Error(String),
}

struct SkillHandle {
  skill_id: String,
  process: ProcessHandle,
  mcp_client: McpClient,
  tools: Vec<ToolDefinition>,
  health: HealthCheck,
}
```

### Responsibilities
- **Discovery**: Scan skill directory for installed skill packages
- **Registration**: Register skill metadata with the skill registry
- **Lifecycle**: Start/stop/restart MCP server processes as needed
- **Routing**: Route tool calls to the correct skill server based on skill ID
- **Relevance**: Recommend which skills are relevant to the current intent
- **Health**: Monitor MCP server health; auto-restart on failure
- **Updates**: Check for skill updates from the update server

### Lifecycle
- **Startup**: Load installed skills from disk, register with registry
- **Runtime**: Spawn MCP servers on first tool call (lazy loading) or based on user preference
- **Shutdown**: Gracefully stop all MCP server processes

### Error Handling
- MCP server crash: Auto-restart with exponential backoff (1s, 2s, 4s, ..., 30s max)
- Startup failure: Log error, mark skill as Error status, notify frontend
- Tool call timeout (30s): Return timeout error, suggest retry
- MCP protocol mismatch: Log error, suggest skill update, fall back to disabled state

---

## Component 6: Knowledge System

### Description
Manages the local knowledge base: import, index, embed, search, and retrieve. Separate from MCP skills — skills provide expertise (tools, workflows), knowledge provides reference material (docs, SOPs, past incidents).

### Sub-Components

#### 6a. Knowledge Import
Ingests documents from various formats: PDF, Markdown, HTML, TXT, DOCX.

```rust
trait KnowledgeImporter {
  async fn import_file(&self, path: &Path, workspace_id: &str) -> Result<Vec<KnowledgeDoc>>;
  async fn import_directory(&self, path: &Path, workspace_id: &str) -> Result<Vec<KnowledgeDoc>>;
  async fn import_url(&self, url: &Url, workspace_id: &str) -> Result<Vec<KnowledgeDoc>>;
}
```

#### 6b. Document Processor
Splits documents into chunks, extracts metadata, generates embeddings.

```rust
struct KnowledgeDoc {
  id: String,
  title: String,
  content: String,        // raw text content
  chunks: Vec<DocumentChunk>,
  metadata: DocMetadata,
  workspace_id: String,
  source: KnowledgeSource,
  created_at: Instant,
  updated_at: Instant,
}

struct DocumentChunk {
  id: String,
  content: String,
  embedding: Vec<f32>,
  token_count: usize,
  parent_doc_id: String,
  start_line: usize,
  end_line: usize,
}
```

#### 6c. Vector Search
Semantic search over knowledge chunks using ChromaDB.

```rust
trait VectorSearch {
  async fn search(&self, query: String, workspace_id: &str, top_k: usize) -> Result<Vec<SearchResult>>;
  async fn upsert(&self, chunks: Vec<DocumentChunk>) -> Result<()>;
  async fn delete(&self, doc_ids: Vec<String>) -> Result<()>;
  async fn get_all_doc_ids(&self, workspace_id: &str) -> Result<Vec<String>>;
}
```

#### 6d. Full-Text Search
SQLite FTS5 index for keyword-based retrieval.

```rust
trait FullTextSearch {
  async fn search(&self, query: String, workspace_id: &str, top_k: usize) -> Result<Vec<SearchResult>>;
  async fn index(&self, doc_ids: Vec<String>) -> Result<()>;
  async fn reindex(&self, doc_id: String) -> Result<()>;
}
```

#### 6e. Hybrid Search
Combines vector search (70% weight) and FTS5 (30% weight) for optimal recall.

```rust
trait HybridSearch {
  async fn search(&self, query: String, workspace_id: &str, top_k: usize) -> Result<Vec<HybridResult>>;
}
```

### Interfaces
```rust
trait KnowledgeSystem {
  async fn import(&self, source: KnowledgeSource, workspace_id: &str) -> Result<ImportResult>;
  async fn search(&self, query: String, workspace_id: &str, limit: usize) -> Result<Vec<KnowledgeResult>>;
  async fn list(&self, workspace_id: &str) -> Result<Vec<KnowledgeDocSummary>>;
  async fn delete(&self, doc_id: String) -> Result<()>;
  async fn refresh_index(&self, workspace_id: &str) -> Result<()>;
  async fn get_document(&self, doc_id: String) -> Result<KnowledgeDoc>;
  async fn get_context(&self, intent: &Intent, workspace_id: &str) -> Result<KnowledgeContext>;
}
```

### Lifecycle
- **Import**: User imports documents → parsed → chunked → embedded → indexed
- **Search**: On each AI request → hybrid search → ranked results → injected into AI context
- **Refresh**: Periodic background job to re-index changed documents
- **Cleanup**: On workspace deletion, all associated knowledge is purged

---

## Component 7: AI Provider Abstraction

### Description
Abstracts the underlying AI provider to allow swapping between OpenAI, vLLM, local models, and enterprise APIs without changing application code.

### Interfaces
```rust
trait AiProvider: Send + Sync {
  /// Send a chat completion request
  async fn chat(&self, messages: Vec<ChatMessage>, params: ChatParams) -> Result<ChatResponse>;
  
  /// Stream chat completion (SSE)
  async fn chat_stream(&self, messages: Vec<ChatMessage>, params: ChatParams) -> Result<ChatStream>;
  
  /// Generate embeddings for a text
  async fn embed(&self, text: String) -> Result<Vec<f32>>;
  
  /// Get provider info
  fn info(&self) -> ProviderInfo;
  
  /// Health check
  async fn health(&self) -> Result<()>;
}

struct ProviderInfo {
  name: String,           // "OpenAI", "vLLM", "Ollama", etc.
  model: String,          // "gpt-4o", "llama-3.3-70b", etc.
  max_context_window: usize,
  supports_streaming: bool,
  supports_embeddings: bool,
}

struct ChatParams {
  temperature: f32,
  max_tokens: Option<usize>,
  top_p: f32,
  stop_sequences: Vec<String>,
  presence_penalty: f32,
  frequency_penalty: f32,
}

struct ChatResponse {
  content: String,
  usage: UsageStats,
  model: String,
  stop_reason: Option<String>,
}

struct ChatStream {
  /// Returns chunks as they stream in
  async fn next_chunk(&mut self) -> Result<Option<ChatChunk>>;
}

struct ChatChunk {
  content: String,
  usage: Option<UsageStats>,
}

struct UsageStats {
  prompt_tokens: usize,
  completion_tokens: usize,
  total_tokens: usize,
}
```

### Supported Providers

| Provider | Implementation Notes |
|----------|---------------------|
| OpenAI | Official API, compatible with OpenAI SDK |
| OpenAI-Compatible | Any API with OpenAI-compatible endpoints (vLLM, Ollama, LM Studio) |
| vLLM | Self-hosted inference server |
| Ollama | Local model serving |
| Enterprise | Custom enterprise AI providers (on-prem, air-gapped) |

### Lifecycle
- **Configure**: User selects provider and model in Settings
- **Validate**: On settings change, health check is performed
- **Operate**: Provider is used for all chat and embedding requests
- **Switch**: On provider change, current conversation continues but new requests use the new provider
- **Failover**: If provider becomes unavailable, show error with suggested actions

### Error Handling
- API key invalid: Prompt user to update credentials
- Rate limited: Backoff with exponential delay, show UI indicator
- Model unavailable: Fall back to next available model in config, notify user
- Network timeout: Retry with backoff (3 attempts), then show offline error
- Provider offline: Show status banner, disable AI features

---

## Component 8: Workflow Engine

### Description
Applies structured engineering troubleshooting workflows to guide AI reasoning. Defines the methodology the AI follows when helping with problems.

### Standard Troubleshooting Workflow
```
1. Understand Symptom    ← Gather initial context from observation + conversation
2. Gather Evidence       ← Suggest commands, analyze output
3. Form Hypothesis       ← Propose root cause based on evidence
4. Validate Hypothesis   ← Suggest targeted tests to confirm/deny
5. Recommend Remediation ← Provide step-by-step fix instructions
6. Verify Result         ← Suggest verification commands
7. Document Findings     ← Generate incident summary for workspace notes
```

### Interfaces
```rust
struct Workflow {
  id: String,
  name: String,
  description: String,
  technology: String,        // "OpenShift", "VMware", "Linux", etc.
  steps: Vec<WorkflowStep>,
  commands: Vec<CommandTemplate>,  // example commands for each step
  checklists: Vec<ChecklistItem>,  // verification checklist
}

struct WorkflowStep {
  step_number: usize,
  name: String,
  description: String,
  suggested_actions: Vec<Action>,
  expected_outcomes: Vec<String>,
}

struct Action {
  type_: ActionType,       // command, check, analyze, review
  description: String,
  command_template: Option<String>,
}
```

### Responsibilities
- Load workflows from skill metadata (each skill provides technology-specific workflows)
- Match current intent to appropriate workflow
- Guide AI reasoning through workflow steps
- Track workflow progress and allow interruption/resumption
- Generate workflow summaries for documentation

### Lifecycle
- **Load**: On skill activation, load associated workflows
- **Match**: When intent is recognized, select the best-matching workflow
- **Execute**: AI references workflow steps during reasoning
- **Complete**: Workflow marks complete when root cause is identified and remediation recommended
- **Archive**: Completed workflows archived in workspace for reference

---

## Component 9: Credential Manager

### Description
Manages secure storage and retrieval of credentials using OS-native credential managers.

### Interfaces
```rust
trait CredentialManager {
  async fn save(&self, service: &str, username: &str, password: &str) -> Result<()>;
  async fn get(&self, service: &str, username: &str) -> Result<Option<String>>;
  async fn delete(&self, service: &str, username: &str) -> Result<()>;
  async fn list(&self) -> Result<Vec<CredentialEntry>>;
}

struct CredentialEntry {
  service: String,
  username: String,
  created_at: Instant,
  updated_at: Instant,
}
```

### Platform Implementation
| Platform | Implementation |
|----------|---------------|
| Windows | Windows Credential Manager via `keyring` crate |
| macOS | macOS Keychain via `keyring` crate |
| Linux | Secret Service API (KDE Wallet, GNOME Keyring) via `keyring` crate |

### Lifecycle
- **Save**: When user enters credentials (API keys, server passwords, etc.)
- **Get**: When MCP server or AI tool needs credentials for a connection
- **Delete**: When user removes credentials from settings
- **Sync**: Credential sync across sessions; no persistence of raw credentials in app memory longer than needed

---

## Component Dependency Matrix

| Component | Depends On |
|-----------|-----------|
| React Frontend | Rust Core (JSON-RPC), Tauri IPC |
| AI Chat | Frontend, AI Provider, Knowledge System |
| Observation Engine | Rust Core (OS APIs), Permission System |
| Intent Recognition | Observation Engine, Event Bus |
| MCP Skill Manager | Rust Core (process management), Credential Manager |
| Knowledge System | Rust Core (persistence), AI Provider (embeddings) |
| AI Provider Abstraction | Network stack, Config Manager |
| Workflow Engine | Intent Recognition, MCP Skill Manager |
| Credential Manager | OS credential store, Config Manager |
| Config Manager | Persistence (SQLite), OS config |
| Persistence | SQLite, File System |
| Event Bus | Tokio (async runtime) |
| RPC Manager | Event Bus, Config Manager |
| Security Manager | Encryption library, Credential Manager |

---

## Thread / Process Model

```
┌─────────────────────────────────────────────────────────┐
│                    MAIN THREAD (Tauri)                   │
│  - Tauri event loop                                      │
│  - WebView lifecycle                                     │
│  - Tauri IPC handlers                                    │
└────────────────────┬────────────────────────────────────┘
                     │
    ┌────────────────┼────────────────┐
    │                │                │
    ▼                ▼                ▼
┌─────────┐  ┌──────────────┐  ┌────────────┐
│Tokio    │  │ Tokio        │  │ Tokio      │
│Main     │  │ I/O Worker   │  │ Worker     │
│Runtime  │  │ Threads      │  │ Threads    │
└─────────┘  └──────────────┘  └────────────┘
    │
    ├── Rust Core (in-process tokio task)
    │   ├── Event Bus (async channels)
    │   ├── RPC Server (HTTP listener)
    │   ├── Observation Engine (screen, app, terminal, clipboard)
    │   ├── Intent Recognition Engine
    │   ├── MCP Skill Manager
    │   ├── Knowledge System
    │   ├── AI Provider Abstraction
    │   ├── Workflow Engine
    │   ├── Credential Manager
    │   ├── Config Manager
    │   └── Persistence (SQLite)
    │
    ├── MCP Server Processes (spawned as needed)
    │   ├── OpenShift MCP Server (independent process)
    │   ├── Linux MCP Server (independent process)
    │   ├── VMware MCP Server (independent process)
    │   └── ... (one per skill)
    │
    └── Background Tokio Tasks
        ├── Knowledge index refresh (periodic)
        ├── Memory compaction (periodic)
        ├── MCP server health checks (periodic)
        ├── Intent analysis (event-driven)
        └── Observation capture (event-driven)
```

### Key Design Decisions
1. **Single Tokio Runtime**: All core components share one Tokio runtime for consistent async semantics
2. **MCP Servers as Separate Processes**: Each skill runs in its own process for isolation and fault tolerance
3. **In-Process Core**: The Rust core runs as a tokio task within the Tauri process (like OpenHuman), avoiding inter-process overhead for core communication
4. **Observation on Dedicated Tasks**: Screen capture and terminal monitoring run as dedicated background tasks, not blocking the main event loop
5. **AI Provider on I/O Threads**: External API calls run on I/O worker threads, not blocking compute threads

---

## Error Handling Strategy

### Error Hierarchy
```rust
enum CopilotError {
  // Transport / Communication
  RpcError { method: String, code: i32, message: String },
  NetworkError { url: String, cause: String },
  McpTransportError { skill_id: String, cause: String },
  
  // Domain Errors
  IntentRecognitionFailed { reason: String },
  KnowledgeSearchFailed { query: String, cause: String },
  WorkflowExecutionFailed { workflow_id: String, step: usize, cause: String },
  
  // Skill / Tool Errors
  SkillNotFound { skill_id: String },
  SkillCrashed { skill_id: String, exit_code: Option<i32> },
  ToolCallFailed { skill_id: String, tool_name: String, error: String },
  
  // Data / Persistence Errors
  DatabaseError { table: String, operation: String, cause: String },
  KnowledgeIndexCorrupted { workspace_id: String },
  
  // Permission / Security Errors
  PermissionDenied { component: String },
  CredentialNotFound { service: String },
  
  // AI Provider Errors
  AiProviderError { provider: String, error: String },
  AiProviderUnavailable { provider: String, retryable: bool },
  
  // System Errors
  ObservationCaptureFailed { source: String, cause: String },
  SystemInsufficientResources { resource: String },
}
```

### Error Handling Patterns
- **Recoverable**: Auto-retry (e.g., network timeout, MCP server crash)
- **User-Action-Required**: Show UI prompt (e.g., permission denied, invalid API key)
- **Non-Recoverable**: Log and disable feature (e.g., database corruption, skill permanently broken)
- **Graceful Degradation**: Continue with reduced functionality (e.g., no AI provider → copilot in offline mode)

---

## State Management Approach

### Frontend State (Redux Toolkit)
```typescript
interface AppState {
  // UI state
  ui: UiState;          // active panels, sidebar, themes
  chat: ChatState;      // conversations, active thread, messages
  workspace: WorkspaceState;  // active workspace, workspace list
  skills: SkillsState;  // installed skills, enabled status, configs
  knowledge: KnowledgeState;  // search results, imported docs
  settings: SettingsState;  // user preferences
  logs: LogsState;      // audit logs, system logs
  notifications: NotificationState;  // toast messages, alerts
}
```

### Core State (Rust)
```rust
struct CoreState {
  // Runtime state
  workspace_id: String,
  intent: Option<Intent>,
  active_skills: HashMap<String, SkillHandle>,
  
  // Memory state
  short_term_memory: Vec<MemoryEntry>,
  long_term_memory: Vec<MemoryEntry>,
  subconscious_memory: Vec<MemoryEntry>,
  
  // Config state
  settings: AppSettings,
  credential_cache: HashMap<String, String>,
  
  // Health state
  provider_health: ProviderHealth,
  skill_health: HashMap<String, SkillHealth>,
  obs_health: ObservationHealth,
}
```

### Persistence Strategy
- **Frontend**: Redux Persist for critical UI state (active workspace, settings, enabled skills)
- **Core**: SQLite for all persistent data (workspaces, knowledge, audit logs, session transcripts)
- **Memory**: In-memory with periodic SQLite persistence (short-term memory compacted, long-term memory archived)

## References

- [ARCHITECTURE.md](ARCHITECTURE.md) — System-level architecture overview
- [DATA_MODEL.md](DATA_MODEL.md) — Data entities and schemas
- [MCP_ARCHITECTURE.md](MCP_ARCHITECTURE.md) — MCP skill architecture
- [SECURITY_ARCHITECTURE.md](SECURITY_ARCHITECTURE.md) — Security model
- [TECHNOLOGY_SELECTION.md](TECHNOLOGY_SELECTION.md) — Technology choices