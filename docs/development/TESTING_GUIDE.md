# Testing Guide — Wiki Labs AI Copilot

## Testing Strategy

### Test Pyramid

```
       /\
      /  \      E2E Tests (few)
     /----\
    /      \    Integration Tests (moderate)
   /--------\
  /          \  Unit Tests (many)
 /------------\
```

## Test Categories

### Unit Tests (`*tests.rs` files, inline tests with `#[cfg(test)]`)

- **Coverage target**: 80%+
- **Purpose**: Test individual functions and methods in isolation
- **Approach**: Mock external dependencies, test logic correctness
- **Location**: Within module source files or parallel `*_tests.rs` files
- **Naming**: `test_<scenario>_<expected_behavior>`

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_not_found_returns_error() {
        let manager = WorkspaceManager::new();
        let result = manager.find_by_id("nonexistent");
        assert!(result.is_err());
    }
}
```

### Integration Tests (`tests/` directory)

- **Purpose**: Test multi-module interactions
- **Approach**: Use real implementations where possible, mock external services
- **Coverage**: All MCP tool handlers, persistence operations, API interactions
- **Naming**: Same as unit tests

### End-to-End Tests (`tests/e2e/` directory)

- **Purpose**: Validate complete user workflows
- **Approach**: Full application stack (Tauri + React + Rust)
- **Tool**: Playwright (browser automation)
- **Coverage**: Core user journeys from `docs/product/USER_STORIES.md`

### Security Tests (`tests/security/` directory)

- **Purpose**: Verify security requirements
- **Approach**:
  - Dependency vulnerability scan (`cargo audit`)
  - Secret scanning (`cargo secret-scan` or similar)
  - Injection detection tests (prompt injection defense)
  - Credential redaction tests
- **Coverage**: All security requirements from `docs/security/SECURITY.md`

### Performance Tests (`tests/performance/` directory)

- **Purpose**: Validate performance targets
- **Coverage**:
  - Cold startup < 5 seconds
  - Idle RAM < 150 MB (Phase 1)
  - AI response < 3 seconds first token
  - Knowledge search < 500 ms
  - Knowledge import (10 MB PDF) < 30 seconds
  - Workspace switch < 1 second

## Test Data

- Use fixtures from `src/testing/src/fixtures/`
- Never use real credentials, API keys, or production data
- All test data must be deterministic (no random values unless seeded)

## Running Tests

```bash
# All tests
cargo test --all

# Specific module
cargo test -p wikilabs-ai

# With coverage
cargo test --all -- --nocapture
cargo llvm-cov --all-features --workspace --html
```

## Test Quality Checklist

For every PR:
- [ ] New code has unit tests with 80%+ coverage
- [ ] Integration tests for all new public APIs
- [ ] No use of `unwrap()` or `expect()` in production code
- [ ] Test data uses fixtures, not hardcoded values
- [ ] No flaky tests (deterministic outcomes)
- [ ] Performance targets verified for performance-critical changes