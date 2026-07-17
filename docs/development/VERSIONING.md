# Versioning — Wiki Labs AI Copilot

## Semantic Versioning

This project follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html):

```
MAJOR.MINOR.PATCH
  │     │     │
  │     │     └─ Patch: Bug fixes, documentation, refactoring
  │     └─────── Minor: New features (backward compatible)
  └───────────── Major: Breaking changes
```

### Version Rules

| Change Type | Version Bump | Examples |
|-------------|-------------|----------|
| **Breaking change** | MAJOR | Rename public trait, change data model, remove API |
| **New feature** | MINOR | New skill module, new search feature, new AI provider |
| **Bug fix** | PATCH | Fix crash, fix search accuracy, fix performance |
| **Documentation** | PATCH | Update README, add ADR |
| **Refactoring** | PATCH | No behavior change |

### Pre-release Versions

Use for development builds:

```
1.0.0-alpha.1    # Alpha release
1.0.0-beta.1     # Beta release
1.0.0-rc.1       # Release candidate
```

Format: `MAJOR.MINOR.PATCH-prerelease.N`

## Release Process

### Release Steps

1. Create release branch: `git checkout -b release/vX.Y.Z develop`
2. Update `CHANGELOG.md` with release notes
3. Update `VERSION` in `Cargo.toml` to `X.Y.Z`
4. Run full CI: `cargo test --all`, `cargo clippy`, `cargo build`
5. Merge to `main`: `git checkout main && git merge release/vX.Y.Z`
6. Tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
7. Merge back to `develop`: `git checkout develop && git merge release/vX.Y.Z`
8. Push: `git push origin main --tags`
9. GitHub Actions release workflow builds artifacts

### Changelog Format

```markdown
## [vX.Y.Z] - YYYY-MM-DD

### Added
- Feature description

### Changed
- Change description

### Fixed
- Bug fix description

### Security
- Security fix description
```