# Event Model

**Phase 6** — Observation Event Schema Reference

---

## Overview

The Observation Framework uses a uniform event model for all providers. Every observation — whether from clipboard, terminal, browser, screen capture, active window, or file observer — produces an event following this schema.

This document describes the complete event model, all event types, provider types, and the payload schema.

---

## Core Event Structure

### ObservationEvent

| Field        | Type                          | Required | Description                          |
| ------------ | ----------------------------- | -------- | ------------------------------------ |
| `id`         | `String`                      | Yes      | Unique event ID (UUID)               |
| `timestamp`  | `DateTime<Utc>`               | Yes      | When the observation was made        |
| `event_type` | `EventType`                   | Yes      | High-level category                  |
| `source`     | `String`                      | Yes      | Specific source (e.g., app name)     |
| `provider`   | `ProviderType`                | Yes      | Which provider generated the event   |
| `workspace`  | `String`                      | No       | Associated workspace context         |
| `confidence` | `f64`                         | Yes      | Confidence in the observation (0–1)  |
| `payload`    | `ObservationPayload`          | Yes      | Structured, serializable data        |

### Confidence Values

| Value  | Meaning                            |
| ------ | ---------------------------------- |
| `1.0`  | Definitive observation             |
| `0.8`  | High confidence                    |
| `0.6`  | Moderate confidence                |
| `0.4`  | Low confidence                     |
| `0.0`  | Unknown/failed                     |

Confidence is automatically clamped to `[0.0, 1.0]`.

---

## ObservationPayload

The payload carries provider-specific structured data as a `serde_json::Value`.

### Common fields by event type

#### ApplicationChanged
```json
{
  "app_name": "vscode",
  "window_title": "main.rs - wikilabs-ai-copilot — Visual Studio Code",
  "executable": "/usr/share/code/code",
  "process_id": 12345,
  "platform": "linux"
}
```

#### TerminalCommand
```json
{
  "session_id": "1",
  "command": "kubectl get pods -n production",
  "shell": "bash",
  "terminal": "alacritty",
  "working_dir": "/home/user/k8s-deploy",
  "platform": "linux"
}
```

#### ScreenshotCaptured
```json
{
  "screen_index": 0,
  "resolution": "1920x1080",
  "file_path": "/tmp/screenshots/2024-01-01_120000.png",
  "file_size_bytes": 123456
}
```

#### BrowserContextChanged
```json
{
  "browser_type": "firefox",
  "url": "https://console-openshift-console.apps.example.com/dashboards",
  "title": "OpenShift Console — Dashboards",
  "is_engineering_portal": true,
  "portal_type": "openshift"
}
```

#### ClipboardChanged
```json
{
  "text_length": 256,
  "text": null,
  "looks_like_error": true,
  "looks_like_stack_trace": true,
  "looks_like_log": false
}
```

Note: `text` is only populated when `store_clipboard_content` is explicitly enabled in privacy settings.

#### ConfigurationFileOpened
```json
{
  "file_path": "/home/user/project/config.yaml",
  "file_extension": ".yaml",
  "file_size": 4096,
  "content": null
}
```

Note: `content` is only populated when `store_file_content` is explicitly enabled in privacy settings.

---

## EventType Enum

| Variant                | Description                          | Provider                |
| ---------------------- | ------------------------------------ | ----------------------- |
| `ApplicationChanged`   | Foreground window/app changed        | `ActiveWindowProvider`  |
| `TerminalCommand`      | Terminal command or session event    | `TerminalProvider`      |
| `ScreenshotCaptured`   | New screenshot captured              | `ScreenCaptureProvider` |
| `BrowserContextChanged`| Browser context changed              | `BrowserProvider`       |
| `ClipboardChanged`     | Clipboard content changed            | `ClipboardProvider`     |
| `ConfigurationFileOpened` | Config file opened/modified       | `FileObserverProvider`  |

---

## ProviderType Enum

| Variant        | Description             |
| -------------- | ----------------------- |
| `ScreenCapture`| Periodic screen capture |
| `ActiveWindow` | Foreground window info  |
| `Terminal`     | Terminal sessions       |
| `Browser`      | Browser context         |
| `Clipboard`    | Clipboard content       |
| `FileObserver` | File system activity    |
| `Custom(String)`| User-defined provider  |

---

## Provider States

| State  | Description                          |
| ------ | ------------------------------------ |
| `Active`   | Provider is running and collecting data |
| `Disabled` | Provider is stopped                    |
| `Paused`   | Provider is temporarily suspended      |
| `Error(String)` | Provider encountered an error   |

---

## Serialization

All events and types are serializable via `serde`:

```rust
use serde::{Serialize, Deserialize};

// Events serialize to JSON for storage, transmission, debugging
let json = serde_json::to_string(&event)?;
let deserialized: ObservationEvent = serde_json::from_str(&json)?;
```

Key serialization attributes:
- `ProviderConfig.settings` uses `#[serde(default)]` for backward compatibility
- `ProviderStatus.details` uses `#[serde(skip_serializing_if = "HashMap::is_empty", default)]`
- All `DateTime<Utc>` fields use ISO 8601 format

---

## Event Statistics

The `EventBus` tracks statistics for all events:

```rust
pub struct EventBusStats {
    pub total_events: u64,
    pub events_published: u64,
    pub events_consumed: u64,
    pub events_dropped: u64,
    pub by_provider: HashMap<ProviderType, u64>,
    pub by_event_type: HashMap<EventType, u64>,
}
```

Statistics support:
- Resetting counters
- Time-windowed aggregation (to be implemented)
- Export for monitoring dashboards

---

## ProviderStatus

A snapshot of a provider's current state:

```rust
pub struct ProviderStatus {
    pub name: String,
    pub provider_type: String,
    pub state: ProviderState,
    pub config: ProviderConfig,
    pub lifecycle: ProviderLifecycle,
    pub details: HashMap<String, serde_json::Value>,
}
```

`details` is provider-specific:
- **ActiveWindow:** `monitor_type`, `platform`
- **Terminal:** `active_sessions`, `recent_commands`, `platform`
- **Browser:** `browser_type`, `current_url`, `is_engineering`
- **Clipboard:** `text_length`, `looks_like_error`, `looks_like_stack_trace`
- **FileObserver:** `tracked_extensions`, `recent_files`
- **ScreenCapture:** `screens`, `capture_interval`, `last_capture_time`

---

## ProviderLifecycle

Tracks provider lifecycle:

```rust
pub struct ProviderLifecycle {
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub restart_count: u32,
}
```

- `restart_count` increments each time `start()` is called after a `stop()`
- `stopped_at` is cleared on restart

---

## Validation

The event model enforces:

- ✅ All required fields present
- ✅ Confidence clamped to [0.0, 1.0]
- ✅ All types serializable/deserializable
- ✅ Payload carries structured, typed data
- ✅ No AI reasoning or intent detection in event creation
- ✅ Events are immutable after creation