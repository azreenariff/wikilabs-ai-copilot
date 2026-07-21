# Wiki Labs AI Copilot v1.0.0

**An AI-powered enterprise engineering assistant.**

Wiki Labs AI Copilot is a desktop application that assists engineers while they perform customer implementation, troubleshooting, and operational tasks. It observes engineer activity (with permission), understands context, and provides recommendations — but the human engineer remains responsible for all actions.

## Key Features

- **AI Chat Interface** — Streamlined chat for troubleshooting and documentation with real-time streaming
- **Knowledge Base** — Vector + keyword search over SOPs, manuals, and documentation
- **MCP Skills** — Domain-specific expertise (OpenShift, Linux, MySQL, etc.)
- **Workspace Management** — Customer-specific technology stacks and context
- **Real-time Observation** — Terminal, app context, and clipboard awareness
- **Enterprise Security** — AES-256-GCM encryption, credential manager, audit trail
- **Guidance Engine** — Proactive, context-aware engineering recommendations
- **Skill Packs** — Extensible technology-specific expertise packs

## Documentation

### User Documentation

| Document | Description |
|----------|-------------|
| [Quick Start Guide](docs/QUICK_START.md) | 5-minute getting started guide |
| [User Guide](docs/user-guide/USER_GUIDE.md) | Complete user manual |
| [Administrator Guide](docs/admin-guide/ADMINISTRATOR_GUIDE.md) | System administration and deployment |
| [Installation Guide](docs/INSTALLATION_GUIDE.md) | Windows installation, upgrade, repair, uninstall |
| [Troubleshooting Guide](docs/TROUBLESHOOTING.md) | Common issues and recovery procedures |
| [FAQ](docs/FAQ.md) | Frequently asked questions |
| [Known Limitations](docs/KNOWN_LIMITATIONS.md) | Current limitations and workarounds |
| [Support Guide](docs/SUPPORT_GUIDE.md) | How to get help and support channels |

### Technical Documentation

| Document | Description |
|----------|-------------|
| [Architecture Guide](docs/ARCHITECTURE_GUIDE.md) | System architecture and components |
| [Security Guide](docs/SECURITY_GUIDE.md) | Security model, threat model, encryption |
| [Operations Guide](docs/OPERATIONS_GUIDE.md) | Monitoring, logging, maintenance |
| [Developer Guide](docs/DEVELOPER_GUIDE.md) | Development workflow, contributing |
| [Release Notes](docs/RELEASE_NOTES.md) | v1.0.0 release notes |
| [Skill Pack Development Guide](docs/SKILL_PACK_DEVELOPMENT_GUIDE.md) | Creating and distributing skill packs |

### Product & Planning

| Document | Description |
|----------|-------------|
| [Vision](docs/product/VISION.md) | Product vision |
| [Specification](docs/product/PRODUCT_SPEC.md) | Product specification |
| [User Stories](docs/product/USER_STORIES.md) | User stories |
| [Roadmap](docs/planning/ROADMAP.md) | Product roadmap |
| [Backlog](docs/planning/BACKLOG.md) | Product backlog |
| [MVP Scope](docs/planning/MVP_SCOPE.md) | MVP scope definition |
| [Release Plan](docs/planning/RELEASE_PLAN.md) | Release plan |

### Architecture & Design

| Document | Description |
|----------|-------------|
| [System Architecture](docs/architecture/ARCHITECTURE.md) | System architecture overview |
| [Component Design](docs/architecture/COMPONENT_DESIGN.md) | Component design details |
| [Data Model](docs/architecture/DATA_MODEL.md) | Data model definitions |
| [Security Model](docs/architecture/SECURITY_MODEL.md) | Security model |
| [ADR Index](docs/adr/README.md) | Architecture Decision Records |

### Development

| Document | Description |
|----------|-------------|
| [Development Setup](docs/development/DEVELOPMENT_SETUP.md) | Setting up the development environment |
| [Coding Standards](docs/development/CODING_STANDARDS.md) | Coding standards and conventions |
| [Testing Guide](docs/development/TESTING_GUIDE.md) | Testing guide and conventions |

### AI Runtime

| Document | Description |
|----------|-------------|
| [AI Runtime Overview](docs/ai/AI_RUNTIME.md) | AI runtime overview |
| [Provider](docs/ai/AI_PROVIDER.md) | Provider abstraction |
| [Conversation Manager](docs/ai/CONVERSATION_MANAGER.md) | Conversation management |
| [Prompt Manager](docs/ai/PROMPT_MANAGER.md) | Prompt management |
| [Context Management](docs/ai/CONTEXT_MANAGEMENT.md) | Context management |

## Getting Started

### Prerequisites

- **Rust** — Latest stable (https://rustup.rs)
- **.NET Desktop Runtime 8.0** — Required for Tauri (https://dotnet.microsoft.com/download)
- **WebView2 Runtime** — Included with installer (https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Build the Project

```bash
# Clone the repository
git clone https://github.com/wikilabs/wikilabs-ai-copilot.git
cd wikilabs-ai-copilot

# Build the project
cargo build --release

# Run the application
cargo tauri dev
```

### System Requirements

| Requirement | Minimum | Recommended |
|------------|---------|-------------|
| OS | Windows 10 64-bit | Windows 11 64-bit |
| RAM | 4 GB | 8 GB |
| Disk | 2 GB free | 5 GB free |

## License

Proprietary. See [LICENSE](LICENSE).