# Wiki Labs AI Copilot

**An AI-powered enterprise engineering assistant.**

Wiki Labs AI Copilot is a desktop application that assists engineers while they perform customer implementation, troubleshooting, and operational tasks. It observes engineer activity (with permission), understands context, and provides recommendations — but the human engineer remains responsible for all actions.

## Key Features

- **AI Chat Interface** — Streamlined chat for troubleshooting and documentation
- **Knowledge Base** — Vector + keyword search over SOPs, manuals, and documentation
- **MCP Skills** — Domain-specific expertise (OpenShift, Linux, MySQL, etc.)
- **Workspace Management** — Customer-specific technology stacks and context
- **Real-time Observation** — Terminal, app context, and clipboard awareness
- **Enterprise Security** — Data residency, encryption, and auditability

## Test Status

All 12 workspace crates compile and pass tests. Tests are written for every module, covering:
- **37 tests** — AI runtime (context window, token counting, response streaming)
- **32 tests** — Intent recognition (confidence, correction, engine patterns)
- **28 tests** — Workspace management (CRUD, switch, delete, multiple workspaces)
- **26 tests** — Security (classification, keychain, encryption, injection defense, audit)
- **21 tests** — Observation (tiers, shell, app monitor, clipboard, capture, OCR, credential filter)
- **16 tests** — Knowledge (document types, search, embedding, import, dedup, quality)
- **10 tests** — Persistence (SQLite repository CRUD, migrations)
- **8 tests** — Data types (chat messages, AI requests, tools, workspace config)
- **3 tests** — Testing utilities (mocks, fixtures)
- **0 tests** — MCP crates (bridge, registry, skill manager — integration tested at runtime)

## Architecture

See [docs/architecture/](docs/architecture/) for the full architecture specification.

## Getting Started

See [docs/development/DEVELOPMENT_SETUP.md](docs/development/DEVELOPMENT_SETUP.md) for setup instructions.

## License

Proprietary. See [LICENSE](LICENSE).