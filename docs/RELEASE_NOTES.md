# Release Notes — Wiki Labs AI Copilot v1.0.0

> General Availability (GA) Release — Tuesday, July 21, 2026

## Overview

Wiki Labs AI Copilot v1.0.0 is the first General Availability release of the enterprise engineering copilot. This release ships the complete production feature set built through 15 development phases, with professional-grade enterprise security, operations skill packs, and all foundational systems operational.

**Version:** 1.0.0 (GA)
**Build Date:** 2026-07-21
**Platform:** Windows (MSI, NSIS)
**License:** Proprietary
**Repository:** https://github.com/wikilabs/wikilabs-ai-copilot

## What's New in v1.0.0

### Desktop Application

- **Tauri v2 Desktop Shell** — Native Windows desktop application with React frontend, SQLite persistence, and WebView2 rendering
- **AI Chat Interface** — Real-time streaming chat with workspace-scoped conversations, message history, and JSON export
- **Workspace Management** — Create, switch, delete workspaces with customer-specific technology stacks and engineering context
- **Knowledge Management** — Vector + keyword hybrid search over SOPs, manuals, and documentation with import from `.wkl` archives
- **Skill Pack Management** — Full lifecycle: discover, load, validate, enable, disable, update enterprise skill packs
- **Settings System** — Profile-based settings with 8 configuration sections: AI Provider, UI, Privacy, Security, Update, Logging, Window State

### AI Engine

- **AI Provider Abstraction** — Unified provider interface supporting OpenAI, vLLM, and Ollama with health checks and feature detection
- **Conversation Manager** — Multi-conversation lifecycle with CRUD, tool call tracking, tag-based categorization, and export/restore
- **Context Manager** — Priority-based context aggregation (High/Normal/Low) with manual injection and source sorting
- **Prompt Manager** — Template-driven prompt assembly with `{{placeholder}}` syntax, versioning, and active template selection
- **Session Manager** — AI session lifecycle (Active → Paused → Suspended → Ended) with idle detection
- **Token Budget Manager** — Three policies (Strict, With Buffer, Aggressive) with intelligent trimming and budget breakdown

### Observation & Intelligence

- **Observation Engine** — Tiered observation system covering screen, terminal, app context, and clipboard (all user-controlled)
- **Technology Recognition Engine** — Evidence-based detection from URL, terminal commands, active app, file patterns across 14+ technology domains
- **Intent Recognition Engine** — Technology-aware intent classification with confidence scoring and human override
- **Engineering Workflow Engine** — Declarative state machine workflows with evidence collection, diagnosis, remediation, and verification
- **Context Fusion Engine** — Unified context combining observations, conversation, workspace, technology, and intent
- **Confidence Engine** — Confidence scoring on every inference with automatic confirmation for low-confidence detections
- **Human Feedback Loop** — Correction tracking and intent override from direct human input

### Copilot System

- **Copilot Engine** — Central orchestration of the observation → recommendation → approval loop
- **Decision Engine** — Multi-criteria recommendation visibility with 9 evaluation rules in priority order
- **Recommendation Engine** — Template-based generation with engineering context and auto-evidence
- **Policy Engine** — 5 operating modes (Minimal/Balanced/Teaching/Expert/Silent) controlling recommendation visibility
- **Lifecycle Manager** — Recommendation state machine (Candidate → Ready → Displayed → Accepted → Completed)
- **Session Memory** — Engineer interaction tracking for personalization (acceptance rate, dismissal reasons, corrections)
- **Explainability Engine** — Traceable reasoning with reason trees, evidence mapping, and human-readable explanations
- **Human Approval System** — Approval lifecycle (Pending → Approved/Denied/AutoApproved) with audit trail
- **Proactive Assistance** — 5 interruption signal types with flooding prevention and urgency classification

### Guidance Engine

- **Guidance Panel** — Tauri-native sidebar panel for context-aware engineering guidance
- **Session Context Provider** — Session-level context: current task, duration, prior decisions, pending approvals
- **Skill Context Provider** — Skill-level context: active technologies, available commands, workflow state
- **Cross-Skill Context Provider** — Multi-skill interactions, shared state, cross-cutting concerns
- **Guidance Item Types** — Recommendations, warnings, suggestions, tips, explanations with priority levels

