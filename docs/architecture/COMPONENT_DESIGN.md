# Component Design — Wiki Labs AI Copilot

## Component Overview

| Component | Description | Phase |
|-----------|-------------|-------|
| **Desktop App Shell** | Tauri v2 window, IPC layer, WebView | Phase 0 |
| **React Frontend** | Chat UI, workspace selector, knowledge management | Phase 0 |
| **Core Engine** | Event bus, RPC, SQLite persistence | Phase 0 |
| **AI Provider** | OpenAI, vLLM, Ollama abstraction | Phase 1 |
| **MCP Skill Runtime** | Consolidated single-process skill modules | Phase 1 |
| **Knowledge System** | SQLite VSS + FTS5 hybrid search | Phase 1 |
| **Workspace Manager** | Create, switch, delete workspaces | Phase 1 |
| **Prompt Injection Defense** | Multi-layer input/output validation | Phase 1 |
| **Observation Engine** | Shell integration, app monitor, clipboard, OCR | Phase 2 |
| **Intent Engine** | Rule-based classification with confidence | Phase 2 |
| **Skill Modules** | Domain expertise (Linux, OpenShift, MySQL, etc.) | Phase 1-3 |
| **Installer** | MSI, DMG, AppImage packaging | Phase 1 |