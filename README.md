# Wiki Labs AI Copilot

**An AI-powered enterprise engineering assistant.**

Wiki Labs AI Copilot is a desktop application that assists engineers while they perform customer implementation, troubleshooting, and operational tasks. It observes engineer activity (with permission), understands context, and provides recommendations — but the human engineer remains responsible for all actions.

## Key Features

- **AI Chat Interface** — Streamlined chat for troubleshooting and documentation with real-time streaming
- **Knowledge Base** — Vector + keyword search over SOPs, manuals, and documentation
- **MCP Skills** — Domain-specific expertise (OpenShift, Linux, MySQL, etc.)
- **Workspace Management** — Customer-specific technology stacks and context
- **Real-time Observation** — Terminal, app context, and clipboard awareness
- **Enterprise Security** — Data residency, encryption, and auditability

## Documentation

See the [Documentation Index](docs/INDEX.md) for full navigation.

| Category | Docs |
|---|---|
| Product | [Vision](docs/product/VISION.md), [Specification](docs/product/PRODUCT_SPEC.md), [User Stories](docs/product/USER_STORIES.md) |
| AI Runtime | [Overview](docs/ai/AI_RUNTIME.md), [Provider](docs/ai/AI_PROVIDER.md), [Conversation](docs/ai/CONVERSATION_MANAGER.md), [Prompt](docs/ai/PROMPT_MANAGER.md), [Context](docs/ai/CONTEXT_MANAGEMENT.md) |
| User Guide | [Getting Started](docs/user-guide/USER_GUIDE.md) |
| Planning | [Roadmap](docs/planning/ROADMAP.md), [Backlog](docs/planning/BACKLOG.md), [MVP Scope](docs/planning/MVP_SCOPE.md), [Release Plan](docs/planning/RELEASE_PLAN.md) |
| Architecture | [System](docs/architecture/ARCHITECTURE.md), [Components](docs/architecture/COMPONENT_DESIGN.md), [Data Model](docs/architecture/DATA_MODEL.md), [Security](docs/architecture/SECURITY_MODEL.md) |
| Development | [Setup](docs/development/DEVELOPMENT_SETUP.md), [Standards](docs/development/CODING_STANDARDS.md), [Testing](docs/development/TESTING_GUIDE.md) |
| ADRs | [Index](docs/adr/README.md) |

## Getting Started

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/wikilabs/wikilabs-ai-copilot.git
cd wikilabs-ai-copilot

# Build the project
cargo build --release
```

## License

Proprietary. See [LICENSE](LICENSE).