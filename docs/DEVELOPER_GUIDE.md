# Developer Guide — Wiki Labs AI Copilot v1.0.0

> Development workflow, architecture, and contributing guidelines.

## Table of Contents

1. [Overview](#overview)
2. [Development Environment Setup](#development-environment-setup)
3. [Project Structure](#project-structure)
4. [Building the Project](#building-the-project)
5. [Running Tests](#running-tests)
6. [Development Workflow](#development-workflow)
7. [Code Style & Standards](#code-style--standards)
8. [Architecture Overview](#architecture-overview)
9. [Key Components](#key-components)
10. [Adding a New Skill Pack](#adding-a-new-skill-pack)
11. [Contributing](#contributing)
12. [Release Process](#release-process)

## Overview

Wiki Labs AI Copilot is a Rust-based Tauri v2 desktop application that assists enterprise infrastructure engineers. It provides AI-powered chat, knowledge management, skill packs, and real-time observation — all running as a native Windows desktop application.

**Tech Stack:**
- **Desktop Framework:** Tauri v2 (Rust + WebView2)
- **Frontend:** React + TypeScript
- **Core:** Rust 2021 Edition
- **Database:** SQLite + rusqlite (bundled)
- **AI:** OpenAI-compatible provider abstraction
- **Packaging:** MSI + NSIS for Windows

## Development Environment Setup

### Prerequisites

| Component | Version | Installation |
|-----------|---------|-------------|
| Rust | 1.77+ | `rustup install stable` |
| Cargo | Included with Rust | Included with rustup |
| Node.js | 18+ | https://nodejs.org/ |
| npm | 9+ | Included with Node.js |
| .NET Desktop Runtime 8.0 | 8.0 | https://dotnet.microsoft.com/download |
| WebView2 Runtime | Latest | Pre-installed on modern Windows |

### Initial Setup

```bash
# 1. Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable
rustup component add clippy rustfmt

# 2. Clone the repository
git clone https://github.com/wikilabs/wikilabs-ai-copilot.git
cd wikilabs-ai-copilot

# 3. Verify Rust installation
rustc --version  # Should be 1.77+
cargo --version

# 4. Build the project
cargo build --all

# 5. Run the workspace tests
cargo test --workspace

# 6. Run linters
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
```

### IDE Setup

#### Visual Studio Code

Recommended extensions:
- **rust-analyzer** — Language server for Rust
- **CodeLLDB** — Debugger for Rust
- **even-better-toml** — TOML syntax support
- **Markdown All in One** — Markdown editing

#### IntelliJ IDEA

Install the **Rust** plugin from JetBrains Marketplace.

### Environment Variables

No environment variables are required to build or run the core engine. Some features (Tauri frontend, specific AI providers) may require additional setup.

## Project Structure

```
wikilabs-ai-copilot/
├── Cargo.toml                    # Workspace root
├── README.md                     # Project overview
├── CHANGELOG.md                  # Release history
├── CONTRIBUTING.md               # Contribution guidelines
│
├── src/                          # Rust workspace crates
│   ├── core/                     # Shared types and persistence
│   │   ├── data_types/           # Domain models
│   │   └── persistence/          # SQLite layer
│   ├── ai/                       # AI provider abstraction
│   │                           # Provider trait, OpenAI-compatible impl
│   ├── mcp/                      # MCP protocol and skill runtime
│   │   ├── skill_manager/        # Consolidated skill engine
│   │   └── registry/             # Tool catalog
│   ├── knowledge/                # Vector + keyword search
│   ├── observation/              # Tiered observation engine
│   ├── intent/                   # Intent recognition
│   ├── workspace/                # Workspace management
│   ├── security/                 # Keychain, encryption, audit
│   ├── testing/                  # Test utilities
│   ├── technology_recognition/   # Technology detection
│   ├── engineering_timeline/     # Activity tracking
│   ├── workflow_engine/          # State machine workflows
│   ├── recommendation_readiness/# Advice readiness assessment
│   ├── human_feedback/           # Human correction handling
│   ├── context_fusion/           # Context aggregation
│   ├── skill_runtime/            # Skill discovery/loading
│   ├── skill_sdk/                # Skill Development Kit
│   ├── skill_discovery/          # Skill discovery engine
│   ├── skill_activation/         # Skill activation engine
│   ├── intelligence_engine/      # Cross-cutting intelligence
│   ├── copilot/                  # Copilot orchestration
│   └── guidance/                 # Guidance engine
│
├── src-tauri/                    # Tauri desktop application
│   ├── src/
│   │   ├── main.rs               # Application entry point
│   │   ├── config.rs             # Settings management
│   │   ├── security.rs           # Encryption & credentials
│   │   ├── logging.rs            # Structured logging
│   │   ├── error_handling.rs     # Crash recovery
│   │   ├── guidance_panel.rs     # Guidance UI commands
│   │   ├── knowledge_panel.rs    # Knowledge UI commands
│   │   └── skill_management.rs   # Skills UI commands
│   ├── tauri.conf.json           # Tauri configuration
│   ├── Cargo.toml                # Desktop crate dependencies
│   ├── build.rs                  # Build script
│   ├── capabilities/             # Tauri capabilities
│   ├── icons/                    # Application icons
│   └── gen/                      # Generated types
│
├── src/skills/                   # Skill packs (loaded at runtime)
│   ├── openshift-skill-pack/     # Red Hat OpenShift 4.x
│   ├── linux-engineering/        # Linux administration
│   ├── vmware-vsphere-skill-pack/# VMware vSphere
│   ├── mysql-skill-pack/         # MySQL DBA
│   ├── edb-postgresql-skill-pack/# EDB PostgreSQL
│   ├── mssql-skill-pack/         # Microsoft SQL Server
│   ├── nagiosxi-skill-pack/      # Nagios XI
│   ├── nagioslogserver-skill-pack/# Nagios Log Server
│   ├── checkmk-skill-pack/       # Checkmk monitoring
│   └── ansible-skill-pack/       # Ansible automation
│
├── docs/                         # Documentation
│   ├── INDEX.md                  # Documentation index
│   ├── QUICK_START.md            # 5-minute quick start
│   ├── RELEASE_NOTES.md          # Release notes
│   ├── user-guide/               # End-user documentation
│   ├── admin-guide/              # Administrator documentation
│   ├── architecture/             # Architecture specs
│   ├── development/              # Developer documentation
│   ├── security/                 # Security documentation
│   ├── planning/                 # Project planning
│   ├── product/                  # Product documentation
│   ├── ai/                       # AI runtime documentation
│   ├── operations/               # Operations documentation
│   ├── engineering-intelligence/# Engineering intelligence docs
│   ├── framework_docs/           # Framework documents
│   └── adr/                      # Architecture Decision Records
│
├── FRAMEWORK_DOCS/               # Framework documents
│   ├── AI_SAFETY.md              # AI safety framework
│   ├── CROSS_SKILL_WORKFLOWS.md  # Cross-skill workflows
│   ├── QUALITY_STANDARD.md       # Operations quality standard
│   └── VERSION_AWARENESS.md      # Version awareness system
│
└── .github/                      # GitHub CI/CD and templates
    ├── workflows/                # CI/CD workflows
    └── ISSUE_TEMPLATE/           # Issue templates
```

## Building the Project

### Build Commands

```bash
# Build all workspace crates
cargo build --all

# Build for release (optimized)
cargo build --release

# Build only the desktop application
cd src-tauri && cargo build --release && cd ..

# Clean build artifacts
cargo clean
```

### Build Output

| Target | Location |
|--------|----------|
| Debug binaries | `target/debug/` |
| Release binaries | `target/release/` |
| Tauri build (Windows) | `src-tauri/target/release/bundle/` |
| MSI installer | `src-tauri/target/release/bundle/msi/` |
| NSIS installer | `src-tauri/target/release/bundle/nsis/` |

### Building the Installer

```bash
# Build the Tauri desktop application with installer
cd src-tauri
cargo tauri build --release
cd ..
```

The installer is created in `src-tauri/target/release/bundle/`:
- MSI: `bundle/msi/`
- NSIS: `bundle/nsis/`

## Running Tests

### Workspace Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test --package wikilabs-ai

# Run tests with output (to see print statements)
cargo test --package wikilabs-ai -- --nocapture

# Run only unit tests
cargo test --workspace --lib

# Run only integration tests
cargo test --workspace --test '*'
```

### Test Organization

Tests are organized per crate in the same file as the implementation (`#[cfg(test)]` modules) and in separate `tests/` directories for integration tests.

### Test Coverage

The project includes comprehensive test suites:
- **AI Runtime:** 181 unit tests across all 12 crates
- **Copilot Engine:** 132 tests across 14 modules
- **Guidance Engine:** 132 tests
- **Knowledge Management:** Tests for document CRUD, search, embedding, import
- **Security:** Tests for classification, keychain, encryption, credentials
- **Observation:** Tests for all observation tiers
- **Intent:** Tests for model prediction, pattern matching, confidence
- **Skill System:** Tests for discovery, validation, dependency resolution

## Development Workflow

### Branch Strategy

```
main (stable)
  └── develop (integration)
        ├── feature/your-feature
        ├── fix/your-fix
        └── docs/your-docs
```

### Workflow Steps

1. **Create a feature branch:**
   ```bash
   git checkout develop
   git pull
   git checkout -b feature/my-feature
   ```

2. **Develop and test:**
   ```bash
   # Make code changes
   # Write tests for new functionality
   cargo test --workspace

   # Run linters
   cargo fmt --all
   cargo clippy --all-targets -- -D warnings
   ```

3. **Commit changes:**
   ```bash
   git add .
   git commit -m "feat: add my new feature"
   ```

4. **Push and create PR:**
   ```bash
   git push origin feature/my-feature
   # Create PR on GitHub
   ```

5. **After review and merge:**
   ```bash
   git checkout develop
   git pull
   git branch -d feature/my-feature  # Delete local branch
   ```

### Commit Message Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) convention:

```
type(scope): description

feat: add skill pack validation
fix: handle missing API key gracefully
docs: update architecture guide
test: add tests for guidance engine
chore: update dependencies
refactor: extract credential handling
```

**Types:** `feat`, `fix`, `docs`, `test`, `chore`, `refactor`, `perf`, `ci`

### Pre-Commit Checklist

- [ ] Code compiles (`cargo build --all`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Linting passes (`cargo fmt --check` and `cargo clippy`)
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Security review (if applicable)

## Code Style & Standards

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting: `cargo fmt --all`
- Run `cargo clippy` before committing: `cargo clippy --all-targets -- -D warnings`

### Error Handling

- Use `thiserror` for error types
- Use `anyhow` for application-level errors
- Never use `.unwrap()` or `.expect()` in production code
- All error paths must be handled explicitly
- Use the `?` operator for error propagation

### Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Modules | `snake_case` | `error_handling`, `skill_runtime` |
| Functions | `snake_case` | `handle_error()`, `load_skill()` |
| Types/Structs | `PascalCase` | `AppSettings`, `ErrorHandler` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_RETRIES`, `API_KEY_PATTERN` |
| Enums | `PascalCase` | `ErrorSeverity`, `RecoveryStrategy` |
| Environment vars | `UPPER_SNAKE_CASE` | `RUST_LOG` |

### Documentation

- Public APIs must have doc comments (`///`)
- Functions should have usage examples in doc comments
- Complex modules should have a module-level doc comment
- Architecture changes should be documented in ADRs

## Architecture Overview

### High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    REACT FRONTEND LAYER                       │
│  - Chat interface, streaming responses                       │
│  - Workspace selector, knowledge management                   │
│  - Skill enable/disable, settings                             │
└──────────────────────────┬───────────────────────────────────┘
                           │ Tauri IPC (commands/events)
┌──────────────────────────▼───────────────────────────────────┐
│                    RUST CORE ENGINE                           │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐   │
│  │ AI Provider │ │ Conversation│ │    Context Manager    │   │
│  │ Abstraction │ │  Manager    │ │   (priority-based)    │   │
│  └─────────────┘ └─────────────┘ └──────────────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐   │
│  │ Knowledge   │ │ Observation │ │  Technology Recogn. │   │
│  │   System    │ │  Engine     │ │      Engine          │   │
│  └─────────────┘ └─────────────┘ └──────────────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐   │
│  │ Intent      │ │ Copilot     │ │     Guidance Engine  │   │
│  │   Engine    │ │  Engine     │ │                      │   │
│  └─────────────┘ └─────────────┘ └──────────────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────────────┐   │
│  │ Security    │ │ Error       │ │     Logging           │   │
│  │   Module    │ │  Handler    │ │      System           │   │
│  └─────────────┘ └─────────────┘ └──────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │           SQLite Database (wikilabs.db)               │   │
│  │  workspaces │ chat_messages │ knowledge_documents     │   │
│  │  knowledge_chunks │ audit_log                          │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### Data Storage

All data stored in a single SQLite database at `%APPDATA%\com.wikilabs.copilot\wikilabs.db`:

| Table | Purpose |
|-------|---------|
| `workspaces` | Workspace metadata (name, customer, technology stack) |
| `chat_messages` | Per-workspace conversation history |
| `knowledge_documents` | Knowledge document metadata |
| `knowledge_chunks` | Indexed chunks with 384-dim embeddings |
| `audit_log` | Hash-chain audit trail |

### Security Architecture

- **Encryption:** AES-256-GCM or ChaCha20-Poly1305 for credentials
- **Key Derivation:** System fingerprint + optional PIN (SHA-256)
- **Credential Storage:** Windows Credential Manager (DPAPI) with local fallback
- **Log Redaction:** Automatic sensitive field redaction
- **Privacy Controls:** Per-feature toggles for observation

## Key Components

### AI Provider Abstraction (`src/ai/`)

Provides a unified interface for multiple AI providers:

```rust
pub trait AiProvider: Send + Sync {
    fn chat(&self, request: AiRequest) -> impl Future<Output = Result<ChatResponse>>;
    fn stream_chat(&self, request: AiRequest) -> impl Future<Output = Result<StreamReceiver>>;
    fn embeddings(&self, text: &str) -> impl Future<Output = Result<EmbeddingResult>>;
    fn health(&self) -> impl Future<Output = Result<()>>;
}
```

Supported providers:
- `OpenAICompatibleProvider` — OpenAI, vLLM, Ollama, any OpenAI-compatible API

### Conversation Manager (`src/ai/`)

Manages multi-conversation lifecycle:
- CRUD operations for conversations
- Message roles (user, assistant, system) with timestamps
- Tool call tracking on assistant messages
- Tag-based categorization and filtering
- Conversation summaries for listing
- JSON export and restore

### Context Manager (`src/ai/`)

Aggregates context from multiple sources:
- Priority-based context (High/Normal/Low)
- Manual context injection with tagging
- Technology stack selection
- Current activity tracking
- Fluent `ContextBuilder` for incremental construction

### Knowledge System (`src/knowledge/`)

Vector + keyword hybrid search:
- Document ingestion from `.wkl` archives
- Automatic chunking and embedding generation
- SQLite VSS extension for vector search (384-dim)
- FTS5 for full-text keyword search
- Quality scoring and deduplication

### Skill System

Three-component architecture:

1. **Skill Discovery Engine** (`src/skill_discovery/`) — Scans workspace for technology signals using glob patterns, command detection, and configuration file matching
2. **Skill Activation Engine** (`src/skill_activation/`) — Activates detected skills with dependency resolution, health monitoring, and lifecycle management
3. **Skill Runtime** (`src/skill_runtime/`) — Orchestrates full skill lifecycle: discover → load → validate → enable → activate → monitor

### Copilot Engine (`src/copilot/`)

Central orchestration of observation → recommendation → approval:
- **Decision Engine** — Multi-criteria recommendation visibility with 9 evaluation rules
- **Recommendation Engine** — Template-based generation with engineering context
- **Policy Engine** — 5 operating modes (Minimal/Balanced/Teaching/Expert/Silent)
- **Lifecycle Manager** — Recommendation state machine
- **Human Approval System** — Approval lifecycle with audit trail

### Guidance Engine (`src/guidance/`)

Context-aware engineering guidance:
- **Guidance Panel** — Tauri-native sidebar panel
- **Session Context Provider** — Current task, duration, decisions
- **Skill Context Provider** — Active technologies, available commands
- **Cross-Skill Context Provider** — Multi-skill interactions

### Security Module (`src/security/` + `src-tauri/src/security.rs`)

- AES-256-GCM / ChaCha20 encryption for API keys
- Windows Credential Manager integration
- Key derivation from system fingerprint + PIN
- Secret redaction in logs
- Certificate validation utility

### Error Handling (`src-tauri/src/error_handling.rs`)

- Global error handler with severity levels (Warning, Degraded, Error, Fatal)
- Automatic crash report collection
- Graceful degradation and shutdown
- Panic hook for crash reporting

## Adding a New Skill Pack

### Skill Pack Structure

```
my-skill-pack/
├── manifest.yaml           # Skill metadata (required)
├── technology.yaml         # Technology definitions (required)
├── detection_rules.yaml    # Detection rules (required)
├── workflows.yaml          # State machine workflows (required)
├── commands.yaml           # Technical commands (required)
├── guidance/
│   └── rules.md            # Engineering guidance rules
├── knowledge/              # Technology-specific knowledge
│   ├── architecture.md
│   └── best-practices.md
├── best-practices.md       # Best practices guide
├── known_issues.md         # Known issues and workarounds
└── README.md               # Skill pack overview
```

### Step-by-Step

1. **Create the directory:**
   ```bash
   mkdir -p src/skills/my-skill-pack
   ```

2. **Create manifest.yaml:**
   ```yaml
   id: "my-skill-pack"
   version: "1.0.0"
   name: "My Technology"
   description: "Expert knowledge for My Technology"
   technologies:
     - name: "my-technology"
       version: "1.0"
   ```

3. **Add technology definitions:**
   ```yaml
   technologies:
     - name: "my-technology"
       platform: "Linux"
       components:
         - "service-a"
         - "service-b"
   ```

4. **Add detection rules:**
   ```yaml
   detection_rules:
     - name: "terminal-command"
       type: "command"
       pattern: "my-cmd"
       confidence: 0.9
     - name: "browser-url"
       type: "url"
       pattern: "my-platform.example.com"
       confidence: 0.85
   ```

5. **Add workflows (optional):**
   ```yaml
   workflows:
     - name: "troubleshoot-issue"
       states:
         - id: "evidence_collection"
           description: "Collect diagnostic evidence"
           transitions:
             - to: "diagnosis"
               condition: "evidence sufficient"
   ```

6. **Validate the skill pack:**
   ```bash
   cargo run --bin knowledge-cli validate src/skills/my-skill-pack
   ```

7. **Add skill pack to the workspace:**
   - Place the skill pack in `src/skills/`
   - The Skill Discovery Engine will automatically detect it on startup

### SDK Commands

The Skill SDK provides commands for skill development:

```bash
# Generate a skill pack template
cargo run --bin skill-sdk create-template --type technology

# Validate a skill pack
cargo run --bin knowledge-cli validate src/skills/my-skill-pack

# Package a skill pack
cargo run --bin knowledge-cli package src/skills/my-skill-pack
```

## Contributing

### Contribution Guidelines

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed contribution guidelines.

### How to Contribute

1. **Find an issue:** Check [GitHub Issues](https://github.com/wikilabs/wikilabs-ai-copilot/issues)
2. **Claim the issue:** Comment on the issue to claim it
3. **Fork and branch:** Fork the repo and create a feature branch
4. **Develop and test:** Make your changes and add tests
5. **Submit a PR:** Open a pull request against `develop`
6. **Respond to review:** Address review comments

### Code of Conduct

- Be respectful and constructive
- Follow the established coding standards
- Write tests for new functionality
- Document public APIs
- Keep PRs focused and small

## Release Process

### Version Bumping

```bash
# Bump version in workspace Cargo.toml
# Set [workspace.package] version to the new version
```

### Release Checklist

- [ ] All tests pass (`cargo test --workspace`)
- [ ] All linting passes (`cargo fmt --check`, `cargo clippy`)
- [ ] CHANGELOG.md updated with new section
- [ ] RELEASE_NOTES.md updated
- [ ] Version number updated in `Cargo.toml` and `tauri.conf.json`
- [ ] Documentation reviewed and updated
- [ ] Security audit completed (`cargo audit`)
- [ ] Release branch created from `develop`
- [ ] PR merged to `main`
- [ ] Tag created (`v1.0.0`)
- [ ] GitHub release created with release notes
- [ ] Installers built (MSI + NSIS)
- [ ] Installers tested on clean Windows VM

### Building Release Installers

```bash
cd src-tauri
cargo tauri build --release
# MSI: target/release/bundle/msi/
# NSIS: target/release/bundle/nsis/
cd ..
```

---

*For architecture details, see [Architecture Guide](ARCHITECTURE_GUIDE.md).*
*For security details, see [Security Guide](SECURITY_GUIDE.md).*
*For skill pack development, see [Skill Pack Development Guide](SKILL_PACK_DEVELOPMENT_GUIDE.md).*