//! Observation Framework — Provider Plugin Architecture
//!
//! Defines the `ObservationProvider` trait that all providers must implement.
//! Providers are independent, configurable, and individually enableable/disableable.
//!
//! This module does NOT implement any specific provider — it only defines the
//! interface that providers must implement.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::event::{ObservationEvent, ProviderType};

/// Configuration for a single observation provider.
/// Each provider can have its own configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Whether this provider is enabled.
    pub enabled: bool,
    /// Capture/polling interval in seconds (0 = manual trigger only).
    pub interval_secs: u64,
    /// Provider-specific configuration stored as JSON.
    #[serde(default)]
    pub settings: serde_json::Value,
}

impl ProviderConfig {
    pub fn new(enabled: bool, interval_secs: u64) -> Self {
        Self {
            enabled,
            interval_secs,
            settings: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn with_settings(mut self, settings: serde_json::Value) -> Self {
        self.settings = settings;
        self
    }

    pub fn as_map(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert("enabled".to_string(), serde_json::json!(self.enabled));
        map.insert(
            "interval_secs".to_string(),
            serde_json::json!(self.interval_secs),
        );
        map.insert("settings".to_string(), self.settings.clone());
        map
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 5,
            settings: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// Current state of an observation provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderState {
    /// Provider is running and collecting observations.
    Active,
    /// Provider is disabled.
    Disabled,
    /// Provider is paused (temporarily stopped).
    Paused,
    /// Provider has encountered an error.
    Error(String),
}

impl std::fmt::Display for ProviderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderState::Active => write!(f, "active"),
            ProviderState::Disabled => write!(f, "disabled"),
            ProviderState::Paused => write!(f, "paused"),
            ProviderState::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}

/// Lifecycle state for provider monitoring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderLifecycle {
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub restart_count: u32,
}

impl ProviderLifecycle {
    pub fn new() -> Self {
        Self {
            started_at: None,
            stopped_at: None,
            restart_count: 0,
        }
    }

    pub fn start(&mut self) {
        self.started_at = Some(Utc::now());
        self.stopped_at = None;
        self.restart_count += 1;
    }

    pub fn stop(&mut self) {
        self.stopped_at = Some(Utc::now());
    }
}

impl Default for ProviderLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

/// The core trait that all observation providers must implement.
///
/// This trait defines the contract for:
/// - Provider identification (name, type, description)
/// - Configuration management
/// - Lifecycle management (start/stop)
/// - State management (enabled/disabled/paused)
/// - Manual observation trigger
/// - Event publishing
///
/// Providers are independent and must not depend on other providers.
#[async_trait]
pub trait ObservationProvider: Send + Sync {
    /// Returns the provider's unique type identifier.
    fn provider_type(&self) -> ProviderType;

    /// Returns the provider's human-readable name.
    fn name(&self) -> &str;

    /// Returns the provider's description.
    fn description(&self) -> &str;

    /// Returns the current configuration.
    fn config(&self) -> ProviderConfig;

    /// Update the provider's configuration.
    fn set_config(&mut self, config: ProviderConfig);

    /// Returns the current state of the provider.
    fn state(&self) -> ProviderState;

    /// Start the provider (begin collecting observations).
    async fn start(&mut self) -> Result<(), String>;

    /// Stop the provider (stop collecting observations).
    async fn stop(&mut self) -> Result<(), String>;

    /// Pause the provider (temporarily stop collecting).
    async fn pause(&mut self) -> Result<(), String>;

    /// Resume the provider (restart collecting after pause).
    async fn resume(&mut self) -> Result<(), String>;

    /// Manually trigger an observation (for providers that normally poll).
    async fn observe(&self) -> Result<Vec<ObservationEvent>, String>;

    /// Get the current lifecycle state.
    fn lifecycle(&self) -> ProviderLifecycle;

    /// Get provider-specific status information.
    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }
}

