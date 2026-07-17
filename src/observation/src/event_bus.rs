//! Observation Framework — Event Bus
//!
//! Central pub/sub event bus for all observation events.
//! Providers publish events to the bus; downstream consumers
//! (intent engine, activity feed, logging) subscribe to receive them.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{bounded, Sender, Receiver, TrySendError};

use crate::event::{ObservationEvent, EventType, ProviderType, ObservationStats};

/// Callback type for event consumers.
pub type EventCallback = Box<dyn Fn(&ObservationEvent) + Send + Sync>;

/// Subscription handle — allows unsubscribing later.
pub struct Subscription {
    pub id: String,
}

/// Configuration for the event bus.
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// Maximum number of events to buffer per channel.
    pub channel_capacity: usize,
    /// Whether to batch events for efficient downstream delivery.
    pub batch_size: usize,
    /// Batch timeout in milliseconds.
    pub batch_timeout_ms: u64,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 1024,
            batch_size: 10,
            batch_timeout_ms: 100,
        }
    }
}

/// Central event bus.
///
/// All providers publish events here. Consumers subscribe to
/// receive events filtered by type, provider, or both.
pub struct EventBus {
    config: EventBusConfig,
    enabled: Arc<Mutex<bool>>,
    provider_enabled: Arc<Mutex<HashMap<ProviderType, bool>>>,
    stats: Arc<Mutex<ObservationStats>>,
    /// key=label, value=vec of senders (multiple subscribers per label)
    all_senders: Arc<Mutex<HashMap<String, Vec<Sender<ObservationEvent>>>>>,
    receivers: Arc<Mutex<Vec<Receiver<ObservationEvent>>>>,
}

impl EventBus {
    pub fn new(config: EventBusConfig) -> Self {
        let stats = Arc::new(Mutex::new(ObservationStats::new()));
        Self {
            config,
            enabled: Arc::new(Mutex::new(true)),
            provider_enabled: Arc::new(Mutex::new(HashMap::new())),
            stats,
            all_senders: Arc::new(Mutex::new(HashMap::new())),
            receivers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(EventBusConfig::default())
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut flag = self.enabled.lock().unwrap();
        *flag = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    pub fn set_provider_enabled(&self, provider: ProviderType, enabled: bool) {
        let mut map = self.provider_enabled.lock().unwrap();
        map.insert(provider, enabled);
    }

    pub fn is_provider_enabled(&self, provider: &ProviderType) -> bool {
        let map = self.provider_enabled.lock().unwrap();
        match map.get(provider) {
            Some(&enabled) => enabled,
            None => true,
        }
    }

    /// Publish an event. Sends to provider channel AND to "all" channel.
    pub fn publish(&self, event: ObservationEvent) -> Result<(), String> {
        if !self.is_enabled() {
            tracing::debug!("Event bus disabled, dropping event: {}", event.event_type);
            return Ok(());
        }

        if !self.is_provider_enabled(&event.provider) {
            tracing::debug!(
                "Provider {} is disabled, dropping event: {}",
                event.provider,
                event.event_type
            );
            return Ok(());
        }

        {
            let mut stats = self.stats.lock().unwrap();
            stats.record_event(&event);
        }

        let senders = self.all_senders.lock().unwrap();

        // Send to provider-specific channel(s)
        if let Some(channel_senders) = senders.get(&event.provider.to_string()) {
            for sender in channel_senders {
                match sender.try_send(event.clone()) {
                    Ok(()) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        tracing::warn!("Provider {} channel disconnected", event.provider);
                    }
                    Err(TrySendError::Full(_)) => {
                        tracing::warn!("Provider {} channel full", event.provider);
                    }
                }
            }
        }

        // Broadcast to "all" channel
        if let Some(all_senders) = senders.get(&"all".to_string()) {
            for sender in all_senders {
                match sender.try_send(event.clone()) {
                    Ok(()) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        tracing::warn!("All channel disconnected");
                    }
                    Err(TrySendError::Full(_)) => {
                        tracing::warn!("All channel full");
                    }
                }
            }
        }

        Ok(())
    }

