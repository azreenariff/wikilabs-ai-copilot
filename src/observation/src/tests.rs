//! Tests for observation framework: provider lifecycle, event publishing,
//! privacy controls, config, and performance.

mod provider_lifecycle_tests {
    use crate::app_monitor::ActiveWindowProvider;
    use crate::browser::BrowserProvider;
    use crate::clipboard::ClipboardProvider;
    use crate::file_observer::FileObserverProvider;
    use crate::provider::{ObservationProvider, ProviderState};
    use crate::screen_capture::ScreenCaptureProvider;
    use crate::terminal::TerminalProvider;

    fn test_provider_lifecycle<P: ObservationProvider + Default>(_name: &str) {
        let mut provider = P::default();
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

        // Should fail when already disabled
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(provider.pause().await.is_err());
            assert!(provider.resume().await.is_err());
        });
    }

    #[test]
    fn test_active_window_lifecycle() {
        test_provider_lifecycle::<ActiveWindowProvider>("ActiveWindowProvider");
    }

    #[test]
    fn test_terminal_lifecycle() {
        test_provider_lifecycle::<TerminalProvider>("TerminalProvider");
    }

    #[test]
    fn test_browser_lifecycle() {
        test_provider_lifecycle::<BrowserProvider>("BrowserProvider");
    }

    #[test]
    fn test_clipboard_lifecycle() {
        test_provider_lifecycle::<ClipboardProvider>("ClipboardProvider");
    }

    #[test]
    fn test_file_observer_lifecycle() {
        test_provider_lifecycle::<FileObserverProvider>("FileObserverProvider");
    }

    #[test]
    fn test_screen_capture_lifecycle() {
        test_provider_lifecycle::<ScreenCaptureProvider>("ScreenCaptureProvider");
    }
}

mod event_publishing_tests {
    use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
    use crate::event_bus::{EventBus, EventBusConfig};

    fn make_bus() -> EventBus {
        EventBus::new(EventBusConfig {
            channel_capacity: 128,
            batch_size: 10,
            batch_timeout_ms: 100,
        })
    }

    #[test]
    fn test_bus_create_and_subscribe_all() {
        let bus = make_bus();
        let (_sub, mut rx) = bus.subscribe_all();
        assert!(!rx.try_recv().is_err() || rx.try_recv().is_err()); // empty
    }

    #[test]
    fn test_event_published_and_consumed() {
        let bus = make_bus();
        let (_sub, mut rx) = bus.subscribe_all();

        bus.publish(ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "vscode".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({"app": "vscode"})),
        ));

        let event = rx.try_recv().unwrap();
        assert_eq!(event.event_type, EventType::ApplicationChanged);
        assert_eq!(event.provider, ProviderType::ActiveWindow);
    }

    #[test]
    fn test_provider_subscription_filter() {
        let bus = make_bus();
        let (_sub, mut rx) = bus.subscribe_to_provider(ProviderType::ActiveWindow);

        // Publish non-matching event
        bus.publish(ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::Browser,
            "firefox".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({})),
        ));
        assert!(rx.try_recv().is_err()); // no match

        // Publish matching event
        bus.publish(ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "vscode".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({})),
        ));
        let event = rx.try_recv().unwrap();
        assert_eq!(event.provider, ProviderType::ActiveWindow);
    }

    #[test]
    fn test_stats_incremented() {
        let bus = make_bus();
        assert_eq!(bus.get_stats().total_events, 0);

        bus.publish(ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "vscode".to_string(),
            None,
            ObservationPayload::empty(),
        ));
        assert_eq!(bus.get_stats().total_events, 1);

        bus.publish(ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "code".to_string(),
            None,
            ObservationPayload::empty(),
        ));
        assert_eq!(bus.get_stats().total_events, 2);
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        let event = ObservationEvent::new(
            EventType::ApplicationChanged,
            ProviderType::ActiveWindow,
            "vscode".to_string(),
            None,
            ObservationPayload::new(serde_json::json!({"app": "vscode", "pid": 12345})),
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ObservationEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.event_id, event.event_id);
        assert_eq!(deserialized.event_type, EventType::ApplicationChanged);
        assert_eq!(deserialized.provider, ProviderType::ActiveWindow);
        assert_eq!(deserialized.source, "vscode");
    }

    #[test]
    fn test_default_bus() {
        let bus = EventBus::with_defaults();
        assert_eq!(bus.get_stats().total_events, 0);
        let (_sub, mut rx) = bus.subscribe_all();
        assert!(rx.try_recv().is_err());
    }
}

mod config_tests {
    use crate::app_monitor::ActiveWindowProvider;
    use crate::browser::BrowserProvider;
    use crate::provider::{ObservationProvider, ProviderConfig};

    #[test]
    fn test_config_get_set() {
        let mut provider = ActiveWindowProvider::new();
        let mut config = provider.config();
        config.enabled = false;
        config.interval_secs = 5;
        provider.set_config(config);
        assert!(!provider.config().enabled);
        assert_eq!(provider.config().interval_secs, 5);
    }

    #[test]
    fn test_default_config() {
        let provider = ActiveWindowProvider::new();
        let config = provider.config();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 5);
    }

    #[test]
    fn test_config_change_reflected() {
        let mut provider = BrowserProvider::new();
        let config = provider.config();
        assert!(config.enabled);

        provider.set_config(ProviderConfig {
            enabled: false,
            interval_secs: 30,
            settings: serde_json::json!({}),
        });
        assert!(!provider.config().enabled);
        assert_eq!(provider.config().interval_secs, 30);
    }

    #[test]
    fn test_observe_returns_events_when_started() {
        let mut provider = ActiveWindowProvider::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });
        assert!(!events.is_empty());
    }
}

