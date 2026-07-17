# Development Guide

## Prerequisites

- **Rust Toolchain** (stable, via `rustup`)
  ```bash
  rustup default stable
  ```
- **Cargo** (included with Rust)
- **Git** for version control

## Project Setup

### Clone the Repository

```bash
git clone https://github.com/wikilabs/wikilabs-ai-copilot.git
cd wikilabs-ai-copilot
```

### Verify the Build

```bash
cargo build --all
cargo test --all
cargo clippy --all-targets --all-features
```

All three commands must complete with zero errors and no `clippy` warnings before contributing.

## Cargo Workspace

This project uses a Cargo workspace with multiple crates:

```
Cargo.toml (workspace root)
├── src/core/data_types   — Shared domain models
├── src/core/persistence  — SQLite persistence layer
├── src/ai                — AI runtime abstraction
├── src/mcp               — MCP protocol bridge
├── src/mcp/skill_manager — Consolidated skill runtime
├── src/mcp/registry      — Global tool catalog
├── src/knowledge         — Vector + keyword search
├── src/observation       — Tiered observation engine
├── src/intent            — Intent recognition
├── src/workspace         — Workspace management
├── src/security          — Encryption + credentials
└── src/testing           — Mocks + fixtures
```

### Adding a New Crate

1. Create the crate directory: `src/<name>/Cargo.toml` and `src/<name>/src/lib.rs`.
2. Add it to `Cargo.toml` workspace `members`.
3. Add shared dependencies to `[workspace.dependencies]`.
4. Reference the crate from other crates: `wikilabs-<name> = { path = "<path>" }`.

### Workspace Dependencies

Shared dependencies are declared in the workspace root's `[workspace.dependencies]` and referenced with `{ workspace = true }` in member crates. This ensures version consistency across the entire workspace.

## Building and Running

### Incremental Build

```bash
# Build all crates
cargo build --all

# Build a specific crate
cargo build -p wikilabs-knowledge

# Release build
cargo build --release --all
```

### Running Tests

```bash
# Run all tests
cargo test --all

# Run tests for a specific crate
cargo test -p wikilabs-data-types

# Run with output (for debugging)
cargo test --all -- --nocapture
```

### Linting

```bash
# Run clippy
cargo clippy --all-targets --all-features

# Fix auto-fixable warnings
cargo clippy --fix --all-targets --all-features

# Format the codebase
cargo fmt
```

## CI/CD

The project uses GitHub Actions with three workflows:

### PR Checks (`.github/workflows/pr.yml`)
- Runs `cargo fmt --check`, `cargo clippy`, and `cargo test` on every pull request.
- Blocks merge if any check fails.

### Main Branch (`.github/workflows/main.yml`)
- Runs on every push to `main`.
- Builds release binary and runs full test suite.

### Release (`.github/workflows/release.yml`)
- Triggered by annotated git tags (`v*`).
- Cross-compiles binaries for Linux, macOS, and Windows.
- Publishes to GitHub Releases.

## Architecture Documentation

All architecture decisions are tracked in:

| Document | Path | Purpose |
|----------|------|---------|
| Architecture Review | `docs/ARCHITECTURE_REVIEW.md` | Critical assessment of existing architecture |
| Risk Register | `docs/ARCHITECTURE_RISKS.md` | 16-item risk analysis with severity scores |
| Architecture Decisions | `docs/ARCHITECTURE_DECISIONS.md` | 8 formal ADRs |
| Revised Architecture | `docs/REVISED_ARCHITECTURE.md` | Updated architecture spec |
| Revised Roadmap | `docs/REVISED_ROADMAP.md` | Phased implementation plan |
| Product Vision | `docs/product/VISION.md` | Product vision and principles |

## Adding Architecture Decision Records

When making a significant technical decision:

1. Create a new ADR in `docs/ARCHITECTURE_DECISIONS.md` following the existing template.
2. Reference it from the Revised Architecture document if it changes the design.
3. Add a bullet to the relevant epic in `BACKLOG.md` if it affects implementation.

## Reporting Issues

Use the GitHub issue templates:
- **Bug Report** — For reproducible errors
- **Feature Request** — For new capabilities
- **Security Issue** — For vulnerability reports (confidential)
- **Architecture Decision** — For tracking new ADRs

## Contributing

1. Fork the repository.
2. Create a feature branch (`feature/your-feature`).
3. Make your changes following the coding standards.
4. Run `cargo fmt`, `cargo clippy`, and `cargo test` locally.
5. Submit a pull request with a detailed description.
6. Address review feedback.
7. Squash and merge (branch naming convention applies).