### Security Module

- **AES-256-GCM and ChaCha20-Poly1305 Encryption** — Configurable encryption for API keys and credentials
- **Windows Credential Manager** — Native OS credential storage (via DPAPI on Windows)
- **Key Derivation** — System fingerprint + optional PIN for key derivation (SHA-256 based)
- **Secret Redaction** — Automatic redaction of sensitive data in logs (API keys, tokens, passwords, secrets)
- **Privacy Controls** — Per-feature toggles for screen observation, OCR, clipboard, diagnostics, telemetry
- **Privacy Mode** — One-click disable of all observation and collection
- **Certificate Validation** — TLS endpoint validation utility
- **Threat Model** — Documented threat model covering data in transit, data at rest, and screen observation

### Error Handling & Reliability

- **Error Handler** — Global error handling with severity levels (Warning, Degraded, Error, Fatal)
- **Crash Reports** — Automatic crash report collection with diagnostic context
- **Graceful Degradation** — Feature-level fallback when individual components fail
- **Graceful Shutdown** — Registered shutdown handlers for clean resource cleanup
- **Panic Hook** — Custom panic handler that captures stack traces and creates crash reports
- **Recovery Strategies** — Configurable strategies: Retry, Fallback, UserPrompt, Shutdown

### Logging & Diagnostics

- **Structured Logging** — tracing-based JSON + console logging with configurable levels
- **Log Rotation** — Daily rotation with configurable max file size (10 MB default) and retention (3 files default)
- **Diagnostic Packages** — Auto-generated diagnostic bundles with redacted settings and log file info
- **Log Redaction** — Automatic redaction of sensitive fields (password, secret, token, api_key, authorization)

### Operations Skill Packs (Phase 15)

New skill packs added in this release:

| Skill Pack | Files | Subdirectories | Version-Aware |
|------------|-------|---------------|---------------|
| Nagios XI | 19 | 13 | 2024/2025 |
| Nagios Log Server | 20 | 13 | Yes |
| Checkmk | 21 | 13 | 2.3/2.4 |
| Ansible | 20 | 13 | Yes |
| MySQL | 41 | 14 | 8.0 |
| EDB PostgreSQL | 34 | 14 | 15/16 |
| Microsoft SQL Server | 28 | 13 | 2022 |

Each skill pack includes:
- Architecture documentation
- Best practices
- Common failures & detection rules
- Diagnostic guides
- Command references
- Knowledge bases
- Reasoning guides
- Workflow definitions
- Quality standard documentation

### Engineering Skill Packs (Prior Phases)

| Skill Pack | Files | Technology |
|------------|-------|------------|
| OpenShift | 40 | Red Hat OpenShift 4.x |
| Linux Engineering | 40 | Linux Administration |
| VMware vSphere | 40 | VMware vSphere |

### Framework & Quality

- **Confidence & Evidence Engine** — Every recommendation includes observation, interpretation, recommendation, evidence, confidence score
- **AI Safety Framework** — Copilot never fabricates observations; clear distinction between facts, hypotheses, and unknowns
- **Version Awareness** — Product version awareness for version-specific guidance across all skill packs
- **Cross-Skill Workflows** — Multi-technology troubleshooting workflows with skill collaboration
- **Operations Quality Standard** — Quality requirements for all Operations Skill Packs

## Installation

### System Requirements

| Requirement | Minimum | Recommended |
|------------|---------|-------------|
| OS | Windows 10 (64-bit) | Windows 11 (64-bit) |
| RAM | 4 GB | 8 GB |
| Disk | 2 GB free | 5 GB free |
| .NET | .NET Desktop Runtime 8.0 | .NET Desktop Runtime 8.0 (latest) |
| WebView2 | WebView2 Runtime | WebView2 Runtime (pre-installed on modern Windows) |

### Installer Formats

- **MSI** — Enterprise deployment, Group Policy, SCCM compatible
- **NSIS (.exe)** — Standard user installation

