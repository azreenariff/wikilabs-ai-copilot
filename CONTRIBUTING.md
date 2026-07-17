# Contributing to Wiki Labs AI Copilot

## Development Workflow

### Branch Strategy

- `main` — Production-ready code
- `develop` — Integration branch for all feature development
- `feature/*` — Feature branches (e.g., `feature/open-shift-skill`)
- `bugfix/*` — Bug fix branches (e.g., `bugfix/crash-on-startup`)
- `release/*` — Release preparation branches (e.g., `release/v0.1.0`)

### Commit Standards

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

feat(core): add workspace manager interface
fix(ai): handle empty provider response
docs(readme): update getting started section
test(skills): add mock skill module tests
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

### Pull Requests

1. Create feature branch from `develop`
2. Implement changes with tests
3. Open PR targeting `develop`
4. Ensure all CI checks pass
5. Request at least one code review
6. Squash merge to `develop` after approval

### Code Review Rules

- All PRs require at least one reviewer
- CI must pass before merge
- No breaking changes without ADR approval
- New code requires tests (minimum 80% coverage for new modules)
- Security-sensitive changes require security review

## Architecture Decisions

All major architectural decisions are documented as ADRs in [docs/adr/](docs/adr/).
New ADRs must be created for any change to the architecture.

## Reporting Issues

- **Bugs**: Use the Bug Report template
- **Features**: Use the Feature Request template
- **Architecture**: Use the Architecture Decision template
- **Security**: Use the Security Issue template (handled with urgency)

## Code of Conduct

Be respectful, inclusive, and professional. Harassment will not be tolerated.