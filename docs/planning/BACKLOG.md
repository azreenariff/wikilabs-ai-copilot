# Wiki Labs AI Copilot — Product Backlog

> **Last Updated:** 2025-07-16  
> **Version:** 1.0  
> **Status:** Planning  
> **Owner:** Technical Lead

---

## Table of Contents

1. [Backlog Structure](#backlog-structure)
2. [Priority Legend](#priority-legend)
3. [Complexity Legend](#complexity-legend)
4. [Labels Legend](#labels-legend)
5. [Epics and Stories](#epics-and-stories)
6. [Backlog Legend](#backlog-legend)

---

## Backlog Structure

This backlog is organized into **epics** (large bodies of work) containing **user stories** (actionable, testable items). Each story includes:

- **ID:** Unique identifier (EPIC-###)
- **Title:** Brief description
- **As a... I want... So that...** (user story format)
- **Acceptance Criteria:** Verifiable conditions
- **Complexity:** S/M/L/XL estimate
- **Labels:** feature / improvement / infrastructure / technical-debt
- **Phase:** Which roadmap phase this belongs to
- **Dependencies:** Stories this depends on

---

## Priority Legend

| Priority | Meaning |
|----------|---------|
| **P0** | Must have — blocks all other work; MVP-critical |
| **P1** | Should have — essential for product value; Phase 1-2 |
| **P2** | Nice to have — adds meaningful value but not critical |
| **P3** | Could have — nice to have when capacity allows |

## Complexity Legend

| Complexity | Meaning |
|-----------|---------|
| **S** | Simple — 1-2 days of work, well-understood problem |
| **M** | Medium — 3-5 days, some unknowns, needs design discussion |
| **L** | Large — 1-2 weeks, significant unknowns, needs spike |
| **XL** | Extra Large — 3+ weeks, multiple unknowns, likely needs splitting |

## Labels Legend

| Label | Meaning |
|-------|---------|
| `feature` | New user-facing capability |
| `improvement` | Enhancement to existing capability |
| `infrastructure` | Build system, CI/CD, tooling, developer experience |
| `technical-debt` | Fixing accumulated technical debt |

---

## Epics and Stories

### EPIC-1: Desktop Application Shell
**Phase:** 0, 1  
**Summary:** Foundation of the desktop application using Tauri v2 + React + Rust.

#### EPIC-1.1 — Main Application Window
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want a native desktop window so that the copilot feels like a first-class application, not a web page.

**Acceptance Criteria:**
- [ ] Tauri v2 window renders on Windows 11 and macOS Sonoma
- [ ] Window is resizable with minimum dimensions (800x600)
- [ ] Window survives minimization and restoration
- [ ] Window title includes app name and version
- [ ] Window icon is set for both platforms
- [ ] Window is draggable by the title bar

**Dependencies:** None

---

#### EPIC-1.2 — Sidebar Navigation
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want a sidebar with navigation sections so that I can switch between features quickly.

**Acceptance Criteria:**
- [ ] Sidebar displays: Chat, Suggestions, Skills, Knowledge, Workspaces, Settings, Logs
- [ ] Active section is highlighted
- [ ] Sidebar can be collapsed (minimized) on narrow screens
- [ ] Sidebar remembers last active section
- [ ] Navigation works with mouse clicks and keyboard shortcuts

**Dependencies:** EPIC-1.1

---

#### EPIC-1.3 — Settings Panel
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0, 1

> As an engineer, I want a settings panel so that I can configure the copilot without editing files.

**Acceptance Criteria:**
- [ ] AI provider configuration (select provider, enter API endpoint, enter API key)
- [ ] Model selection within each provider
- [ ] Skill toggle controls (on/off per skill)
- [ ] Privacy controls (observation source toggles)
- [ ] Connection test button for AI provider
- [ ] Settings persist across app restarts
- [ ] API key stored in credential manager, not in settings file
- [ ] Settings validation (e.g., API key format check)

**Dependencies:** EPIC-1.1

---

#### EPIC-1.4 — Dark Theme
**Priority:** P1 | **Complexity:** S  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want a dark theme so that the app is easy on the eyes during long work sessions.

**Acceptance Criteria:**
- [ ] Professional dark theme with engineering-tool aesthetic
- [ ] Theme applies consistently across all panels
- [ ] Sufficient contrast for readability (WCAG AA minimum)
- [ ] No theme-related rendering bugs in React components

**Dependencies:** EPIC-1.1

---

#### EPIC-1.5 — Version Display
**Priority:** P1 | **Complexity:** S  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to see the app version so that I know I'm running the latest build.

**Acceptance Criteria:**
- [ ] About dialog shows app version, build date, and build number
- [ ] Version displayed in Settings panel
- [ ] Version string matches git tag or build artifact version

**Dependencies:** EPIC-1.1, EPIC-1.2

---

### EPIC-2: AI Chat Interface
**Phase:** 0, 1  
**Summary:** Conversational interface for interacting with the AI copilot.

#### EPIC-2.1 — Chat Window
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want a chat window so that I can ask questions and receive answers from the copilot.

**Acceptance Criteria:**
- [ ] Message input field with send button and keyboard shortcut (Enter)
- [ ] Chat messages displayed in chronological order with user/assistant differentiation
- [ ] User messages right-aligned, assistant messages left-aligned
- [ ] Markdown rendering for assistant responses (code blocks, lists, bold, etc.)
- [ ] Empty state with guidance text when no messages exist
- [ ] Scroll-to-bottom when new message arrives

**Dependencies:** EPIC-1.2

---

#### EPIC-2.2 — Context Injection
**Priority:** P0 | **Complexity:** L  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to automatically include my current context (screen, terminal, app) in my questions so that I don't have to describe what I'm working on.

**Acceptance Criteria:**
- [ ] Current screen OCR text is included in the system prompt when sending a message
- [ ] Active app info (app name, window title, URL) is included in the system prompt
- [ ] Recent terminal commands (last 5) are included in the system prompt
- [ ] Detected intent is included in the system prompt
- [ ] Engineer can toggle context injection on/off per message
- [ ] System prompt context is not persisted in chat history (only sent to AI)
- [ ] Context size is bounded to fit within model context window

**Dependencies:** EPIC-2.1, EPIC-6 (Observation Engine)

---

#### EPIC-2.3 — Knowledge Citation
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to cite its knowledge sources so that I can verify the information it provides.

**Acceptance Criteria:**
- [ ] Knowledge sources are shown as inline citations in the response (e.g., `[1]`, `[2]`)
- [ ] Clicking a citation shows the source document name and relevant excerpt
- [ ] At least 3 sources are shown per response (or all if fewer available)
- [ ] If no knowledge was used, response states "Answered from general knowledge"
- [ ] Citations are rendered as clickable links

**Dependencies:** EPIC-2.1, EPIC-5 (Knowledge System)

---

#### EPIC-2.4 — Command Suggestions
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to display terminal commands as suggested actions so that I can copy and execute them with confidence.

**Acceptance Criteria:**
- [ ] AI-generated terminal commands are rendered in a code block with a "Copy" button
- [ ] Commands include a brief explanation of what the command does
- [ ] Warning: "Always verify commands before execution on production systems" is shown near suggested commands
- [ ] Command copy button copies only the command text (not explanation)
- [ ] Commands that could be dangerous (rm, dd, chmod 777, etc.) are flagged with a warning badge

**Dependencies:** EPIC-2.1

---

#### EPIC-2.5 — Streaming Responses
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want AI responses to stream in real-time so that I don't wait for the full response before reading.

**Acceptance Criteria:**
- [ ] Response tokens appear as they are generated (streaming via SSE or similar)
- [ ] Typing indicator is shown while waiting for first token
- [ ] Streaming is interruptible (user can send a new message to stop streaming)
- [ ] Markdown rendering updates progressively as tokens arrive
- [ ] Full response is persisted to chat history when streaming completes

**Dependencies:** EPIC-2.1, EPIC-3 (AI Provider Layer)

---

#### EPIC-2.6 — Conversation History
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want my chat conversations to persist across app restarts so that I can return to previous discussions.

**Acceptance Criteria:**
- [ ] Chat sessions are saved to local encrypted storage on every message
- [ ] Sessions are loaded on app startup, restored to the last active session
- [ ] Sessions are scoped per workspace (each workspace has its own conversation history)
- [ ] Engineer can rename a session by clicking the session title
- [ ] Engineer can delete a session
- [ ] Session list shows last message preview and timestamp
- [ ] Sessions are encrypted at rest (AES-256-GCM)

**Dependencies:** EPIC-2.1, EPIC-8 (Workspace System), EPIC-9 (Security)

---

#### EPIC-2.7 — Message Actions
**Priority:** P1 | **Complexity:** S  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want actions on chat messages so that I can copy, regenerate, or provide feedback on responses.

**Acceptance Criteria:**
- [ ] Hovering a message reveals action buttons: Copy, Regenerate, 👍, 👎
- [ ] Copy copies message text to clipboard
- [ ] Regenerate sends the same prompt to AI and replaces the response
- [ ] 👍/👎 sends anonymous feedback to local telemetry (opt-in, stored locally)
- [ ] Feedback is tagged with the current skill and context (for internal analytics)

**Dependencies:** EPIC-2.1

---

#### EPIC-2.8 — Error Handling
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want clear error messages when the AI provider is unavailable so that I know what to do.

**Acceptance Criteria:**
- [ ] API connectivity error displays clear message: "Could not connect to AI provider. Check your settings and network."
- [ ] Rate limit error displays: "API rate limit reached. Please wait and try again."
- [ ] Malformed response error displays: "Unexpected response from AI provider. Try switching providers."
- [ ] Offline mode is gracefully handled (message: "AI provider unavailable. You can still browse knowledge.")
- [ ] Error messages include a "Retry" button for recoverable errors
- [ ] No crash or hang on any error condition

**Dependencies:** EPIC-2.1, EPIC-3 (AI Provider Layer)

---

### EPIC-3: AI Provider Layer
**Phase:** 0  
**Summary:** Abstracted layer for connecting to different AI providers.

#### EPIC-3.1 — OpenAI Integration
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to connect the copilot to OpenAI API so that I can use GPT-4o for reasoning.

**Acceptance Criteria:**
- [ ] Connect to OpenAI API (`api.openai.com`)
- [ ] Supports gpt-4o and gpt-4o-mini models
- [ ] Streaming responses supported
- [ ] API key stored in OS credential manager
- [ ] Connection test verifies API key and endpoint

**Dependencies:** EPIC-1.3 (Settings Panel)

---

#### EPIC-3.2 — OpenAI-Compatible API Integration
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to connect to any OpenAI-compatible API so that I can use self-hosted or local models.

**Acceptance Criteria:**
- [ ] Configurable API endpoint (any URL)
- [ ] Configurable API key
- [ ] Supports any model that exposes OpenAI-compatible chat completion endpoint
- [ ] Connection test verifies endpoint and model availability
- [ ] Falls back to standard OpenAI endpoint format if no custom endpoint specified

**Dependencies:** EPIC-3.1, EPIC-1.3 (Settings Panel)

---

#### EPIC-3.3 — Provider Selection and Switching
**Priority:** P0 | **Complexity:** S  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to switch between AI providers so that I can choose the best model for each task.

**Acceptance Criteria:**
- [ ] Settings panel lists all configured providers
- [ ] User selects current active provider from dropdown
- [ ] Model selector shows available models for the selected provider
- [ ] Provider change takes effect on next chat message (no restart required)
- [ ] Default provider is set on first launch (OpenAI)

**Dependencies:** EPIC-3.1, EPIC-3.2, EPIC-1.3 (Settings Panel)

---

#### EPIC-3.4 — Rate Limit Handling
**Priority:** P1 | **Complexity:** S  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want the copilot to handle API rate limits gracefully so that my workflow isn't disrupted.

**Acceptance Criteria:**
- [ ] 429 rate limit responses are detected
- [ ] User sees message: "Rate limit reached. Retrying in X seconds..." with countdown
- [ ] Retry logic implements exponential backoff (max 3 retries)
- [ ] If all retries fail, user sees clear error with manual retry option
- [ ] Retry logic does not block the UI thread

**Dependencies:** EPIC-3.1, EPIC-3.2

---

### EPIC-4: Observation Engine
**Phase:** 1  
**Summary:** Engine that observes the engineer's environment: screen, terminal, applications.

#### EPIC-4.1 — Screen Capture
**Priority:** P0 | **Complexity:** L  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to capture my screen so that it can understand what I'm looking at.

**Acceptance Criteria:**
- [ ] Rust backend captures current screen (screenshot) on-demand
- [ ] Capture can be triggered when the copilot window receives or loses focus
- [ ] Capture is on-demand only in MVP (no background continuous capture)
- [ ] Screen capture respects OS-level privacy (e.g., macOS screen capture permission)
- [ ] Capture resolution is configurable (default: 1920x1080)
- [ ] Capture interval is bounded (max 1 capture per 5 seconds to reduce CPU usage)
- [ ] User can pause all screen capture with privacy mode

**Dependencies:** None (Tauri v2 has screen capture APIs)

---

#### EPIC-4.2 — Screen OCR
**Priority:** P0 | **Complexity:** L  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to extract text from my screen so that it can understand terminal output, browser content, and application UIs.

**Acceptance Criteria:**
- [ ] OCR is run on each screen capture
- [ ] OCR output includes text content and approximate bounding box positions
- [ ] OCR handles terminal-style text (monospace, colored) correctly
- [ ] OCR handles mixed-content screens (terminal + browser + IDE) with reasonable accuracy
- [ ] OCR processing time is under 2 seconds for a 1920x1080 capture
- [ ] OCR results are cached for 10 seconds (to avoid redundant processing)
- [ ] Engineer is informed when OCR detects terminal output vs. other UI content

**Dependencies:** EPIC-4.1 (Screen Capture)

---

#### EPIC-4.3 — Active App Detection
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to know which application is active so that it can provide relevant context.

**Acceptance Criteria:**
- [ ] Detects the currently focused application (process name and title)
- [ ] Detects the active window title
- [ ] Works on both Windows and macOS
- [ ] Detects common applications by name (Terminal, VS Code, Chrome, Firefox, etc.)
- [ ] App detection updates on window focus change (within 500ms)
- [ ] Fallback: if detection fails, shows "Unknown application"

**Dependencies:** None (Tauri has active window APIs)

---

#### EPIC-4.4 — Browser URL Detection
**Priority:** P1 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to know which browser page I'm viewing so that it can provide relevant assistance.

**Acceptance Criteria:**
- [ ] Detects if a browser window is active
- [ ] Extracts the current URL from the browser's address bar
- [ ] Works with Chrome, Firefox, and Edge on both platforms
- [ ] URL is shown in the suggestion panel when a browser is active
- [ ] If URL extraction fails, shows "Browser window active" without URL
- [ ] Does NOT extract page content or DOM (only URL)

**Dependencies:** EPIC-4.3 (Active App Detection)

---

#### EPIC-4.5 — Terminal Command Tracking
**Priority:** P0 | **Complexity:** L  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to observe my terminal commands so that it can understand what I'm troubleshooting.

**Acceptance Criteria:**
- [ ] Detects when a terminal window is active (Terminal, iTerm, Windows Terminal, PowerShell)
- [ ] Captures the current command being typed (not yet executed)
- [ ] Captures the most recent command that was executed (last 5 commands)
- [ ] Captures the output of the last executed command (truncated to 1000 chars)
- [ ] Detects common error patterns in command output (exit code, error keywords)
- [ ] Does NOT capture passwords or sensitive data in commands (mask * patterns)
- [ ] Terminal tracking updates within 1 second of command execution
- [ ] User can exclude specific terminal processes from observation in privacy settings

**Dependencies:** EPIC-4.1 (Screen Capture), EPIC-4.3 (Active App Detection)

---

#### EPIC-4.6 — Clipboard Observation
**Priority:** P1 | **Complexity:** S  
**Labels:** feature  
**Phase:** 1

> As an engineer, I optionally want the copilot to observe my clipboard so that it can provide context from copied text.

**Acceptance Criteria:**
- [ ] Clipboard observation is off by default
- [ ] User can enable clipboard observation in privacy settings
- [ ] When enabled, copilot observes clipboard content on change
- [ ] Clipboard content is NOT stored; only analyzed at the moment of observation
- [ ] User can pause clipboard observation independently
- [ ] Clear warning: "Clipboard content is analyzed in real-time and not stored"

**Dependencies:** None (Tauri has clipboard APIs)

---

#### EPIC-4.7 — Observation Controls
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want to control which data the copilot observes so that I maintain privacy and comfort.

**Acceptance Criteria:**
- [ ] Privacy controls panel lists all observation sources: Screen, Terminal, Clipboard, App Info
- [ ] Each source has an independent on/off toggle
- [ ] Privacy mode button ("Pause Observation") disables ALL sources simultaneously
- [ ] Current observation state is visible in the UI (e.g., "Observing: Screen ✓ Terminal ✗ Clipboard ✗")
- [ ] Changes to observation settings take effect immediately (no restart)
- [ ] Settings persist across app restarts

**Dependencies:** EPIC-4.1 through EPIC-4.6

---

### EPIC-5: Knowledge System
**Phase:** 0, 1  
**Summary:** Local-first knowledge base with vector search for retrieval-augmented generation.

#### EPIC-5.1 — Local Vector Store
**Priority:** P0 | **Complexity:** M  
**Labels:** infrastructure  
**Phase:** 0

> As an engineer, I need a local vector database so that I can store and search knowledge documents without sending data to the cloud.

**Acceptance Criteria:**
- [ ] Chroma (or LanceDB) is embedded and runs locally (no server process)
- [ ] Vector store persists data to disk between app restarts
- [ ] Vector store is scoped per workspace
- [ ] Storage is encrypted at rest (AES-256-GCM)
- [ ] Vector store initialization takes under 1 second
- [ ] Vector store memory usage is under 200 MB

**Dependencies:** None

---

#### EPIC-5.2 — Document Ingestion Pipeline
**Priority:** P0 | **Complexity:** L  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to upload documents so that I can add my organization's knowledge to the copilot.

**Acceptance Criteria:**
- [ ] User can upload PDF, Markdown (.md), and plain text (.txt) files via UI
- [ ] Upload queue shows progress for each file
- [ ] Uploaded files are stored locally in encrypted storage
- [ ] Upload validates file type and size (max 50 MB per file)
- [ ] User sees status: "Processing," "Embedding," "Indexed," "Complete," or "Error"
- [ ] Error message shown if processing fails (with reason)
- [ ] Bulk upload supported (multiple files at once)

**Dependencies:** EPIC-5.1 (Local Vector Store)

---

#### EPIC-5.3 — Document Chunking
**Priority:** P0 | **Complexity:** M  
**Labels:** infrastructure  
**Phase:** 0

> As an engineer, I want documents to be automatically split into chunks so that they can be searched effectively.

**Acceptance Criteria:**
- [ ] PDF files are converted to text before chunking
- [ ] Markdown files preserve headings and structure
- [ ] Plain text files are chunked by paragraph
- [ ] Default chunk size: 500 tokens, overlap: 50 tokens
- [ ] Chunk size is configurable in settings
- [ ] Chunks retain metadata (source document, page number, heading)
- [ ] Chunking is idempotent (re-ingesting same file doesn't create duplicates)

**Dependencies:** EPIC-5.2 (Document Ingestion)

---

#### EPIC-5.4 — Embedding Generation
**Priority:** P0 | **Complexity:** M  
**Labels:** infrastructure  
**Phase:** 0

> As an engineer, I want knowledge documents to be embedded so that they can be searched semantically.

**Acceptance Criteria:**
- [ ] Embedding model is configurable (default: text-embedding-3-small)
- [ ] Embedding generation uses the configured AI provider
- [ ] Embedding generation processes chunks in batches (max 100 per batch)
- [ ] User sees progress: "Generating embeddings: X/Y chunks"
- [ ] Embedding generation is resumable (if interrupted, continues from last successful batch)
- [ ] Embedding vector is stored in Chroma alongside chunk text

**Dependencies:** EPIC-5.3 (Document Chunking), EPIC-3 (AI Provider Layer)

---

#### EPIC-5.5 — Hybrid Search
**Priority:** P0 | **Complexity:** L  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to search knowledge documents using both keyword and semantic search so that I get the most relevant results.

**Acceptance Criteria:**
- [ ] Hybrid search combines BM25 (keyword) and vector (semantic) scores
- [ ] BM25 scores are computed from local index (Chroma's native BM25 or SQLite FTS5)
- [ ] Vector scores are computed from embedding similarity
- [ ] Scores are combined using configurable weights (default: 0.5 BM25 + 0.5 vector)
- [ ] Top 5 most relevant chunks are returned per query
- [ ] Each result includes the source document name and relevant excerpt
- [ ] Search query is under 500 characters
- [ ] Search latency is under 1 second for knowledge bases up to 500 documents

**Dependencies:** EPIC-5.1 (Local Vector Store), EPIC-5.3 (Document Chunking)

---

#### EPIC-5.6 — Knowledge UI
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to browse and manage my knowledge documents so that I can curate the copilot's knowledge base.

**Acceptance Criteria:**
- [ ] Knowledge panel shows list of all uploaded documents
- [ ] Each document entry shows: name, type, size, ingestion date, status, chunk count
- [ ] Click on a document to view its contents
- [ ] Search within knowledge base (local keyword search)
- [ ] Delete documents from the knowledge base
- [ ] Re-ingest documents (to update or re-embed)
- [ ] Knowledge count shown in sidebar badge

**Dependencies:** EPIC-5.2 (Document Ingestion), EPIC-5.4 (Embedding Generation), EPIC-5.5 (Hybrid Search)

---

#### EPIC-5.7 — Knowledge in Chat
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to use my knowledge documents when answering questions so that answers are grounded in my organization's information.

**Acceptance Criteria:**
- [ ] Knowledge retrieval is triggered automatically when the user sends a chat message
- [ ] Retrieved chunks are injected into the system prompt alongside screen/terminal context
- [ ] Retrieved source documents are shown as citations in the response
- [ ] If no knowledge is retrieved, the AI response falls back to general knowledge
- [ ] Knowledge injection respects context window limits (truncates least-relevant chunks if needed)
- [ ] Knowledge retrieval can be toggled off in settings

**Dependencies:** EPIC-5.5 (Hybrid Search), EPIC-2.1 (Chat Window), EPIC-2.3 (Knowledge Citation)

---

### EPIC-6: Intent Recognition Engine
**Phase:** 1  
**Summary:** Infers what the engineer is accomplishing based on observed context.

#### EPIC-6.1 — Rule-Based Intent Classification
**Priority:** P0 | **Complexity:** L  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to infer what I'm working on so that it can proactively surface relevant skills and knowledge.

**Acceptance Criteria:**
- [ ] Intent classification runs on screen content + terminal commands + app info + detected OCR
- [ ] Rule engine matches keywords, patterns, and app names to known intent categories
- [ ] Intent categories defined per skill (Linux, OpenShift, VMware, etc.)
- [ ] Multiple intents can be detected simultaneously (with confidence scores)
- [ ] Default intent: "General Engineering" when no specific intent matches
- [ ] Classification runs within 1 second of context change
- [ ] Classification results are cached for 30 seconds (to avoid redundant processing)

**Intent Categories (MVP):**
- Linux Performance Issue
- Linux Service Failure
- OpenShift Pod Crash
- OpenShift Upgrade
- OpenShift Deployment
- VMware Host Performance
- VMware VM Issue
- Nagios Alert Investigation
- Windows Service Issue
- Windows Performance Issue
- General Infrastructure Question

**Dependencies:** EPIC-4 (Observation Engine)

---

#### EPIC-6.2 — Intent Display
**Priority:** P1 | **Complexity:** S  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want to see the copilot's intent detection so that I know what it thinks I'm working on.

**Acceptance Criteria:**
- [ ] Detected intent is displayed in the Suggestions panel (e.g., "Seems like you're troubleshooting a Linux performance issue")
- [ ] Detected intent is shown with a confidence badge (e.g., "High confidence")
- [ ] If no intent is detected, shows "General engineering context"
- [ ] Intent is only displayed when confidence > 50%

**Dependencies:** EPIC-6.1 (Rule-Based Intent Classification)

---

#### EPIC-6.3 — Intent Override
**Priority:** P1 | **Complexity:** S  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want to correct the copilot's intent detection so that it provides better suggestions when it misunderstands my context.

**Acceptance Criteria:**
- [ ] When intent is displayed, engineer can click a "Not correct" button
- [ ] Engineer can select the correct intent from a dropdown of skill categories
- [ ] Corrected intent overrides detected intent for the current session
- [ ] Correction is logged (for internal analytics and future ML model training)
- [ ] Corrected intent is respected until the engineer changes it or starts a new task

**Dependencies:** EPIC-6.2 (Intent Display), EPIC-6.1 (Rule-Based Intent Classification)

---

#### EPIC-6.4 — Intent-Driven Suggestions
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to show relevant skill suggestions based on what it thinks I'm working on so that I can quickly access the right expertise.

**Acceptance Criteria:**
- [ ] When an intent is detected, the Suggestions panel shows the relevant skill(s)
- [ ] Suggestions include: quick action buttons ("View Linux troubleshooting guide"), relevant knowledge doc snippets, and commonly asked questions for that intent
- [ ] If no intent is detected, Suggestions panel shows generic engineering tips
- [ ] Suggestions update automatically when intent changes
- [ ] Suggestions panel shows at most 5 suggestions at a time

**Dependencies:** EPIC-6.1 (Rule-Based Intent Classification), EPIC-7 (MCP Skills)

---

### EPIC-7: MCP Skill Architecture
**Phase:** 1  
**Summary:** MCP-based skills that provide domain expertise for specific technologies.

#### EPIC-7.1 — Skill System Framework
**Priority:** P0 | **Complexity:** L  
**Labels:** infrastructure  
**Phase:** 1

> As a developer, I want a framework for defining MCP skills so that skills can be added as reusable, testable modules.

**Acceptance Criteria:**
- [ ] Skill definition format: metadata (name, version, description), knowledge references, troubleshooting workflows, best practices, examples
- [ ] Skills are loaded from a local directory structure (per-skill directories)
- [ ] Skill metadata is validated on load (schema validation)
- [ ] Skills can be toggled on/off per workspace
- [ ] Skill context is injected into AI prompts when the skill is applicable to current intent
- [ ] Skill execution is read-only: MCP servers provide information, never execute commands
- [ ] Skill system logs all skill activations and context injections (for internal analytics)

**Skills Directory Structure:**
```
skills/
  linux/
    metadata.yaml          # name, version, description, supported platforms
    knowledge/             # .md files with knowledge base content
    workflows/             # troubleshooting workflow definitions
    examples/              # example commands and responses
    tests/                 # automated tests
    MCP_SERVER.md          # MCP server specification
  openshift/
    ...
  vmware/
    ...
```

**Dependencies:** None

---

#### EPIC-7.2 — Linux Skill
**Priority:** P0 | **Complexity:** XL  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want a comprehensive Linux skill so that I can troubleshoot Linux infrastructure issues.

**Scope — Knowledge Areas:**
1. **Performance Troubleshooting** — CPU, memory, disk I/O, network performance, system calls analysis
2. **Service Management** — systemd services, process monitoring, process management, service dependencies
3. **Log Analysis** — journalctl, /var/log files, log parsing, log rotation, log shipping
4. **Package Management** — yum/dnf, apt, RPM, DEB, package troubleshooting, dependency resolution
5. **Kernel Tuning** — sysctl parameters, kernel modules, kernel parameters, network stack tuning
6. **Common Error Patterns** — OOM killer, disk full, permission denied, service crashes, network timeouts

**Acceptance Criteria:**
- [ ] Covers all 6 knowledge areas with actionable troubleshooting guidance
- [ ] At least 10 common Linux scenarios covered with full troubleshooting workflows
- [ ] Each workflow includes: symptom description, evidence gathering steps, common hypotheses, validation commands, recommended remediation, verification steps
- [ ] Knowledge is sourced from vendor docs (Red Hat, Debian), internal SOPs, and best practices
- [ ] Skills provides 20+ example commands with explanations
- [ ] Skill tests cover all troubleshooting workflows (automated)
- [ ] Skill context injection works correctly when Linux-related intent is detected

**Dependencies:** EPIC-7.1 (Skill System Framework)

---

#### EPIC-7.3 — OpenShift Skill
**Priority:** P0 | **Complexity:** XL  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want a comprehensive OpenShift skill so that I can troubleshoot OpenShift infrastructure issues.

**Scope — Knowledge Areas:**
1. **Cluster Status Checks** — cluster health, node status, operator status, version compatibility
2. **Pod Debugging** — pod crashes (CrashLoopBackOff, OOMKilled, ImagePullBackOff), pod scheduling, pod networking
3. **Resource Management** — resource quotas, limits, requests, HPA, cluster capacity planning
4. **Upgrade Awareness** — OpenShift upgrade paths, known issues, version-specific notes
5. **Common Error Patterns** — ImageStream issues, route failures, persistent volume problems, network policy errors

**Acceptance Criteria:**
- [ ] Covers all 5 knowledge areas with actionable troubleshooting guidance
- [ ] At least 10 common OpenShift scenarios covered with full troubleshooting workflows
- [ ] Each workflow includes: symptom description, evidence gathering steps, common hypotheses, validation commands, recommended remediation, verification steps
- [ ] Knowledge is sourced from Red Hat OpenShift documentation, internal SOPs, and best practices
- [ ] Skills provides 20+ example commands with explanations
- [ ] Skill tests cover all troubleshooting workflows (automated)
- [ ] Skill context injection works correctly when OpenShift-related intent is detected

**Dependencies:** EPIC-7.1 (Skill System Framework)

---

#### EPIC-7.4 — VMware Skill
**Priority:** P0 | **Complexity:** XL  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want a comprehensive VMware skill so that I can troubleshoot VMware vSphere infrastructure issues.

**Scope — Knowledge Areas:**
1. **VM Performance** — CPU ready time, memory ballooning, disk latency, network throughput, VM resource contention
2. **Resource Pools** — resource pool hierarchy, allocation, reservations, limits, share calculations
3. **Host Status** — host connectivity, ESXi health, cluster status, vCenter connectivity, host maintenance mode
4. **Datastore Monitoring** — datastore capacity, IOPS, latency, thin/thick provisioning, storage vMotion
5. **Common Alerts** — VMware alerts: heartbeat loss, datastore full, network partition, snapshot too large, HA failure

**Acceptance Criteria:**
- [ ] Covers all 5 knowledge areas with actionable troubleshooting guidance
- [ ] At least 10 common VMware scenarios covered with full troubleshooting workflows
- [ ] Each workflow includes: symptom description, evidence gathering steps, common hypotheses, validation commands, recommended remediation, verification steps
- [ ] Knowledge is sourced from VMware documentation, internal SOPs, and best practices
- [ ] Skills provides 20+ example commands with explanations (PowerCLI, ESXi CLI)
- [ ] Skill tests cover all troubleshooting workflows (automated)
- [ ] Skill context injection works correctly when VMware-related intent is detected

**Dependencies:** EPIC-7.1 (Skill System Framework)

---

#### EPIC-7.5 — Skill Selector UI
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want to see and manage available skills so that I can enable the skills relevant to my work.

**Acceptance Criteria:**
- [ ] Skills panel lists all installed skills with: name, description, version, status (enabled/disabled)
- [ ] Skills can be toggled on/off per workspace
- [ ] Enabled skills count is displayed in the sidebar
- [ ] Current skill status is shown when a skill is contextually active (e.g., "Linux skill active")
- [ ] Disabled skills still appear in the list but with a visual distinction

**Dependencies:** EPIC-7.1 (Skill System Framework)

---

#### EPIC-7.6 — Command Safety
**Priority:** P0 | **Complexity:** S  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want to review all commands before execution so that I maintain full control over my systems.

**Acceptance Criteria:**
- [ ] All commands generated by skills or AI are displayed to the user before execution
- [ ] Commands are NEVER auto-executed by the copilot (MVP requirement)
- [ ] Commands include a warning: "Review before executing on production systems"
- [ ] High-risk commands (rm, dd, mkfs, shutdown, reboot, chmod 777, rm -rf) are flagged with a red warning badge
- [ ] Copy button copies command to clipboard; engineer must manually paste and execute
- [ ] Confirmation dialog for high-risk commands: "This is a high-risk command. Are you sure?"

**Dependencies:** EPIC-2.4 (Command Suggestions)

---

### EPIC-8: Engineering Workflow Engine
**Phase:** 1  
**Summary:** Structured troubleshooting methodology guided through the copilot interface.

#### EPIC-8.1 — Troubleshooting Workflow Steps
**Priority:** P0 | **Complexity:** L  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want the copilot to guide me through a structured troubleshooting methodology so that I don't miss steps and can document my findings.

**Workflow Steps:**
1. **Understand Symptom** — Describe the problem in your own words
2. **Gather Evidence** — Collect logs, metrics, configs, screenshots (copilot suggests what to gather)
3. **Form Hypothesis** — Based on evidence, propose possible root causes
4. **Validate Hypothesis** — Test each hypothesis with specific commands or checks
5. **Recommend Remediation** — Propose specific fix actions with risk assessment
6. **Verify Result** — Confirm the fix worked with verification steps
7. **Document Findings** — Auto-generate session summary for documentation

**Acceptance Criteria:**
- [ ] All 7 workflow steps are implemented and displayed in the chat interface
- [ ] Visual progress indicator shows current step (e.g., "Step 3 of 7: Form Hypothesis")
- [ ] Copilot guides the engineer through each step with specific prompts and suggestions
- [ ] Engineer can skip steps, revisit previous steps, or stop the workflow at any time
- [ ] Copilot adapts to the engineer's pace (doesn't rush through steps)
- [ ] Each step shows expected evidence or outcome before proceeding
- [ ] Completed workflow is persisted to the workspace session history

**Dependencies:** EPIC-2 (AI Chat Interface), EPIC-7 (MCP Skills)

---

### EPIC-9: Workspace System
**Phase:** 0, 1  
**Summary:** Customer-specific workspace containing all context, knowledge, and history.

#### EPIC-9.1 — Create Workspace
**Priority:** P0 | **Complexity:** S  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to create a workspace for each customer so that I can keep work organized and isolated.

**Acceptance Criteria:**
- [ ] User can create a workspace with a name, description, and tech stack tags
- [ ] Workspace name is unique (validated)
- [ ] Default workspace is created on first launch ("Default")
- [ ] Workspace creation is instant (no loading state > 1 second)

**Dependencies:** None

---

#### EPIC-9.2 — Tech Stack Tags
**Priority:** P0 | **Complexity:** S  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want to tag my workspace with applicable technologies so that the copilot knows which skills to prioritize.

**Acceptance Criteria:**
- [ ] User can select multiple tech stack tags when creating or editing a workspace
- [ ] Available tags: Linux, Windows, OpenShift, VMware, Ansible, Nagios, MySQL, PostgreSQL, MSSQL, etc.
- [ ] Tech stack tags influence skill context injection (skills matching the tech stack are prioritized)
- [ ] Tags are displayed in the workspace info panel

**Dependencies:** EPIC-9.1 (Create Workspace)

---

#### EPIC-9.3 — Workspace Switching
**Priority:** P0 | **Complexity:** S  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to switch between workspaces so that I can quickly change context.

**Acceptance Criteria:**
- [ ] Workspace switcher in sidebar lists all workspaces with names and last-active timestamps
- [ ] Switching workspaces loads all knowledge, sessions, and settings for that workspace
- [ ] Switching is fast (< 1 second for workspaces with < 100 documents)
- [ ] Current workspace is highlighted in the switcher
- [ ] Knowledge, sessions, and recommendations are isolated per workspace (no cross-contamination)

**Dependencies:** EPIC-9.1 (Create Workspace), EPIC-5 (Knowledge System), EPIC-2.6 (Conversation History)

---

#### EPIC-9.4 — Workspace Notes
**Priority:** P0 | **Complexity:** S  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to add free-form notes to my workspace so that I can capture context the copilot might not infer.

**Acceptance Criteria:**
- [ ] User can add, edit, and delete notes in a workspace
- [ ] Notes are plain text with markdown support
- [ ] Notes are included in the context when the copilot answers questions about the workspace
- [ ] Notes are stored in encrypted local storage
- [ ] Notes panel shows all notes for the current workspace in a list

**Dependencies:** EPIC-9.3 (Workspace Switching)

---

#### EPIC-9.5 — Session History
**Priority:** P0 | **Complexity:** S  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want to see previous chat sessions so that I can revisit previous discussions.

**Acceptance Criteria:**
- [ ] Session history panel shows all sessions for the current workspace
- [ ] Each session shows: title (auto-generated from first message), last message preview, timestamp, message count
- [ ] Clicking a session restores the full conversation in the chat window
- [ ] Sessions can be renamed and deleted
- [ ] Session history persists across app restarts

**Dependencies:** EPIC-9.3 (Workspace Switching), EPIC-2.6 (Conversation History)

---

### EPIC-10: Security
**Phase:** 0, 1  
**Summary:** Security and privacy infrastructure for the copilot.

#### EPIC-10.1 — Local Encryption
**Priority:** P0 | **Complexity:** M  
**Labels:** infrastructure  
**Phase:** 0

> As an engineer, I want all my data encrypted at rest so that it's protected even if my device is compromised.

**Acceptance Criteria:**
- [ ] All local data (knowledge documents, sessions, notes) is encrypted with AES-256-GCM
- [ ] Encryption key is derived from OS-level key material (not user-defined password)
- [ ] Known files are scanned and verified as encrypted
- [ ] Unencrypted data is never written to disk (even temporarily, except in memory)
- [ ] Encryption is transparent to the user (no manual encryption steps)
- [ ] Encryption key change on app update does not require re-encryption of all data

**Dependencies:** None

---

#### EPIC-10.2 — Credential Manager Integration
**Priority:** P0 | **Complexity:** L  
**Labels:** infrastructure  
**Phase:** 1

> As an engineer, I want my credentials stored in the OS credential manager so that they're protected by the OS.

**Acceptance Criteria:**
- [ ] Windows: API keys and credentials are stored in Windows Credential Manager (vault)
- [ ] macOS: API keys and credentials are stored in macOS Keychain
- [ ] Credential retrieval is automatic on app startup (user doesn't need to re-enter keys)
- [ ] If credential manager is unavailable, user is prompted to store credentials securely
- [ ] No plaintext credentials in any file, log, or memory dump
- [ ] Credentials can be rotated by updating them in settings

**Dependencies:** EPIC-10.1 (Local Encryption)

---

#### EPIC-10.3 — No Plaintext Credentials
**Priority:** P0 | **Complexity:** S  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want assurance that my credentials are never stored in plaintext so that I can trust the copilot with sensitive information.

**Acceptance Criteria:**
- [ ] Code review confirms no plaintext credential storage anywhere in the codebase
- [ ] No credentials in: config files, logs, cache files, temporary files, memory dumps
- [ ] No credentials printed in console output or error messages
- [ ] Credential masking in UI (e.g., `sk-****abc123` instead of full key)
- [ ] Automated test verifies no hardcoded credentials in source code
- [ ] Security audit report included in release documentation

**Dependencies:** EPIC-10.1 (Local Encryption), EPIC-10.2 (Credential Manager Integration)

---

#### EPIC-10.4 — Audit Log
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As an engineer, I want an audit log of significant actions so that I can track what the copilot has done.

**Acceptance Criteria:**
- [ ] Audit log records: timestamp, action, user, outcome, context (skill, workspace)
- [ ] Logged actions: skill activations, knowledge ingestion, credential access, settings changes, workspace changes, update installations
- [ ] Audit log does NOT include: terminal commands, screen content, API keys, or raw credentials
- [ ] Audit log is stored in encrypted local storage
- [ ] Engineer can view the audit log in the Logs panel
- [ ] Audit log is exported in standard format (JSON) for compliance purposes

**Dependencies:** EPIC-10.1 (Local Encryption)

---

### EPIC-11: Update Mechanism
**Phase:** 0, 1  
**Summary:** Desktop application update delivery.

#### EPIC-11.1 — Auto-Update Mechanism
**Priority:** P0 | **Complexity:** M  
**Labels:** feature  
**Phase:** 0

> As an engineer, I want the copilot to check for and install updates automatically so that I'm always running the latest version.

**Acceptance Criteria:**
- [ ] Update check on app startup and manually via Settings
- [ ] New version download happens in background without disrupting work
- [ ] User is notified of available update with changelog summary
- [ ] User confirms before installing the update
- [ ] Update installs and app restarts automatically after confirmation
- [ ] Configuration, knowledge, and sessions are preserved across updates
- [ ] Failed update is rolled back automatically (app returns to previous working version)
- [ ] Update mechanism works on both Windows and macOS

**Dependencies:** None (Tauri has built-in update plugin support)

---

### EPIC-12: Infrastructure and DevOps
**Phase:** 0  
**Summary:** Build systems, CI/CD, testing infrastructure, and developer tooling.

#### EPIC-12.1 — CI/CD Pipeline
**Priority:** P0 | **Complexity:** L  
**Labels:** infrastructure  
**Phase:** 0

> As a developer, I want an automated CI/CD pipeline so that builds, tests, and releases are reliable and reproducible.

**Acceptance Criteria:**
- [ ] GitHub Actions workflows for: lint, test, build (Windows), build (macOS), release
- [ ] Frontend tests run on every push (Vitest)
- [ ] Rust tests run on every push (cargo test)
- [ ] Desktop builds run on every push and every PR
- [ ] MSI and DMG artifacts are produced per build
- [ ] Code signing is configured for Windows (EV certificate) and macOS (Developer ID)
- [ ] Release workflow cuts a tagged release and creates a GitHub Release with assets

**Dependencies:** None

---

#### EPIC-12.2 — Testing Framework
**Priority:** P0 | **Complexity:** M  
**Labels:** infrastructure  
**Phase:** 0

> As a developer, I want a testing framework so that I can verify the copilot works correctly before shipping.

**Acceptance Criteria:**
- [ ] Vitest for frontend unit and integration tests
- [ ] cargo test for Rust unit tests
- [ ] Playwright for E2E tests on critical user flows (install, chat, settings, workspace)
- [ ] Skill tests run as part of CI (automated test per skill)
- [ ] Test coverage gate: ≥ 80% on changed lines (CI Lite)
- [ ] Local test scripts available for each component

**Dependencies:** EPIC-12.1 (CI/CD Pipeline)

---

#### EPIC-12.3 — Onboarding Experience
**Priority:** P1 | **Complexity:** M  
**Labels:** feature  
**Phase:** 1

> As a new user, I want an onboarding experience so that I can get started with the copilot without reading documentation.

**Acceptance Criteria:**
- [ ] First-launch wizard guides through: create workspace, configure AI provider, accept privacy policy
- [ ] In-app tips and tooltips explain key features on first use
- [ ] Example workflow is demonstrated on first use (guided through a sample troubleshooting scenario)
- [ ] Getting started checklist in sidebar: "Complete your setup" with steps
- [ ] Onboarding is skip-able

**Dependencies:** EPIC-9.1 (Create Workspace), EPIC-1.3 (Settings Panel)

---

#### EPIC-12.4 — Documentation
**Priority:** P1 | **Complexity:** S  
**Labels:** infrastructure  
**Phase:** 0

> As a user, I want documentation so that I can understand how to use the copilot.

**Acceptance Criteria:**
- [ ] README.md in the repository with installation, usage, and contributing instructions
- [ ] In-app help documentation (markdown files bundled with the app)
- [ ] Quick-start guide (1-page PDF or web page)
- [ ] FAQ section addressing common questions (privacy, data handling, supported platforms)
- [ ] Troubleshooting guide for common issues

**Dependencies:** None

---

## Backlog Legend

### Priority-to-Phase Mapping

| Priority | Typical Phase |
|----------|--------------|
| P0 | Phase 0-1 (MVP) |
| P1 | Phase 1-2 (MVP+) |
| P2 | Phase 2 (Expansion) |
| P3 | Phase 3+ (Maturity) |

### Story Count by Phase

| Phase | Stories | Epic Count |
|-------|---------|------------|
| Phase 0 | 18 | 6 |
| Phase 1 | 24 | 7 |
| Phase 2 | TBD (not detailed in MVP backlog) | TBD |
| Phase 3+ | TBD (not detailed in MVP backlog) | TBD |
| **Total (MVP)** | **42** | **13** |

### Labels Distribution

| Label | Count |
|-------|-------|
| `feature` | 30 |
| `infrastructure` | 8 |
| `improvement` | 0 (will emerge during development) |
| `technical-debt` | 0 (will emerge during development) |

---

*See also: [ROADMAP.md](./ROADMAP.md) · [MVP_SCOPE.md](./MVP_SCOPE.md) · [RELEASE_PLAN.md](./RELEASE_PLAN.md)*