    pub fn subscribe_to_provider(
        &self,
        provider: ProviderType,
    ) -> (Subscription, Receiver<ObservationEvent>) {
        let (tx, rx) = bounded::<ObservationEvent>(self.config.channel_capacity);
        let label = provider.to_string();
        let mut senders = self.all_senders.lock().unwrap();
        senders.entry(label).or_default().push(tx);
        let mut receivers = self.receivers.lock().unwrap();
        receivers.push(rx.clone());
        (Subscription { id: format!("sub_{}", provider) }, rx)
    }

    pub fn subscribe_to_event_type(
        &self,
        event_type: EventType,
    ) -> (Subscription, Receiver<ObservationEvent>) {
        let (tx, rx) = bounded::<ObservationEvent>(self.config.channel_capacity);
        let source_id = format!("evt_{}", event_type);
        let mut senders = self.all_senders.lock().unwrap();
        senders.entry(source_id).or_default().push(tx);
        let mut receivers = self.receivers.lock().unwrap();
        receivers.push(rx.clone());
        (Subscription { id: format!("sub_{}", event_type) }, rx)
    }

    pub fn subscribe_all(&self) -> (Subscription, Receiver<ObservationEvent>) {
        let (tx, rx) = bounded::<ObservationEvent>(self.config.channel_capacity);
        let mut senders = self.all_senders.lock().unwrap();
        senders.entry("all".to_string()).or_default().push(tx);
        let mut receivers = self.receivers.lock().unwrap();
        receivers.push(rx.clone());
        (Subscription { id: "sub_all".to_string() }, rx)
    }