/// Registry that manages all observation providers.
///
/// The registry handles:
/// - Provider registration/unregistration
/// - Global start/stop
/// - Per-provider enable/disable
/// - Provider discovery
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn ObservationProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a new provider.
    pub fn register(&mut self, provider: Box<dyn ObservationProvider>) {
        let name = provider.name().to_string();
        tracing::info!("Registered observation provider: {}", name);
        self.providers.insert(name, provider);
    }

    /// Unregister a provider by name.
    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn ObservationProvider>> {
        tracing::info!("Unregistering observation provider: {}", name);
        self.providers.remove(name)
    }

    /// Get a provider by name.
    pub fn get(&self, name: &str) -> Option<&dyn ObservationProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Get all registered provider names.
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get all registered providers.
    pub fn all_providers(&self) -> Vec<&dyn ObservationProvider> {
        self.providers.values().map(|p| p.as_ref()).collect()
    }

    /// Start all enabled providers.
    pub async fn start_all(&mut self) -> Vec<(String, Result<(), String>)> {
        let mut results = Vec::new();
        for (name, provider) in self.providers.iter_mut() {
            let config = provider.config();
            if config.enabled {
                let result = provider.start().await;
                results.push((name.clone(), result));
            }
        }
        results
    }

    /// Stop all providers.
    pub async fn stop_all(&mut self) -> Vec<(String, Result<(), String>)> {
        let mut results = Vec::new();
        for provider in self.providers.values_mut() {
            let result = provider.stop().await;
            results.push((provider.name().to_string(), result));
        }
        results
    }

    /// Enable or disable a specific provider.
    pub fn set_provider_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(provider) = self.providers.get_mut(name) {
            let mut config = provider.config();
            config.enabled = enabled;
            provider.set_config(config);
            true
        } else {
            false
        }
    }

    /// Get status of all providers.
    pub fn all_status(&self) -> Vec<ProviderStatus> {
        self.providers
            .values()
            .map(|p| ProviderStatus {
                name: p.name().to_string(),
                provider_type: p.provider_type().to_string(),
                state: p.state().clone(),
                config: p.config(),
                lifecycle: p.lifecycle().clone(),
                details: p.status_details(),
            })
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Status snapshot of a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub name: String,
    pub provider_type: String,
    pub state: ProviderState,
    pub config: ProviderConfig,
    pub lifecycle: ProviderLifecycle,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub details: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_defaults() {
        let config = ProviderConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 5);
        assert!(config.settings.is_object());
    }

    #[test]
    fn test_provider_config_new() {
        let config = ProviderConfig::new(false, 10);
        assert!(!config.enabled);
        assert_eq!(config.interval_secs, 10);
    }

    #[test]
    fn test_provider_config_with_settings() {
        let settings = serde_json::json!({"monitor_type": "all", "excluded_apps": ["chrome"]});
        let config = ProviderConfig::new(true, 5).with_settings(settings.clone());
        assert_eq!(config.settings, settings);
    }

    #[test]
    fn test_provider_state_display() {
        assert_eq!(ProviderState::Active.to_string(), "active");
        assert_eq!(ProviderState::Disabled.to_string(), "disabled");
        assert_eq!(ProviderState::Paused.to_string(), "paused");
        assert_eq!(
            ProviderState::Error("test error".to_string()).to_string(),
            "error: test error"
        );
    }

    #[test]
    fn test_provider_lifecycle() {
        let mut lifecycle = ProviderLifecycle::new();
        assert!(lifecycle.started_at.is_none());
        assert_eq!(lifecycle.restart_count, 0);

        lifecycle.start();
        assert!(lifecycle.started_at.is_some());
        assert_eq!(lifecycle.restart_count, 1);

        lifecycle.stop();
        assert!(lifecycle.stopped_at.is_some());

        lifecycle.start();
        assert_eq!(lifecycle.restart_count, 2);
        assert!(lifecycle.stopped_at.is_none());
    }

    #[test]
    fn test_provider_registry_new() {
        let registry = ProviderRegistry::new();
        assert_eq!(registry.provider_names().len(), 0);
    }

    #[test]
    fn test_provider_registry_get_unregister() {
        let registry = ProviderRegistry::new();
        assert!(registry.get("nonexistent").is_none());

        // Can't easily test registration without a concrete provider impl,
        // but the HashMap-based API is straightforward.
    }

    #[test]
    fn test_provider_status_serialization() {
        let config = ProviderConfig::new(true, 5);
        let status = ProviderStatus {
            name: "test_provider".to_string(),
            provider_type: "test".to_string(),
            state: ProviderState::Active,
            config,
            lifecycle: ProviderLifecycle::new(),
            details: HashMap::new(),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ProviderStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test_provider");
        assert_eq!(deserialized.state, ProviderState::Active);
    }
}
