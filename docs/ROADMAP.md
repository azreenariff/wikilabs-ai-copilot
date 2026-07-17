# Wiki Labs AI Copilot — Roadmap

## Phase 1 — Foundation ✅

- [x] Project structure and Cargo workspace
- [x] Core data types (chat, tools, skills, workspace, knowledge, intent)
- [x] Persistence layer with database, schema, and migrations
- [x] Application entry point

## Phase 2 — MCP Protocol ✅

- [x] MCP Registry — Tool and skill discovery
- [x] MCP Skill Manager — Skill lifecycle
- [x] MCP Transport — Protocol transport layer
- [x] MCP Protocol — Message types

## Phase 3 — Observation & Intent ✅

- [x] Observation utilities
- [x] Intent recognition

## Phase 4 — Knowledge Base ✅

- [x] Knowledge management
- [x] Vector storage and retrieval

## Phase 5 — AI Runtime ✅

- [x] Provider abstraction (OpenAI, vLLM, custom)
- [x] Streaming responses with cancellation
- [x] Conversation Manager with CRUD
- [x] Context Manager with multi-source aggregation
- [x] Prompt Manager with templates and versioning
- [x] Engineering Persona
- [x] Workspace Context
- [x] Memory Architecture
- [x] Token Budget Manager
- [x] Manual Context Selection
- [x] 148+ unit tests

## Phase 6 — UI Layer (Planned)

- [ ] CLI interface with interactive prompt
- [ ] Web-based chat interface
- [ ] Real-time streaming UI updates
- [ ] Conversation management UI
- [ ] Settings/configuration panel
- [ ] Theme customization

## Phase 7 — Enhanced AI Capabilities (Planned)

- [ ] Function/tool calling with automatic execution
- [ ] Multi-model routing (fast model for simple tasks, strong model for complex)
- [ ] Image/Vision support
- [ ] Structured/JSON output support
- [ ] Tool result summarization
- [ ] Self-correction and retry loops
- [ ] Multi-turn tool orchestration

## Phase 8 — Advanced Features (Planned)

- [ ] Real-time collaboration
- [ ] Shared knowledge base across team
- [ ] Custom persona definitions
- [ ] Prompt template marketplace
- [ ] Analytics and usage reporting
- [ ] Plugin/extension system
- [ ] Multi-language support

## Phase 9 — Integration (Planned)

- [ ] Git integration (PR reviews, commit messages)
- [ ] CI/CD integration
- [ ] Slack/Discord bot mode
- [ ] IDE extensions (VS Code, JetBrains)
- [ ] GitHub integration

## Technical Debt

- [ ] Improve token counting accuracy (replace char-based heuristic)
- [ ] Add comprehensive integration tests
- [ ] Performance optimization for large workspaces
- [ ] Add telemetry/analytics
- [ ] Docker containerization
- [ ] Documentation site with search