    pub fn get_stats(&self) -> ObservationStats {
        let mut stats = self.stats.lock().unwrap();
        stats.is_paused = !self.is_enabled();
        stats.is_enabled = self.is_enabled();
        stats.clone()
    }

    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = ObservationStats::new();
    }

    pub fn active_providers(&self) -> Vec<ProviderType> {
        let stats = self.stats.lock().unwrap();
        stats
            .events_by_provider
            .keys()
            .filter_map(|name| {
                Some(match name.as_str() {
                    "active_window" => ProviderType::ActiveWindow,
                    "terminal" => ProviderType::Terminal,
                    "browser" => ProviderType::Browser,
                    "clipboard" => ProviderType::Clipboard,
                    "file_observer" => ProviderType::FileObserver,
                    "screen_capture" => ProviderType::ScreenCapture,
                    _ => ProviderType::Custom(name.clone()),
                })
            })
            .collect()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            enabled: Arc::clone(&self.enabled),
            provider_enabled: Arc::clone(&self.provider_enabled),
            stats: Arc::clone(&self.stats),
            all_senders: Arc::clone(&self.all_senders),
            receivers: Arc::clone(&self.receivers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::ObservationPayload;

    #[test]
    fn test_event_bus_creation() {
        let config = EventBusConfig::default();
        let bus = EventBus::new(config);
        assert!(bus.is_enabled());
    }

    #[test]
    fn test_event_bus_disable() {
        let config = EventBusConfig::default();
        let bus = EventBus::new(config);
        bus.set_enabled(false);
        assert!(!bus.is_enabled());
        bus.set_enabled(true);
        assert!(bus.is_enabled());
    }

    #[test]
    fn test_publish_and_consume() {
        let bus = EventBus::with_defaults();
        let (_sub, mut rx) = bus.subscribe_all();

        let payload = ObservationPayload::empty();
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "vscode".to_string(),
            None,
            payload,
        );

        bus.publish(event).unwrap();
        let received = rx.try_recv().unwrap();
        assert_eq!(received.provider, ProviderType::ActiveWindow);
        assert_eq!(received.source, "vscode");
    }

    #[test]
    fn test_publish_to_disabled_provider() {
        let bus = EventBus::with_defaults();
        let (_sub, mut rx) = bus.subscribe_all();

        bus.set_provider_enabled(ProviderType::ActiveWindow, false);

        let payload = ObservationPayload::empty();
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "vscode".to_string(),
            None,
            payload,
        );

        bus.publish(event).unwrap();
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_provider_enable_disable() {
        let bus = EventBus::with_defaults();
        assert!(bus.is_provider_enabled(&ProviderType::ActiveWindow));

        bus.set_provider_enabled(ProviderType::ActiveWindow, false);
        assert!(!bus.is_provider_enabled(&ProviderType::ActiveWindow));

        bus.set_provider_enabled(ProviderType::ActiveWindow, true);
        assert!(bus.is_provider_enabled(&ProviderType::ActiveWindow));
    }

    #[test]
    fn test_multiple_subscribers() {
        let bus = EventBus::with_defaults();

        let (_sub1, mut rx1) = bus.subscribe_to_provider(ProviderType::ActiveWindow);
        let (_sub2, mut rx2) = bus.subscribe_to_provider(ProviderType::ActiveWindow);

        let payload = ObservationPayload::empty();
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "test".to_string(),
            None,
            payload,
        );

        assert!(bus.publish(event).is_ok());

        let e1 = rx1.try_recv().unwrap();
        let e2 = rx2.try_recv().unwrap();
        assert_eq!(e1.source, "test");
        assert_eq!(e2.source, "test");
    }

    #[test]
    fn test_statistics_tracking() {
        let bus = EventBus::with_defaults();
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
            payload.clone(),
        );

        bus.publish(event1).unwrap();
        bus.publish(event2).unwrap();
        bus.publish(event3).unwrap();

        let stats = bus.get_stats();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.events_by_type.get("application_changed"), Some(&2));
        assert_eq!(stats.events_by_type.get("terminal_command"), Some(&1));
        assert_eq!(stats.events_by_provider.get("active_window"), Some(&2));
        assert_eq!(stats.events_by_provider.get("terminal"), Some(&1));
    }

    #[test]
    fn test_subscribe_all() {
        let bus = EventBus::with_defaults();
        let (_sub, mut rx) = bus.subscribe_all();

        let payload = ObservationPayload::empty();

        let event1 = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "app1".to_string(),
            None,
            payload.clone(),
        );
        let event2 = ObservationEvent::new(
            EventType::TerminalCommand,
            ProviderType::Terminal,
            "bash".to_string(),
            None,
            payload.clone(),
        );

        bus.publish(event1).unwrap();
        bus.publish(event2).unwrap();

        let e1 = rx.try_recv().unwrap();
        let e2 = rx.try_recv().unwrap();
        assert_eq!(e1.provider, ProviderType::ActiveWindow);
        assert_eq!(e2.provider, ProviderType::Terminal);
    }

    #[test]
    fn test_stats_is_paused() {
        let bus = EventBus::with_defaults();
        let stats = bus.get_stats();
        assert!(!stats.is_paused);

        bus.set_enabled(false);
        let stats = bus.get_stats();
        assert!(stats.is_paused);
    }

    #[test]
    fn test_reset_stats() {
        let bus = EventBus::with_defaults();
        let payload = ObservationPayload::empty();
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "test".to_string(),
            None,
            payload,
        );
        bus.publish(event).unwrap();

        let stats = bus.get_stats();
        assert_eq!(stats.total_events, 1);

        bus.reset_stats();
        let stats = bus.get_stats();
        assert_eq!(stats.total_events, 0);
    }

    #[test]
    fn test_active_providers() {
        let bus = EventBus::with_defaults();
        assert!(bus.active_providers().is_empty());

        let payload = ObservationPayload::empty();
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "test".to_string(),
            None,
            payload.clone(),
        );
        bus.publish(event).unwrap();

        let providers = bus.active_providers();
        assert!(!providers.is_empty());
        assert!(providers.contains(&ProviderType::ActiveWindow));
    }

    #[test]
    fn test_event_bus_clone() {
        let bus1 = EventBus::with_defaults();
        let bus2 = bus1.clone();

        bus2.set_enabled(false);
        assert!(!bus1.is_enabled());

        bus1.set_enabled(true);
        bus2.set_enabled(false);
        assert!(!bus1.is_enabled());
    }
}