### Installation Paths

- **Application Data:** `%APPDATA%\com.wikilabs.copilot\`
- **Database:** `%APPDATA%\com.wikilabs.copilot\wikilabs.db`
- **Logs:** `%APPDATA%\com.wikilabs.copilot\logs\`
- **Crash Reports:** `%APPDATA%\com.wikilabs.copilot\crash\`

## Upgrading

### From Previous Versions

The auto-update system (tauri-plugin-updater) handles version upgrades:

1. When a new version is available, a dialog appears automatically
2. Click **Download and Install** to proceed
3. The application restarts with the new version
4. Settings and data are preserved across upgrades

### Manual Upgrade

1. Download the latest MSI or NSIS installer
2. Run the installer — it detects existing installations
3. Click **Upgrade** to upgrade in place
4. All settings, workspaces, and knowledge are preserved

### Uninstall & Clean Install

1. Uninstall via **Settings → Apps → Installed Apps** (Windows)
2. Or run the installer with **Repair** to fix corrupted installations
3. Manual cleanup: delete `%APPDATA%\com.wikilabs.copilot\` (caution: removes all data)

## Architecture

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | Tauri v2 (Rust + WebView2) |
| Frontend | React + TypeScript |
| Core Language | Rust 2021 Edition |
| Database | SQLite + rusqlite (bundled) |
| Vector Search | SQLite VSS extension |
| Local Embedding | all-MiniLM-L6-v2 (ONNX Runtime) |
| AI Providers | OpenAI, vLLM, Ollama |
| Logging | tracing + tracing-subscriber |
| CI/CD | GitHub Actions |

### Data Storage

All data stored in a single SQLite database at `%APPDATA%\com.wikilabs.copilot\wikilabs.db`:

- Workspaces and configuration
- Chat history (per workspace)
- Knowledge documents and chunks (VSS indexed, 384-dim embeddings)
- Audit log entries (hash-chain signed)
- Credential hashes (referenced from OS keychain)

### Database Schema

**Tables:**
- `workspaces` — Workspace metadata (id, name, customer_name, technology_stack)
- `chat_messages` — Per-workspace conversation history
- `knowledge_documents` — Knowledge document metadata
- `knowledge_chunks` — Indexed chunks with embeddings
- `audit_log` — Hash-chain audit trail

## Supported AI Providers

| Provider | Endpoint | API Key | Notes |
|----------|----------|---------|-------|
| OpenAI | `https://api.openai.com/v1` | Required | Standard GPT models |
| vLLM | `http://localhost:8000/v1` | Optional | Self-hosted OpenAI-compatible |
| Ollama | `http://localhost:11434/v1` | Optional | Local model serving |

## Configuration Settings

The application settings system manages 8 configuration sections:

### 1. AI Provider
- Provider name, endpoint, API key, model, max tokens, context window

### 2. UI Settings
- Theme (dark/light/system), font size, zoom level, language, minimize to tray, shortcuts help

### 3. Privacy Settings
- Screen observation, OCR, clipboard observation, diagnostics, telemetry, consent, privacy mode

### 4. Security Settings
- Windows Credential Manager, local encryption, encryption algorithm (AES-256-GCM/ChaCha20), auto-lock, PIN protection

### 5. Update Settings
- Auto-check, channel (stable/preview/internal), show dialog, allow deferral

### 6. Logging Settings
- Log level, file logging, max log size, max log files, structured logging

### 7. Window Settings
- Dimensions, position, maximized state, last workspace, active panel

### 8. Profile Settings
- Named profiles with independent settings, current profile, created/updated timestamps

## Known Issues

See [Known Limitations](KNOWN_LIMITATIONS.md) for detailed documentation.

## Support

- **Documentation:** https://github.com/wikilabs/wikilabs-ai-copilot/tree/main/docs
- **Issues:** https://github.com/wikilabs/wikilabs-ai-copilot/issues
- **Support Guide:** [docs/SUPPORT_GUIDE.md](SUPPORT_GUIDE.md)

---

*Thank you for choosing Wiki Labs AI Copilot.*