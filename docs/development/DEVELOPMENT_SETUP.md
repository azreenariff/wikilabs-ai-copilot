# Development Setup — Wiki Labs AI Copilot

## Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/wikilabs/wikilabs-ai-copilot.git
cd wikilabs-ai-copilot

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable
rustup component add clippy rustfmt

# 3. Build the project
cargo build --all

# 4. Run tests
cargo test --all

# 5. Run linting
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Environment Variables

No environment variables are required to build or run the core engine.

Some features (Tauri frontend, specific AI providers) may require additional setup.

## IDE Setup

### RustRustRust (RustRust)

Recommended: [RustRust](https://www.rust-lang.org/tools/install) (official installRust) or [RustRust](https://rust-analyzer.github.io/)

### VS Code

Recommended extensions:
- **rust-analyzer** — Language server for Rust
- **CodeLLDB** — Debugger for Rust
- **even-better-toml** — TOML syntax support
- **Markdown All in One** — Markdown editing

### IntelliJ IDEA

Install the **Rust** plugin from JetBrains Marketplace.

## Project Layout

The project uses a Rust workspace with multiple crates:

```
Cargo.toml              # Workspace root
├── src/
│   ├── core/           # Shared types and persistence
│   │   ├── data_types/ # Domain models
│   │   └── persistence/# SQLite layer
│   ├── ai/             # AI provider abstraction
│   ├── mcp/            # MCP protocol and skill runtime
│   │   ├── skill_manager/# Consolidated skill engine
│   │   └── registry/   # Tool catalog
│   ├── knowledge/      # Vector + keyword search
│   ├── observation/    # Tiered observation engine
│   ├── intent/         # Intent recognition
│   ├── workspace/      # Workspace management
│   ├── security/       # Keychain, encryption, audit
│   └── testing/        # Test utilities
├── skills/             # Individual skill modules (future)
├── installer/          # Packaging (future)
├── tests/              # Integration tests
└── docs/               # Documentation
```

## Troubleshooting

### Rust build fails

Ensure you have the latest stable toolchain:
```bash
rustup update stable
```

### `cargo clippy` fails with warnings

Run auto-fix first:
```bash
cargo clippy --all-targets --all-features --fix --allow-dirty
```

### SQLite compilation fails

Ensure SQLite development headers are installed:
- **Ubuntu/Debian**: `sudo apt install libsqlite3-dev`
- **macOS**: `brew install sqlite3`
- **RHEL/Fedora**: `sudo dnf install sqlite-devel`
- **Windows**: Bundled via `rusqlite` feature `bundled` (no manual install needed)