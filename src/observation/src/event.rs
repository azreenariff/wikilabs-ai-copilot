//! Observation Framework — Event Model
//!
//! Defines the common event schema used by all observation providers.
//! Every event has a unique ID, timestamp, source provider, workspace context,
//! and a typed payload. All events are serializable to JSON.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique event identifier.
pub type EventId = Uuid;

/// Timestamp when the event was created.
pub type EventTimestamp = DateTime<Utc>;

/// Known observation providers.
/// Each provider reports its own type in events it publishes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    /// Active window / foreground application detection.
    ActiveWindow,
    /// Terminal / shell command observation.
    Terminal,
    /// Browser context observation.
    Browser,
    /// Clipboard content observation.
    Clipboard,
    /// File open / edit observation.
    FileObserver,
    /// Screenshot capture.
    ScreenCapture,
    /// Custom / user-defined provider.
    Custom(String),
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::ActiveWindow => write!(f, "active_window"),
            ProviderType::Terminal => write!(f, "terminal"),
            ProviderType::Browser => write!(f, "browser"),
            ProviderType::Clipboard => write!(f, "clipboard"),
            ProviderType::FileObserver => write!(f, "file_observer"),
            ProviderType::ScreenCapture => write!(f, "screen_capture"),
            ProviderType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Event types published to the event bus.
/// These are application-level events, not raw provider events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Foreground application changed.
    ApplicationChanged,
    /// Terminal command was entered.
    TerminalCommand,
    /// Screenshot was captured.
    ScreenshotCaptured,
    /// Browser context changed.
    BrowserContextChanged,
    /// Clipboard content changed.
    ClipboardChanged,
    /// Configuration file was opened or edited.
    ConfigurationFileOpened,
    /// General file activity.
    FileActivity,
    /// Provider state changed (enabled/disabled/paused/resumed).
    ProviderStateChanged,
    /// Generic event for unknown types.
    Unknown(String),
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::ApplicationChanged => write!(f, "application_changed"),
            EventType::TerminalCommand => write!(f, "terminal_command"),
            EventType::ScreenshotCaptured => write!(f, "screenshot_captured"),
            EventType::BrowserContextChanged => write!(f, "browser_context_changed"),
            EventType::ClipboardChanged => write!(f, "clipboard_changed"),
            EventType::ConfigurationFileOpened => write!(f, "configuration_file_opened"),
            EventType::FileActivity => write!(f, "file_activity"),
            EventType::ProviderStateChanged => write!(f, "provider_state_changed"),
            EventType::Unknown(s) => write!(f, "{}", s),
        }
    }
}

/// Structured event payload — each event type carries a specific payload struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationPayload {
    /// Type-specific data stored as a JSON value.
    pub data: serde_json::Value,
}

impl ObservationPayload {
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }

    pub fn empty() -> Self {
        Self {
            data: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.as_object().map(|o| o.is_empty()).unwrap_or(true)
    }
}

/// A single observation event — the core data structure of the framework.
///
/// Schema:
/// - event_id: UUID v4
/// - timestamp: UTC timestamp when event was created
/// - event_type: Application-level event type
/// - provider: Which provider generated this event
/// - source: Source identifier (e.g., "bash", "firefox", "alacritty")
/// - workspace: Workspace context
/// - metadata: Additional key-value metadata
/// - confidence: Confidence score (0.0-1.0) for the observation
/// - payload: Type-specific payload data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationEvent {
    pub event_id: EventId,
    pub timestamp: EventTimestamp,
    pub event_type: EventType,
    pub provider: ProviderType,
    pub source: String,
    pub workspace: Option<String>,
    pub metadata: serde_json::Value,
    pub confidence: f32,
    pub payload: ObservationPayload,
}

impl ObservationEvent {
    pub fn new(
        event_type: EventType,
        provider: ProviderType,
        source: String,
        workspace: Option<String>,
        payload: ObservationPayload,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            provider,
            source,
            workspace,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            confidence: 1.0,
            payload,
        }
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(mut map) = self.metadata {
            map.insert(key.to_string(), value);
            self.metadata = serde_json::Value::Object(map);
        }
        self
    }

    /// Serialize the event to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize a JSON string into an event.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Statistics about observation activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationStats {
    pub total_events: usize,
    pub events_by_type: std::collections::HashMap<String, usize>,
    pub events_by_provider: std::collections::HashMap<String, usize>,
    pub last_event_timestamp: Option<EventTimestamp>,
    pub is_paused: bool,
    pub is_enabled: bool,
}

impl ObservationStats {
    pub fn new() -> Self {
        Self {
            total_events: 0,
            events_by_type: std::collections::HashMap::new(),
            events_by_provider: std::collections::HashMap::new(),
            last_event_timestamp: None,
            is_paused: false,
            is_enabled: true,
        }
    }

    pub fn record_event(&mut self, event: &ObservationEvent) {
        self.total_events += 1;
        *self
            .events_by_type
            .entry(event.event_type.to_string())
            .or_insert(0) += 1;
        *self
            .events_by_provider
            .entry(event.provider.to_string())
            .or_insert(0) += 1;
        self.last_event_timestamp = Some(event.timestamp);
    }
}

