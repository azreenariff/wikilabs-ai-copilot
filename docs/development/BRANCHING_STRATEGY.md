# Branching Strategy — Wiki Labs AI Copilot

## Branch Model

```
main
  ├── release/v0.1.0
  │     ├── merge ──► v0.1.0 tag
  │
develop
  ├── feature/workspace-manager
  │     ├── merge ──► develop
  ├── feature/open-shift-skill
  │     ├── merge ──► develop
  ├── bugfix/crash-on-startup
  │     ├── merge ──► develop
  │
  (hotfix branches from main)
```

### Branch Types

| Branch | Purpose | Target | Example |
|--------|---------|--------|---------|
| `main` | Production-ready code | — | `main` |
| `develop` | Integration branch | — | `develop` |
| `feature/*` | New features | `develop` | `feature/knowledge-import` |
| `bugfix/*` | Bug fixes | `develop` | `bugfix/memory-leak` |
| `release/*` | Release preparation | `main` + `develop` | `release/v0.1.0` |

## Workflow

### Feature Branches

1. `git checkout develop && git pull`
2. `git checkout -b feature/your-feature`
3. Implement + test
4. `git push origin feature/your-feature`
5. Open PR to `develop`
6. After review: squash merge

### Bug Fix Branches

Same as feature branches.

### Release Branches

1. `git checkout develop && git pull`
2. `git checkout -b release/v0.1.0`
3. Version bumps, changelog, bug fixes
4. `git merge release/v0.1.0 into main`
5. Tag `v0.1.0` on main
6. `git merge release/v0.1.0 into develop`
7. Delete branch

### Hotfix Branches

1. `git checkout main && git pull`
2. `git checkout -b hotfix/fix-description`
3. Fix + test
4. Merge to `main` + `develop`
5. Create new release tag if needed