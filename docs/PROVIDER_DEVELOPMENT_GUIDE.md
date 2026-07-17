# Provider Development Guide

**Phase 6** — How to write new Observation Providers

---

## Overview

Providers are the pluggable units of the Observation Framework. Each provider implements the `ObservationProvider` trait and is registered with the `ProviderRegistry`. New providers can be added without modifying the core framework.

---

## The ObservationProvider Trait

```rust
#[async_trait]
pub trait ObservationProvider: Send + Sync {
    // Identity
    fn provider_type(&self) -> ProviderType;
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    // Configuration
    fn config(&self) -> ProviderConfig;
    fn set_config(&mut self, config: ProviderConfig);

    // Lifecycle
    fn state(&self) -> ProviderState;
    async fn start(&mut self) -> Result<(), String>;
    async fn stop(&mut self) -> Result<(), String>;
    async fn pause(&mut self) -> Result<(), String>;
    async fn resume(&mut self) -> Result<(), String>;

    // Observation
    async fn observe(&self) -> Result<Vec<ObservationEvent>, String>;

    // Status
    fn lifecycle(&self) -> ProviderLifecycle;
    fn status_details(&self) -> HashMap<String, serde_json::Value>;
}
```

---

## Step-by-Step: Creating a New Provider

### 1. Define Your Provider State

Create a state struct that holds configuration, lifecycle info, and provider-specific data:

```rust
pub struct MyProviderState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub custom_data: MyProviderData,
}

impl MyProviderState {
    fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            custom_data: MyProviderData::default(),
        }
    }
}
```

### 2. Create the Provider Struct

```rust
pub struct MyProvider {
    state: Arc<Mutex<MyProviderState>>,
}
```

Using `Arc<Mutex<...>>` ensures thread safety and allows the provider to be shared across async tasks.

### 3. Implement the Provider Trait

```rust
impl MyProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MyProviderState::new(
                ProviderConfig::default()
            ))),
        }
    }
}

#[async_trait]
impl ObservationProvider for MyProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::MyProvider  // Add variant to the enum
    }

    fn name(&self) -> &str {
        "My Provider"
    }

    fn description(&self) -> &str {
        "Provides observations of X"
    }

    fn config(&self) -> ProviderConfig {
        self.state.lock().unwrap().config.clone()
    }

    fn set_config(&mut self, config: ProviderConfig) {
        self.state.lock().unwrap().config = config;
    }

    fn state(&self) -> ProviderState {
        self.state.lock().unwrap().state.clone()
    }

    async fn start(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.start();
        state.state = ProviderState::Active;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.stop();
        state.state = ProviderState::Disabled;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if matches!(state.state, ProviderState::Active) {
            state.state = ProviderState::Paused;
        }
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if matches!(state.state, ProviderState::Paused) {
            state.state = ProviderState::Active;
        }
        Ok(())
    }

    async fn observe(&self) -> Result<Vec<ObservationEvent>, String> {
        let events = Vec::new();  // Collect your observations here

        // Create events
        events.push(ObservationEvent::new(
            EventType::MyEventType,
            ProviderType::MyProvider,
            "my_source".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({
                "key": "value",
            })),
        ));

        Ok(events)
    }

    fn lifecycle(&self) -> ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        details.insert("platform".to_string(), serde_json::json!(std::env::consts::OS));
        details
    }
}
```

### 4. Register the Provider

In `src/observation/src/lib.rs`, add your provider to the registration:

```rust
use crate::my_provider::MyProvider;

fn register_all_providers(registry: &mut ProviderRegistry) {
    registry.register(Box::new(MyProvider::new()));
    // ... other providers
}
```

### 5. Add Event Types

In `src/observation/src/event.rs`:

```rust
// Add to EventType enum
pub enum EventType {
    // ... existing types
    MyEventType,
}

// Add to ProviderType enum
pub enum ProviderType {
    // ... existing types
    MyProvider,
}
```

Update the `Display` implementations and any `match` statements.

---

## Testing Your Provider

### Unit Tests Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = MyProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::MyProvider);
        assert_eq!(provider.name(), "My Provider");
    }

    #[test]
    fn test_provider_lifecycle() {
        let mut provider = MyProvider::new();
        assert_eq!(provider.state(), ProviderState::Disabled);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(provider.start().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);

            assert!(provider.pause().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Paused);

            assert!(provider.resume().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);

            assert!(provider.stop().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Disabled);
        });
    }

    #[test]
    fn test_config_get_set() {
        let mut provider = MyProvider::new();
        let mut config = provider.config();
        config.enabled = false;
        provider.set_config(config);
        assert!(!provider.config().enabled);
    }

    #[test]
    fn test_observe_emits_event() {
        let mut provider = MyProvider::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::MyEventType);
    }

    #[test]
    fn test_status_details() {
        let provider = MyProvider::new();
        let details = provider.status_details();
        assert!(details.contains_key("platform"));
    }
}
```

### Integration Tests

Test the provider through the event bus:

```rust
#[test]
fn test_provider_through_event_bus() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(MyProvider::new()));

    let bus = EventBus::with_defaults();
    // Test event publishing and consumption
}
```

---

## Guidelines

### Do ✅

- Use `Arc<Mutex<...>>` for shared state
- Return `Result<Vec<ObservationEvent>, String>` from `observe()`
- Create events with `ObservationEvent::new()`
- Include provider-specific details in `status_details()`
- Write tests covering lifecycle, config, and observation
- Add the provider to `lib.rs` registration

### Do NOT ❌

- Perform AI reasoning or intent detection
- Make blocking syscalls in async context
- Store sensitive data without privacy controls
- Create unbounded buffers (use bounded channels)
- Assume the OS is Linux — handle platform differences
- Modify global state outside the provider

---

## Privacy Integration

Your provider MUST respect the privacy layer:

1. **Check `PrivacyManager`** before observing sensitive data
2. **Use `store_clipboard_content` / `store_file_content`** flags for content storage
3. **Only store metadata** unless explicitly enabled for content
4. **Respect observation mode** — don't collect in Disabled/Paused mode

Example:
```rust
async fn observe(&self) -> Result<Vec<ObservationEvent>, String> {
    // Privacy check
    if !self.privacy_manager.is_provider_allowed(&self.provider_type()) {
        return Ok(vec![]);
    }

    let cfg = self.config();
    if !cfg.enabled {
        return Ok(vec![]);
    }

    // ... proceed with observation
}
```

---

## Performance Tips

- **Configurable intervals** — don't poll at fixed high frequency
- **Batch observations** — collect multiple observations before publishing
- **Async I/O** — use async file/network operations
- **Bounded buffers** — prevent memory growth in event queue
- **Profile your provider** — use `tracing` for instrumentation

---

## Debugging

Enable tracing:
```rust
tracing::info!("Provider started");
tracing::debug!("Observing: {:?}", data);
tracing::warn!("Observation failed: {}", error);
```

Check provider status:
```rust
let statuses = registry.all_status();
for status in statuses {
    println!("{:?}: {:?}", status.name, status.state);
}
```

---

## References

- `ObservationProvider` trait — `src/observation/src/provider.rs`
- Event model — `docs/EVENT_MODEL.md`
- Existing providers — `src/observation/src/`