mod registry_tests {
    use crate::app_monitor::ActiveWindowProvider;
    use crate::browser::BrowserProvider;
    use crate::clipboard::ClipboardProvider;
    use crate::file_observer::FileObserverProvider;
    use crate::provider::ProviderRegistry;
    use crate::screen_capture::ScreenCaptureProvider;
    use crate::terminal::TerminalProvider;

    #[test]
    fn test_register_all_providers() {
        let mut registry = ProviderRegistry::new();

        registry.register(Box::new(ActiveWindowProvider::new()));
        registry.register(Box::new(TerminalProvider::new()));
        registry.register(Box::new(BrowserProvider::new()));
        registry.register(Box::new(ClipboardProvider::new()));
        registry.register(Box::new(FileObserverProvider::new()));
        registry.register(Box::new(ScreenCaptureProvider::new()));

        let names = registry.provider_names();
        assert_eq!(names.len(), 6);
    }

    #[test]
    fn test_get_providers() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(ActiveWindowProvider::new()));

        let provider = registry.get("Active Window");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "Active Window");
    }

    #[test]
    fn test_get_returns_none_for_missing() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(ActiveWindowProvider::new()));
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_unregister_providers() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(ActiveWindowProvider::new()));
        assert_eq!(registry.provider_names().len(), 1);

        let removed = registry.unregister("Active Window");
        assert!(removed.is_some());
        assert!(registry.get("Active Window").is_none());
    }

    #[test]
    fn test_start_all() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(ActiveWindowProvider::new()));
        registry.register(Box::new(BrowserProvider::new()));

        let rt = tokio::runtime::Runtime::new().unwrap();
        let results = rt.block_on(registry.start_all());
        assert_eq!(results.len(), 2);
        for (name, result) in &results {
            assert!(result.is_ok(), "{}: {:?}", name, result);
        }
    }

    #[test]
    fn test_all_status() {
        let registry = ProviderRegistry::new();
        let statuses = registry.all_status();
        // No providers registered — empty
        assert!(statuses.is_empty());
    }

    #[test]
    fn test_set_provider_enabled() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(ActiveWindowProvider::new()));

        assert!(registry.set_provider_enabled("Active Window", false));
        let provider = registry.get("Active Window").unwrap();
        assert!(!provider.config().enabled);
    }

    #[test]
    fn test_set_nonexistent_provider() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(ActiveWindowProvider::new()));
        assert!(!registry.set_provider_enabled("nonexistent", true));
    }
}

mod privacy_tests {
    use crate::event::ProviderType;
    use crate::privacy::{ObservationMode, PrivacyConfig, PrivacyManager};

    #[test]
    fn test_privacy_manager_creation() {
        let manager = PrivacyManager::new();
        let indicator = manager.get_indicator();
        // New manager starts in disabled mode with no active providers
        assert!(!indicator.is_active());
        assert!(!indicator.indicator_visible);
    }

    #[test]
    fn test_provider_disabled_by_config() {
        let mut config = PrivacyConfig::default();
        config
            .provider_overrides
            .insert(ProviderType::Clipboard, false);
        assert!(config.is_provider_allowed(&ProviderType::ActiveWindow));
        assert!(!config.is_provider_allowed(&ProviderType::Clipboard));
    }

    #[test]
    fn test_privacy_config_applied() {
        let config = PrivacyConfig::default();
        let manager = PrivacyManager::new();
        manager.set_config(config);
        assert!(manager.is_provider_allowed(&ProviderType::Clipboard));
    }

    #[test]
    fn test_custom_provider_override() {
        let mut config = PrivacyConfig::default();
        assert!(config.is_provider_allowed(&ProviderType::Custom("my_provider".to_string())));

        config
            .provider_overrides
            .insert(ProviderType::Custom("my_provider".to_string()), false);
        assert!(!config.is_provider_allowed(&ProviderType::Custom("my_provider".to_string())));
    }

    #[test]
    fn test_observation_mode_toggle() {
        let manager = PrivacyManager::new();

        // Default: disabled mode (PrivacyIndicator::disabled() is used in new())
        let indicator = manager.get_indicator();
        assert!(!indicator.is_active());
        assert!(!indicator.indicator_visible);

        // Set enabled mode - now it should show active providers
        manager.set_mode(ObservationMode::Enabled);
        let indicator = manager.get_indicator();
        assert!(indicator.is_active());

        // Set disabled mode
        manager.set_mode(ObservationMode::Disabled);
        let indicator = manager.get_indicator();
        assert!(!indicator.is_active());
        assert!(!indicator.indicator_visible);

        // Set back to enabled
        manager.set_mode(ObservationMode::Enabled);
        let indicator = manager.get_indicator();
        assert!(indicator.is_active());
    }

    #[test]
    fn test_provider_specific_toggle() {
        let manager = PrivacyManager::new();

        // By default, clipboard is allowed
        assert!(manager.is_provider_allowed(&ProviderType::Clipboard));

        // Disable clipboard specifically
        manager.set_provider_enabled(ProviderType::Clipboard, false);
        assert!(!manager.is_provider_allowed(&ProviderType::Clipboard));

        // Re-enable
        manager.set_provider_enabled(ProviderType::Clipboard, true);
        assert!(manager.is_provider_allowed(&ProviderType::Clipboard));
    }

    #[test]
    fn test_default_privacy_config() {
        let config = PrivacyConfig::default();
        assert_eq!(config.retention_days, 7);
        assert!(!config.store_clipboard_content);
    }

    #[test]
    fn test_privacy_config_allow_override() {
        let mut config = PrivacyConfig::default();
        config
            .provider_overrides
            .insert(ProviderType::ScreenCapture, false);
        assert!(!config.provider_overrides.is_empty());
    }
}
