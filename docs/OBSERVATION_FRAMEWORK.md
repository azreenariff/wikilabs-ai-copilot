# Observation Framework

**Phase 6** — Observation Infrastructure for Wiki Labs AI Copilot

---

## Overview

The Observation Framework collects, normalizes, and publishes structured observation events from multiple sources. It does **not** interpret engineer intent, perform AI reasoning, or provide recommendations. It only observes activity and produces structured events for downstream consumers.

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  Observation Engine              │
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │ Provider │  │ Provider │  │ Provider │ ...  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘      │
│       │              │              │             │
│       └──────────────┼──────────────┘             │
│                      │                            │
│              ┌───────▼────────┐                   │
│              │  Event Bus     │                   │
│              └───────┬────────┘                   │
│                      │                            │
│              ┌───────▼────────┐                   │
│              │ Privacy Layer  │                   │
│              └────────────────┘                   │
└─────────────────────────────────────────────────┘
```

### Principles

1. **Observation only** — providers collect data, they do not analyze it.
2. **Modular** — each provider is independent and pluggable.
3. **Configurable** — every provider can be individually enabled/disabled with its own settings.
4. **Privacy-first** — master control, per-provider toggle, pause/resume, and visible indicator.
5. **Lightweight** — low CPU/memory by default, efficient event batching, configurable intervals.

---

## Providers

### ScreenCaptureProvider

Captures periodic screenshots of the current screen.

- **Responsibilities:** Periodic capture, multi-monitor support, window-aware capture, pause/resume
- **Does NOT:** Perform OCR, AI analysis, or content inspection
- **Configurable:** Capture interval, source screens, resolution
- **Event:** `ScreenshotCaptured`

### ActiveWindowProvider

Detects the current foreground application and window.

- **Detects:** Foreground application, window title, executable, process information
- **Event:** `ApplicationChanged`

### TerminalProvider

Observes terminal sessions and commands.

- **Supports:** Bash, Zsh, PowerShell, Command Prompt, SSH
- **Observes:** Commands entered, session lifecycle (not command output by default)
- **Engineering detection:** Identifies sessions related to Kubernetes, Docker, CI/CD, cloud infrastructure
- **Does NOT:** Execute commands or analyze command output
- **Event:** `TerminalCommand`

### BrowserProvider

Detects browser context and focuses on engineering portals.

- **Detects:** Browser type, current URL, page title
- **Engineering portals:** OpenShift, vCenter, Nagios, Checkmk, Grafana, CI/CD dashboards
- **Does NOT:** Inspect unrelated browsing, track personal accounts
- **Event:** `BrowserContextChanged`

### ClipboardProvider

Observes copied text content (opt-in only).

- **Detects:** Error messages, log entries, stack traces, plain text
- **Filtering:** Automatically identifies error patterns, stack traces, and log formats
- **Privacy:** Content stored only when explicitly enabled
- **Event:** `ClipboardChanged`

### FileObserverProvider

Detects file system activity on configuration files.

- **Detects:** Files opened, configuration files edited
- **Tracked extensions:** `.yaml`, `.yml`, `.json`, `.conf`, `.xml`, `.ini`, `.properties`
- **Privacy:** Metadata only by default; content inspection requires explicit opt-in
- **Event:** `ConfigurationFileOpened`

---

## Event Bus

The central Event Bus (`EventBus`) routes events between providers and subscribers.

### Key features

- **Pub/Sub pattern** — providers publish events, subscribers consume them
- **Filtered subscriptions** — subscribe to all events or filter by provider type
- **Clonable** — multiple consumers can share the same bus
- **Statistics tracking** — event counts, publish/subscriber metrics
- **Backpressure safe** — bounded buffer, dropped events logged

### Event lifecycle

```
Provider.observe() → creates Event → publish() → EventBus → subscribers[0..N] → try_recv()
```

---

## Event Model

Every observation event follows this schema:

| Field          | Type              | Description                          |
| -------------- | ----------------- | ------------------------------------ |
| `id`           | `String`          | Unique event identifier              |
| `timestamp`    | `DateTime<Utc>`   | When the observation occurred        |
| `event_type`   | `EventType`       | High-level category                  |
| `source`       | `String`          | Specific source within the provider  |
| `provider`     | `ProviderType`    | Which provider generated this event  |
| `workspace`    | `String`          | Associated workspace                 |
| `confidence`   | `f64` (0–1)       | Confidence in the observation        |
| `payload`      | `ObservationPayload` | Structured, serializable payload  |

### EventType enum

```rust
ApplicationChanged
TerminalCommand
ScreenshotCaptured
BrowserContextChanged
ClipboardChanged
ConfigurationFileOpened
```

### ProviderType enum

```rust
ScreenCapture
ActiveWindow
Terminal
Browser
Clipboard
FileObserver
```

All events are serializable via `serde_json`.

---

## Provider Registry

The `ProviderRegistry` manages all registered providers:

| Method               | Description                               |
| -------------------- | ----------------------------------------- |
| `register()`         | Register a new provider                   |
| `unregister()`       | Remove a provider                         |
| `get()`              | Get a provider by name                    |
| `provider_names()`   | List all registered provider names        |
| `start_all()`        | Start all enabled providers               |
| `stop_all()`         | Stop all providers                        |
| `set_provider_enabled()` | Enable/disable a specific provider    |
| `all_status()`       | Get status snapshot of all providers      |

---

## Privacy Controls

The `PrivacyManager` ensures the engineer maintains full control:

### Master Controls

| Mode          | Behavior                                |
| ------------- | --------------------------------------- |
| `Enabled`     | All allowed providers collect data      |
| `Disabled`    | No observation occurs                   |
| `Paused`      | Observation suspended, no data collected|

### Per-Provider Controls

Individual providers can be disabled without affecting others. This is useful for:
- Disabling clipboard while keeping other providers active
- Pausing screen capture during sensitive operations

### Privacy Indicator

A visible `PrivacyIndicator` is maintained, showing:
- Current observation mode
- Which providers are currently active
- Whether observation is visibly occurring

The indicator is updated automatically whenever mode or provider configuration changes.

### Data Retention

- **Clipboard:** Content stored only when `store_clipboard_content` is explicitly enabled
- **Files:** Metadata only by default; content requires explicit opt-in
- **Screenshots:** Configurable retention period (default: 1 day)
- **Retention days:** Configurable globally (default: 7 days)

---

## Performance

The framework is designed for low overhead:

- **Default intervals:** Screen capture at reasonable intervals (not continuous)
- **Event batching:** Multiple observations can be batched before publishing
- **Efficient serialization:** Events use optimized serde serialization
- **No blocking:** Providers use async traits to avoid blocking the main thread
- **Bounded buffers:** Event bus uses bounded channels to prevent memory growth

---

## Plugin Architecture

Providers implement the `ObservationProvider` trait, making them inherently pluggable:

```rust
#[async_trait]
trait ObservationProvider {
    fn provider_type(&self) -> ProviderType;
    fn name(&self) -> &str;
    fn config(&self) -> ProviderConfig;
    fn set_config(&mut self, config: ProviderConfig);
    fn state(&self) -> ProviderState;
    async fn start(&mut self) -> Result<(), String>;
    async fn stop(&mut self) -> Result<(), String>;
    async fn pause(&mut self) -> Result<(), String>;
    async fn resume(&mut self) -> Result<(), String>;
    async fn observe(&self) -> Result<Vec<ObservationEvent>, String>;
    fn lifecycle(&self) -> ProviderLifecycle;
    fn status_details(&self) -> HashMap<String, serde_json::Value>;
}
```

New providers can be added by:
1. Implementing `ObservationProvider`
2. Registering with the `ProviderRegistry`
3. No changes to core framework needed

---

## Validation Checklist

- ✅ Providers load correctly
- ✅ Events publish correctly
- ✅ Event bus functions (publish, subscribe, filter, stats)
- ✅ Privacy controls work (master toggle, per-provider, pause/resume, indicator)
- ✅ Observation indicator visible
- ✅ No AI reasoning implemented
- ✅ No intent recognition implemented
- ✅ No recommendations generated
- ✅ All 129 tests passing

---

## Files

- `src/observation/src/lib.rs` — Main entry point, provider registration
- `src/observation/src/provider.rs` — `ObservationProvider` trait, `ProviderRegistry`
- `src/observation/src/event.rs` — Event model, types, serialization
- `src/observation/src/event_bus.rs` — Pub/sub event bus
- `src/observation/src/privacy.rs` — Privacy controls and indicator
- `src/observation/src/engine.rs` — Observation orchestration
- `src/observation/src/screen_capture.rs` — Screen capture provider
- `src/observation/src/app_monitor.rs` — Active window provider
- `src/observation/src/terminal.rs` — Terminal observation provider
- `src/observation/src/browser.rs` — Browser observation provider
- `src/observation/src/clipboard.rs` — Clipboard observation provider
- `src/observation/src/file_observer.rs` — File observation provider
- `docs/OBSERVATION_FRAMEWORK.md` — This document
- `docs/EVENT_MODEL.md` — Event schema reference
- `docs/PROVIDER_DEVELOPMENT_GUIDE.md` — How to write new providers