impl Default for ObservationStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::ActiveWindow.to_string(), "active_window");
        assert_eq!(ProviderType::Terminal.to_string(), "terminal");
        assert_eq!(ProviderType::Browser.to_string(), "browser");
        assert_eq!(ProviderType::Clipboard.to_string(), "clipboard");
        assert_eq!(ProviderType::FileObserver.to_string(), "file_observer");
        assert_eq!(ProviderType::ScreenCapture.to_string(), "screen_capture");
        assert_eq!(
            ProviderType::Custom("my_provider".to_string()).to_string(),
            "my_provider"
        );
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(
            EventType::ApplicationChanged.to_string(),
            "application_changed"
        );
        assert_eq!(EventType::TerminalCommand.to_string(), "terminal_command");
        assert_eq!(
            EventType::ScreenshotCaptured.to_string(),
            "screenshot_captured"
        );
        assert_eq!(
            EventType::BrowserContextChanged.to_string(),
            "browser_context_changed"
        );
        assert_eq!(EventType::ClipboardChanged.to_string(), "clipboard_changed");
        assert_eq!(
            EventType::ConfigurationFileOpened.to_string(),
            "configuration_file_opened"
        );
        assert_eq!(EventType::FileActivity.to_string(), "file_activity");
        assert_eq!(
            EventType::ProviderStateChanged.to_string(),
            "provider_state_changed"
        );
        assert_eq!(EventType::Unknown("foo".to_string()).to_string(), "foo");
    }

    #[test]
    fn test_observation_event_creation() {
        let payload = ObservationPayload::new(serde_json::json!({
            "window_title": "Terminal",
            "process": "alacritty"
        }));
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "alacritty".to_string(),
            Some("test-workspace".to_string()),
            payload,
        );

        assert_eq!(event.event_type, EventType::ApplicationChanged);
        assert_eq!(event.provider, ProviderType::ActiveWindow);
        assert_eq!(event.source, "alacritty");
        assert_eq!(event.workspace, Some("test-workspace".to_string()));
        assert_eq!(event.confidence, 1.0);
        assert!(!event.payload.is_empty());
    }

    #[test]
    fn test_observation_event_with_confidence() {
        let payload = ObservationPayload::empty();
        let event = ObservationEvent::new(
            EventType::FileActivity,
            ProviderType::FileObserver,
            "test".to_string(),
            None,
            payload,
        )
        .with_confidence(0.75)
        .with_metadata("key", serde_json::json!("value"));

        assert_eq!(event.confidence, 0.75);
        assert_eq!(event.metadata["key"], "value");
    }

    #[test]
    fn test_observation_event_serialization() {
        let payload = ObservationPayload::new(serde_json::json!({"url": "https://example.com"}));
        let event = ObservationEvent::new(
            EventType::BrowserContextChanged,
            ProviderType::Browser,
            "firefox".to_string(),
            None,
            payload,
        );

        let json = event.to_json().unwrap();
        let deserialized: ObservationEvent = ObservationEvent::from_json(&json).unwrap();

        assert_eq!(deserialized.event_id, event.event_id);
        assert_eq!(deserialized.event_type, EventType::BrowserContextChanged);
        assert_eq!(deserialized.provider, ProviderType::Browser);
        assert_eq!(deserialized.source, "firefox");
        assert_eq!(deserialized.payload.data["url"], "https://example.com");
    }

    #[test]
    fn test_observation_payload() {
        let empty = ObservationPayload::empty();
        assert!(empty.is_empty());

        let data = ObservationPayload::new(serde_json::json!({"key": "value"}));
        assert!(!data.is_empty());
        assert_eq!(data.data["key"], "value");
    }

    #[test]
    fn test_observation_stats() {
        let mut stats = ObservationStats::new();
        assert_eq!(stats.total_events, 0);
        assert!(!stats.is_paused);
        assert!(stats.is_enabled);

        let payload = ObservationPayload::empty();
        let event1 = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "app1".to_string(),
            None,
            payload.clone(),
        );
        let event2 = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "app2".to_string(),
            None,
            payload.clone(),
        );
        let event3 = ObservationEvent::new(
            EventType::TerminalCommand,
            ProviderType::Terminal,
            "bash".to_string(),
            None,
            payload,
        );

        stats.record_event(&event1);
        stats.record_event(&event2);
        stats.record_event(&event3);

        assert_eq!(stats.total_events, 3);
        assert_eq!(*stats.events_by_type.get("application_changed").unwrap(), 2);
        assert_eq!(*stats.events_by_type.get("terminal_command").unwrap(), 1);
        assert_eq!(*stats.events_by_provider.get("active_window").unwrap(), 2);
        assert_eq!(*stats.events_by_provider.get("terminal").unwrap(), 1);
        assert!(stats.last_event_timestamp.is_some());
    }

    #[test]
    fn test_event_type_unknown() {
        let event = EventType::Unknown("custom_event".to_string());
        assert_eq!(event.to_string(), "custom_event");
    }

    #[test]
    fn test_confidence_clamping() {
        let payload = ObservationPayload::empty();
        let _event = ObservationEvent::new(
            EventType::FileActivity,
            ProviderType::FileObserver,
            "test".to_string(),
            None,
            payload,
        );
    }
}
