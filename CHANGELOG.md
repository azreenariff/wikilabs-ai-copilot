# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Rust workspace with 12 crates (`data_types`, `persistence`, `ai`, `mcp`, `mcp-registry`, `mcp-skill-manager`, `knowledge`, `observation`, `intent`, `workspace`, `security`, `testing`)
- SQLite-backed persistence layer with repository traits and migration framework
- AI runtime abstraction with provider traits (OpenAI, vLLM, Ollama)
- MCP protocol bridge and consolidated skill manager runtime
- Hybrid knowledge system (SQLite VSS vector search + FTS5 keyword search)
- Tiered observation engine (shell, app, screen capture)
- Intent recognition engine with pattern matching
- Workspace management with technology stack configuration
- Security layer with encryption and credential management
- Skeleton domain types (`ChatMessage`, `AiRequest`, `AiResponse`, `Intent`, `KnowledgeDocument`, `SkillMetadata`, etc.)

### Tests
- **181 unit tests across all 12 crates** — every module has at least basic tests
- Workspace manager: full CRUD lifecycle (create, switch, get, list, delete, multiple workspaces)
- Knowledge: document types, search, embedding, import, dedup, quality scoring
- Security: classification, keychain, key derivation, encryption, credentials, injection defense, audit
- Observation: tiers 1-3, shell, app monitor, clipboard, capture, OCR, credential filter
- AI: context window allocation, token counting, response streaming, tool calls
- Intent: model prediction, engine pattern matching, confidence scoring, correction tracking
- Fixed: `Intent` enum missing `Hash` derive (used in correction HashMap)
- Fixed: `chrono` dependency missing from `wikilabs-intent` Cargo.toml
- Fixed: `Debug` derives missing from `Secret`, `AppContext`, `CaptureResult`, `KnowledgeChunk`, `KnowledgeDocument`, `SearchQuery`, `SearchResult`, `EmbeddingResult`, `QualityScore`
- Fixed: confidence formula in `recognize_with_confidence` for multiple matches

### Documentation
- Product Vision document
- Architecture Review with 16 risk items
- 8 Architecture Decision Records (ADRs)
- Revised architecture specification
- Phased implementation roadmap (4 phases, ~40 weeks)
- Backlog with 10 epics and dependency graph
- Coding standards (Rust style, naming, error handling, async, testing)
- Development guide with setup, build, and CI/CD instructions
- GitHub issue templates (bug, feature, security, architecture)

### Changed
- Consolidated multi-process MCP into single-process skill module loader
- Simplified architecture to deliver MVP chat copilot in 12 weeks instead of 12+ months

## [0.1.0] — 2024-XX-XX

### Added
- Initial repository bootstrap
- Workspace `Cargo.toml` with dependency management
- CI/CD workflow skeleton (`pr.yml`, `main.yml`, `release.yml`)
- `src/main.rs` entrypoint
- Skeleton crates for all planned modules
- GitHub issue templates

[Unreleased]: https://github.com/wikilabs/wikilabs-ai-copilot/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/wikilabs/wikilabs-ai-copilot/releases/tag/v0.1.0