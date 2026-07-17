# Coding Standards

## Rust Coding Standards

### Style Guide

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) where applicable.
- Use `rustfmt` with default settings (run `cargo fmt` before every commit).
- Use `clippy` with default lints (`cargo clippy --all-targets --all-features`). Fix all warnings.

### Naming Conventions

| Type | Convention | Example |
|------|-----------|---------|
| Modules | `snake_case` | `data_types`, `skill_manager` |
| Structs | `PascalCase` | `AiProvider`, `ToolDefinition` |
| Traits | `PascalCase` + adjective suffix | `Send`, `AiProvider`, `SkillModule` |
| Functions/Methods | `snake_case` | `list_tools`, `call_tool` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_RETRIES`, `DEFAULT_TIMEOUT` |
| Type Parameters | `PascalCase` (single letter for generic) | `T`, `Repository<T>`, `Entity` |
| Attributes/Annotations | `snake_case` | `#[allow(unused)]` |

### Module Structure

- Each crate should have a `lib.rs` that re-exports public types and documents the module layout.
- Keep files under 300 lines where possible. Split into submodules when a file grows larger.
- Group related types in submodules (e.g., `chat.rs`, `ai.rs`, `tool.rs` under `data_types/`).

```
src/
├── core/
│   ├── data_types/      # Shared domain models
│   │   ├── lib.rs       # Module declarations + re-exports
│   │   ├── chat.rs      # ChatMessage
│   │   ├── ai.rs        # AiRequest, AiResponse
│   │   └── ...
│   └── persistence/     # Database layer
│       ├── lib.rs
│       ├── db.rs
│       ├── schema.rs
│       ├── migrations.rs
│       └── repositories.rs
├── ai/                  # AI runtime
├── mcp/                 # MCP protocol bridge + skill manager
├── knowledge/           # Vector + keyword search
├── observation/         # Tiered observation engine
├── intent/              # Intent recognition
├── workspace/           # Workspace management
├── security/            # Encryption + credentials
└── testing/             # Mocks + fixtures
```

### Error Handling

- Use `anyhow::Result` for application-level errors.
- Use `thiserror::Error` for library-defined error types.
- Never use `panic!()` except in unrecoverable conditions (e.g., invalid configuration).
- Propagate errors with `?` operator.

```rust
// Library error types
#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Database connection failed: {0}")]
    Connection(String),
    #[error("Migration failed: {0}")]
    Migration(#[from] rusqlite::Error),
}

// Application functions
pub async fn query() -> anyhow::Result<Vec<Record>> { ... }
```

### Async/Await

- Use `#[async_trait]` for trait methods. For skeleton code, suppress warnings with `#[allow(async_fn_in_trait)]` — these will be fixed when real implementations are added.
- Prefer `tokio` for the async runtime.
- Use `tokio::sync::mpsc` for streaming and `futures` for combinators.

### Documentation

- Document every public function, struct, and module with doc comments (`///`).
- Include at minimum: purpose, parameters, return value, and error conditions.
- Add doc tests where the logic is straightforward.

```rust
/// Initialize the AI provider with the given configuration.
///
/// # Arguments
///
/// * `config` — The provider configuration (API key, endpoint, model).
///
/// # Errors
///
/// Returns an error if the API key is invalid or the endpoint is unreachable.
pub async fn init(config: &ProviderConfig) -> anyhow::Result<Self>;
```

### Testing

- Unit tests: One `#[cfg(test)]` module per file, with test functions in the same file.
- Integration tests: Place in `tests/` directory with `mod` imports.
- Use `anyhow::Result` in test functions to simplify error handling.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_with_valid_config() -> anyhow::Result<()> {
        let config = create_valid_config();
        let provider = Provider::init(&config).await?;
        assert!(!provider.name().is_empty());
        Ok(())
    }
}
```

### Dependencies

- Pin all dependencies to specific versions in `Cargo.toml`.
- Use `workspace.dependencies` for shared dependencies across crates.
- Avoid pulling in unnecessary crates — every dependency increases compile time and attack surface.

### Git Practices

- Write atomic commits with descriptive messages (imperative mood, <72 chars).
- Squash intermediate commits before merge; keep PR descriptions detailed.
- Branch naming: `feature/xxx`, `fix/xxx`, `docs/xxx`.