# Development Rules — Wiki Labs AI Copilot

## Core Rules

1. **No feature without documentation** — Every feature must have:
   - Updated module `README.md` (if applicable)
   - Updated `ARCHITECTURE_DECISIONS.md` (if architectural change)
   - Updated `CHANGELOG.md`
   - Test cases (unit or integration)

2. **No module without tests** — Every new module must have:
   - Unit tests with 80%+ coverage
   - At least one integration test
   - Test fixtures in `src/testing/`

3. **No breaking changes without ADR** — Any change to:
   - Public interfaces / traits
   - Data models
   - Database schema
   - API contracts
   Must have an approved ADR.

4. **No secrets in source code** — Never commit:
   - API keys, tokens, passwords
   - PEM files, certificates
   - `.env` files
   - SSH keys

5. **No direct coupling between skills and core** — Skills must interact with core only through:
   - `AiProvider` trait
   - MCP tool definitions
   - Shared data types in `core/data_types`

6. **All external integrations through interfaces** — Any external service (AI provider, OS API, database) must be accessed through a trait or interface. This enables:
   - Mocking in tests
   - Provider abstraction
   - Swappable implementations

## Git Rules

- Commit often, commit small
- Use conventional commit format
- One feature/fix per branch
- Squash merge to `develop`
- Tag releases: `vMAJOR.MINOR.PATCH`

## Security Rules

- Run dependency vulnerability scan on every CI run
- Use `cargo audit` in PR checks
- Never use hardcoded configuration values
- Apply least-privilege principle to all permissions
- Validate all external inputs before processing

## Code Review Rules

- At least one reviewer required
- CI must pass before merge
- Reviewers must verify:
  - Security implications
  - Performance impact
  - Test coverage
  - Documentation updates
  - Error handling completeness