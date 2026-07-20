# CHANGELOG

All notable changes to Wiki Labs AI Copilot will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.7.0-alpha] — 2026-07-20

### Added

- **Phase 10 — Engineering Guidance Framework**
  - `wikilabs-guidance` crate with 9 modules (~3,700 lines, 91 tests)
  - Guidance Decision Engine with 4 modes (Teaching, Balanced, Expert, Silent)
  - Engineering Recommendation Framework with structured recommendations
  - Troubleshooting Workflow Framework with 5 built-in workflows
  - Read-Only Context Provider Framework with generic trait
  - MCP Context Integration for external systems
  - Evidence Collection Framework with gap tracking
  - Command Recommendation Engine with CLI/SQL/API suggestions
  - Command Safety Framework with 5-level risk classification
  - Engineer Guidance Timeline for session history
  - Guidance Feedback System with 5 feedback types
  - Copilot Mode management in `wikilabs-copilot`

- **Documentation**
  - `GUIDANCE_ENGINE.md` — Decision engine design and usage
  - `RECOMMENDATION_FRAMEWORK.md` — Recommendation structure and builder
  - `TROUBLESHOOTING_WORKFLOW.md` — Workflow templates and usage
  - `CONTEXT_PROVIDER_SPEC.md` — Generic context provider interface
  - `MCP_CONTEXT_INTEGRATION.md` — MCP protocol integration guide
  - `EVIDENCE_FRAMEWORK.md` — Evidence tracking and evaluation
  - `COMMAND_GUIDANCE.md` — Command suggestion and classification
  - `SAFETY_MODEL.md` — Risk levels and warning generation
  - `GUIDANCE_FEEDBACK.md` — Feedback system and adaptation

### Changed

- Fixed regex backreferences in `wikilabs-knowledge` (unsupported by Rust `regex` crate)
  - `html.rs`: Removed `</h\1>` backreference, added per-level patterns
  - `xml.rs`: Removed `</\1>` backreference, added closing tag stripping

### Fixed

- `wikilabs-guidance` crate: All 11 tests failing fixed (11 → 91 passing)
  - Unused imports, wrong borrows, method name changes, test assertions
- `wikilabs-knowledge`: Regex compilation errors fixed

### Technical

- Full workspace: 23 crates, 871 tests, 0 failures
- Build: `cargo build --all` succeeds
- Build time: ~35s for full workspace

---

## [0.6.0-alpha] — Earlier

### Added

- Phase 9 — Engineering Copilot Engine & Decision Framework
- Phase 8 — Enterprise Knowledge Platform
- Phase 7 — Engineering Intelligence Engine
- Phase 6 — Enterprise Knowledge Platform (knowledge extraction)
- Phase 5 — Desktop Application Foundation (Tauri)
- Phase 4 — AI Runtime Abstraction
- Phase 3 — Observation Framework
- Phase 2 — Engineering Intelligence Engine (knowledge)

---

[0.7.0-alpha]: https://github.com/wikilabs/wikilabs-ai-copilot/compare/v0.6.0-alpha...v0.7.0-alpha
[0.6.0-alpha]: https://github.com/wikilabs/wikilabs-ai-copilot/compare/v0.5.0-alpha...v0.6.0-alpha